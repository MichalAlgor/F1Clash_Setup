pub mod admin;
pub mod domain;
pub mod geoip;
pub mod middleware;
pub mod postgres;

pub use domain::{AnalyticsQuery, AnalyticsSink};

use std::sync::Arc;

/// A single object that implements both Sink and Query.
pub trait AnalyticsBoth: AnalyticsSink + AnalyticsQuery {}
impl<T: AnalyticsSink + AnalyticsQuery> AnalyticsBoth for T {}

/// Alias so the rest of the app depends on traits, not the concrete type.
pub type AnalyticsHandle = Arc<dyn AnalyticsBoth>;
