use sqlx::{Pool, Postgres};

/// Run all database schema setup and migrations programmatically.
/// This replaces file-based migrations for more flexibility.
pub async fn run_migrations(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    log::info!("Running database schema setup...");

    // Create schema version tracking table
    create_schema_version_table(pool).await?;

    // Run migrations in order - each checks if already applied
    migration_1_music_files(pool).await?;
    migration_2_playlists(pool).await?;
    migration_3_playlist_items(pool).await?;
    migration_4_youtube_downloads(pool).await?;
    migration_5_artist_genres(pool).await?;
    migration_6_genres(pool).await?;
    migration_7_genre_aliases(pool).await?;
    migration_8_internet_streams(pool).await?;
    migration_9_genre_columns(pool).await?;
    migration_10_track_number(pool).await?;
    migration_11_indexes(pool).await?;
    migration_12_youtube_playlists(pool).await?;
    migration_13_auto_download_config(pool).await?;
    migration_14_file_hash(pool).await?;

    log::info!("Database schema setup completed successfully");
    Ok(())
}

/// Create the schema version tracking table if it doesn't exist
async fn create_schema_version_table(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Check if a migration has been applied
async fn is_migration_applied(pool: &Pool<Postgres>, version: i32) -> Result<bool, sqlx::Error> {
    let result: Option<i32> = sqlx::query_scalar(
        "SELECT version FROM schema_version WHERE version = $1"
    )
    .bind(version)
    .fetch_optional(pool)
    .await?;
    Ok(result.is_some())
}

/// Record that a migration has been applied
async fn record_migration(pool: &Pool<Postgres>, version: i32, description: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO schema_version (version, description) VALUES ($1, $2)"
    )
    .bind(version)
    .bind(description)
    .execute(pool)
    .await?;
    Ok(())
}

// ============================================================================
// Migrations
// ============================================================================

