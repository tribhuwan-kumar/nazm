use axum::{
    extract::Json,
    http::StatusCode,
    extract::{State, Query},
    response::{IntoResponse},
};
use std::path::Path;
use serde_json::{json, Value};
use std::collections::HashMap;
use super::types::{
    Kind,
    GidReq,
    MoveReq,
    StopReq,
    PauseReq,
    RetryReq,
    GidStatus,
    ResumeReq,
    AddUriReq,
    ShutdownReq,
    OpenFileReq,
    ItemMetaData,
    Aria2Options,
    RemoveHisReq,
    SetOptionReq,
    AddTorrentReq,
    PaginationQuery,
    TorrentMagnetMeta,
    BatchAddTorrentReq,
    SetGlobalOptionReq,
	SetCmdOptionsReq,
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use tracing::{info, debug, error, warn};
use crate::{
    utils,
    app::AppState,
    his::service::HistoryService,
    auth::types::User as AuthenticatedUser,
};


/// Add all most all types of URIs
pub async fn add_uris(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<AddUriReq>,
) -> impl IntoResponse {
    /*
      * Convert list of URIs into a Batch Multicall
      * addUri([a,b,c]), do [addUri(a), addUri(b), addUri(c)]
    */
    let mut calls: Vec<Value> = Vec::new();
    let mut generated_gids: Vec<String> = Vec::new();

    for uri in payload.uris.clone() {
        let gid = utils::generate_gid();
        /* inject the custom GID into the Aria2 options */
        let mut opt = payload.options.clone().unwrap_or_default();
        opt.insert("gid".to_string(), json!(gid));

        let kind = match url::Url::parse(&uri) {
            Ok(u) => match u.scheme().to_ascii_lowercase().as_str() {
                "magnet" => Kind::Torrent,
                "http" | "https" | "ftp" | "sftp" | "file" => Kind::Uri,
                _ => Kind::Unknown,
            }
            Err(_) => Kind::Unknown,
        };

        if let Err(e) = sqlx::query!(
            r#"
                INSERT INTO download_history (gid, user_id, kind, status, source_uri)
                VALUES (?, ?, ?, 'waiting', ?)
            "#,
            gid,
            user.user_id,
            kind,
			uri,
        ).execute(&state.db).await {
            error!("Failed to register record for GID {:?}: {:?}", gid, e);
        }

        generated_gids.push(gid.clone());

        calls.push(json!({
            "methodName": "aria2.addUri",
            "params": [ [uri], opt ]       /* addUri expects an array of mirrors, so wrap single uri in [] */
        }));
    }

    debug!("Add uri calls: {:?}", calls);

    match state.aria2.call("system.multicall", vec![json!(calls)]).await {
        Ok(gids_resp) => {
            let mut results = Vec::new();
            if let Some(arr) = gids_resp.as_array() {
                for (i, res) in arr.iter().enumerate() {
                    let expected_gid = &generated_gids[i];

                    if let Some(returned_gid) = res.as_array().and_then(|g| g.first()).and_then(|v| v.as_str()) {
                        info!("Successfully added uri with gid: {}", returned_gid);
                        let state_clone = state.clone();
                        let gid_clone = returned_gid.to_string();

                        tokio::spawn(async move {
                            if let Ok(opts_val) = state_clone.aria2.call("getOption", vec![json!(gid_clone)]).await {
                                let opts_str = serde_json::to_string(&opts_val).unwrap_or_default();
								let result = sqlx::query!(
									"UPDATE download_history SET options = ? WHERE gid = ?",
									opts_str,
									gid_clone
								)
								.execute(&state_clone.db)
								.await;

								if let Err(e) = result {
									error!("Failed to update options in database for GID {:?}: {:?}", gid_clone, e);
								}
                            }
                            HistoryService::refresh_gid(&state_clone, gid_clone).await;
                        });
                        results.push(json!(
                                {
                                    "index": i,
                                    "gid": returned_gid
                                })
                            );
                    }

                    else if let Some(err_obj) = res.as_object() {
                        let borrow_j = &json!(0);
                        let code = err_obj.get("code").unwrap_or(borrow_j);
                        let msg = err_obj.get("message").and_then(|v| v.as_str()).unwrap_or("Unknown error");

                        warn!("Aria2 rejected uri addition for gid '{}', code {:?}, msg {:?}", expected_gid, code, msg);
                        let _ = sqlx::query!(
                            "DELETE FROM download_history WHERE gid = ?",
                            expected_gid
                        ).execute(&state.db).await;

                        results.push(json!({
							"index": i,
							"code": code,
							"error": msg,
                        }));
                    }
                    else {
                        results.push(json!({
							"index": i,
							"error": "Aria2 returned invalid response format"
						}));
                    }
                }
            }
            (StatusCode::OK, Json(json!({ "results": results })))
        },
        Err(e) => {
            error!("Failed to execute multicall: {}", e);
            for gid in generated_gids {
                let _ = sqlx::query!(
                    "DELETE FROM download_history WHERE gid = ?",
                    gid
				).execute(&state.db).await;
            }
            (StatusCode::BAD_GATEWAY, Json(json!({ "error": e })))
        }
    }
}


/// Add a torrent file
/// Ref: https://aria2.github.io/manual/en/html/aria2c.html#aria2.addTorrent
#[deprecated(note = "Please use `add_torrents` instead")]
pub async fn add_torrent(
    State(state): State<AppState>,
    Json(payload): Json<AddTorrentReq>,
) -> impl IntoResponse {
    let options = Value::Object(payload.options.unwrap_or_default());
    /*
      *  The empty array is for web seeding URIs,
      *  They're usually empty
    */
    // Params: [base64_torrent, [], options]
    let params = vec![
        json!(payload.torrent),
        json!([]),
        options
    ];

    match state.aria2.call("addTorrent", params).await {
        Ok(gid) => {
            info!("`add_torrent` Successfully executed multicall: {}", gid);
            (StatusCode::OK, Json(json!({ "gid": gid })))
        },
        Err(e) => {
            error!("`add_torrent` Failed to executed multicall: {}", e);
            (StatusCode::BAD_GATEWAY, Json(json!({ "error": e })))
        }
    }
}


/// Specially for adding torrents files
pub async fn add_torrents(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<BatchAddTorrentReq>,
) -> impl IntoResponse {
    if payload.torrents.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "No torrents provided" })));
    }
    /*
      * Build the multicall params
      * Transform list of inputs into a list of aria2 method calls
    */
    let mut calls: Vec<Value> = Vec::new();
    let mut generated_gids: Vec<String> = Vec::new();
	let mut saved_file_paths: Vec<std::path::PathBuf> = Vec::new();

    for item in payload.torrents.into_iter() {
        let gid = utils::generate_gid();
        let mut opt = item.options.unwrap_or_else(|| serde_json::json!({}));

        if let Some(map) = opt.as_object_mut() {
            map.insert("gid".to_string(), serde_json::json!(gid));
        }

		let torrent_bytes = match STANDARD.decode(&item.torrent) {
			Ok(bytes) => bytes,
			Err(e) => {
				error!("Failed to decode Base64 torrent string: {:?}", e);
				for path in saved_file_paths {
					let _ = tokio::fs::remove_file(path).await;
				}
				return (StatusCode::BAD_REQUEST, Json(json!(
							{ "error": format!("Invalid Base64 payload: {}", e) })));
			}
		};

		let file_name = format!("{}-{}.torrent", user.user_id, gid);
		let torrent_path = state.config.data_dir.join("torrents").join(&file_name);

		if let Some(parent_dir) = torrent_path.parent() {
			if let Err(e) = tokio::fs::create_dir_all(parent_dir).await {
				error!("Failed to create torrents directory: {:?}", e);
				for path in saved_file_paths {
					let _ = tokio::fs::remove_file(path).await;
				}
				return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(
							{ "error": "Failed to initialize server storage directories" })));
			}
		}

		if let Err(e) = tokio::fs::write(&torrent_path, torrent_bytes).await {
			error!("Failed to write torrent file to disk: {:?}", e);
			for path in saved_file_paths {
				let _ = tokio::fs::remove_file(path).await;
			}
			return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(
						{ "error": "Failed to persist torrent data to server storage" })));
		}

		saved_file_paths.push(torrent_path.clone());

        if let Err(e) = sqlx::query!(
            r#"
                INSERT INTO download_history (gid, user_id, kind, status)
                VALUES (?, ?, 'torrent', 'waiting')
            "#,
            gid,
            user.user_id
        ).execute(&state.db).await{
            error!("Failed to register record for GID {:?}: {:?}", gid, e);
			for path in saved_file_paths {
				let _ = tokio::fs::remove_file(path).await;
			}
			return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(
						{ "error": "Database tracking registration failed" })));
		};

        generated_gids.push(gid.clone());

        calls.push(json!({
            "methodName": "aria2.addTorrent",
            "params": [
                item.torrent,
                [],                 /* Webseeding URIs, keeping it empty till now */
                opt
            ]
        }));
    }

    /*
      * Call system.multicall
      * Params for multicall is an array containing the array of calls: [ [call1, call2] ]
    */
    match state.aria2.call("system.multicall", vec![json!(calls)]).await {
        Ok(gids_resp) => {
            let mut results = Vec::new();

            if let Some(arr) = gids_resp.as_array() {
                for (i, res) in arr.iter().enumerate() {

                    let expected_gid = &generated_gids[i];
					let file_to_keep = &saved_file_paths[i];

                    if let Some(returned_gid) = res.as_array().and_then(|g| g.first()).and_then(|v| v.as_str()) {
                        let state_clone = state.clone();
                        let gid_clone = returned_gid.to_string();

                        tokio::spawn(async move {
                            if let Ok(opts_val) = state_clone.aria2.call("getOption", vec![json!(gid_clone)]).await {
                                let opts_str = serde_json::to_string(&opts_val).unwrap_or_default();
								let result = sqlx::query!(
									"UPDATE download_history SET options = ? WHERE gid = ?",
									opts_str,
									gid_clone
								)
								.execute(&state_clone.db)
								.await;

								if let Err(e) = result {
									error!("Failed to update options in database for GID {:?}: {:?}", gid_clone, e);
								}
                            }
                            HistoryService::refresh_gid(&state_clone, gid_clone).await;
                        });
                        results.push(json!({
                            "index": i,
                            "gid": returned_gid,
                        }));
                    }
                    else if let Some(err_obj) = res.as_object() {
                        let borrow_j = &json!(0);
                        let code = err_obj.get("code").unwrap_or(borrow_j);
                        let msg = err_obj.get("message").and_then(|v| v.as_str()).unwrap_or("Unknown error");
                        let _ = sqlx::query!(
                            "DELETE FROM download_history WHERE gid = ?",
                            expected_gid
                        ).execute(&state.db).await;

						// Also delete its companion file from the filesystem
                        let _ = tokio::fs::remove_file(file_to_keep).await;

                        results.push(json!({
                            "index": i,
                            "code": code,
                            "error": msg,
                        }));
                    }
                    else {
                        results.push(json!({
							"index": i,
							"error": "Aria2 returned invalid response format"
						}));
                    }
                }
            }
            (StatusCode::OK, Json(json!({ "results": results })))
        }
        Err(e) => {
            error!("Failed to batch torrents: {}", e);
            for (i, gid) in generated_gids.iter().enumerate() {
                let _ = sqlx::query!(
                    "DELETE FROM download_history WHERE gid = ?",
                    gid
                ).execute(&state.db).await;
				let _ = tokio::fs::remove_file(&saved_file_paths[i]).await;
            }
            (StatusCode::BAD_GATEWAY, Json(json!({ "error": e })))
        }
    }
}


