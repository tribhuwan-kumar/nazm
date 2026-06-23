use reqwest::Client;
use std::time::Duration;
use tracing::{debug, info, warn};
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::{HashMap, HashSet};

use crate::{
    AppState,
    aria2::types::UriCacheState,
    his::service::HistoryService,
};

const HTTP_CLIENT_TIMEOUT_DURATION: u64 = 20;

pub const UNTITLED: &str = "<Untitled>";
pub const URI_FETCHING_NAME: &str = "Fetching file name...";
pub const URI_NO_INTERNET_NAME: &str = "Waiting for network...";

static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();
static URI_CACHE: OnceLock<Arc<Mutex<HashMap<String, UriCacheState>>>> = OnceLock::new();
static PENDING_PROBES: OnceLock<Arc<Mutex<HashSet<String>>>> = OnceLock::new();

pub fn uri_cache() -> &'static Arc<Mutex<HashMap<String, UriCacheState>>> {
    URI_CACHE.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

pub fn pending_probes() -> &'static Arc<Mutex<HashSet<String>>> {
    PENDING_PROBES.get_or_init(|| Arc::new(Mutex::new(HashSet::new())))
}

pub fn http_client() -> &'static Client {
    HTTP_CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(HTTP_CLIENT_TIMEOUT_DURATION))
            .build()
            .unwrap_or_default()
    })
}

