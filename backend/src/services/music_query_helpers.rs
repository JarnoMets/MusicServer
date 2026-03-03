// Shared SQL helpers for the `music_files` table.
//
// The full column list and filter logic were duplicated across many services.
// This module provides a single source of truth.

use crate::models::MusicQueryParams;
use crate::db::Database;

use sqlx::QueryBuilder;
use sqlx::Postgres;

/// All columns selected from `music_files` in the standard order.
pub const MUSIC_FILE_COLUMNS: &str =
    "id, title, artist, album, genre, guessed_genre, release_date, duration, \
     file_path, track_number, file_hash, bpm, initial_key, beat_grid_offset, beat_map, metadata_analyzed, \
     created_at, updated_at";

/// Build a `SELECT <columns> FROM music_files` prefix.
pub fn select_music_files() -> String {
    format!("SELECT {} FROM music_files", MUSIC_FILE_COLUMNS)
}

/// Get all canonical genres that an alias or genre name maps to
/// This helps us filter music when the user selects a canonical genre
pub async fn get_all_matching_genres(
    db: &Database,
    genre_name: &str,
) -> Result<Vec<String>, sqlx::Error> {
    let lower_name = genre_name.to_lowercase();
    
    // 1. Find the canonical name this input resolves to
    let canonical_result: Option<String> = sqlx::query_scalar(
        r#"
        SELECT name FROM genres WHERE LOWER(name) = $1
        UNION ALL
        SELECT g.name FROM genre_aliases a JOIN genres g ON a.genre_id = g.id WHERE LOWER(a.alias) = $1
        LIMIT 1
        "#
    )
    .bind(&lower_name)
    .fetch_optional(&db.pool)
    .await?;

    let mut results = Vec::new();

    if let Some(canonical) = canonical_result {
        results.push(canonical.clone());
        
        // 2. Find all aliases that point to this canonical genre
        let aliases: Vec<String> = sqlx::query_scalar(
            "SELECT alias FROM genre_aliases a JOIN genres g ON a.genre_id = g.id WHERE g.name = $1"
        )
        .bind(&canonical)
        .fetch_all(&db.pool)
        .await?;
        
        for alias in aliases {
            if !results.contains(&alias) {
                results.push(alias);
            }
        }
    } else {
        // If not found in genres or aliases, just match the name itself
        results.push(genre_name.to_string());
    }

    Ok(results)
}

/// Append WHERE-clause filters from `MusicQueryParams` onto a `QueryBuilder`.
///
/// The builder is expected to already contain `… WHERE 1=1` or similar so
/// that each filter can be appended with `AND`.
pub fn apply_music_filters(builder: &mut QueryBuilder<'_, Postgres>, params: &MusicQueryParams) {
    if let Some(search) = &params.search {
        let like = format!("%{}%", search.to_lowercase());
        builder
            .push(" AND (LOWER(title) LIKE ")
            .push_bind(like.clone())
            .push(" OR LOWER(artist) LIKE ")
            .push_bind(like.clone())
            .push(" OR LOWER(album) LIKE ")
            .push_bind(like)
            .push(")");
    }

    if let Some(genre) = &params.genre {
        // Filter for both confirmed genre and guessed_genre
        // This matches exact genre names as they appear in the database
        builder
            .push(" AND (genre = ")
            .push_bind(genre.clone())
            .push(" OR guessed_genre = ")
            .push_bind(genre.clone())
            .push(")");
    }

    if let Some(artist) = &params.artist {
        builder
            .push(" AND LOWER(artist) = LOWER(")
            .push_bind(artist.clone())
            .push(")");
    }

    if params.unconfirmed_only.unwrap_or(false) {
        builder.push(
            " AND (genre IS NULL OR genre = '') AND guessed_genre IS NOT NULL AND guessed_genre != ''",
        );
    }

    if params.missing_metadata.unwrap_or(false) {
        builder.push(
            " AND (genre IS NULL OR genre = '' OR release_date IS NULL OR album IS NULL OR album = '')",
        );
    }
}
