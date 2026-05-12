//! Personal Access Token management endpoints.
//!
//! All routes require full JWT authentication (Google SSO).
//! User-generated access tokens (`ms_…`) cannot manage other tokens —
//! this is an intentional security boundary.
//!
//! # Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET    | /api/tokens | List all tokens for the current user |
//! | POST   | /api/tokens | Create a new token (returns plaintext once) |
//! | GET    | /api/tokens/:id | Get a specific token's metadata |
//! | PATCH  | /api/tokens/:id | Update a token's name or permissions |
//! | DELETE | /api/tokens/:id | Revoke / delete a token |

use actix_web::{web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::auth_middleware::{AuthInfo, AuthType};
use crate::db::Database;
use crate::models::access_token::{
    CreateAccessTokenRequest, CreateAccessTokenResponse, UpdateAccessTokenRequest,
};
use crate::models::AppState;

// ============================================================================
// Auth guard helper
// ============================================================================

/// Ensure the request was authenticated via JWT (Google SSO) or a static admin token.
/// User-generated access tokens are not allowed to manage tokens.
fn require_jwt_user(req: &HttpRequest) -> Result<Uuid, HttpResponse> {
    if let Some(auth_info) = req.extensions().get::<AuthInfo>() {
        match auth_info.auth_type {
            AuthType::Jwt | AuthType::AdminToken => {
                if let Some(uid) = auth_info.user_id {
                    return Ok(uid);
                }
                // Admin token without a user ID — no associated account.
                return Err(HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "forbidden",
                    "message": "Static admin tokens cannot manage personal access tokens"
                })));
            }
            _ => {
                return Err(HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "forbidden",
                    "message": "Personal access tokens cannot manage other tokens. Please authenticate with Google SSO."
                })));
            }
        }
    }
    Err(HttpResponse::Unauthorized().json(serde_json::json!({ "error": "unauthorized" })))
}

// ============================================================================
// Handlers
// ============================================================================

/// `GET /api/tokens`
///
/// Returns all access tokens owned by the authenticated user.
/// Token hashes are never returned — only metadata.
pub async fn list_tokens(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let user_id = match require_jwt_user(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    match state.db.list_access_tokens(&user_id).await {
        Ok(tokens) => {
            let responses: Vec<_> = tokens
                .into_iter()
                .map(crate::models::AccessTokenResponse::from)
                .collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => {
            log::error!("Failed to list access tokens for user {}: {}", user_id, e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to list tokens" }))
        }
    }
}

/// `POST /api/tokens`
///
/// Create a new personal access token.
///
/// # Request body
/// ```json
/// {
///   "name": "Home Assistant",
///   "can_read": true,
///   "can_create": false,
///   "can_edit": false,
///   "can_delete": false,
///   "expires_at": null
/// }
/// ```
///
/// # Response
/// Returns a `CreateAccessTokenResponse` containing the **plaintext token** (`token` field).
/// **This is the only time the token is returned.** Store it securely.
pub async fn create_token(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<CreateAccessTokenRequest>,
) -> HttpResponse {
    let user_id = match require_jwt_user(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    // Validate name is not empty
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return HttpResponse::UnprocessableEntity().json(serde_json::json!({
            "error": "validation_error",
            "message": "Token name cannot be empty"
        }));
    }
    if name.len() > 255 {
        return HttpResponse::UnprocessableEntity().json(serde_json::json!({
            "error": "validation_error",
            "message": "Token name must be 255 characters or fewer"
        }));
    }

    // Require at least one permission
    if !body.can_read && !body.can_create && !body.can_edit && !body.can_delete {
        return HttpResponse::UnprocessableEntity().json(serde_json::json!({
            "error": "validation_error",
            "message": "Token must have at least one permission enabled"
        }));
    }

    let (raw_token, token_hash) = Database::generate_access_token();

    match state
        .db
        .create_access_token(
            &user_id,
            &name,
            &token_hash,
            body.can_read,
            body.can_create,
            body.can_edit,
            body.can_delete,
            body.expires_at,
        )
        .await
    {
        Ok(token) => {
            let info = crate::models::AccessTokenResponse::from(token);
            HttpResponse::Created().json(CreateAccessTokenResponse {
                token: raw_token,
                info,
            })
        }
        Err(e) => {
            log::error!("Failed to create access token: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to create token" }))
        }
    }
}

/// `GET /api/tokens/:id`
///
/// Retrieve metadata for a specific token owned by the current user.
pub async fn get_token(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let user_id = match require_jwt_user(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let token_id = path.into_inner();

    match state.db.get_access_token_by_id(&token_id, &user_id).await {
        Ok(Some(token)) => HttpResponse::Ok().json(crate::models::AccessTokenResponse::from(token)),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({ "error": "Token not found" })),
        Err(e) => {
            log::error!("Failed to get access token {}: {}", token_id, e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to get token" }))
        }
    }
}

/// `PATCH /api/tokens/:id`
///
/// Update a token's name, permissions, or expiry.
///
/// # Request body (all fields optional)
/// ```json
/// {
///   "name": "Updated name",
///   "can_read": true,
///   "can_create": true,
///   "can_edit": false,
///   "can_delete": false,
///   "expires_at": "2026-12-31T00:00:00Z",
///   "clear_expires_at": false
/// }
/// ```
/// Set `clear_expires_at: true` to remove the expiry (token will never expire).
pub async fn update_token(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateAccessTokenRequest>,
) -> HttpResponse {
    let user_id = match require_jwt_user(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let token_id = path.into_inner();

    // Validate name if provided
    if let Some(ref name) = body.name {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return HttpResponse::UnprocessableEntity().json(serde_json::json!({
                "error": "validation_error",
                "message": "Token name cannot be empty"
            }));
        }
    }

    match state
        .db
        .update_access_token(
            &token_id,
            &user_id,
            body.name.as_deref(),
            body.can_read,
            body.can_create,
            body.can_edit,
            body.can_delete,
            body.expires_at,
            body.clear_expires_at,
        )
        .await
    {
        Ok(Some(token)) => HttpResponse::Ok().json(crate::models::AccessTokenResponse::from(token)),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({ "error": "Token not found" })),
        Err(e) => {
            log::error!("Failed to update access token {}: {}", token_id, e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to update token" }))
        }
    }
}

/// `DELETE /api/tokens/:id`
///
/// Revoke and permanently delete a token.
/// The token will immediately stop working.
pub async fn delete_token(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let user_id = match require_jwt_user(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let token_id = path.into_inner();

    match state.db.delete_access_token(&token_id, &user_id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => {
            HttpResponse::NotFound().json(serde_json::json!({ "error": "Token not found" }))
        }
        Err(e) => {
            log::error!("Failed to delete access token {}: {}", token_id, e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to delete token" }))
        }
    }
}
