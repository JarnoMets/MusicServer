use crate::db::Database;
use crate::models::audit::{AuditLog, CreateAuditLogRequest};
use uuid::Uuid;
use serde_json::Value;

pub async fn create_audit_log(
    db: &Database,
    req: CreateAuditLogRequest,
) -> Result<AuditLog, sqlx::Error> {
    let now = chrono::Utc::now();
    let id = Uuid::new_v4();

    sqlx::query_as::<_, AuditLog>(
        r#"
        INSERT INTO audit_logs (id, table_name, record_id, action, old_values, new_values, user_id, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(&req.table_name)
    .bind(req.record_id)
    .bind(&req.action)
    .bind(&req.old_values)
    .bind(&req.new_values)
    .bind(&req.user_id)
    .bind(now)
    .fetch_one(&db.pool)
    .await
}

pub async fn get_audit_logs(
    db: &Database,
    table_name: Option<String>,
    record_id: Option<Uuid>,
) -> Result<Vec<AuditLog>, sqlx::Error> {
    // Use parameterized queries to prevent SQL injection
    match (table_name, record_id) {
        (Some(table), Some(id)) => {
            sqlx::query_as::<_, AuditLog>(
                "SELECT id, table_name, record_id, action, old_values, new_values, user_id, created_at FROM audit_logs WHERE table_name = $1 AND record_id = $2 ORDER BY created_at DESC"
            )
            .bind(table)
            .bind(id)
            .fetch_all(&db.pool)
            .await
        }
        (Some(table), None) => {
            sqlx::query_as::<_, AuditLog>(
                "SELECT id, table_name, record_id, action, old_values, new_values, user_id, created_at FROM audit_logs WHERE table_name = $1 ORDER BY created_at DESC"
            )
            .bind(table)
            .fetch_all(&db.pool)
            .await
        }
        (None, Some(id)) => {
            sqlx::query_as::<_, AuditLog>(
                "SELECT id, table_name, record_id, action, old_values, new_values, user_id, created_at FROM audit_logs WHERE record_id = $1 ORDER BY created_at DESC"
            )
            .bind(id)
            .fetch_all(&db.pool)
            .await
        }
        (None, None) => {
            sqlx::query_as::<_, AuditLog>(
                "SELECT id, table_name, record_id, action, old_values, new_values, user_id, created_at FROM audit_logs ORDER BY created_at DESC"
            )
            .fetch_all(&db.pool)
            .await
        }
    }
}

pub async fn get_audit_log(db: &Database, id: Uuid) -> Result<Option<AuditLog>, sqlx::Error> {
    sqlx::query_as::<_, AuditLog>("SELECT id, table_name, record_id, action, old_values, new_values, user_id, created_at FROM audit_logs WHERE id = $1")
        .bind(id)
        .fetch_optional(&db.pool)
        .await
}

pub async fn revert_audit_log(db: &Database, id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
    let audit_log = get_audit_log(db, id).await?.ok_or("Log not found")?;
    
    // We can only revert if we have old values
    let old_values = audit_log.old_values.ok_or("No old values to revert to")?;
    
    match audit_log.table_name.as_str() {
        "music_files" => revert_music_file(db, audit_log.record_id, old_values.clone()).await?,
        "playlists" => revert_playlist(db, audit_log.record_id, old_values.clone()).await?,
        _ => return Err(format!("Revert for table '{}' not implemented", audit_log.table_name).into()),
    }
    
    // Create a new audit log entry for the revert action itself
    create_audit_log(db, CreateAuditLogRequest {
        table_name: audit_log.table_name,
        record_id: audit_log.record_id,
        action: format!("REVERT_{}", audit_log.id),
        old_values: audit_log.new_values,
        new_values: Some(old_values),
        user_id: Some("system".to_string()),
    }).await?;

    Ok(())
}

async fn revert_music_file(db: &Database, id: Uuid, old_values: Value) -> Result<(), Box<dyn std::error::Error>> {
    let fields: serde_json::Map<String, Value> = old_values.as_object().cloned().ok_or("Invalid old values")?;
    
    let mut builder = sqlx::QueryBuilder::new("UPDATE music_files SET ");
    let mut first = true;
    
    for (key, val) in fields {
        if key == "id" || key == "created_at" || key == "updated_at" {
            continue;
        }
        
        if !first { builder.push(", "); }
        builder.push(format!("{} = ", key));
        
        match val {
            Value::String(s) => { builder.push_bind(s); },
            Value::Number(n) => {
                if let Some(f) = n.as_f64() { builder.push_bind(f); }
                else if let Some(i) = n.as_i64() { builder.push_bind(i); }
            },
            Value::Bool(b) => { builder.push_bind(b); },
            Value::Null => { builder.push("NULL"); },
            _ => { return Err(format!("Unsupported type for field '{}'", key).into()); }
        }
        first = false;
    }
    
    if first { return Ok(()); }
    
    builder.push(" WHERE id = ").push_bind(id);
    builder.build().execute(&db.pool).await?;
    
    Ok(())
}

async fn revert_playlist(db: &Database, id: Uuid, old_values: Value) -> Result<(), Box<dyn std::error::Error>> {
    // Similar to revert_music_file
    let fields: serde_json::Map<String, Value> = old_values.as_object().cloned().ok_or("Invalid old values")?;
    
    let mut builder = sqlx::QueryBuilder::new("UPDATE playlists SET ");
    let mut first = true;
    
    for (key, val) in fields {
        if key == "id" || key == "created_at" || key == "updated_at" {
            continue;
        }
        
        if !first { builder.push(", "); }
        builder.push(format!("{} = ", key));
        
        match val {
            Value::String(s) => { builder.push_bind(s); },
            Value::Number(n) => {
                if let Some(f) = n.as_f64() { builder.push_bind(f); }
                else if let Some(i) = n.as_i64() { builder.push_bind(i); }
            },
            Value::Null => { builder.push("NULL"); },
            _ => { return Err(format!("Unsupported type for field '{}'", key).into()); }
        }
        first = false;
    }
    
    if first { return Ok(()); }
    
    builder.push(" WHERE id = ").push_bind(id);
    builder.build().execute(&db.pool).await?;
    
    Ok(())
}
