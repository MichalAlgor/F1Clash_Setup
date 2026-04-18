use async_trait::async_trait;
use std::net::IpAddr;

#[async_trait]
pub trait GeoIpProvider: Send + Sync + 'static {
    /// Returns ISO 3166-1 alpha-2 (e.g., "US") or None if unknown.
    async fn lookup(&self, ip: IpAddr) -> Option<String>;
}

/// No-op provider — always returns None. Good for local dev.
pub struct NoopGeoIp;

#[async_trait]
impl GeoIpProvider for NoopGeoIp {
    async fn lookup(&self, _ip: IpAddr) -> Option<String> {
        None
    }
}

// ---------------------------------------------------------------------------
// MaxMind GeoLite2 — offline, self-contained, free for personal use.
// ---------------------------------------------------------------------------

#[cfg(feature = "maxmind")]
pub mod maxmind {
    use super::*;
    use maxminddb::{Reader, geoip2};
    use std::path::Path;
    use std::sync::Arc;

    pub struct MaxMindGeoIp {
        reader: Arc<Reader<Vec<u8>>>,
    }

    impl MaxMindGeoIp {
        pub fn open(path: impl AsRef<Path>) -> Result<Self, maxminddb::MaxMindDbError> {
            let reader = Reader::open_readfile(path)?;
            Ok(Self {
                reader: Arc::new(reader),
            })
        }
    }

    #[async_trait]
    impl GeoIpProvider for MaxMindGeoIp {
        async fn lookup(&self, ip: IpAddr) -> Option<String> {
            let reader = self.reader.clone();
            tokio::task::spawn_blocking(move || {
                reader
                    .lookup::<geoip2::Country>(ip)
                    .ok()
                    .and_then(|c| c.country.and_then(|c| c.iso_code.map(|s| s.to_string())))
            })
            .await
            .ok()
            .flatten()
        }
    }
}

// ---------------------------------------------------------------------------
// HTTP lookup (ip-api.com) — zero setup, rate-limited (45 req/min free tier).
// ---------------------------------------------------------------------------

#[cfg(feature = "http-geoip")]
pub mod http {
    use super::*;
    use moka::future::Cache;
    use serde::Deserialize;
    use std::time::Duration;

    #[derive(Deserialize)]
    struct IpApiResp {
        #[serde(rename = "countryCode")]
        country_code: Option<String>,
    }

    pub struct HttpGeoIp {
        client: reqwest::Client,
        cache: Cache<IpAddr, Option<String>>,
    }

    impl HttpGeoIp {
        pub fn new() -> Self {
            Self {
                client: reqwest::Client::builder()
                    .timeout(Duration::from_millis(500))
                    .build()
                    .unwrap(),
                cache: Cache::builder()
                    .max_capacity(10_000)
                    .time_to_live(Duration::from_secs(86_400))
                    .build(),
            }
        }
    }

    #[async_trait]
    impl GeoIpProvider for HttpGeoIp {
        async fn lookup(&self, ip: IpAddr) -> Option<String> {
            if let Some(hit) = self.cache.get(&ip).await {
                return hit;
            }

            let url = format!("http://ip-api.com/json/{}?fields=countryCode", ip);
            let result = async {
                let r: IpApiResp = self.client.get(&url).send().await.ok()?.json().await.ok()?;
                r.country_code
            }
            .await;

            self.cache.insert(ip, result.clone()).await;
            result
        }
    }
}
