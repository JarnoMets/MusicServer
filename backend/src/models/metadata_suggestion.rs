use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct MetadataSuggestion {
    pub music_file_id: Uuid,
    pub release_date: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
