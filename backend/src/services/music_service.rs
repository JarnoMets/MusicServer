use crate::db::Database;
use crate::models::{
    BulkAddToPlaylistByRegexRequest, BulkAddToPlaylistResponse, BulkRenameByRegexRequest, 
    BulkRenameResponse, CreateMusicFileRequest, MusicFile, MusicQueryParams, 
    UpdateMusicFileRequest, CreateAuditLogRequest
};
use crate::services::music_query_helpers::{select_music_files};
use sqlx::QueryBuilder;
use uuid::Uuid;
use crate::services::audit_service;

pub async fn get_all_music_files(
    db: &Database,
    params: MusicQueryParams,
) -> Result<Vec<MusicFile>, sqlx::Error> {
    perform_music_query(db, params).await
}

/// The actual SQL query logic
async fn perform_music_query(
    db: &Database,
    params: MusicQueryParams,
) -> Result<Vec<MusicFile>, sqlx::Error> {
    let mut builder = QueryBuilder::new(format!(
        "{} WHERE 1=1",
        select_music_files()
    ));

    crate::services::music_query_helpers::apply_music_filters(&mut builder, &params);

    let sort_column = match params.sort.as_deref() {
        Some("artist") => "mf.artist",
        Some("album") => "mf.album",
        Some("genre") => "g.name",
        Some("duration") => "mf.duration",
        Some("created_at") => "mf.created_at",
        Some("updated_at") => "mf.updated_at",
        Some("release_date") => "mf.release_date",
        _ => "mf.title",
    };
    let order = match params.order.as_deref() {
        Some("desc") | Some("DESC") => "DESC",
        _ => "ASC",
    };

    builder
        .push(" ORDER BY ")
        .push(sort_column)
        .push(" ")
        .push(order);

    if let Some(limit) = params.limit {
        builder.push(" LIMIT ").push_bind(limit);
    }
    if let Some(offset) = params.offset {
        builder.push(" OFFSET ").push_bind(offset);
    }

    builder
        .build_query_as::<MusicFile>()
        .fetch_all(&db.pool)
        .await
}

pub async fn create_music_file(
    db: &Database,
    req: CreateMusicFileRequest,
) -> Result<MusicFile, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO music_files (id, title, artist, album, genre_id, genre_source, release_date, duration, file_path, track_number, file_hash, bpm, initial_key, beat_grid_offset, beat_map, metadata_analyzed, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)"
    )
    .bind(id)
    .bind(&req.title)
    .bind(&req.artist)
    .bind(&req.album)
    .bind(req.genre_id)
    .bind(&req.genre_source)
    .bind(req.release_date)
    .bind(req.duration)
    .bind(&req.file_path)
    .bind(req.track_number)
    .bind(&req.file_hash)
    .bind(req.bpm)
    .bind(&req.initial_key)
    .bind(req.beat_grid_offset)
    .bind(&req.beat_map)
    .bind(req.metadata_analyzed.unwrap_or(false))
    .bind(now)
    .bind(now)
    .execute(&db.pool)
    .await?;

    Ok(MusicFile {
        id,
        title: req.title,
        artist: req.artist,
        album: req.album,
        genre_id: req.genre_id,
        genre_name: None, // would need a JOIN to resolve; callers can fetch via get_music_file
        genre_source: req.genre_source,
        release_date: req.release_date,
        duration: req.duration,
        file_path: req.file_path,
        track_number: req.track_number,
        file_hash: req.file_hash,
        bpm: req.bpm,
        initial_key: req.initial_key,
        beat_grid_offset: req.beat_grid_offset,
        beat_map: req.beat_map,
        metadata_analyzed: req.metadata_analyzed.unwrap_or(false),
        created_at: now,
        updated_at: now,
    })
}

