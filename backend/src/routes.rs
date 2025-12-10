use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use std::time::Instant;
use futures_util::StreamExt;
use sanitize_filename::sanitize;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::models::{
    AppState, CreateMusicFileRequest, CreatePlaylistRequest, CreateStreamRequest,
    CreateYoutubePlaylistRequest, DetectGenreRequest, DetectGenreResponse, MusicQueryParams,
    PlaylistTrackRequest, UpdateAutoDownloadConfigRequest, UpdateMusicFileRequest,
    UpdatePlaylistRequest, UpdateStreamRequest, UpdateYoutubePlaylistRequest,
    YoutubeDownloadRequest, YoutubeDownloadResponse,
};
use crate::services::auto_download_service;
use crate::services::backfill_manager;
use crate::services::reprocess_manager;
use crate::services::sync_manager;
use crate::services::{
    artist_service, file_sync_service, genre_cache_service, genre_detection, genre_label_service,
    internet_stream_service, music_service, playlist_service, yt_download_service,
    youtube_playlist_service,
};
use crate::yt_downloader::{self, DownloadOptions};

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
    match playlist_service::get_all_playlists(&db).await {
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
    match playlist_service::create_playlist(&db, &req.name, req.description.clone()).await {
        Ok(playlist) => HttpResponse::Created().json(playlist),
        Err(e) => {
            log::error!("Error creating playlist: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_playlist(state: web::Data<AppState>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();
    let db = &state.db;
    match playlist_service::get_playlist_with_items(&db, id).await {
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
    match playlist_service::delete_playlist(&db, id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
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
    match playlist_service::update_playlist(&db, id, payload.into_inner()).await {
        Ok(Some(playlist)) => HttpResponse::Ok().json(playlist),
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
    match playlist_service::add_track_to_playlist(&db, playlist_id, payload.into_inner()).await {
        Ok(_) => HttpResponse::Created().finish(),
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
    match playlist_service::remove_track_from_playlist(&db, playlist_id, track_id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            log::error!("Error removing track from playlist {}: {}", playlist_id, e);
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
    match music_service::get_all_music_files(&db, query.into_inner()).await {
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
    let db = &state.db;
    match music_service::create_music_file(&db, req.into_inner()).await {
        Ok(file) => HttpResponse::Created().json(file),
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

    match file_sync_service::sync_folder(&db, &folder).await {
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
            let mut rx = session.progress_tx.subscribe();
            drop(sessions_map);

            let stream = async_stream::stream! {
                while let Ok(progress) = rx.recv().await {
                    let data = serde_json::to_string(&progress).unwrap_or_default();
                    yield Ok::<_, actix_web::Error>(web::Bytes::from(format!("data: {}\n\n", data)));

                    if progress.progress == Some(100.0) || progress.is_cancelled == Some(true) {
                        break;
                    }
                }
            };

            return HttpResponse::Ok()
                .insert_header(("Content-Type", "text/event-stream"))
                .insert_header(("Cache-Control", "no-cache"))
                .insert_header(("Connection", "keep-alive"))
                .streaming(stream);
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
    match music_service::delete_music_file(&db, id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            log::error!("Error deleting music file: {}", e);
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
    match music_service::get_music_file(&db, id).await {
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
    match music_service::update_music_file(&db, id, payload.into_inner()).await {
        Ok(Some(file)) => HttpResponse::Ok().json(file),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            log::error!("Error updating music file {}: {}", id, e);
            HttpResponse::InternalServerError().finish()
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
    let record = music_service::get_music_file(&db, id).await.map_err(|e| {
        log::error!("Error getting music file {} for streaming: {}", id, e);
        actix_web::error::ErrorInternalServerError("db")
    })?;

    if let Some(file) = record {
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

    while let Some(field) = payload.next().await {
        match field {
            Ok(mut field) => {
                let filename = field
                    .content_disposition()
                    .and_then(|cd| cd.get_filename())
                    .map(|f| sanitize(f))
                    .unwrap_or_else(|| format!("upload-{}", chrono::Utc::now().timestamp()));

                let filepath = Path::new(&upload_root).join(&filename);
                match File::create(&filepath).await {
                    Ok(mut file) => {
                        while let Some(chunk) = field.next().await {
                            match chunk {
                                Ok(bytes) => {
                                    if let Err(e) = file.write_all(&bytes).await {
                                        errors.push(format!("Failed to write {}: {}", filename, e));
                                        break;
                                    }
                                }
                                Err(e) => {
                                    errors.push(format!("Chunk error for {}: {}", filename, e));
                                    break;
                                }
                            }
                        }

                        // Extract metadata from the uploaded file using lofty
                        let filepath_clone = filepath.clone();
                        let meta = tokio::task::spawn_blocking(move || {
                            use lofty::{Accessor, AudioFile, Probe, TaggedFileExt};
                            
                            let probed = Probe::open(&filepath_clone).and_then(|p| p.read());
                            match probed {
                                Ok(tagged) => {
                                    let tag = tagged.primary_tag();
                                    let properties = tagged.properties();

                                    let artist = tag.and_then(|t| t.artist()).map(|s| s.to_string());
                                    let title = tag.and_then(|t| t.title()).map(|s| s.to_string());
                                    let album = tag.and_then(|t| t.album()).map(|s| s.to_string());
                                    let duration_ms = Some(properties.duration().as_millis() as i32);
                                    (artist, title, album, duration_ms)
                                }
                                Err(e) => {
                                    log::warn!("Failed to read metadata from {}: {}", filepath_clone.display(), e);
                                    (None, None, None, None)
                                }
                            }
                        })
                        .await
                        .unwrap_or((None, None, None, None));

                        let (artist_opt, title_opt, album_opt, duration_ms) = meta;

                        // Fallback to filename parsing when metadata missing
                        let file_name = Path::new(&filename)
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or_default();
                        let (file_artist, file_title) = parse_filename_for_upload(file_name);

                        // Use metadata artist if available and not empty, otherwise use parsed artist
                        let artist = artist_opt
                            .filter(|s| !s.trim().is_empty())
                            .or_else(|| file_artist.map(|s| s.to_string()));
                        
                        // Use metadata title if available and not empty, otherwise use parsed title
                        let mut title = title_opt
                            .filter(|s| !s.trim().is_empty())
                            .unwrap_or_else(|| file_title.to_string());
                        
                        // If title contains "Artist - Title" pattern and we have an artist, extract just the title
                        if let Some(ref artist_name) = artist {
                            // Check if title starts with "Artist - " pattern
                            let artist_prefix = format!("{} - ", artist_name);
                            if title.starts_with(&artist_prefix) {
                                title = title[artist_prefix.len()..].to_string();
                            }
                            // Also check case-insensitive
                            else if title.to_lowercase().starts_with(&artist_prefix.to_lowercase()) {
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
                                    // File is a duplicate - delete the uploaded file and skip
                                    let _ = tokio::fs::remove_file(&filepath).await;
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
                        };

                        match music_service::create_music_file(&db, req).await {
                            Ok(record) => {
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
                                
                                inserted.push(record);
                            }
                            Err(e) => {
                                errors.push(format!("DB insert failed for {}: {}", filename, e));
                            }
                        }
                    }
                    Err(e) => {
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

    HttpResponse::Ok().json(serde_json::json!({
        "inserted": inserted,
        "errors": errors,
    }))
}

/// Parse filename to extract artist and title (used for uploads)
fn parse_filename_for_upload(file_name: &str) -> (Option<&str>, &str) {
    // Try to split on " - " to extract artist and title
    if let Some(pos) = file_name.find(" - ") {
        let artist = &file_name[..pos];
        let mut title = &file_name[(pos + 3)..];
        // strip extension
        if let Some(dot) = title.rfind('.') {
            title = &title[..dot];
        }
        (Some(artist.trim()), title.trim())
    } else {
        // No artist, strip extension and return None for artist
        let mut title = file_name;
        if let Some(dot) = title.rfind('.') {
            title = &title[..dot];
        }
        (None, title.trim())
    }
}

pub async fn list_streams(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match internet_stream_service::list_streams(&db).await {
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
    match internet_stream_service::create_stream(&db, payload.into_inner()).await {
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
    match internet_stream_service::update_stream(&db, id, payload.into_inner()).await {
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
    match internet_stream_service::delete_stream(&db, id).await {
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
    match artist_service::get_all_artists_with_summary(&db).await {
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
    
    match artist_service::get_music_featuring_artist(&db, &decoded_name).await {
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
    
    match artist_service::set_artist_genre(&db, &decoded_name, &payload.genre).await {
        Ok(()) => {
            // Also update guessed_genre for all their songs
            let _ = artist_service::update_guessed_genre_for_artist(&db, &decoded_name, &payload.genre).await;
            HttpResponse::Ok().json(serde_json::json!({"success": true}))
        }
        Err(e) => {
            log::error!("Error setting genre for artist {}: {}", decoded_name, e);
            HttpResponse::InternalServerError().finish()
        }
    }
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
    
    match artist_service::rename_artist(&db, &decoded_old_name, new_name).await {
        Ok(result) => HttpResponse::Ok().json(result),
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

    match yt_downloader::download_youtube_playlist(url, output_dir, Some(options), sessions, pool)
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
        let mut rx = session.progress_tx.subscribe();
        drop(sessions); // Release the lock

        let stream = async_stream::stream! {
            while let Ok(progress) = rx.recv().await {
                let data = serde_json::to_string(&progress).unwrap_or_default();
                yield Ok::<_, actix_web::Error>(web::Bytes::from(format!("data: {}\n\n", data)));

                // Break if download is complete or cancelled
                if progress.progress == Some(100.0) || progress.is_cancelled == Some(true) {
                    break;
                }
            }
        };

        HttpResponse::Ok()
            .insert_header(("Content-Type", "text/event-stream"))
            .insert_header(("Cache-Control", "no-cache"))
            .insert_header(("Connection", "keep-alive"))
            .streaming(stream)
    } else {
        HttpResponse::NotFound().json(serde_json::json!({"error": "Session not found"}))
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
            total_downloaded: count,
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

// YouTube Playlist management routes (for saving playlist links)
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
    
    match yt_downloader::download_youtube_playlist(
        playlist.url.clone(),
        "/music/downloads".to_string(),
        Some(options),
        sessions,
        pool,
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
    match genre_detection::detect_genre_for_artist(&db, req.artist_name.clone()).await {
        Ok(genre) => {
            let cached = genre_cache_service::is_cached(&db, &req.artist_name)
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
    match genre_cache_service::get_all_cached_genres(&db).await {
        Ok(cache) => HttpResponse::Ok().json(cache),
        Err(e) => {
            log::error!("Error fetching genre cache: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn clear_genre_cache(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match genre_cache_service::clear_all_cache(&db).await {
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
    match genre_label_service::list_genres_with_counts(&db).await {
        Ok(genres) => HttpResponse::Ok().json(genres),
        Err(e) => {
            log::error!("Error listing genres: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// List canonical genres (admin-defined genre taxonomy)
pub async fn list_canonical_genres(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match genre_label_service::list_genres(&db).await {
        Ok(genres) => HttpResponse::Ok().json(genres),
        Err(e) => {
            log::error!("Error listing canonical genres: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn create_genre(
    state: web::Data<AppState>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let name = body
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let description = body
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    if name.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Missing name"}));
    }

    match genre_label_service::create_genre(&db, &name.unwrap(), description.as_deref()).await {
        Ok(g) => HttpResponse::Created().json(g),
        Err(e) => {
            log::error!("Error creating genre: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn add_genre_alias(
    state: web::Data<AppState>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let alias = body
        .get("alias")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let genre_id = body
        .get("genre_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok());

    if alias.is_none() || genre_id.is_none() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Missing alias or genre_id"}));
    }

    match genre_label_service::add_alias(&db, &alias.unwrap(), genre_id.unwrap()).await {
        Ok(_) => HttpResponse::Created().json(serde_json::json!({"ok": true})),
        Err(e) => {
            log::error!("Error adding genre alias: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn suggest_genre_matches(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let raw = path.into_inner();
    let db = &state.db;
    match genre_label_service::suggest_similar(&db, &raw, 10).await {
        Ok(suggestions) => HttpResponse::Ok().json(suggestions),
        Err(e) => {
            log::error!("Error suggesting genres: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn add_genre_alias_and_backfill(
    state: web::Data<AppState>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let alias = body
        .get("alias")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let genre_id = body
        .get("genre_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok());

    if alias.is_none() || genre_id.is_none() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Missing alias or genre_id"}));
    }

    // resolve canonical name for backfill
    let canonical = match genre_label_service::list_genres(&db).await {
        Ok(list) => list
            .into_iter()
            .find(|g| g.id == genre_id.unwrap())
            .map(|g| g.name),
        Err(_) => None,
    };

    if canonical.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Genre id not found"}));
    }

    match genre_label_service::add_alias(&db, &alias.clone().unwrap(), genre_id.unwrap()).await {
        Ok(_) => {
            // backfill
            match genre_label_service::backfill_alias(&db, &alias.unwrap(), &canonical.unwrap())
                .await
            {
                Ok(updated) => HttpResponse::Ok().json(serde_json::json!({"backfilled": updated})),
                Err(e) => {
                    log::error!("Backfill error: {}", e);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
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
    let alias = path.into_inner();
    let db = &state.db;
    match genre_label_service::preview_backfill(&db, &alias).await {
        Ok((music_count, artist_count)) => HttpResponse::Ok()
            .json(serde_json::json!({"music_rows": music_count, "artist_rows": artist_count})),
        Err(e) => {
            log::error!("Error previewing backfill: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn start_backfill_handler(
    state: web::Data<AppState>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let alias = body
        .get("alias")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let genre_id = body
        .get("genre_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok());
    if alias.is_none() || genre_id.is_none() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Missing alias or genre_id"}));
    }

    let db = state.db.clone();
    let session_id = backfill_manager::start_backfill(db, alias.unwrap(), genre_id.unwrap()).await;
    HttpResponse::Accepted().json(serde_json::json!({"session_id": session_id}))
}

pub async fn backfill_progress_stream(path: web::Path<String>) -> HttpResponse {
    let session_id = path.into_inner();
    if let Some(mut rx) = backfill_manager::subscribe(&session_id).await {
        let stream = async_stream::stream! {
            while let Ok(progress) = rx.recv().await {
                let data = serde_json::to_string(&progress).unwrap_or_default();
                yield Ok::<_, actix_web::Error>(web::Bytes::from(format!("data: {}\n\n", data)));
                if progress.finished {
                    break;
                }
            }
        };

        return HttpResponse::Ok()
            .insert_header(("Content-Type", "text/event-stream"))
            .insert_header(("Cache-Control", "no-cache"))
            .insert_header(("Connection", "keep-alive"))
            .streaming(stream);
    }

    HttpResponse::NotFound().json(serde_json::json!({"error": "Session not found"}))
}

/// Start a background job to re-run detection for artists missing cached/canonical genres
pub async fn reprocess_missing_genres(state: web::Data<AppState>) -> HttpResponse {
    let db = state.db.clone();
    let session_id = reprocess_manager::start_reprocess(db).await;
    HttpResponse::Accepted().json(serde_json::json!({"session_id": session_id}))
}

pub async fn reprocess_progress_stream(path: web::Path<String>) -> HttpResponse {
    let session_id = path.into_inner();
    if let Some(mut rx) = reprocess_manager::subscribe(&session_id).await {
        let stream = async_stream::stream! {
            while let Ok(progress) = rx.recv().await {
                let data = serde_json::to_string(&progress).unwrap_or_default();
                yield Ok::<_, actix_web::Error>(web::Bytes::from(format!("data: {}\n\n", data)));
                if progress.finished {
                    break;
                }
            }
        };

        return HttpResponse::Ok()
            .insert_header(("Content-Type", "text/event-stream"))
            .insert_header(("Cache-Control", "no-cache"))
            .insert_header(("Connection", "keep-alive"))
            .streaming(stream);
    }

    HttpResponse::NotFound().json(serde_json::json!({"error": "Session not found"}))
}

pub async fn list_unmapped_genres(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match genre_label_service::list_unmapped_tags(&db, 100).await {
        Ok(tags) => HttpResponse::Ok().json(tags),
        Err(e) => {
            log::error!("Error listing unmapped tags: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Auto-download config routes
pub async fn get_auto_download_config(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match auto_download_service::get_config(db).await {
        Ok(config) => HttpResponse::Ok().json(config),
        Err(e) => {
            log::error!("Error getting auto-download config: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

pub async fn update_auto_download_config(
    state: web::Data<AppState>,
    body: web::Json<UpdateAutoDownloadConfigRequest>,
) -> HttpResponse {
    let db = &state.db;
    match auto_download_service::update_config(db, body.into_inner()).await {
        Ok(config) => HttpResponse::Ok().json(config),
        Err(e) => {
            log::error!("Error updating auto-download config: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

pub async fn get_auto_download_status(state: web::Data<AppState>) -> HttpResponse {
    let auto_state = &state.auto_download_state;
    let current_playlist = auto_state.current_playlist.read().await.clone();
    
    HttpResponse::Ok().json(serde_json::json!({
        "is_running": auto_state.is_running.load(std::sync::atomic::Ordering::Relaxed),
        "current_playlist": current_playlist,
        "downloads_completed": auto_state.downloads_completed.load(std::sync::atomic::Ordering::Relaxed),
        "downloads_skipped": auto_state.downloads_skipped.load(std::sync::atomic::Ordering::Relaxed),
        "downloads_in_progress": auto_state.downloads_in_progress.load(std::sync::atomic::Ordering::Relaxed),
    }))
}

pub async fn trigger_auto_download(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    let sessions = &state.download_sessions;
    let auto_state = state.auto_download_state.clone();
    
    match auto_download_service::trigger_now(&db.pool, sessions, auto_state).await {
        Ok(msg) => HttpResponse::Accepted().json(serde_json::json!({"message": msg})),
        Err(e) => {
            log::error!("Error triggering auto-download: {}", e);
            HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
        }
    }
}

pub async fn stop_auto_download(state: web::Data<AppState>) -> HttpResponse {
    auto_download_service::stop_current_run(&state.auto_download_state);
    HttpResponse::Ok().json(serde_json::json!({"message": "Stop signal sent"}))
}
