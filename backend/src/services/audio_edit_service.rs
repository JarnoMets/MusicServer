use tokio::process::Command;
use std::path::Path;
use uuid::Uuid;
use crate::db::Database;
use crate::services::music_service;
use std::fs;

pub async fn cut_audio(
    db: &Database,
    id: Uuid,
    start_secs: f64,
    end_secs: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let music_file = music_service::get_music_file(db, id).await?
        .ok_or("Music file not found")?;

    let input_path = Path::new(&music_file.file_path);
    if !input_path.exists() {
        return Err("Source file does not exist".into());
    }

    let extension = input_path.extension().and_then(|e| e.to_str()).unwrap_or("mp3");
    let temp_output = format!("{}_cut.{}", music_file.file_path, extension);
    let output_path = Path::new(&temp_output);

    // Use nice to lower priority of the intensive FFmpeg process
    let mut cmd = Command::new("nice");
    cmd.arg("-n")
       .arg("10")
       .arg("ffmpeg")
       .arg("-y") // Overwrite output
       .arg("-ss")
       .arg(format!("{:.3}", start_secs))
       .arg("-to")
       .arg(format!("{:.3}", end_secs))
       .arg("-i")
       .arg(input_path)
       .arg("-c")
       .arg("copy") // Try to copy stream first
       .arg(output_path);

    let status = cmd.status().await?;

    if !status.success() {
        // Fallback to re-encoding if copy fails
        log::warn!("ffmpeg copy failed, falling back to re-encoding for cut");
        let mut fallback_cmd = Command::new("nice");
        fallback_cmd.arg("-n")
            .arg("10")
            .arg("ffmpeg")
            .arg("-y")
            .arg("-ss")
            .arg(format!("{:.3}", start_secs))
            .arg("-to")
            .arg(format!("{:.3}", end_secs))
            .arg("-i")
            .arg(input_path)
            .arg(output_path);
            
        let status_fallback = fallback_cmd.status().await?;
            
        if !status_fallback.success() {
            return Err("ffmpeg failed to cut audio".into());
        }
    }

    // Replace original with cut version
    // First backup or just replace? Let's replace but update DB stats
    let original_path = input_path.to_owned();
    fs::rename(output_path, &original_path)?;

    // Update duration in database
    let new_duration_ms = (end_secs - start_secs) * 1000.0;
    let update = crate::models::UpdateMusicFileRequest {
        duration: Some(new_duration_ms as i32),
        metadata_analyzed: Some(false),
        ..Default::default()
    };

    music_service::update_music_file(db, id, update).await?;

    Ok(())
}