/// ref: https://aria2.github.io/manual/en/html/aria2c.html#aria2.pause
/// aria2.pause
/// aria2.pauseAll
/// aria2.forcePause
/// aria2.forcePauseAll
pub async fn pause(
    State(state): State<AppState>,
    Json(payload): Json<PauseReq>,
) -> impl IntoResponse {
    // `*all` method takes no gid
    let mut params = Vec::new();

    let method = if payload.force_pause_all.unwrap_or(false) {
        "forcePauseAll"
    } else if payload.pause_all.unwrap_or(false) {
        "pauseAll"
    } else if payload.force_pause.unwrap_or(false) {
        "forcePause"
    } else {
        "pause"
    };

    let needs_gid = !payload.pause_all.unwrap_or(false) && !payload.force_pause_all.unwrap_or(false);

    if needs_gid {
        match &payload.gid {
            Some(gid) => params.push(serde_json::json!(gid)),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "GID is required for this operation" }))
                ).into_response();
            }
        }
    }

    match state.aria2.call(method, params).await {
        Ok(_) => {
			HistoryService::handle_event(&state, payload.gid.clone().unwrap().to_string(), Some("paused")).await;
            info!("Successfully paused GID: {:?}, method: {:?}", payload.gid, method);
            (StatusCode::OK, Json(serde_json::json!({ "status": "paused", "method": method }))).into_response()
        },
        Err(e) => {
            error!("`{}` failed: {}", method, e);
            (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e, "success": false }))).into_response()
        },
    }
}


