use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub table_name: String,
    pub record_id: Uuid,
    pub action: String,
    pub old_values: Option<Value>,
    pub new_values: Option<Value>,
    pub user_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuditLogRequest {
    pub table_name: String,
    pub record_id: Uuid,
    pub action: String,
    pub old_values: Option<Value>,
    pub new_values: Option<Value>,
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevertChangeRequest {
    pub audit_log_id: Uuid,
}
