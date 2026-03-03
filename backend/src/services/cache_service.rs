use crate::models::AppState;
use serde_json::Value;

/// Lightweight cache and notification helpers.
/// - Provides lazy-loading getters for a couple of commonly used lists (all tracks, artists summary)
/// - Provides a helper to publish change notifications to connected clients via AppState.cache_update_tx
pub async fn notify_change(state: &AppState, event_type: &str, payload: Value) {
    let mut msg = serde_json::Map::new();
    msg.insert("type".to_string(), Value::String(event_type.to_string()));
    msg.insert("payload".to_string(), payload);

    let json = Value::Object(msg);
    // Best-effort send; ignore error when there are no listeners
    let _ = state.cache_update_tx.send(json);
}

pub async fn get_all_tracks_cached(state: &AppState) -> Result<Vec<crate::models::MusicFile>, sqlx::Error> {
    // Check cache
    {
        let read = state.cached_all_tracks.read().await;
        if let Some(ref v) = *read {
            return Ok(v.clone());
        }
    }

    // Load from DB and populate cache
    let params = crate::models::MusicQueryParams {
        limit: Some(2000), // reasonable upper bound for 'all tracks' in UI
        ..Default::default()
    };

    let files = crate::services::music_service::get_all_music_files(&state.db, params).await?;

    let mut write = state.cached_all_tracks.write().await;
    *write = Some(files.clone());
    Ok(files)
}

pub async fn invalidate_all_tracks_cache(state: &AppState) {
    let mut write = state.cached_all_tracks.write().await;
    *write = None;
}

pub async fn get_artists_summary_cached(state: &AppState) -> Result<Vec<crate::models::ArtistSummary>, sqlx::Error> {
    {
        let read = state.cached_artists_summary.read().await;
        if let Some(ref v) = *read {
            return Ok(v.clone());
        }
    }

    let artists = crate::services::artist_service::get_all_artists_with_summary(&state.db).await?;

    let mut write = state.cached_artists_summary.write().await;
    *write = Some(artists.clone());
    Ok(artists)
}

pub async fn invalidate_artists_summary_cache(state: &AppState) {
    let mut write = state.cached_artists_summary.write().await;
    *write = None;
}