/// Ref: https://aria2.github.io/manual/en/html/aria2c.html#aria2.unpause
/// `aria2.unpause`
/// `aria2.unpauseAll`
/// Paused to wating
pub async fn resume(
    State(state): State<AppState>,
    Json(payload): Json<ResumeReq>,
) -> impl IntoResponse {
    let mut params = Vec::new();
    let method = if payload.resume_all.unwrap_or(false) {
        "unpauseAll"
    } else {
        "unpause"
    };

    let needs_gid = !payload.resume_all.unwrap_or(false);
    if needs_gid {
        match &payload.gid {
            Some(gid) => params.push(serde_json::json!(gid)),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "gid is required for this operation" }))
                ).into_response();
            }
        }
    }

    match state.aria2.call(method, params).await {
        Ok(_) => {
			HistoryService::handle_event(&state, payload.gid.clone().unwrap().to_string(), Some("waiting")).await;
            info!("Successfully resumed GID: {:?}, method: {:?}", payload.gid, method);
            (StatusCode::OK, Json(serde_json::json!({ "status": "resumed", "method": method }))).into_response()
        },
        Err(e) => {
            error!("`{}` failed: {}", method, e);
            (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e, }))).into_response()
        },
    }
}


/// Ref: https://aria2.github.io/manual/en/html/aria2c.html#aria2.remove
/// It'll remove the dl from the queue & free the memory
/// Remove only from downloading queue, but don't delete the `file-allocation`
pub async fn stop(
    State(state): State<AppState>,
    Json(payload): Json<StopReq>,
) -> impl IntoResponse {
    let method = if payload.force_stop.unwrap_or(false) {
        "aria2.forceRemove"
    } else {
        "aria2.remove"
    };
    let methods = vec![
        json!({ "methodName": method, "params": [payload.gid.clone()] }),
        json!({ "methodName": "aria2.removeDownloadResult", "params": [payload.gid.clone()] }),
    ];

    HistoryService::handle_event(&state, payload.gid.clone(), Some("stopped")).await;

    match state.aria2.call("system.multicall", vec![json!(methods)]).await {
        Ok(_) => {
            info!("Successfully stopped gid: {:?}", payload.gid);
            (StatusCode::OK, Json(json!({ "status": "stopped" })))
        },
        Err(e) => {
            error!("Failed to remove gid {:?}: {:?}", payload.gid, e);
            (StatusCode::BAD_GATEWAY, Json(json!({ "error": e.to_string() })))
        },
    }
}


