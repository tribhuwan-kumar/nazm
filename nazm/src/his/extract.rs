use url::Url;
use std::path::Path;
use serde_json::Value;
use tracing::{error, info, debug};

use crate::AppState;
use crate::aria2::types::{
	Kind,
    Aria2Res,
    GidStatus,
    ItemMetaData,
	UriCacheState,
    TorrentMagnetMeta
};

use super::header::{
	UNTITLED,
	URI_FETCHING_NAME,
	URI_NO_INTERNET_NAME,
    uri_cache,
    spawn_uri_probe,
};

pub struct Extraction;

impl Extraction {
    pub fn extract(
        state: &AppState,
        json: &Value, /* Res of `aria2.tellStatus` */
        gid: &str
    ) -> ItemMetaData {
        let info: Aria2Res = match serde_json::from_value(json.clone()) {
            Ok(v) => v,
            Err(e) => {
                /* If empty response, return skeleton, something diasater happened */
                error!("Failed to parse aria2 status for gid: {:?}, error: {:?}", gid, e);
                return Self::skeleton(gid);
            }
        };
        // debug!("Extracted info from aria2 response: {:?}", info);
        let file_paths: Vec<String> = info.files.iter()
            .map(|f| f.path.clone())
            .filter(|p| !p.is_empty())
            .collect();
        let files_json = serde_json::to_string(&file_paths).ok();

        let mut source_uri: Option<String> = None;
        if let Some(file) = info.files.first() {
            // Find first non-empty uri
            source_uri = file.uris.iter()
                .find(|u| !u.uri.is_empty())
                .map(|u| u.uri.clone());
        }

        let is_magnet = source_uri.as_deref().map_or(false, |uri| uri.starts_with("magnet:"));
        let is_torrent = info.bittorrent.is_some() || info.info_hash.is_some() || is_magnet;
        // debug!("Is it a torrent: {:?}", is_torrent);
        // debug!("Files json: {:?}", files_json);

		let is_seeder: Option<bool> = info.seeder
			.as_deref()
			.and_then(|s| match s {
				"true" => Some(true),
				"false" => Some(false),
				_ => None,
			});

		let is_verify_pending: Option<bool> = info.verify_integrity_pending
			.as_deref()
			.and_then(|s| match s {
				"true" => Some(true),
				"false" => Some(false),
				_ => None,
			});

        let status = match info.status.as_str() {
            "active" => {
                if is_torrent &&
                    info.completed_length == info.total_length &&
                    info.total_length != "0"
                    || is_seeder.is_some_and(|s| s) {
                    GidStatus::Seeding
                } else {
                    GidStatus::Active
                }
            },
            "waiting"  => GidStatus::Waiting,
            "paused"   => GidStatus::Paused,
            "error"    => GidStatus::Error,
            "complete" => GidStatus::Complete,
            "removed"  => {
                let exists = file_paths.iter().any(|p| Path::new(p).exists());
                if exists { GidStatus::Stopped } else { GidStatus::Removed }
            },
            _ => GidStatus::Stopped,
        };

        let mut cache_state = None;
        if let Some(uri) = &source_uri {
            if let Ok(map) = uri_cache().lock() {
                cache_state = map.get(gid).cloned();
            }
            if cache_state.is_none() {
                spawn_uri_probe(state.clone(), gid.to_string(), uri.clone());
                cache_state = Some(UriCacheState::Pending);
            }
        }

        let name = Self::resolve_name(&info, cache_state.as_ref());

        // Determine the kind
        let kind = if is_torrent {
             Kind::Torrent
        } else if source_uri.is_some() {
             Kind::Uri
        } else {
             Kind::Unknown
        };

		let is_resume = if is_torrent {
			Some(true)
		} else {
			// For URIs, can't know synchronously.
			// Returning `None`, so sql query ignore this field
			// And keep whatever the database already has.
			None
		};

        let torrent = info.bittorrent.as_ref().map(|bt| TorrentMagnetMeta {
            announce_list: bt.announce_list.clone().unwrap_or_default(),
            comment: bt.comment.clone(),
            creation_date: bt.creation_date,
            mode: bt.mode.clone(),
            name: bt.info.as_ref().and_then(|i| i.name.clone()),
        });

        // debug!("Item kind: {:?}", kind);

        if torrent.is_some() {
            // debug!("Torrent info: {:?}", torrent);
        }

        if source_uri.is_some() {
            // debug!("Extracting metadata for uri {:?}", source_uri);
        }

        ItemMetaData {
            gid: gid.to_string(),
            name: Some(name),
            kind,
            status,
            dir: Some(info.dir),
			files: files_json,
            total_length: Some(info.total_length),
            completed_length: Some(info.completed_length),
            upload_length: Some(info.upload_length),
            source_uri,
			info_hash: info.info_hash,
			seeder: is_seeder,
			content_hash: None,
			error_code: info.error_code.and_then(|c| c.parse().ok()),
			error_message: info.error_message,
			torrent: torrent.map(sqlx::types::Json),
			options: None,                                             // Getting saved in proxy handler
			is_resume_supported: is_resume,
			connections: info.connections,
			num_pieces: info.num_pieces,
			num_seeders: info.num_seeders,
			piece_length: info.piece_length,
			verified_length: info.verified_length,
			verify_integrity_pending: is_verify_pending,
            created_at: None,
            completed_at: None,
            user_id: "".to_string(),
        }
    }

