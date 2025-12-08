#[async_trait::async_trait]
pub trait CDN {
    async fn update_ssl(&self, domain: &str, ssl_file: &str) -> Result<(), String>;
}
