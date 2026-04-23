use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;
use std::collections::HashMap;

use crate::AppState;
use crate::auth::AuthStatus;
use crate::catalog;
use crate::error::AppError;
use crate::get_session_season;
use crate::models::driver::OwnedDriverLevelStats;
use crate::models::part::OwnedLevelStats;
use crate::session::UserSession;
use crate::templates;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/parts", get(list_parts).post(create_part))
        .route("/admin/parts/export", get(export_parts))
        .route("/admin/parts/new", get(new_part_form))
        .route("/admin/parts/{id}/edit", get(edit_part_form))
        .route("/admin/parts/{id}", post(update_part))
        .route("/admin/parts/{id}/delete", post(delete_part))
        .route("/admin/drivers", get(list_drivers).post(create_driver))
        .route("/admin/drivers/export", get(export_drivers))
        .route("/admin/drivers/new", get(new_driver_form))
        .route("/admin/drivers/{id}/edit", get(edit_driver_form))
        .route("/admin/drivers/{id}", post(update_driver))
        .route("/admin/drivers/{id}/delete", post(delete_driver))
        .route(
            "/admin/seasons",
            get(list_seasons).post(save_season_categories),
        )
}

/// Returns a redirect if auth is required but the user isn't logged in.
fn guard(auth: &AuthStatus) -> Option<Response> {
    if auth.enabled && !auth.logged_in {
        Some(Redirect::to("/").into_response())
    } else {
        None
    }
}

async fn list_parts(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
) -> impl IntoResponse {
    if let Some(r) = guard(&auth) {
        return r;
    }
    let season = get_session_season(&state.pool, &session_id).await;
    let catalog = state.catalog_for_season(&season).await;
    templates::admin::parts_list_page(&catalog, &season, &auth).into_response()
}

async fn new_part_form(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
) -> impl IntoResponse {
    if let Some(r) = guard(&auth) {
        return r;
    }
    let season = get_session_season(&state.pool, &session_id).await;
    templates::admin::part_form_page(None, &season, &auth).into_response()
}

async fn edit_part_form(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Path(id): Path<i32>,
    auth: AuthStatus,
) -> impl IntoResponse {
    if let Some(r) = guard(&auth) {
        return r;
    }
    let season = get_session_season(&state.pool, &session_id).await;
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
    #[serde(default)]
    pub additional_stat_name: Option<String>,
    pub levels_json: String,
}

async fn create_part(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
    Form(form): Form<PartForm>,
) -> Result<Response, AppError> {
    if let Some(r) = guard(&auth) {
        return Ok(r);
    }
    let season = get_session_season(&state.pool, &session_id).await;

    let levels: Vec<OwnedLevelStats> = match serde_json::from_str(&form.levels_json) {
        Ok(v) => v,
        Err(_) => return Ok(Redirect::to("/admin/parts/new").into_response()),
    };

    let additional_stat_name = form
        .additional_stat_name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string());

    let sort_order: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM part_catalog WHERE season = $1",
    )
    .bind(&season)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let part_id: i32 = sqlx::query_scalar(
        "INSERT INTO part_catalog (name, season, category, series, rarity, sort_order, additional_stat_name)
         VALUES ($1, $2, $3::part_category, $4, $5, $6, $7)
         RETURNING id",
    )
    .bind(&form.name)
    .bind(&season)
    .bind(&form.category)
    .bind(form.series)
    .bind(&form.rarity)
    .bind(sort_order)
    .bind(&additional_stat_name)
    .fetch_one(&state.pool)
    .await?;

    insert_levels(&state, part_id, &levels).await?;
    reload_catalog(&state).await;
    Ok(Redirect::to("/admin/parts").into_response())
}

async fn update_part(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    auth: AuthStatus,
    Form(form): Form<PartForm>,
) -> Result<Response, AppError> {
    if let Some(r) = guard(&auth) {
        return Ok(r);
    }

    let levels: Vec<OwnedLevelStats> = match serde_json::from_str(&form.levels_json) {
        Ok(v) => v,
        Err(_) => return Ok(Redirect::to("/admin/parts").into_response()),
    };

    let additional_stat_name = form
        .additional_stat_name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string());

    sqlx::query(
        "UPDATE part_catalog SET name=$1, category=$2::part_category, series=$3, rarity=$4, additional_stat_name=$5
         WHERE id=$6",
    )
    .bind(&form.name)
    .bind(&form.category)
    .bind(form.series)
    .bind(&form.rarity)
    .bind(&additional_stat_name)
    .bind(id)
    .execute(&state.pool)
    .await?;

    sqlx::query("DELETE FROM part_level_stats WHERE part_id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    insert_levels(&state, id, &levels).await?;
    reload_catalog(&state).await;
    Ok(Redirect::to("/admin/parts").into_response())
}

