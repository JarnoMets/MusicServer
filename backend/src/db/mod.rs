mod schema;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use url::Url;

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
            .min_connections(1)           // Keep at least 1 connection ready
            .max_connections(10)          // Allow up to 10 connections under load
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
}