/// Get details for a gid
pub async fn details(
    State(state): State<AppState>,
    Json(payload): Json<GidReq>,
) -> impl IntoResponse {
    let gid = payload.gid.clone();
    let history_item = match sqlx::query_as!(
        ItemMetaData,
        r#"
			SELECT
				gid,
				name,
				kind as "kind: Kind",
				status as "status: GidStatus",
				dir,
				files,
				total_length,
				completed_length,
				upload_length,
				source_uri,
				info_hash,
				seeder,
				content_hash,
				error_code,
				error_message,
				torrent as "torrent: sqlx::types::Json<TorrentMagnetMeta>",
				options,
				is_resume_supported,
				connections,
				num_pieces,
				num_seeders,
				piece_length,
				verified_length,
				verify_integrity_pending,
				created_at as "created_at!",
				completed_at as "completed_at",
				user_id
			FROM download_history
			WHERE gid = ?
        "#,
        gid
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(item) => item,
        Err(e) => {
            warn!("Failed to fetch download history for gid {}: {}", gid, e);
            None
        }
    };


    let is_torrent = history_item
        .as_ref()
        .map(|item| matches!(item.kind.clone(), Kind::Torrent)
            || item.torrent.is_some()
            || item.info_hash.is_some())
        .unwrap_or(false);

    let mut methods = vec![
        json!({ "methodName": "aria2.tellStatus", "params": [gid.clone()] }),
        json!({ "methodName": "aria2.getFiles", "params": [gid.clone()] }),
        json!({ "methodName": "aria2.getOption", "params": [gid.clone()] }),
    ];

    if is_torrent {
        methods.push(json!({ "methodName": "aria2.getPeers", "params": [gid.clone()] }));
    } else {
        /*
          * Specific to `uri`
        */
        methods.push(json!({ "methodName": "aria2.getUris", "params": [gid.clone()] }));
        methods.push(json!({ "methodName": "aria2.getServers", "params": [gid.clone()] }));
    }

    let params = vec![json!(methods)];

    match state.aria2.call("system.multicall", params).await {
        Ok(res) => {
            let is_empty = res.as_array().map(|arr| arr.is_empty()).unwrap_or(true);
            if is_empty {
                if let Some(item) = history_item {
                    return (
                        StatusCode::OK,
                        Json(json!({
                            "source": "database",
                            "data": item
                        })),
                    );
                }

                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": format!("GID {} not found", gid)
                    })),
                );
            }

            (
                StatusCode::OK,
                Json(json!({
                    "source": "aria2",
                    "data": res,
                    "history": history_item
                })),
            )
        }
        Err(e) => {
            if let Some(item) = history_item {
                warn!("aria2 details lookup failed for gid {:?}: {:?}", gid, e);
                (
                    StatusCode::OK,
                    Json(json!({
                        "source": "database",
                        "data": item,
                        "warning": e.to_string()
                    })),
                )
            } else {
                error!("Failed to fetch details for gid {:?}: {:?}", gid, e);
                (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({
                        "error": e.to_string()
                    })),
                )
            }
        }
    }
}

