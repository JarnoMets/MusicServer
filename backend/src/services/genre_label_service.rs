use crate::db::Database;
use crate::models::genre::Genre;
use deunicode::deunicode;
use strsim::normalized_levenshtein;
use uuid::Uuid;

#[allow(dead_code)]
pub struct GenreLabelService;

/// Genre with track count for display
#[derive(Debug, serde::Serialize)]
pub struct GenreWithCount {
    pub name: String,
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

/// List genres from actual music files with track counts
/// This combines genre and guessed_genre columns, preferring genre when set
pub async fn list_genres_with_counts(db: &Database) -> Result<Vec<GenreWithCount>, sqlx::Error> {
    sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT 
            COALESCE(NULLIF(genre, ''), guessed_genre) as effective_genre,
            COUNT(*) as track_count
        FROM music_files
        WHERE COALESCE(NULLIF(genre, ''), guessed_genre) IS NOT NULL 
          AND COALESCE(NULLIF(genre, ''), guessed_genre) != ''
        GROUP BY effective_genre
        ORDER BY track_count DESC, effective_genre ASC
        "#
    )
    .fetch_all(&db.pool)
    .await
    .map(|rows| {
        rows.into_iter().map(|(name, track_count)| {
            GenreWithCount {
                name,
                track_count,
            }
        }).collect()
    })
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

/// Resolve a raw genre/tag to a canonical genre name if possible
pub async fn canonicalize(db: &Database, raw: &str) -> Result<Option<String>, sqlx::Error> {
    // Basic normalization: lowercase, remove diacritics, map '&' to 'and', strip punctuation, collapse whitespace
    let mut cleaned = raw.trim().to_lowercase();
    // strip diacritics (é -> e)
    cleaned = deunicode(&cleaned);
    // map ampersand to 'and'
    cleaned = cleaned.replace('&', " and ");
    // remove punctuation except whitespace
    cleaned = cleaned.replace(|c: char| !c.is_alphanumeric() && !c.is_whitespace(), " ");
    cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

    // Common abbreviation expansions
    let mut expanded = cleaned.clone();
    let abbr_map = vec![
        ("dnb", "drum and bass"),
        ("drum'n'bass", "drum and bass"),
        ("edm", "electronic"),
        ("hiphop", "hip-hop"),
        ("hip hop", "hip-hop"),
    ];

    for (k, v) in abbr_map.iter() {
        if cleaned == *k || cleaned.contains(k) {
            expanded = cleaned.replacen(k, v, 1);
            break;
        }
    }

    // direct alias match against normalized/expanded forms
    for candidate in vec![expanded.as_str(), cleaned.as_str()] {
        if let Some(genre_name) = sqlx::query_scalar::<_, String>(
            "SELECT g.name FROM genre_aliases a JOIN genres g ON a.genre_id = g.id WHERE lower(a.alias) = lower($1)"
        ).bind(candidate).fetch_optional(&db.pool).await? {
            return Ok(Some(genre_name));
        }

        // try matching genre name directly
        if let Some(genre_name) =
            sqlx::query_scalar::<_, String>("SELECT name FROM genres WHERE lower(name) = lower($1)")
                .bind(candidate)
                .fetch_optional(&db.pool)
                .await?
        {
            return Ok(Some(genre_name));
        }
    }

    Ok(None)
}

// Expose a small normalization helper for unit testing
pub fn normalize_str(raw: &str) -> String {
    let mut cleaned = raw.trim().to_lowercase();
    cleaned = deunicode(&cleaned);
    cleaned = cleaned.replace('&', " and ");
    cleaned = cleaned.replace(|c: char| !c.is_alphanumeric() && !c.is_whitespace(), " ");
    cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    // abbreviation expand first match — check the raw input first so punctuation variants match
    let raw_lower = raw.trim().to_lowercase();
    let abbr_map = vec![
        ("dnb", "drum and bass"),
        ("drum'n'bass", "drum and bass"),
        ("edm", "electronic"),
        ("hiphop", "hip-hop"),
        ("hip hop", "hip-hop"),
    ];
    for (k, v) in abbr_map.iter() {
        if raw_lower == *k || raw_lower.contains(k) {
            return raw_lower.replacen(k, v, 1);
        }
        if cleaned == *k || cleaned.contains(k) {
            return cleaned.replacen(k, v, 1);
        }
    }

    cleaned
}

#[cfg(test)]
mod tests {
    use super::normalize_str;
    use strsim::normalized_levenshtein;

    #[test]
    fn test_normalize_variants() {
        assert_eq!(normalize_str("DnB"), "drum and bass");
        assert_eq!(normalize_str("drum'n'bass"), "drum and bass");
        assert_eq!(
            normalize_str("Électronique & Dance"),
            "electronique and dance"
        );
        assert_eq!(normalize_str("Hip Hop"), "hip-hop");
    }

    #[test]
    fn test_suggestion_score() {
        let raw = "drum n bass";
        let candidate = "Drum and Bass";
        let score = normalized_levenshtein(&raw.to_lowercase(), &candidate.to_lowercase());
        assert!(score > 0.7, "score was {}", score);
    }
}

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
    // Update music_files guessed_genre where matches alias (case-insensitive)
    let result = sqlx::query("UPDATE music_files SET guessed_genre = $1, updated_at = NOW() WHERE lower(guessed_genre) = lower($2)")
        .bind(canonical)
        .bind(alias)
        .execute(&db.pool)
        .await?;

    // Update artist_genres (cache) as well
    let _ = sqlx::query(
        "UPDATE artist_genres SET genre = $1, last_updated = NOW() WHERE lower(genre) = lower($2)",
    )
    .bind(canonical)
    .bind(alias)
    .execute(&db.pool)
    .await?;

    Ok(result.rows_affected())
}

/// Preview how many rows would be affected by backfilling an alias to a canonical genre
pub async fn preview_backfill(db: &Database, alias: &str) -> Result<(i64, i64), sqlx::Error> {
    // Count music_files rows where guessed_genre matches alias (case-insensitive)
    let music_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM music_files WHERE guessed_genre IS NOT NULL AND lower(guessed_genre) = lower($1)"
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