pub async fn update_music_file(
    db: &Database,
    id: Uuid,
    req: UpdateMusicFileRequest,
) -> Result<Option<MusicFile>, sqlx::Error> {
    // Get current record first for audit log
    let old_file = get_music_file(db, id).await?;
    if old_file.is_none() {
        return Ok(None);
    }
    let old_file = old_file.unwrap();
    let old_values = serde_json::to_value(&old_file).unwrap_or(serde_json::Value::Null);

    let mut builder = QueryBuilder::new("UPDATE music_files SET ");
    let mut has_set = false;

    if let Some(title) = req.title {
        if has_set {
            builder.push(", ");
        }
        builder.push("title = ").push_bind(title);
        has_set = true;
    }
    if let Some(artist) = req.artist {
        if has_set {
            builder.push(", ");
        }
        builder.push("artist = ").push_bind(artist);
        has_set = true;
    }
    if let Some(album) = req.album {
        if has_set {
            builder.push(", ");
        }
        builder.push("album = ").push_bind(album);
        has_set = true;
    }
    if let Some(genre_id) = req.genre_id {
        if has_set {
            builder.push(", ");
        }
        builder.push("genre_id = ").push_bind(genre_id);
        has_set = true;
    }
    if let Some(genre_source) = req.genre_source {
        if has_set {
            builder.push(", ");
        }
        builder.push("genre_source = ").push_bind(genre_source);
        has_set = true;
    }
    if let Some(release_date) = req.release_date {
        if has_set {
            builder.push(", ");
        }
        builder.push("release_date = ").push_bind(release_date);
        has_set = true;
    }
    if let Some(duration) = req.duration {
        if has_set {
            builder.push(", ");
        }
        builder.push("duration = ").push_bind(duration);
        has_set = true;
    }
    if let Some(track_number) = req.track_number {
        if has_set {
            builder.push(", ");
        }
        builder.push("track_number = ").push_bind(track_number);
        has_set = true;
    }
    if let Some(bpm) = req.bpm {
        if has_set {
            builder.push(", ");
        }
        builder.push("bpm = ").push_bind(bpm);
        has_set = true;
    }
    if let Some(initial_key) = req.initial_key {
        if has_set {
            builder.push(", ");
        }
        builder.push("initial_key = ").push_bind(initial_key);
        has_set = true;
    }
    if let Some(beat_grid_offset) = req.beat_grid_offset {
        if has_set {
            builder.push(", ");
        }
        builder.push("beat_grid_offset = ").push_bind(beat_grid_offset);
        has_set = true;
    }
    if let Some(beat_map) = req.beat_map {
        if has_set {
            builder.push(", ");
        }
        builder.push("beat_map = ").push_bind(beat_map);
        has_set = true;
    }
    if let Some(metadata_analyzed) = req.metadata_analyzed {
        if has_set {
            builder.push(", ");
        }
        builder.push("metadata_analyzed = ").push_bind(metadata_analyzed);
        has_set = true;
    }

    if !has_set {
        return Ok(Some(old_file));
    }

    builder.push(", updated_at = NOW() WHERE id = ").push_bind(id);
    builder.build().execute(&db.pool).await?;

    let updated_file = get_music_file(db, id).await?;

    if let Some(new_file) = &updated_file {
        let new_values = serde_json::to_value(new_file).unwrap_or(serde_json::Value::Null);

        // Record audit log
        let _ = audit_service::create_audit_log(db, CreateAuditLogRequest {
            table_name: "music_files".to_string(),
            record_id: id,
            action: "UPDATE".to_string(),
            old_values: Some(old_values),
            new_values: Some(new_values),
            user_id: None, // Will fill with user from SSO once implemented
        }).await;
    }

    Ok(updated_file)
}

pub async fn get_music_file(db: &Database, id: Uuid) -> Result<Option<MusicFile>, sqlx::Error> {
    let sql = format!("{} WHERE mf.id = $1", select_music_files());
    sqlx::query_as::<_, MusicFile>(&sql)
        .bind(id)
        .fetch_optional(&db.pool)
        .await
}

pub async fn delete_music_file(db: &Database, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM music_files WHERE id = $1")
        .bind(id)
        .execute(&db.pool)
        .await?;
    Ok(())
}

/// Check if a file with the given hash already exists in the database
pub async fn is_duplicate_hash(db: &Database, file_hash: &str) -> Result<bool, sqlx::Error> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM music_files WHERE file_hash = $1)"
    )
    .bind(file_hash)
    .fetch_one(&db.pool)
    .await?;
    
    Ok(exists)
}

/// Check if a file with the same path already exists
#[allow(dead_code)]
pub async fn is_duplicate_path(db: &Database, file_path: &str) -> Result<bool, sqlx::Error> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM music_files WHERE file_path = $1)"
    )
    .bind(file_path)
    .fetch_one(&db.pool)
    .await?;
    
    Ok(exists)
}

