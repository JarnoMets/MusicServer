use crate::db::Database;
use crate::models::{BulkAddToPlaylistByRegexRequest, BulkAddToPlaylistResponse, BulkRenameByRegexRequest, BulkRenameResponse, CreateMusicFileRequest, MusicFile, MusicQueryParams, UpdateMusicFileRequest};
use sqlx::QueryBuilder;
use uuid::Uuid;

#[allow(dead_code)]
pub struct MusicService;

pub async fn get_all_music_files(
    db: &Database,
    params: MusicQueryParams,
) -> Result<Vec<MusicFile>, sqlx::Error> {
    let mut builder = QueryBuilder::new(
        "SELECT id, title, artist, album, genre, guessed_genre, release_date, duration, file_path, track_number, file_hash, created_at, updated_at FROM music_files WHERE 1=1",
    );

    if let Some(search) = &params.search {
        builder
            .push(" AND (")
            .push("LOWER(title) LIKE ")
            .push_bind(format!("%{}%", search.to_lowercase()))
            .push(" OR LOWER(artist) LIKE ")
            .push_bind(format!("%{}%", search.to_lowercase()))
            .push(" OR LOWER(album) LIKE ")
            .push_bind(format!("%{}%", search.to_lowercase()))
            .push(")");
    }

    if let Some(genre) = &params.genre {
        builder
            .push(" AND (")
            .push("genre = ")
            .push_bind(genre)
            .push(" OR guessed_genre = ")
            .push_bind(genre)
            .push(")");
    }

    if let Some(artist) = &params.artist {
        builder
            .push(" AND LOWER(artist) = LOWER(")
            .push_bind(artist)
            .push(")");
    }

    // Filter for unconfirmed genres: has guessed_genre but no confirmed genre
    if params.unconfirmed_only.unwrap_or(false) {
        builder.push(" AND (genre IS NULL OR genre = '') AND guessed_genre IS NOT NULL AND guessed_genre != ''");
    }

    let sort_column = match params.sort.as_deref() {
        Some("artist") => "artist",
        Some("album") => "album",
        Some("created_at") => "created_at",
        Some("updated_at") => "updated_at",
        _ => "title",
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
        "INSERT INTO music_files (id, title, artist, album, genre, guessed_genre, release_date, duration, file_path, track_number, file_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"
    )
    .bind(id)
    .bind(&req.title)
    .bind(&req.artist)
    .bind(&req.album)
    .bind(&req.genre)
    .bind(&req.guessed_genre)
    .bind(req.release_date)
    .bind(req.duration)
    .bind(&req.file_path)
    .bind(req.track_number)
    .bind(&req.file_hash)
    .bind(now)
    .bind(now)
    .execute(&db.pool)
    .await?;

    // Genre detection is now handled separately by background processes
    // This keeps the upload fast by not blocking on genre lookups
    let final_guessed_genre = req.guessed_genre.clone();

    Ok(MusicFile {
        id,
        title: req.title,
        artist: req.artist,
        album: req.album,
        genre: req.genre,
        guessed_genre: final_guessed_genre,
        release_date: req.release_date,
        duration: req.duration,
        file_path: req.file_path,
        track_number: req.track_number,
        file_hash: None,
        created_at: now,
        updated_at: now,
    })
}

pub async fn update_music_file(
    db: &Database,
    id: Uuid,
    req: UpdateMusicFileRequest,
) -> Result<Option<MusicFile>, sqlx::Error> {
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
    if let Some(genre) = req.genre {
        if has_set {
            builder.push(", ");
        }
        builder.push("genre = ").push_bind(genre);
        has_set = true;
    }
    if let Some(guessed_genre) = req.guessed_genre {
        if has_set {
            builder.push(", ");
        }
        builder.push("guessed_genre = ").push_bind(guessed_genre);
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

    if !has_set {
        return get_music_file(db, id).await;
    }

    builder.push(", updated_at = NOW() WHERE id = ").push_bind(id).push(" RETURNING id, title, artist, album, genre, guessed_genre, release_date, duration, file_path, track_number, file_hash, created_at, updated_at");
    builder
        .build_query_as::<MusicFile>()
        .fetch_optional(&db.pool)
        .await
}

pub async fn get_music_file(db: &Database, id: Uuid) -> Result<Option<MusicFile>, sqlx::Error> {
    sqlx::query_as::<_, MusicFile>(
        "SELECT id, title, artist, album, genre, guessed_genre, release_date, duration, file_path, track_number, file_hash, created_at, updated_at FROM music_files WHERE id = $1"
    )
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
pub async fn get_by_hash(db: &Database, file_hash: &str) -> Result<Option<MusicFile>, sqlx::Error> {
    sqlx::query_as::<_, MusicFile>(
        "SELECT id, title, artist, album, genre, guessed_genre, release_date, duration, file_path, track_number, file_hash, created_at, updated_at FROM music_files WHERE file_hash = $1"
    )
    .bind(file_hash)
    .fetch_optional(&db.pool)
    .await
}

/// Compute SHA-256 hash of a file
pub async fn compute_file_hash(file_path: &str) -> Result<String, std::io::Error> {
    use sha2::{Digest, Sha256};
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;
    
    let mut file = File::open(file_path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    
    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    let result = hasher.finalize();
    Ok(hex::encode(result))
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
    let all_music = sqlx::query_as::<_, MusicFile>(
        "SELECT id, title, artist, album, genre, guessed_genre, release_date, duration, file_path, track_number, file_hash, created_at, updated_at FROM music_files ORDER BY title"
    )
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
        
        // Update the field in the database
        let updated = match field {
            "title" => {
                sqlx::query_as::<_, MusicFile>(
                    "UPDATE music_files SET title = $1, updated_at = NOW() WHERE id = $2 RETURNING id, title, artist, album, genre, guessed_genre, release_date, duration, file_path, track_number, file_hash, created_at, updated_at"
                )
                .bind(&new_value)
                .bind(music_file.id)
                .fetch_one(&db.pool)
                .await?
            },
            "artist" => {
                sqlx::query_as::<_, MusicFile>(
                    "UPDATE music_files SET artist = $1, updated_at = NOW() WHERE id = $2 RETURNING id, title, artist, album, genre, guessed_genre, release_date, duration, file_path, track_number, file_hash, created_at, updated_at"
                )
                .bind(if new_value.is_empty() { None } else { Some(&new_value) })
                .bind(music_file.id)
                .fetch_one(&db.pool)
                .await?
            },
            "album" => {
                sqlx::query_as::<_, MusicFile>(
                    "UPDATE music_files SET album = $1, updated_at = NOW() WHERE id = $2 RETURNING id, title, artist, album, genre, guessed_genre, release_date, duration, file_path, track_number, file_hash, created_at, updated_at"
                )
                .bind(if new_value.is_empty() { None } else { Some(&new_value) })
                .bind(music_file.id)
                .fetch_one(&db.pool)
                .await?
            },
            _ => continue,
        };
        
        updated_files.push(updated);
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
    let all_music = sqlx::query_as::<_, MusicFile>(
        "SELECT id, title, artist, album, genre, guessed_genre, release_date, duration, file_path, track_number, file_hash, created_at, updated_at FROM music_files ORDER BY title"
    )
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
        sqlx::query(
            "INSERT INTO playlist_items (id, playlist_id, music_file_id, position, created_at) VALUES ($1, $2, $3, $4, NOW())"
        )
        .bind(Uuid::new_v4())
        .bind(req.playlist_id)
        .bind(music_file.id)
        .bind(position)
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
