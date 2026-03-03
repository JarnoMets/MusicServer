use crate::db::Database;
use crate::models::{AutoDownloadConfig, UpdateAutoDownloadConfigRequest, YoutubePlaylist};
use crate::yt_downloader::{self, DownloadOptions, DownloadSessions};
use chrono::{Timelike, Utc};
use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

/// Shared state for the auto-download scheduler
pub struct AutoDownloadState {
    pub is_running: Arc<AtomicBool>,
    pub current_playlist: Arc<RwLock<Option<String>>>,
    pub downloads_in_progress: Arc<AtomicI32>,
    pub downloads_completed: Arc<AtomicI32>,
    pub downloads_skipped: Arc<AtomicI32>,
    pub should_stop: Arc<AtomicBool>,
}

impl AutoDownloadState {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            current_playlist: Arc::new(RwLock::new(None)),
            downloads_in_progress: Arc::new(AtomicI32::new(0)),
            downloads_completed: Arc::new(AtomicI32::new(0)),
            downloads_skipped: Arc::new(AtomicI32::new(0)),
            should_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn reset_counters(&self) {
        self.downloads_completed.store(0, Ordering::Relaxed);
        self.downloads_skipped.store(0, Ordering::Relaxed);
    }
}

impl Default for AutoDownloadState {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the auto-download configuration (there's only one row)
pub async fn get_config(db: &Database) -> Result<AutoDownloadConfig, sqlx::Error> {
    sqlx::query_as::<_, AutoDownloadConfig>(
        r#"
        SELECT id, enabled, check_interval_minutes, max_concurrent_downloads, 
               delay_between_downloads_seconds, allowed_start_hour, allowed_end_hour,
               last_check_at, next_check_at, created_at, updated_at
        FROM auto_download_config
        LIMIT 1
        "#
    )
    .fetch_one(&db.pool)
    .await
}

/// Update the auto-download configuration
pub async fn update_config(
    db: &Database,
    req: UpdateAutoDownloadConfigRequest,
) -> Result<AutoDownloadConfig, sqlx::Error> {
    // Get current config to get the ID
    let current = get_config(db).await?;
    
    let enabled = req.enabled.unwrap_or(current.enabled);
    let check_interval = req.check_interval_minutes.unwrap_or(current.check_interval_minutes);
    let max_concurrent = req.max_concurrent_downloads.unwrap_or(current.max_concurrent_downloads);
    let delay = req.delay_between_downloads_seconds.unwrap_or(current.delay_between_downloads_seconds);
    
    // Calculate next check time based on new interval
    let next_check = if enabled {
        Some(Utc::now() + chrono::Duration::minutes(check_interval as i64))
    } else {
        None
    };

    sqlx::query_as::<_, AutoDownloadConfig>(
        r#"
        UPDATE auto_download_config 
        SET enabled = $1, 
            check_interval_minutes = $2, 
            max_concurrent_downloads = $3,
            delay_between_downloads_seconds = $4,
            allowed_start_hour = $5,
            allowed_end_hour = $6,
            next_check_at = $7,
            updated_at = NOW()
        WHERE id = $8
        RETURNING id, enabled, check_interval_minutes, max_concurrent_downloads,
                  delay_between_downloads_seconds, allowed_start_hour, allowed_end_hour,
                  last_check_at, next_check_at, created_at, updated_at
        "#
    )
    .bind(enabled)
    .bind(check_interval)
    .bind(max_concurrent)
    .bind(delay)
    .bind(req.allowed_start_hour.or(current.allowed_start_hour))
    .bind(req.allowed_end_hour.or(current.allowed_end_hour))
    .bind(next_check)
    .bind(current.id)
    .fetch_one(&db.pool)
    .await
}

/// Update last check timestamp
async fn update_last_check(pool: &PgPool, next_interval_minutes: i32) -> Result<(), sqlx::Error> {
    let next_check = Utc::now() + chrono::Duration::minutes(next_interval_minutes as i64);
    
    sqlx::query(
        r#"
        UPDATE auto_download_config 
        SET last_check_at = NOW(), next_check_at = $1, updated_at = NOW()
        "#
    )
    .bind(next_check)
    .execute(pool)
    .await?;
    
    Ok(())
}

/// Check if current time is within allowed window
fn is_within_allowed_window(config: &AutoDownloadConfig) -> bool {
    match (config.allowed_start_hour, config.allowed_end_hour) {
        (Some(start), Some(end)) => {
            let current_hour = Utc::now().hour() as i32;
            if start <= end {
                // Normal range, e.g., 9-17
                current_hour >= start && current_hour < end
            } else {
                // Overnight range, e.g., 22-6
                current_hour >= start || current_hour < end
            }
        }
        _ => true, // No restriction
    }
}

/// Start the auto-download background scheduler
pub fn start_scheduler(
    pool: PgPool,
    download_sessions: DownloadSessions,
    state: Arc<AutoDownloadState>,
    update_tx: tokio::sync::broadcast::Sender<serde_json::Value>,
) {
    tokio::spawn(async move {
        log::info!("Auto-download scheduler started");
        
        // Check every minute if we should run
        let mut check_interval = interval(Duration::from_secs(60));
        
        loop {
            check_interval.tick().await;
            
            if state.should_stop.load(Ordering::Relaxed) {
                log::info!("Auto-download scheduler stopping");
                break;
            }
            
            // Skip if already running
            if state.is_running.load(Ordering::Relaxed) {
                continue;
            }
            
            // Get config
            let config = match get_config_from_pool(&pool).await {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to get auto-download config: {}", e);
                    continue;
                }
            };
            
            // Check if enabled
            if !config.enabled {
                continue;
            }
            
            // Check if it's time to run
            let should_run = match config.next_check_at {
                Some(next) => Utc::now() >= next,
                None => true, // Never run before
            };
            
            if !should_run {
                continue;
            }
            
            // Check time window
            if !is_within_allowed_window(&config) {
                log::debug!("Auto-download: outside allowed time window");
                continue;
            }
            
            // Run the auto-download process
            log::info!("Starting auto-download check");
            state.is_running.store(true, Ordering::Relaxed);
            state.reset_counters();
            
            if let Err(e) = run_auto_download(
                &pool,
                &download_sessions,
                &config,
                state.clone(),
                Some(update_tx.clone()),
            ).await {
                log::error!("Auto-download error: {}", e);
            }
            
            // Update last check time
            if let Err(e) = update_last_check(&pool, config.check_interval_minutes).await {
                log::error!("Failed to update last check time: {}", e);
            }
            
            state.is_running.store(false, Ordering::Relaxed);
            *state.current_playlist.write().await = None;
            
            log::info!(
                "Auto-download check complete: {} completed, {} skipped",
                state.downloads_completed.load(Ordering::Relaxed),
                state.downloads_skipped.load(Ordering::Relaxed)
            );
        }
    });
}

/// Helper to get config directly from pool
async fn get_config_from_pool(pool: &PgPool) -> Result<AutoDownloadConfig, sqlx::Error> {
    sqlx::query_as::<_, AutoDownloadConfig>(
        r#"
        SELECT id, enabled, check_interval_minutes, max_concurrent_downloads, 
               delay_between_downloads_seconds, allowed_start_hour, allowed_end_hour,
               last_check_at, next_check_at, created_at, updated_at
        FROM auto_download_config
        LIMIT 1
        "#
    )
    .fetch_one(pool)
    .await
}

/// Run the auto-download process for all enabled playlists
async fn run_auto_download(
    pool: &PgPool,
    download_sessions: &DownloadSessions,
    config: &AutoDownloadConfig,
    state: Arc<AutoDownloadState>,
    update_tx: Option<tokio::sync::broadcast::Sender<serde_json::Value>>,
) -> Result<(), String> {
    // Get all playlists with auto_download enabled
    let playlists: Vec<YoutubePlaylist> = sqlx::query_as(
        r#"
        SELECT id, name, url, description, auto_download, last_synced_at, created_at, updated_at
        FROM youtube_playlists
        WHERE auto_download = true
        ORDER BY last_synced_at ASC NULLS FIRST
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get playlists: {}", e))?;
    
    if playlists.is_empty() {
        log::info!("No playlists with auto-download enabled");
        return Ok(());
    }
    
    log::info!("Found {} playlists to check for new content", playlists.len());
    
    for playlist in playlists {
        if state.should_stop.load(Ordering::Relaxed) {
            log::info!("Auto-download stopping early due to stop signal");
            break;
        }
        
        *state.current_playlist.write().await = Some(playlist.name.clone());
        log::info!("Checking playlist: {}", playlist.name);
        
        // Download with throttling
        let options = DownloadOptions {
            limit: None, // Download all new videos
            max_concurrent: Some(config.max_concurrent_downloads as u32),
            audio_quality: Some("192".to_string()),
        };
        
        match yt_downloader::download_youtube_playlist(
            playlist.url.clone(),
            "/music/downloads".to_string(),
            Some(options),
            download_sessions.clone(),
            pool.clone(),
            update_tx.clone(),
        ).await {
            Ok(session_id) => {
                log::info!("Started download session {} for playlist {}", session_id, playlist.name);
                
                // Wait for this session to complete before moving to next playlist
                // This helps with throttling
                wait_for_session_complete(download_sessions, &session_id).await;
                
                // Update last synced
                let _ = sqlx::query(
                    "UPDATE youtube_playlists SET last_synced_at = NOW(), updated_at = NOW() WHERE id = $1"
                )
                .bind(playlist.id)
                .execute(pool)
                .await;
                
                state.downloads_completed.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                log::error!("Failed to start download for playlist {}: {}", playlist.name, e);
            }
        }
        
        // Delay between playlists
        if config.delay_between_downloads_seconds > 0 {
            tokio::time::sleep(Duration::from_secs(config.delay_between_downloads_seconds as u64)).await;
        }
    }
    
    Ok(())
}

/// Wait for a download session to complete
async fn wait_for_session_complete(sessions: &DownloadSessions, session_id: &str) {
    let timeout = Duration::from_secs(3600); // 1 hour max wait
    let start = std::time::Instant::now();
    
    loop {
        if start.elapsed() > timeout {
            log::warn!("Download session {} timed out", session_id);
            break;
        }
        
        let sessions_read = sessions.read().await;
        if let Some(session) = sessions_read.get(session_id) {
            let progress = session.progress.read().await;
            if progress.progress == Some(100.0) || progress.is_cancelled == Some(true) {
                break;
            }
        } else {
            // Session no longer exists, assume complete
            break;
        }
        drop(sessions_read);
        
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

/// Manually trigger an auto-download run
pub async fn trigger_now(
    pool: &PgPool,
    download_sessions: &DownloadSessions,
    state: Arc<AutoDownloadState>,
    update_tx: Option<tokio::sync::broadcast::Sender<serde_json::Value>>,
) -> Result<String, String> {
    if state.is_running.load(Ordering::Relaxed) {
        return Err("Auto-download is already running".to_string());
    }
    
    let config = get_config_from_pool(pool).await
        .map_err(|e| format!("Failed to get config: {}", e))?;
    
    state.is_running.store(true, Ordering::Relaxed);
    state.reset_counters();
    
    let pool = pool.clone();
    let sessions = download_sessions.clone();
    let state_clone = state.clone();
    
    tokio::spawn(async move {
        if let Err(e) = run_auto_download(&pool, &sessions, &config, state_clone.clone(), update_tx).await {
            log::error!("Manual auto-download trigger error: {}", e);
        }
        
        if let Err(e) = update_last_check(&pool, config.check_interval_minutes).await {
            log::error!("Failed to update last check time: {}", e);
        }
        
        state_clone.is_running.store(false, Ordering::Relaxed);
        *state_clone.current_playlist.write().await = None;
    });
    
    Ok("Auto-download triggered".to_string())
}

/// Stop the current auto-download run
pub fn stop_current_run(state: &AutoDownloadState) {
    state.should_stop.store(true, Ordering::Relaxed);
}
