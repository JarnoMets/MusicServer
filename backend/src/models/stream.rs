use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct InternetStream {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub genre: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateStreamRequest {
    pub name: String,
    pub url: String,
    pub genre: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateStreamRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub genre: Option<String>,
    pub description: Option<String>,
}
