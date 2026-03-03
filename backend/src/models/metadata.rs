use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MetadataConfig {
    pub id: Uuid,
    pub metadata_source: String, // Merged from release_date_source and genre_source
    pub discogs_token: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMetadataConfigRequest {
    pub metadata_source: Option<String>,
    pub discogs_token: Option<String>,
}

impl MetadataConfig {
    pub async fn get_config(pool: &sqlx::PgPool) -> Result<Self, sqlx::Error> {
        let mut config = sqlx::query_as::<_, MetadataConfig>(
            "SELECT id, metadata_source, discogs_token, updated_at FROM metadata_config LIMIT 1"
        )
        .fetch_one(pool)
        .await?;
        
        // Helper to validate a token string (must be ASCII and not empty)
        let is_valid = |t: &str| {
            let trimmed = t.trim();
            !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii() && !c.is_control())
        };

        // If DB token is corrupted (e.g. from previous base64 decoding error), ignore it
        if let Some(ref t) = config.discogs_token {
            if !is_valid(t) {
                log::warn!("Discogs token in Database is invalid/corrupted, falling back to environment variable...");
                config.discogs_token = None;
            } else {
                config.discogs_token = Some(t.trim().to_string());
            }
        }

        // If DB token is not set or empty, fallback to environment variable or file
        if config.discogs_token.is_none() {
            if let Ok(env_token) = std::env::var("DISCOGS_TOKEN") {
                let trimmed = env_token.trim();
                if is_valid(trimmed) {
                    config.discogs_token = Some(trimmed.to_string());
                } else if !trimmed.is_empty() {
                    log::error!("DISCOGS_TOKEN environment variable contains corrupted binary data.");
                }
            }
            
            // Still empty? Try local file (helpful for dev/homelab)
            if config.discogs_token.is_none() {
                for path in &["discogs_token", "../discogs_token"] {
                    if let Ok(token) = std::fs::read_to_string(path) {
                        let trimmed = token.trim();
                        if is_valid(trimmed) {
                            config.discogs_token = Some(trimmed.to_string());
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(config)
    }

    pub async fn update_config(pool: &sqlx::PgPool, req: UpdateMetadataConfigRequest) -> Result<Self, sqlx::Error> {
        let current = Self::get_config(pool).await?;
        
        let metadata_source = req.metadata_source.unwrap_or(current.metadata_source);
        let discogs_token = req.discogs_token.or(current.discogs_token);
        
        let updated = sqlx::query_as::<_, MetadataConfig>(
            "UPDATE metadata_config SET metadata_source = $1, discogs_token = $2, updated_at = NOW() WHERE id = $3 RETURNING id, metadata_source, discogs_token, updated_at"
        )
        .bind(metadata_source)
        .bind(discogs_token)
        .bind(current.id)
        .fetch_one(pool)
        .await?;
        
        Ok(updated)
    }
}
