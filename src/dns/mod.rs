mod tencent;

pub use tencent::TencentDNS;

use crate::Result;

#[async_trait::async_trait]
pub trait DNS: Send + Sync {
    async fn add_record(&self, record: &str, domain: &str, sub_domain: &str) -> Result<u64>;
    async fn modify_record(&self, record: &str, record_id: u64, domain: &str) -> Result<u64>;
    async fn delete_record(&self, record_id: u64, domain: &str) -> Result<String>;
}
