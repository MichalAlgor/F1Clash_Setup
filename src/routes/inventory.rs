use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{delete, get, post};
use axum::{Form, Router};
use maud::html;
use serde::Deserialize;

use crate::auth::AuthStatus;
use crate::AppState;

use crate::models::setup::InventoryItem;
use crate::templates;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/inventory", get(list))
        .route("/inventory/bulk", get(bulk_form).post(bulk_save))
        .route("/inventory/{id}", delete(destroy))
        .route("/inventory/{id}/level", post(update_level))
}

async fn index() -> impl IntoResponse {
    Redirect::to("/inventory")
}

async fn list(State(state): State<AppState>, auth: AuthStatus) -> impl IntoResponse {
    let season = state.season().await;
    let catalog = state.catalog_for_season().await;
    let items = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory WHERE season = $1 ORDER BY part_name",
    )
    .bind(&season)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    templates::inventory::list_page(&items, &catalog, &auth)
}

async fn bulk_form(State(state): State<AppState>, auth: AuthStatus) -> impl IntoResponse {
    let season = state.season().await;
    let catalog = state.catalog_for_season().await;
    let items = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory WHERE season = $1 ORDER BY part_name",
    )
    .bind(&season)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    templates::inventory::bulk_page(&items, &catalog, &auth)
}

async fn bulk_save(
    State(state): State<AppState>,
    Form(form): Form<Vec<(String, String)>>,
) -> impl IntoResponse {
    let season = state.season().await;

    sqlx::query("DELETE FROM inventory WHERE season = $1")
        .bind(&season)
        .execute(&state.pool)
        .await
        .unwrap();

    for (key, value) in &form {
        let Some(part_name) = key.strip_prefix("part:") else {
            continue;
        };
        let level: i32 = value.parse().unwrap_or(0);
        if level < 1 {
            continue;
        }
        if state.find_part(part_name).await.is_none() {
            continue;
        }
        sqlx::query("INSERT INTO inventory (part_name, level, season) VALUES ($1, $2, $3)")
            .bind(part_name)
            .bind(level)
            .bind(&season)
            .execute(&state.pool)
            .await
            .unwrap();
    }

    Redirect::to("/inventory")
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
    sqlx::query("UPDATE inventory SET level = $1 WHERE id = $2")
        .bind(form.level)
        .bind(id)
        .execute(&state.pool)
        .await
        .unwrap();

    Redirect::to("/inventory")
}

async fn destroy(State(state): State<AppState>, Path(id): Path<i32>) -> impl IntoResponse {
    sqlx::query("DELETE FROM inventory WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .unwrap();

    html! {}
}
