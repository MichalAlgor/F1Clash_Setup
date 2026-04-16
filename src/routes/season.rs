use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Form, Router};
use maud::html;
use serde::Deserialize;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/season", post(switch))
        .route("/api/season", get(api_season))
        .route("/api/season-selector", get(season_selector))
}

async fn api_season(State(state): State<AppState>) -> impl IntoResponse {
    state.season().await
}

/// Returns an inline form with a season <select> for the nav bar.
async fn season_selector(State(state): State<AppState>) -> impl IntoResponse {
    let current = state.season().await;

    let seasons: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT s FROM (
            SELECT season AS s FROM inventory
            UNION SELECT season AS s FROM driver_inventory
            UNION SELECT season AS s FROM driver_catalog
            UNION SELECT season AS s FROM setups
            UNION SELECT value AS s FROM settings WHERE key = 'active_season'
            UNION SELECT season AS s FROM season_categories
        ) AS all_seasons ORDER BY s",
    )
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

async fn switch(State(state): State<AppState>, Form(form): Form<SeasonForm>) -> impl IntoResponse {
    let target = form.season.trim().to_string();
    if target.is_empty() {
        return Redirect::to("/");
    }

    sqlx::query("UPDATE settings SET value = $1 WHERE key = 'active_season'")
        .bind(&target)
        .execute(&state.pool)
        .await
        .unwrap();

    *state.active_season.write().await = target;

    Redirect::to("/")
}
