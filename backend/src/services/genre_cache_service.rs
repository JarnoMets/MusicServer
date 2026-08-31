use crate::db::Database;
use crate::models::ArtistGenre;
use uuid::Uuid;

#[allow(dead_code)]
pub struct GenreCacheService;

/// Get cached genre_id for an artist (returns None if not cached or genre_id not yet resolved)
pub async fn get_cached_genre_id(
    db: &Database,
    artist_name: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT genre_id FROM artist_genres WHERE artist_name = $1 AND genre_id IS NOT NULL",
    )
    .bind(artist_name)
    .fetch_optional(&db.pool)
    .await
}

/// Get the detection status for an artist entry
pub async fn get_detection_status(
    db: &Database,
    artist_name: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT detection_status FROM artist_genres WHERE artist_name = $1",
    )
    .bind(artist_name)
    .fetch_optional(&db.pool)
    .await
}

/// Check if an artist is in the cache (regardless of detection status)
pub async fn is_cached(db: &Database, artist_name: &str) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM artist_genres WHERE artist_name = $1)",
    )
    .bind(artist_name)
    .fetch_one(&db.pool)
    .await?;
    Ok(exists)
}

/// Cache a resolved genre_id for an artist and mark detection as successful
pub async fn cache_genre_id(
    db: &Database,
    artist_name: &str,
    genre_id: Uuid,
    raw_tag: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO artist_genres (id, artist_name, genre_id, raw_detected_tag, detection_status, last_updated, created_at)
        VALUES (gen_random_uuid(), $1, $2, $3, 'detected', NOW(), NOW())
        ON CONFLICT (artist_name) DO UPDATE
            SET genre_id = $2,
                raw_detected_tag = COALESCE($3, artist_genres.raw_detected_tag),
                detection_status = 'detected',
                last_updated = NOW()
        "#,
    )
    .bind(artist_name)
    .bind(genre_id)
    .bind(raw_tag)
    .execute(&db.pool)
    .await?;
    Ok(())
}

/// Cache a raw tag for an artist when the tag couldn't be resolved to a canonical genre yet
pub async fn cache_raw_tag(
    db: &Database,
    artist_name: &str,
    raw_tag: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO artist_genres (id, artist_name, raw_detected_tag, detection_status, last_updated, created_at)
        VALUES (gen_random_uuid(), $1, $2, 'pending', NOW(), NOW())
        ON CONFLICT (artist_name) DO UPDATE
            SET raw_detected_tag = $2,
                last_updated = NOW()
        "#,
    )
    .bind(artist_name)
    .bind(raw_tag)
    .execute(&db.pool)
    .await?;
    Ok(())
}

/// Mark an artist as not found so the scheduler skips them
pub async fn mark_not_found(db: &Database, artist_name: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO artist_genres (id, artist_name, detection_status, last_updated, created_at)
        VALUES (gen_random_uuid(), $1, 'not_found', NOW(), NOW())
        ON CONFLICT (artist_name) DO UPDATE
            SET detection_status = 'not_found',
                last_updated = NOW()
        "#,
    )
    .bind(artist_name)
    .execute(&db.pool)
    .await?;
    Ok(())
}

/// Get all cached artist genres (joined with genre name for display)
pub async fn get_all_cached_genres(db: &Database) -> Result<Vec<ArtistGenre>, sqlx::Error> {
    sqlx::query_as::<_, ArtistGenre>(
        r#"
        SELECT ag.id, ag.artist_name, ag.genre_id, g.name as genre_name,
               ag.raw_detected_tag, ag.detection_status, ag.last_updated, ag.created_at
        FROM artist_genres ag
        LEFT JOIN genres g ON ag.genre_id = g.id
        ORDER BY ag.artist_name ASC
        "#,
    )
    .fetch_all(&db.pool)
    .await
}

/// Delete a cached genre entry
#[allow(dead_code)]
pub async fn delete_cached_genre(db: &Database, artist_name: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM artist_genres WHERE artist_name = $1")
        .bind(artist_name)
        .execute(&db.pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Clear all cached genres
pub async fn clear_all_cache(db: &Database) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM artist_genres")
        .execute(&db.pool)
        .await?;
    Ok(result.rows_affected())
}
