use crate::db::Database;
use crate::models::{CreateStreamRequest, InternetStream, UpdateStreamRequest};
use sqlx::QueryBuilder;
use uuid::Uuid;

pub async fn list_streams(db: &Database) -> Result<Vec<InternetStream>, sqlx::Error> {
    sqlx::query_as::<_, InternetStream>(
        "SELECT id, name, url, genre, description, created_at, updated_at FROM internet_streams ORDER BY name ASC",
    )
    .fetch_all(&db.pool)
    .await
}

pub async fn create_stream(
    db: &Database,
    payload: CreateStreamRequest,
) -> Result<InternetStream, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO internet_streams (id, name, url, genre, description, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&payload.url)
    .bind(&payload.genre)
    .bind(&payload.description)
    .bind(now)
    .bind(now)
    .execute(&db.pool)
    .await?;

    Ok(InternetStream {
        id,
        name: payload.name,
        url: payload.url,
        genre: payload.genre,
        description: payload.description,
        created_at: now,
        updated_at: now,
    })
}

pub async fn update_stream(
    db: &Database,
    id: Uuid,
    payload: UpdateStreamRequest,
) -> Result<Option<InternetStream>, sqlx::Error> {
    let mut builder = QueryBuilder::new("UPDATE internet_streams SET ");
    let mut separated = builder.separated(", ");

    if let Some(name) = payload.name {
        separated.push("name = ");
        separated.push_bind(name);
    }
    if let Some(url) = payload.url {
        separated.push("url = ");
        separated.push_bind(url);
    }
    if let Some(genre) = payload.genre {
        separated.push("genre = ");
        separated.push_bind(genre);
    }
    if let Some(description) = payload.description {
        separated.push("description = ");
        separated.push_bind(description);
    }

    separated.push("updated_at = NOW()");
    builder.push(" WHERE id = ");
    builder.push_bind(id);
    builder.push(" RETURNING id, name, url, genre, description, created_at, updated_at");

    builder
        .build_query_as::<InternetStream>()
        .fetch_optional(&db.pool)
        .await
}

pub async fn delete_stream(db: &Database, id: Uuid) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("DELETE FROM internet_streams WHERE id = $1")
        .bind(id)
        .execute(&db.pool)
        .await?;
    Ok(res.rows_affected() > 0)
}