/// Retry a failed download on status `stopped`, `removed`, `error`
/// If server `is_resume_supported`, resume the download
/// If not download again with
/// `allow-overwrite` & `continue=false` for uri
/// `check-integrity=false` for torrents and magnets
pub async fn retry(
    State(state): State<AppState>,
    Json(payload): Json<RetryReq>,
) -> impl IntoResponse {
    let gid = payload.gid.clone();

    let record = sqlx::query!(
        r#"
			SELECT
				dir,
				files,
				options,
				info_hash,
				source_uri,
				total_length,
				is_resume_supported,
				kind as "kind: Kind",
				status as "status: GidStatus",
				torrent as "torrent: sqlx::types::Json<TorrentMagnetMeta>"
			FROM download_history
			WHERE gid = ?
        "#,
        gid
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let rec = match record {
        Some(r) => r,
        None => {
            return (StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("GID {} not found", gid) })))
                .into_response();
        }
    };

    if rec.status != GidStatus::Error
		|| rec.status != GidStatus::Stopped
		|| rec.status != GidStatus::Stopped {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Only GID with 'stopped', 'removed' and 'error' status can be retried",
                "message": format!("GID {:?} status is {:?}", gid, rec.status)
            })),
        ).into_response();
    }

    let resource_to_add = if let Some(uri) = rec.source_uri.clone() {
        uri
    } else if let Some(hash) = rec.info_hash.as_deref() {
        format!("magnet:?xt=urn:btih:{}", hash)
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Can't retry, no source URI or info hash available" })),
        ).into_response();
    };

    let is_torrent = matches!(rec.kind, Kind::Torrent)
        || rec.torrent.is_some()
        || rec.info_hash.is_some();

    let should_resume = rec.is_resume_supported.unwrap_or(false);

	// clear from aria2 memory
    if !should_resume {
        let _ = state.aria2.call("forceRemove", vec![json!(gid.clone())]).await;
        let _ = state.aria2.call("removeDownloadResult", vec![json!(gid.clone())]) .await;
    }

    let mut options = serde_json::Map::new();

    if let Some(dir) = rec.dir.clone() {
        options.insert("dir".to_string(), json!(dir));
    }

    if should_resume {
        options.insert("continue".to_string(), json!(true));
        if is_torrent {
            options.insert("check-integrity".to_string(), json!(true));
        }
    } else {
        options.insert("allow-overwrite".to_string(), json!(true));
        options.insert("continue".to_string(), json!(false));
        options.insert("check-integrity".to_string(), json!(false));
    }

    let params = vec![json!([resource_to_add]), json!(options)];

    match state.aria2.call("addUri", params).await {
        Ok(new_gid_val) => {
            let new_gid = new_gid_val.as_str().unwrap_or_default();

            let _ = sqlx::query!(
                r#"
					UPDATE download_history
						SET gid = ?,
							error_code = NULL,
							error_message = NULL,
							updated_at = CURRENT_TIMESTAMP
					WHERE gid = ?
                "#,
                new_gid,
                gid
            )
            .execute(&state.db)
            .await;

            (StatusCode::OK, Json(json!({ "status": "success", "newGid": new_gid }))) .into_response()
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": e })),
        ).into_response(),
    }
}


