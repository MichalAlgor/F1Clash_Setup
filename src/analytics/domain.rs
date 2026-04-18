use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// A single recorded page view. Fully anonymous.
#[derive(Debug, Clone)]
pub struct PageEvent {
    pub path: String,
    pub method: String,
    pub status: u16,
    pub referrer: Option<String>,
    pub device: Device,
    pub country: Option<String>, // ISO 3166-1 alpha-2
    pub response_ms: u32,
    pub ts: DateTime<Utc>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Device {
    Mobile,
    Desktop,
    Bot,
    Other,
}

impl Device {
    pub fn as_str(self) -> &'static str {
        match self {
            Device::Mobile => "mobile",
            Device::Desktop => "desktop",
            Device::Bot => "bot",
            Device::Other => "other",
        }
    }

    /// Coarse classification from a User-Agent string.
    /// Intentionally simple — we do not fingerprint.
    pub fn from_user_agent(ua: Option<&str>) -> Self {
        let Some(ua) = ua else {
            return Device::Other;
        };
        let lower = ua.to_ascii_lowercase();
        if lower.contains("bot")
            || lower.contains("crawler")
            || lower.contains("spider")
            || lower.contains("curl")
            || lower.contains("wget")
        {
            Device::Bot
        } else if lower.contains("mobile") || lower.contains("android") || lower.contains("iphone")
        {
            Device::Mobile
        } else if lower.contains("mozilla") {
            Device::Desktop
        } else {
            Device::Other
        }
    }
}

/// Write side of the analytics backend.
#[async_trait]
pub trait AnalyticsSink: Send + Sync + 'static {
    async fn record(&self, event: PageEvent) -> Result<(), AnalyticsError>;
}

/// Read side — aggregates only, never raw events.
#[async_trait]
pub trait AnalyticsQuery: Send + Sync + 'static {
    async fn visits_per_day(&self, days: u32) -> Result<Vec<DailyCount>, AnalyticsError>;
    async fn top_paths(&self, days: u32, limit: u32) -> Result<Vec<PathCount>, AnalyticsError>;
    async fn top_referrers(
        &self,
        days: u32,
        limit: u32,
    ) -> Result<Vec<ReferrerCount>, AnalyticsError>;
    async fn top_countries(
        &self,
        days: u32,
        limit: u32,
    ) -> Result<Vec<CountryCount>, AnalyticsError>;
    async fn device_breakdown(&self, days: u32) -> Result<Vec<DeviceCount>, AnalyticsError>;
    async fn summary(&self, days: u32) -> Result<Summary, AnalyticsError>;
}

#[derive(Debug, Serialize)]
pub struct DailyCount {
    pub day: chrono::NaiveDate,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct PathCount {
    pub path: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct ReferrerCount {
    pub referrer: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct CountryCount {
    pub country: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct DeviceCount {
    pub device: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub total_events: i64,
    pub unique_visitors: i64,
    pub unique_paths: i64,
    pub avg_response_ms: f64,
    pub bot_percentage: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum AnalyticsError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("{0}")]
    Other(String),
}

/// IP is only used by the GeoIP provider; it's never stored.
pub type ClientIp = Option<IpAddr>;