/// Get an existing music file by its hash
#[allow(dead_code)]
pub async fn get_by_hash(db: &Database, file_hash: &str) -> Result<Option<MusicFile>, sqlx::Error> {
    let sql = format!("{} WHERE mf.file_hash = $1", select_music_files());
    sqlx::query_as::<_, MusicFile>(&sql)
        .bind(file_hash)
        .fetch_optional(&db.pool)
        .await
}

/// Compute SHA-256 hash of a file using blocking IO on a dedicated threadpool
/// to prevent blocking the async runtime during intensive hashing/IO.
pub async fn compute_file_hash(file_path: &str) -> Result<String, std::io::Error> {
    use sha2::{Digest, Sha256};
    use std::fs::File;
    use std::io::{Read, self};
    
    let path = file_path.to_string();
    
    tokio::task::spawn_blocking(move || {
        let mut file = File::open(&path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536]; // Larger 64KB buffer for throughput
        
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        
        let result = hasher.finalize();
        Ok(hex::encode(result))
    })
    .await
    .map_err(io::Error::other)?
}

/// Stats response for filtered music
#[derive(Debug, serde::Serialize)]
pub struct MusicStats {
    pub total_count: i64,
    pub total_duration_ms: i64,
}

/// Get total count and duration for filtered music (ignoring pagination)
pub async fn get_music_stats(
    db: &Database,
    params: MusicQueryParams,
) -> Result<MusicStats, sqlx::Error> {
    let mut builder = QueryBuilder::new(
        "SELECT COUNT(*) as total_count, COALESCE(SUM(mf.duration), 0) as total_duration_ms \
         FROM music_files mf LEFT JOIN genres g ON mf.genre_id = g.id WHERE 1=1",
    );

    crate::services::music_query_helpers::apply_music_filters(&mut builder, &params);

    let row: (i64, i64) = builder
        .build_query_as::<(i64, i64)>()
        .fetch_one(&db.pool)
        .await?;

    Ok(MusicStats {
        total_count: row.0,
        total_duration_ms: row.1,
    })
}

/// Get all playlists that contain a specific track
pub async fn get_track_playlists(
    db: &Database,
    track_id: Uuid,
) -> Result<Vec<crate::models::PlaylistSummary>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, String, Option<String>, i64, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        r#"
        SELECT 
            p.id, 
            p.name, 
            p.description, 
            COALESCE(COUNT(pi2.id), 0) as track_count,
            p.created_at, 
            p.updated_at
        FROM playlists p
        JOIN playlist_items pi ON p.id = pi.playlist_id
        LEFT JOIN playlist_items pi2 ON p.id = pi2.playlist_id
        WHERE pi.music_file_id = $1
        GROUP BY p.id
        ORDER BY p.name ASC
        "#
    )
    .bind(track_id)
    .fetch_all(&db.pool)
    .await
    .map(|rows| {
        rows.into_iter().map(|r| crate::models::PlaylistSummary {
            id: r.0,
            name: r.1,
            description: r.2,
            track_count: r.3,
            created_at: r.4,
            updated_at: r.5,
        }).collect()
    })
}

