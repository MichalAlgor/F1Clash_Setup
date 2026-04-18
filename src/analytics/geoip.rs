use async_trait::async_trait;
use std::net::IpAddr;

#[async_trait]
pub trait GeoIpProvider: Send + Sync + 'static {
    async fn lookup(&self, ip: IpAddr) -> Option<String>;
}

/// No-op provider — always returns None. Swap in a real implementation later.
pub struct NoopGeoIp;

#[async_trait]
impl GeoIpProvider for NoopGeoIp {
    async fn lookup(&self, _ip: IpAddr) -> Option<String> {
        None
    }
}
