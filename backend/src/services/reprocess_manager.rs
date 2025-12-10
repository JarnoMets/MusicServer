use crate::db::Database;
use crate::services::{genre_cache_service, genre_detection};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct ReprocessProgress {
    pub processed: usize,
    pub total: usize,
    pub current: Option<String>,
    pub finished: bool,
}

#[derive(Clone)]
pub struct ReprocessSession {
    pub id: String,
    pub tx: broadcast::Sender<ReprocessProgress>,
}

type Sessions = RwLock<HashMap<String, ReprocessSession>>;

static REPROCESS_SESSIONS: OnceCell<Arc<Sessions>> = OnceCell::new();

fn get_sessions() -> Arc<Sessions> {
    REPROCESS_SESSIONS
        .get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
        .clone()
}

/// Start a reprocess job and return a session id
pub async fn start_reprocess(db: Database) -> String {
    let sessions = get_sessions();
    let session_id = Uuid::new_v4().to_string();
    let (tx, _rx) = broadcast::channel(16);

    let session = ReprocessSession {
        id: session_id.clone(),
        tx: tx.clone(),
    };

    {
        let mut map = sessions.write().await;
        map.insert(session_id.clone(), session.clone());
    }

    // Spawn background task
    let cleanup_id = session_id.clone();
    tokio::spawn(async move {
        // Fetch artists without a cached genre
        let artists_res: Result<Vec<Option<String>>, sqlx::Error> = sqlx::query_scalar(
            "SELECT DISTINCT artist FROM music_files WHERE artist IS NOT NULL AND artist != '' AND NOT EXISTS (SELECT 1 FROM artist_genres ag WHERE lower(ag.artist_name) = lower(music_files.artist))"
        )
        .fetch_all(&db.pool)
        .await;

        let artists: Vec<String> = match artists_res {
            Ok(list) => list.into_iter().filter_map(|o| o).collect(),
            Err(_) => vec![],
        };

        let total = artists.len();
        let mut processed = 0usize;

        // batching and rate limiting
        let batch_size = 50usize;
        let pause_ms = 300u64; // pause between requests to avoid hitting MB rate limits

        for chunk in artists.chunks(batch_size) {
            for artist in chunk {
                let _ = tx.send(ReprocessProgress {
                    processed,
                    total,
                    current: Some(artist.clone()),
                    finished: false,
                });

                // Call detection which caches canonical genre when possible
                let _ = genre_detection::detect_genre_for_artist(&db, artist.clone()).await;

                // If cached, update guessed_genre for files by artist
                if let Ok(Some(cached)) = genre_cache_service::get_cached_genre(&db, artist).await {
                    let _ = sqlx::query("UPDATE music_files SET guessed_genre = $1, updated_at = NOW() WHERE lower(artist) = lower($2)")
                        .bind(&cached)
                        .bind(artist)
                        .execute(&db.pool)
                        .await;
                }

                processed += 1;

                let _ = tx.send(ReprocessProgress {
                    processed,
                    total,
                    current: None,
                    finished: false,
                });

                // small delay between individual requests
                tokio::time::sleep(std::time::Duration::from_millis(pause_ms)).await;
            }

            // slight pause between batches
            tokio::time::sleep(std::time::Duration::from_millis(pause_ms * 2)).await;
        }

        let _ = tx.send(ReprocessProgress {
            processed,
            total,
            current: None,
            finished: true,
        });

        // cleanup session
        let sessions = get_sessions();
        let mut map = sessions.write().await;
        map.remove(&cleanup_id);
    });

    session_id
}

pub async fn subscribe(session_id: &str) -> Option<broadcast::Receiver<ReprocessProgress>> {
    let sessions = get_sessions();
    let map = sessions.read().await;
    map.get(session_id).map(|s| s.tx.subscribe())
}
