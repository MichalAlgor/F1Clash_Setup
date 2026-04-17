use axum::extract::FromRequestParts;
use axum::http::header;
use axum::http::request::Parts;
use std::convert::Infallible;

use crate::AppState;

/// Whether auth is enabled and whether the current request is authenticated.
#[derive(Clone)]
pub struct AuthStatus {
    pub enabled: bool,
    pub logged_in: bool,
}

impl FromRequestParts<AppState> for AuthStatus {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let enabled = state.session_token.is_some();
        if !enabled {
            return Ok(AuthStatus {
                enabled: false,
                logged_in: false,
            });
        }
        let expected = state.session_token.as_deref().unwrap_or("");
        let logged_in = get_cookie(&parts.headers, "admin_session")
            .map(|v| v == expected)
            .unwrap_or(false);
        Ok(AuthStatus { enabled, logged_in })
    }
}

pub fn get_cookie(headers: &axum::http::HeaderMap, name: &str) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    let prefix = format!("{name}=");
    cookie_header
        .split("; ")
        .find(|s| s.starts_with(&prefix))
        .map(|s| s[prefix.len()..].to_string())
}
