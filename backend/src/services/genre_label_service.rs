use crate::db::Database;
use crate::models::genre::{Genre, GenreWithAliases};
use strsim::normalized_levenshtein;
use uuid::Uuid;

#[allow(dead_code)]
pub struct GenreLabelService;

/// Genre with track count for display
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct GenreWithCount {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub track_count: i64,
}

/// List all canonical genres
pub async fn list_genres(db: &Database) -> Result<Vec<Genre>, sqlx::Error> {
    sqlx::query_as::<_, Genre>(
        "SELECT id, name, description, created_at, updated_at FROM genres ORDER BY name ASC"
    )
    .fetch_all(&db.pool)
    .await
}

/// List genres from actual music files with track counts (FK-based, no alias resolution needed)
pub async fn list_genres_with_counts(db: &Database) -> Result<Vec<GenreWithCount>, sqlx::Error> {
    sqlx::query_as::<_, GenreWithCount>(
        r#"
        SELECT g.id, g.name, g.description, COUNT(mf.id)::int8 as track_count
        FROM genres g
        INNER JOIN music_files mf ON mf.genre_id = g.id
        GROUP BY g.id
        HAVING COUNT(mf.id) > 0
        ORDER BY track_count DESC, g.name ASC
        "#,
    )
    .fetch_all(&db.pool)
    .await
}

/// Resolve a raw tag/name to its canonical genre_id.
/// Returns Some(id) if matched via direct name or alias, None otherwise.
pub async fn resolve_to_genre_id(
    db: &Database,
    raw: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let lower = raw.trim().to_lowercase();

    // Check alias first
    if let Some(id) = sqlx::query_scalar::<_, Uuid>(
        "SELECT genre_id FROM genre_aliases WHERE LOWER(alias) = $1",
    )
    .bind(&lower)
    .fetch_optional(&db.pool)
    .await?
    {
        return Ok(Some(id));
    }

    // Check canonical genre name
    if let Some(id) = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM genres WHERE LOWER(name) = $1",
    )
    .bind(&lower)
    .fetch_optional(&db.pool)
    .await?
    {
        return Ok(Some(id));
    }

    Ok(None)
}

/// Resolve a raw tag to genre_id; creates a new canonical genre if no match found.
/// Used by the detection pipeline so unrecognised API tags end up in the taxonomy.
pub async fn resolve_or_create_genre_id(
    db: &Database,
    raw: &str,
) -> Result<Uuid, sqlx::Error> {
    if let Some(id) = resolve_to_genre_id(db, raw).await? {
        return Ok(id);
    }

    // Create a new canonical genre from this raw tag
    let trimmed = raw.trim();
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    sqlx::query(
        "INSERT INTO genres (id, name, created_at, updated_at) VALUES ($1, $2, $3, $4) ON CONFLICT (name) DO NOTHING",
    )
    .bind(id)
    .bind(trimmed)
    .bind(now)
    .bind(now)
    .execute(&db.pool)
    .await?;

    // Re-fetch in case of conflict
    let actual_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM genres WHERE LOWER(name) = LOWER($1)")
        .bind(trimmed)
        .fetch_one(&db.pool)
        .await?;

    Ok(actual_id)
}

/// Assign a canonical genre to a track — the single authoritative write path.
pub async fn assign_genre_to_track(
    db: &Database,
    track_id: Uuid,
    genre_id: Uuid,
    source: &str, // 'user', 'auto', or 'file_tag'
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE music_files SET genre_id = $1, genre_source = $2, updated_at = NOW() WHERE id = $3",
    )
    .bind(genre_id)
    .bind(source)
    .bind(track_id)
    .execute(&db.pool)
    .await?;
    Ok(())
}

/// Set the genre_id for all tracks by an artist (used when artist genre is set manually)
pub async fn assign_genre_to_artist_tracks(
    db: &Database,
    artist_name: &str,
    genre_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE music_files SET genre_id = $1, genre_source = 'auto', updated_at = NOW() WHERE LOWER(artist) = LOWER($2) AND (genre_source IS NULL OR genre_source = 'auto')",
    )
    .bind(genre_id)
    .bind(artist_name)
    .execute(&db.pool)
    .await?;
    Ok(result.rows_affected())
}

