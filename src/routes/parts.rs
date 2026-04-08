use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{delete, get, post};
use axum::{Form, Router};
use maud::html;
use serde::Deserialize;
use sqlx::PgPool;

use crate::models::part::{Part, PartCategory};
use crate::templates;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", get(index))
        .route("/parts", get(list).post(create))
        .route("/parts/new", get(new))
        .route("/parts/{id}", post(update))
        .route("/parts/{id}", delete(destroy))
        .route("/parts/{id}/edit", get(edit))
}

async fn index() -> impl IntoResponse {
    Redirect::to("/parts")
}

async fn list(State(pool): State<PgPool>) -> impl IntoResponse {
    let parts = sqlx::query_as::<_, Part>("SELECT * FROM parts ORDER BY category, name")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    templates::parts::list_page(&parts)
}

async fn new() -> impl IntoResponse {
    templates::parts::form_page(None)
}

#[derive(Deserialize)]
pub struct PartForm {
    pub name: String,
    pub category: PartCategory,
    pub level: i32,
    pub speed: i32,
    pub cornering: i32,
    pub power_unit: i32,
    pub qualifying: i32,
    pub pit_stop_time: f64,
}

async fn create(State(pool): State<PgPool>, Form(form): Form<PartForm>) -> impl IntoResponse {
    sqlx::query(
        "INSERT INTO parts (name, category, level, speed, cornering, power_unit, qualifying, pit_stop_time)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&form.name)
    .bind(&form.category)
    .bind(form.level)
    .bind(form.speed)
    .bind(form.cornering)
    .bind(form.power_unit)
    .bind(form.qualifying)
    .bind(form.pit_stop_time)
    .execute(&pool)
    .await
    .unwrap();

    Redirect::to("/parts")
}

async fn edit(State(pool): State<PgPool>, Path(id): Path<i32>) -> impl IntoResponse {
    let part = sqlx::query_as::<_, Part>("SELECT * FROM parts WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();

    templates::parts::form_page(Some(&part))
}

async fn update(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Form(form): Form<PartForm>,
) -> impl IntoResponse {
    sqlx::query(
        "UPDATE parts SET name=$1, category=$2, level=$3, speed=$4, cornering=$5, power_unit=$6, qualifying=$7, pit_stop_time=$8
         WHERE id=$9",
    )
    .bind(&form.name)
    .bind(&form.category)
    .bind(form.level)
    .bind(form.speed)
    .bind(form.cornering)
    .bind(form.power_unit)
    .bind(form.qualifying)
    .bind(form.pit_stop_time)
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();

    Redirect::to("/parts")
}

async fn destroy(State(pool): State<PgPool>, Path(id): Path<i32>) -> impl IntoResponse {
    sqlx::query("DELETE FROM parts WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    html! {}
}
