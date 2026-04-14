use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{delete, get, post};
use axum::{Form, Router};
use maud::html;
use serde::Deserialize;

use crate::AppState;
use crate::drivers_data;
use crate::models::driver::DriverInventoryItem;
use crate::templates;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/drivers", get(list))
        .route("/drivers/bulk", get(bulk_form).post(bulk_save))
        .route("/drivers/{id}", delete(destroy))
        .route("/drivers/{id}/level", post(update_level))
}

async fn list(State(state): State<AppState>) -> impl IntoResponse {
    let season = state.season().await;
    let items = sqlx::query_as::<_, DriverInventoryItem>(
        "SELECT * FROM driver_inventory WHERE season = $1 ORDER BY driver_name",
    )
    .bind(&season)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    templates::drivers::list_page(&items)
}

async fn bulk_form(State(state): State<AppState>) -> impl IntoResponse {
    let season = state.season().await;
    let items = sqlx::query_as::<_, DriverInventoryItem>(
        "SELECT * FROM driver_inventory WHERE season = $1 ORDER BY driver_name",
    )
    .bind(&season)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    templates::drivers::bulk_page(&items)
}

async fn bulk_save(
    State(state): State<AppState>,
    Form(form): Form<Vec<(String, String)>>,
) -> impl IntoResponse {
    let season = state.season().await;

    sqlx::query("UPDATE setups SET driver1_id = NULL WHERE driver1_id IN (SELECT id FROM driver_inventory WHERE season = $1)")
        .bind(&season).execute(&state.pool).await.unwrap();
    sqlx::query("UPDATE setups SET driver2_id = NULL WHERE driver2_id IN (SELECT id FROM driver_inventory WHERE season = $1)")
        .bind(&season).execute(&state.pool).await.unwrap();
    sqlx::query("DELETE FROM driver_inventory WHERE season = $1")
        .bind(&season)
        .execute(&state.pool)
        .await
        .unwrap();

    for (key, value) in &form {
        // Keys are "driver:<Name>:<Rarity>"
        let Some(rest) = key.strip_prefix("driver:") else { continue };
        let Some((name, rarity_str)) = rest.rsplit_once(':') else { continue };
        let level: i32 = value.parse().unwrap_or(0);
        if level < 1 { continue; }
        if drivers_data::find_driver_by_db(name, rarity_str).is_none() { continue; }

        sqlx::query("INSERT INTO driver_inventory (driver_name, rarity, level, season) VALUES ($1, $2, $3, $4)")
            .bind(name)
            .bind(rarity_str)
            .bind(level)
            .bind(&season)
            .execute(&state.pool)
            .await
            .unwrap();
    }

    Redirect::to("/drivers")
}

#[derive(Deserialize)]
pub struct LevelForm {
    pub level: i32,
}

async fn update_level(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Form(form): Form<LevelForm>,
) -> impl IntoResponse {
    sqlx::query("UPDATE driver_inventory SET level = $1 WHERE id = $2")
        .bind(form.level)
        .bind(id)
        .execute(&state.pool)
        .await
        .unwrap();

    Redirect::to("/drivers")
}

async fn destroy(State(state): State<AppState>, Path(id): Path<i32>) -> impl IntoResponse {
    sqlx::query("UPDATE setups SET driver1_id = NULL WHERE driver1_id = $1")
        .bind(id).execute(&state.pool).await.unwrap();
    sqlx::query("UPDATE setups SET driver2_id = NULL WHERE driver2_id = $1")
        .bind(id).execute(&state.pool).await.unwrap();
    sqlx::query("DELETE FROM driver_inventory WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .unwrap();

    html! {}
}
