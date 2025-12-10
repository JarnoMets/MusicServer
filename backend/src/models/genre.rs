use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ArtistGenre {
    pub id: Uuid,
    pub artist_name: String,
    pub genre: String,
    pub last_updated: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Genre {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetectGenreRequest {
    pub artist_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetectGenreResponse {
    pub artist_name: String,
    pub genre: Option<String>,
    pub cached: bool,
}
