use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::{Form, Router};
use maud::html;
use serde::Deserialize;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/season", get(show).post(switch))
        .route("/api/season", get(api_season))
}

async fn api_season(State(state): State<AppState>) -> impl IntoResponse {
    state.season().await
}

async fn show(State(state): State<AppState>) -> impl IntoResponse {
    let current = state.season().await;

    // Get all known seasons
    let seasons: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT s FROM (
            SELECT season AS s FROM inventory
            UNION SELECT season AS s FROM driver_inventory
            UNION SELECT season AS s FROM setups
            UNION SELECT value AS s FROM settings WHERE key = 'active_season'
        ) AS all_seasons ORDER BY s",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    crate::templates::layout::page(
        "Season",
        html! {
            hgroup {
                h1 { "Season" }
                p { "Switch between seasons or create a new one" }
            }

            p { "Active season: " strong { (&current) } }

            form method="post" action="/season" {
                label for="season" { "Switch to season" }
                select id="season" name="season" {
                    @for s in &seasons {
                        option value=(s) selected[*s == current] { (s) }
                    }
                }

                label for="new_season" { "Or create new season" }
                input type="text" id="new_season" name="new_season" placeholder="e.g. 2026";

                button type="submit" { "Switch Season" }
            }
        },
    )
}

#[derive(Deserialize)]
pub struct SeasonForm {
    pub season: String,
    #[serde(default)]
    pub new_season: String,
}

async fn switch(State(state): State<AppState>, Form(form): Form<SeasonForm>) -> impl IntoResponse {
    let target = if form.new_season.trim().is_empty() {
        form.season.clone()
    } else {
        form.new_season.trim().to_string()
    };

    // Update DB setting
    sqlx::query("UPDATE settings SET value = $1 WHERE key = 'active_season'")
        .bind(&target)
        .execute(&state.pool)
        .await
        .unwrap();

    // Update in-memory state
    *state.active_season.write().await = target;

    Redirect::to("/season")
}
