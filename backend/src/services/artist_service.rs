use crate::db::Database;
use crate::models::{ArtistSummary, MusicFile};
use uuid::Uuid;

/// Get all unique artists from the music library
#[allow(dead_code)]
pub async fn get_all_artists(db: &Database) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT artist FROM music_files WHERE artist IS NOT NULL AND artist != '' ORDER BY artist ASC"
    )
    .fetch_all(&db.pool)
    .await
}

/// Get all artists with their genre and song count
pub async fn get_all_artists_with_summary(db: &Database) -> Result<Vec<ArtistSummary>, sqlx::Error> {
    sqlx::query_as::<_, (String, Option<String>, i64)>(
        r#"
        SELECT 
            mf.artist,
            g.name as genre,
            COUNT(mf.id) as song_count
        FROM music_files mf
        LEFT JOIN artist_genres ag ON ag.artist_name = mf.artist
        LEFT JOIN genres g ON ag.genre_id = g.id
        WHERE mf.artist IS NOT NULL AND mf.artist != ''
        GROUP BY mf.artist, g.name
        ORDER BY mf.artist ASC
        "#
    )
    .fetch_all(&db.pool)
    .await
    .map(|rows| {
        rows.into_iter().map(|(name, genre, song_count)| {
            ArtistSummary {
                name,
                genre,
                song_count,
            }
        }).collect()
    })
}

/// Get all music files by a specific artist (exact match)
#[allow(dead_code)]
pub async fn get_music_by_artist(
    db: &Database,
    artist: &str,
) -> Result<Vec<MusicFile>, sqlx::Error> {
    let sql = format!("{} WHERE mf.artist = $1 ORDER BY mf.title ASC", crate::services::music_query_helpers::select_music_files());
    sqlx::query_as::<_, MusicFile>(&sql)
    .bind(artist)
    .fetch_all(&db.pool)
    .await
}

/// Get all music files featuring an artist (including collaborations, remixes, features)
pub async fn get_music_featuring_artist(
    db: &Database,
    artist: &str,
) -> Result<Vec<MusicFile>, sqlx::Error> {
    // Search for artist in title (for features/remixes) or as main artist
    let search_pattern = format!("%{}%", artist);
    let sql = format!(
        "{} WHERE mf.artist = $1 OR mf.artist ILIKE $2 OR mf.title ILIKE $2 ORDER BY CASE WHEN mf.artist = $1 THEN 0 ELSE 1 END, mf.title ASC",
        crate::services::music_query_helpers::select_music_files()
    );
    
    sqlx::query_as::<_, MusicFile>(&sql)
    .bind(artist)
    .bind(&search_pattern)
    .fetch_all(&db.pool)
    .await
}

/// Update genre for all files by a specific artist using the canonical genre_id
#[allow(dead_code)]
pub async fn update_genre_for_artist(
    db: &Database,
    artist: &str,
    genre_id: Uuid,
) -> Result<u64, sqlx::Error> {
    crate::services::genre_label_service::assign_genre_to_artist_tracks(db, artist, genre_id).await
}

/// Assign auto-detected genre to all tracks by an artist (does not overwrite user-confirmed genres)
pub async fn assign_auto_genre_for_artist(
    db: &Database,
    artist: &str,
    genre_id: Uuid,
) -> Result<u64, sqlx::Error> {
    crate::services::genre_label_service::assign_genre_to_artist_tracks(db, artist, genre_id).await
}

/// Set genre for an artist (updates artist_genres table)
pub async fn set_artist_genre(
    db: &Database,
    artist: &str,
    genre_id: Uuid,
    raw_tag: Option<&str>,
) -> Result<(), sqlx::Error> {
    crate::services::genre_cache_service::cache_genre_id(db, artist, genre_id, raw_tag).await
}

/// Rename an artist - updates all music files and artist_genres table
/// If the new name already exists, this effectively merges the artists
pub async fn rename_artist(
    db: &Database,
    old_name: &str,
    new_name: &str,
) -> Result<RenameArtistResult, sqlx::Error> {
    // Check if target artist already exists (for merge info)
    let existing_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM music_files WHERE artist = $1"
    )
    .bind(new_name)
    .fetch_one(&db.pool)
    .await?;

    // Update all music files with old artist name
    let result = sqlx::query(
        "UPDATE music_files SET artist = $1, updated_at = NOW() WHERE artist = $2"
    )
    .bind(new_name)
    .bind(old_name)
    .execute(&db.pool)
    .await?;

    let tracks_updated = result.rows_affected();

    // Handle artist_genres table
    // Check if new artist already has a genre entry
    let new_has_genre: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM artist_genres WHERE artist_name = $1)"
    )
    .bind(new_name)
    .fetch_one(&db.pool)
    .await?;

    if !new_has_genre {
        // Move the old artist's genre to the new name
        let _ = sqlx::query(
            "UPDATE artist_genres SET artist_name = $1 WHERE artist_name = $2"
        )
        .bind(new_name)
        .bind(old_name)
        .execute(&db.pool)
        .await;
    } else {
        // Delete the old artist's genre entry (merge scenario)
        let _ = sqlx::query(
            "DELETE FROM artist_genres WHERE artist_name = $1"
        )
        .bind(old_name)
        .execute(&db.pool)
        .await;
    }

    Ok(RenameArtistResult {
        old_name: old_name.to_string(),
        new_name: new_name.to_string(),
        tracks_updated,
        was_merge: existing_count > 0,
    })
}

#[derive(Debug, serde::Serialize)]
pub struct RenameArtistResult {
    pub old_name: String,
    pub new_name: String,
    pub tracks_updated: u64,
    pub was_merge: bool,
}

/// Ensure an artist exists in the artist_genres table
/// If they don't exist, adds them with detection_status='pending' (to be detected later)
pub async fn ensure_artist_exists(db: &Database, artist: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO artist_genres (artist_name, detection_status) VALUES ($1, 'pending') ON CONFLICT (artist_name) DO NOTHING"
    )
    .bind(artist)
    .execute(&db.pool)
    .await?;
    Ok(())
}

/// Ensure multiple artists exist in the artist_genres table
pub async fn ensure_artists_exist(db: &Database, artists: &[String]) -> Result<(), sqlx::Error> {
    for artist in artists {
        if !artist.trim().is_empty() {
            ensure_artist_exists(db, artist.trim()).await?;
        }
    }
    Ok(())
}

/// Reprocess all music files to extract artists from artist field and titles
/// This is useful for backfilling after the artist parser was added
pub async fn reprocess_all_for_artists(db: &Database) -> Result<u64, sqlx::Error> {
    use crate::services::artist_parser;
    
    // Get all music files
    let files: Vec<(String, Option<String>, String)> = sqlx::query_as(
        "SELECT id::text, artist, title FROM music_files"
    )
    .fetch_all(&db.pool)
    .await?;
    
    let mut artists_added = 0u64;
    
    for (_id, artist, title) in files {
        let parsed = artist_parser::parse_artists(
            artist.as_deref(),
            Some(&title),
        );
        
        let all_artists = parsed.all_artists();
        for artist_name in all_artists {
            if !artist_name.trim().is_empty() {
                // Check if already exists
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM artist_genres WHERE artist_name = $1)"
                )
                .bind(&artist_name)
                .fetch_one(&db.pool)
                .await?;
                
                if !exists {
                    ensure_artist_exists(db, &artist_name).await?;
                    artists_added += 1;
                }
            }
        }
    }
    
    Ok(artists_added)
}
