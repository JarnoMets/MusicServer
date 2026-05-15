// Shared SQL helpers for the `music_files` table.
//
// The full column list and filter logic were duplicated across many services.
// This module provides a single source of truth.

use crate::models::MusicQueryParams;
use crate::db::Database;
use crate::services::genre_label_service;

use sqlx::QueryBuilder;
use sqlx::Postgres;

/// All columns selected from `music_files mf` (with LEFT JOIN genres g) in the standard order.
/// Requires the query to use `FROM music_files mf LEFT JOIN genres g ON mf.genre_id = g.id`.
pub const MUSIC_FILE_COLUMNS: &str =
    "mf.id, mf.title, mf.artist, mf.album, mf.genre_id, g.name as genre_name, mf.genre_source, \
     mf.release_date, mf.duration, mf.file_path, mf.track_number, mf.file_hash, mf.bpm, \
     mf.initial_key, mf.beat_grid_offset, mf.beat_map, mf.metadata_analyzed, \
     mf.created_at, mf.updated_at";

/// Build a `SELECT <columns> FROM music_files mf LEFT JOIN genres g ON mf.genre_id = g.id` prefix.
pub fn select_music_files() -> String {
    format!(
        "SELECT {} FROM music_files mf LEFT JOIN genres g ON mf.genre_id = g.id",
        MUSIC_FILE_COLUMNS
    )
}

/// Append WHERE-clause filters from `MusicQueryParams` onto a `QueryBuilder`.
///
/// The builder is expected to already contain `… WHERE 1=1` or similar so
/// that each filter can be appended with `AND`.
pub fn apply_music_filters(builder: &mut QueryBuilder<'_, Postgres>, params: &MusicQueryParams) {
    if let Some(search) = &params.search {
        let like = format!("%{}%", search.to_lowercase());
        builder
            .push(" AND (LOWER(mf.title) LIKE ")
            .push_bind(like.clone())
            .push(" OR LOWER(mf.artist) LIKE ")
            .push_bind(like.clone())
            .push(" OR LOWER(mf.album) LIKE ")
            .push_bind(like)
            .push(")");
    }

    if let Some(genre_name) = &params.genre {
        // genre filter is resolved to genre_id asynchronously before this function is called.
        // For query building, we store the genre_name hint and resolve in the service layer.
        // Here we just bind the name as a fallback equality check on g.name.
        builder
            .push(" AND LOWER(g.name) = LOWER(")
            .push_bind(genre_name.clone())
            .push(")");
    }

    if let Some(artist) = &params.artist {
        builder
            .push(" AND LOWER(mf.artist) = LOWER(")
            .push_bind(artist.clone())
            .push(")");
    }

    if params.unconfirmed_only.unwrap_or(false) {
        // Tracks with auto-detected genre (not user-confirmed, not file tag)
        builder.push(" AND mf.genre_source = 'auto'");
    }

    if params.missing_metadata.unwrap_or(false) {
        builder.push(
            " AND (mf.genre_id IS NULL OR mf.release_date IS NULL OR mf.album IS NULL OR mf.album = '')",
        );
    }
}

/// Resolve a genre filter name to its canonical genre_id for use in precise FK queries.
/// Returns None if the genre name doesn't match any canonical genre or alias.
pub async fn resolve_genre_filter(
    db: &Database,
    genre_name: &str,
) -> Result<Option<uuid::Uuid>, sqlx::Error> {
    genre_label_service::get_genre_id_for_filter(db, genre_name).await
}
