use crate::error::AppError;

use super::{ApplyStatus, SSL};
use tencent_sdk::{
    client::TencentCloudAsync,
    core::TencentCloudResult,
    middleware::RetryAsync,
    services::ssl::{ApplyCertificate, CheckCertificate, DownloadCertificate},
    transport::async_impl::ReqwestAsync,
};
use tracing::debug;

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

    async fn check_status(&self, certificate_id: &str) -> crate::Result<ApplyStatus> {
        let request = CheckCertificate::new(certificate_id);
        match self.client.request(&request).await {
            Ok(response) => {
                debug!("CheckCertificate response: {:?}", response);
                if let Some(status) = response.response.status
                    && status == 1
                {
                    return Ok(ApplyStatus {
                        certificate_id: certificate_id.to_string(),
                        dns_key: "".to_string(),
                        dns_value: "".to_string(),
                        status,
                        can_download: true,
                    });
                }

                if let Some(status) = response.response.status
                    && status == 0
                    && let Some(dv_auth) = response.response.dv_auth_detail
                    && let Some(dv_auth_key) = dv_auth.dv_auth_key
                    && let Some(dv_auth_value) = dv_auth.dv_auth_value
                {
                    return Ok(ApplyStatus {
                        certificate_id: certificate_id.to_string(),
                        dns_key: dv_auth_key,
                        dns_value: dv_auth_value,
                        status,
                        can_download: false,
                    });
                }

                if let Some(status) = response.response.status {
                    Ok(ApplyStatus {
                        certificate_id: certificate_id.to_string(),
                        dns_key: "".to_string(),
                        dns_value: "".to_string(),
                        status,
                        can_download: false,
                    })
                } else {
                    Err(AppError::CloudError(
                        "tencent cloud check ssl certificate status missing fields".to_string(),
                    ))
                }
            }
            Err(e) => {
                return Err(AppError::CloudError(format!(
                    "tencent cloud check ssl certificate status failed: {}",
                    e
                )));
            }
        }
    }
}