async fn migration_1_music_files(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    if is_migration_applied(pool, 1).await? {
        return Ok(());
    }
    log::info!("Applying migration 1: Create music_files table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS music_files (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title VARCHAR(255) NOT NULL,
            artist VARCHAR(255),
            album VARCHAR(255),
            duration INTEGER,
            file_path VARCHAR(500) NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    record_migration(pool, 1, "Create music_files table").await
}

async fn migration_2_playlists(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    if is_migration_applied(pool, 2).await? {
        return Ok(());
    }
    log::info!("Applying migration 2: Create playlists table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS playlists (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(255) NOT NULL,
            description TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    record_migration(pool, 2, "Create playlists table").await
}

async fn migration_3_playlist_items(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    if is_migration_applied(pool, 3).await? {
        return Ok(());
    }
    log::info!("Applying migration 3: Create playlist_items table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS playlist_items (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            playlist_id UUID NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
            music_file_id UUID NOT NULL REFERENCES music_files(id) ON DELETE CASCADE,
            position INTEGER NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(playlist_id, position)
        )
        "#,
    )
    .execute(pool)
    .await?;

    record_migration(pool, 3, "Create playlist_items table").await
}

async fn migration_4_youtube_downloads(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    if is_migration_applied(pool, 4).await? {
        return Ok(());
    }
    log::info!("Applying migration 4: Create youtube_downloads table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS youtube_downloads (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            video_id VARCHAR(20) NOT NULL UNIQUE,
            video_url TEXT NOT NULL,
            title VARCHAR(500),
            uploader VARCHAR(255),
            file_path TEXT,
            downloaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    record_migration(pool, 4, "Create youtube_downloads table").await
}

async fn migration_5_artist_genres(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    if is_migration_applied(pool, 5).await? {
        return Ok(());
    }
    log::info!("Applying migration 5: Create artist_genres table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS artist_genres (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            artist_name VARCHAR(255) NOT NULL UNIQUE,
            genre VARCHAR(255) NOT NULL,
            last_updated TIMESTAMPTZ DEFAULT NOW(),
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    record_migration(pool, 5, "Create artist_genres table").await
}

async fn migration_6_genres(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    if is_migration_applied(pool, 6).await? {
        return Ok(());
    }
    log::info!("Applying migration 6: Create genres table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS genres (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    record_migration(pool, 6, "Create genres table").await
}

async fn migration_7_genre_aliases(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    if is_migration_applied(pool, 7).await? {
        return Ok(());
    }
    log::info!("Applying migration 7: Create genre_aliases table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS genre_aliases (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            alias TEXT NOT NULL UNIQUE,
            genre_id UUID NOT NULL REFERENCES genres(id) ON DELETE CASCADE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    record_migration(pool, 7, "Create genre_aliases table").await
}

async fn migration_8_internet_streams(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    if is_migration_applied(pool, 8).await? {
        return Ok(());
    }
    log::info!("Applying migration 8: Create internet_streams table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS internet_streams (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            genre TEXT,
            description TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    record_migration(pool, 8, "Create internet_streams table").await
}

async fn migration_9_genre_columns(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    if is_migration_applied(pool, 9).await? {
        return Ok(());
    }
    log::info!("Applying migration 9: Add genre columns to music_files");

    // Add columns if they don't exist using DO block
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'music_files' AND column_name = 'genre') THEN
                ALTER TABLE music_files ADD COLUMN genre VARCHAR(255);
            END IF;
            IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'music_files' AND column_name = 'guessed_genre') THEN
                ALTER TABLE music_files ADD COLUMN guessed_genre VARCHAR(255);
            END IF;
            IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'music_files' AND column_name = 'release_date') THEN
                ALTER TABLE music_files ADD COLUMN release_date TIMESTAMPTZ;
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    record_migration(pool, 9, "Add genre columns to music_files").await
}

async fn migration_10_track_number(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    if is_migration_applied(pool, 10).await? {
        return Ok(());
    }
    log::info!("Applying migration 10: Add track_number to music_files");

    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'music_files' AND column_name = 'track_number') THEN
                ALTER TABLE music_files ADD COLUMN track_number INTEGER;
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    record_migration(pool, 10, "Add track_number to music_files").await
}

async fn migration_11_indexes(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    if is_migration_applied(pool, 11).await? {
        return Ok(());
    }
    log::info!("Applying migration 11: Create indexes");

    // Create indexes - these are all idempotent with IF NOT EXISTS
    let indexes = [
        "CREATE INDEX IF NOT EXISTS idx_playlists_name ON playlists(name)",
        "CREATE INDEX IF NOT EXISTS idx_music_files_artist ON music_files(artist)",
        "CREATE INDEX IF NOT EXISTS idx_music_files_album ON music_files(album)",
        "CREATE INDEX IF NOT EXISTS idx_music_files_genre ON music_files(genre)",
        "CREATE INDEX IF NOT EXISTS idx_music_files_guessed_genre ON music_files(guessed_genre)",
        "CREATE INDEX IF NOT EXISTS idx_youtube_downloads_video_id ON youtube_downloads(video_id)",
        "CREATE INDEX IF NOT EXISTS idx_youtube_downloads_video_url ON youtube_downloads(video_url)",
        "CREATE INDEX IF NOT EXISTS idx_artist_genres_artist_name ON artist_genres(artist_name)",
        "CREATE INDEX IF NOT EXISTS idx_genre_alias_alias ON genre_aliases(alias)",
        "CREATE INDEX IF NOT EXISTS idx_internet_streams_genre ON internet_streams(genre)",
    ];

    for index_sql in indexes {
        sqlx::query(index_sql).execute(pool).await?;
    }

    record_migration(pool, 11, "Create indexes").await
}

async fn migration_12_youtube_playlists(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    if is_migration_applied(pool, 12).await? {
        return Ok(());
    }
    log::info!("Applying migration 12: Create youtube_playlists table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS youtube_playlists (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name TEXT NOT NULL,
            url TEXT NOT NULL UNIQUE,
            description TEXT,
            auto_download BOOLEAN NOT NULL DEFAULT false,
            last_synced_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    record_migration(pool, 12, "Create youtube_playlists table").await
}

async fn migration_13_auto_download_config(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    if is_migration_applied(pool, 13).await? {
        return Ok(());
    }
    log::info!("Applying migration 13: Create auto_download_config table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS auto_download_config (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            enabled BOOLEAN NOT NULL DEFAULT false,
            -- Schedule: cron-like fields
            check_interval_minutes INTEGER NOT NULL DEFAULT 60,
            -- Throttling
            max_concurrent_downloads INTEGER NOT NULL DEFAULT 2,
            delay_between_downloads_seconds INTEGER NOT NULL DEFAULT 5,
            -- Time window restrictions (optional)
            allowed_start_hour INTEGER,  -- 0-23, NULL means no restriction
            allowed_end_hour INTEGER,    -- 0-23, NULL means no restriction
            -- Last run info
            last_check_at TIMESTAMPTZ,
            next_check_at TIMESTAMPTZ,
            -- Timestamps
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Insert default config row
    sqlx::query(
        r#"
        INSERT INTO auto_download_config (id, enabled, check_interval_minutes, max_concurrent_downloads, delay_between_downloads_seconds)
        VALUES (gen_random_uuid(), false, 60, 2, 5)
        ON CONFLICT DO NOTHING
        "#,
    )
    .execute(pool)
    .await?;

    record_migration(pool, 13, "Create auto_download_config table").await
}

async fn migration_14_file_hash(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    if is_migration_applied(pool, 14).await? {
        return Ok(());
    }
    log::info!("Applying migration 14: Add file_hash column to music_files for duplicate detection");

    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'music_files' AND column_name = 'file_hash') THEN
                ALTER TABLE music_files ADD COLUMN file_hash VARCHAR(64);
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    // Create index on file_hash for fast duplicate lookups
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_music_files_file_hash ON music_files(file_hash)")
        .execute(pool)
        .await?;

    record_migration(pool, 14, "Add file_hash column to music_files").await
}

/// Fix timestamp columns for databases that were created with TIMESTAMP instead of TIMESTAMPTZ
/// This is safe to run multiple times - it only converts if needed
pub async fn fix_timestamp_columns(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    // This migration handles legacy databases that may have been created with TIMESTAMP
    // It converts them to TIMESTAMPTZ which works correctly with DateTime<Utc>
    
    let tables_and_columns = [
        ("music_files", vec!["created_at", "updated_at"]),
        ("playlists", vec!["created_at", "updated_at"]),
        ("playlist_items", vec!["created_at"]),
        ("artist_genres", vec!["last_updated", "created_at"]),
    ];

    for (table, columns) in tables_and_columns {
        for column in columns {
            // Check if column exists and is not already timestamptz
            let result: Option<String> = sqlx::query_scalar(
                r#"
                SELECT data_type 
                FROM information_schema.columns 
                WHERE table_name = $1 AND column_name = $2
                "#
            )
            .bind(table)
            .bind(column)
            .fetch_optional(pool)
            .await?;

            if let Some(data_type) = result {
                if data_type == "timestamp without time zone" {
                    log::info!("Converting {}.{} from TIMESTAMP to TIMESTAMPTZ", table, column);
                    let sql = format!(
                        "ALTER TABLE {} ALTER COLUMN {} TYPE TIMESTAMPTZ USING {} AT TIME ZONE 'UTC'",
                        table, column, column
                    );
                    sqlx::query(&sql).execute(pool).await?;
                }
            }
        }
    }

    Ok(())
}
