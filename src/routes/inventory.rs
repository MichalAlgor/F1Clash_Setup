use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{delete, get, post};
use axum::{Form, Router};
use maud::html;
use serde::Deserialize;
use sqlx::PgPool;

use crate::data;
use crate::models::setup::InventoryItem;
use crate::templates;

pub fn router() -> Router<PgPool> {
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

async fn list(State(pool): State<PgPool>) -> impl IntoResponse {
    let items = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory ORDER BY part_name",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    templates::inventory::list_page(&items)
}

async fn bulk_form(State(pool): State<PgPool>) -> impl IntoResponse {
    let items = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory ORDER BY part_name",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    templates::inventory::bulk_page(&items)
}

/// Each part is submitted as `part:<name>=<level>` (0 means not owned)
async fn bulk_save(
    State(pool): State<PgPool>,
    Form(form): Form<Vec<(String, String)>>,
) -> impl IntoResponse {
    // Clear existing inventory and re-insert
    sqlx::query("DELETE FROM inventory")
        .execute(&pool)
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
        // Verify the part exists in catalog
        if data::find_part(part_name).is_none() {
            continue;
        }
        sqlx::query("INSERT INTO inventory (part_name, level) VALUES ($1, $2)")
            .bind(part_name)
            .bind(level)
            .execute(&pool)
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
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Form(form): Form<LevelForm>,
) -> impl IntoResponse {
    sqlx::query("UPDATE inventory SET level = $1 WHERE id = $2")
        .bind(form.level)
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    Redirect::to("/inventory")
}

async fn destroy(State(pool): State<PgPool>, Path(id): Path<i32>) -> impl IntoResponse {
    sqlx::query("DELETE FROM inventory WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    html! {}
}
