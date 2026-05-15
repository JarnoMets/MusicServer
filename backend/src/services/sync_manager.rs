use crate::db::Database;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;
use once_cell::sync::OnceCell;

/// Maximum files per batch insert to balance memory and performance
const INSERT_BATCH_SIZE: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgress {
    pub status: String,
    pub progress: Option<f32>,
    pub current_file: Option<String>,
    pub total_files: Option<u32>,
    pub completed_files: Option<u32>,
    pub inserted_files: Option<u32>,
    pub failed_files: Option<u32>,
    pub is_cancelled: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct SyncSession {
    #[allow(dead_code)]
    pub id: String,
    pub cancelled: Arc<AtomicBool>,
    pub progress: Arc<RwLock<SyncProgress>>,
    pub progress_tx: broadcast::Sender<SyncProgress>,
}

pub type SyncSessions = Arc<RwLock<HashMap<String, SyncSession>>>;

pub fn create_sync_sessions() -> SyncSessions {
    let s = Arc::new(RwLock::new(HashMap::new()));
    SYNC_SESSIONS.set(s.clone()).ok();
    s
}

static SYNC_SESSIONS: OnceCell<SyncSessions> = OnceCell::new();

pub fn get_sync_sessions() -> Option<SyncSessions> {
    SYNC_SESSIONS.get().cloned()
}

pub async fn cancel_session(session_id: &str) -> bool {
    if let Some(sessions) = get_sync_sessions() {
        let map = sessions.write().await;
        if let Some(sess) = map.get(session_id) {
            sess.cancelled.store(true, Ordering::Relaxed);
            return true;
        }
    }
    false
}

async fn send_progress(session: &SyncSession, progress: SyncProgress) {
    {
        let mut stored = session.progress.write().await;
        *stored = progress.clone();
    }
    let _ = session.progress_tx.send(progress);
}

/// Row data for batch insert - kept minimal to reduce memory
struct FileRow {
    id: uuid::Uuid,
    title: String,
    artist: Option<String>,
    album: Option<String>,
    release_date: Option<chrono::DateTime<chrono::Utc>>,
    duration_ms: Option<i32>,
    track_num: Option<i32>,
    file_path: String,
}

/// Extract metadata from a file path using shared helper
async fn extract_file_metadata(path: &std::path::Path) -> (Option<String>, Option<String>, Option<String>, Option<i32>, Option<i32>, Option<i32>) {
    let meta = crate::services::metadata_extractor::extract_metadata(path).await;
    (meta.artist, meta.title, meta.album, meta.track_number, meta.year, meta.duration_ms)
}

/// Parse title from filename when metadata is missing
fn title_from_filename(path: &std::path::Path) -> String {
    let fname = path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let (_artist, title) = crate::services::filename_parser::parse_filename(fname);
    title
}

/// Insert a batch of rows into the database
async fn insert_batch(db: &Database, rows: &[FileRow]) -> Result<u32, sqlx::Error> {
    if rows.is_empty() {
        return Ok(0);
    }

    use sqlx::{Postgres, QueryBuilder};
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "INSERT INTO music_files (id, title, artist, album, genre_id, genre_source, release_date, duration, file_path, created_at, updated_at, track_number) "
    );
    
    let now = chrono::Utc::now();
    qb.push_values(rows.iter(), |mut b, row| {
        b.push_bind(row.id)
            .push_bind(&row.title)
            .push_bind(row.artist.clone())
            .push_bind(row.album.clone())
            .push_bind::<Option<String>>(None)
            .push_bind::<Option<String>>(None)
            .push_bind(row.release_date)
            .push_bind(row.duration_ms)
            .push_bind(&row.file_path)
            .push_bind(now)
            .push_bind(now)
            .push_bind(row.track_num);
    });

    let result = qb.build().execute(&db.pool).await?;
    Ok(result.rows_affected() as u32)
}

