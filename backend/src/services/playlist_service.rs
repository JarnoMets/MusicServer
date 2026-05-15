use crate::db::Database;
use crate::models::{Playlist, PlaylistSummary, PlaylistTrackRequest, PlaylistWithItems, UpdatePlaylistRequest};
use sqlx::QueryBuilder;
use uuid::Uuid;
use crate::services::audit_service;
use crate::models::audit::CreateAuditLogRequest;

#[allow(dead_code)]
pub struct PlaylistService;

/// Get all playlists with track counts
pub async fn get_all_playlists(db: &Database) -> Result<Vec<PlaylistSummary>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, String, Option<String>, i64, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        r#"
        SELECT 
            p.id, 
            p.name, 
            p.description, 
            COALESCE(COUNT(pi.id), 0) as track_count,
            p.created_at, 
            p.updated_at 
        FROM playlists p
        LEFT JOIN playlist_items pi ON pi.playlist_id = p.id
        GROUP BY p.id, p.name, p.description, p.created_at, p.updated_at
        ORDER BY p.created_at DESC
        "#
    )
    .fetch_all(&db.pool)
    .await
    .map(|rows| {
        rows.into_iter().map(|(id, name, description, track_count, created_at, updated_at)| {
            PlaylistSummary {
                id,
                name,
                description,
                track_count,
                created_at,
                updated_at,
            }
        }).collect()
    })
}

pub async fn create_playlist(
    db: &Database,
    name: &str,
    description: Option<String>,
) -> Result<Playlist, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO playlists (id, name, description, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(id)
    .bind(name)
    .bind(&description)
    .bind(now)
    .bind(now)
    .execute(&db.pool)
    .await?;

    Ok(Playlist {
        id,
        name: name.to_string(),
        description,
        created_at: now,
        updated_at: now,
    })
}