async fn delete_part(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    auth: AuthStatus,
) -> Result<Response, AppError> {
    if auth.enabled && !auth.logged_in {
        return Ok(maud::html! {}.into_response());
    }

    sqlx::query("DELETE FROM part_catalog WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    reload_catalog(&state).await;
    Ok(maud::html! {}.into_response())
}

async fn export_parts(State(state): State<AppState>, auth: AuthStatus) -> impl IntoResponse {
    if let Some(r) = guard(&auth) {
        return r.into_response();
    }

    let all = state.catalog.read().await.clone();

    let mut by_season: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    for part in &all {
        by_season
            .entry(part.season.clone())
            .or_default()
            .push(serde_json::json!({
                "name": part.name,
                "category": part.category.slug(),
                "series": part.series,
                "rarity": part.rarity,
                "sort_order": part.sort_order,
                "additional_stat_name": part.additional_stat_name,
                "levels": part.levels,
            }));
    }

    let json = serde_json::to_string_pretty(&by_season).unwrap_or_default();
    (
        [
            (header::CONTENT_TYPE, "application/json".to_string()),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"parts.json\"".to_string(),
            ),
        ],
        json,
    )
        .into_response()
}

// ── Driver catalog admin ─────────────────────────────────────────────────────

async fn list_drivers(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
) -> impl IntoResponse {
    if let Some(r) = guard(&auth) {
        return r;
    }
    let season = get_session_season(&state.pool, &session_id).await;
    let catalog = state.drivers_catalog_for_season(&season).await;
    templates::admin::drivers_list_page(&catalog, &season, &auth).into_response()
}

async fn new_driver_form(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
) -> impl IntoResponse {
    if let Some(r) = guard(&auth) {
        return r;
    }
    let season = get_session_season(&state.pool, &session_id).await;
    templates::admin::driver_form_page(None, &season, &auth).into_response()
}

async fn edit_driver_form(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Path(id): Path<i32>,
    auth: AuthStatus,
) -> impl IntoResponse {
    if let Some(r) = guard(&auth) {
        return r;
    }
    let season = get_session_season(&state.pool, &session_id).await;
    let catalog = state.drivers_catalog.read().await;
    let driver = catalog.iter().find(|d| d.id == id).cloned();
    drop(catalog);
    match driver {
        Some(d) => templates::admin::driver_form_page(Some(&d), &season, &auth).into_response(),
        None => Redirect::to("/admin/drivers").into_response(),
    }
}

#[derive(Deserialize)]
pub struct DriverForm {
    pub name: String,
    pub rarity: String,
    pub series: String,
    pub levels_json: String,
}

async fn create_driver(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
    Form(form): Form<DriverForm>,
) -> Result<Response, AppError> {
    if let Some(r) = guard(&auth) {
        return Ok(r);
    }
    let season = get_session_season(&state.pool, &session_id).await;

    let levels: Vec<OwnedDriverLevelStats> = match serde_json::from_str(&form.levels_json) {
        Ok(v) => v,
        Err(_) => return Ok(Redirect::to("/admin/drivers/new").into_response()),
    };

    let sort_order: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM driver_catalog WHERE season = $1",
    )
    .bind(&season)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let driver_id: i32 = sqlx::query_scalar(
        "INSERT INTO driver_catalog (name, season, rarity, series, sort_order)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id",
    )
    .bind(&form.name)
    .bind(&season)
    .bind(&form.rarity)
    .bind(&form.series)
    .bind(sort_order)
    .fetch_one(&state.pool)
    .await?;

    insert_driver_levels(&state, driver_id, &levels).await?;
    reload_drivers_catalog(&state).await;
    Ok(Redirect::to("/admin/drivers").into_response())
}

