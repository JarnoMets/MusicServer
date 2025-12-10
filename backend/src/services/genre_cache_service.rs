use crate::db::Database;
use crate::models::ArtistGenre;
use uuid::Uuid;

#[allow(dead_code)]
pub struct GenreCacheService;

/// Get cached genre for an artist
pub async fn get_cached_genre(
    db: &Database,
    artist_name: &str,
) -> Result<Option<String>, sqlx::Error> {
    let result =
        sqlx::query_scalar::<_, String>("SELECT genre FROM artist_genres WHERE artist_name = $1")
            .bind(artist_name)
            .fetch_optional(&db.pool)
            .await?;

    Ok(result)
}

/// Check if an artist is in the cache
pub async fn is_cached(db: &Database, artist_name: &str) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM artist_genres WHERE artist_name = $1)",
    )
    .bind(artist_name)
    .fetch_one(&db.pool)
    .await?;

    Ok(exists)
}

/// Store a genre for an artist in the cache
pub async fn cache_genre(
    db: &Database,
    artist_name: &str,
    genre: &str,
) -> Result<ArtistGenre, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO artist_genres (id, artist_name, genre, last_updated, created_at) VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (artist_name) DO UPDATE SET genre = $3, last_updated = $4"
    )
    .bind(id)
    .bind(artist_name)
    .bind(genre)
    .bind(now)
    .bind(now)
    .execute(&db.pool)
    .await?;

    // Re-fetch to get the actual values
    let artist_genre = sqlx::query_as::<_, ArtistGenre>(
        "SELECT id, artist_name, genre, last_updated, created_at FROM artist_genres WHERE artist_name = $1"
    )
    .bind(artist_name)
    .fetch_one(&db.pool)
    .await?;

    Ok(artist_genre)
}

/// Get all cached artist genres
pub async fn get_all_cached_genres(db: &Database) -> Result<Vec<ArtistGenre>, sqlx::Error> {
    sqlx::query_as::<_, ArtistGenre>(
        "SELECT id, artist_name, genre, last_updated, created_at FROM artist_genres ORDER BY artist_name ASC"
    )
    .fetch_all(&db.pool)
    .await
}

/// Delete a cached genre entry
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
