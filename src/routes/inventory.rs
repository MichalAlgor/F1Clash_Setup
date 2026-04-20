use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{delete, get, post};
use axum::{Form, Router};
use maud::html;
use serde::Deserialize;

use crate::AppState;
use crate::auth::AuthStatus;
use crate::error::AppError;
use crate::get_session_season;
use crate::session::UserSession;

use crate::models::setup::InventoryItem;
use crate::templates;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/inventory", get(list))
        .route("/inventory/bulk", get(bulk_form).post(bulk_save))
        .route("/inventory/{id}", delete(destroy))
        .route("/inventory/{id}/level", post(update_level))
        .route("/inventory/{id}/cards", post(update_cards))
}

async fn index() -> impl IntoResponse {
    Redirect::to("/inventory")
}

async fn list(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
) -> impl IntoResponse {
    let season = get_session_season(&state.pool, &session_id).await;
    let catalog = state.catalog_for_season(&season).await;
    let categories = state.categories_for_season(&season).await;
    let items = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory WHERE season = $1 AND session_id = $2 ORDER BY part_name",
    )
    .bind(&season)
    .bind(&session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    templates::inventory::list_page(&items, &catalog, &categories, &auth)
}

async fn bulk_form(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
) -> impl IntoResponse {
    let season = get_session_season(&state.pool, &session_id).await;
    let catalog = state.catalog_for_season(&season).await;
    let categories = state.categories_for_season(&season).await;
    let items = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory WHERE season = $1 AND session_id = $2 ORDER BY part_name",
    )
    .bind(&season)
    .bind(&session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    templates::inventory::bulk_page(&items, &catalog, &categories, &auth)
}

async fn bulk_save(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Form(form): Form<Vec<(String, String)>>,
) -> Result<impl IntoResponse, AppError> {
    let season = get_session_season(&state.pool, &session_id).await;
    let parts_count = form.iter().filter(|(k, _)| k.starts_with("part:")).count();
    let bucket = match parts_count {
        0 => "0",
        1..=7 => "1-7",
        8..=14 => "8-14",
        _ => "15+",
    };
    crate::analytics::fire(
        &state.analytics,
        session_id.clone(),
        "bulk_inventory_save",
        serde_json::json!({ "season": season, "parts_count_bucket": bucket }),
    );

    // NULL out setup references before deleting to avoid FK violations
    sqlx::query(
        "UPDATE setups SET engine_id=NULL, front_wing_id=NULL, rear_wing_id=NULL, \
         suspension_id=NULL, brakes_id=NULL, gearbox_id=NULL, battery_id=NULL \
         WHERE engine_id      IN (SELECT id FROM inventory WHERE session_id=$1) \
            OR front_wing_id  IN (SELECT id FROM inventory WHERE session_id=$1) \
            OR rear_wing_id   IN (SELECT id FROM inventory WHERE session_id=$1) \
            OR suspension_id  IN (SELECT id FROM inventory WHERE session_id=$1) \
            OR brakes_id      IN (SELECT id FROM inventory WHERE session_id=$1) \
            OR gearbox_id     IN (SELECT id FROM inventory WHERE session_id=$1) \
            OR battery_id     IN (SELECT id FROM inventory WHERE session_id=$1)",
    )
    .bind(&session_id)
    .execute(&state.pool)
    .await?;

    sqlx::query("DELETE FROM inventory WHERE season = $1 AND session_id = $2")
        .bind(&season)
        .bind(&session_id)
        .execute(&state.pool)
        .await?;

    for (key, value) in &form {
        let Some(part_name) = key.strip_prefix("part:") else {
            continue;
        };
        let level: i32 = value.parse().unwrap_or(0);
        if level < 1 {
            continue;
        }
        if state.find_part(part_name, &season).await.is_none() {
            continue;
        }

        sqlx::query(
            "INSERT INTO inventory (part_name, level, season, session_id) VALUES ($1, $2, $3, $4)",
        )
        .bind(part_name)
        .bind(level)
        .bind(&season)
        .bind(&session_id)
        .execute(&state.pool)
        .await?;
    }

    Ok(Redirect::to("/inventory"))
}

#[derive(Deserialize)]
pub struct LevelForm {
    pub level: i32,
}

async fn update_level(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Path(id): Path<i32>,
    Form(form): Form<LevelForm>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query("UPDATE inventory SET level = $1 WHERE id = $2 AND session_id = $3")
        .bind(form.level)
        .bind(id)
        .bind(&session_id)
        .execute(&state.pool)
        .await?;

    Ok(Redirect::to("/inventory"))
}

#[derive(Deserialize)]
pub struct CardsForm {
    pub cards: i32,
}

async fn update_cards(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Path(id): Path<i32>,
    Form(form): Form<CardsForm>,
) -> Result<impl IntoResponse, AppError> {
    let cards = form.cards.max(0);

    sqlx::query("UPDATE inventory SET cards_owned = $1 WHERE id = $2 AND session_id = $3")
        .bind(cards)
        .bind(id)
        .bind(&session_id)
        .execute(&state.pool)
        .await?;

    let item = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory WHERE id = $1 AND session_id = $2",
    )
    .bind(id)
    .bind(&session_id)
    .fetch_one(&state.pool)
    .await?;

    let season = get_session_season(&state.pool, &session_id).await;
    let catalog = state.catalog_for_season(&season).await;
    let part_def = catalog.iter().find(|p| p.name == item.part_name);

    Ok(templates::inventory::cards_cell(
        id, cards, item.level, part_def,
    ))
}

async fn destroy(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query("DELETE FROM inventory WHERE id = $1 AND session_id = $2")
        .bind(id)
        .bind(&session_id)
        .execute(&state.pool)
        .await?;

    Ok(html! {})
}
