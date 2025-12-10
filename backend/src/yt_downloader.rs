use crate::services::yt_download_service;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc,
};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadOptions {
    pub limit: Option<u32>,
    pub max_concurrent: Option<u32>,
    pub audio_quality: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub status: String,
    pub progress: Option<f32>,
    pub current_file: Option<String>,
    pub total_files: Option<u32>,
    pub completed_files: Option<u32>,
    pub failed_files: Option<u32>,
    pub is_cancelled: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct DownloadSession {
    pub id: String,
    pub cancelled: Arc<AtomicBool>,
    pub progress: Arc<RwLock<DownloadProgress>>,
    pub progress_tx: broadcast::Sender<DownloadProgress>,
}

pub type DownloadSessions = Arc<RwLock<HashMap<String, DownloadSession>>>;

pub fn create_download_sessions() -> DownloadSessions {
    Arc::new(RwLock::new(HashMap::new()))
}

fn cleanup_incomplete_downloads(output_dir: &str) -> Result<(), String> {
    let output_path = PathBuf::from(output_dir);
    if !output_path.exists() {
        return Ok(());
    }

    // Look for temporary files and unwanted formats that might be left from incomplete downloads
    if let Ok(entries) = fs::read_dir(&output_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                let name = file_name.to_string_lossy();
                if let Some(extension) = path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    // Remove common temporary files from yt-dlp and unwanted video formats
                    if name.contains(".part")
                        || name.contains(".tmp")
                        || name.contains(".ytdl")
                        || ext == "mp4"
                        || ext == "webm"
                        || ext == "m4a"
                        || ext == "flv"
                        || ext == "mkv"
                    {
                        log::info!("Cleaning up file: {:?}", path);
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }
    }
    Ok(())
}

async fn send_progress(session: &DownloadSession, progress: DownloadProgress) {
    // Update stored progress
    {
        let mut stored_progress = session.progress.write().await;
        *stored_progress = progress.clone();
    }

    // Send to subscribers (ignore if no subscribers)
    let _ = session.progress_tx.send(progress);
}

pub async fn download_youtube_playlist(
    url: String,
    output_dir: String,
    options: Option<DownloadOptions>,
    sessions: DownloadSessions,
    pool: PgPool,
) -> Result<String, String> {
    // Create a unique session ID for this download
    let session_id = Uuid::new_v4().to_string();
    let cancelled = Arc::new(AtomicBool::new(false));
    let progress = Arc::new(RwLock::new(DownloadProgress {
        status: "Initializing...".to_string(),
        progress: Some(0.0),
        current_file: None,
        total_files: None,
        completed_files: Some(0),
        failed_files: Some(0),
        is_cancelled: Some(false),
    }));

    let (progress_tx, _) = broadcast::channel(32);

    let session = DownloadSession {
        id: session_id.clone(),
        cancelled: cancelled.clone(),
        progress: progress.clone(),
        progress_tx: progress_tx.clone(),
    };

    // Store the session
    {
        let mut sessions_map = sessions.write().await;
        sessions_map.insert(session_id.clone(), session.clone());
    }

    let output_path = PathBuf::from(&output_dir);

    // Apply default options if none provided
    let options = options.unwrap_or(DownloadOptions {
        limit: None,
        max_concurrent: Some(3),
        audio_quality: Some("192".to_string()),
    });

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_path).map_err(|e| format!("Failed to create directory: {}", e))?;

    // Send initial progress
    send_progress(
        &session,
        DownloadProgress {
            status: "Fetching playlist information...".to_string(),
            progress: Some(0.0),
            current_file: None,
            total_files: None,
            completed_files: Some(0),
            failed_files: Some(0),
            is_cancelled: Some(false),
        },
    )
    .await;

    // Check for cancellation
    if cancelled.load(Ordering::Relaxed) {
        cleanup_incomplete_downloads(&output_dir)?;
        return Err("Download cancelled by user".to_string());
    }

    // Build playlist items argument
    let playlist_items = if let Some(limit) = options.limit {
        format!("1-{}", limit)
    } else {
        "1-".to_string() // No limit
    };

    // First, get the playlist info to count total videos
    let mut info_cmd = Command::new("yt-dlp");
    info_cmd
        .arg("--flat-playlist")
        .arg("--print")
        .arg("%(id)s|%(title)s")
        .arg("--playlist-items")
        .arg(&playlist_items)
        .arg(&url);

    let info_output = info_cmd.output().map_err(|e| {
        format!(
            "Failed to get playlist info: {}. Make sure yt-dlp is installed.",
            e
        )
    })?;

    if !info_output.status.success() {
        return Err(format!(
            "Failed to get playlist info: {}",
            String::from_utf8_lossy(&info_output.stderr)
        ));
    }

    let playlist_info = String::from_utf8_lossy(&info_output.stdout);
    let videos: Vec<(String, String)> = playlist_info
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            let video_id = parts.get(0).unwrap_or(&"").to_string();
            let title = parts.get(1).unwrap_or(&"Unknown Title").to_string();
            (video_id, title)
        })
        .collect();

    let total_videos = videos.len() as u32;

    if total_videos == 0 {
        return Err("No videos found in playlist or invalid URL".to_string());
    }

    // Filter out already downloaded videos
    let mut skipped_count = 0;
    let filtered_videos: Vec<(String, String)> = {
        let mut filtered = Vec::new();
        for (video_id, title) in videos {
            match yt_download_service::is_video_downloaded(&pool, &video_id).await {
                Ok(true) => {
                    log::info!("Skipping already downloaded video: {}", title);
                    skipped_count += 1;
                }
                _ => {
                    filtered.push((video_id, title));
                }
            }
        }
        filtered
    };

    let remaining_videos = filtered_videos.len() as u32;

    let limit_text = if options.limit.is_some() {
        format!(" (limited to {})", options.limit.unwrap())
    } else {
        " (no limit)".to_string()
    };

    let skip_text = if skipped_count > 0 {
        format!(" - Skipping {} already downloaded", skipped_count)
    } else {
        String::new()
    };

    send_progress(
        &session,
        DownloadProgress {
            status: format!(
                "Found {} videos{}{}. Starting parallel download...",
                total_videos, limit_text, skip_text
            ),
            progress: Some(5.0),
            current_file: None,
            total_files: Some(remaining_videos),
            completed_files: Some(0),
            failed_files: Some(0),
            is_cancelled: Some(false),
        },
    )
    .await;

    // Check for cancellation before starting downloads
    if cancelled.load(Ordering::Relaxed) {
        cleanup_incomplete_downloads(&output_dir)?;
        return Err("Download cancelled by user".to_string());
    }

    // Use tokio semaphore to limit concurrent downloads
    let semaphore = Arc::new(tokio::sync::Semaphore::new(
        options.max_concurrent.unwrap_or(3) as usize,
    ));
    let downloaded_count = Arc::new(AtomicU32::new(0));
    let failed_count = Arc::new(AtomicU32::new(0));

    // Create download tasks
    let mut handles = Vec::new();

    for (video_id, title) in filtered_videos.into_iter() {
        // Check for cancellation before each task
        if cancelled.load(Ordering::Relaxed) {
            break;
        }

        let semaphore = semaphore.clone();
        let downloaded_count = downloaded_count.clone();
        let failed_count = failed_count.clone();
        let session = session.clone();
        let output_dir = output_dir.clone();
        let audio_quality = options
            .audio_quality
            .clone()
            .unwrap_or_else(|| "192".to_string());
        let individual_url = if url.contains("playlist") {
            format!("https://www.youtube.com/watch?v={}", video_id)
        } else {
            url.clone()
        };
        let cancelled = cancelled.clone();
        let pool = pool.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            // Check for cancellation before starting individual download
            if cancelled.load(Ordering::Relaxed) {
                return false;
            }

            // Download individual video
            let mut download_cmd = Command::new("yt-dlp");
            download_cmd
                .arg("-x")
                .arg("--audio-format")
                .arg("mp3")
                .arg("--audio-quality")
                .arg(&audio_quality)
                .arg("--no-playlist")
                .arg("--output")
                .arg(format!("{}/%(uploader)s - %(title)s.%(ext)s", output_dir))
                .arg("--embed-metadata")
                .arg("--add-metadata")
                .arg("--parse-metadata")
                .arg("%(uploader)s:%(artist)s")
                .arg("--parse-metadata")
                .arg("%(title)s:%(track)s")
                .arg("--concurrent-fragments")
                .arg("4")
                .arg("--retries")
                .arg("2")
                .arg("--fragment-retries")
                .arg("2")
                .arg("--no-warnings")
                .arg("--prefer-free-formats")
                .arg(&individual_url);

            match download_cmd.output() {
                Ok(output) => {
                    if output.status.success() {
                        let downloaded = downloaded_count.fetch_add(1, Ordering::Relaxed) + 1;
                        let current_progress =
                            5.0 + (downloaded as f32 / remaining_videos as f32) * 90.0;

                        // Save to database
                        let download_record = yt_download_service::CreateYoutubeDownload {
                            video_id: video_id.clone(),
                            video_url: individual_url.clone(),
                            title: Some(title.clone()),
                            uploader: None,
                            file_path: None,
                        };

                        if let Err(e) =
                            yt_download_service::save_download(&pool, download_record).await
                        {
                            log::warn!("Failed to save download record for {}: {}", video_id, e);
                        }

                        send_progress(
                            &session,
                            DownloadProgress {
                                status: format!(
                                    "Downloaded {} of {} videos",
                                    downloaded, remaining_videos
                                ),
                                progress: Some(current_progress),
                                current_file: Some(title.clone()),
                                total_files: Some(remaining_videos),
                                completed_files: Some(downloaded),
                                failed_files: Some(failed_count.load(Ordering::Relaxed)),
                                is_cancelled: Some(cancelled.load(Ordering::Relaxed)),
                            },
                        )
                        .await;

                        true
                    } else {
                        failed_count.fetch_add(1, Ordering::Relaxed);
                        log::error!(
                            "Failed to download {}: {}",
                            title,
                            String::from_utf8_lossy(&output.stderr)
                        );
                        false
                    }
                }
                Err(e) => {
                    failed_count.fetch_add(1, Ordering::Relaxed);
                    log::error!("Error downloading {}: {}", title, e);
                    false
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all downloads to complete or be cancelled
    for handle in handles {
        let _ = handle.await;

        // Check for cancellation periodically
        if cancelled.load(Ordering::Relaxed) {
            cleanup_incomplete_downloads(&output_dir)?;

            send_progress(
                &session,
                DownloadProgress {
                    status: "Download cancelled by user".to_string(),
                    progress: Some(0.0),
                    current_file: None,
                    total_files: Some(total_videos),
                    completed_files: Some(downloaded_count.load(Ordering::Relaxed)),
                    failed_files: Some(failed_count.load(Ordering::Relaxed)),
                    is_cancelled: Some(true),
                },
            )
            .await;

            return Err("Download cancelled by user".to_string());
        }
    }

    let final_downloaded = downloaded_count.load(Ordering::Relaxed);
    let final_failed = failed_count.load(Ordering::Relaxed);

    // Clean up any leftover files after completion
    cleanup_incomplete_downloads(&output_dir)?;

    // Send final progress
    send_progress(
        &session,
        DownloadProgress {
            status: "Download completed!".to_string(),
            progress: Some(100.0),
            current_file: None,
            total_files: Some(total_videos),
            completed_files: Some(final_downloaded),
            failed_files: Some(final_failed),
            is_cancelled: Some(false),
        },
    )
    .await;

    // Clean up the session after a delay to allow final progress to be received
    let sessions_cleanup = sessions.clone();
    let session_id_cleanup = session_id.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        let mut sessions_map = sessions_cleanup.write().await;
        sessions_map.remove(&session_id_cleanup);
    });

    Ok(session_id)
}
