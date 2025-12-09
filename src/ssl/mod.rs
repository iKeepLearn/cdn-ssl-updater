mod check;
mod tencent;

pub use check::{CertificateInfo, check_ssl_certificate};
pub use tencent::TencentSSL;

use crate::Result;

#[async_trait::async_trait]
pub trait SSL {
    async fn apply(&self, domain: &str, dv_auth_method: &str) -> Result<String>;
    async fn download(&self, certificate_id: &str) -> Result<String>;
    async fn check_status(&self, certificate_id: &str) -> Result<(i32, bool)>;
}
