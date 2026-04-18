use axum::{
    Router,
    extract::{Query, State},
    response::{Html, Json},
    routing::get,
};
use serde::{Deserialize, Serialize};

use super::domain::*;
use crate::AppState;
use crate::auth::AuthStatus;

#[derive(Deserialize)]
pub struct RangeParams {
    #[serde(default = "default_days")]
    pub days: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_days() -> u32 {
    30
}
fn default_limit() -> u32 {
    10
}

#[derive(Serialize)]
pub struct DashboardPayload {
    pub summary: Summary,
    pub visits_per_day: Vec<DailyCount>,
    pub top_paths: Vec<PathCount>,
    pub top_referrers: Vec<ReferrerCount>,
    pub top_countries: Vec<CountryCount>,
    pub device_breakdown: Vec<DeviceCount>,
    pub days: u32,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/stats", get(dashboard_page))
        .route("/admin/api/v1/stats/summary", get(summary))
        .route("/admin/api/v1/stats/visits", get(visits))
        .route("/admin/api/v1/stats/paths", get(paths))
        .route("/admin/api/v1/stats/referrers", get(referrers))
        .route("/admin/api/v1/stats/countries", get(countries))
        .route("/admin/api/v1/stats/devices", get(devices))
        .route("/admin/api/v1/stats/dashboard", get(dashboard))
}

async fn dashboard_page(auth: AuthStatus) -> axum::response::Response {
    if auth.enabled && !auth.logged_in {
        return axum::response::Redirect::to("/").into_response();
    }
    Html(include_str!("dashboard.html")).into_response()
}

use axum::response::IntoResponse;

async fn summary(
    State(state): State<AppState>,
    auth: AuthStatus,
    Query(q): Query<RangeParams>,
) -> Result<Json<Summary>, ApiError> {
    guard(&auth)?;
    Ok(Json(state.analytics.summary(q.days).await?))
}

async fn visits(
    State(state): State<AppState>,
    auth: AuthStatus,
    Query(q): Query<RangeParams>,
) -> Result<Json<Vec<DailyCount>>, ApiError> {
    guard(&auth)?;
    Ok(Json(state.analytics.visits_per_day(q.days).await?))
}

async fn paths(
    State(state): State<AppState>,
    auth: AuthStatus,
    Query(q): Query<RangeParams>,
) -> Result<Json<Vec<PathCount>>, ApiError> {
    guard(&auth)?;
    Ok(Json(state.analytics.top_paths(q.days, q.limit).await?))
}

async fn referrers(
    State(state): State<AppState>,
    auth: AuthStatus,
    Query(q): Query<RangeParams>,
) -> Result<Json<Vec<ReferrerCount>>, ApiError> {
    guard(&auth)?;
    Ok(Json(state.analytics.top_referrers(q.days, q.limit).await?))
}

async fn countries(
    State(state): State<AppState>,
    auth: AuthStatus,
    Query(q): Query<RangeParams>,
) -> Result<Json<Vec<CountryCount>>, ApiError> {
    guard(&auth)?;
    Ok(Json(state.analytics.top_countries(q.days, q.limit).await?))
}

async fn devices(
    State(state): State<AppState>,
    auth: AuthStatus,
    Query(q): Query<RangeParams>,
) -> Result<Json<Vec<DeviceCount>>, ApiError> {
    guard(&auth)?;
    Ok(Json(state.analytics.device_breakdown(q.days).await?))
}

async fn dashboard(
    State(state): State<AppState>,
    auth: AuthStatus,
    Query(q): Query<RangeParams>,
) -> Result<Json<DashboardPayload>, ApiError> {
    guard(&auth)?;
    let (summary, vpd, paths, refs, countries, devices) = tokio::try_join!(
        state.analytics.summary(q.days),
        state.analytics.visits_per_day(q.days),
        state.analytics.top_paths(q.days, q.limit),
        state.analytics.top_referrers(q.days, q.limit),
        state.analytics.top_countries(q.days, q.limit),
        state.analytics.device_breakdown(q.days),
    )?;

    Ok(Json(DashboardPayload {
        summary,
        visits_per_day: vpd,
        top_paths: paths,
        top_referrers: refs,
        top_countries: countries,
        device_breakdown: devices,
        days: q.days,
    }))
}

// ---- auth guard + error handling --------------------------------------------

fn guard(auth: &AuthStatus) -> Result<(), ApiError> {
    if auth.enabled && !auth.logged_in {
        Err(ApiError::Unauthorized)
    } else {
        Ok(())
    }
}

pub enum ApiError {
    Analytics(AnalyticsError),
    Unauthorized,
}

impl From<AnalyticsError> for ApiError {
    fn from(e: AnalyticsError) -> Self {
        Self::Analytics(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::Unauthorized => (
                axum::http::StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "unauthorized" })),
            )
                .into_response(),
            ApiError::Analytics(e) => {
                tracing::error!(error = %e, "analytics query failed");
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response()
            }
        }
    }
}
