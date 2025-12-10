use crate::{Result, error::AppError};

use super::CDN;
use tencent_sdk::{
    client::TencentCloudAsync, core::TencentCloudResult, middleware::RetryAsync,
    services::cdn::UpdateDomainConfig, transport::async_impl::ReqwestAsync,
};

pub struct TencentCDN {
    pub client: TencentCloudAsync<RetryAsync<ReqwestAsync>>,
}

impl TencentCDN {
    pub fn new(secret_id: &str, secret_key: &str) -> TencentCloudResult<Self> {
        let client = TencentCloudAsync::builder(secret_id, secret_key)?
            .no_system_proxy() // optional convenience helper
            .with_default_region("ap-guangzhou")
            .with_retry(3, std::time::Duration::from_millis(200))
            .build()?;

        Ok(TencentCDN { client })
    }
}

#[async_trait::async_trait]
impl CDN for TencentCDN {
    async fn update_ssl(&self, domain: &str, cert_id: &str) -> Result<String> {
        let request = UpdateDomainConfig::new(domain, cert_id);
        match self.client.request(&request).await {
            Ok(response) => Ok(response.response.request_id),
            Err(e) => Err(AppError::CloudError(format!(
                "tencent cloud download ssl certificate failed:{}",
                e
            ))),
        }
    }
}
