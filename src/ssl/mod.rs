mod check;

pub use check::{CertificateInfo, check_ssl_certificate};

#[async_trait::async_trait]
pub trait SSL {
    async fn apply(&self, domain: &str) -> Result<String, String>;
    async fn download(&self, domain: &str) -> Result<String, String>;
}
