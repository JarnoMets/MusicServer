use crate::db::Database;
use crate::models::genre::{Genre, GenreWithAliases};
use std::collections::HashMap;
use strsim::normalized_levenshtein;
use uuid::Uuid;

#[allow(dead_code)]
pub struct GenreLabelService;

/// Genre with track count for display
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct GenreWithCount {
    pub name: String,
    pub track_count: i64,
    pub id: Option<Uuid>,
    pub description: Option<String>,
}

/// List all canonical genres
pub async fn list_genres(db: &Database) -> Result<Vec<Genre>, sqlx::Error> {
    sqlx::query_as::<_, Genre>(
        "SELECT id, name, description, created_at, updated_at FROM genres ORDER BY name ASC"
    )
    .fetch_all(&db.pool)
    .await
}

/// Resolve a genre (canonical or alias) to its canonical name
/// Returns the canonical genre name if found, otherwise None
pub async fn resolve_to_canonical(
    db: &Database,
    genre_name: &str,
) -> Result<Option<String>, sqlx::Error> {
    let lower_name = genre_name.to_lowercase();

    // Check if it's an alias
    if let Some(canonical) = sqlx::query_scalar::<_, String>(
        "SELECT g.name FROM genre_aliases a JOIN genres g ON a.genre_id = g.id WHERE LOWER(a.alias) = $1"
    )
    .bind(&lower_name)
    .fetch_optional(&db.pool)
    .await?
    {
        return Ok(Some(canonical));
    }

    // Check if it's already a canonical genre
    if let Some(canonical) = sqlx::query_scalar::<_, String>(
        "SELECT name FROM genres WHERE LOWER(name) = $1"
    )
    .bind(&lower_name)
    .fetch_optional(&db.pool)
    .await?
    {
        return Ok(Some(canonical));
    }

    Ok(None)
}

/// List genres from actual music files with track counts
/// This combines genre and guessed_genre columns, preferring genre when set
/// It also joins with the genres table to get canonical IDs and descriptions
/// Aliases are resolved to their canonical genre names and aggregated
pub async fn list_genres_with_counts(db: &Database) -> Result<Vec<GenreWithCount>, sqlx::Error> {
    // 1. Get raw stats from DB
    let raw_counts: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT 
            COALESCE(NULLIF(genre, ''), guessed_genre) as raw_name,
            COUNT(*)::int8 as track_count
        FROM music_files
        WHERE (genre IS NOT NULL AND genre != '') 
           OR (guessed_genre IS NOT NULL AND guessed_genre != '')
        GROUP BY raw_name
        "#
    )
    .fetch_all(&db.pool)
    .await?;

    // 2. Load resolution map
    let resolver_map = get_genre_resolver_map(db).await?;
    
    // 3. Aggregate by resolved canonical name
    let mut resolved_stats: HashMap<String, GenreWithCount> = HashMap::new();
    
    for (raw_name, count) in raw_counts {
        let resolved_name = resolve_genre_name(&raw_name, &resolver_map);
        
        let entry = resolved_stats.entry(resolved_name.clone()).or_insert_with(|| GenreWithCount {
            name: resolved_name,
            track_count: 0,
            id: None,
            description: None,
        });
        entry.track_count += count;
    }

    // 4. Fill in canonical details (ID, description) from DB for matches
    let mut result_list: Vec<GenreWithCount> = resolved_stats.into_values().collect();
    
    let canonical_genres = list_genres(db).await?;
    let canonical_map: HashMap<String, (Uuid, Option<String>)> = canonical_genres
        .into_iter()
        .map(|g| (g.name.clone(), (g.id, g.description)))
        .collect();

    for genre in &mut result_list {
        if let Some((id, desc)) = canonical_map.get(&genre.name) {
            genre.id = Some(*id);
            genre.description = desc.clone();
        }
    }

    // 5. Final filter: only return genres with at least one track
    result_list.retain(|g| g.track_count > 0);

    // 6. Final sort by count
    result_list.sort_by(|a, b| b.track_count.cmp(&a.track_count).then(a.name.cmp(&b.name)));

    Ok(result_list)
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
        "INSERT INTO genres (id, name, description, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)"
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

/// Add an alias mapping raw tag -> canonical genre
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

