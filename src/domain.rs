use crate::Result;
use crate::cdn::CDN;
use crate::dns::DNS;
use crate::ssl::{ApplyStatus, CertificateInfo, SSL, parse_cert_from_base64};
use serde::Deserialize;
use std::sync::Arc;
use tokio::time::{Duration, sleep};
use tracing::{debug, info};

#[derive(Debug, Deserialize, Clone)]
pub struct Domain {
    pub name: String,
    pub original_name: String,
    pub ssl_provider: CloudProvider,
    pub cdn_provider: CloudProvider,
    pub dns_provider: CloudProvider,
    pub ssl_info: Option<ApplyStatus>,
    pub dns_info: Option<DnsInfo>,
    pub certificate_info: Option<CertificateInfo>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
pub struct CloudProvider {
    pub name: String,
    pub secret_id: String,
    pub secret_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DnsInfo {
    pub dns_status: u8,
    pub dns_record_id: Option<u64>,
}

impl Domain {
    pub fn can_direct_update_ssl(&self) -> bool {
        self.ssl_provider == self.cdn_provider
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn original_name(&self) -> String {
        self.original_name.clone()
    }

    pub fn ssl_certificate_id(&self) -> Option<String> {
        self.ssl_info
            .as_ref()
            .map(|info| info.certificate_id.clone())
    }

    pub fn ssl_client(&self) -> Result<Arc<dyn SSL>> {
        let secret_id = &self.ssl_provider.secret_id;
        let secret_key = &self.ssl_provider.secret_key;
        let provider = &self.ssl_provider.name;
        Ok(crate::ssl_client(provider, secret_id, secret_key)?)
    }

    pub fn dns_client(&self) -> Result<Arc<dyn DNS>> {
        let secret_id = &self.dns_provider.secret_id;
        let secret_key = &self.dns_provider.secret_key;
        let provider = &self.dns_provider.name;
        Ok(crate::dns_client(provider, secret_id, secret_key)?)
    }

    pub fn cdn_client(&self) -> Result<Arc<dyn CDN>> {
        let secret_id = &self.cdn_provider.secret_id;
        let secret_key = &self.cdn_provider.secret_key;
        let provider = &self.cdn_provider.name;
        Ok(crate::cdn_client(provider, secret_id, secret_key)?)
    }

    pub fn set_dns_info(&mut self, info: DnsInfo) {
        self.dns_info = Some(info)
    }

    pub fn set_ssl_info(&mut self, info: ApplyStatus) {
        self.ssl_info = Some(info)
    }

    pub fn dns_status(&self) -> u8 {
        match &self.dns_info {
            Some(info) => info.dns_status,
            None => 255,
        }
    }

    pub async fn apply_ssl(&mut self, dv_auth_method: &str) -> Result<()> {
        let ssl_client = self.ssl_client()?;
        let certificate_id = ssl_client.apply(&self.name, dv_auth_method).await?;
        info!(
            "Applied SSL certificate for domain {}: {}",
            self.name, certificate_id
        );
        self.ssl_info = Some(ApplyStatus {
            certificate_id,
            dns_key: "".to_string(),
            dns_value: "".to_string(),
            status: 0,
            can_download: false,
        });
        Ok(())
    }

    pub async fn check_ssl_status(&mut self) -> Result<Option<ApplyStatus>> {
        let ssl_client = self.ssl_client()?;
        if let Some(certificate_id) = &self.ssl_certificate_id() {
            let result = ssl_client.check_status(certificate_id).await?;
            info!(
                "Checked SSL certificate status for domain {}: {},can_download:{}",
                self.name, result.status, result.can_download
            );
            self.ssl_info = Some(result.clone());
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    pub async fn add_dns_record(&mut self, record: &str, sub_domain: &str) -> Result<u64> {
        let dns_client = self.dns_client()?;
        let original_name = format!(".{}", self.original_name);
        let sub_domain = sub_domain.replace(&original_name, "");
        let record_id = dns_client
            .add_record(record, &self.original_name, &sub_domain)
            .await?;
        self.set_dns_info(DnsInfo {
            dns_status: 1,
            dns_record_id: Some(record_id),
        });
        Ok(record_id)
    }

    pub async fn modify_dns_record(&self, record: &str, record_id: u64) -> Result<u64> {
        let dns_client = self.dns_client()?;
        let record_id = dns_client
            .modify_record(record, record_id, &self.original_name)
            .await?;
        Ok(record_id)
    }

    pub async fn delete_dns_record(&self) -> Result<String> {
        let dns_client = self.dns_client()?;
        if let Some(info) = &self.dns_info
            && let Some(record_id) = info.dns_record_id
        {
            let request_id = dns_client
                .delete_record(record_id, &self.original_name)
                .await?;
            return Ok(request_id);
        }
        Ok("ok".to_string())
    }
}

pub async fn auto_update_ssl(mut domain: Domain) -> Result<()> {
    if domain.ssl_certificate_id().is_none() {
        domain.apply_ssl("DNS").await?;
    }

    debug!("Applied SSL certificate for domain: {:?}", domain);
    if let Some(mut certificate_id) = domain.ssl_certificate_id() {
        let ssl_client = domain.ssl_client()?;
        let cdn_client = domain.cdn_client()?;
        loop {
            let result = ssl_client.check_status(&certificate_id).await?;
            info!(
                "Auto update SSL certificate status for domain {} certificate id {}: {},can_download:{}",
                domain.name, certificate_id, result.status, result.can_download
            );
            debug!("ApplyStatus: {:?}", result);
            if result.can_download {
                if !domain.can_direct_update_ssl() {
                    let content = ssl_client.download(&certificate_id).await?;
                    let cert = parse_cert_from_base64(&content)?;
                    let other_ssl_client = crate::ssl_client(
                        &domain.cdn_provider.name,
                        &domain.cdn_provider.secret_id,
                        &domain.cdn_provider.secret_key,
                    )?;
                    certificate_id = other_ssl_client
                        .upload(&cert.public_key, &cert.private_key)
                        .await?;
                }
                let result = cdn_client
                    .update_ssl(&domain.name(), &certificate_id)
                    .await?;
                let _ = domain.delete_dns_record().await?;
                info!(
                    "Update SSL certificate for domain {} success: {}",
                    domain.name(),
                    result
                );
                return Ok(());
            }
            if domain.dns_status() == 0 || domain.dns_info.is_none() {
                let add_dns_record = domain
                    .add_dns_record(&result.dns_value, &result.dns_key)
                    .await?;
                info!(
                    "Added DNS record for domain {}: record id {}",
                    domain.name(),
                    add_dns_record
                );
            }
            info!("sleep 6 minutes for wait dns record verify");
            sleep(Duration::from_mins(6)).await;
        }
    }
    Ok(())
}
