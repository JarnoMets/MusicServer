use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct YoutubeDownloadRequest {
    pub url: String,
    pub output_dir: String,
    pub limit: Option<u32>,
    pub max_concurrent: Option<u32>,
    pub audio_quality: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeDownloadResponse {
    pub session_id: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YoutubeDownloadStats {
    pub total_downloaded: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct YoutubePlaylist {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub auto_download: bool,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateYoutubePlaylistRequest {
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub auto_download: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateYoutubePlaylistRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub auto_download: Option<bool>,
}
