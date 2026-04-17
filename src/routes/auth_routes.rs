use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Redirect};
use axum::routing::post;
use axum::{Form, Router};
use serde::Deserialize;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub password: String,
}

async fn login(State(state): State<AppState>, Form(form): Form<LoginForm>) -> impl IntoResponse {
    if let (Some(token), Some(password)) = (&state.session_token, &state.admin_password)
        && form.password == *password
    {
        let cookie = format!("admin_session={token}; HttpOnly; Path=/; SameSite=Lax");
        return ([(header::SET_COOKIE, cookie)], Redirect::to("/admin/parts")).into_response();
    }
    Redirect::to("/").into_response()
}

async fn logout() -> impl IntoResponse {
    let clear = "admin_session=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0";
    ([(header::SET_COOKIE, clear.to_string())], Redirect::to("/"))
}
