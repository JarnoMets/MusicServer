// Shared audio metadata extraction using lofty.
//
// This consolidates the metadata-reading logic that was duplicated across
// `routes.rs` (upload), `file_sync_service.rs`, and `sync_manager.rs`.

use std::path::Path;


/// Extracted audio metadata from a file.
#[derive(Debug, Default)]
pub struct AudioMetadata {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
    pub duration_ms: Option<i32>,
    pub track_number: Option<i32>,
    pub year: Option<i32>,
}

/// Extract audio metadata from a file path using lofty.
/// Runs on a blocking thread to avoid blocking the async runtime.
pub async fn extract_metadata(path: &Path) -> AudioMetadata {
    let path_clone = path.to_path_buf();
    tokio::task::spawn_blocking(move || extract_metadata_blocking(&path_clone))
        .await
        .unwrap_or_default()
}

/// Blocking version of metadata extraction (runs on dedicated thread).
fn extract_metadata_blocking(path: &Path) -> AudioMetadata {
    use lofty::{Accessor, AudioFile, Probe, TaggedFileExt};

    let probed = Probe::open(path).and_then(|p| p.read());
    match probed {
        Ok(tagged) => {
            let tag = tagged.primary_tag();
            let props = tagged.properties();

            AudioMetadata {
                artist: tag.and_then(|t| t.artist()).map(|s| s.to_string()),
                title: tag.and_then(|t| t.title()).map(|s| s.to_string()),
                album: tag.and_then(|t| t.album()).map(|s| s.to_string()),
                duration_ms: Some(props.duration().as_millis() as i32),
                track_number: tag.and_then(|t| t.track()).map(|n| n as i32),
                year: tag.and_then(|t| t.year()).map(|y| y as i32),
            }
        }
        Err(e) => {
            log::warn!("Failed to read metadata from {}: {}", path.display(), e);
            AudioMetadata::default()
        }
    }
}
