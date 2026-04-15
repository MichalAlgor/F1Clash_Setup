use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{delete, get};
use axum::{Form, Router};
use serde::Deserialize;
use std::collections::HashMap;

use crate::auth::AuthStatus;
use crate::catalog;
use crate::models::part::OwnedLevelStats;
use crate::templates;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/parts", get(list_parts).post(create_part))
        .route("/admin/parts/export", get(export_parts))
        .route("/admin/parts/new", get(new_part_form))
        .route("/admin/parts/{id}/edit", get(edit_part_form))
        .route("/admin/parts/{id}", delete(delete_part).post(update_part))
}

/// Returns a redirect if auth is required but the user isn't logged in.
fn guard(auth: &AuthStatus) -> Option<axum::response::Response> {
    if auth.enabled && !auth.logged_in {
        Some(Redirect::to("/").into_response())
    } else {
        None
    }
}

async fn list_parts(State(state): State<AppState>, auth: AuthStatus) -> impl IntoResponse {
    if let Some(r) = guard(&auth) { return r; }
    let season = state.season().await;
    let catalog = state.catalog_for_season().await;
    templates::admin::parts_list_page(&catalog, &season, &auth).into_response()
}

async fn new_part_form(State(state): State<AppState>, auth: AuthStatus) -> impl IntoResponse {
    if let Some(r) = guard(&auth) { return r; }
    let season = state.season().await;
    templates::admin::part_form_page(None, &season, &auth).into_response()
}

async fn edit_part_form(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    auth: AuthStatus,
) -> impl IntoResponse {
    if let Some(r) = guard(&auth) { return r; }
    let season = state.season().await;
    let catalog = state.catalog.read().await;
    let part = catalog.iter().find(|p| p.id == id).cloned();
    drop(catalog);
    match part {
        Some(p) => templates::admin::part_form_page(Some(&p), &season, &auth).into_response(),
        None => Redirect::to("/admin/parts").into_response(),
    }
}

#[derive(Deserialize)]
pub struct PartForm {
    pub name: String,
    pub category: String,
    pub series: i32,
    pub rarity: String,
    pub levels_json: String,
}

async fn create_part(
    State(state): State<AppState>,
    auth: AuthStatus,
    Form(form): Form<PartForm>,
) -> impl IntoResponse {
    if let Some(r) = guard(&auth) { return r; }
    let season = state.season().await;

    let levels: Vec<OwnedLevelStats> = match serde_json::from_str(&form.levels_json) {
        Ok(v) => v,
        Err(_) => return Redirect::to("/admin/parts/new").into_response(),
    };

    let sort_order: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM part_catalog WHERE season = $1",
    )
    .bind(&season)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let part_id: i32 = sqlx::query_scalar(
        "INSERT INTO part_catalog (name, season, category, series, rarity, sort_order)
         VALUES ($1, $2, $3::part_category, $4, $5, $6)
         RETURNING id",
    )
    .bind(&form.name)
    .bind(&season)
    .bind(&form.category)
    .bind(form.series)
    .bind(&form.rarity)
    .bind(sort_order)
    .fetch_one(&state.pool)
    .await
    .unwrap();

    insert_levels(&state, part_id, &levels).await;
    reload_catalog(&state).await;
    Redirect::to("/admin/parts").into_response()
}

async fn update_part(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    auth: AuthStatus,
    Form(form): Form<PartForm>,
) -> impl IntoResponse {
    if let Some(r) = guard(&auth) { return r; }

    let levels: Vec<OwnedLevelStats> = match serde_json::from_str(&form.levels_json) {
        Ok(v) => v,
        Err(_) => return Redirect::to("/admin/parts").into_response(),
    };

    sqlx::query(
        "UPDATE part_catalog SET name=$1, category=$2::part_category, series=$3, rarity=$4
         WHERE id=$5",
    )
    .bind(&form.name)
    .bind(&form.category)
    .bind(form.series)
    .bind(&form.rarity)
    .bind(id)
    .execute(&state.pool)
    .await
    .unwrap();

    sqlx::query("DELETE FROM part_level_stats WHERE part_id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .unwrap();

    insert_levels(&state, id, &levels).await;
    reload_catalog(&state).await;
    Redirect::to("/admin/parts").into_response()
}

async fn delete_part(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    auth: AuthStatus,
) -> impl IntoResponse {
    if auth.enabled && !auth.logged_in {
        return maud::html! {}.into_response();
    }

    sqlx::query("DELETE FROM part_catalog WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .unwrap();

    reload_catalog(&state).await;
    maud::html! {}.into_response()
}

async fn export_parts(State(state): State<AppState>, auth: AuthStatus) -> impl IntoResponse {
    if let Some(r) = guard(&auth) { return r.into_response(); }

    let all = state.catalog.read().await.clone();

    let mut by_season: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    for part in &all {
        by_season.entry(part.season.clone()).or_default().push(serde_json::json!({
            "name": part.name,
            "category": part.category.slug(),
            "series": part.series,
            "rarity": part.rarity,
            "sort_order": part.sort_order,
            "levels": part.levels,
        }));
    }

    let json = serde_json::to_string_pretty(&by_season).unwrap_or_default();
    (
        [
            (header::CONTENT_TYPE, "application/json".to_string()),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"parts.json\"".to_string()),
        ],
        json,
    )
        .into_response()
}

async fn insert_levels(state: &AppState, part_id: i32, levels: &[OwnedLevelStats]) {
    for lvl in levels {
        sqlx::query(
            "INSERT INTO part_level_stats
             (part_id, level, speed, cornering, power_unit, qualifying, pit_stop_time, drs)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(part_id)
        .bind(lvl.level)
        .bind(lvl.speed)
        .bind(lvl.cornering)
        .bind(lvl.power_unit)
        .bind(lvl.qualifying)
        .bind(lvl.pit_stop_time)
        .bind(lvl.drs)
        .execute(&state.pool)
        .await
        .unwrap();
    }
}

async fn reload_catalog(state: &AppState) {
    let new_catalog = catalog::load_catalog(&state.pool).await;
    *state.catalog.write().await = new_catalog;
}
