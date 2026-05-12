mod schema;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use url::Url;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::user::User;
use crate::models::access_token::AccessToken;

#[derive(Clone)]
pub struct Database {
    pub pool: Pool<Postgres>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        // Parse the database URL to extract components
        let url = Url::parse(database_url)
            .map_err(|_| sqlx::Error::Configuration("Invalid DATABASE_URL".into()))?;

        let db_name = url.path().trim_start_matches('/');
        if db_name.is_empty() {
            return Err(sqlx::Error::Configuration(
                "DATABASE_URL must include a database name".into(),
            ));
        }

        // Build admin URL (connect to postgres database instead)
        let admin_url = format!(
            "postgres://{}{}@{}:{}/postgres",
            url.username(),
            if let Some(password) = url.password() {
                format!(":{}", password)
            } else {
                String::new()
            },
            url.host_str().unwrap_or("localhost"),
            url.port().unwrap_or(5432)
        );

        // Connect as admin and create database if it doesn't exist
        let admin_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&admin_url)
            .await?;

        // Create database if it doesn't exist
        let create_db_query = format!(
            "CREATE DATABASE \"{}\" ENCODING = 'UTF8'",
            db_name
        );
        
        // Try to create the database, ignore error if it already exists
        match sqlx::raw_sql(&create_db_query)
            .execute(&admin_pool)
            .await
        {
            Ok(_) => {
                log::info!("Database '{}' created successfully", db_name);
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("already exists") {
                    log::info!("Database '{}' already exists", db_name);
                } else {
                    log::warn!("Could not create database: {}", e);
                }
            }
        }

        drop(admin_pool);

        // Now connect to the actual database with optimized pool settings
        let pool = PgPoolOptions::new()
            .min_connections(2)           // Keep a couple connections ready
            .max_connections(20)          // Increased from 10 to prevent starvation during heavy IO/sync
            .idle_timeout(std::time::Duration::from_secs(300))  // Close idle connections after 5 min
            .max_lifetime(std::time::Duration::from_secs(1800)) // Recycle connections after 30 min
            .acquire_timeout(std::time::Duration::from_secs(30)) // Fail fast on connection issues
            .connect(database_url)
            .await?;

        // Test the connection
        sqlx::query("SELECT 1").fetch_one(&pool).await?;

        // Run programmatic migrations (can be disabled by setting RUN_MIGRATIONS=false)
        let run_migrations = std::env::var("RUN_MIGRATIONS")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase();
        if run_migrations != "false" && run_migrations != "0" {
            schema::run_migrations(&pool).await?;
        } else {
            log::info!("Skipping programmatic migrations because RUN_MIGRATIONS is set to false");
        }
        
        // Fix any legacy timestamp columns
        schema::fix_timestamp_columns(&pool).await?;

        log::info!("Database setup completed successfully");

        Ok(Database { pool })
    }

    // --- User Related Methods ---

    pub async fn get_user_by_id(&self, id: &Uuid) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, name, google_id, avatar_url, is_admin, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }

    #[allow(dead_code)]
    pub async fn get_user_by_email(&self, email: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, name, google_id, avatar_url, is_admin, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn upsert_google_user(
        &self,
        email: &str,
        google_id: &str,
        name: &str,
        avatar_url: Option<&str>,
    ) -> Result<User, sqlx::Error> {
        // First try to find by google_id
        let existing_user: Option<User> = sqlx::query_as::<_, User>(
            "SELECT id, email, name, google_id, avatar_url, is_admin, created_at, updated_at FROM users WHERE google_id = $1"
        )
        .bind(google_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(user) = existing_user {
            // Update existing user's info
            return sqlx::query_as::<_, User>(
                "UPDATE users SET email = $1, name = $2, avatar_url = $3, updated_at = NOW() WHERE id = $4 RETURNING id, email, name, google_id, avatar_url, is_admin, created_at, updated_at"
            )
            .bind(email)
            .bind(name)
            .bind(avatar_url)
            .bind(user.id)
            .fetch_one(&self.pool)
            .await;
        }

        // Then try to find by email
        let existing_email_user: Option<User> = sqlx::query_as::<_, User>(
            "SELECT id, email, name, google_id, avatar_url, is_admin, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(user) = existing_email_user {
            // Update existing user with google_id
            return sqlx::query_as::<_, User>(
                "UPDATE users SET google_id = $1, name = $2, avatar_url = $3, updated_at = NOW() WHERE id = $4 RETURNING id, email, name, google_id, avatar_url, is_admin, created_at, updated_at"
            )
            .bind(google_id)
            .bind(name)
            .bind(avatar_url)
            .bind(user.id)
            .fetch_one(&self.pool)
            .await;
        }

        // Create new user (first user is admin)
        let count: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;
        
        let is_admin = count == 0;

        sqlx::query_as::<_, User>(
            "INSERT INTO users (email, google_id, name, avatar_url, is_admin) VALUES ($1, $2, $3, $4, $5) RETURNING id, email, name, google_id, avatar_url, is_admin, created_at, updated_at"
        )
        .bind(email)
        .bind(google_id)
        .bind(name)
        .bind(avatar_url)
        .bind(is_admin)
        .fetch_one(&self.pool)
        .await
    }

    // --- Access Token Methods ---

    /// Generate a cryptographically secure access token.
    /// Returns `(raw_token, sha256_hex_hash)`. Only store the hash.
    pub fn generate_access_token() -> (String, String) {
        use sha2::{Digest, Sha256};
        // Two UUID v4s each contribute 122 bits of CSPRNG entropy → 244 bits total
        let a = Uuid::new_v4().to_string().replace('-', "");
        let b = Uuid::new_v4().to_string().replace('-', "");
        let token = format!("ms_{}{}", a, b); // ms_ + 64 hex chars = 67 chars
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let hash = hex::encode(hasher.finalize());
        (token, hash)
    }

    /// Hash an existing raw token (for middleware lookup).
    pub fn hash_access_token(raw: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(raw.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub async fn create_access_token(
        &self,
        user_id: &Uuid,
        name: &str,
        token_hash: &str,
        can_read: bool,
        can_create: bool,
        can_edit: bool,
        can_delete: bool,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<AccessToken, sqlx::Error> {
        sqlx::query_as::<_, AccessToken>(
            r#"
            INSERT INTO access_tokens
                (user_id, name, token_hash, can_read, can_create, can_edit, can_delete, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, user_id, name, token_hash, can_read, can_create, can_edit, can_delete,
                      last_used_at, expires_at, created_at
            "#,
        )
        .bind(user_id)
        .bind(name)
        .bind(token_hash)
        .bind(can_read)
        .bind(can_create)
        .bind(can_edit)
        .bind(can_delete)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_access_tokens(&self, user_id: &Uuid) -> Result<Vec<AccessToken>, sqlx::Error> {
        sqlx::query_as::<_, AccessToken>(
            r#"
            SELECT id, user_id, name, token_hash, can_read, can_create, can_edit, can_delete,
                   last_used_at, expires_at, created_at
            FROM access_tokens
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_access_token_by_id(
        &self,
        id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Option<AccessToken>, sqlx::Error> {
        sqlx::query_as::<_, AccessToken>(
            r#"
            SELECT id, user_id, name, token_hash, can_read, can_create, can_edit, can_delete,
                   last_used_at, expires_at, created_at
            FROM access_tokens
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Look up an access token by its SHA-256 hash (used by auth middleware).
    pub async fn get_access_token_by_hash(
        &self,
        hash: &str,
    ) -> Result<Option<AccessToken>, sqlx::Error> {
        sqlx::query_as::<_, AccessToken>(
            r#"
            SELECT id, user_id, name, token_hash, can_read, can_create, can_edit, can_delete,
                   last_used_at, expires_at, created_at
            FROM access_tokens
            WHERE token_hash = $1
            "#,
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update_access_token(
        &self,
        id: &Uuid,
        user_id: &Uuid,
        name: Option<&str>,
        can_read: Option<bool>,
        can_create: Option<bool>,
        can_edit: Option<bool>,
        can_delete: Option<bool>,
        new_expires_at: Option<DateTime<Utc>>,
        clear_expires_at: bool,
    ) -> Result<Option<AccessToken>, sqlx::Error> {
        // Build dynamic update — only change fields that were provided
        let expires_value: Option<Option<DateTime<Utc>>> = if clear_expires_at {
            Some(None) // set to NULL
        } else {
            new_expires_at.map(Some) // set to provided date
        };

        let result = sqlx::query_as::<_, AccessToken>(
            r#"
            UPDATE access_tokens SET
                name = COALESCE($3, name),
                can_read = COALESCE($4, can_read),
                can_create = COALESCE($5, can_create),
                can_edit = COALESCE($6, can_edit),
                can_delete = COALESCE($7, can_delete),
                expires_at = CASE WHEN $8 THEN NULL
                                  WHEN $9::TIMESTAMPTZ IS NOT NULL THEN $9::TIMESTAMPTZ
                                  ELSE expires_at
                             END
            WHERE id = $1 AND user_id = $2
            RETURNING id, user_id, name, token_hash, can_read, can_create, can_edit, can_delete,
                      last_used_at, expires_at, created_at
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(name)
        .bind(can_read)
        .bind(can_create)
        .bind(can_edit)
        .bind(can_delete)
        .bind(clear_expires_at)
        .bind(expires_value.and_then(|v| v))
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_access_token(&self, id: &Uuid, user_id: &Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM access_tokens WHERE id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
