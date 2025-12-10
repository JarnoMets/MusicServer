use crate::db::Database;
use crate::models::{CreateYoutubePlaylistRequest, UpdateYoutubePlaylistRequest, YoutubePlaylist};
use uuid::Uuid;

/// List all saved YouTube playlists
pub async fn list_playlists(db: &Database) -> Result<Vec<YoutubePlaylist>, sqlx::Error> {
    sqlx::query_as::<_, YoutubePlaylist>(
        r#"
        SELECT id, name, url, description, auto_download, last_synced_at, created_at, updated_at 
        FROM youtube_playlists 
        ORDER BY name ASC
        "#
    )
    .fetch_all(&db.pool)
    .await
}

/// Get a single YouTube playlist by ID
pub async fn get_playlist(db: &Database, id: Uuid) -> Result<Option<YoutubePlaylist>, sqlx::Error> {
    sqlx::query_as::<_, YoutubePlaylist>(
        r#"
        SELECT id, name, url, description, auto_download, last_synced_at, created_at, updated_at 
        FROM youtube_playlists 
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&db.pool)
    .await
}

/// Create a new YouTube playlist entry
pub async fn create_playlist(
    db: &Database,
    req: CreateYoutubePlaylistRequest,
) -> Result<YoutubePlaylist, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let auto_download = req.auto_download.unwrap_or(false);

    sqlx::query(
        r#"
        INSERT INTO youtube_playlists (id, name, url, description, auto_download, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#
    )
    .bind(id)
    .bind(&req.name)
    .bind(&req.url)
    .bind(&req.description)
    .bind(auto_download)
    .bind(now)
    .bind(now)
    .execute(&db.pool)
    .await?;

    Ok(YoutubePlaylist {
        id,
        name: req.name,
        url: req.url,
        description: req.description,
        auto_download,
        last_synced_at: None,
        created_at: now,
        updated_at: now,
    })
}

/// Update a YouTube playlist
pub async fn update_playlist(
    db: &Database,
    id: Uuid,
    req: UpdateYoutubePlaylistRequest,
) -> Result<Option<YoutubePlaylist>, sqlx::Error> {
    let _now = chrono::Utc::now();
    
    // Build dynamic update query
    let mut updates = Vec::new();
    let mut bind_count = 1;
    
    if req.name.is_some() {
        bind_count += 1;
        updates.push(format!("name = ${}", bind_count));
    }
    if req.url.is_some() {
        bind_count += 1;
        updates.push(format!("url = ${}", bind_count));
    }
    if req.description.is_some() {
        bind_count += 1;
        updates.push(format!("description = ${}", bind_count));
    }
    if req.auto_download.is_some() {
        bind_count += 1;
        updates.push(format!("auto_download = ${}", bind_count));
    }
    
    if updates.is_empty() {
        return get_playlist(db, id).await;
    }
    
    updates.push("updated_at = NOW()".to_string());
    
    let query = format!(
        "UPDATE youtube_playlists SET {} WHERE id = $1 RETURNING id, name, url, description, auto_download, last_synced_at, created_at, updated_at",
        updates.join(", ")
    );
    
    let mut query_builder = sqlx::query_as::<_, YoutubePlaylist>(&query).bind(id);
    
    if let Some(ref name) = req.name {
        query_builder = query_builder.bind(name);
    }
    if let Some(ref url) = req.url {
        query_builder = query_builder.bind(url);
    }
    if let Some(ref description) = req.description {
        query_builder = query_builder.bind(description);
    }
    if let Some(auto_download) = req.auto_download {
        query_builder = query_builder.bind(auto_download);
    }
    
    query_builder.fetch_optional(&db.pool).await
}

/// Delete a YouTube playlist
pub async fn delete_playlist(db: &Database, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM youtube_playlists WHERE id = $1")
        .bind(id)
        .execute(&db.pool)
        .await?;
    
    Ok(result.rows_affected() > 0)
}

/// Update the last synced timestamp for a playlist
pub async fn mark_synced(db: &Database, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE youtube_playlists SET last_synced_at = NOW(), updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&db.pool)
        .await?;
    
    Ok(())
}

/// Get all playlists with auto_download enabled
pub async fn get_auto_download_playlists(db: &Database) -> Result<Vec<YoutubePlaylist>, sqlx::Error> {
    sqlx::query_as::<_, YoutubePlaylist>(
        r#"
        SELECT id, name, url, description, auto_download, last_synced_at, created_at, updated_at 
        FROM youtube_playlists 
        WHERE auto_download = true
        ORDER BY name ASC
        "#
    )
    .fetch_all(&db.pool)
    .await
}
