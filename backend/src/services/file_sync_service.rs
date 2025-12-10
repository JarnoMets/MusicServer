use crate::db::Database;
use crate::models::CreateMusicFileRequest;
use crate::services::artist_parser;
use crate::services::music_service;
use lofty::{Accessor, AudioFile, Probe, TaggedFileExt};
use std::path::Path;
use tokio::fs;
use tokio::io;

/// Sync all files in the provided folder (recursively) into the database.
/// For each file found, if there is no matching music_files.file_path in the DB, insert a new record.
/// Returns the number of inserted records.
pub async fn sync_folder(db: &Database, folder: &str) -> Result<u64, sqlx::Error> {
    let mut inserted: u64 = 0;
    let mut entries = collect_files(folder)
        .await
        .map_err(|_| sqlx::Error::RowNotFound)?;

    for path in entries.drain(..) {
        // Normalize path to string
        if let Some(path_str) = path.to_str() {
            // Check if exists in DB
            let exists: Option<String> =
                sqlx::query_scalar("SELECT file_path FROM music_files WHERE file_path = $1")
                    .bind(path_str)
                    .fetch_optional(&db.pool)
                    .await?;

            if exists.is_none() {
                // Attempt to read tags with lofty in a blocking task
                let path_clone = path.clone();
                let meta = tokio::task::spawn_blocking(move || {
                    // Use Probe to read the file
                    let probed = Probe::open(path_clone).and_then(|p| p.read());
                    match probed {
                        Ok(tagged) => {
                            let tag = tagged.primary_tag();
                            let properties = tagged.properties();

                            let artist = tag.and_then(|t| t.artist()).map(|s| s.to_string());
                            let title = tag.and_then(|t| t.title()).map(|s| s.to_string());
                            let album = tag.and_then(|t| t.album()).map(|s| s.to_string());
                            let duration_opt = Some(properties.duration());
                            (artist, title, album, duration_opt)
                        }
                        Err(_) => (None, None, None, None),
                    }
                })
                .await
                .unwrap_or((None, None, None, None));

                let (artist_opt, title_opt, album_opt, duration_opt) = meta;

                // Fallback to filename parsing when metadata missing
                let file_name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default();
                let (file_artist, file_title) = parse_filename(file_name);

                let artist = artist_opt.or_else(|| file_artist.map(|s| s.to_string()));
                let title = title_opt
                    .or_else(|| Some(file_title.to_string()))
                    .unwrap_or_else(|| file_name.to_string());
                let album = album_opt.or_else(|| None);

                let duration_ms = duration_opt.map(|d| d.as_millis() as i32);

                // Compute file hash for duplicate detection
                let file_hash = match music_service::compute_file_hash(path_str).await {
                    Ok(hash) => Some(hash),
                    Err(e) => {
                        log::warn!("Failed to compute file hash for {}: {}", path_str, e);
                        None
                    }
                };

                // Check for duplicates by hash
                if let Some(ref hash) = file_hash {
                    match music_service::is_duplicate_hash(db, hash).await {
                        Ok(true) => {
                            log::info!("Skipping duplicate file (by hash): {}", path_str);
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
                    album,
                    genre: None,
                    guessed_genre: None,
                    release_date: None,
                    duration: duration_ms,
                    file_path: path_str.to_string(),
                    track_number: None,
                    file_hash,
                };

                // Insert using music_service
                match music_service::create_music_file(db, req).await {
                    Ok(file) => {
                        inserted += 1;
                        
                        // Parse all artists from artist field and title
                        let parsed = artist_parser::parse_artists(
                            file.artist.as_deref(),
                            Some(&file.title),
                        );
                        
                        // Ensure all extracted artists exist in artist_genres
                        let all_artists = parsed.all_artists();
                        if let Err(e) = crate::services::artist_service::ensure_artists_exist(db, &all_artists).await {
                            log::warn!("Failed to ensure artists exist: {}", e);
                        }
                        
                        // After insertion, check genre cache and detect genre if needed
                        let db_clone = db;
                        let artist_for_lookup = file.artist.clone();
                        if let Some(artist_name) = artist_for_lookup {
                            // If artist has cached genre, set guessed_genre accordingly; otherwise detect and cache
                            match crate::services::genre_cache_service::get_cached_genre(
                                db_clone,
                                &artist_name,
                            )
                            .await
                            {
                                Ok(Some(cached_genre)) => {
                                    // update guessed_genre
                                    let _ = crate::services::artist_service::update_guessed_genre_for_artist(db_clone, &artist_name, &cached_genre).await;
                                }
                                Ok(None) => {
                                    // perform detection
                                    match crate::services::genre_detection::detect_genre_for_artist(
                                        db_clone,
                                        artist_name.clone(),
                                    )
                                    .await
                                    {
                                        Ok(Some(detected)) => {
                                            let _ = crate::services::artist_service::update_guessed_genre_for_artist(db_clone, &artist_name, &detected).await;
                                        }
                                        _ => {}
                                    }
                                }
                                Err(e) => {
                                    log::warn!("Error checking genre cache: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to insert file {}: {}", path_str, e);
                    }
                }
            }
        }
    }

    Ok(inserted)
}

async fn collect_files(folder: &str) -> io::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![std::path::PathBuf::from(folder)];

    while let Some(dir_path) = stack.pop() {
        let read_dir = fs::read_dir(&dir_path).await;
        if let Ok(mut rd) = read_dir {
            while let Ok(Some(entry)) = rd.next_entry().await {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if is_audio_file(&path) {
                    files.push(path);
                }
            }
        } else {
            log::warn!("Failed to read directory: {}", dir_path.display());
        }
    }

    Ok(files)
}

fn is_audio_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(
            ext.to_lowercase().as_str(),
            "mp3" | "flac" | "m4a" | "wav" | "ogg" | "aac"
        )
    } else {
        false
    }
}

fn parse_filename(file_name: &str) -> (Option<&str>, &str) {
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
