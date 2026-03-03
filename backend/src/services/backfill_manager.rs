use crate::db::Database;
use crate::services::genre_label_service;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct BackfillProgress {
    pub processed: usize,
    pub total: usize,
    pub current: Option<String>,
    pub finished: bool,
}

#[derive(Clone)]
pub struct BackfillSession {
    #[allow(dead_code)]
    pub id: String,
    pub tx: broadcast::Sender<BackfillProgress>,
}

type Sessions = RwLock<HashMap<String, BackfillSession>>;

static BACKFILL_SESSIONS: OnceCell<Arc<Sessions>> = OnceCell::new();

pub fn get_sessions() -> Arc<Sessions> {
    BACKFILL_SESSIONS.get_or_init(|| Arc::new(RwLock::new(HashMap::new()))).clone()
}

pub async fn start_backfill(db: Database, alias: String, genre_id: uuid::Uuid) -> String {
    let sessions = get_sessions();
    let session_id = Uuid::new_v4().to_string();
    let (tx, _rx) = broadcast::channel(16);
    let session = BackfillSession {
        id: session_id.clone(),
        tx: tx.clone(),
    };

    {
        let mut map = sessions.write().await;
        map.insert(session_id.clone(), session.clone());
    }

    let remove_id = session_id.clone();
    tokio::spawn(async move {
        // Resolve canonical name
        let canonical: Option<String> = sqlx::query_scalar("SELECT name FROM genres WHERE id = $1")
            .bind(genre_id)
            .fetch_optional(&db.pool)
            .await
            .ok()
            .flatten();

        if canonical.is_none() {
            let _ = tx.send(BackfillProgress {
                processed: 0,
                total: 0,
                current: None,
                finished: true,
            });
            let sessions2 = get_sessions();
            let mut map = sessions2.write().await;
            map.remove(&remove_id);
            return;
        }

        let canonical = canonical.unwrap();

        // Insert alias
        let _ = genre_label_service::add_alias(&db, &alias, genre_id).await;

        // Count matching rows
        let (music_count, artist_count) =
            (genre_label_service::preview_backfill(&db, &alias).await).unwrap_or((0, 0));

        let total = (music_count + artist_count) as usize;
        let mut processed: usize = 0;

        // Backfill music_files in chunks: select ids then update in batches
        if music_count > 0 {
            // Include both confirmed genre and guessed_genre
            let ids: Vec<uuid::Uuid> = sqlx::query_scalar("SELECT id FROM music_files WHERE (guessed_genre IS NOT NULL AND lower(guessed_genre) = lower($1)) OR (genre IS NOT NULL AND lower(genre) = lower($1))")
                .bind(&alias)
                .fetch_all(&db.pool)
                .await
                .unwrap_or_default();

            let chunk_size = 200usize;
            for chunk in ids.chunks(chunk_size) {
                // Update both genre and guessed_genre columns
                let _ = sqlx::query("UPDATE music_files SET guessed_genre = CASE WHEN lower(guessed_genre) = lower($1) THEN $2 ELSE guessed_genre END, genre = CASE WHEN lower(genre) = lower($1) THEN $2 ELSE genre END, updated_at = NOW() WHERE id = ANY($3)")
                    .bind(&alias)
                    .bind(&canonical)
                    .bind(chunk)
                    .execute(&db.pool)
                    .await;

                processed += chunk.len();
                let _ = tx.send(BackfillProgress {
                    processed,
                    total,
                    current: None,
                    finished: false,
                });
            }
        }

        // Backfill artist_genres
        if artist_count > 0 {
            let ids: Vec<uuid::Uuid> = sqlx::query_scalar(
                "SELECT id FROM artist_genres WHERE genre IS NOT NULL AND lower(genre) = lower($1)",
            )
            .bind(&alias)
            .fetch_all(&db.pool)
            .await
            .unwrap_or_default();

            let chunk_size = 200usize;
            for chunk in ids.chunks(chunk_size) {
                let _ = sqlx::query(
                    "UPDATE artist_genres SET genre = $1, last_updated = NOW() WHERE id = ANY($2)",
                )
                .bind(&canonical)
                .bind(chunk)
                .execute(&db.pool)
                .await;

                processed += chunk.len();
                let _ = tx.send(BackfillProgress {
                    processed,
                    total,
                    current: None,
                    finished: false,
                });
            }
        }

        let _ = tx.send(BackfillProgress {
            processed,
            total,
            current: None,
            finished: true,
        });

        let sessions2 = get_sessions();
        let mut map = sessions2.write().await;
        map.remove(&remove_id);
    });

    session_id
}

#[allow(dead_code)]
pub async fn subscribe(session_id: &str) -> Option<broadcast::Receiver<BackfillProgress>> {
    let sessions = get_sessions();
    let map = sessions.read().await;
    map.get(session_id).map(|s| s.tx.subscribe())
}
