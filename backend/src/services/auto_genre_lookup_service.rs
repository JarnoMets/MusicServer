use crate::db::Database;
use chrono::Utc;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use sqlx::PgPool;

/// Shared state for the auto-genre lookup scheduler
pub struct AutoGenreLookupState {
    #[allow(dead_code)]
    pub is_running: Arc<AtomicBool>,
    pub artists_processed: Arc<AtomicI32>,
    pub lookup_errors: Arc<AtomicI32>,
    pub should_stop: Arc<AtomicBool>,
    /// Interval in seconds to restart a full pass through unmapped genres (default: 3600 = 1 hour)
    pub restart_interval_secs: Arc<AtomicU64>,
}

impl AutoGenreLookupState {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            artists_processed: Arc::new(AtomicI32::new(0)),
            lookup_errors: Arc::new(AtomicI32::new(0)),
            should_stop: Arc::new(AtomicBool::new(false)),
            restart_interval_secs: Arc::new(AtomicU64::new(3600)), // Default: 1 hour
        }
    }

    pub fn reset_counters(&self) {
        self.artists_processed.store(0, Ordering::Relaxed);
        self.lookup_errors.store(0, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    pub fn set_restart_interval(&self, secs: u64) {
        self.restart_interval_secs.store(secs, Ordering::Relaxed);
    }

    pub fn get_restart_interval(&self) -> u64 {
        self.restart_interval_secs.load(Ordering::Relaxed)
    }
}

impl Default for AutoGenreLookupState {
    fn default() -> Self {
        Self::new()
    }
}

/// Start the background genre lookup scheduler
/// Continuously processes artists with unknown genres on a schedule
/// Every `restart_interval_secs`, restarts processing from the oldest unmapped artist
pub fn start_scheduler(pool: PgPool) -> Arc<AutoGenreLookupState> {
    let state = Arc::new(AutoGenreLookupState::new());
    let state_clone = state.clone();

    tokio::spawn(async move {
        let mut pass_interval = interval(Duration::from_secs(12)); // Process 1 artist every 12 seconds (5 per minute)
        let mut last_reset = Utc::now();
        let mut last_restart = Utc::now();

        log::info!(
            "Auto-genre lookup scheduler started. Restart interval: {} seconds",
            state_clone.get_restart_interval()
        );

        loop {
            if state_clone.should_stop.load(Ordering::Relaxed) {
                log::info!("Auto-genre lookup scheduler shutting down...");
                break;
            }

            pass_interval.tick().await;

            // Get the current restart interval (allows runtime updates)
            let restart_interval = Duration::from_secs(state_clone.get_restart_interval());

            // Check if we should restart the pass (log at restart points)
            let now = Utc::now();
            if (now - last_restart).num_seconds() as u64 >= restart_interval.as_secs() {
                log::info!(
                    "Auto-genre lookup: Starting new pass through unmapped artists"
                );
                state_clone.reset_counters();
                last_restart = now;
            }

            // Reset counters every minute for monitoring
            if (now - last_reset).num_seconds() > 60 {
                let processed = state_clone.artists_processed.swap(0, Ordering::Relaxed);
                let errors = state_clone.lookup_errors.swap(0, Ordering::Relaxed);
                if processed > 0 || errors > 0 {
                    log::info!(
                        "Auto-genre lookup (last minute): processed={}, errors={}",
                        processed,
                        errors
                    );
                }
                last_reset = now;
            }

            // Process one artist with unknown/default genre
            if let Err(e) = process_one_unknown_artist(&pool).await {
                if e.contains("429") || e.contains("Too Many Requests") {
                    log::warn!("Auto-genre lookup hit rate limit. Pausing for 60 seconds...");
                    tokio::time::sleep(Duration::from_secs(60)).await;
                } else {
                    log::warn!("Error processing unknown artist: {}", e);
                }
                state_clone.lookup_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    state
}

/// Get one artist with "Unknown" genre and try to detect their actual genre
async fn process_one_unknown_artist(pool: &PgPool) -> Result<(), String> {
    // Find the first artist with "Unknown" genre (or no genre) but NOT "NotFound"
    let artist: Option<String> = sqlx::query_scalar(
        r#"
        SELECT artist_name 
        FROM artist_genres 
        WHERE (genre = 'Unknown' OR genre IS NULL)
        AND genre != 'NotFound'
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
                // Mark as NotFound to avoid retrying constantly
                if let Err(e) = sqlx::query(
                    "UPDATE artist_genres SET genre = 'NotFound' WHERE artist_name = $1"
                )
                .bind(&artist_name)
                .execute(pool)
                .await {
                    log::warn!("Failed to mark {} as NotFound: {}", artist_name, e);
                }
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
#[allow(dead_code)]
pub fn get_state(state: &AutoGenreLookupState) -> serde_json::Value {
    serde_json::json!({
        "is_running": state.is_running.load(Ordering::Relaxed),
        "artists_processed": state.artists_processed.load(Ordering::Relaxed),
        "lookup_errors": state.lookup_errors.load(Ordering::Relaxed),
    })
}
