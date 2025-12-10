use crate::db::Database;
use crate::services::auto_download_service::AutoDownloadState;
use crate::services::auto_genre_lookup_service::AutoGenreLookupState;
use crate::yt_downloader::DownloadSessions;
use std::sync::Arc;

pub struct AppState {
    pub db: Database,
    pub download_sessions: DownloadSessions,
    /// Shared HTTP client for all outgoing requests (connection pooling, memory efficient)
    pub http_client: reqwest::Client,
    /// State for the auto-download scheduler
    pub auto_download_state: Arc<AutoDownloadState>,
    /// State for the auto-genre lookup scheduler
    pub auto_genre_lookup_state: Arc<AutoGenreLookupState>,
}
