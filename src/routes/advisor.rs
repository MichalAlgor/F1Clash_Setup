use axum::Router;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use serde::Deserialize;

use crate::AppState;
use crate::auth::AuthStatus;
use crate::data::StatPriorities;
use crate::get_session_season;
use crate::models::setup::{Boost, InventoryItem};
use crate::routes::optimizer::resolve_parts;
use crate::session::UserSession;
use crate::templates;
use crate::upgrade_advisor::run_upgrade_advisor;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/advisor", get(advisor_form))
        .route("/advisor/run", get(run_advisor))
}

async fn advisor_form(auth: AuthStatus) -> impl IntoResponse {
    templates::advisor::form_page(&auth)
}

#[derive(Deserialize)]
struct AdvisorQuery {
    #[serde(default)]
    speed: bool,
    #[serde(default)]
    cornering: bool,
    #[serde(default)]
    power_unit: bool,
    #[serde(default)]
    qualifying: bool,
    #[serde(default, deserialize_with = "deserialize_opt_i32")]
    max_part_series: Option<i32>,
}

fn deserialize_opt_i32<'de, D>(d: D) -> Result<Option<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    if s.trim().is_empty() {
        Ok(None)
    } else {
        s.trim()
            .parse::<i32>()
            .map(Some)
            .map_err(serde::de::Error::custom)
    }
}

async fn run_advisor(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
    axum::extract::Query(q): axum::extract::Query<AdvisorQuery>,
) -> impl IntoResponse {
    let season = get_session_season(&state.pool, &session_id).await;
    let max_part_series = q.max_part_series.unwrap_or(12);

    let priorities = StatPriorities {
        speed: q.speed,
        cornering: q.cornering,
        power_unit: q.power_unit,
        qualifying: q.qualifying,
    };

    // Resolve the pruned parts pool (same as optimizer)
    let (parts_per_cat, categories) =
        resolve_parts(&state, &season, &session_id, max_part_series).await;

    // Also load raw inventory, catalog, and boosts for the advisor simulation
    let catalog = state.catalog_for_season(&season).await;
    let inventory = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory WHERE season = $1 AND session_id = $2 ORDER BY part_name",
    )
    .bind(&season)
    .bind(&session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let boosts =
        sqlx::query_as::<_, Boost>("SELECT * FROM boosts WHERE season = $1 AND session_id = $2")
            .bind(&season)
            .bind(&session_id)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

    // Filter inventory by series limit (match what resolve_parts does)
    let filtered_inventory: Vec<InventoryItem> = inventory
        .into_iter()
        .filter(|item| {
            catalog
                .iter()
                .find(|p| p.name == item.part_name)
                .is_some_and(|p| p.series <= max_part_series)
        })
        .collect();

    let result = run_upgrade_advisor(
        &parts_per_cat,
        &categories,
        &catalog,
        &filtered_inventory,
        &boosts,
        &priorities,
    );

    templates::advisor::result_page(&result, &auth)
}
