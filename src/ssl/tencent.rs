use crate::error::AppError;

use super::SSL;
use tencent_sdk::{
    client::TencentCloudAsync,
    core::TencentCloudResult,
    middleware::RetryAsync,
    services::ssl::{ApplyCertificate, DescribeCertificate, DownloadCertificate},
    transport::async_impl::ReqwestAsync,
};

pub struct TencentSSL {
    pub client: TencentCloudAsync<RetryAsync<ReqwestAsync>>,
}

impl TencentSSL {
    pub fn new(secret_id: &str, secret_key: &str) -> TencentCloudResult<Self> {
        let client = TencentCloudAsync::builder(secret_id, secret_key)?
            .no_system_proxy() // optional convenience helper
            .with_default_region("ap-guangzhou")
            .with_retry(3, std::time::Duration::from_millis(200))
            .build()?;

        Ok(TencentSSL { client })
    }
}

#[async_trait::async_trait]
impl SSL for TencentSSL {
    async fn apply(&self, domain: &str, dv_auth_method: &str) -> crate::Result<String> {
        let request = ApplyCertificate::new(dv_auth_method, domain);
        let response = self.client.request(&request).await?;
        match response.response.certificate_id {
            Some(certificate_id) => Ok(certificate_id),
            None => Err(AppError::CloudError(
                "tencent cloud apply ssl certificate failed".to_string(),
            )),
        }
    }

    async fn download(&self, certificate_id: &str) -> crate::Result<String> {
        let request = DownloadCertificate::new(certificate_id);
        let response = self.client.request(&request).await?;
        match response.response.content {
            Some(certificate_content) => Ok(certificate_content),
            None => Err(AppError::CloudError(
                "tencent cloud download ssl certificate failed".to_string(),
            )),
        }
    }

    async fn check_status(&self, certificate_id: &str) -> crate::Result<(i32, bool)> {
        let request = DescribeCertificate::new(certificate_id);
        let response = self.client.request(&request).await?;
        match response.response.status {
            Some(status) => Ok((status, status == 1)),
            None => Err(AppError::CloudError(
                "tencent cloud check ssl certificate status failed".to_string(),
            )),
        }
    }
}
