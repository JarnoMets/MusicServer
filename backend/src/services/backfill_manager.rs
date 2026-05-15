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
        // Verify genre exists
        let genre_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM genres WHERE id = $1)")
            .bind(genre_id)
            .fetch_optional(&db.pool)
            .await
            .ok()
            .flatten()
            .unwrap_or(false);

        if !genre_exists {
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

        // Insert alias
        let _ = genre_label_service::add_alias(&db, &alias, genre_id).await;

        // Count matching rows for progress tracking
        let (music_count, artist_count) =
            genre_label_service::preview_backfill(&db, &alias, genre_id).await.unwrap_or((0, 0));

        let total = (music_count + artist_count) as usize;

        // Run the FK-based backfill (updates artist_genres + tracks in one call)
        let affected = genre_label_service::backfill_alias(&db, &alias, genre_id)
            .await
            .unwrap_or(0);

        let _ = tx.send(BackfillProgress {
            processed: affected as usize,
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
