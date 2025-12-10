use crate::db::Database;
use chrono::Utc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use sqlx::PgPool;

/// Shared state for the auto-genre lookup scheduler
pub struct AutoGenreLookupState {
    pub is_running: Arc<AtomicBool>,
    pub artists_processed: Arc<AtomicI32>,
    pub lookup_errors: Arc<AtomicI32>,
    pub should_stop: Arc<AtomicBool>,
}

impl AutoGenreLookupState {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            artists_processed: Arc::new(AtomicI32::new(0)),
            lookup_errors: Arc::new(AtomicI32::new(0)),
            should_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn reset_counters(&self) {
        self.artists_processed.store(0, Ordering::Relaxed);
        self.lookup_errors.store(0, Ordering::Relaxed);
    }
}

impl Default for AutoGenreLookupState {
    fn default() -> Self {
        Self::new()
    }
}

/// Start the background genre lookup scheduler
/// Processes up to 5 artists per minute (1 every 12 seconds)
pub fn start_scheduler(pool: PgPool) {
    let state = Arc::new(AutoGenreLookupState::new());
    let state_clone = state.clone();

    tokio::spawn(async move {
        let mut interval_timer = interval(Duration::from_secs(12)); // 5 per minute = 1 every 12 seconds
        let mut last_reset = Utc::now();

        loop {
            if state_clone.should_stop.load(Ordering::Relaxed) {
                log::info!("Auto-genre lookup scheduler shutting down...");
                break;
            }

            interval_timer.tick().await;

            // Reset counters every minute for monitoring
            let now = Utc::now();
            if (now - last_reset).num_seconds() > 60 {
                let processed = state_clone.artists_processed.swap(0, Ordering::Relaxed);
                let errors = state_clone.lookup_errors.swap(0, Ordering::Relaxed);
                if processed > 0 || errors > 0 {
                    log::info!(
                        "Auto-genre lookup: processed={}, errors={}",
                        processed,
                        errors
                    );
                }
                last_reset = now;
            }

            // Process one artist with unknown/default genre
            if let Err(e) = process_one_unknown_artist(&pool).await {
                log::warn!("Error processing unknown artist: {}", e);
                state_clone.lookup_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    });
}

/// Get one artist with "Unknown" genre and try to detect their actual genre
async fn process_one_unknown_artist(pool: &PgPool) -> Result<(), String> {
    // Find the first artist with "Unknown" genre (or no genre)
    let artist: Option<String> = sqlx::query_scalar(
        r#"
        SELECT artist_name 
        FROM artist_genres 
        WHERE genre = 'Unknown' OR genre IS NULL
        ORDER BY created_at ASC
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    if let Some(artist_name) = artist {
        log::debug!("Looking up genre for artist: {}", artist_name);

        // Create a temporary database wrapper for the detection function
        let db = Database { pool: pool.clone() };

        // Attempt to detect genre using MusicBrainz
        match super::genre_detection::detect_genre_for_artist(&db, artist_name.clone()).await {
            Ok(Some(genre)) => {
                log::info!("Auto-detected genre for {}: {}", artist_name, genre);
                // The genre_detection service already caches it
            }
            Ok(None) => {
                log::debug!("No genre found for artist: {}", artist_name);
            }
            Err(e) => {
                log::warn!("Error detecting genre for {}: {}", artist_name, e);
                return Err(e);
            }
        }
    }

    Ok(())
}

/// Get the current state of the auto-genre lookup
pub fn get_state(state: &AutoGenreLookupState) -> serde_json::Value {
    serde_json::json!({
        "is_running": state.is_running.load(Ordering::Relaxed),
        "artists_processed": state.artists_processed.load(Ordering::Relaxed),
        "lookup_errors": state.lookup_errors.load(Ordering::Relaxed),
    })
}