pub async fn get_playlist(db: &Database, id: Uuid) -> Result<Option<Playlist>, sqlx::Error> {
    sqlx::query_as::<_, Playlist>(
        "SELECT id, name, description, created_at, updated_at FROM playlists WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&db.pool)
    .await
}

pub async fn delete_playlist(db: &Database, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM playlists WHERE id = $1")
        .bind(id)
        .execute(&db.pool)
        .await?;
    Ok(())
}

pub async fn update_playlist(
    db: &Database,
    id: Uuid,
    payload: UpdatePlaylistRequest,
) -> Result<Option<Playlist>, sqlx::Error> {
    // Get old playlist for audit log
    let old_playlist = get_playlist(db, id).await?;
    if old_playlist.is_none() {
        return Ok(None);
    }
    let old_playlist = old_playlist.unwrap();
    let old_values = serde_json::to_value(&old_playlist).unwrap_or(serde_json::Value::Null);

    let mut builder = QueryBuilder::new("UPDATE playlists SET ");
    let mut separated = builder.separated(", ");

    if let Some(name) = payload.name {
        separated.push("name = ");
        separated.push_bind(name);
    }
    if let Some(description) = payload.description {
        separated.push("description = ");
        separated.push_bind(description);
    }

    separated.push("updated_at = NOW()");
    builder.push(" WHERE id = ");
    builder.push_bind(id);
    builder.push(" RETURNING id, name, description, created_at, updated_at");

    let updated_playlist = builder
        .build_query_as::<Playlist>()
        .fetch_optional(&db.pool)
        .await?;

    if let Some(new_playlist) = &updated_playlist {
        let new_values = serde_json::to_value(new_playlist).unwrap_or(serde_json::Value::Null);

        // Record audit log
        let _ = audit_service::create_audit_log(db, CreateAuditLogRequest {
            table_name: "playlists".to_string(),
            record_id: id,
            action: "UPDATE".to_string(),
            old_values: Some(old_values),
            new_values: Some(new_values),
            user_id: None,
        }).await;
    }

    Ok(updated_playlist)
}

pub async fn get_playlist_with_items(
    db: &Database,
    id: Uuid,
) -> Result<Option<PlaylistWithItems>, sqlx::Error> {
    if let Some(playlist) = get_playlist(db, id).await? {
        let items = sqlx::query_as::<_, crate::models::MusicFile>(
            "SELECT mf.id, mf.title, mf.artist, mf.album, mf.genre_id, g.name as genre_name, mf.genre_source, mf.release_date, mf.duration, mf.file_path, mf.track_number, mf.file_hash, mf.bpm, mf.initial_key, mf.beat_grid_offset, mf.beat_map, mf.metadata_analyzed, mf.created_at, mf.updated_at
             FROM playlist_items pi
             JOIN music_files mf ON mf.id = pi.music_file_id
             LEFT JOIN genres g ON mf.genre_id = g.id
             WHERE pi.playlist_id = $1
             ORDER BY pi.position ASC",
        )
        .bind(id)
        .fetch_all(&db.pool)
        .await?;

        return Ok(Some(PlaylistWithItems {
            id: playlist.id,
            name: playlist.name,
            description: playlist.description,
            items,
            created_at: playlist.created_at,
            updated_at: playlist.updated_at,
        }));
    }

    Ok(None)
}

pub async fn add_track_to_playlist(
    db: &Database,
    playlist_id: Uuid,
    payload: PlaylistTrackRequest,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now();
    let position = if let Some(pos) = payload.position {
        pos
    } else {
        sqlx::query_scalar::<_, Option<i32>>(
            "SELECT MAX(position) FROM playlist_items WHERE playlist_id = $1",
        )
        .bind(playlist_id)
        .fetch_one(&db.pool)
        .await?
        .unwrap_or(0)
            + 1
    };

    sqlx::query(
        "INSERT INTO playlist_items (id, playlist_id, music_file_id, position, created_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(playlist_id)
    .bind(payload.music_file_id)
    .bind(position)
    .bind(now)
    .execute(&db.pool)
    .await?;

    // Update the playlist's updated_at timestamp
    sqlx::query("UPDATE playlists SET updated_at = $1 WHERE id = $2")
        .bind(now)
        .bind(playlist_id)
        .execute(&db.pool)
        .await?;

    Ok(())
}

pub async fn remove_track_from_playlist(
    db: &Database,
    playlist_id: Uuid,
    music_file_id: Uuid,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now();
    sqlx::query("DELETE FROM playlist_items WHERE playlist_id = $1 AND music_file_id = $2")
        .bind(playlist_id)
        .bind(music_file_id)
        .execute(&db.pool)
        .await?;
    
    // Update the playlist's updated_at timestamp
    sqlx::query("UPDATE playlists SET updated_at = $1 WHERE id = $2")
        .bind(now)
        .bind(playlist_id)
        .execute(&db.pool)
        .await?;

    Ok(())
}

    pub async fn reorder_tracks(
    db: &Database,
    playlist_id: Uuid,
    music_file_ids: Vec<Uuid>,
) -> Result<(), sqlx::Error> {
    let mut tx = db.pool.begin().await?;

        // 1. Temporarily null out positions to avoid unique constraint violations
        sqlx::query("UPDATE playlist_items SET position = -1 * (position + 1000) WHERE playlist_id = $1")
            .bind(playlist_id)
            .execute(&mut *tx)
            .await?;

        // 2. Clear out any items that are NOT in our new list (optional, but good for sync)
        sqlx::query("DELETE FROM playlist_items WHERE playlist_id = $1 AND music_file_id NOT IN (SELECT * FROM UNNEST($2::uuid[]))")
            .bind(playlist_id)
            .bind(&music_file_ids)
            .execute(&mut *tx)
            .await?;

        // 3. Update each item with its new position
        for (index, music_file_id) in music_file_ids.iter().enumerate() {
            sqlx::query(
                "UPDATE playlist_items 
                 SET position = $1 
                 WHERE playlist_id = $2 AND music_file_id = $3"
            )
            .bind(index as i32)
            .bind(playlist_id)
            .bind(music_file_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
