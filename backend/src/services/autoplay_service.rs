use crate::db::Database;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutoplayConfig {
    pub match_time_seconds: i32,
    pub overlap_seconds: i32,
    pub exit_time_seconds: i32,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn get_autoplay_config(db: &Database) -> Result<AutoplayConfig, sqlx::Error> {
    let row = sqlx::query_as::<_, (i32, i32, i32, chrono::DateTime<chrono::Utc>)>(
        "SELECT match_time_seconds, overlap_seconds, exit_time_seconds, updated_at FROM autoplay_settings LIMIT 1"
    )
    .fetch_one(&db.pool)
    .await?;

    Ok(AutoplayConfig {
        match_time_seconds: row.0,
        overlap_seconds: row.1,
        exit_time_seconds: row.2,
        updated_at: row.3,
    })
}

pub async fn update_autoplay_config(
    db: &Database,
    match_time_seconds: i32,
    overlap_seconds: i32,
    exit_time_seconds: i32,
) -> Result<AutoplayConfig, sqlx::Error> {
    // Ensure overlap is not greater than match_time
    let overlap = if overlap_seconds > match_time_seconds { match_time_seconds } else { overlap_seconds };

    // Upsert into the single-row table (use INSERT ... ON CONFLICT (id) DO UPDATE). We don't know the id; use a simple update if exists else insert
    let exists: Option<Uuid> = sqlx::query_scalar::<_, Uuid>("SELECT id FROM autoplay_settings LIMIT 1").fetch_optional(&db.pool).await?;

    let now = chrono::Utc::now();

    if let Some(id) = exists {
        sqlx::query(
            "UPDATE autoplay_settings SET match_time_seconds = $1, overlap_seconds = $2, exit_time_seconds = $3, updated_at = $4 WHERE id = $5"
        )
        .bind(match_time_seconds)
        .bind(overlap)
        .bind(exit_time_seconds)
        .bind(now)
        .bind(id)
        .execute(&db.pool)
        .await?;
    } else {
        sqlx::query(
            "INSERT INTO autoplay_settings (id, match_time_seconds, overlap_seconds, exit_time_seconds, updated_at) VALUES (gen_random_uuid(), $1, $2, $3, $4)"
        )
        .bind(match_time_seconds)
        .bind(overlap)
        .bind(exit_time_seconds)
        .bind(now)
        .execute(&db.pool)
        .await?;
    }

    Ok(AutoplayConfig {
        match_time_seconds,
        overlap_seconds: overlap,
        exit_time_seconds,
        updated_at: now,
    })
}