pub fn spawn_uri_probe(state: AppState, gid: String, uri: String) {
    // Concurrency prevention & state init
    if let Ok(mut pending) = pending_probes().lock() {
        if !pending.insert(gid.clone()) {
            return;
        }
    }

    if let Ok(mut cache) = uri_cache().lock() {
        cache.insert(gid.clone(), UriCacheState::Pending);
    }

    tokio::spawn(async move {
        // Cleanup guard
        struct ProbeCleanup { gid: String }
        impl Drop for ProbeCleanup {
            fn drop(&mut self) {
                if let Ok(mut pending) = pending_probes().lock() {
                    pending.remove(&self.gid);
                }
            }
        }
        let _cleanup = ProbeCleanup { gid: gid.clone() };

        if let Ok(Some(row)) = sqlx::query!("SELECT name FROM download_history WHERE gid = ?", gid)
            .fetch_optional(&state.db).await
        {
            if let Some(name) = row.name {
                if !name.is_empty()
                    && name != UNTITLED
                    && name != URI_FETCHING_NAME
                    && name != URI_NO_INTERNET_NAME
                {
                    // Valid name exists in db!!
                    // Ensure memory is clear and abort probe
                    if let Ok(mut cache) = uri_cache().lock() {
                        cache.remove(&gid);
                    }
                    return;
                }
            }
        }

        let client = http_client().clone();

        loop {
            match client.get("http://clients3.google.com/generate_204").send().await {
                Ok(res) if res.status() == 204 => break,
                _ => {
                    warn!("No internet connection. Waiting for network for GID: {}", gid);
                    if let Ok(mut cache) = uri_cache().lock() {
                        cache.insert(gid.clone(), UriCacheState::NoInternet);
                    }

                    let _ = sqlx::query!(
                        "UPDATE download_history
						SET name = ?
						WHERE gid = ?",
                        URI_NO_INTERNET_NAME, gid
                    ).execute(&state.db).await;

                    HistoryService::handle_event(&state, gid.clone(), None).await;

                    tokio::time::sleep(Duration::from_secs(7)).await;
                }
            }
        }

        // Internet Restored -> Switch state back to Pending
        if let Ok(mut cache) = uri_cache().lock() {
            cache.insert(gid.clone(), UriCacheState::Pending);
        }

        let _ = sqlx::query!( "UPDATE download_history SET name = ? WHERE gid = ?",
			URI_FETCHING_NAME, gid
        ).execute(&state.db).await;

        HistoryService::handle_event(&state, gid.clone(), None).await;

        // Target probing
        let max_retries = 3;
        let mut filename = None;
        let mut resumable = false;
        let mut success = false;

        for attempt in 1..=max_retries {
            let mut head_success = false;
			info!("URI being probed: {:?}", &uri);
            match client.head(&uri).send().await {
                Ok(res) => {
                    head_success = true;
                    if let Some(val) = res.headers().get(reqwest::header::ACCEPT_RANGES) {
                        resumable = val.to_str().map(|v| v.eq_ignore_ascii_case("bytes")).unwrap_or(false);
                    }
                    if let Some(val) = res.headers().get(reqwest::header::CONTENT_DISPOSITION) {
                        if let Ok(s) = val.to_str() {
                            filename = extract_filename(s);
							info!("Extracted file name from head: {:?}", filename);
                        }
                    }
                }
                Err(e) => {
                    warn!("HEAD request failed for {:?}: {}. Falling back to GET...", uri, e);
                }
            }

            if !head_success || !resumable || filename.is_none() {
                match client.get(&uri).header(reqwest::header::RANGE, "bytes=0-0").send().await {
                    Ok(res_get) => {
                        if !resumable {
                            resumable = res_get.status() == reqwest::StatusCode::PARTIAL_CONTENT;
                        }
                        if filename.is_none() {
                            if let Some(val) = res_get.headers().get(reqwest::header::CONTENT_DISPOSITION) {
                                if let Ok(s) = val.to_str() {
                                    filename = extract_filename(s);
                                }
                            }
                        }
                        success = true;
                    }
                    Err(e) => warn!("GET probe failed for {:?} (Attempt {}/{}): {}", uri, attempt, max_retries, e),
                }
            } else {
                success = true;
            }

            if success { break; }
            if attempt < max_retries { tokio::time::sleep(Duration::from_secs(5)).await; }
        }

        // Resolution & persistence
        if success {
			let final_name = filename.unwrap_or_else(|| {
				if !uri.is_empty() {
					uri.to_string()
				} else {
					UNTITLED.to_string()
				}
			});

            let final_name = urlencoding::decode(&final_name)
                .map(|d| d.into_owned())
                .unwrap_or(final_name);

			info!("Uri final name: {:?}", final_name);

            // Update Synchronous Cache
            if let Ok(mut cache) = uri_cache().lock() {
                cache.remove(&gid);
            }

            let _ = sqlx::query!(
                "UPDATE download_history SET name = ?, is_resume_supported = ? WHERE gid = ?",
                final_name, resumable, gid
            ).execute(&state.db).await;

            HistoryService::handle_event(&state, gid.clone(), None).await;

        } else {
            // Target is unreachable
            let err_msg = format!("URI server resolution failed or server is down");
            warn!("Failing GID {:?}: {:?}", gid, err_msg);

            // Update synchronous cache with empty resolved state so `resolve_name` falls back to the raw URL
            if let Ok(mut cache) = uri_cache().lock() {
                cache.remove(&gid);
            }

            let _ = sqlx::query!(
                "UPDATE download_history
				SET
                    status = 'error',
                    error_code = 1,
                    error_message = ?
                WHERE gid = ?",
                err_msg, gid
            ).execute(&state.db).await;

            HistoryService::handle_event(&state, gid.clone(), None).await;
        }
    });
}

fn extract_filename(header_val: &str) -> Option<String> {
    if let Some(idx) = header_val.find("filename=\"") {
        let start = idx + 10;
        if let Some(end) = header_val[start..].find('"') {
            return Some(header_val[start..start+end].to_string());
        }
    }
    if let Some(idx) = header_val.find("filename=") {
        let start = idx + 9;
        let end = header_val[start..].find(';').unwrap_or(header_val[start..].len());
        let clean = header_val[start..start+end].trim_matches(|c| c == '"' || c == '\'');
        return Some(clean.to_string());
    }
    None
}
