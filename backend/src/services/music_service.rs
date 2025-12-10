use crate::db::Database;
use crate::models::{CreateMusicFileRequest, MusicFile, MusicQueryParams, UpdateMusicFileRequest};
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