/// Update a canonical genre
pub async fn update_genre(
    db: &Database,
    id: Uuid,
    name: &str,
    description: Option<&str>,
) -> Result<Option<Genre>, sqlx::Error> {
    let now = chrono::Utc::now();
    let result = sqlx::query(
        "UPDATE genres SET name = $1, description = $2, updated_at = $3 WHERE id = $4"
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
        created_at: now, // This is technically inaccurate but Genre struct doesn't have all fields correctly in this return
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

/// Resolve a raw genre/tag to a canonical genre name if possible
pub async fn canonicalize(db: &Database, raw: &str) -> Result<Option<String>, sqlx::Error> {
    let raw_lower = raw.trim().to_lowercase();

    // Direct alias match against the database
    if let Some(genre_name) = sqlx::query_scalar::<_, String>(
        "SELECT g.name FROM genre_aliases a JOIN genres g ON a.genre_id = g.id WHERE lower(a.alias) = lower($1)"
    ).bind(&raw_lower).fetch_optional(&db.pool).await? {
        return Ok(Some(genre_name));
    }

    // Try matching genre name directly
    if let Some(genre_name) =
        sqlx::query_scalar::<_, String>("SELECT name FROM genres WHERE lower(name) = lower($1)")
            .bind(&raw_lower)
            .fetch_optional(&db.pool)
            .await?
    {
        return Ok(Some(genre_name));
    }

    Ok(None)
}

// Expose a small normalization helper for unit testing
#[allow(dead_code)]
pub fn normalize_str(raw: &str) -> String {
    // Only basic trimming for external uses, real normalization is via canonicalize() or resolve_genre_name()
    raw.trim().to_string()
}

// Unit tests moved to end of file to satisfy clippy `items_after_test_module` lint

/// List detected genres/tags that currently have no mapping (for UIs to present)
pub async fn list_unmapped_tags(db: &Database, limit: i64) -> Result<Vec<String>, sqlx::Error> {
    // naive approach: look at artist_genres (cached detected values) and return those without mapping
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT genre FROM artist_genres WHERE genre IS NOT NULL AND NOT EXISTS (SELECT 1 FROM genre_aliases a JOIN genres g ON a.genre_id = g.id WHERE lower(a.alias) = lower(artist_genres.genre) OR lower(g.name) = lower(artist_genres.genre)) LIMIT $1"
    )
    .bind(limit)
    .fetch_all(&db.pool)
    .await?;

    Ok(rows)
}

/// Suggest similar canonical genres or aliases for a raw tag using fuzzy matching
pub async fn suggest_similar(
    db: &Database,
    raw: &str,
    limit: usize,
) -> Result<Vec<(String, f64)>, sqlx::Error> {
    let cleaned = raw.trim().to_lowercase();

    // Collect candidate names: genres and aliases
    let mut candidates: Vec<String> = Vec::new();
    let genre_names = sqlx::query_scalar::<_, String>("SELECT name FROM genres")
        .fetch_all(&db.pool)
        .await?;
    candidates.extend(genre_names.into_iter());
    let alias_names = sqlx::query_scalar::<_, String>("SELECT alias FROM genre_aliases")
        .fetch_all(&db.pool)
        .await?;
    candidates.extend(alias_names.into_iter());

    // Score and sort
    let mut scored: Vec<(String, f64)> = candidates
        .into_iter()
        .map(|c| {
            let score = normalized_levenshtein(&cleaned, &c.to_lowercase());
            (c, score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    Ok(scored)
}

/// When an alias is added (alias -> genre), backfill existing rows so guessed_genre and artist_genres are updated
pub async fn backfill_alias(
    db: &Database,
    alias: &str,
    canonical: &str,
) -> Result<u64, sqlx::Error> {
    // 1. Update music_files confirmed genre where matches alias (case-insensitive)
    let confirmed_result = sqlx::query("UPDATE music_files SET genre = $1, updated_at = NOW() WHERE lower(genre) = lower($2)")
        .bind(canonical)
        .bind(alias)
        .execute(&db.pool)
        .await?;

    // 2. Update music_files guessed_genre where matches alias (case-insensitive)
    let guessed_result = sqlx::query("UPDATE music_files SET guessed_genre = $1, updated_at = NOW() WHERE lower(guessed_genre) = lower($2)")
        .bind(canonical)
        .bind(alias)
        .execute(&db.pool)
        .await?;

    // 3. Update artist_genres (cache) as well
    let _ = sqlx::query(
        "UPDATE artist_genres SET genre = $1, last_updated = NOW() WHERE lower(genre) = lower($2)",
    )
    .bind(canonical)
    .bind(alias)
    .execute(&db.pool)
    .await?;

    Ok(confirmed_result.rows_affected() + guessed_result.rows_affected())
}

/// Preview how many rows would be affected by backfilling an alias to a canonical genre
pub async fn preview_backfill(db: &Database, alias: &str) -> Result<(i64, i64), sqlx::Error> {
    // Count music_files rows where either genre or guessed_genre matches alias (case-insensitive)
    let music_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM music_files WHERE (guessed_genre IS NOT NULL AND lower(guessed_genre) = lower($1)) OR (genre IS NOT NULL AND lower(genre) = lower($1))"
    )
    .bind(alias)
    .fetch_one(&db.pool)
    .await?;

    // Count artist_genres entries where genre matches alias
    let artist_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM artist_genres WHERE genre IS NOT NULL AND lower(genre) = lower($1)",
    )
    .bind(alias)
    .fetch_one(&db.pool)
    .await?;

    Ok((music_count, artist_count))
}

// unit tests are placed at the end of the file to satisfy clippy's
// `items_after_test_module` lint

/// Merge one genre into another. All tracks and aliases will be moved to the target genre.
pub async fn merge_genres(db: &Database, source_id: Uuid, target_id: Uuid) -> Result<(), sqlx::Error> {
    // Get genre names
    let source: (String,) = sqlx::query_as("SELECT name FROM genres WHERE id = $1")
        .bind(source_id)
        .fetch_one(&db.pool)
        .await?;
    let target: (String,) = sqlx::query_as("SELECT name FROM genres WHERE id = $1")
        .bind(target_id)
        .fetch_one(&db.pool)
        .await?;

    let source_name = source.0;
    let target_name = target.0;

    let mut tx = db.pool.begin().await?;

    // 1. Update confirmation status in music_files
    sqlx::query("UPDATE music_files SET genre = $1 WHERE genre = $2")
        .bind(&target_name)
        .bind(&source_name)
        .execute(&mut *tx)
        .await?;

    // 2. Update guessed_genre in music_files
    sqlx::query("UPDATE music_files SET guessed_genre = $1 WHERE guessed_genre = $2")
        .bind(&target_name)
        .bind(&source_name)
        .execute(&mut *tx)
        .await?;

    // 3. Update artist_genres (cache)
    sqlx::query("UPDATE artist_genres SET genre = $1 WHERE genre = $2")
        .bind(&target_name)
        .bind(&source_name)
        .execute(&mut *tx)
        .await?;

    // 4. Move aliases, handling conflicts
    // We try to update aliases from source to target. If an alias already exists for target, we delete it from source.
    let aliases: Vec<String> = sqlx::query_scalar("SELECT alias FROM genre_aliases WHERE genre_id = $1")
        .bind(source_id)
        .fetch_all(&mut *tx)
        .await?;

    for alias in aliases {
        // Check if target already has this alias or if the target name itself is this alias
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM genre_aliases WHERE alias = $1 AND genre_id = $2)")
            .bind(&alias)
            .bind(target_id)
            .fetch_one(&mut *tx)
            .await?;
        
        if exists || alias.to_lowercase() == target_name.to_lowercase() {
            // Conflict, just delete the source alias
            sqlx::query("DELETE FROM genre_aliases WHERE alias = $1 AND genre_id = $2")
                .bind(&alias)
                .bind(source_id)
                .execute(&mut *tx)
                .await?;
        } else {
            // Move it
            sqlx::query("UPDATE genre_aliases SET genre_id = $1 WHERE alias = $2 AND genre_id = $3")
                .bind(target_id)
                .bind(&alias)
                .bind(source_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    // 5. Delete the source genre
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
            "SELECT alias FROM genre_aliases WHERE genre_id = $1 ORDER BY alias ASC"
        )
        .bind(genre.id)
        .fetch_all(&db.pool)
        .await?;

        // track count: include all tracks matching this canonical genre or its aliases
        let track_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM music_files 
            WHERE 
                LOWER(COALESCE(NULLIF(genre, ''), guessed_genre)) = LOWER($1)
                OR LOWER(COALESCE(NULLIF(genre, ''), guessed_genre)) IN (
                    SELECT LOWER(alias) FROM genre_aliases WHERE genre_id = $2
                )
            "#
        )
        .bind(&genre.name)
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

/// Get a map of raw genre names to their canonicalized versions
pub async fn get_genre_resolver_map(db: &Database) -> Result<HashMap<String, String>, sqlx::Error> {
    let mut map = HashMap::new();

    // 1. Map canonical genre names to themselves (lower-case index)
    let genres = list_genres(db).await?;
    for g in &genres {
        map.insert(g.name.to_lowercase(), g.name.clone());
    }

    // 2. Map aliases to their canonical names
    let aliases: Vec<(String, String)> = sqlx::query_as(
        "SELECT LOWER(a.alias), g.name FROM genre_aliases a JOIN genres g ON a.genre_id = g.id"
    )
    .fetch_all(&db.pool)
    .await?;

    for (alias, canonical) in aliases {
        map.insert(alias, canonical);
    }

    Ok(map)
}

/// Helper to resolve a name using the provided map
pub fn resolve_genre_name(name: &str, resolver_map: &HashMap<String, String>) -> String {
    let lower = name.to_lowercase();
    
    // Direct match in map (canonical or alias)
    if let Some(canonical) = resolver_map.get(&lower) {
        return canonical.clone();
    }

    // Return the name as is if no mapping exists
    name.to_string()
}

// unit tests are placed at the end of the file to satisfy clippy's
// `items_after_test_module` lint

#[cfg(test)]
mod tests {
    use super::{normalize_str, resolve_genre_name};
    use std::collections::HashMap;

    #[test]
    fn normalize_str_trims_whitespace() {
        assert_eq!(normalize_str("  Drum & Bass  "), "Drum & Bass");
    }

    #[test]
    fn resolve_genre_name_maps_aliases_case_insensitively() {
        let mut map = HashMap::new();
        map.insert("drum & bass".to_string(), "Drum & Bass".to_string());
        map.insert("dnb".to_string(), "Drum & Bass".to_string());

        assert_eq!(resolve_genre_name("dnb", &map), "Drum & Bass");
        assert_eq!(resolve_genre_name("DNB", &map), "Drum & Bass");
    }

    #[test]
    fn resolve_genre_name_returns_original_when_unmapped() {
        let map = HashMap::new();
        assert_eq!(resolve_genre_name("Unknown Genre", &map), "Unknown Genre");
    }
}
