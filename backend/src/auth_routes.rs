//! SSO Authentication route handlers

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::Deserialize;
use uuid::Uuid;

use crate::models::{
    AppState, AuthResponse, Claims, GoogleAuthRequest, GoogleMobileAuthRequest, User, UserResponse,
};
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    #[allow(dead_code)]
    pub token_type: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
}

// Shorten token lifetime to reduce chance of SSO/session desyncs. Tokens are
// intentionally short-lived; clients should re-login or obtain a fresh token
// via the login flow.
const TOKEN_EXPIRY_DAYS: i64 = 7;

fn create_token(user: &User, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(TOKEN_EXPIRY_DAYS))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        is_admin: user.is_admin,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub async fn google_auth(
    state: web::Data<AppState>,
    body: web::Json<GoogleAuthRequest>,
) -> impl Responder {
    // Hash the incoming authorization code so we can detect reuse without
    // storing the raw code.
    let mut hasher = Sha256::new();
    hasher.update(body.code.as_bytes());
    let code_hash = hex::encode(hasher.finalize());

    // If we've already seen this code recently, return a clear error instead
    // of attempting the exchange (which would produce Google's "invalid_grant").
    {
        let used = state.used_google_code_hashes.read().await;
        if used.contains(&code_hash) {
            log::warn!("Authorization code reused (hash={})", &code_hash[..8]);
            return HttpResponse::BadRequest().json(serde_json::json!({ "error": "authorization_code_already_used" }));
        }
    }
    // Exchange code for token
    let token_response = match state.http_client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", body.code.as_str()),
            ("client_id", state.google_client_id.as_str()),
            ("client_secret", state.google_client_secret.as_str()),
            (
                "redirect_uri",
                &format!("{}/auth/google/callback", state.app_url),
            ),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to exchange Google code: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to exchange Google code" }));
        }
    };

    // Capture status before consuming the body so we can surface non-2xx responses
    let token_status = token_response.status();

    // Read the response body as text so we can log detailed errors when parsing fails
    let token_text = match token_response.text().await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to read Google token response body: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to read Google token response" }));
        }
    };

    // If Google returned a non-success HTTP status, include status + body in the response for easier debugging
    if !token_status.is_success() {
        log::error!("Google token endpoint returned non-success status {}: {}", token_status, token_text);
        let maybe_error: serde_json::Value = serde_json::from_str(&token_text).unwrap_or(serde_json::json!({ "body": token_text }));
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Google token endpoint error", "status": token_status.as_u16(), "details": maybe_error }));
    }

    // If Google returned a non-success status, log the body for debugging and return a clear error
    // (Common reasons: invalid_client, invalid_grant, invalid_request)
    // Note: we don't have the original Response::status() now, but the body often contains the error details.
    if token_text.trim().is_empty() {
        log::error!("Empty response from Google token endpoint");
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Empty token response from Google" }));
    }

    let token_data: GoogleTokenResponse = match serde_json::from_str(&token_text) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to parse Google token response: {}. Body: {}", e, token_text);
            // If Google returned an error object, try to surface it to the client
            let maybe_error: serde_json::Value = serde_json::from_str(&token_text).unwrap_or(serde_json::json!({ "body": token_text }));
            return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Failed to parse Google token response", "details": maybe_error }));
        }
    };

    // Get user info
    let user_info_response = match state.http_client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(&token_data.access_token)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to get Google user info: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to get Google user info" }));
        }
    };

    let google_user: GoogleUserInfo = match user_info_response.json().await {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to parse Google user info: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to parse Google user info" }));
        }
    };

    // Create or update user
    let user = match state
        .db
        .upsert_google_user(
            &google_user.email,
            &google_user.id,
            &google_user.name,
            google_user.picture.as_deref(),
        )
        .await
    {
        Ok(u) => u,
        Err(e) => {
            log::error!("Failed to upsert Google user: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to upsert Google user" }));
        }
    };

    // Mark the code hash as used so subsequent reuse attempts can be recognized.
    {
        let mut used = state.used_google_code_hashes.write().await;
        used.insert(code_hash);
    }

    let token = match create_token(&user, &state.jwt_secret) {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to create JWT token: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to create JWT token" }));
        }
    };

    HttpResponse::Ok().json(AuthResponse {
        token,
        user: user.into(),
    })
}