/// Spawn genre detection tasks for inserted rows (non-blocking)
fn spawn_genre_detection(db: Database, rows: Vec<FileRow>) {
    for row in rows {
        if let Some(artist_name) = row.artist {
            let db_clone = db.clone();
            let artist_name_clone = artist_name.clone();
            tokio::spawn(async move {
                // Check cache first
                if let Ok(Some(genre_id)) = crate::services::genre_cache_service::get_cached_genre_id(&db_clone, &artist_name_clone).await {
                    let _ = crate::services::genre_label_service::assign_genre_to_artist_tracks(&db_clone, &artist_name_clone, genre_id).await;
                } else {
                    // Detect and propagate
                    if let Ok(Some(genre_id)) = crate::services::genre_detection::detect_genre_for_artist(&db_clone, artist_name_clone.clone()).await {
                        let _ = crate::services::genre_label_service::assign_genre_to_artist_tracks(&db_clone, &artist_name_clone, genre_id).await;
                    }
                }
            });
        }
    }
}

/// Start a background sync. Returns session id or error string.
pub async fn start_sync(
    db: Database,
    folder: String,
    sessions: SyncSessions,
) -> Result<String, String> {
    let session_id = Uuid::new_v4().to_string();
    let cancelled = Arc::new(AtomicBool::new(false));
    let progress = Arc::new(RwLock::new(SyncProgress {
        status: "Initializing...".to_string(),
        progress: Some(0.0),
        current_file: None,
        total_files: None,
        completed_files: Some(0),
        inserted_files: Some(0),
        failed_files: Some(0),
        is_cancelled: Some(false),
    }));

    let (progress_tx, _) = broadcast::channel(32);

    let session = SyncSession {
        id: session_id.clone(),
        cancelled: cancelled.clone(),
        progress: progress.clone(),
        progress_tx,
    };
    
    {
        let mut map = sessions.write().await;
        map.insert(session_id.clone(), session.clone());
    }

    let session_id_for_spawn = session_id.clone();

    tokio::spawn(async move {
        // Phase 1: Count files first (lightweight scan)
        send_progress(&session, SyncProgress {
            status: "Scanning folder...".to_string(),
            progress: Some(0.0),
            current_file: None,
            total_files: None,
            completed_files: Some(0),
            inserted_files: Some(0),
            failed_files: Some(0),
            is_cancelled: Some(false),
        }).await;

        let mut file_count: u32 = 0;
        let mut stack = vec![PathBuf::from(&folder)];
        
        // First pass: just count files
        while let Some(dir_path) = stack.pop() {
            if let Ok(mut rd) = tokio::fs::read_dir(&dir_path).await {
                while let Ok(Some(entry)) = rd.next_entry().await {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                    } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if matches!(ext.to_lowercase().as_str(), "mp3" | "flac" | "m4a" | "wav" | "ogg" | "aac") {
                            file_count += 1;
                        }
                    }
                }
            }
        }

        send_progress(&session, SyncProgress {
            status: format!("Found {} files, starting sync", file_count),
            progress: Some(0.0),
            current_file: None,
            total_files: Some(file_count),
            completed_files: Some(0),
            inserted_files: Some(0),
            failed_files: Some(0),
            is_cancelled: Some(false),
        }).await;

        let mut inserted: u32 = 0;
        let mut failed: u32 = 0;
        let mut completed: u32 = 0;
        let mut batch: Vec<FileRow> = Vec::with_capacity(INSERT_BATCH_SIZE);
        
        // Phase 2: Process files in streaming fashion
        stack = vec![PathBuf::from(&folder)];
        
        while let Some(dir_path) = stack.pop() {
            if cancelled.load(Ordering::Relaxed) {
                send_progress(&session, SyncProgress {
                    status: "Sync cancelled".to_string(),
                    progress: None,
                    current_file: None,
                    total_files: Some(file_count),
                    completed_files: Some(completed),
                    inserted_files: Some(inserted),
                    failed_files: Some(failed),
                    is_cancelled: Some(true),
                }).await;
                return;
            }

            if let Ok(mut rd) = tokio::fs::read_dir(&dir_path).await {
                while let Ok(Some(entry)) = rd.next_entry().await {
                    if cancelled.load(Ordering::Relaxed) {
                        break;
                    }
                    
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                        continue;
                    }
                    
                    let ext = path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase());
                    if !matches!(ext.as_deref(), Some("mp3" | "flac" | "m4a" | "wav" | "ogg" | "aac")) {
                        continue;
                    }

                    let path_str = path.to_string_lossy().to_string();
                    
                    // Check if file already exists in DB
                    let exists: bool = sqlx::query_scalar::<_, bool>(
                        "SELECT EXISTS(SELECT 1 FROM music_files WHERE file_path = $1)"
                    )
                    .bind(&path_str)
                    .fetch_one(&db.pool)
                    .await
                    .unwrap_or(false);

                    completed += 1;
                    
                    if exists {
                        let progress_pct = (completed as f32 / file_count as f32) * 100.0;
                        send_progress(&session, SyncProgress {
                            status: "Skipping existing file".to_string(),
                            progress: Some(progress_pct),
                            current_file: Some(path_str),
                            total_files: Some(file_count),
                            completed_files: Some(completed),
                            inserted_files: Some(inserted),
                            failed_files: Some(failed),
                            is_cancelled: Some(false),
                        }).await;
                        continue;
                    }

                    // Extract metadata
                    let (artist, title_opt, album, track, year, duration_ms) = extract_file_metadata(&path).await;
                    let title = title_opt.unwrap_or_else(|| title_from_filename(&path));
                    
                    let release_date = year.and_then(|y| {
                        chrono::NaiveDate::from_ymd_opt(y, 1, 1)
                            .and_then(|d| d.and_hms_opt(0, 0, 0))
                            .map(|ndt| chrono::DateTime::from_naive_utc_and_offset(ndt, chrono::Utc))
                    });

                    batch.push(FileRow {
                        id: uuid::Uuid::new_v4(),
                        title,
                        artist,
                        album,
                        release_date,
                        duration_ms,
                        track_num: track,
                        file_path: path_str.clone(),
                    });

                    // Insert batch when full
                    if batch.len() >= INSERT_BATCH_SIZE {
                        match insert_batch(&db, &batch).await {
                            Ok(count) => {
                                inserted += count;
                                // Spawn genre detection in background
                                spawn_genre_detection(db.clone(), std::mem::take(&mut batch));
                            }
                            Err(e) => {
                                log::error!("Batch insert failed: {}", e);
                                failed += batch.len() as u32;
                                batch.clear();
                            }
                        }
                    }

                    let progress_pct = (completed as f32 / file_count as f32) * 100.0;
                    send_progress(&session, SyncProgress {
                        status: "Sync in progress".to_string(),
                        progress: Some(progress_pct),
                        current_file: Some(path_str),
                        total_files: Some(file_count),
                        completed_files: Some(completed),
                        inserted_files: Some(inserted),
                        failed_files: Some(failed),
                        is_cancelled: Some(false),
                    }).await;
                }
            }
        }

        // Insert remaining batch
        if !batch.is_empty() {
            match insert_batch(&db, &batch).await {
                Ok(count) => {
                    inserted += count;
                    spawn_genre_detection(db.clone(), batch);
                }
                Err(e) => {
                    log::error!("Final batch insert failed: {}", e);
                    failed += batch.len() as u32;
                }
            }
        }

        // Send completion progress
        send_progress(&session, SyncProgress {
            status: "Sync completed".to_string(),
            progress: Some(100.0),
            current_file: None,
            total_files: Some(file_count),
            completed_files: Some(completed),
            inserted_files: Some(inserted),
            failed_files: Some(failed),
            is_cancelled: Some(false),
        }).await;

        // Cleanup session after delay
        let sessions_cleanup = sessions.clone();
        let session_id_for_cleanup = session_id_for_spawn.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            let mut map = sessions_cleanup.write().await;
            map.remove(&session_id_for_cleanup);
        });
    });

    Ok(session_id)
}
