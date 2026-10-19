use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::{Form, Router};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::AuthStatus;
use crate::error::AppError;
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
    #[serde(default, alias = "cards_owned")]
    pub cards: i32,
}

#[derive(Serialize, Deserialize)]
pub struct DriverEntry {
    pub name: String,
    pub rarity: String,
    pub level: i32,
    #[serde(default, alias = "cards_owned")]
    pub cards: i32,
}

async fn export(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
) -> Result<impl IntoResponse, AppError> {
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
                cards: p.cards_owned,
            })
            .collect(),
        drivers: drivers
            .iter()
            .map(|d| DriverEntry {
                name: d.driver_name.clone(),
                rarity: d.rarity.clone(),
                level: d.level,
                cards: d.cards_owned,
            })
            .collect(),
    };

    let json = serde_json::to_string_pretty(&export)?;
    let filename = format!("f1clash_inventory_{season}.json");

    crate::analytics::fire(
        &state.analytics,
        session_id.clone(),
        "export",
        serde_json::json!({ "season": season }),
    );

    Ok((
        [
            (header::CONTENT_TYPE, "application/json".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        json,
    ))
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
) -> Result<impl IntoResponse, AppError> {
    let season = get_session_season(&state.pool, &session_id).await;

    let Ok(data) = serde_json::from_str::<ExportData>(&form.json_data) else {
        return Ok(Redirect::to("/import"));
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
    .await?;

    sqlx::query(
        "UPDATE setups SET driver1_id=NULL WHERE driver1_id IN \
         (SELECT id FROM driver_inventory WHERE session_id=$1)",
    )
    .bind(&session_id)
    .execute(&state.pool)
    .await?;

    sqlx::query(
        "UPDATE setups SET driver2_id=NULL WHERE driver2_id IN \
         (SELECT id FROM driver_inventory WHERE session_id=$1)",
    )
    .bind(&session_id)
    .execute(&state.pool)
    .await?;

    sqlx::query("DELETE FROM inventory WHERE season = $1 AND session_id = $2")
        .bind(&season)
        .bind(&session_id)
        .execute(&state.pool)
        .await?;
    sqlx::query("DELETE FROM driver_inventory WHERE season = $1 AND session_id = $2")
        .bind(&season)
        .bind(&session_id)
        .execute(&state.pool)
        .await?;

    for part in &data.parts {
        if part.level < 1 {
            continue;
        }
        if state.find_part(&part.name, &season).await.is_none() {
            continue;
        }
        sqlx::query(
            "INSERT INTO inventory (part_name, level, cards_owned, season, session_id) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&part.name)
        .bind(part.level)
        .bind(part.cards.max(0))
        .bind(&season)
        .bind(&session_id)
        .execute(&state.pool)
        .await?;
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
            "INSERT INTO driver_inventory (driver_name, rarity, level, cards_owned, season, session_id) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&driver.name)
        .bind(&driver.rarity)
        .bind(driver.level)
        .bind(driver.cards.max(0))
        .bind(&season)
        .bind(&session_id)
        .execute(&state.pool)
        .await?;
    }

    let parts_count_bucket = match data.parts.len() {
        0 => "0",
        1..=7 => "1-7",
        8..=14 => "8-14",
        _ => "15+",
    };
    crate::analytics::fire(
        &state.analytics,
        session_id.clone(),
        "import",
        serde_json::json!({
            "season": season,
            "parts_count_bucket": parts_count_bucket,
        }),
    );

    Ok(Redirect::to("/inventory"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_data_serialization_with_cards() {
        let export = ExportData {
            season: "2026".to_string(),
            parts: vec![PartEntry {
                name: "Front Wing".to_string(),
                level: 5,
                cards: 120,
            }],
            drivers: vec![DriverEntry {
                name: "Verstappen".to_string(),
                rarity: "Epic".to_string(),
                level: 3,
                cards: 45,
            }],
        };
        let json = serde_json::to_string(&export).unwrap();
        assert!(json.contains("\"cards\":120"));
        assert!(json.contains("\"cards\":45"));

        let deserialized: ExportData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.parts[0].cards, 120);
        assert_eq!(deserialized.drivers[0].cards, 45);
    }

    #[test]
    fn test_import_backwards_compatible_without_cards() {
        let legacy_json = r#"{
            "season": "2026",
            "parts": [{"name": "Rear Wing", "level": 4}],
            "drivers": [{"name": "Hamilton", "rarity": "Rare", "level": 6}]
        }"#;

        let parsed: ExportData = serde_json::from_str(legacy_json).unwrap();
        assert_eq!(parsed.parts[0].cards, 0);
        assert_eq!(parsed.drivers[0].cards, 0);
    }

    #[test]
    fn test_import_with_cards_owned_alias() {
        let alias_json = r#"{
            "season": "2026",
            "parts": [{"name": "Engine", "level": 7, "cards_owned": 80}],
            "drivers": [{"name": "Norris", "rarity": "Epic", "level": 5, "cards_owned": 15}]
        }"#;

        let parsed: ExportData = serde_json::from_str(alias_json).unwrap();
        assert_eq!(parsed.parts[0].cards, 80);
        assert_eq!(parsed.drivers[0].cards, 15);
    }
}