async fn update_driver(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    auth: AuthStatus,
    Form(form): Form<DriverForm>,
) -> Result<Response, AppError> {
    if let Some(r) = guard(&auth) {
        return Ok(r);
    }

    let levels: Vec<OwnedDriverLevelStats> = match serde_json::from_str(&form.levels_json) {
        Ok(v) => v,
        Err(_) => return Ok(Redirect::to("/admin/drivers").into_response()),
    };

    sqlx::query("UPDATE driver_catalog SET name=$1, rarity=$2, series=$3 WHERE id=$4")
        .bind(&form.name)
        .bind(&form.rarity)
        .bind(&form.series)
        .bind(id)
        .execute(&state.pool)
        .await?;

    sqlx::query("DELETE FROM driver_level_stats WHERE driver_id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    insert_driver_levels(&state, id, &levels).await?;
    reload_drivers_catalog(&state).await;
    Ok(Redirect::to("/admin/drivers").into_response())
}

async fn delete_driver(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    auth: AuthStatus,
) -> Result<Response, AppError> {
    if auth.enabled && !auth.logged_in {
        return Ok(maud::html! {}.into_response());
    }

    sqlx::query("DELETE FROM driver_catalog WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    reload_drivers_catalog(&state).await;
    Ok(maud::html! {}.into_response())
}

async fn export_drivers(State(state): State<AppState>, auth: AuthStatus) -> impl IntoResponse {
    if let Some(r) = guard(&auth) {
        return r.into_response();
    }

    let all = state.drivers_catalog.read().await.clone();

    let mut by_season: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    for driver in &all {
        by_season
            .entry(driver.season.clone())
            .or_default()
            .push(serde_json::json!({
                "name": driver.name,
                "rarity": driver.rarity,
                "series": driver.series,
                "sort_order": driver.sort_order,
                "levels": driver.levels,
            }));
    }

    let json = serde_json::to_string_pretty(&by_season).unwrap_or_default();
    (
        [
            (header::CONTENT_TYPE, "application/json".to_string()),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"drivers.json\"".to_string(),
            ),
        ],
        json,
    )
        .into_response()
}

// ── Season categories ────────────────────────────────────────────────────────

async fn list_seasons(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
) -> impl IntoResponse {
    if let Some(r) = guard(&auth) {
        return r;
    }
    let season_cats = state.season_categories.read().await.clone();
    let active = get_session_season(&state.pool, &session_id).await;
    templates::admin::seasons_page(&season_cats, &active, &auth).into_response()
}

async fn save_season_categories(
    State(state): State<AppState>,
    auth: AuthStatus,
    Form(form): Form<Vec<(String, String)>>,
) -> Result<Response, AppError> {
    if let Some(r) = guard(&auth) {
        return Ok(r);
    }

    let season = form
        .iter()
        .find(|(k, _)| k == "season")
        .map(|(_, v)| v.clone())
        .unwrap_or_default();

    if season.is_empty() {
        return Ok(Redirect::to("/admin/seasons").into_response());
    }

    let categories: Vec<&str> = form
        .iter()
        .filter(|(k, _)| k == "categories")
        .map(|(_, v)| v.as_str())
        .collect();

    // Replace categories for this season
    sqlx::query("DELETE FROM season_categories WHERE season = $1")
        .bind(&season)
        .execute(&state.pool)
        .await?;

    for cat_slug in &categories {
        sqlx::query(
            "INSERT INTO season_categories (season, category)
             VALUES ($1, $2::part_category)
             ON CONFLICT DO NOTHING",
        )
        .bind(&season)
        .bind(*cat_slug)
        .execute(&state.pool)
        .await?;
    }

    // Reload season categories
    let new_cats = catalog::load_season_categories(&state.pool).await;
    *state.season_categories.write().await = new_cats;

    Ok(Redirect::to("/admin/seasons").into_response())
}

// ── Helpers ──────────────────────────────────────────────────────────────────

async fn insert_levels(
    state: &AppState,
    part_id: i32,
    levels: &[OwnedLevelStats],
) -> Result<(), AppError> {
    for lvl in levels {
        let details =
            serde_json::to_value(&lvl.additional_stat_details).unwrap_or(serde_json::json!({}));
        sqlx::query(
            "INSERT INTO part_level_stats
             (part_id, level, speed, cornering, power_unit, qualifying, pit_stop_time,
              additional_stat_value, additional_stat_details)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(part_id)
        .bind(lvl.level)
        .bind(lvl.speed)
        .bind(lvl.cornering)
        .bind(lvl.power_unit)
        .bind(lvl.qualifying)
        .bind(lvl.pit_stop_time)
        .bind(lvl.additional_stat_value)
        .bind(details)
        .execute(&state.pool)
        .await?;
    }
    Ok(())
}

async fn reload_catalog(state: &AppState) {
    let new_catalog = catalog::load_catalog(&state.pool).await;
    *state.catalog.write().await = new_catalog;
}

async fn insert_driver_levels(
    state: &AppState,
    driver_id: i32,
    levels: &[OwnedDriverLevelStats],
) -> Result<(), AppError> {
    for lvl in levels {
        sqlx::query(
            "INSERT INTO driver_level_stats
             (driver_id, level, overtaking, defending, qualifying, race_start, tyre_management,
              cards_required, coins_cost, legacy_points)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(driver_id)
        .bind(lvl.level)
        .bind(lvl.overtaking)
        .bind(lvl.defending)
        .bind(lvl.qualifying)
        .bind(lvl.race_start)
        .bind(lvl.tyre_management)
        .bind(lvl.cards_required)
        .bind(lvl.coins_cost)
        .bind(lvl.legacy_points)
        .execute(&state.pool)
        .await?;
    }
    Ok(())
}

async fn reload_drivers_catalog(state: &AppState) {
    let new_catalog = catalog::load_drivers_catalog(&state.pool).await;
    *state.drivers_catalog.write().await = new_catalog;
}