/// Ref: https://aria2.github.io/manual/en/html/aria2c.html#aria2.saveSession
/// Try to save session
pub async fn save_session(
    State(state): State<AppState>,
) -> impl IntoResponse {

}


/// Ref: https://aria2.github.io/manual/en/html/aria2c.html#aria2.changeUri
pub async fn change_uri(
    State(state): State<AppState>,
    Json(payload): Json<MoveReq>,
) -> impl IntoResponse {

}


/// Ref: https://aria2.github.io/manual/en/html/aria2c.html#aria2.shutdown
/// aria2.forceShutdown
pub async fn shutdown(
    State(state): State<AppState>,
    Json(payload): Json<ShutdownReq>,
) -> impl IntoResponse {

}


/// Ref: https://aria2.github.io/manual/en/html/aria2c.html#aria2.getSessionInfo
/// Ref: https://aria2.github.io/manual/en/html/aria2c.html#aria2.getVersion
/// Combine session info and version info
pub async fn get_aria2_info(
    State(state): State<AppState>,
) -> impl IntoResponse {

}


/// Fetches the options snapshot from the local database instead of Aria2,
/// ensuring availability even for `stopped` `removed` downloads.
/// Ref: https://aria2.github.io/manual/en/html/aria2c.html#aria2.getOption
pub async fn get_option(
    State(state): State<AppState>,
    Json(payload): Json<GidReq>,
) -> impl IntoResponse {
    let row = match sqlx::query!(
        "SELECT options FROM download_history WHERE gid = ?",
        payload.gid
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("No download record found for GID '{}'", payload.gid) })),
            );
        }
        Err(e) => {
            error!("Database error fetching options for GID '{}': {:?}", payload.gid, e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal server database error" })),
            );
        }
    };

    // Options column is present but null, fall back to an empty json
    let options_json: serde_json::Value = match row.options {
        Some(opts_str) if !opts_str.is_empty() => {
            serde_json::from_str(&opts_str).unwrap_or_else(|_| serde_json::json!({}))
        }
        _ => serde_json::json!({}),
    };

    (StatusCode::OK, Json(serde_json::json!({ "data": options_json })))
}



