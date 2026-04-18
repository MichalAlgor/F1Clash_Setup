use axum::{
    extract::{ConnectInfo, Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use std::{net::SocketAddr, sync::Arc, time::Instant};

use super::{
    AnalyticsHandle,
    domain::{Device, PageEvent},
    geoip::GeoIpProvider,
};

/// State that the analytics middleware needs. Kept separate from AppState;
/// wired via `from_fn_with_state`.
#[derive(Clone)]
pub struct AnalyticsState {
    pub sink: AnalyticsHandle,
    pub geoip: Arc<dyn GeoIpProvider>,
}

/// Axum middleware that records a PageEvent for every non-skipped request.
/// Writes are fire-and-forget — they never block the response.
pub async fn record_analytics(
    State(state): State<AnalyticsState>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path();
    if should_skip(path) {
        return next.run(req).await;
    }

    // Extract request-side data BEFORE consuming the request.
    let path = path.to_string();
    let method = req.method().as_str().to_string();
    let headers = req.headers().clone();
    let addr = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0);

    let started = Instant::now();
    let response = next.run(req).await;
    let response_ms = started.elapsed().as_millis().min(u32::MAX as u128) as u32;

    let status = response.status().as_u16();
    let device = Device::from_user_agent(header_str(&headers, "user-agent"));
    let referrer = extract_referrer_host(header_str(&headers, "referer"));

    let ip = client_ip(&headers, addr);
    let geoip = state.geoip.clone();
    let sink = state.sink.clone();

    tokio::spawn(async move {
        let country = match ip {
            Some(ip) => geoip.lookup(ip).await,
            None => None,
        };

        let event = PageEvent {
            path,
            method,
            status,
            referrer,
            device,
            country,
            response_ms,
            ts: Utc::now(),
        };

        if let Err(e) = sink.record(event).await {
            tracing::warn!(error = %e, "failed to record page event");
        }
    });

    response
}

fn should_skip(path: &str) -> bool {
    path.starts_with("/admin")
        || path.starts_with("/auth")
        || path == "/favicon.ico"
        || path.starts_with("/static/")
}

fn header_str<'a>(h: &'a HeaderMap, name: &str) -> Option<&'a str> {
    h.get(name).and_then(|v| v.to_str().ok())
}

/// Return the Referer's host only — avoids leaking query strings or paths from referring sites.
fn extract_referrer_host(raw: Option<&str>) -> Option<String> {
    let raw = raw?;
    let parsed = url::Url::parse(raw).ok()?;
    parsed.host_str().map(|s| s.to_string())
}

/// Extract client IP. Prefers proxy headers (Render / Cloudflare set these).
fn client_ip(h: &HeaderMap, addr: Option<SocketAddr>) -> Option<std::net::IpAddr> {
    // Cloudflare
    if let Some(v) = header_str(h, "cf-connecting-ip").and_then(|s| s.parse().ok()) {
        return Some(v);
    }
    // Render / generic reverse proxy
    if let Some(first) = header_str(h, "x-forwarded-for")
        .and_then(|s| s.split(',').next())
        .map(str::trim)
        .and_then(|s| s.parse().ok())
    {
        return Some(first);
    }
    addr.map(|a| a.ip())
}
