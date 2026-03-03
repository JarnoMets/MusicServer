use crate::db::Database;
use crate::services::auto_download_service::AutoDownloadState;
use crate::services::auto_genre_lookup_service::AutoGenreLookupState;
use crate::yt_downloader::DownloadSessions;
use std::sync::Arc;
use std::collections::HashSet;
use tokio::sync::{broadcast, RwLock};

pub struct AppState {
    pub db: Database,
    pub download_sessions: DownloadSessions,
    /// Shared HTTP client for all outgoing requests (connection pooling, memory efficient)
    pub http_client: reqwest::Client,
    /// State for the auto-download scheduler
    pub auto_download_state: Arc<AutoDownloadState>,
    /// State for the auto-genre lookup scheduler
    #[allow(dead_code)]
    pub auto_genre_lookup_state: Arc<AutoGenreLookupState>,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub jwt_secret: String,
    pub app_url: String,

    // Channel used to broadcast cache/update events to connected clients (SSE / websockets)
    pub cache_update_tx: broadcast::Sender<serde_json::Value>,

    // Simple in-memory caches (lazy-loaded) protected by async RwLocks
    // - cached_all_tracks: optionally holds a full list of music files for 'All Tracks' views
    // - cached_artists_summary: optionally holds artist summaries used in many UI lists
    pub cached_all_tracks: Arc<RwLock<Option<Vec<crate::models::MusicFile>>>>,
    pub cached_artists_summary: Arc<RwLock<Option<Vec<crate::models::ArtistSummary>>>>,
    // Keep a short-term in-memory set of recently exchanged Google auth code hashes
    // to detect and return a clearer error when a client attempts to reuse the
    // same authorization code. We store a SHA-256 hex of the code (not the raw
    // code) to avoid keeping sensitive plaintext in memory.
    pub used_google_code_hashes: Arc<RwLock<HashSet<String>>>,
}
