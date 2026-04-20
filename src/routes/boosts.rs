use axum::Form;
use axum::Router;
use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;

use crate::AppState;
use crate::auth::AuthStatus;
use crate::error::AppError;
use crate::get_session_season;
use crate::models::driver::DriverBoost;
use crate::models::setup::Boost;
use crate::session::UserSession;
use crate::templates;

pub fn router() -> Router<AppState> {
    Router::new().route("/boosts", get(show).post(save))
}

async fn show(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
) -> impl IntoResponse {
    let season = get_session_season(&state.pool, &session_id).await;
    let catalog = state.catalog_for_season(&season).await;

    let part_boosts = sqlx::query_as::<_, Boost>(
        "SELECT * FROM boosts WHERE season = $1 AND session_id = $2 ORDER BY part_name",
    )
    .bind(&season)
    .bind(&session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let driver_boosts = sqlx::query_as::<_, DriverBoost>(
        "SELECT * FROM driver_boosts WHERE season = $1 AND session_id = $2 ORDER BY driver_name",
    )
    .bind(&season)
    .bind(&session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let drivers_catalog = state.drivers_catalog_for_season(&season).await;
    templates::boosts::page(
        &part_boosts,
        &driver_boosts,
        &catalog,
        &drivers_catalog,
        &auth,
    )
}

async fn save(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Form(form): Form<Vec<(String, String)>>,
) -> Result<impl IntoResponse, AppError> {
    let season = get_session_season(&state.pool, &session_id).await;

    sqlx::query("DELETE FROM boosts WHERE season = $1 AND session_id = $2")
        .bind(&season)
        .bind(&session_id)
        .execute(&state.pool)
        .await?;
    sqlx::query("DELETE FROM driver_boosts WHERE season = $1 AND session_id = $2")
        .bind(&season)
        .bind(&session_id)
        .execute(&state.pool)
        .await?;

    for (key, value) in &form {
        let percentage: i32 = value.parse().unwrap_or(0);
        if percentage == 0 {
            continue;
        }

        if let Some(part_name) = key.strip_prefix("part:") {
            if state.find_part(part_name, &season).await.is_none() {
                continue;
            }
            sqlx::query(
                "INSERT INTO boosts (part_name, percentage, season, session_id) VALUES ($1, $2, $3, $4)",
            )
            .bind(part_name)
            .bind(percentage)
            .bind(&season)
            .bind(&session_id)
            .execute(&state.pool)
            .await?;
        } else if let Some(rest) = key.strip_prefix("driver:") {
            let Some((name, rarity_str)) = rest.rsplit_once(':') else {
                continue;
            };
            if state
                .find_driver_def(name, rarity_str, &season)
                .await
                .is_none()
            {
                continue;
            }
            sqlx::query(
                "INSERT INTO driver_boosts (driver_name, rarity, percentage, season, session_id) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(name)
            .bind(rarity_str)
            .bind(percentage)
            .bind(&season)
            .bind(&session_id)
            .execute(&state.pool)
            .await?;
        }
    }

    Ok(Redirect::to("/boosts"))
}
