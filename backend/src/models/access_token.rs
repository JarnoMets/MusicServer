use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A user-generated personal access token stored in the database.
/// The plaintext token is never stored — only a SHA-256 hex hash.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AccessToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    /// SHA-256 hex hash of the raw token (never store plaintext).
    pub token_hash: String,
    /// Allows GET requests to music, playlists, streams, artists, genres.
    pub can_read: bool,
    /// Allows POST requests — upload tracks, create playlists/streams.
    pub can_create: bool,
    /// Allows PATCH/PUT requests — edit track metadata, update playlists.
    pub can_edit: bool,
    /// Allows DELETE requests — remove tracks, playlists, streams.
    pub can_delete: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Safe API representation — never exposes the token hash.
#[derive(Debug, Serialize)]
pub struct AccessTokenResponse {
    pub id: Uuid,
    pub name: String,
    pub can_read: bool,
    pub can_create: bool,
    pub can_edit: bool,
    pub can_delete: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Returned **only once** after creating a token.  
/// The `token` field will not be accessible again — instruct users to save it.
#[derive(Debug, Serialize)]
pub struct CreateAccessTokenResponse {
    /// The raw token value. Store it securely — it is shown only once.
    pub token: String,
    /// Metadata about the created token.
    pub info: AccessTokenResponse,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccessTokenRequest {
    /// Human-readable label for this token (e.g. "Home Assistant integration").
    pub name: String,
    /// Allow read access (stream music, browse playlists). Defaults to true.
    #[serde(default = "default_true")]
    pub can_read: bool,
    /// Allow create operations (upload, create playlists). Defaults to false.
    #[serde(default)]
    pub can_create: bool,
    /// Allow edit operations (update metadata, rename). Defaults to false.
    #[serde(default)]
    pub can_edit: bool,
    /// Allow delete operations. Defaults to false.
    #[serde(default)]
    pub can_delete: bool,
    /// Optional expiry. If omitted the token never expires.
    pub expires_at: Option<DateTime<Utc>>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccessTokenRequest {
    /// Rename the token.
    pub name: Option<String>,
    pub can_read: Option<bool>,
    pub can_create: Option<bool>,
    pub can_edit: Option<bool>,
    pub can_delete: Option<bool>,
    /// Set a new expiry date, or leave absent to leave unchanged.
    pub expires_at: Option<DateTime<Utc>>,
    /// Set to `true` to remove the expiry (token will never expire).
    #[serde(default)]
    pub clear_expires_at: bool,
}

impl From<AccessToken> for AccessTokenResponse {
    fn from(t: AccessToken) -> Self {
        AccessTokenResponse {
            id: t.id,
            name: t.name,
            can_read: t.can_read,
            can_create: t.can_create,
            can_edit: t.can_edit,
            can_delete: t.can_delete,
            last_used_at: t.last_used_at,
            expires_at: t.expires_at,
            created_at: t.created_at,
        }
    }
}
