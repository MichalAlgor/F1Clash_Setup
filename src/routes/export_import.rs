use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::{Form, Router};
use serde::{Deserialize, Serialize};

use crate::auth::AuthStatus;
use crate::get_session_season;
use crate::session::UserSession;
use crate::models::driver::DriverInventoryItem;
use crate::models::setup::InventoryItem;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/export", get(export))
        .route("/import", get(import_form).post(import))
}

#[derive(Serialize, Deserialize)]
pub struct ExportData {
    pub season: String,
    pub parts: Vec<PartEntry>,
    pub drivers: Vec<DriverEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct PartEntry {
    pub name: String,
    pub level: i32,
}

#[derive(Serialize, Deserialize)]
pub struct DriverEntry {
    pub name: String,
    pub rarity: String,
    pub level: i32,
}

async fn export(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
) -> impl IntoResponse {
    let season = get_session_season(&state.pool, &session_id).await;

    let parts = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory WHERE season = $1 ORDER BY part_name",
    )
    .bind(&season)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let drivers = sqlx::query_as::<_, DriverInventoryItem>(
        "SELECT * FROM driver_inventory WHERE season = $1 ORDER BY driver_name",
    )
    .bind(&season)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let export = ExportData {
        season: season.clone(),
        parts: parts.iter().map(|p| PartEntry { name: p.part_name.clone(), level: p.level }).collect(),
        drivers: drivers.iter().map(|d| DriverEntry { name: d.driver_name.clone(), rarity: d.rarity.clone(), level: d.level }).collect(),
    };

    let json = serde_json::to_string_pretty(&export).unwrap();
    let filename = format!("f1clash_inventory_{season}.json");

    (
        [
            (header::CONTENT_TYPE, "application/json".to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{filename}\"")),
        ],
        json,
    )
}

async fn import_form(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
) -> impl IntoResponse {
    let season = get_session_season(&state.pool, &session_id).await;

    crate::templates::layout::page(
        "Import",
        &auth,
        maud::html! {
            hgroup {
                h1 { "Import Inventory" }
                p { "Paste exported JSON to import parts and drivers into season " strong { (&season) } }
            }

            form method="post" action="/import" {
                label for="json_data" { "JSON data" }
                textarea id="json_data" name="json_data" rows="12" required
                    placeholder="Paste the contents of your exported JSON file here..." {}
                button type="submit" { "Import" }
            }

            p.secondary { "This will replace all parts and drivers in the current season." }
        },
    )
}

#[derive(Deserialize)]
pub struct ImportForm {
    pub json_data: String,
}

async fn import(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Form(form): Form<ImportForm>,
) -> impl IntoResponse {
    let season = get_session_season(&state.pool, &session_id).await;

    let Ok(data) = serde_json::from_str::<ExportData>(&form.json_data) else {
        return Redirect::to("/import");
    };

    sqlx::query("DELETE FROM inventory WHERE season = $1")
        .bind(&season).execute(&state.pool).await.unwrap();
    sqlx::query("DELETE FROM driver_inventory WHERE season = $1")
        .bind(&season).execute(&state.pool).await.unwrap();

    for part in &data.parts {
        if part.level < 1 { continue; }
        if state.find_part(&part.name, &season).await.is_none() { continue; }
        sqlx::query("INSERT INTO inventory (part_name, level, season) VALUES ($1, $2, $3)")
            .bind(&part.name).bind(part.level).bind(&season)
            .execute(&state.pool).await.unwrap();
    }

    for driver in &data.drivers {
        if driver.level < 1 { continue; }
        if state.find_driver_def(&driver.name, &driver.rarity, &season).await.is_none() { continue; }
        sqlx::query("INSERT INTO driver_inventory (driver_name, rarity, level, season) VALUES ($1, $2, $3, $4)")
            .bind(&driver.name).bind(&driver.rarity).bind(driver.level).bind(&season)
            .execute(&state.pool).await.unwrap();
    }

    Redirect::to("/inventory")
}
