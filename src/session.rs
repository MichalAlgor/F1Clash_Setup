use axum::extract::FromRequestParts;
use axum::http::header;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::extract::Request;
use axum::response::Response;
use sha2::{Sha256, Digest};
use std::convert::Infallible;

use crate::auth::get_cookie;

/// Hash a raw token for safe DB storage.
/// Input:  raw UUID string from the cookie
/// Output: lowercase hex string of SHA-256 digest (64 chars)
pub fn hash_token(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hex::encode(hasher.finalize())
}

/// The hashed session ID for the current request.
/// Value is SHA-256(raw_uuid) — safe to use directly in DB queries.
/// The raw UUID is only ever held in the cookie; handlers never see it.
#[derive(Debug, Clone)]
pub struct UserSession(pub String);

impl<S: Send + Sync> FromRequestParts<S> for UserSession {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Infallible> {
        let session = parts
            .extensions
            .get::<UserSession>()
            .cloned()
            .unwrap_or_else(|| UserSession(hash_token(&uuid::Uuid::new_v4().to_string())));
        Ok(session)
    }
}

/// Middleware that reads the `user_session` cookie.
/// - If present: hashes the raw value and inserts `UserSession(hash)` as a request extension.
/// - If absent: generates a new UUID, sets the cookie on the response, inserts the hash.
///
/// Handlers always receive the hash via `UserSession` — never the raw token.
pub async fn session_middleware(mut req: Request, next: Next) -> Response {
    let raw_token = get_cookie(req.headers(), "user_session");
    let is_new = raw_token.is_none();
    let raw_token = raw_token
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let session_id = hash_token(&raw_token);
    req.extensions_mut().insert(UserSession(session_id));

    let mut response = next.run(req).await;

    if is_new {
        let cookie = format!(
            "user_session={raw_token}; HttpOnly; Path=/; SameSite=Lax; Max-Age=31536000"
        );
        response
            .headers_mut()
            .append(header::SET_COOKIE, cookie.parse().unwrap());
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_token_is_64_char_hex() {
        let h = hash_token("550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hash_token_is_deterministic() {
        let a = hash_token("test-token");
        let b = hash_token("test-token");
        assert_eq!(a, b);
    }

    #[test]
    fn hash_token_different_inputs_differ() {
        let a = hash_token("token-a");
        let b = hash_token("token-b");
        assert_ne!(a, b);
    }
}
