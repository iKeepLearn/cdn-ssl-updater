use crate::error::AppError;

use super::DNS;
use tencent_sdk::{
    client::TencentCloudAsync,
    core::TencentCloudResult,
    middleware::RetryAsync,
    services::dns::{CreateTXTRecord, DeleteRecord, ModifyTXTRecord},
    transport::async_impl::ReqwestAsync,
};

pub struct TencentDNS {
    pub client: TencentCloudAsync<RetryAsync<ReqwestAsync>>,
}

impl TencentDNS {
    pub fn new(secret_id: &str, secret_key: &str) -> TencentCloudResult<Self> {
        let client = TencentCloudAsync::builder(secret_id, secret_key)?
            .no_system_proxy() // optional convenience helper
            .with_default_region("ap-guangzhou")
            .with_retry(3, std::time::Duration::from_millis(200))
            .build()?;

        Ok(TencentDNS { client })
    }
}

#[async_trait::async_trait]
impl DNS for TencentDNS {
    async fn add_record(&self, record: &str, domain: &str, sub_domain: &str) -> crate::Result<u64> {
        let request = CreateTXTRecord::new(domain, "默认", record).with_sub_domain(sub_domain);
        let response = self.client.request(&request).await?;
        match response.response.record_id {
            Some(record_id) => Ok(record_id),
            None => Err(AppError::CloudError(
                "tencent cloud add dns record failed".to_string(),
            )),
        }
    }

    async fn modify_record(
        &self,
        record: &str,
        record_id: u64,
        domain: &str,
    ) -> crate::Result<u64> {
        let request = ModifyTXTRecord::new(domain, "默认", record, record_id);
        let response = self.client.request(&request).await?;
        match response.response.record_id {
            Some(record_id) => Ok(record_id),
            None => Err(AppError::CloudError(
                "tencent cloud modify dns record failed".to_string(),
            )),
        }
    }

    async fn delete_record(&self, record_id: u64, domain: &str) -> crate::Result<String> {
        let request = DeleteRecord::new(domain, record_id);
        match self.client.request(&request).await {
            Ok(response) => Ok(response.response.request_id),
            Err(e) => Err(AppError::CloudError(e.to_string())),
        }
    }
}
