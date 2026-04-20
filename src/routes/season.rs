use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Form, Router};
use maud::html;
use serde::Deserialize;

use crate::AppState;
use crate::error::AppError;
use crate::get_session_season;
use crate::session::UserSession;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/season", post(switch))
        .route("/api/season", get(api_season))
        .route("/api/season-selector", get(season_selector))
}

async fn api_season(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
) -> impl IntoResponse {
    get_session_season(&state.pool, &session_id).await
}

async fn season_selector(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
) -> impl IntoResponse {
    let current = get_session_season(&state.pool, &session_id).await;

    let seasons: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT s FROM (
            SELECT season AS s FROM inventory        WHERE session_id = $1
            UNION SELECT season AS s FROM driver_inventory WHERE session_id = $1
            UNION SELECT season AS s FROM setups           WHERE session_id = $1
            UNION SELECT value  AS s FROM settings         WHERE key = 'active_season' AND session_id = $1
            UNION SELECT season AS s FROM driver_catalog
            UNION SELECT season AS s FROM season_categories
        ) AS all_seasons ORDER BY s",
    )
    .bind(&session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    html! {
        form method="post" action="/season" style="display:inline;margin:0" {
            select name="season" onchange="this.form.submit()" class="inline-select season-select" {
                @for s in &seasons {
                    option value=(s) selected[*s == current] { (s) }
                }
            }
        }
    }
}

#[derive(Deserialize)]
pub struct SeasonForm {
    pub season: String,
}

async fn switch(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Form(form): Form<SeasonForm>,
) -> Result<impl IntoResponse, AppError> {
    let target = form.season.trim().to_string();
    if target.is_empty() {
        return Ok(Redirect::to("/"));
    }

    let current = crate::get_session_season(&state.pool, &session_id).await;
    crate::analytics::fire(
        &state.analytics,
        session_id.clone(),
        "season_switch",
        serde_json::json!({ "from": current, "to": target }),
    );

    sqlx::query(
        "INSERT INTO settings (key, value, session_id) VALUES ('active_season', $1, $2)
         ON CONFLICT (key, session_id) DO UPDATE SET value = EXCLUDED.value",
    )
    .bind(&target)
    .bind(&session_id)
    .execute(&state.pool)
    .await?;

    Ok(Redirect::to("/"))
}