/// Ref: https://aria2.github.io/manual/en/html/aria2c.html#aria2.changeOption
pub async fn set_option(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(payload): Json<SetOptionReq>,
) -> impl IntoResponse {
    let params = vec![
        serde_json::json!(payload.gid),
        serde_json::json!(payload.options),
    ];

    match state.aria2.call("changeOption", params).await {
        Ok(_) => {
            let existing = sqlx::query!(
                    "SELECT options
                    FROM download_history
                    WHERE gid = ?",
                    payload.gid
                ).fetch_optional(&state.db)
                .await
                .ok()
                .flatten()
                .and_then(|r| r.options)
                .unwrap_or_else(|| "{}".to_string());

            let mut current_opt: Aria2Options
                = serde_json::from_str(&existing).unwrap_or(Aria2Options(serde_json::Map::new()));

            current_opt.merge(payload.options);
            let final_json = serde_json::to_string(&current_opt).unwrap();

            let _ = sqlx::query!(
                "UPDATE download_history
                SET options = ?,
                updated_at = CURRENT_TIMESTAMP
                WHERE gid = ?",
                final_json,
                payload.gid
            ).execute(&state.db).await;

            (StatusCode::OK, Json(serde_json::json!({ "status": "optionsUpdated"})))
        }
        Err(e) => {
            error!("Failed to change options for GID {}: {}", payload.gid, e);
            (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}


/// Ref: https://aria2.github.io/manual/en/html/aria2c.html#aria2.getGlobalOption
pub async fn get_global_option(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.aria2.call("getGlobalOption", vec![]).await {
        Ok(options) => (StatusCode::OK,  Json(serde_json::json!({ "data": options }))),
        Err(e) => {
            error!("Failed to fetch global options: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}


/// Ref: https://aria2.github.io/manual/en/html/aria2c.html#aria2.changeGlobalOption
pub async fn set_global_option(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<SetGlobalOptionReq>,
) -> impl IntoResponse {
    let params = vec![serde_json::json!(payload.options)];

    match state.aria2.call("changeGlobalOption", params).await {
        Ok(_) => {
            let existing = sqlx::query!(
                "SELECT value FROM settings WHERE key = 'aria2_global_options' AND user_id = ?",
                user.user_id
            )
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
            .map(|r| r.value)
            .unwrap_or_else(|| "{}".to_string());

            let mut current_opt: Aria2Options = serde_json::from_str(&existing)
                .unwrap_or(Aria2Options(serde_json::Map::new()));

            current_opt.merge(payload.options);
            let final_json = serde_json::to_string(&current_opt).unwrap();

            let _ = sqlx::query!(
                r#"
                    INSERT INTO settings (
                        key,
                        value,
                        user_id,
                        updated_at
                    )
                    VALUES (
                        'aria2_global_options',
                        ?,
                        ?,
                        CURRENT_TIMESTAMP
                    )
                    ON CONFLICT(key, user_id) DO UPDATE
                    SET value = excluded.value, updated_at = CURRENT_TIMESTAMP
                "#,
                final_json,
                user.user_id
            )
            .execute(&state.db)
            .await;

            (StatusCode::OK, Json(serde_json::json!({ "status": "globalOptionsUpdated" })))
        },
        Err(e) => {
            error!("Failed to set global options: {}", e);
            (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

/// Those options that can't be set via aria2 rpc
pub async fn set_cmd_option(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<SetCmdOptionsReq>,
) -> impl IntoResponse {
    let existing = sqlx::query!(
        "SELECT value FROM settings WHERE key = 'aria2_cmd_options' AND user_id = ?",
        user.user_id
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .map(|r| r.value)
    .unwrap_or_else(|| "{}".to_string());

    let mut existing_cmd: HashMap<String, String> = serde_json::from_str(&existing)
		.unwrap_or_default();

    // `Aria2Options` into safe cli strings
    let safe_new_options = payload.options.to_string_map();

    // Delta modifications
    for (key, value) in safe_new_options {
        if value.trim().is_empty() {
            existing_cmd.remove(&key);
        } else {
            existing_cmd.insert(key, value);
        }
    }

    if let Err(err) = state.daemon.restart(&state.config, &existing_cmd).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "status": "error", "error": format!("Daemon launch failed: {}", err) })),
        );
    }

    // Record mutations safely
    let final_json = serde_json::to_string(&existing_cmd).unwrap();
    let _ = sqlx::query!(
        r#"
            INSERT INTO settings (
				key,
				value,
				user_id,
				updated_at
			)
            VALUES (
				'aria2_cmd_options',
				?,
				?,
				CURRENT_TIMESTAMP
			)
            ON CONFLICT(key, user_id) DO UPDATE
            SET value = excluded.value, updated_at = CURRENT_TIMESTAMP
        "#,
        final_json,
        user.user_id
    )
    .execute(&state.db)
    .await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "daemonRestarted",
            "message": "Aria2 daemon restarted with updated arguments."
        })),
    )
}


/// use `xdg-open` and platform specific utility to open the file
pub async fn open(
    State(state): State<AppState>,
    Json(payload): Json<OpenFileReq>,
) -> impl IntoResponse {

}


pub async fn purge_results(State(state): State<AppState>) -> impl IntoResponse {
    match state.aria2.call("purgeDownloadResult", vec![]).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "purged" }))),
        Err(e) => (StatusCode::BAD_GATEWAY, Json(json!({ "error": e }))),
    }
}


pub async fn move_position(
    State(state): State<AppState>,
    Json(payload): Json<MoveReq>,
) -> impl IntoResponse {
    let params = vec![
        json!(payload.gid),
        json!(payload.pos),
        json!(payload.how)
    ];

    state.aria2.call("changePosition", params)
        .await
        .inspect(|res| info!("Successfully executed multicall: {:?}", res))
        .inspect_err(|e| info!("failed to add uri: {:?}", e))
        .map(|results| (StatusCode::OK, Json(json!({ "newPosition": results }))))
        .unwrap_or_else(|e| (StatusCode::BAD_GATEWAY, Json(json!({ "error": e.to_string() }))))
}


