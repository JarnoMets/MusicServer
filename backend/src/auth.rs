use actix_web::http::header::HeaderMap;
use hex::FromHex;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// Check the provided token (raw) by hashing with SHA256 and comparing to ADMIN_TOKEN_SHA256 env var.
pub fn verify_admin_token(raw_token: &str) -> bool {
    verify_static_token(raw_token, "ADMIN_TOKEN_SHA256")
}

/// Check the provided token (raw) by hashing with SHA256 and comparing to API_TOKEN_SHA256 env var.
pub fn verify_api_token(raw_token: &str) -> bool {
    verify_static_token(raw_token, "API_TOKEN_SHA256")
}

fn verify_static_token(raw_token: &str, env_var: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    let result = hasher.finalize();
    let expected_hex = match std::env::var(env_var) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let expected_bytes = match Vec::from_hex(expected_hex.trim()) {
        Ok(b) => b,
        Err(_) => return false,
    };
    // constant time compare
    if expected_bytes.len() != result.len() {
        return false;
    }
    expected_bytes.ct_eq(result.as_ref()).into()
}

/// Extract token from `x-admin-token` header and verify.
#[allow(dead_code)]
pub fn check_headers_for_admin(headers: &HeaderMap) -> bool {
    if let Some(v) = headers.get("x-admin-token") {
        if let Ok(s) = v.to_str() {
            return verify_admin_token(s);
        }
    }
    false
}