pub async fn bulk_update_music(
    db: &Database,
    req: crate::models::BulkUpdateMusicRequest,
) -> Result<usize, sqlx::Error> {
    if req.ids.is_empty() {
        return Ok(0);
    }

    // Get old files for audit log
    let mut old_files_builder = QueryBuilder::new(format!("{} WHERE mf.id IN (", select_music_files()));
    let mut it = req.ids.iter().peekable();
    while let Some(id) = it.next() {
        old_files_builder.push_bind(id);
        if it.peek().is_some() {
            old_files_builder.push(", ");
        }
    }
    old_files_builder.push(")");
    let old_files = old_files_builder.build_query_as::<MusicFile>().fetch_all(&db.pool).await?;

    let mut builder = QueryBuilder::new("UPDATE music_files SET ");
    let mut first = true;

    if let Some(genre_id) = &req.genre_id {
        if !first { builder.push(", "); }
        builder.push("genre_id = ").push_bind(*genre_id);
        first = false;
    }

    if let Some(artist) = &req.artist {
        if !first { builder.push(", "); }
        builder.push("artist = ").push_bind(artist);
        first = false;
    }

    if let Some(album) = &req.album {
        if !first { builder.push(", "); }
        builder.push("album = ").push_bind(album);
        first = false;
    }

    if let Some(release_date) = &req.release_date {
        if !first { builder.push(", "); }
        builder.push("release_date = ");
        builder.push_bind(release_date);
        first = false;
    }

    if let Some(bpm) = &req.bpm {
        if !first { builder.push(", "); }
        builder.push("bpm = ");
        builder.push_bind(bpm);
        first = false;
    }

    if let Some(initial_key) = &req.initial_key {
        if !first { builder.push(", "); }
        builder.push("initial_key = ");
        builder.push_bind(initial_key);
        first = false;
    }

    if let Some(true) = req.clear_bpm {
        if !first { builder.push(", "); }
        builder.push("bpm = NULL, beat_grid_offset = NULL");
        first = false;
    }

    if let Some(true) = req.clear_key {
        if !first { builder.push(", "); }
        builder.push("initial_key = NULL");
        first = false;
    }

    if let Some(true) = req.clear_beat_map {
        if !first { builder.push(", "); }
        builder.push("beat_map = NULL");
        first = false;
    }

    // Force metadata_analyzed to false if we are clearing values so they can be re-analyzed
    if (req.clear_bpm == Some(true) || req.clear_key == Some(true) || req.clear_beat_map == Some(true)) && !first {
        builder.push(", metadata_analyzed = false");
    }

    if first {
        return Ok(0);
    }

    builder.push(", updated_at = ").push_bind(chrono::Utc::now());
    builder.push(" WHERE id IN (");
    
    let mut it = req.ids.iter().peekable();
    while let Some(id) = it.next() {
        builder.push_bind(id);
        if it.peek().is_some() {
            builder.push(", ");
        }
    }
    builder.push(")");

    let updated_count = builder.build().execute(&db.pool).await?.rows_affected() as usize;

    if updated_count > 0 {
        // Get new files for audit log
        let mut new_files_builder = QueryBuilder::new(format!("{} WHERE mf.id IN (", select_music_files()));
        let mut it = req.ids.iter().peekable();
        while let Some(id) = it.next() {
            new_files_builder.push_bind(id);
            if it.peek().is_some() {
                new_files_builder.push(", ");
            }
        }
        new_files_builder.push(")");
        let new_files = new_files_builder.build_query_as::<MusicFile>().fetch_all(&db.pool).await?;

        // Create audit logs for each record
        for old_file in old_files {
            let new_file = new_files.iter().find(|f| f.id == old_file.id);
            if let Some(new_file) = new_file {
                let _ = audit_service::create_audit_log(db, CreateAuditLogRequest {
                    table_name: "music_files".to_string(),
                    record_id: old_file.id,
                    action: "BULK_UPDATE".to_string(),
                    old_values: Some(serde_json::to_value(&old_file).unwrap_or(serde_json::Value::Null)),
                    new_values: Some(serde_json::to_value(new_file).unwrap_or(serde_json::Value::Null)),
                    user_id: None,
                }).await;
            }
        }
    }

    Ok(updated_count)
}

/// Bulk rename music files by regex pattern
pub async fn bulk_rename_by_regex(
    db: &Database,
    req: BulkRenameByRegexRequest,
) -> Result<BulkRenameResponse, Box<dyn std::error::Error>> {
    // Validate the regex pattern
    let re = regex::Regex::new(&req.pattern)?;
    
    // Validate the field name
    let field = match req.field.as_str() {
        "title" | "artist" | "album" => req.field.as_str(),
        _ => return Err("Invalid field. Must be 'title', 'artist', or 'album'".into()),
    };
    
    // Get all music files
    let sql = format!("{} ORDER BY mf.title", select_music_files());
    let all_music = sqlx::query_as::<_, MusicFile>(&sql)
        .fetch_all(&db.pool)
        .await?;

    let mut updated_files = Vec::new();
    
    // Process each file
    for music_file in all_music {
        let old_value = match field {
            "title" => music_file.title.clone(),
            "artist" => music_file.artist.clone().unwrap_or_default(),
            "album" => music_file.album.clone().unwrap_or_default(),
            _ => continue,
        };
        
        // Check if the regex matches
        if !re.is_match(&old_value) {
            continue;
        }
        
        // Apply the replacement
        let new_value = re.replace(&old_value, req.replacement.as_str()).to_string();
        
        // Skip if the value hasn't changed
        if new_value == old_value {
            continue;
        }
        
        // Update the field in the database, then re-fetch via JOIN to include genre_name
        let update_sql = format!("UPDATE music_files SET {} = $1, updated_at = NOW() WHERE id = $2", field);
        let bind_value: Option<&str> = if field != "title" && new_value.is_empty() {
            None
        } else {
            Some(&new_value)
        };
        sqlx::query(&update_sql)
            .bind(bind_value)
            .bind(music_file.id)
            .execute(&db.pool)
            .await?;

        if let Some(updated) = get_music_file(db, music_file.id).await? {
            updated_files.push(updated);
        }
    }
    
    let updated_count = updated_files.len() as i32;
    
    Ok(BulkRenameResponse {
        updated_count,
        updated_files,
    })
}