pub async fn get_history(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(params): Query<PaginationQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(15);
    let offset = (page - 1) * limit;

    let total_items = sqlx::query!(
        "SELECT COUNT(*) as count FROM download_history WHERE user_id = ?",
        user.user_id
    )
    .fetch_one(&state.db)
    .await
    .map(|row| row.count)
    .unwrap_or(0);

    let total_pages = if total_items == 0 {
        0
    } else {
        (total_items as f64 / limit as f64).ceil() as u32
    };

    let has_more = page < total_pages;

    let history = sqlx::query_as!(
        ItemMetaData,
        r#"
			SELECT
				gid,
				name,
				kind as "kind: Kind",
				status as "status: GidStatus",
				dir,
				files,
				total_length,
				completed_length,
				upload_length,
				source_uri,
				info_hash,
				seeder,
				content_hash,
				error_code,
				error_message,
				torrent as "torrent: sqlx::types::Json<TorrentMagnetMeta>",
				options,
				is_resume_supported,
				connections,
				num_pieces,
				num_seeders,
				piece_length,
				verified_length,
				verify_integrity_pending,
				created_at as "created_at!",
				completed_at as "completed_at",
				user_id
			FROM download_history
			WHERE user_id = ?
			ORDER BY created_at DESC
			LIMIT ? OFFSET ?
        "#,
        user.user_id,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(json!({
        "data": history,
        "meta": {
            "currentPage": page,
            "perPage": limit,
            "totalItems": total_items,
            "totalPages": total_pages,
            "hasMore": has_more
        }
    }))
}


/// Remove dl along with deleting file
pub async fn delete(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<RemoveHisReq>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    for gid in payload.gids {
        /*
          * First free from aria2
          * Kinda inefficient, later check the status then do according to it
        */
        let _ = state.aria2.call("forcePause", vec![json!(gid)]).await;
        let _ = state.aria2.call("forceRemove", vec![json!(gid)]).await;
        let _ = state.aria2.call("removeDownloadResult", vec![json!(gid)]).await;

        if payload.delete_file {
            let record = sqlx::query!(
                "SELECT name, files, dir FROM download_history
                WHERE gid = ? AND user_id = ?",
                gid, user.user_id
            )
            .fetch_optional(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

            debug!("Found gid to delete: {:?}", record);

            if let Some(rec) = record {
                let download_dir = rec.dir.as_deref().map(Path::new);
                if let Some(name) = rec.name.as_ref() {
                    let name = name.to_string();
                    if let Some(files_str) = rec.files {
                        let file_paths: Vec<String> = serde_json::from_str(&files_str).unwrap_or_default();
                        for path in file_paths {
                            let path = Path::new(&path);
                            debug!("Removing file: {:?}", path);
                            if !path.exists() {
                                continue;
                            }
                            if path.is_dir() {
                                let _ = std::fs::remove_dir_all(&path);
                            } else {
                                let _ = std::fs::remove_file(&path);
                            }
                            if let Some(root) = download_dir {
                                let aria_name = format!("{}.aria2", name);
                                let aria2_path = root.join(Path::new(&aria_name));
                                let _ = std::fs::remove_file(aria2_path);
                                let mut current = path.parent();
                                while let Some(dir) = current {
                                    if dir == root {
                                        break;
                                    }

                                    if !dir.exists() || !dir.is_dir() {
                                        break;
                                    }

                                    let is_empty = std::fs::read_dir(dir)
                                        .map(|mut entries| entries.next().is_none())
                                        .unwrap_or(false);

                                    if !is_empty {
                                        break;
                                    }

                                    let _ = std::fs::remove_dir(dir);
                                    current = dir.parent();
                                }
                            }
                        }
                    }
                } else {
                    debug!("Skipping file deletion for gid {:?}: name is NULL", gid);
                }
            } else {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": format!("No download history found from GID {}", gid) })),
                ));
            }
        }

        /* Ensure `user_id` matches history */
        let res = sqlx::query!(
            "DELETE FROM download_history WHERE gid = ? AND user_id = ?",
            gid, user.user_id
        )
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

        if res.rows_affected() == 0 {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": format!("Download history record not found from GID {}", gid)
                })),
            ));
        }

        info!("Deleted gid: {:?}", gid);
    }

    Ok(Json(json!({ "status": "deleted" })))
}

