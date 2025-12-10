mod tencent;

pub use tencent::TencentCDN;

use crate::Result;

#[async_trait::async_trait]
pub trait CDN: Send + Sync {
    async fn update_ssl(&self, domain: &str, cert_id: &str) -> Result<String>;
}