/// Bulk add music files to a playlist by regex pattern
pub async fn bulk_add_to_playlist_by_regex(
    db: &Database,
    req: BulkAddToPlaylistByRegexRequest,
) -> Result<BulkAddToPlaylistResponse, Box<dyn std::error::Error>> {
    // Validate the regex pattern
    let re = regex::Regex::new(&req.pattern)?;
    
    // Validate the field name
    let field = match req.field.as_str() {
        "title" | "artist" | "album" => req.field.as_str(),
        _ => return Err("Invalid field. Must be 'title', 'artist', or 'album'".into()),
    };
    
    // Verify the playlist exists
    let playlist_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM playlists WHERE id = $1)"
    )
    .bind(req.playlist_id)
    .fetch_one(&db.pool)
    .await?;
    
    if !playlist_exists {
        return Err("Playlist not found".into());
    }
    
    // Get all music files that match the pattern
    let sql = format!("{} ORDER BY mf.title", select_music_files());
    let all_music = sqlx::query_as::<_, MusicFile>(&sql)
        .fetch_all(&db.pool)
        .await?;
    
    let mut added_count = 0;
    
    // Process each file
    for music_file in all_music {
        let value = match field {
            "title" => &music_file.title,
            "artist" => {
                if let Some(ref artist) = music_file.artist {
                    artist
                } else {
                    continue;
                }
            },
            "album" => {
                if let Some(ref album) = music_file.album {
                    album
                } else {
                    continue;
                }
            },
            _ => continue,
        };
        
        // Check if the regex matches
        if !re.is_match(value) {
            continue;
        }
        
        // Check if the track is already in the playlist
        let already_in_playlist: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM playlist_items WHERE playlist_id = $1 AND music_file_id = $2)"
        )
        .bind(req.playlist_id)
        .bind(music_file.id)
        .fetch_one(&db.pool)
        .await?;
        
        if already_in_playlist {
            continue;
        }
        
        // Get the next position
        let max_position: Option<i32> = sqlx::query_scalar(
            "SELECT MAX(position) FROM playlist_items WHERE playlist_id = $1"
        )
        .bind(req.playlist_id)
        .fetch_one(&db.pool)
        .await?;
        
        let position = max_position.unwrap_or(0) + 1;
        
        // Add the track to the playlist
        let now = chrono::Utc::now();
        sqlx::query(
            "INSERT INTO playlist_items (id, playlist_id, music_file_id, position, created_at) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(Uuid::new_v4())
        .bind(req.playlist_id)
        .bind(music_file.id)
        .bind(position)
        .bind(now)
        .execute(&db.pool)
        .await?;
        
        // Update the playlist's updated_at timestamp
        sqlx::query("UPDATE playlists SET updated_at = $1 WHERE id = $2")
            .bind(now)
            .bind(req.playlist_id)
            .execute(&db.pool)
            .await?;
        
        added_count += 1;
    }
    
    // Get the total track count for the playlist
    let total_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM playlist_items WHERE playlist_id = $1"
    )
    .bind(req.playlist_id)
    .fetch_one(&db.pool)
    .await?;
    
    Ok(BulkAddToPlaylistResponse {
        added_count,
        total_playlist_count: total_count,
    })
}
