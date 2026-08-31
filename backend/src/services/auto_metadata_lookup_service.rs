use crate::models::metadata::MetadataConfig;
use crate::services::discogs_service::DiscogsService;
use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::Duration;
use uuid::Uuid;
use serde_json::Value;

pub struct AutoMetadataLookupState {
    pub is_running: Arc<AtomicBool>,
    pub should_stop: Arc<AtomicBool>,
}

impl Default for AutoMetadataLookupState {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoMetadataLookupState {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            should_stop: Arc::new(AtomicBool::new(false)),
        }
    }
}

pub fn start_scheduler(pool: PgPool, cache_update_tx: broadcast::Sender<Value>) -> Arc<AutoMetadataLookupState> {
    let state = Arc::new(AutoMetadataLookupState::new());
    let state_clone = state.clone();

    tokio::spawn(async move {
        log::info!("Auto-metadata lookup scheduler starting...");
        // Discogs rate limit is 60 per minute.
        // A single track lookup can trigger 1-5 search requests + detail requests.
        // So we should be conservative. 5 seconds per track allows for ~12 tracks/min
        // which could mean 12 * 5 = 60 requests/min in worst case.
        let mut pass_interval = tokio::time::interval(Duration::from_secs(5));
        pass_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        
        state_clone.is_running.store(true, Ordering::Relaxed);

        // Client with timeouts
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        loop {
            if state_clone.should_stop.load(Ordering::Relaxed) {
                log::info!("Auto-metadata lookup scheduler shutting down...");
                break;
            }

            pass_interval.tick().await;

            if let Err(e) = process_one_track(&pool, &client, &cache_update_tx).await {
                // If we hit a rate limit error that propagated up, wait longer
                if e.contains("429") || e.contains("Too Many Requests") {
                    log::warn!("Auto-lookup hit rate limit. Pausing for 60 seconds...");
                    tokio::time::sleep(Duration::from_secs(60)).await;
                } else {
                    log::warn!("Error processing track for metadata: {}. Retrying in 10s...", e);
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            } else {
                // If success (or no track found), check if we should slow down or speed up
                // Use a short sleep to not hammer the DB if empty
                // But if we processed a track, the 5s interval is already limiting us.
                // The `pass_interval.tick().await` handles the pacing.
            }
        }
        
        state_clone.is_running.store(false, Ordering::Relaxed);
    });

    state
}

async fn process_one_track(pool: &PgPool, client: &reqwest::Client, cache_update_tx: &broadcast::Sender<Value>) -> Result<(), String> {
    // Find one track that needs metadata analysis and doesn't have a suggestion yet
    // LIMIT 1 is correct, but we might want to prioritize recent adds or randomise to avoid stuck-on-error loops
    let track = sqlx::query_as::<_, (Uuid, String, Option<String>)>(
        r#"
        SELECT m.id, m.title, m.artist
        FROM music_files m
        LEFT JOIN metadata_suggestions s ON m.id = s.music_file_id
        WHERE s.music_file_id IS NULL
          AND (
               m.release_date IS NULL 
            OR m.album IS NULL OR m.album = '' 
            OR m.genre IS NULL OR m.genre = ''
          )
        ORDER BY RANDOM()
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    if let Some((id, title, artist)) = track {
        log::debug!("Looking up metadata for track: {} - {:?}", title, artist);

        let config = match MetadataConfig::get_config(pool).await {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to load metadata config: {}", e);
                return Err(e.to_string());
            }
        };

        let artist_str = artist.unwrap_or_default();
        
        match DiscogsService::lookup_release_date(client, &config, &title, &artist_str).await {
            Ok(Some((date, album, style, confidence))) => {
                log::info!("Found metadata for {}: date={:?}, album={:?}, genre={:?}", title, date, album, style);
                
                let result = sqlx::query_as::<_, crate::models::metadata_suggestion::MetadataSuggestion>(
                    r#"
                    INSERT INTO metadata_suggestions (music_file_id, release_date, album, genre, confidence)
                    VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT (music_file_id) DO UPDATE SET
                        release_date = EXCLUDED.release_date,
                        album = EXCLUDED.album,
                        genre = EXCLUDED.genre,
                        confidence = EXCLUDED.confidence,
                        updated_at = NOW()
                    RETURNING *
                    "#
                )
                .bind(id)
                .bind(date)
                .bind(album)
                .bind(style)
                .bind(confidence)
                .fetch_one(pool)
                .await
                .map_err(|e| format!("Failed to save suggestion: {}", e))?;

                // Notify clients
                let mut msg = serde_json::Map::new();
                msg.insert("type".to_string(), serde_json::Value::String("metadata_suggestion_found".to_string()));
                msg.insert("payload".to_string(), serde_json::to_value(&result).unwrap_or(serde_json::Value::Null));
                let _ = cache_update_tx.send(serde_json::Value::Object(msg));
            }
            Ok(None) => {
                log::debug!("No metadata found for track: {}", title);
                // Insert a negative result so we don't keep retrying
                sqlx::query(
                    r#"
                    INSERT INTO metadata_suggestions (music_file_id, release_date, album, genre, confidence)
                    VALUES ($1, NULL, NULL, NULL, 0.0)
                    ON CONFLICT (music_file_id) DO NOTHING
                    "#
                )
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to save negative suggestion: {}", e))?;
            }
            Err(e) => {
                log::warn!("Error looking up metadata for {}: {}", title, e);
                // If we error out, we should probably still mark it so we don't retry forever.
                // Insert a failure record with confidence -1.0 to indicate error
                sqlx::query(
                    r#"
                    INSERT INTO metadata_suggestions (music_file_id, release_date, album, genre, confidence)
                    VALUES ($1, NULL, NULL, NULL, -1.0)
                    ON CONFLICT (music_file_id) DO NOTHING
                    "#
                )
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to save error status: {}", e))?;
                
                return Err(e.to_string());
            }
        }
    }

    Ok(())
}
