use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct MusicFile {
    pub id: Uuid,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub guessed_genre: Option<String>,
    pub release_date: Option<DateTime<Utc>>,
    pub duration: Option<i32>,
    pub file_path: String,
    pub track_number: Option<i32>,
    pub file_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateMusicFileRequest {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub guessed_genre: Option<String>,
    pub release_date: Option<DateTime<Utc>>,
    pub duration: Option<i32>,
    pub file_path: String,
    pub track_number: Option<i32>,
    pub file_hash: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct UpdateMusicFileRequest {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub guessed_genre: Option<String>,
    pub release_date: Option<DateTime<Utc>>,
    pub duration: Option<i32>,
    pub track_number: Option<i32>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct MusicQueryParams {
    pub search: Option<String>,
    pub genre: Option<String>,
    pub artist: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    /// Filter to only show unconfirmed genres (guessed_genre set but genre not set)
    pub unconfirmed_only: Option<bool>,
}

/// Artist summary with genre and song count
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtistSummary {
    pub name: String,
    pub genre: Option<String>,
    pub song_count: i64,
}
