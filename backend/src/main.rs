use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
// no custom timing middleware needed; Logger handles timing
use actix_web::middleware::Logger;
use std::sync::Arc;
use std::collections::HashSet;

use music_server::admin_middleware::AdminMiddleware;
use music_server::auth_middleware::AuthMiddleware;
use music_server::services::auto_download_service::{self, AutoDownloadState};
use music_server::services::auto_genre_lookup_service;
use music_server::services::auto_metadata_lookup_service;
use music_server::db::Database;
use music_server::models::AppState;
use music_server::routes;
use music_server::audit_routes;
use music_server::auth_routes;
use music_server::token_routes;
use music_server::yt_downloader;
use music_server::services;
use uuid::Uuid;
use chrono::Utc;
use music_server::services::discogs_service::DiscogsService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up panic hook to log panics before crashing
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())).unwrap_or_else(|| "unknown".to_string());
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        };
        // Emit to structured logger (if initialized) and always write to stderr so container logs capture it.
        // Note: the logger may not yet be initialized when the panic occurs, so keep `eprintln!`.
        log::error!("PANIC at {}: {}", location, message);
        eprintln!("PANIC at {}: {}", location, message);
    }));

    // Default to WARN to reduce noisy logs in production clusters. Individual
    // subsystems can be made more verbose via the RUST_LOG environment variable
    // (for example: `RUST_LOG=debug,sqlx=debug` for short-lived debugging).
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("warn"));

    log::info!("Music Server starting up...");
    log::info!("Music Server version: {}", env!("CARGO_PKG_VERSION"));
    log::info!("Build: commit={} built_at={}", env!("GIT_COMMIT"), env!("BUILD_TIME"));

    // Quick local Discogs test mode (bypass DB). Set DISCOGS_LOCAL_TEST=1 to run lookups and exit.
    if std::env::var("DISCOGS_LOCAL_TEST").unwrap_or_default() == "1" {
        log::info!("Running local Discogs tests (no DB)\n");
        let token = std::fs::read_to_string("../discogs_token").or_else(|_| std::fs::read_to_string("discogs_token")).ok().map(|s| s.trim().to_string());
        let config = music_server::models::MetadataConfig {
            id: Uuid::new_v4(),
            metadata_source: "discogs".to_string(),
            discogs_token: token.clone(),
            updated_at: Utc::now(),
        };

        let client = reqwest::Client::builder()
            .user_agent("MusicManager/1.0.0 (jarno.mets@gmail.com)")
            .build()
            .expect("Failed to create client");

        let cases: Vec<(String, String)> = if let Ok(single) = std::env::var("DISCOGS_LOCAL_SINGLE") {
            // Expect format: "Artist|Title"
            if let Some(pos) = single.find('|') {
                let artist = single[..pos].to_string();
                let title = single[pos+1..].to_string();
                vec![(artist, title)]
            } else {
                vec![
                    ("Turno".to_string(), "1989".to_string()),
                    ("Ivy Lab".to_string(), "20 Questions".to_string()),
                    ("Conway the Machine".to_string(), "200 Pies (ft. 2Chainz)".to_string()),
                    ("Disrupta & Furniss".to_string(), "3 seconds".to_string()),
                    ("Dillinja".to_string(), "30Hz (L-Side Remix)".to_string()),
                    ("1991".to_string(), "Jungle Cats".to_string()),
                ]
            }
        } else {
            vec![
                ("Turno".to_string(), "1989".to_string()),
                ("Ivy Lab".to_string(), "20 Questions".to_string()),
                ("Conway the Machine".to_string(), "200 Pies (ft. 2Chainz)".to_string()),
                ("Disrupta & Furniss".to_string(), "3 seconds".to_string()),
                ("Dillinja".to_string(), "30Hz (L-Side Remix)".to_string()),
                ("1991".to_string(), "Jungle Cats".to_string()),
            ]
        };

        for (artist, title) in &cases {
            log::info!("Lookup: {} - {}", artist, title);
            match DiscogsService::lookup_release_date(&client, &config, title.as_str(), artist.as_str()).await {
                Ok(Some((year, album, style, confidence))) => {
                    log::info!("Found: year={} album={:?} style={:?} conf={}", year, album, style, confidence);
                }
                Ok(None) => log::info!("No match found"),
                Err(e) => log::error!("Error during lookup: {}", e),
            }
        }

        return Ok(());
    }

    // Get database URL from environment or use default
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgresuser:postgrespwd@localhost:5432/musicdb".to_string()
    });
    log::info!("Connecting to database...");

    // Initialize database connection
    let db = match Database::new(&database_url).await {
        Ok(db) => {
            log::info!("Connected to PostgreSQL database");
            db
        }
        Err(e) => {
            log::error!("Failed to connect to database: {}", e);
            panic!("Database connection failed: {}", e);
        }
    };

    log::info!("Initializing application state...");
    
    // Create a shared HTTP client with connection pooling for all outgoing requests
    let http_client = reqwest::Client::builder()
        .pool_max_idle_per_host(4)  // Increased from 2 for better connection reuse
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .timeout(std::time::Duration::from_secs(60))  // Increased from 30s for large file uploads
        .user_agent("MusicManager/1.0.0 (jarno.mets@gmail.com)")
        .build()
        .expect("Failed to create HTTP client");
    
    // Create auto-download state
    let auto_download_state = Arc::new(AutoDownloadState::new());
    
    // Channel for broadcasting cache/update events to clients
    let (cache_update_tx, _) = tokio::sync::broadcast::channel(256);

    // Start the auto-genre lookup scheduler (runs in background continuously)
    let auto_genre_lookup_state = auto_genre_lookup_service::start_scheduler(db.pool.clone());
    log::info!("Auto-genre lookup scheduler initialized");
    
    // Start the auto-metadata lookup scheduler
    let _auto_metadata_lookup_state = auto_metadata_lookup_service::start_scheduler(db.pool.clone(), cache_update_tx.clone());
    log::info!("Auto-metadata lookup scheduler initialized");
    
    let download_sessions = yt_downloader::create_download_sessions();
    
    // Cleanup any orphaned temp directories from previous runs
    yt_downloader::init_cleanup("/music/downloads");
    
    // Start the auto-download scheduler
    auto_download_service::start_scheduler(
        db.pool.clone(),
        download_sessions.clone(),
        auto_download_state.clone(),
        cache_update_tx.clone(),
    );
    log::info!("Auto-download scheduler initialized");
    
    // Start background analysis - clone db BEFORE moving it into app_state
    let analysis_db = db.clone();
    services::rekordbox_service::start_background_analysis(Arc::new(analysis_db)).await;

    // Load environment variables for Google SSO
    let google_client_id = std::env::var("GOOGLE_CLIENT_ID")
        .unwrap_or_else(|_| "".to_string());
    let google_client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
        .unwrap_or_else(|_| "".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "change-this-to-a-secure-random-key-in-production".to_string());
    let app_url = std::env::var("APP_URL")
        .unwrap_or_else(|_| "http://localhost:8081".to_string());

    let app_state = web::Data::new(AppState {
        db,
        download_sessions,
        http_client,
        auto_download_state,
        auto_genre_lookup_state,
        google_client_id,
        google_client_secret,
        jwt_secret,
        app_url,
        cache_update_tx,
        cached_all_tracks: Arc::new(tokio::sync::RwLock::new(None)),
        cached_artists_summary: Arc::new(tokio::sync::RwLock::new(None)),
        used_google_code_hashes: Arc::new(tokio::sync::RwLock::new(HashSet::new())),
    });

    log::info!("Starting server at http://0.0.0.0:8081");
    log::info!("Server configuration: CORS=any, payload_limit=500MB");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        // Configure payload limits (500MB for file uploads)
        let payload_config = web::PayloadConfig::default()
            .limit(500 * 1024 * 1024); // 500MB

        App::new()
            .wrap(cors)
            // Logger middleware prints standardized request/response details including timing
            .wrap(Logger::default())
            .app_data(app_state.clone())
            .app_data(payload_config)
            .service(
                web::scope("/api")
                    .route("/health", web::get().to(routes::health_check))
                    .route("/health/db", web::get().to(routes::db_health_check))
                    // Machine-readable API documentation (unauthenticated)
                    .route("/llm.txt", web::get().to(serve_llm_txt))
                    // Authentication routes (unprotected)
                    .service(
                        web::scope("/auth")
                            .route("/google/url", web::get().to(auth_routes::get_google_auth_url))
                            .route("/google/callback", web::post().to(auth_routes::google_auth))
                            .route("/google/mobile", web::post().to(auth_routes::google_auth_mobile))
                    )
                    .service(
                        web::scope("")
                            .wrap(AuthMiddleware)
                            // Identity route
                            .route("/me", web::get().to(auth_routes::get_me))
                            // Personal access token management (requires JWT auth)
                            .route("/tokens", web::get().to(token_routes::list_tokens))
                            .route("/tokens", web::post().to(token_routes::create_token))
                            .route("/tokens/{id}", web::get().to(token_routes::get_token))
                            .route("/tokens/{id}", web::patch().to(token_routes::update_token))
                            .route("/tokens/{id}", web::delete().to(token_routes::delete_token))
                            // Existing routes that now require authentication
                            .route("/playlists", web::get().to(routes::get_playlists))
                            .route("/playlists", web::post().to(routes::create_playlist))
                            .route("/playlists/{id}", web::get().to(routes::get_playlist))
                            .route(
                                "/playlists/{id}",
                                web::patch().to(routes::update_playlist_handler),
                            )
                            .route("/playlists/{id}", web::delete().to(routes::delete_playlist))
                            .route(
                                "/playlists/{id}/tracks",
                                web::post().to(routes::add_playlist_track),
                            )
                            .route(
                                "/playlists/{playlist_id}/tracks/{track_id}",
                                web::delete().to(routes::remove_playlist_track),
                            )
                            .route(
                                "/playlists/{id}/reorder",
                                web::post().to(routes::reorder_playlist_tracks),
                            )
                            .route(
                                "/playlists/{id}/export/zip",
                                web::get().to(routes::export_playlist_zip),
                            )
                            .route(
                                "/playlists/{id}/export/rekordbox",
                                web::get().to(routes::export_playlist_rekordbox),
                            )
                            .route("/music", web::get().to(routes::get_music_files))
                            .route("/music/all-cached", web::get().to(routes::get_cached_tracks))
                            .route("/music", web::post().to(routes::create_music_file))
                            .route("/music/stats", web::get().to(routes::get_music_stats))
                            .route("/music/upload", web::post().to(routes::upload_music_files))
                            .route("/music/release-date-lookup", web::post().to(routes::lookup_release_date))
                            .route("/music/metadata-suggestions", web::get().to(routes::get_metadata_suggestions))
                            .route("/music/metadata-suggestions", web::delete().to(routes::delete_all_metadata_suggestions))
                            .route("/music/metadata-suggestions/{id}", web::delete().to(routes::delete_metadata_suggestion))
                            .route("/music/check-duplicate", web::post().to(routes::check_duplicate_hash))
                            .route("/music/sync", web::post().to(routes::sync_music_folder))
                            .route("/music/{id}", web::get().to(routes::get_music_file_detail))
                            .route(
                                "/music/{id}",
                                web::patch().to(routes::update_music_file_handler),
                            )
                            .route("/music/{id}", web::delete().to(routes::delete_music_file))
                            .route(
                                "/music/{id}/bpm-detect",
                                web::post().to(routes::detect_bpm_handler),
                            )
                            .route(
                                "/autoplay/config",
                                web::get().to(routes::get_autoplay_config),
                            )
                            .route(
                                "/autoplay/config",
                                web::put().to(routes::update_autoplay_config),
                            )
                            .route(
                                "/music/{id}/cut",
                                web::post().to(routes::cut_music_file),
                            )
                            .route(
                                "/music/{id}/stream",
                                web::get().to(routes::stream_music_file),
                            )
                            .route(
                                "/music/{id}/playlists",
                                web::get().to(routes::get_track_playlists),
                            )
                            .route("/streams", web::get().to(routes::list_streams))
                            .route("/streams", web::post().to(routes::create_stream))
                            .route("/streams/{id}", web::patch().to(routes::update_stream))
                            .route("/streams/{id}", web::delete().to(routes::delete_stream))
                            .route(
                                "/youtube/download",
                                web::post().to(routes::download_youtube),
                            )
                            .route(
                                "/youtube/progress/{id}",
                                web::get().to(routes::youtube_download_progress),
                            )
                            .route(
                                "/youtube/stream/{id}",
                                web::get().to(routes::youtube_download_stream),
                            )
                            .route(
                                "/youtube/cancel/{id}",
                                web::post().to(routes::cancel_youtube_download),
                            )
                            .route(
                                "/youtube/downloads",
                                web::get().to(routes::get_downloaded_videos),
                            )
                            .route("/youtube/stats", web::get().to(routes::get_download_stats))
                            .route(
                                "/youtube/downloads/{video_id}",
                                web::delete().to(routes::delete_download_record),
                            )
                            .route("/youtube/playlists", web::get().to(routes::list_youtube_playlists))
                            .route("/youtube/playlists/{id}", web::get().to(routes::get_youtube_playlist))
                            .route("/artists", web::get().to(routes::list_artists))
                            .route("/artists/cached", web::get().to(routes::get_cached_artists))
                            .route("/artists/{artist}", web::get().to(routes::get_artist_music))
                            .route("/genres/detect", web::post().to(routes::detect_genre))
                            .route("/genres/cache", web::get().to(routes::get_genre_cache))
                            .route("/genres/cache", web::delete().to(routes::clear_genre_cache))
                            .route("/genres", web::get().to(routes::list_genres))
                            .route(
                                "/genres/canonical",
                                web::get().to(routes::list_canonical_genres),
                            )
                            .route(
                                "/genres/unmapped",
                                web::get().to(routes::list_unmapped_genres),
                            )
                            .route(
                                "/genres/suggest/{raw}",
                                web::get().to(routes::suggest_genre_matches),
                            )
                            .route(
                                "/genres/aliases/backfill/{id}/stream",
                                web::get().to(routes::backfill_progress_stream),
                            )
                            .route("/updates/stream", web::get().to(routes::updates_stream))
                            .route(
                                "/genres/reprocess/{id}/stream",
                                web::get().to(routes::reprocess_progress_stream),
                            )
                            .route(
                                "/genres/aliases/preview/{alias}",
                                web::get().to(routes::preview_backfill_handler),
                            )
                            .route(
                                "/artists/{artist}/genre",
                                web::put().to(routes::set_artist_genre_handler),
                            )
                            .route(
                                "/music/confirm-genre",
                                web::post().to(routes::confirm_genre_handler),
                            )
                            .route(
                                "/music/sync/start",
                                web::post().to(routes::start_background_sync),
                            )
                            .route(
                                "/music/sync/stream/{id}",
                                web::get().to(routes::sync_progress_stream),
                            )
                            .route(
                                "/music/sync/cancel/{id}",
                                web::post().to(routes::cancel_sync),
                            )
                            // Admin routes nested inside authenticated scope
                            .service(
                                web::scope("/admin")
                                    .wrap(AdminMiddleware)
                                    .route("/genres", web::post().to(routes::create_genre))
                                    .route("/genres/merge", web::post().to(routes::merge_genres_handler))
                                    .route("/genres/{id}", web::patch().to(routes::update_genre_handler))
                                    .route("/genres/{id}", web::delete().to(routes::delete_genre_handler))
                                    .route("/genres/aliases", web::post().to(routes::add_genre_alias))
                                    .route(
                                        "/genres/aliases/backfill",
                                        web::post().to(routes::add_genre_alias_and_backfill),
                                    )
                                    .route(
                                        "/genres/aliases/backfill/start",
                                        web::post().to(routes::start_backfill_handler),
                                    )
                                    .route(
                                        "/genres/reprocess-missing",
                                        web::post().to(routes::reprocess_missing_genres),
                                    )
                                    .route(
                                        "/genres/cache/clear",
                                        web::post().to(routes::clear_genre_cache),
                                    )
                                    .route(
                                        "/artists/{artist}/rename",
                                        web::put().to(routes::rename_artist_handler),
                                    )
                                    // YouTube Playlist management (admin)
                                    .route("/youtube/playlists", web::post().to(routes::create_youtube_playlist))
                                    .route("/youtube/playlists/{id}", web::patch().to(routes::update_youtube_playlist))
                                    .route("/youtube/playlists/{id}", web::delete().to(routes::delete_youtube_playlist))
                                    .route("/youtube/playlists/{id}/sync", web::post().to(routes::sync_youtube_playlist))
                                    // Artist reprocessing
                                    .route("/artists/reprocess", web::post().to(routes::reprocess_artists))
                                    // Auto-download management
                                    .route("/auto-download/config", web::get().to(routes::get_auto_download_config))
                                    .route("/auto-download/config", web::put().to(routes::update_auto_download_config))
                                    .route("/auto-download/status", web::get().to(routes::get_auto_download_status))
                                    .route("/auto-download/trigger", web::post().to(routes::trigger_auto_download))
                                    .route("/auto-download/stop", web::post().to(routes::stop_auto_download))
                                    // Metadata management
                                    .route("/metadata/config", web::get().to(routes::get_metadata_config))
                                    .route("/metadata/config", web::put().to(routes::update_metadata_config))
                                    .route("/debug/discogs-lookup", web::post().to(routes::admin_debug_discogs_lookup))
                                    // Bulk operations
                                    .route("/music/bulk-rename", web::post().to(routes::bulk_rename_by_regex_handler))
                                    .route("/music/bulk-add-to-playlist", web::post().to(routes::bulk_add_to_playlist_by_regex_handler))
                                    .route("/music/bulk-update", web::post().to(routes::bulk_update_music_handler))
                                    // Audit logs (admin)
                                    .route("/audit/logs", web::get().to(audit_routes::get_audit_logs_handler))
                                    .route("/audit/revert", web::post().to(audit_routes::revert_audit_log_handler)),
                            ),
                    ),
            )
    })
    .bind("0.0.0.0:8081")?
    .workers(num_cpus::get())  // Use number of CPU cores for worker threads
    .keep_alive(std::time::Duration::from_secs(75))  // Keep-alive timeout for long uploads
    .run()
    .await
}

/// Serve machine-readable API documentation for LLM tooling.
/// Available at `GET /api/llm.txt` without authentication.
async fn serve_llm_txt() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(include_str!("llm.txt"))
}
