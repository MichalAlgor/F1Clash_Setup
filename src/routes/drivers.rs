use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{delete, get, post};
use axum::{Form, Router};
use maud::html;
use serde::Deserialize;
use sqlx::PgPool;

use crate::drivers_data;
use crate::models::driver::DriverInventoryItem;
use crate::templates;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/drivers", get(list))
        .route("/drivers/bulk", get(bulk_form).post(bulk_save))
        .route("/drivers/{id}", delete(destroy))
        .route("/drivers/{id}/level", post(update_level))
}

async fn list(State(pool): State<PgPool>) -> impl IntoResponse {
    let items = sqlx::query_as::<_, DriverInventoryItem>(
        "SELECT * FROM driver_inventory ORDER BY driver_name",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    templates::drivers::list_page(&items)
}

async fn bulk_form(State(pool): State<PgPool>) -> impl IntoResponse {
    let items = sqlx::query_as::<_, DriverInventoryItem>(
        "SELECT * FROM driver_inventory ORDER BY driver_name",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    templates::drivers::bulk_page(&items)
}

async fn bulk_save(
    State(pool): State<PgPool>,
    Form(form): Form<Vec<(String, String)>>,
) -> impl IntoResponse {
    sqlx::query("DELETE FROM driver_inventory")
        .execute(&pool)
        .await
        .unwrap();

    for (key, value) in &form {
        // Keys are "driver:<Name>:<Rarity>"
        let Some(rest) = key.strip_prefix("driver:") else { continue };
        let Some((name, rarity_str)) = rest.rsplit_once(':') else { continue };
        let level: i32 = value.parse().unwrap_or(0);
        if level < 1 { continue; }
        if drivers_data::find_driver_by_db(name, rarity_str).is_none() { continue; }

        sqlx::query("INSERT INTO driver_inventory (driver_name, rarity, level) VALUES ($1, $2, $3)")
            .bind(name)
            .bind(rarity_str)
            .bind(level)
            .execute(&pool)
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
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Form(form): Form<LevelForm>,
) -> impl IntoResponse {
    sqlx::query("UPDATE driver_inventory SET level = $1 WHERE id = $2")
        .bind(form.level)
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    Redirect::to("/drivers")
}

async fn destroy(State(pool): State<PgPool>, Path(id): Path<i32>) -> impl IntoResponse {
    sqlx::query("DELETE FROM driver_inventory WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    html! {}
}
