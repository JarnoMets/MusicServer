use crate::db::Database;
use crate::models::MusicFile;
use crate::services::music_service;
use lofty::{Probe, TaggedFileExt};
use std::path::Path;
use std::sync::Arc;
use tokio::time::{self, Duration};

pub async fn start_background_analysis(db: Arc<Database>) {
    tokio::spawn(async move {
        // Delay first run to let system start up
        tokio::time::sleep(Duration::from_secs(60)).await;
        log::info!("Background Rekordbox analysis scheduler starting (tick = 5 minutes)");
        let mut interval = time::interval(Duration::from_secs(300)); // Every 5 minutes
        loop {
            interval.tick().await;
            log::info!("Background Rekordbox analysis tick: checking for files to analyze");
            if let Err(e) = run_analysis_pass(&db).await {
                log::error!("Error in background Rekordbox analysis: {}", e);
            }
        }
    });
}

async fn run_analysis_pass(db: &Database) -> Result<(), sqlx::Error> {
    // Find files that need analysis: either marked unanalyzed, missing BPM, or missing Key
    let sql = format!(
        "{} WHERE metadata_analyzed = FALSE \
           OR bpm IS NULL OR bpm <= 0 \
           OR initial_key IS NULL OR initial_key = '' OR initial_key = 'NONE' \
        LIMIT 50",
        crate::services::music_query_helpers::select_music_files()
    );

    let files = sqlx::query_as::<_, MusicFile>(&sql)
    .fetch_all(&db.pool)
    .await?;

    if files.is_empty() {
        log::debug!("Background analysis pass: no files found needing analysis");
        return Ok(());
    }

    log::info!("Starting background analysis for {} files", files.len());

    for file in files {
        if !Path::new(&file.file_path).exists() {
            continue;
        }

        let id = file.id;
        let mut update = crate::models::UpdateMusicFileRequest::default();
        let path_str = file.file_path.clone();

        // Try to extract existing tags using lofty
        let meta = tokio::task::spawn_blocking(move || {
            let path = Path::new(&path_str);
            let probed = Probe::open(path).and_then(|p| p.read());
            match probed {
                Ok(tagged) => {
                    let mut bpm = None;
                    let mut initial_key = None;

                    // Try to get BPM and Key from any available tags
                    for tag in tagged.tags() {
                        if bpm.is_none() {
                            bpm = tag.get_string(&lofty::ItemKey::BPM).and_then(|s| s.parse::<f32>().ok());
                        }
                        if initial_key.is_none() {
                            initial_key = tag.get_string(&lofty::ItemKey::InitialKey).map(|s| s.to_string());
                        }
                    }

                    (bpm, initial_key)
                }
                Err(_) => (None, None),
            }
        })
        .await
        .unwrap_or((None, None));

        let (bpm_tag, key_tag) = meta;

        // 1. Process BPM
        if let Some(b) = bpm_tag.filter(|&b| b > 0.0) {
            update.bpm = Some(b as f64);
        } else if file.bpm.is_none() || file.bpm.unwrap_or(0.0) <= 0.0 {
            // Signal detection if tag is missing and DB value is invalid
            let path_clone = file.file_path.clone();
            match tokio::task::spawn_blocking(move || {
                crate::services::bpm_service::detect_bpm(&path_clone)
            }).await {
                Ok(Ok(bpm_res)) => {
                    update.bpm = Some(bpm_res.bpm);
                    update.beat_grid_offset = Some(bpm_res.offset);
                    update.beat_map = Some(serde_json::to_value(&bpm_res.beats).unwrap_or(serde_json::Value::Null));
                    log::info!("Background BPM analysis for {}: detected {:.2} BPM (offset: {:.4}s)", file.title, bpm_res.bpm, bpm_res.offset);
                }
                Ok(Err(e)) => {
                    log::warn!("Background BPM detection failed for {}: {}", file.title, e);
                }
                Err(e) => {
                    log::error!("Background BPM task join error for {}: {}", file.title, e);
                }
            }
        }

        // 2. Process Initial Key
        if let Some(k) = key_tag.filter(|s| !s.is_empty() && s != "NONE") {
            update.initial_key = Some(crate::services::key_service::to_camelot(&k));
        } else if file.initial_key.is_none() || file.initial_key.as_deref().unwrap_or("") == "" || file.initial_key.as_deref().unwrap_or("") == "NONE" {
            // Signal detection if tag is missing and DB value is invalid
            let path_clone = file.file_path.clone();
            match tokio::task::spawn_blocking(move || {
                crate::services::key_service::detect_key(&path_clone)
            }).await {
                Ok(Ok(key)) => {
                    let camelot = crate::services::key_service::to_camelot(&key);
                    update.initial_key = Some(camelot);
                    log::info!("Background Key analysis for {}: detected {} ({})", file.title, key, update.initial_key.as_ref().unwrap());
                }
                Ok(Err(e)) => {
                    log::warn!("Background Key detection failed for {}: {}", file.title, e);
                }
                Err(e) => {
                    log::error!("Background Key task join error for {}: {}", file.title, e);
                }
            }
        }

        // Only mark as analyzed if we actually found metadata (tags or successful detection).
        // If detection failed, leave metadata_analyzed unset so the file will be retried later.
        let mut found_any = false;
        if update.bpm.is_some() || update.initial_key.is_some() {
            found_any = true;
        }

        if found_any {
            update.metadata_analyzed = Some(true);
        }

        music_service::update_music_file(db, id, update).await?;
    }

    Ok(())
}
