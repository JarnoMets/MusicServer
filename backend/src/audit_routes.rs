use actix_web::{web, HttpResponse};
use uuid::Uuid;
use crate::models::{AppState, audit::RevertChangeRequest};
use crate::services::audit_service;

pub async fn get_audit_logs_handler(
    state: web::Data<AppState>,
    query: web::Query<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let table_name = query.get("table_name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let record_id = query.get("record_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok());

    match audit_service::get_audit_logs(db, table_name, record_id).await {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(e) => {
            log::error!("Error fetching audit logs: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn revert_audit_log_handler(
    state: web::Data<AppState>,
    req: web::Json<RevertChangeRequest>,
) -> HttpResponse {
    let db = &state.db;
    match audit_service::revert_audit_log(db, req.audit_log_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "success": true })),
        Err(e) => {
            log::error!("Error reverting audit log {}: {}", req.audit_log_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() }))
        }
    }
}
