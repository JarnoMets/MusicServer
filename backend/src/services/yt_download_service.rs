use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct YoutubeDownload {
    pub id: Uuid,
    pub video_id: String,
    pub video_url: String,
    pub title: Option<String>,
    pub uploader: Option<String>,
    pub file_path: Option<String>,
    pub downloaded_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CreateYoutubeDownload {
    pub video_id: String,
    pub video_url: String,
    pub title: Option<String>,
    pub uploader: Option<String>,
    pub file_path: Option<String>,
}

/// Extract video ID from YouTube URL
#[allow(dead_code)]
pub fn extract_video_id(url: &str) -> Option<String> {
    // Handle youtube.com/watch?v=ID format
    if let Some(pos) = url.find("v=") {
        let id: String = url[pos + 2..].chars().take_while(|c| *c != '&').collect();
        if !id.is_empty() {
            return Some(id);
        }
    }

    // Handle youtu.be/ID format
    if let Some(pos) = url.find("youtu.be/") {
        let id: String = url[pos + 9..]
            .chars()
            .take_while(|c| *c != '?' && *c != '&')
            .collect();
        if !id.is_empty() {
            return Some(id);
        }
    }

    None
}

/// Check if a video has already been downloaded
pub async fn is_video_downloaded(pool: &PgPool, video_id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM youtube_downloads WHERE video_id = $1)",
    )
    .bind(video_id)
    .fetch_one(pool)
    .await?;

    Ok(result)
}

/// Save a successfully downloaded video
pub async fn save_download(
    pool: &PgPool,
    download: CreateYoutubeDownload,
) -> Result<YoutubeDownload, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let result = sqlx::query_as::<_, YoutubeDownload>(
        r#"
        INSERT INTO youtube_downloads (id, video_id, video_url, title, uploader, file_path, downloaded_at, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (video_id) DO UPDATE SET
            updated_at = NOW()
        RETURNING id, video_id, video_url, title, uploader, file_path, downloaded_at, created_at, updated_at
        "#
    )
    .bind(id)
    .bind(&download.video_id)
    .bind(&download.video_url)
    .bind(&download.title)
    .bind(&download.uploader)
    .bind(&download.file_path)
    .bind(now)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(result)
}

/// Get all downloaded videos
pub async fn get_all_downloads(pool: &PgPool) -> Result<Vec<YoutubeDownload>, sqlx::Error> {
    sqlx::query_as::<_, YoutubeDownload>(
        "SELECT id, video_id, video_url, title, uploader, file_path, downloaded_at, created_at, updated_at FROM youtube_downloads ORDER BY downloaded_at DESC"
    )
    .fetch_all(pool)
    .await
}

/// Get downloaded video by video ID
#[allow(dead_code)]
pub async fn get_download_by_video_id(
    pool: &PgPool,
    video_id: &str,
) -> Result<Option<YoutubeDownload>, sqlx::Error> {
    sqlx::query_as::<_, YoutubeDownload>(
        "SELECT id, video_id, video_url, title, uploader, file_path, downloaded_at, created_at, updated_at FROM youtube_downloads WHERE video_id = $1"
    )
    .bind(video_id)
    .fetch_optional(pool)
    .await
}

/// Delete a downloaded video record
pub async fn delete_download(pool: &PgPool, video_id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM youtube_downloads WHERE video_id = $1")
        .bind(video_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Get count of downloaded videos
pub async fn get_download_count(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM youtube_downloads")
        .fetch_one(pool)
        .await?;

    Ok(count.0)
}