    /*
      * Parse uri (magnet dn / url path)
      * And Uri req name
    */
    fn resolve_name(info: &Aria2Res,  cache_state: Option<&UriCacheState>) -> String {
        /* BitTorrent name metadata exists */
        if let Some(name) = info.bittorrent.as_ref()
            .and_then(|bt| bt.info.as_ref())
            .and_then(|bt_info| bt_info.name.as_ref())
            .filter(|n| !n.is_empty())
        {
            return name.clone();
        }

        /* File system path, file created on disk */
        if let Some(file) = info.files.first() {
            if !file.path.is_empty() {
                if let Some(n) = Path::new(&file.path).file_name().and_then(|s| s.to_str()) {
                    return n.to_string();
                }
            }
        }

		if let Some(state) = cache_state {
			match state {
				UriCacheState::Pending => return URI_FETCHING_NAME.to_string(),
				UriCacheState::NoInternet => return URI_NO_INTERNET_NAME.to_string(),
			}
		}

        /* Parse uri (magnet dn / url path) */
        for file in &info.files {
            for aria_uri in &file.uris {
                if let Ok(u) = Url::parse(&aria_uri.uri) {
                    // Magnet link; try 'dn' parameter
                    if u.scheme() == "magnet" {
                        for (k, v) in u.query_pairs() {
                            if k == "dn" && !v.trim().is_empty() {
                                return v.into_owned();
                            }
                        }
                    }
                    // http/ftp; try path segment
					else {
						return u.to_string();
					}
                }
            }
        }

        info!("Failed to resolve name, using: {:?}", UNTITLED);
        UNTITLED.to_string()
    }

    fn skeleton(gid: &str) -> ItemMetaData {
        ItemMetaData {
			gid: gid.to_string(),
            name: Some(UNTITLED.to_string()),
			kind: Kind::Unknown,
			status: GidStatus::Waiting,
			dir: None,
			files: None,
			total_length: Some("0".into()),
			completed_length: Some("0".into()),
			upload_length: Some("0".into()),
			source_uri: None,
			info_hash: None,
			seeder: None,
			content_hash: None,
			error_code: None,
			error_message: None,
			torrent: None,
			options: None,
			is_resume_supported: Some(false),
			connections: Some("0".into()),
			num_pieces: Some("0".into()),
			num_seeders: Some("0".into()),
			piece_length: Some("0".into()),
			verified_length: Some("0".into()),
			verify_integrity_pending: Some(false),
            created_at: None,
            completed_at: None,
            user_id: "".to_string(),
        }
    }
}