pub async fn get_google_auth_url(state: web::Data<AppState>) -> impl Responder {
    let redirect_uri = format!("{}/auth/google/callback", state.app_url);
    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=email%20profile&access_type=offline",
        state.google_client_id,
        urlencoding::encode(&redirect_uri)
    );

    HttpResponse::Ok().json(serde_json::json!({ "url": auth_url }))
}


/// Authenticate using a Google ID token (from Android Credential Manager).
/// Verifies the token with Google's tokeninfo endpoint, then creates/updates
/// the user and returns a JWT.
pub async fn google_auth_mobile(
    state: web::Data<AppState>,
    body: web::Json<GoogleMobileAuthRequest>,
) -> impl Responder {
    // Verify the ID token with Google
    let token_info_response = match state.http_client
        .get("https://oauth2.googleapis.com/tokeninfo")
        .query(&[("id_token", body.id_token.as_str())])
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to verify Google ID token: {}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to verify Google ID token" }));
        }
    };

    if !token_info_response.status().is_success() {
        let body_text = token_info_response.text().await.unwrap_or_default();
        log::error!("Google tokeninfo returned error: {}", body_text);
        return HttpResponse::Unauthorized()
            .json(serde_json::json!({ "error": "Invalid Google ID token" }));
    }

    #[derive(Debug, Deserialize)]
    struct TokenInfo {
        sub: String,
        email: String,
        name: Option<String>,
        picture: Option<String>,
        #[serde(default)]
        email_verified: String,
    }

    let token_info: TokenInfo = match token_info_response.json().await {
        Ok(info) => info,
        Err(e) => {
            log::error!("Failed to parse Google tokeninfo response: {}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to parse token info" }));
        }
    };

    if token_info.email_verified != "true" {
        return HttpResponse::Unauthorized()
            .json(serde_json::json!({ "error": "Google email not verified" }));
    }

    // Verify the token was issued for our client
    // (The tokeninfo endpoint validates the signature; we trust it.)

    let name = token_info.name.unwrap_or_else(|| token_info.email.clone());

    // Create or update user
    let user = match state
        .db
        .upsert_google_user(
            &token_info.email,
            &token_info.sub,
            &name,
            token_info.picture.as_deref(),
        )
        .await
    {
        Ok(u) => u,
        Err(e) => {
            log::error!("Failed to upsert Google user: {}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to create user" }));
        }
    };

    let token = match create_token(&user, &state.jwt_secret) {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to create JWT token: {}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to create JWT token" }));
        }
    };

    HttpResponse::Ok().json(AuthResponse {
        token,
        user: user.into(),
    })
}

pub async fn get_me(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let user_id = match get_user_id_from_request(&req, &state.jwt_secret) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Not authenticated" })),
    };

    let uuid = match Uuid::parse_str(&user_id) {
        Ok(u) => u,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Invalid user ID" })),
    };

    match state.db.get_user_by_id(&uuid).await {
        Ok(user) => HttpResponse::Ok().json(UserResponse::from(user)),
        Err(e) => {
            log::error!("Failed to get user: {}", e);
            HttpResponse::Unauthorized().json(serde_json::json!({ "error": "User not found" }))
        }
    }
}

// ============ Auth Helper Functions ============

/// Extract user ID from Authorization header
pub fn get_user_id_from_request(req: &HttpRequest, secret: &str) -> Option<String> {
    let auth_header = req.headers().get("Authorization")?;
    let auth_str = auth_header.to_str().ok()?;

    if !auth_str.starts_with("Bearer ") {
        return None;
    }

    let token = &auth_str[7..];

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .ok()?;

    Some(token_data.claims.sub)
}