/// Get the genre_id to use for filtering (resolves name or alias → id)
pub async fn get_genre_id_for_filter(
    db: &Database,
    genre_name: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    resolve_to_genre_id(db, genre_name).await
}

/// Create a new canonical genre
pub async fn create_genre(
    db: &Database,
    name: &str,
    description: Option<&str>,
) -> Result<Genre, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO genres (id, name, description, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(now)
    .bind(now)
    .execute(&db.pool)
    .await?;

    Ok(Genre {
        id,
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
        created_at: now,
        updated_at: now,
    })
}

/// Update a canonical genre
pub async fn update_genre(
    db: &Database,
    id: Uuid,
    name: &str,
    description: Option<&str>,
) -> Result<Option<Genre>, sqlx::Error> {
    let now = chrono::Utc::now();
    let result = sqlx::query(
        "UPDATE genres SET name = $1, description = $2, updated_at = $3 WHERE id = $4",
    )
    .bind(name)
    .bind(description)
    .bind(now)
    .bind(id)
    .execute(&db.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    Ok(Some(Genre {
        id,
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
        created_at: now,
        updated_at: now,
    }))
}

/// Delete a canonical genre
pub async fn delete_genre(db: &Database, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM genres WHERE id = $1")
        .bind(id)
        .execute(&db.pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Add an alias mapping raw tag → canonical genre
pub async fn add_alias(db: &Database, alias: &str, genre_id: Uuid) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO genre_aliases (id, alias, genre_id, created_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind(id)
    .bind(alias)
    .bind(genre_id)
    .execute(&db.pool)
    .await?;
    Ok(())
}

/// List raw detected tags from artist_genres that have no canonical genre mapping yet
pub async fn list_unmapped_tags(db: &Database, limit: i64) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT raw_detected_tag
        FROM artist_genres
        WHERE raw_detected_tag IS NOT NULL
          AND genre_id IS NULL
          AND detection_status != 'not_found'
        ORDER BY raw_detected_tag ASC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(&db.pool)
    .await
}

/// Suggest similar canonical genres or aliases for a raw tag using fuzzy matching
pub async fn suggest_similar(
    db: &Database,
    raw: &str,
    limit: usize,
) -> Result<Vec<(String, f64)>, sqlx::Error> {
    let cleaned = raw.trim().to_lowercase();

    let mut candidates: Vec<String> = sqlx::query_scalar::<_, String>("SELECT name FROM genres")
        .fetch_all(&db.pool)
        .await?;
    let aliases = sqlx::query_scalar::<_, String>("SELECT alias FROM genre_aliases")
        .fetch_all(&db.pool)
        .await?;
    candidates.extend(aliases);

    let mut scored: Vec<(String, f64)> = candidates
        .into_iter()
        .map(|c| {
            let score = normalized_levenshtein(&cleaned, &c.to_lowercase());
            (c, score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.retain(|(_, score)| *score >= 0.4);
    scored.truncate(limit);
    Ok(scored)
}

/// Preview how many rows would be affected by backfilling an alias to a genre
pub async fn preview_backfill(
    db: &Database,
    alias: &str,
    target_genre_id: Uuid,
) -> Result<(i64, i64), sqlx::Error> {
    // Tracks that currently have no genre_id or a different genre_id and whose
    // raw tag (old genre/guessed_genre text, now gone) matched the alias.
    // Since we've migrated, preview instead counts how many existing artist_genres
    // have raw_detected_tag matching the alias but no genre_id assigned.
    let artist_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM artist_genres WHERE LOWER(raw_detected_tag) = LOWER($1) AND genre_id IS NULL",
    )
    .bind(alias)
    .fetch_one(&db.pool)
    .await?;

    // Tracks that would be updated: those where artist has matching raw tag but no genre_id
    let music_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM music_files mf
        JOIN artist_genres ag ON LOWER(mf.artist) = LOWER(ag.artist_name)
        WHERE LOWER(ag.raw_detected_tag) = LOWER($1)
          AND ag.genre_id IS NULL
          AND (mf.genre_id IS NULL OR mf.genre_id != $2)
        "#,
    )
    .bind(alias)
    .bind(target_genre_id)
    .fetch_one(&db.pool)
    .await?;

    Ok((music_count, artist_count))
}

/// When an alias is added, backfill: update artist_genres and their tracks where the raw_detected_tag matches
pub async fn backfill_alias(
    db: &Database,
    alias: &str,
    canonical_genre_id: Uuid,
) -> Result<u64, sqlx::Error> {
    // 1. Update artist_genres where raw_detected_tag matches the alias
    let artist_result = sqlx::query(
        "UPDATE artist_genres SET genre_id = $1, detection_status = 'detected', last_updated = NOW() WHERE LOWER(raw_detected_tag) = LOWER($2) AND genre_id IS NULL",
    )
    .bind(canonical_genre_id)
    .bind(alias)
    .execute(&db.pool)
    .await?;

    // 2. Update tracks for those artists that still have no user-confirmed genre
    let track_result = sqlx::query(
        r#"
        UPDATE music_files mf
        SET genre_id = $1, genre_source = 'auto', updated_at = NOW()
        FROM artist_genres ag
        WHERE LOWER(ag.artist_name) = LOWER(mf.artist)
          AND ag.genre_id = $1
          AND (mf.genre_id IS NULL OR mf.genre_source = 'auto')
        "#,
    )
    .bind(canonical_genre_id)
    .execute(&db.pool)
    .await?;

    Ok(artist_result.rows_affected() + track_result.rows_affected())
}

/// Merge one genre into another. All tracks and aliases will be moved to the target genre.
pub async fn merge_genres(db: &Database, source_id: Uuid, target_id: Uuid) -> Result<(), sqlx::Error> {
    let mut tx = db.pool.begin().await?;

    // 1. Move all tracks from source to target
    sqlx::query("UPDATE music_files SET genre_id = $1, updated_at = NOW() WHERE genre_id = $2")
        .bind(target_id)
        .bind(source_id)
        .execute(&mut *tx)
        .await?;

    // 2. Move artist_genres from source to target
    sqlx::query("UPDATE artist_genres SET genre_id = $1, last_updated = NOW() WHERE genre_id = $2")
        .bind(target_id)
        .bind(source_id)
        .execute(&mut *tx)
        .await?;

    // 3. Move aliases, handling conflicts
    let aliases: Vec<String> = sqlx::query_scalar(
        "SELECT alias FROM genre_aliases WHERE genre_id = $1",
    )
    .bind(source_id)
    .fetch_all(&mut *tx)
    .await?;

    let target_name: String = sqlx::query_scalar("SELECT name FROM genres WHERE id = $1")
        .bind(target_id)
        .fetch_one(&mut *tx)
        .await?;

    for alias in aliases {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM genre_aliases WHERE LOWER(alias) = LOWER($1) AND genre_id = $2)",
        )
        .bind(&alias)
        .bind(target_id)
        .fetch_one(&mut *tx)
        .await?;

        if exists || alias.to_lowercase() == target_name.to_lowercase() {
            sqlx::query("DELETE FROM genre_aliases WHERE alias = $1 AND genre_id = $2")
                .bind(&alias)
                .bind(source_id)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query("UPDATE genre_aliases SET genre_id = $1 WHERE alias = $2 AND genre_id = $3")
                .bind(target_id)
                .bind(&alias)
                .bind(source_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    // 4. Delete the source genre
    sqlx::query("DELETE FROM genres WHERE id = $1")
        .bind(source_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

/// List all canonical genres with their aliases and track counts
pub async fn list_genres_extended(db: &Database) -> Result<Vec<GenreWithAliases>, sqlx::Error> {
    let genres = list_genres(db).await?;
    let mut result = Vec::with_capacity(genres.len());

    for genre in genres {
        let aliases: Vec<String> = sqlx::query_scalar(
            "SELECT alias FROM genre_aliases WHERE genre_id = $1 ORDER BY alias ASC",
        )
        .bind(genre.id)
        .fetch_all(&db.pool)
        .await?;

        let track_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM music_files WHERE genre_id = $1",
        )
        .bind(genre.id)
        .fetch_one(&db.pool)
        .await?;

        result.push(GenreWithAliases {
            id: genre.id,
            name: genre.name,
            description: genre.description,
            aliases,
            track_count,
            created_at: genre.created_at,
            updated_at: genre.updated_at,
        });
    }

    Ok(result)
}

// unit tests
#[cfg(test)]
mod tests {
    use super::GenreLabelService;
}
