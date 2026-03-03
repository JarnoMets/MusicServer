use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Playlist {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Playlist with track count for list views
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub track_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistWithItems {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub items: Vec<crate::models::MusicFile>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePlaylistRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistTrackRequest {
    pub music_file_id: Uuid,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderPlaylistTracksRequest {
    pub music_file_ids: Vec<Uuid>,
}
