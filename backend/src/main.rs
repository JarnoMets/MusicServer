use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
// no custom timing middleware needed; Logger handles timing
use actix_web::middleware::Logger;
use std::sync::Arc;

mod admin_middleware;
mod auth;
mod db;
mod models;
mod routes;
mod services;
mod yt_downloader;

use crate::admin_middleware::AdminMiddleware;
use crate::services::auto_download_service::{self, AutoDownloadState};
use db::Database;
use models::AppState;

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
        log::error!("PANIC at {}: {}", location, message);
        eprintln!("PANIC at {}: {}", location, message);
    }));

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Music Server starting up...");
    log::info!("Rust version: {}", env!("CARGO_PKG_VERSION"));

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
        .pool_max_idle_per_host(2)  // Limit idle connections per host
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("MusicManager/1.0.0 (jarno.mets@gmail.com)")
        .build()
        .expect("Failed to create HTTP client");
    
    // Create auto-download state
    let auto_download_state = Arc::new(AutoDownloadState::new());
    
    let download_sessions = yt_downloader::create_download_sessions();
    
    // Start the auto-download scheduler
    auto_download_service::start_scheduler(
        db.pool.clone(),
        download_sessions.clone(),
        auto_download_state.clone(),
    );
    log::info!("Auto-download scheduler initialized");
    
    let app_state = web::Data::new(AppState {
        db,
        download_sessions,
        http_client,
        auto_download_state,
    });

    log::info!("Starting Music Server API on http://0.0.0.0:8081");
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
                    .route("/music", web::get().to(routes::get_music_files))
                    .route("/music", web::post().to(routes::create_music_file))
                    .route("/music/upload", web::post().to(routes::upload_music_files))
                    .route("/music/sync", web::post().to(routes::sync_music_folder))
                    .route("/music/{id}", web::get().to(routes::get_music_file_detail))
                    .route(
                        "/music/{id}",
                        web::patch().to(routes::update_music_file_handler),
                    )
                    .route("/music/{id}", web::delete().to(routes::delete_music_file))
                    .route(
                        "/music/{id}/stream",
                        web::get().to(routes::stream_music_file),
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
                    // YouTube Playlist management (saved playlist links)
                    .route("/youtube/playlists", web::get().to(routes::list_youtube_playlists))
                    .route("/youtube/playlists/{id}", web::get().to(routes::get_youtube_playlist))
                    .route("/artists", web::get().to(routes::list_artists))
                    .route("/artists/{artist}", web::get().to(routes::get_artist_music))
                    .route("/genres/detect", web::post().to(routes::detect_genre))
                    .route("/genres/cache", web::get().to(routes::get_genre_cache))
                    .route("/genres/cache", web::delete().to(routes::clear_genre_cache))
                    .route("/genres", web::get().to(routes::list_genres))
                    // Admin-protected routes are registered in /api/admin scope below
                    .route(
                        "/genres/unmapped",
                        web::get().to(routes::list_unmapped_genres),
                    )
                    .route(
                        "/genres/suggest/{raw}",
                        web::get().to(routes::suggest_genre_matches),
                    )
                    .route(
                        "/genres/aliases/backfill",
                        web::post().to(routes::add_genre_alias_and_backfill),
                    )
                    .route(
                        "/genres/aliases/backfill/start",
                        web::post().to(routes::start_backfill_handler),
                    )
                    .route(
                        "/genres/aliases/backfill/{id}/stream",
                        web::get().to(routes::backfill_progress_stream),
                    )
                    .route(
                        "/genres/reprocess-missing",
                        web::post().to(routes::reprocess_missing_genres),
                    )
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
                    // Admin routes with middleware - nested inside /api scope
                    .service(
                        web::scope("/admin")
                            .wrap(AdminMiddleware)
                            .route("/genres", web::post().to(routes::create_genre))
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
                                "/genres/canonical",
                                web::get().to(routes::list_canonical_genres),
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
                            .route("/auto-download/stop", web::post().to(routes::stop_auto_download)),
                    ),
            )
    })
    .bind("0.0.0.0:8081")?
    .run()
    .await
}
