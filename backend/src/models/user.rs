use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub google_id: Option<String>,
    pub avatar_url: Option<String>,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    #[allow(dead_code)]
    pub fn new_google(email: String, google_id: String, name: String, avatar_url: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            email,
            name,
            google_id: Some(google_id),
            avatar_url,
            is_admin: false,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub is_admin: bool,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            avatar_url: user.avatar_url,
            is_admin: user.is_admin,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GoogleAuthRequest {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleMobileAuthRequest {
    pub id_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub is_admin: bool,
    pub exp: usize,
}
