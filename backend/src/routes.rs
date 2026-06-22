use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web::http::header::CONTENT_TYPE;
use std::time::Instant;
use futures_util::StreamExt;
use sanitize_filename::sanitize;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::models::{
    app_state::AppState,
    genre::{DetectGenreRequest, DetectGenreResponse},
    metadata::{MetadataConfig, UpdateMetadataConfigRequest},
    metadata_suggestion::MetadataSuggestion,
    music::{
        BulkAddToPlaylistByRegexRequest, BulkRenameByRegexRequest,
        CreateMusicFileRequest, CutAudioRequest, MusicQueryParams, UpdateMusicFileRequest,
    },
    auto_download::UpdateAutoDownloadConfigRequest,
    playlist::{
        CreatePlaylistRequest, PlaylistTrackRequest, ReorderPlaylistTracksRequest, UpdatePlaylistRequest,
    },
    stream::{CreateStreamRequest, UpdateStreamRequest},
    youtube::{
        CreateYoutubePlaylistRequest, UpdateYoutubePlaylistRequest, YoutubeDownloadRequest,
        YoutubeDownloadResponse,
    },
};
use crate::services::auto_download_service;
use crate::services::backfill_manager;
use crate::services::reprocess_manager;
use crate::services::sync_manager;
use crate::services::discogs_service::DiscogsService;
use crate::services::{
    artist_service, audio_edit_service, bpm_service, file_sync_service, genre_cache_service, genre_detection, genre_label_service,
    internet_stream_service, music_service, playlist_service, playlist_export_service,
    yt_download_service, youtube_playlist_service,
};
use crate::yt_downloader::{self, DownloadOptions};
use urlencoding::encode as url_encode;

// Health check endpoint
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "healthy"}))
}

// DB-aware health check with timing and logging
pub async fn db_health_check(state: web::Data<AppState>) -> HttpResponse {
    let start = Instant::now();
    let db = &state.db;

    // Execute a lightweight DB query to check connectivity and measure latency
    let result = sqlx::query("SELECT 1").fetch_one(&db.pool).await;
    let elapsed_ms = start.elapsed().as_millis();

    match result {
        Ok(_) => {
            if elapsed_ms > 500 {
                log::warn!("DB health check slow: {}ms", elapsed_ms);
            } else {
                log::info!("DB health check ok: {}ms", elapsed_ms);
            }
            HttpResponse::Ok().json(serde_json::json!({
                "status": "ok",
                "db": "up",
                "response_ms": elapsed_ms
            }))
        }
        Err(e) => {
            log::error!("DB health check FAILED after {}ms: {}", elapsed_ms, e);
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "degraded",
                "db": "down",
                "error": e.to_string(),
                "response_ms": elapsed_ms
            }))
        }
    }
}

