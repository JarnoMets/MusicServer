use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, web};
use actix_web::http::header::AUTHORIZATION;
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;
use uuid::Uuid;
use sha2::{Digest, Sha256};

use crate::models::{AppState, Claims};
use jsonwebtoken::{decode, DecodingKey, Validation};

// ============================================================================
// Auth info types (attached to every authenticated request as an extension)
// ============================================================================

/// How the request was authenticated.
#[derive(Debug, Clone, PartialEq)]
pub enum AuthType {
    /// Google SSO — JWT token. Full access, user identity known.
    Jwt,
    /// Static admin token from `ADMIN_TOKEN_SHA256` env var. Full admin access.
    AdminToken,
    /// Static API token from `API_TOKEN_SHA256` env var. Read/write, no user identity.
    ApiToken,
    /// User-generated personal access token (`ms_…`). Permissions stored in DB.
    UserAccessToken,
}

/// Permission flags for a user-generated access token.
#[derive(Debug, Clone)]
pub struct TokenPermissions {
    pub can_read: bool,
    pub can_create: bool,
    pub can_edit: bool,
    pub can_delete: bool,
}

/// Authentication context attached to every successfully authenticated request.
/// Retrieve it in handlers via `req.extensions().get::<AuthInfo>()`.
#[derive(Debug, Clone)]
pub struct AuthInfo {
    /// UUID of the authenticated user. None for static admin/API tokens.
    pub user_id: Option<Uuid>,
    /// Whether this authentication context has full admin privileges.
    pub is_admin: bool,
    /// How the request was authenticated.
    pub auth_type: AuthType,
    /// ID of the access_token row (only for `UserAccessToken`).
    pub token_id: Option<Uuid>,
    /// Per-token permissions (only for `UserAccessToken`).
    pub token_permissions: Option<TokenPermissions>,
}

// ============================================================================
// Middleware wiring
// ============================================================================

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let state = req.app_data::<web::Data<AppState>>().expect("AppState not found");
        let jwt_secret = state.jwt_secret.clone();
        let pool = state.db.pool.clone();

        // Extract the raw token string synchronously — avoids moving `req` before the async block.
        let token_opt = extract_token_str(&req);

        Box::pin(async move {
            let auth_info = match token_opt {
                None => None,
                Some(ref token) => authenticate(token, &jwt_secret, &pool).await,
            };

            let auth_info = match auth_info {
                None => {
                    log::debug!("AuthMiddleware: request UNAUTHORIZED");
                    let resp = actix_web::HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": "unauthorized",
                        "message": "No valid authentication token provided"
                    }));
                    return Err(actix_web::error::InternalError::from_response("unauthorized", resp).into());
                }
                Some(info) => info,
            };

            // For user-generated access tokens enforce per-token permissions based on HTTP method.
            if let AuthType::UserAccessToken = &auth_info.auth_type {
                let perms = auth_info.token_permissions.as_ref()
                    .expect("UserAccessToken always has permissions");
                let method = req.method().as_str();
                let allowed = match method {
                    "GET" | "HEAD" | "OPTIONS" => perms.can_read,
                    "POST" => perms.can_create,
                    "PATCH" | "PUT" => perms.can_edit,
                    "DELETE" => perms.can_delete,
                    _ => false,
                };

                if !allowed {
                    log::debug!("AuthMiddleware: access token lacks {} permission", method);
                    let resp = actix_web::HttpResponse::Forbidden().json(serde_json::json!({
                        "error": "forbidden",
                        "message": "This token does not have permission for this operation"
                    }));
                    return Err(actix_web::error::InternalError::from_response("forbidden", resp).into());
                }

                // Best-effort background update of last_used_at (fire-and-forget).
                if let Some(token_id) = auth_info.token_id {
                    let pool_bg = pool.clone();
                    tokio::spawn(async move {
                        let _ = sqlx::query(
                            "UPDATE access_tokens SET last_used_at = NOW() WHERE id = $1"
                        )
                        .bind(token_id)
                        .execute(&pool_bg)
                        .await;
                    });
                }
            }

            log::debug!("AuthMiddleware: authorized via {:?}", auth_info.auth_type);

            // Attach auth context so handlers can read identity / permissions.
            req.extensions_mut().insert(auth_info);

            svc.call(req).await
        })
    }
}

// ============================================================================
// Token extraction helpers
// ============================================================================

/// Pull the raw token string from the Authorization header or `?token=` query param.
fn extract_token_str(req: &ServiceRequest) -> Option<String> {
    // Primary: Authorization: Bearer <token>
    if let Some(auth_hdr) = req.headers().get(AUTHORIZATION) {
        if let Ok(s) = auth_hdr.to_str() {
            if let Some(token) = s.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }

    // Fallback: ?token=<value> — needed for EventSource (SSE) which cannot set headers.
    if let Some(q) = req.uri().query() {
        for pair in q.split('&') {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or("");
            let val = parts.next().unwrap_or("");
            if key == "token" {
                let decoded = urlencoding::decode(val)
                    .unwrap_or_else(|_| val.into())
                    .into_owned();
                return Some(decoded);
            }
        }
    }

    None
}

// ============================================================================
// Authentication logic
// ============================================================================

/// Validate a raw token string and return an `AuthInfo` on success.
async fn authenticate(
    token: &str,
    jwt_secret: &str,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> Option<AuthInfo> {
    // 1. Static admin token (env var ADMIN_TOKEN_SHA256)
    if crate::auth::verify_admin_token(token) {
        return Some(AuthInfo {
            user_id: None,
            is_admin: true,
            auth_type: AuthType::AdminToken,
            token_id: None,
            token_permissions: None,
        });
    }

    // 2. Static API token (env var API_TOKEN_SHA256)
    if crate::auth::verify_api_token(token) {
        return Some(AuthInfo {
            user_id: None,
            is_admin: false,
            auth_type: AuthType::ApiToken,
            token_id: None,
            token_permissions: None,
        });
    }

    // 3. JWT (Google SSO)
    if let Ok(data) = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        let user_id = Uuid::parse_str(&data.claims.sub).ok();
        return Some(AuthInfo {
            user_id,
            is_admin: data.claims.is_admin,
            auth_type: AuthType::Jwt,
            token_id: None,
            token_permissions: None,
        });
    }

    // 4. User-generated access token (ms_ prefix)
    if token.starts_with("ms_") {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let hash = hex::encode(hasher.finalize());

        let result = sqlx::query_as::<_, crate::models::AccessToken>(
            r#"
            SELECT id, user_id, name, token_hash, can_read, can_create, can_edit, can_delete,
                   last_used_at, expires_at, created_at
            FROM access_tokens
            WHERE token_hash = $1
            "#,
        )
        .bind(&hash)
        .fetch_optional(pool)
        .await;

        if let Ok(Some(at)) = result {
            // Reject expired tokens
            if let Some(exp) = at.expires_at {
                if exp < chrono::Utc::now() {
                    log::debug!("AuthMiddleware: access token expired");
                    return None;
                }
            }

            return Some(AuthInfo {
                user_id: Some(at.user_id),
                is_admin: false,
                auth_type: AuthType::UserAccessToken,
                token_id: Some(at.id),
                token_permissions: Some(TokenPermissions {
                    can_read: at.can_read,
                    can_create: at.can_create,
                    can_edit: at.can_edit,
                    can_delete: at.can_delete,
                }),
            });
        }
    }

    None
}
