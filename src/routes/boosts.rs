use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::Form;
use axum::Router;

use crate::data;
use crate::drivers_data;
use crate::models::driver::DriverBoost;
use crate::models::setup::Boost;
use crate::templates;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/boosts", get(show).post(save))
}

async fn show(State(state): State<AppState>) -> impl IntoResponse {
    let season = state.season().await;

    let part_boosts = sqlx::query_as::<_, Boost>(
        "SELECT * FROM boosts WHERE season = $1 ORDER BY part_name",
    )
    .bind(&season)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let driver_boosts = sqlx::query_as::<_, DriverBoost>(
        "SELECT * FROM driver_boosts WHERE season = $1 ORDER BY driver_name",
    )
    .bind(&season)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    templates::boosts::page(&part_boosts, &driver_boosts)
}

/// Single save handler — form fields prefixed with `part:` or `driver:`
async fn save(
    State(state): State<AppState>,
    Form(form): Form<Vec<(String, String)>>,
) -> impl IntoResponse {
    let season = state.season().await;

    // Clear both tables
    sqlx::query("DELETE FROM boosts WHERE season = $1")
        .bind(&season)
        .execute(&state.pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM driver_boosts WHERE season = $1")
        .bind(&season)
        .execute(&state.pool)
        .await
        .unwrap();

    for (key, value) in &form {
        let percentage: i32 = value.parse().unwrap_or(0);
        if percentage == 0 {
            continue;
        }

        if let Some(part_name) = key.strip_prefix("part:") {
            if data::find_part(part_name).is_none() { continue; }
            sqlx::query("INSERT INTO boosts (part_name, percentage, season) VALUES ($1, $2, $3)")
                .bind(part_name)
                .bind(percentage)
                .bind(&season)
                .execute(&state.pool)
                .await
                .unwrap();
        } else if let Some(rest) = key.strip_prefix("driver:") {
            let Some((name, rarity_str)) = rest.rsplit_once(':') else { continue };
            if drivers_data::find_driver_by_db(name, rarity_str).is_none() { continue; }
            sqlx::query("INSERT INTO driver_boosts (driver_name, rarity, percentage, season) VALUES ($1, $2, $3, $4)")
                .bind(name)
                .bind(rarity_str)
                .bind(percentage)
                .bind(&season)
                .execute(&state.pool)
                .await
                .unwrap();
        }
    }

    Redirect::to("/boosts")
}