// Playlist routes
pub async fn get_playlists(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match playlist_service::get_all_playlists(db).await {
        Ok(playlists) => HttpResponse::Ok().json(playlists),
        Err(e) => {
            log::error!("Error fetching playlists: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn create_playlist(
    state: web::Data<AppState>,
    req: web::Json<CreatePlaylistRequest>,
) -> HttpResponse {
    let db = &state.db;
    match playlist_service::create_playlist(db, &req.name, req.description.clone()).await {
        Ok(playlist) => {
            // Notify clients about new playlist
            let _ = crate::services::cache_service::notify_change(
                state.get_ref(),
                "playlist_created",
                serde_json::to_value(&playlist).unwrap_or(serde_json::Value::Null)
            ).await;
            HttpResponse::Created().json(playlist)
        },
        Err(e) => {
            log::error!("Error creating playlist: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_playlist(state: web::Data<AppState>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;
    match playlist_service::get_playlist_with_items(db, id).await {
        Ok(Some(playlist)) => HttpResponse::Ok().json(playlist),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            log::error!("Error fetching playlist: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn delete_playlist(state: web::Data<AppState>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;
    match playlist_service::delete_playlist(db, id).await {
        Ok(_) => {
            // Notify clients about deleted playlist
            let payload = serde_json::json!({ "id": id });
            let _ = crate::services::cache_service::notify_change(
                state.get_ref(),
                "playlist_deleted",
                payload
            ).await;
            HttpResponse::NoContent().finish()
        },
        Err(e) => {
            log::error!("Error deleting playlist: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_playlist_handler(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    payload: web::Json<UpdatePlaylistRequest>,
) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;
    match playlist_service::update_playlist(db, id, payload.into_inner()).await {
        Ok(Some(playlist)) => {
            // Notify clients about updated playlist
            let _ = crate::services::cache_service::notify_change(
                state.get_ref(),
                "playlist_updated",
                serde_json::to_value(&playlist).unwrap_or(serde_json::Value::Null)
            ).await;
            HttpResponse::Ok().json(playlist)
        },
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            log::error!("Error updating playlist {}: {}", id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn add_playlist_track(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    payload: web::Json<PlaylistTrackRequest>,
) -> HttpResponse {
    let playlist_id = path.into_inner();
    let db = &state.db;
    match playlist_service::add_track_to_playlist(db, playlist_id, payload.into_inner()).await {
        Ok(_) => {
            // Notify clients that a playlist has changed (track count/items)
            let payload = serde_json::json!({ "id": playlist_id });
            let _ = crate::services::cache_service::notify_change(
                state.get_ref(),
                "playlist_items_updated",
                payload
            ).await;
            HttpResponse::Created().finish()
        },
        Err(e) => {
            log::error!("Error adding track to playlist {}: {}", playlist_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn remove_playlist_track(
    state: web::Data<AppState>,
    path: web::Path<(Uuid, Uuid)>,
) -> HttpResponse {
    let (playlist_id, track_id) = path.into_inner();
    let db = &state.db;
    match playlist_service::remove_track_from_playlist(db, playlist_id, track_id).await {
        Ok(_) => {
            // Notify clients that a playlist has changed
            let payload = serde_json::json!({ "id": playlist_id });
            let _ = crate::services::cache_service::notify_change(
                state.get_ref(),
                "playlist_items_updated",
                payload
            ).await;
            HttpResponse::NoContent().finish()
        },
        Err(e) => {
            log::error!("Error removing track from playlist {}: {}", playlist_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn export_playlist_zip(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;
    match playlist_export_service::export_playlist_zip(db, id, false).await {
        Ok((temp_file, filename)) => {
            let path = temp_file.path().to_path_buf();
            match NamedFile::open(path) {
                Ok(named_file) => named_file
                    .set_content_disposition(actix_web::http::header::ContentDisposition {
                        disposition: actix_web::http::header::DispositionType::Attachment,
                        parameters: vec![actix_web::http::header::DispositionParam::Filename(filename)],
                    })
                    .into_response(&req),
                Err(e) => {
                    log::error!("Error opening temp zip for export: {}", e);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(e) => {
            log::error!("Error exporting playlist ZIP: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn export_playlist_rekordbox(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;
    match playlist_export_service::export_playlist_zip(db, id, true).await {
        Ok((temp_file, filename)) => {
            let path = temp_file.path().to_path_buf();
            match NamedFile::open(path) {
                Ok(named_file) => named_file
                    .set_content_disposition(actix_web::http::header::ContentDisposition {
                        disposition: actix_web::http::header::DispositionType::Attachment,
                        parameters: vec![actix_web::http::header::DispositionParam::Filename(filename)],
                    })
                    .into_response(&req),
                Err(e) => {
                    log::error!("Error opening temp zip for export: {}", e);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(e) => {
            log::error!("Error exporting playlist Rekordbox: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Music file routes
pub async fn get_music_files(
    state: web::Data<AppState>,
    query: web::Query<MusicQueryParams>,
) -> HttpResponse {
    let db = &state.db;
    match music_service::get_all_music_files(db, query.into_inner()).await {
        Ok(files) => HttpResponse::Ok().json(files),
        Err(e) => {
            log::error!("Error fetching music files: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn create_music_file(
    state: web::Data<AppState>,
    req: web::Json<CreateMusicFileRequest>,
) -> HttpResponse {
    let req = req.into_inner();
    if !crate::services::path_safety::is_allowed_music_path(&req.file_path) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "file_path must be within the configured music directory"
        }));
    }

    let db = &state.db;
    match music_service::create_music_file(db, req).await {
        Ok(file) => {
            // Invalidate cached 'all tracks' and notify clients.
            let _ = crate::services::cache_service::invalidate_all_tracks_cache(state.get_ref()).await;
            let _ = crate::services::cache_service::notify_change(state.get_ref(), "music_created", serde_json::to_value(&file).unwrap_or(serde_json::Value::Null)).await;
            HttpResponse::Created().json(file)
        }
        Err(e) => {
            log::error!("Error creating music file: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Sync a music folder into the database. JSON body: { "folder": "/path/to/music" }
pub async fn sync_music_folder(
    state: web::Data<AppState>,
    folder: Option<web::Json<serde_json::Value>>,
) -> HttpResponse {
    let db = &state.db;
    let folder_path = folder
        .and_then(|v| {
            v.get("folder")
                .and_then(|f| f.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| std::env::var("MUSIC_FOLDER").ok());

    let folder = match folder_path {
        Some(p) => p,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "No folder provided and MUSIC_FOLDER not set"}))
        }
    };

    match file_sync_service::sync_folder(db, &folder).await {
        Ok(inserted) => HttpResponse::Ok().json(serde_json::json!({"inserted": inserted})),
        Err(e) => {
            log::error!("Error syncing folder {}: {}", folder, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Background sync using sync_manager with SSE progress
pub async fn start_background_sync(
    state: web::Data<AppState>,
    folder: Option<web::Json<serde_json::Value>>,
) -> HttpResponse {
    let db = state.db.clone();
    let folder_path = folder
        .and_then(|v| {
            v.get("folder")
                .and_then(|f| f.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| std::env::var("MUSIC_FOLDER").ok());

    let folder = match folder_path {
        Some(p) => p,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "No folder provided and MUSIC_FOLDER not set"}))
        }
    };

    // ensure sync_sessions exist on AppState (create lazily)
    // We'll store it in a global once_cell to avoid changing AppState struct for now
    let sessions = crate::services::sync_manager::create_sync_sessions();

    match sync_manager::start_sync(db, folder, sessions).await {
        Ok(session_id) => {
            HttpResponse::Accepted().json(serde_json::json!({"session_id": session_id}))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

pub async fn sync_progress_stream(path: web::Path<String>) -> HttpResponse {
    let session_id = path.into_inner();
    // Lookup global sessions from sync_manager
    if let Some(sessions) = crate::services::sync_manager::get_sync_sessions() {
        let sessions_map = sessions.read().await;
        if let Some(session) = sessions_map.get(&session_id) {
            let rx = session.progress_tx.subscribe();
            drop(sessions_map);

            return crate::services::sse_helpers::sse_response(rx, |p: &crate::services::sync_manager::SyncProgress| {
                p.progress == Some(100.0) || p.is_cancelled == Some(true)
            });
        }
    }

    HttpResponse::NotFound().json(serde_json::json!({"error": "Session not found"}))
}

pub async fn cancel_sync(_path: web::Path<String>) -> HttpResponse {
    let session_id = _path.into_inner();
    if crate::services::sync_manager::cancel_session(&session_id).await {
        HttpResponse::Ok().json(serde_json::json!({"message": "Sync cancelled"}))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({"error": "Session not found"}))
    }
}

pub async fn delete_music_file(state: web::Data<AppState>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;

    match music_service::delete_music_file(db, id).await {
        Ok(_) => {
            // Invalidate cache and notify clients
            let _ = crate::services::cache_service::invalidate_all_tracks_cache(state.get_ref()).await;
            let payload = serde_json::json!({"id": id});
            let _ = crate::services::cache_service::notify_change(state.get_ref(), "music_deleted", payload).await;
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            log::error!("Error deleting music file: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

pub async fn cut_music_file(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: web::Json<CutAudioRequest>,
) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;

    match audio_edit_service::cut_audio(db, id, req.start, req.end).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "success"})),
        Err(e) => {
            log::error!("Error cutting music file: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

pub async fn stream_music_file(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> actix_web::Result<actix_web::HttpResponse> {
    let id = path.into_inner();
    let db = &state.db;
    let record = music_service::get_music_file(db, id).await.map_err(|e| {
        log::error!("Error getting music file {} for streaming: {}", id, e);
        actix_web::error::ErrorInternalServerError("db")
    })?;

    if let Some(file) = record {
        if !crate::services::path_safety::is_allowed_music_path(&file.file_path) {
            log::warn!("Blocked stream for disallowed path: {}", file.file_path);
            return Ok(HttpResponse::Forbidden().finish());
        }

        let file_path = Path::new(&file.file_path);
        if !file_path.exists() {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "file_not_found"
            })));
        }

        let mime = mime_guess::from_path(file_path).first_or_octet_stream();
        let named = NamedFile::open_async(file_path).await?;
        Ok(named.set_content_type(mime).into_response(&req))
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

pub async fn upload_music_files(
    state: web::Data<AppState>,
    mut payload: Multipart,
) -> HttpResponse {
    let upload_root = std::env::var("MUSIC_UPLOAD_DIR")
        .or_else(|_| std::env::var("MUSIC_FOLDER"))
        .unwrap_or_else(|_| "uploads".to_string());

    if let Err(e) = tokio::fs::create_dir_all(&upload_root).await {
        log::error!("Failed to ensure upload dir {}: {}", upload_root, e);
        return HttpResponse::InternalServerError().finish();
    }

    let db = state.db.clone();
    let mut inserted = Vec::new();
    let mut errors = Vec::new();
    let mut files_to_cleanup = Vec::new(); // Track files for cleanup on failure

    while let Some(field) = payload.next().await {
        match field {
            Ok(mut field) => {
                let filename = field
                    .content_disposition()
                    .and_then(|cd| cd.get_filename())
                    .map(sanitize)
                    .unwrap_or_else(|| format!("upload-{}", chrono::Utc::now().timestamp()));

                let filepath = Path::new(&upload_root).join(&filename);
                
                // Track the file for potential cleanup
                files_to_cleanup.push(filepath.clone());
                
                match File::create(&filepath).await {
                    Ok(mut file) => {
                        let mut write_failed = false;
                        while let Some(chunk) = field.next().await {
                            match chunk {
                                Ok(bytes) => {
                                    if let Err(e) = file.write_all(&bytes).await {
                                        errors.push(format!("Failed to write {}: {}", filename, e));
                                        write_failed = true;
                                        break;
                                    }
                                }
                                Err(e) => {
                                    errors.push(format!("Chunk error for {}: {}", filename, e));
                                    write_failed = true;
                                    break;
                                }
                            }
                        }

                        // If write failed, clean up the incomplete file and continue
                        if write_failed {
                            let _ = tokio::fs::remove_file(&filepath).await;
                            files_to_cleanup.pop(); // Don't need to track this anymore
                            continue;
                        }

                        // Extract metadata from the uploaded file using shared helper
                        let meta = crate::services::metadata_extractor::extract_metadata(&filepath).await;

                        let (artist_opt, title_opt, album_opt, duration_ms) = (meta.artist, meta.title, meta.album, meta.duration_ms);

                        // Fallback to filename parsing when metadata missing
                        let file_name = Path::new(&filename)
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or_default();
                        let (file_artist, file_title) = crate::services::filename_parser::parse_filename(file_name);

                        // Use metadata artist if available and not empty, otherwise use parsed artist
                        let artist = artist_opt
                            .filter(|s| !s.trim().is_empty())
                            .or(file_artist);
                        
                        // Use metadata title if available and not empty, otherwise use parsed title
                        let mut title = title_opt
                            .filter(|s| !s.trim().is_empty())
                            .unwrap_or(file_title);
                        
                        // If title contains "Artist - Title" pattern and we have an artist, extract just the title
                        if let Some(ref artist_name) = artist {
                            // Check if title starts with "Artist - " pattern
                            let artist_prefix = format!("{} - ", artist_name);
                            if title.starts_with(&artist_prefix) || title.to_lowercase().starts_with(&artist_prefix.to_lowercase()) {
                                title = title[artist_prefix.len()..].to_string();
                            }
                        }

                        // Compute file hash for duplicate detection
                        let file_path_str = filepath.to_string_lossy().into_owned();
                        let file_hash = match music_service::compute_file_hash(&file_path_str).await {
                            Ok(hash) => Some(hash),
                            Err(e) => {
                                log::warn!("Failed to compute file hash for {}: {}", filename, e);
                                None
                            }
                        };

                        // Check for duplicates
                        if let Some(ref hash) = file_hash {
                            match music_service::is_duplicate_hash(&db, hash).await {
                                Ok(true) => {
                                    // Only delete when the duplicate lives at a different path.
                                    // Retries that rewrite the same path must not remove the library file.
                                    let same_path_as_library = match music_service::get_by_hash(&db, hash).await {
                                        Ok(Some(existing)) => existing.file_path == file_path_str,
                                        _ => false,
                                    };
                                    if !same_path_as_library {
                                        let _ = tokio::fs::remove_file(&filepath).await;
                                    }
                                    files_to_cleanup.pop(); // Remove from tracking
                                    errors.push(format!("Duplicate file skipped: {} (same content already exists)", filename));
                                    continue;
                                }
                                Err(e) => {
                                    log::warn!("Failed to check for duplicate: {}", e);
                                }
                                _ => {}
                            }
                        }

                        let req = CreateMusicFileRequest {
                            title,
                            artist,
                            album: album_opt,
                            genre: None,
                            guessed_genre: None,
                            release_date: None,
                            duration: duration_ms,
                            file_path: file_path_str,
                            track_number: None,
                            file_hash,
                            bpm: None,
                            initial_key: None,
                            beat_grid_offset: None,
                            beat_map: None,
                            metadata_analyzed: Some(false),
                        };

                        match music_service::create_music_file(&db, req).await {
                            Ok(record) => {
                                // Invalidate cache and notify clients
                                let _ = crate::services::cache_service::invalidate_all_tracks_cache(state.get_ref()).await;
                                let _ = crate::services::cache_service::notify_change(
                                    state.get_ref(),
                                    "music_created",
                                    serde_json::to_value(&record).unwrap_or(serde_json::Value::Null)
                                ).await;

                                // Parse all artists from artist field and title
                                let parsed = crate::services::artist_parser::parse_artists(
                                    record.artist.as_deref(),
                                    Some(&record.title),
                                );
                                
                                // Ensure all extracted artists exist in artist_genres
                                let all_artists = parsed.all_artists();
                                if let Err(e) = crate::services::artist_service::ensure_artists_exist(&db, &all_artists).await {
                                    log::warn!("Failed to ensure artists exist: {}", e);
                                }
                                
                                // File was successfully inserted - remove from cleanup list
                                files_to_cleanup.pop();
                                inserted.push(record);
                            }
                            Err(e) => {
                                // DB insert failed - clean up the file
                                let _ = tokio::fs::remove_file(&filepath).await;
                                files_to_cleanup.pop();
                                errors.push(format!("DB insert failed for {}: {}", filename, e));
                                log::warn!("Cleaned up file {} due to DB insert failure", filename);
                            }
                        }
                    }
                    Err(e) => {
                        files_to_cleanup.pop();
                        errors.push(format!(
                            "Failed to create file {}: {}",
                            filepath.display(),
                            e
                        ));
                    }
                }
            }
            Err(e) => errors.push(format!("Upload field error: {}", e)),
        }
    }

    // Clean up any remaining tracked files (shouldn't happen in normal operation)
    for filepath in files_to_cleanup {
        if filepath.exists() {
            let _ = std::fs::remove_file(&filepath);
            log::warn!("Cleaned up leftover file: {}", filepath.display());
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "inserted": inserted,
        "errors": errors,
    }))
}


/// Get music stats (total count + duration) for current filters (ignoring pagination)
pub async fn get_music_stats(
    state: web::Data<AppState>,
    query: web::Query<MusicQueryParams>,
) -> HttpResponse {
    let db = &state.db;
    match music_service::get_music_stats(db, query.into_inner()).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => {
            log::error!("Error fetching music stats: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_music_file_detail(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;
    match music_service::get_music_file(db, id).await {
        Ok(Some(file)) => HttpResponse::Ok().json(file),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            log::error!("Error fetching music file {}: {}", id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_music_file_handler(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    payload: web::Json<UpdateMusicFileRequest>,
) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;
    match music_service::update_music_file(db, id, payload.into_inner()).await {
        Ok(Some(file)) => {
            // Invalidate cache and notify clients about update
            let _ = crate::services::cache_service::invalidate_all_tracks_cache(state.get_ref()).await;
            let _ = crate::services::cache_service::notify_change(state.get_ref(), "music_updated", serde_json::to_value(&file).unwrap_or(serde_json::Value::Null)).await;
            HttpResponse::Ok().json(file)
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            log::error!("Error updating music file {}: {}", id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(serde::Deserialize)]
pub struct CheckDuplicateHashRequest {
    pub hash: String,
}

pub async fn check_duplicate_hash(
    state: web::Data<AppState>,
    payload: web::Json<CheckDuplicateHashRequest>,
) -> HttpResponse {
    let db = &state.db;
    match music_service::is_duplicate_hash(db, &payload.hash).await {
        Ok(exists) => HttpResponse::Ok().json(serde_json::json!({ "exists": exists })),
        Err(e) => {
            log::error!("Error checking duplicate hash: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Get playlists that contain a specific track
pub async fn get_track_playlists(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;
    match music_service::get_track_playlists(db, id).await {
        Ok(playlists) => HttpResponse::Ok().json(playlists),
        Err(e) => {
            log::error!("Error fetching track playlists: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Lookup release date from MusicBrainz for a track
#[derive(serde::Deserialize)]
pub struct ReleaseDateLookupRequest {
    pub title: String,
    pub artist: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ReleaseDateLookupResponse {
    pub release_date: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub confidence: f64,
}

pub async fn lookup_release_date(
    state: web::Data<AppState>,
    body: web::Json<ReleaseDateLookupRequest>,
) -> HttpResponse {
    let title = &body.title;
    let artist = body.artist.as_deref().unwrap_or("");

    // Load metadata configuration
    let config = match MetadataConfig::get_config(&state.db.pool).await {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to load metadata config: {}", e);
            // Fallback to Discogs as default if DB fails
            MetadataConfig {
                id: Uuid::new_v4(),
                metadata_source: "discogs".to_string(),
                discogs_token: std::env::var("DISCOGS_TOKEN").ok(),
                updated_at: chrono::Utc::now(),
            }
        }
    };

    let client = &state.http_client;

    if config.metadata_source == "discogs" {
        log::info!("Looking up release date on Discogs for: {} - {}", artist, title);
        match DiscogsService::lookup_release_date(client, &config, title, artist).await {
            Ok(Some((date, album, style, confidence))) => {
                return HttpResponse::Ok().json(ReleaseDateLookupResponse {
                    release_date: Some(date),
                    album,
                    genre: style,
                    confidence,
                });
            }
            Ok(None) => {
                log::info!("No release date found on Discogs for: {} - {}", artist, title);
                return HttpResponse::Ok().json(serde_json::json!({
                    "release_date": null,
                    "album": null,
                    "genre": null,
                    "confidence": 0.0,
                    "error": "No match found on Discogs"
                }));
            }
            Err(e) => {
                log::error!("Discogs lookup failed: {}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Lookup failed: {}", e)
                }));
            }
        }
    }

    // Default: MusicBrainz
    // Build a MusicBrainz recording search query
    let query = if artist.is_empty() {
        format!("recording:{}", urlencoding::encode(title))
    } else {
        format!(
            "recording:{} AND artist:{}",
            urlencoding::encode(title),
            urlencoding::encode(artist)
        )
    };

    let url = format!(
        "https://musicbrainz.org/ws/2/recording/?query={}&fmt=json&limit=5",
        query
    );

    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            log::error!("MusicBrainz request failed: {}", e);
            return HttpResponse::Ok().json(ReleaseDateLookupResponse {
                release_date: None,
                album: None,
                genre: None,
                confidence: 0.0,
            });
        }
    };

    let json: serde_json::Value = match response.json().await {
        Ok(j) => j,
        Err(e) => {
            log::error!("MusicBrainz JSON parse failed: {}", e);
            return HttpResponse::Ok().json(ReleaseDateLookupResponse {
                release_date: None,
                album: None,
                genre: None,
                confidence: 0.0,
            });
        }
    };

    // Parse the response - look for the earliest release date among high-confidence matches
    let mut best_match: Option<(String, Option<String>, f64)> = None;

    if let Some(recordings) = json.get("recordings").and_then(|r| r.as_array()) {
        for recording in recordings {
            let score = recording.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0);
            if score < 50.0 {
                continue;
            }

            // Look at releases for this recording
            if let Some(releases) = recording.get("releases").and_then(|r| r.as_array()) {
                for release in releases {
                    let date = release.get("date").and_then(|d| d.as_str());
                    let album_title = release.get("title").and_then(|t| t.as_str());
                    
                    if let Some(date_str) = date {
                        if !date_str.is_empty() {
                            // If we have a new date, check if it's earlier than our current best
                            let is_earlier = match &best_match {
                                None => true,
                                Some((best_date, _, _)) => date_str < best_date.as_str(),
                            };

                            if is_earlier {
                                best_match = Some((
                                    date_str.to_string(),
                                    album_title.map(|s| s.to_string()),
                                    score / 100.0,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some((date, album, confidence)) = best_match {
        return HttpResponse::Ok().json(ReleaseDateLookupResponse {
            release_date: Some(date),
            album,
            genre: None, // MusicBrainz doesn't give genre directly here easily without more calls
            confidence,
        });
    }

    HttpResponse::Ok().json(ReleaseDateLookupResponse {
        release_date: None,
        album: None,
        genre: None,
        confidence: 0.0,
    })
}

#[derive(serde::Deserialize)]
pub struct PaginationQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn get_metadata_suggestions(
    state: web::Data<AppState>,
    query: web::Query<PaginationQuery>,
) -> HttpResponse {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let suggestions = match sqlx::query_as::<_, MetadataSuggestion>(
        r#"
        SELECT music_file_id, release_date, album, genre, confidence, created_at, updated_at
        FROM metadata_suggestions
        WHERE confidence > 0.0 OR release_date IS NOT NULL OR album IS NOT NULL OR genre IS NOT NULL
        ORDER BY confidence DESC, created_at DESC
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db.pool)
    .await
    {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to fetch metadata suggestions: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to fetch metadata suggestions: {}", e)
            }));
        }
    };

    // Also return total count for pagination
    let total_count: (i64,) = match sqlx::query_as("SELECT COUNT(*) FROM metadata_suggestions")
        .fetch_one(&state.db.pool)
        .await
    {
        Ok(c) => c,
        Err(_) => (0,),
    };

    HttpResponse::Ok().json(serde_json::json!({
        "suggestions": suggestions,
        "total": total_count.0,
        "offset": offset,
        "limit": limit
    }))
}

pub async fn delete_metadata_suggestion(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let id = path.into_inner();
    match sqlx::query("DELETE FROM metadata_suggestions WHERE music_file_id = $1")
        .bind(id)
        .execute(&state.db.pool)
        .await
    {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            log::error!("Failed to delete metadata suggestion: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

pub async fn delete_all_metadata_suggestions(state: web::Data<AppState>) -> HttpResponse {
    match sqlx::query("DELETE FROM metadata_suggestions")
        .execute(&state.db.pool)
        .await
    {
        Ok(result) => HttpResponse::Ok().json(serde_json::json!({
            "deleted": result.rows_affected()
        })),
        Err(e) => {
            log::error!("Failed to delete all metadata suggestions: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

pub async fn list_streams(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match internet_stream_service::list_streams(db).await {
        Ok(streams) => HttpResponse::Ok().json(streams),
        Err(e) => {
            log::error!("Error listing streams: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn create_stream(
    state: web::Data<AppState>,
    payload: web::Json<CreateStreamRequest>,
) -> HttpResponse {
    let db = &state.db;
    match internet_stream_service::create_stream(db, payload.into_inner()).await {
        Ok(stream) => HttpResponse::Created().json(stream),
        Err(e) => {
            log::error!("Error creating stream: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_stream(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    payload: web::Json<UpdateStreamRequest>,
) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;
    match internet_stream_service::update_stream(db, id, payload.into_inner()).await {
        Ok(Some(stream)) => HttpResponse::Ok().json(stream),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            log::error!("Error updating stream {}: {}", id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn delete_stream(state: web::Data<AppState>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;
    match internet_stream_service::delete_stream(db, id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().finish(),
        Err(e) => {
            log::error!("Error deleting stream {}: {}", id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Artist routes
pub async fn list_artists(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match artist_service::get_all_artists_with_summary(db).await {
        Ok(artists) => HttpResponse::Ok().json(artists),
        Err(e) => {
            log::error!("Error listing artists: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_artist_music(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let artist_name = path.into_inner();
    let db = &state.db;
    
    // URL decode the artist name
    let decoded_name = urlencoding::decode(&artist_name)
        .map(|s| s.into_owned())
        .unwrap_or(artist_name);
    
    match artist_service::get_music_featuring_artist(db, &decoded_name).await {
        Ok(music) => HttpResponse::Ok().json(music),
        Err(e) => {
            log::error!("Error fetching music for artist {}: {}", decoded_name, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(serde::Deserialize)]
pub struct SetArtistGenreRequest {
    pub genre: String,
}

pub async fn set_artist_genre_handler(
    state: web::Data<AppState>,
    path: web::Path<String>,
    payload: web::Json<SetArtistGenreRequest>,
) -> HttpResponse {
    let artist_name = path.into_inner();
    let db = &state.db;
    
    // URL decode the artist name
    let decoded_name = urlencoding::decode(&artist_name)
        .map(|s| s.into_owned())
        .unwrap_or(artist_name);
    
    match artist_service::set_artist_genre(db, &decoded_name, &payload.genre).await {
        Ok(()) => {
            // Also update guessed_genre for all their songs
            let _ = artist_service::update_guessed_genre_for_artist(db, &decoded_name, &payload.genre).await;
            // Invalidate artist summary cache and all tracks cache and notify clients
            let _ = crate::services::cache_service::invalidate_artists_summary_cache(state.get_ref()).await;
            let _ = crate::services::cache_service::invalidate_all_tracks_cache(state.get_ref()).await;
            let payload_val = serde_json::json!({"artist": decoded_name.clone(), "genre": payload.genre});
            let _ = crate::services::cache_service::notify_change(state.get_ref(), "artist_genre_set", payload_val).await;
            HttpResponse::Ok().json(serde_json::json!({"success": true}))
        }
        Err(e) => {
            log::error!("Error setting genre for artist {}: {}", decoded_name, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(serde::Deserialize)]
pub struct ConfirmGenreRequest {
    pub track_id: String,
    pub genre: String,
}

pub async fn confirm_genre_handler(
    state: web::Data<AppState>,
    payload: web::Json<ConfirmGenreRequest>,
) -> HttpResponse {
    let db = &state.db;
    
    // Parse track ID
    let track_id = match uuid::Uuid::parse_str(&payload.track_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid track ID"
            }))
        }
    };

    let select_sql = format!("{} WHERE id = $1", crate::services::music_query_helpers::select_music_files());
    let track = match sqlx::query_as::<_, crate::models::MusicFile>(&select_sql)
    .bind(track_id)
    .fetch_optional(&db.pool)
    .await {
        Ok(Some(track)) => track,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Track not found"
            }))
        }
        Err(e) => {
            log::error!("Error fetching track: {}", e);
            return HttpResponse::InternalServerError().finish()
        }
    };

    // Update the track's genre
    if let Err(e) = sqlx::query("UPDATE music_files SET genre = $1, updated_at = NOW() WHERE id = $2")
        .bind(&payload.genre)
        .bind(track_id)
        .execute(&db.pool)
        .await {
        log::error!("Error updating track genre: {}", e);
        return HttpResponse::InternalServerError().finish()
    }

    // If the artist doesn't have a confirmed genre yet, set it
    if let Some(artist) = &track.artist {
        if !artist.trim().is_empty() {
            // Check if artist has a confirmed genre (not "Unknown")
            match sqlx::query_scalar::<_, String>(
                "SELECT genre FROM artist_genres WHERE artist_name = $1"
            )
            .bind(artist)
            .fetch_optional(&db.pool)
            .await {
                Ok(Some(current_genre)) if current_genre != "Unknown" => {
                    // Artist already has a confirmed genre, skip
                }
                _ => {
                    // Artist doesn't exist or has "Unknown" genre, set it
                    if let Err(e) = artist_service::set_artist_genre(db, artist, &payload.genre).await {
                        log::warn!("Error setting artist genre: {}", e);
                    }
                }
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "track_id": payload.track_id,
        "genre": payload.genre
    }))
}

#[derive(serde::Deserialize)]
pub struct RenameArtistRequest {
    pub new_name: String,
}

pub async fn rename_artist_handler(
    state: web::Data<AppState>,
    path: web::Path<String>,
    payload: web::Json<RenameArtistRequest>,
) -> HttpResponse {
    let old_name = path.into_inner();
    let db = &state.db;
    
    // URL decode the artist name
    let decoded_old_name = urlencoding::decode(&old_name)
        .map(|s| s.into_owned())
        .unwrap_or(old_name);
    
    let new_name = payload.new_name.trim();
    
    if new_name.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "New artist name cannot be empty"
        }));
    }
    
    if decoded_old_name == new_name {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "New name is the same as the old name"
        }));
    }
    
    match artist_service::rename_artist(db, &decoded_old_name, new_name).await {
        Ok(result) => {
            // Invalidate caches and notify
            let _ = crate::services::cache_service::invalidate_artists_summary_cache(state.get_ref()).await;
            let _ = crate::services::cache_service::invalidate_all_tracks_cache(state.get_ref()).await;
            let payload_val = serde_json::json!({"old_name": decoded_old_name, "new_name": new_name});
            let _ = crate::services::cache_service::notify_change(state.get_ref(), "artist_renamed", payload_val).await;
            HttpResponse::Ok().json(result)
        },
        Err(e) => {
            log::error!("Error renaming artist {} to {}: {}", decoded_old_name, new_name, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to rename artist: {}", e)
            }))
        }
    }
}

/// Reprocess all music files to extract artists from compound names and titles
pub async fn reprocess_artists(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match artist_service::reprocess_all_for_artists(db).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Added {} new artists from existing tracks", count),
            "artists_added": count
        })),
        Err(e) => {
            log::error!("Error reprocessing artists: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to reprocess artists: {}", e)
            }))
        }
    }
}

// YouTube download routes
pub async fn download_youtube(
    state: web::Data<AppState>,
    req: web::Json<YoutubeDownloadRequest>,
) -> HttpResponse {
    log::info!("Starting YouTube download for URL: {}", req.url);
    log::info!("Output directory: {}", req.output_dir);

    if !crate::services::path_safety::is_allowed_download_dir(&req.output_dir) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "output_dir must be within the configured downloads directory"
        }));
    }
    
    let options = DownloadOptions {
        limit: req.limit,
        max_concurrent: req.max_concurrent,
        audio_quality: req.audio_quality.clone(),
    };

    let db = &state.db;
    let pool = db.pool.clone();

    // Start the download and get the session ID
    let url = req.url.clone();
    let output_dir = req.output_dir.clone();
    let sessions = state.download_sessions.clone();
    let update_tx = state.cache_update_tx.clone();

    match yt_downloader::download_youtube_playlist(url, output_dir, Some(options), sessions, pool, Some(update_tx))
        .await
    {
        Ok(session_id) => HttpResponse::Accepted().json(YoutubeDownloadResponse {
            session_id,
            message: "Download started".to_string(),
        }),
        Err(e) => {
            log::error!("Failed to start download: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
        }
    }
}

pub async fn youtube_download_progress(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let session_id = path.into_inner();
    let sessions = state.download_sessions.read().await;

    if let Some(session) = sessions.get(&session_id) {
        let progress = session.progress.read().await;
        HttpResponse::Ok().json(&*progress)
    } else {
        HttpResponse::NotFound().json(serde_json::json!({"error": "Session not found"}))
    }
}

pub async fn cancel_youtube_download(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let session_id = path.into_inner();
    let sessions = state.download_sessions.read().await;

    if let Some(session) = sessions.get(&session_id) {
        session
            .cancelled
            .store(true, std::sync::atomic::Ordering::Relaxed);
        HttpResponse::Ok().json(serde_json::json!({"message": "Download cancelled"}))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({"error": "Session not found"}))
    }
}

pub async fn youtube_download_stream(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let session_id = path.into_inner();
    let sessions = state.download_sessions.read().await;

    if let Some(session) = sessions.get(&session_id) {
        let rx = session.progress_tx.subscribe();
        drop(sessions); // Release the lock

        crate::services::sse_helpers::sse_response(rx, |p: &crate::yt_downloader::DownloadProgress| {
            p.progress == Some(100.0) || p.is_cancelled == Some(true)
        })
    } else {
        HttpResponse::NotFound().json(serde_json::json!({"error": "Session not found"}))
    }
}

/// SSE stream for generic updates/notifications (cache invalidation, created/updated/deleted items)
pub async fn updates_stream(state: web::Data<AppState>) -> HttpResponse {
    let rx = state.cache_update_tx.subscribe();

    crate::services::sse_helpers::sse_response(rx, |_: &serde_json::Value| false)
}

pub async fn get_cached_tracks(state: web::Data<AppState>) -> HttpResponse {
    match crate::services::cache_service::get_all_tracks_cached(&state).await {
        Ok(files) => HttpResponse::Ok().json(files),
        Err(e) => {
            log::error!("Error fetching cached tracks: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_cached_artists(state: web::Data<AppState>) -> HttpResponse {
    match crate::services::cache_service::get_artists_summary_cached(&state).await {
        Ok(artists) => HttpResponse::Ok().json(artists),
        Err(e) => {
            log::error!("Error fetching cached artists: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_downloaded_videos(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match yt_download_service::get_all_downloads(&db.pool).await {
        Ok(downloads) => HttpResponse::Ok().json(downloads),
        Err(e) => {
            log::error!("Error fetching downloaded videos: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_download_stats(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match yt_download_service::get_download_count(&db.pool).await {
        Ok(count) => HttpResponse::Ok().json(crate::models::YoutubeDownloadStats {
            total_downloads: count,
            unique_videos: count,
            total_size: 0, // Not currently tracked
        }),
        Err(e) => {
            log::error!("Error getting download count: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn delete_download_record(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let video_id = path.into_inner();
    let db = &state.db;
    match yt_download_service::delete_download(&db.pool, &video_id).await {
        Ok(deleted) => {
            if deleted {
                HttpResponse::NoContent().finish()
            } else {
                HttpResponse::NotFound().finish()
            }
        }
        Err(e) => {
            log::error!("Error deleting download: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn list_youtube_playlists(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match youtube_playlist_service::list_playlists(db).await {
        Ok(playlists) => HttpResponse::Ok().json(playlists),
        Err(e) => {
            log::error!("Error listing YouTube playlists: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_youtube_playlist(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;
    match youtube_playlist_service::get_playlist(db, id).await {
        Ok(Some(playlist)) => HttpResponse::Ok().json(playlist),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "Playlist not found"})),
        Err(e) => {
            log::error!("Error getting YouTube playlist: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn create_youtube_playlist(
    state: web::Data<AppState>,
    body: web::Json<CreateYoutubePlaylistRequest>,
) -> HttpResponse {
    let db = &state.db;
    match youtube_playlist_service::create_playlist(db, body.into_inner()).await {
        Ok(playlist) => HttpResponse::Created().json(playlist),
        Err(e) => {
            log::error!("Error creating YouTube playlist: {}", e);
            if e.to_string().contains("duplicate key") {
                HttpResponse::Conflict().json(serde_json::json!({"error": "Playlist URL already exists"}))
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

pub async fn update_youtube_playlist(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateYoutubePlaylistRequest>,
) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;
    match youtube_playlist_service::update_playlist(db, id, body.into_inner()).await {
        Ok(Some(playlist)) => HttpResponse::Ok().json(playlist),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "Playlist not found"})),
        Err(e) => {
            log::error!("Error updating YouTube playlist: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn delete_youtube_playlist(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;
    match youtube_playlist_service::delete_playlist(db, id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({"error": "Playlist not found"})),
        Err(e) => {
            log::error!("Error deleting YouTube playlist: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn sync_youtube_playlist(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;
    
    // Get the playlist
    let playlist = match youtube_playlist_service::get_playlist(db, id).await {
        Ok(Some(p)) => p,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({"error": "Playlist not found"})),
        Err(e) => {
            log::error!("Error getting YouTube playlist: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    // Start the download
    let options = DownloadOptions {
        limit: None,
        max_concurrent: Some(3),
        audio_quality: Some("192".to_string()),
    };
    
    let sessions = state.download_sessions.clone();
    let pool = db.pool.clone();
    let update_tx = state.cache_update_tx.clone();
    
    match yt_downloader::download_youtube_playlist(
        playlist.url.clone(),
        "/music/downloads".to_string(),
        Some(options),
        sessions,
        pool,
        Some(update_tx),
    ).await {
        Ok(session_id) => {
            // Mark as synced
            let _ = youtube_playlist_service::mark_synced(db, id).await;
            HttpResponse::Accepted().json(serde_json::json!({
                "sessionId": session_id,
                "message": "Sync started"
            }))
        }
        Err(e) => {
            log::error!("Failed to start playlist sync: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
        }
    }
}

// Genre detection routes
pub async fn detect_genre(
    state: web::Data<AppState>,
    req: web::Json<DetectGenreRequest>,
) -> HttpResponse {
    let db = &state.db;
    match genre_detection::detect_genre_for_artist(db, req.artist_name.clone()).await {
        Ok(genre) => {
            let cached = genre_cache_service::is_cached(db, &req.artist_name)
                .await
                .unwrap_or(false);
            HttpResponse::Ok().json(DetectGenreResponse {
                artist_name: req.artist_name.clone(),
                genre,
                cached,
            })
        }
        Err(e) => {
            log::error!("Error detecting genre: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
        }
    }
}

pub async fn get_genre_cache(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match genre_cache_service::get_all_cached_genres(db).await {
        Ok(cache) => HttpResponse::Ok().json(cache),
        Err(e) => {
            log::error!("Error fetching genre cache: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn clear_genre_cache(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match genre_cache_service::clear_all_cache(db).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({"cleared": count})),
        Err(e) => {
            log::error!("Error clearing genre cache: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Genre label endpoints (canonical genres and aliases)
pub async fn list_genres(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    // Return genres with track counts from actual music files
    match genre_label_service::list_genres_with_counts(db).await {
        Ok(genres) => HttpResponse::Ok().json(genres),
        Err(e) => {
            // Log full error server-side and return a small JSON error so the client can show details
            log::error!("Error listing genres: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": format!("Failed to list genres: {}", e.to_string()) }))
        }
    }
}

// List canonical genres (admin-defined genre taxonomy)
pub async fn list_canonical_genres(
    state: web::Data<AppState>,
) -> HttpResponse {
    let db = &state.db;
    match genre_label_service::list_genres_extended(db).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => {
            log::error!("Error listing canonical genres: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn merge_genres_handler(
    state: web::Data<AppState>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let source_id = body
        .get("source_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let target_id = body
        .get("target_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    if source_id.is_none() || target_id.is_none() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Missing source_id or target_id"}));
    }

    match genre_label_service::merge_genres(db, source_id.unwrap(), target_id.unwrap()).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => {
            log::error!("Error merging genres: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn list_unmapped_genres(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match genre_label_service::list_unmapped_tags(db, 100).await {
        Ok(tags) => HttpResponse::Ok().json(tags),
        Err(e) => {
            log::error!("Error listing unmapped genres: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn suggest_genre_matches(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let db = &state.db;
    let raw = path.into_inner();
    match genre_label_service::suggest_similar(db, &raw, 5).await {
        Ok(suggestions) => HttpResponse::Ok().json(suggestions),
        Err(e) => {
            log::error!("Error suggesting genres: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn create_genre(
    state: web::Data<AppState>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let name = body.get("name").and_then(|v| v.as_str());
    let description = body.get("description").and_then(|v| v.as_str());

    if name.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Missing name"}));
    }

    match genre_label_service::create_genre(db, name.unwrap(), description).await {
        Ok(genre) => HttpResponse::Created().json(genre),
        Err(e) => {
            log::error!("Error creating genre: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_genre_handler(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let id = path.into_inner();
    let name = body.get("name").and_then(|v| v.as_str());
    let description = body.get("description").and_then(|v| v.as_str());

    if name.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Missing name"}));
    }

    match genre_label_service::update_genre(db, id, name.unwrap(), description).await {
        Ok(Some(genre)) => HttpResponse::Ok().json(genre),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            log::error!("Error updating genre: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn delete_genre_handler(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let db = &state.db;
    let id = path.into_inner();
    match genre_label_service::delete_genre(db, id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().finish(),
        Err(e) => {
            log::error!("Error deleting genre: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn add_genre_alias(
    state: web::Data<AppState>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let alias = body.get("alias").and_then(|v| v.as_str());
    let genre_id = body
        .get("genre_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    if alias.is_none() || genre_id.is_none() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Missing alias or genre_id"}));
    }

    match genre_label_service::add_alias(db, alias.unwrap(), genre_id.unwrap()).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => {
            log::error!("Error adding genre alias: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn preview_backfill_handler(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let db = &state.db;
    let alias = path.into_inner();
    match genre_label_service::preview_backfill(db, &alias).await {
        Ok((music_rows, artist_rows)) => HttpResponse::Ok().json(serde_json::json!({
            "music_rows": music_rows,
            "artist_rows": artist_rows
        })),
        Err(e) => {
            log::error!("Error previewing backfill: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn add_genre_alias_and_backfill(
    state: web::Data<AppState>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let alias = body.get("alias").and_then(|v| v.as_str());
    let genre_id = body
        .get("genre_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    if alias.is_none() || genre_id.is_none() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Missing alias or genre_id"}));
    }

    // Get the canonical name
    let canonical_name: String = match sqlx::query_scalar("SELECT name FROM genres WHERE id = $1")
        .bind(genre_id.unwrap())
        .fetch_one(&db.pool)
        .await
    {
        Ok(name) => name,
        Err(_) => return HttpResponse::NotFound().finish(),
    };

    // Add alias
    if let Err(e) = genre_label_service::add_alias(db, alias.unwrap(), genre_id.unwrap()).await {
        log::error!("Error adding alias: {}", e);
        return HttpResponse::InternalServerError().finish();
    }

    // Backfill
    match genre_label_service::backfill_alias(db, alias.unwrap(), &canonical_name).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({ "updated_tracks": count })),
        Err(e) => {
            log::error!("Error backfilling alias: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn start_backfill_handler(
    state: web::Data<AppState>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = state.db.clone();
    let alias = body.get("alias").and_then(|v| v.as_str());
    let genre_id = body
        .get("genre_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    if alias.is_none() || genre_id.is_none() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Missing alias or genre_id"}));
    }

    let session_id = backfill_manager::start_backfill(db, alias.unwrap().to_string(), genre_id.unwrap()).await;
    HttpResponse::Accepted().json(serde_json::json!({ "session_id": session_id }))
}

pub async fn backfill_progress_stream(path: web::Path<String>) -> HttpResponse {
    let session_id = path.into_inner();
    let sessions = backfill_manager::get_sessions();
    let sessions_map = sessions.read().await;
    if let Some(session) = sessions_map.get(&session_id) {
        let rx = session.tx.subscribe();
        drop(sessions_map);

        return crate::services::sse_helpers::sse_response(rx, |p: &crate::services::backfill_manager::BackfillProgress| {
            p.finished
        });
    }
    HttpResponse::NotFound().finish()
}

pub async fn reprocess_missing_genres(state: web::Data<AppState>) -> HttpResponse {
    let db = state.db.clone();
    let session_id = reprocess_manager::start_reprocess(db).await;
    HttpResponse::Accepted().json(serde_json::json!({ "session_id": session_id }))
}

pub async fn reprocess_progress_stream(path: web::Path<String>) -> HttpResponse {
    let session_id = path.into_inner();
    let sessions = reprocess_manager::get_sessions();
    let sessions_map = sessions.read().await;
    if let Some(session) = sessions_map.get(&session_id) {
        let rx = session.tx.subscribe();
        drop(sessions_map);

        return crate::services::sse_helpers::sse_response(rx, |p: &crate::services::reprocess_manager::ReprocessProgress| {
            p.finished
        });
    }
    HttpResponse::NotFound().finish()
}

#[derive(serde::Deserialize)]
pub struct DebugDiscogsRequest {
    pub artist: Option<String>,
    pub title: Option<String>,
    /// If true, returns an HTML page that console.logs the result in the browser
    pub log: Option<bool>,
}

/// Admin-only debug endpoint to show which Discogs token source is chosen and a safe token preview.
pub async fn admin_debug_discogs_lookup(
    state: web::Data<AppState>,
    body: web::Json<DebugDiscogsRequest>,
    _req: HttpRequest,
) -> HttpResponse {
    // Determine artist/title
    let artist = body.artist.clone().unwrap_or_default();
    let title = body.title.clone().unwrap_or_default();

    // 1) Try DB
    let mut token_source = "none".to_string();
    let mut token_opt: Option<String> = None;

    // Try reading token from DB (if metadata_config exists)
    if let Ok(Some(t)) = sqlx::query_scalar::<_, String>("SELECT discogs_token FROM metadata_config LIMIT 1").fetch_optional(&state.db.pool).await {
        if !t.trim().is_empty() {
            token_source = "db".to_string();
            token_opt = Some(t);
        }
    }

    // 2) Env fallback
    if token_opt.is_none() {
        if let Ok(env_token) = std::env::var("DISCOGS_TOKEN") {
            if !env_token.trim().is_empty() {
                token_source = "env".to_string();
                token_opt = Some(env_token);
            }
        }
    }

    // 3) Local file fallback
    if token_opt.is_none() {
        for path in &["/home/jarno/Homelab/MusicServer/discogs_token", "discogs_token", "../discogs_token"] {
            if let Ok(s) = std::fs::read_to_string(path) {
                let t = s.trim().to_string();
                if !t.is_empty() {
                    token_source = "file".to_string();
                    token_opt = Some(t);
                    break;
                }
            }
        }
    }

    // Build tier1 URL (redacted)
    let base_url = "https://api.discogs.com";
    let q_title = title.clone();
    let q_artist = artist.clone();
    let mut tier1 = format!("{}/database/search?type=release&release_title={}&artist={}", base_url, url_encode(&q_title), url_encode(&q_artist));

    let token_preview = token_opt.as_ref().map(|t| {
        if t.len() <= 8 { t.clone() } else { format!("{}...{}", &t[..4], &t[t.len()-4..]) }
    });

    if token_opt.is_some() {
        tier1.push_str("&token=REDACTED");
    }

    let payload = serde_json::json!({
        "token_source": token_source,
        "token_preview": token_preview,
        "tier1_url_redacted": tier1,
        "artist": artist,
        "title": title,
    });

        // small helper to escape HTML when embedding JSON into a page
        fn escape_html(s: &str) -> String {
                s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        }

        if body.log.unwrap_or(false) {
        // Return an HTML page that logs payload to the browser console for convenience
        let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
                let js = format!(r#"<!doctype html>
<html>
    <head><meta charset='utf-8'/></head>
    <body>
        <pre id='payload'>{}</pre>
        <script>
            const payload = {};
            console.log('Discogs debug payload:', payload);
            document.getElementById('payload').innerText = JSON.stringify(payload, null,  2);
        </script>
    </body>
</html>"#, escape_html(&pretty), serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string()));

        return HttpResponse::Ok()
            .insert_header((CONTENT_TYPE, "text/html; charset=utf-8"))
            .body(js);
    }

    HttpResponse::Ok().json(payload)
}


/// Auto-download routes
pub async fn get_auto_download_config(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match auto_download_service::get_config(db).await {
        Ok(config) => HttpResponse::Ok().json(config),
        Err(e) => {
            log::error!("Error getting auto-download config: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_auto_download_config(
    state: web::Data<AppState>,
    req: web::Json<UpdateAutoDownloadConfigRequest>,
) -> HttpResponse {
    let db = &state.db;
    match auto_download_service::update_config(db, req.into_inner()).await {
        Ok(config) => HttpResponse::Ok().json(config),
        Err(e) => {
            log::error!("Error updating auto-download config: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Autoplay config endpoints
#[derive(serde::Deserialize)]
pub struct UpdateAutoplayConfigRequest {
    pub match_time_seconds: i32,
    pub overlap_seconds: i32,
    pub exit_time_seconds: i32,
}

pub async fn get_autoplay_config(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match crate::services::autoplay_service::get_autoplay_config(db).await {
        Ok(cfg) => HttpResponse::Ok().json(cfg),
        Err(e) => {
            log::error!("Error getting autoplay config: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_autoplay_config(
    state: web::Data<AppState>,
    req: web::Json<UpdateAutoplayConfigRequest>,
) -> HttpResponse {
    let db = &state.db;
    let body = req.into_inner();
    match crate::services::autoplay_service::update_autoplay_config(db, body.match_time_seconds, body.overlap_seconds, body.exit_time_seconds).await {
        Ok(cfg) => HttpResponse::Ok().json(cfg),
        Err(e) => {
            log::error!("Error updating autoplay config: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_auto_download_status(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    let config = match auto_download_service::get_config(db).await {
        Ok(c) => c,
        Err(e) => {
            log::error!("Error getting config for status: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let state_lock = &state.auto_download_state;
    let current_playlist = state_lock.current_playlist.read().await.clone();
    
    HttpResponse::Ok().json(crate::models::AutoDownloadStatus {
        config,
        is_running: state_lock.is_running.load(std::sync::atomic::Ordering::Relaxed),
        current_playlist,
        downloads_in_progress: state_lock.downloads_in_progress.load(std::sync::atomic::Ordering::Relaxed),
        downloads_completed_this_run: state_lock.downloads_completed.load(std::sync::atomic::Ordering::Relaxed),
        downloads_skipped_this_run: state_lock.downloads_skipped.load(std::sync::atomic::Ordering::Relaxed),
    })
}

pub async fn trigger_auto_download(state: web::Data<AppState>) -> HttpResponse {
    let pool = &state.db.pool;
    let sessions = state.download_sessions.clone();
    let ad_state = state.auto_download_state.clone();
    let update_tx = state.cache_update_tx.clone();
    
    match auto_download_service::trigger_now(pool, &sessions, ad_state, Some(update_tx)).await {
        Ok(msg) => HttpResponse::Ok().json(serde_json::json!({ "message": msg })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}

pub async fn stop_auto_download(state: web::Data<AppState>) -> HttpResponse {
    auto_download_service::stop_current_run(&state.auto_download_state);
    HttpResponse::Ok().json(serde_json::json!({ "message": "Stop signal sent" }))
}

pub async fn bulk_rename_by_regex_handler(
    state: web::Data<AppState>,
    req: web::Json<BulkRenameByRegexRequest>,
) -> HttpResponse {
    let db = &state.db;
    match music_service::bulk_rename_by_regex(db, req.into_inner()).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(e) => {
            log::error!("Bulk rename error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() }))
        }
    }
}

pub async fn bulk_add_to_playlist_by_regex_handler(
    state: web::Data<AppState>,
    req: web::Json<BulkAddToPlaylistByRegexRequest>,
) -> HttpResponse {
    let db = &state.db;
    let playlist_id = req.playlist_id;
    match music_service::bulk_add_to_playlist_by_regex(db, req.into_inner()).await {
        Ok(resp) => {
            // Notify clients that a playlist has changed
            let payload = serde_json::json!({ "id": playlist_id });
            let _ = crate::services::cache_service::notify_change(
                state.get_ref(),
                "playlist_items_updated",
                payload
            ).await;
            HttpResponse::Ok().json(resp)
        },
        Err(e) => {
            log::error!("Bulk add to playlist error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() }))
        }
    }
}

pub async fn bulk_update_music_handler(
    state: web::Data<AppState>,
    req: web::Json<crate::models::BulkUpdateMusicRequest>,
) -> HttpResponse {
    let db = &state.db;
    match music_service::bulk_update_music(db, req.into_inner()).await {
        Ok(count) => {
            if count > 0 {
                let _ = crate::services::cache_service::invalidate_all_tracks_cache(state.get_ref()).await;
                let payload_val = serde_json::json!({"updated_count": count});
                let _ = crate::services::cache_service::notify_change(state.get_ref(), "music_bulk_updated", payload_val).await;
            }
            HttpResponse::Ok().json(serde_json::json!({ "updated_count": count }))
        },
        Err(e) => {
            log::error!("Bulk update error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() }))
        }
    }
}

/// Metadata configuration routes
pub async fn get_metadata_config(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match MetadataConfig::get_config(&db.pool).await {
        Ok(config) => HttpResponse::Ok().json(config),
        Err(e) => {
            log::error!("Error getting metadata config: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_metadata_config(
    state: web::Data<AppState>,
    body: web::Json<UpdateMetadataConfigRequest>,
) -> HttpResponse {
    let db = &state.db;
       match MetadataConfig::update_config(&db.pool, body.into_inner()).await {
        Ok(config) => HttpResponse::Ok().json(config),
        Err(e) => {
            log::error!("Error updating metadata config: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn detect_bpm_handler(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;

    // Get the music file record to find the path
    let file = match music_service::get_music_file(db, id).await {
        Ok(Some(file)) => file,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({"error": "Music file not found"})),
        Err(e) => {
            log::error!("Error fetching music file {}: {}", id, e);
            return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}));
        }
    };

    // Run BPM detection
    // Note: This operation might take a few seconds, it's blocking if not careful.
    // Command::output blocks the thread, but it's okay for a short task in a handler if concurrency isn't massive.
    // For better perf, we should use tokio::process::Command or spawn_blocking.
    // Since bpm_service::detect_bpm uses std::process::Command, we should wrap it in spawn_blocking.
    
    let file_path = file.file_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        bpm_service::detect_bpm(&file_path)
    })
    .await;

    match result {
        Ok(Ok(bpm_res)) => {
            // Update the music file with the detected BPM, offset, and beat map
            let update_req = UpdateMusicFileRequest {
                bpm: Some(bpm_res.bpm),
                beat_grid_offset: Some(bpm_res.offset),
                beat_map: Some(serde_json::to_value(&bpm_res.beats).unwrap_or(serde_json::Value::Null)),
                ..Default::default()
            };
            
            match music_service::update_music_file(db, id, update_req).await {
                Ok(Some(updated_file)) => {
                    // Update cache
                    let _ = crate::services::cache_service::invalidate_all_tracks_cache(state.get_ref()).await;
                    let _ = crate::services::cache_service::notify_change(state.get_ref(), "music_updated", serde_json::to_value(&updated_file).unwrap_or(serde_json::Value::Null)).await;
                    HttpResponse::Ok().json(serde_json::json!({ "bpm": bpm_res.bpm, "offset": bpm_res.offset, "track": updated_file }))
                },
                Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "Music file not found during update"})),
                Err(e) => {
                    log::error!("Error update music file with BPM {}: {}", id, e);
                    HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
                }
            }
        }
        Ok(Err(e)) => {
            log::error!("BPM detection failed for {}: {}", file.id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
        Err(e) => {
            log::error!("Task join error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn reorder_playlist_tracks(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    payload: web::Json<ReorderPlaylistTracksRequest>,
) -> HttpResponse {
    let playlist_id = path.into_inner();
    let db = &state.db;
    match playlist_service::reorder_tracks(db, playlist_id, payload.music_file_ids.clone()).await {
        Ok(_) => {
            // Notify clients that tracks were reordered
            let payload_val = serde_json::json!({ "id": playlist_id });
            let _ = crate::services::cache_service::notify_change(
                state.get_ref(),
                "playlist_items_updated",
                payload_val
            ).await;
            HttpResponse::NoContent().finish()
        },
        Err(e) => {
            log::error!("Error reordering tracks for playlist {}: {}", playlist_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
