mod check;
mod tencent;
mod utils;

pub use check::{CertificateInfo, check_ssl_certificate};
use serde::Deserialize;
pub use tencent::TencentSSL;
pub use utils::parse_cert_from_base64;

use crate::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct ApplyStatus {
    pub certificate_id: String,
    pub dns_key: String,
    pub dns_value: String,
    pub status: i32,
    pub can_download: bool,
}

#[async_trait::async_trait]
pub trait SSL: Send + Sync {
    async fn apply(&self, domain: &str, dv_auth_method: &str) -> Result<String>;
    async fn download(&self, certificate_id: &str) -> Result<String>;
    async fn check_status(&self, certificate_id: &str) -> Result<ApplyStatus>;
    async fn upload(&self, certificate_public_key: &str, private_key: &str) -> Result<String>;
}
