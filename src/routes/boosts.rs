use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::Form;
use axum::Router;

use crate::auth::AuthStatus;
use crate::models::driver::DriverBoost;
use crate::models::setup::Boost;
use crate::templates;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/boosts", get(show).post(save))
}

async fn show(State(state): State<AppState>, auth: AuthStatus) -> impl IntoResponse {
    let season = state.season().await;
    let catalog = state.catalog_for_season().await;

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

    let drivers_catalog = state.drivers_catalog_for_season().await;
    templates::boosts::page(&part_boosts, &driver_boosts, &catalog, &drivers_catalog, &auth)
}

async fn save(
    State(state): State<AppState>,
    Form(form): Form<Vec<(String, String)>>,
) -> impl IntoResponse {
    let season = state.season().await;

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
            if state.find_part(part_name).await.is_none() { continue; }
            sqlx::query("INSERT INTO boosts (part_name, percentage, season) VALUES ($1, $2, $3)")
                .bind(part_name)
                .bind(percentage)
                .bind(&season)
                .execute(&state.pool)
                .await
                .unwrap();
        } else if let Some(rest) = key.strip_prefix("driver:") {
            let Some((name, rarity_str)) = rest.rsplit_once(':') else { continue };
            if state.find_driver_def(name, rarity_str).await.is_none() { continue; }
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
