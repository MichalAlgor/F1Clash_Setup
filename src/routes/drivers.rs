use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{delete, get, post};
use axum::{Form, Router};
use maud::html;
use serde::Deserialize;

use crate::auth::AuthStatus;
use crate::get_session_season;
use crate::session::UserSession;
use crate::AppState;
use crate::models::driver::DriverInventoryItem;
use crate::templates;
use crate::templates::drivers::driver_cards_cell;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/drivers", get(list))
        .route("/drivers/bulk", get(bulk_form).post(bulk_save))
        .route("/drivers/{id}", delete(destroy))
        .route("/drivers/{id}/level", post(update_level))
        .route("/drivers/{id}/cards", post(update_cards))
}

async fn list(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
) -> impl IntoResponse {
    let season = get_session_season(&state.pool, &session_id).await;
    let items = sqlx::query_as::<_, DriverInventoryItem>(
        "SELECT * FROM driver_inventory WHERE season = $1 AND session_id = $2 ORDER BY driver_name",
    )
    .bind(&season)
    .bind(&session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let catalog = state.drivers_catalog_for_season(&season).await;
    templates::drivers::list_page(&items, &catalog, &auth)
}

async fn bulk_form(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
) -> impl IntoResponse {
    let season = get_session_season(&state.pool, &session_id).await;
    let items = sqlx::query_as::<_, DriverInventoryItem>(
        "SELECT * FROM driver_inventory WHERE season = $1 AND session_id = $2 ORDER BY driver_name",
    )
    .bind(&season)
    .bind(&session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let catalog = state.drivers_catalog_for_season(&season).await;
    templates::drivers::bulk_page(&items, &catalog, &auth)
}

async fn bulk_save(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Form(form): Form<Vec<(String, String)>>,
) -> impl IntoResponse {
    let season = get_session_season(&state.pool, &session_id).await;

    sqlx::query(
        "UPDATE setups SET driver1_id = NULL WHERE driver1_id IN \
         (SELECT id FROM driver_inventory WHERE season = $1 AND session_id = $2)",
    )
    .bind(&season).bind(&session_id).execute(&state.pool).await.unwrap();

    sqlx::query(
        "UPDATE setups SET driver2_id = NULL WHERE driver2_id IN \
         (SELECT id FROM driver_inventory WHERE season = $1 AND session_id = $2)",
    )
    .bind(&season).bind(&session_id).execute(&state.pool).await.unwrap();

    sqlx::query("DELETE FROM driver_inventory WHERE season = $1 AND session_id = $2")
        .bind(&season)
        .bind(&session_id)
        .execute(&state.pool)
        .await
        .unwrap();

    for (key, value) in &form {
        let Some(rest) = key.strip_prefix("driver:") else { continue };
        let Some((name, rarity_str)) = rest.rsplit_once(':') else { continue };
        let level: i32 = value.parse().unwrap_or(0);
        if level < 1 { continue; }
        if state.find_driver_def(name, rarity_str, &season).await.is_none() { continue; }

        sqlx::query(
            "INSERT INTO driver_inventory (driver_name, rarity, level, season, session_id) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(name)
        .bind(rarity_str)
        .bind(level)
        .bind(&season)
        .bind(&session_id)
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
    UserSession(session_id): UserSession,
    Path(id): Path<i32>,
    Form(form): Form<LevelForm>,
) -> impl IntoResponse {
    sqlx::query("UPDATE driver_inventory SET level = $1 WHERE id = $2 AND session_id = $3")
        .bind(form.level)
        .bind(id)
        .bind(&session_id)
        .execute(&state.pool)
        .await
        .unwrap();

    Redirect::to("/drivers")
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
) -> impl IntoResponse {
    let cards = form.cards.max(0);

    sqlx::query("UPDATE driver_inventory SET cards_owned = $1 WHERE id = $2 AND session_id = $3")
        .bind(cards)
        .bind(id)
        .bind(&session_id)
        .execute(&state.pool)
        .await
        .unwrap();

    let item = sqlx::query_as::<_, DriverInventoryItem>(
        "SELECT * FROM driver_inventory WHERE id = $1 AND session_id = $2",
    )
    .bind(id)
    .bind(&session_id)
    .fetch_one(&state.pool)
    .await
    .unwrap();

    let season = get_session_season(&state.pool, &session_id).await;
    let def = state.find_driver_def(&item.driver_name, &item.rarity, &season).await;
    driver_cards_cell(id, cards, item.level, def.as_ref())
}

async fn destroy(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    sqlx::query("UPDATE setups SET driver1_id = NULL WHERE driver1_id = $1 AND session_id = $2")
        .bind(id).bind(&session_id).execute(&state.pool).await.unwrap();
    sqlx::query("UPDATE setups SET driver2_id = NULL WHERE driver2_id = $1 AND session_id = $2")
        .bind(id).bind(&session_id).execute(&state.pool).await.unwrap();
    sqlx::query("DELETE FROM driver_inventory WHERE id = $1 AND session_id = $2")
        .bind(id)
        .bind(&session_id)
        .execute(&state.pool)
        .await
        .unwrap();

    html! {}
}
