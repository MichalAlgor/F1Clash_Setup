use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::{Form, Router};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::AuthStatus;
use crate::get_session_season;
use crate::models::driver::DriverInventoryItem;
use crate::models::setup::InventoryItem;
use crate::session::UserSession;

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
        "SELECT * FROM inventory WHERE season = $1 AND session_id = $2 ORDER BY part_name",
    )
    .bind(&season)
    .bind(&session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let drivers = sqlx::query_as::<_, DriverInventoryItem>(
        "SELECT * FROM driver_inventory WHERE season = $1 AND session_id = $2 ORDER BY driver_name",
    )
    .bind(&season)
    .bind(&session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let export = ExportData {
        season: season.clone(),
        parts: parts
            .iter()
            .map(|p| PartEntry {
                name: p.part_name.clone(),
                level: p.level,
            })
            .collect(),
        drivers: drivers
            .iter()
            .map(|d| DriverEntry {
                name: d.driver_name.clone(),
                rarity: d.rarity.clone(),
                level: d.level,
            })
            .collect(),
    };

    let json = serde_json::to_string_pretty(&export).unwrap();
    let filename = format!("f1clash_inventory_{season}.json");

    (
        [
            (header::CONTENT_TYPE, "application/json".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
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
        "Export / Import",
        &auth,
        maud::html! {
            hgroup {
                h1 { "Export / Import" }
                p { "Back up your inventory or restore it from a file." }
            }

            // ── Session info ──────────────────────────────────────────────────
            details style="margin-bottom:1.5rem" {
                summary { small.secondary { "Your session ID" } }
                p style="margin-top:0.5rem" {
                    code style="word-break:break-all;font-size:0.75em" { (&session_id) }
                }
                p {
                    small.secondary {
                        "Your data is tied to this browser's cookie. "
                        "Set the " code { "user_session" } " cookie to this value in another browser to access the same data."
                    }
                }
            }

            // ── Export ────────────────────────────────────────────────────────
            h2 { "Export" }
            p { "Download your current inventory for season " strong { (&season) } " as a JSON file." }
            a href="/export" role="button" { "Download inventory JSON" }

            hr style="margin:2rem 0";

            // ── Import ────────────────────────────────────────────────────────
            h2 { "Import" }
            p { "Restore inventory into season " strong { (&season) } " from a previously exported file." }

            form method="post" action="/import" {
                label for="json_data" { "JSON data" }
                textarea id="json_data" name="json_data" rows="10" required
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

    // NULL out setup references to this session's inventory/driver rows
    // before deleting — avoids FK constraint violations.
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
    .await
    .unwrap();

    sqlx::query(
        "UPDATE setups SET driver1_id=NULL WHERE driver1_id IN \
         (SELECT id FROM driver_inventory WHERE session_id=$1)",
    )
    .bind(&session_id)
    .execute(&state.pool)
    .await
    .unwrap();

    sqlx::query(
        "UPDATE setups SET driver2_id=NULL WHERE driver2_id IN \
         (SELECT id FROM driver_inventory WHERE session_id=$1)",
    )
    .bind(&session_id)
    .execute(&state.pool)
    .await
    .unwrap();

    sqlx::query("DELETE FROM inventory WHERE season = $1 AND session_id = $2")
        .bind(&season)
        .bind(&session_id)
        .execute(&state.pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM driver_inventory WHERE season = $1 AND session_id = $2")
        .bind(&season)
        .bind(&session_id)
        .execute(&state.pool)
        .await
        .unwrap();

    for part in &data.parts {
        if part.level < 1 {
            continue;
        }
        if state.find_part(&part.name, &season).await.is_none() {
            continue;
        }
        sqlx::query(
            "INSERT INTO inventory (part_name, level, season, session_id) VALUES ($1, $2, $3, $4)",
        )
        .bind(&part.name)
        .bind(part.level)
        .bind(&season)
        .bind(&session_id)
        .execute(&state.pool)
        .await
        .unwrap();
    }

    for driver in &data.drivers {
        if driver.level < 1 {
            continue;
        }
        if state
            .find_driver_def(&driver.name, &driver.rarity, &season)
            .await
            .is_none()
        {
            continue;
        }
        sqlx::query(
            "INSERT INTO driver_inventory (driver_name, rarity, level, season, session_id) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&driver.name).bind(&driver.rarity).bind(driver.level).bind(&season).bind(&session_id)
        .execute(&state.pool).await.unwrap();
    }

    Redirect::to("/inventory")
}
