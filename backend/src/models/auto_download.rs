use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AutoDownloadConfig {
    pub id: Uuid,
    pub enabled: bool,
    pub check_interval_minutes: i32,
    pub max_concurrent_downloads: i32,
    pub delay_between_downloads_seconds: i32,
    pub allowed_start_hour: Option<i32>,
    pub allowed_end_hour: Option<i32>,
    pub last_check_at: Option<DateTime<Utc>>,
    pub next_check_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAutoDownloadConfigRequest {
    pub enabled: Option<bool>,
    pub check_interval_minutes: Option<i32>,
    pub max_concurrent_downloads: Option<i32>,
    pub delay_between_downloads_seconds: Option<i32>,
    pub allowed_start_hour: Option<i32>,
    pub allowed_end_hour: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoDownloadStatus {
    pub config: AutoDownloadConfig,
    pub is_running: bool,
    pub current_playlist: Option<String>,
    pub downloads_in_progress: i32,
    pub downloads_completed_this_run: i32,
    pub downloads_skipped_this_run: i32,
}
