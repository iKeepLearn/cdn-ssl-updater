#[async_trait::async_trait]
pub trait DNS {
    async fn add_record(&self, record: &str) -> Result<(), String>;
}
