use crate::cdn::{CDN, TencentCDN};
use crate::config::TencentCloudConfig;
use crate::dns::{DNS, TencentDNS};
use crate::ssl::{ApplyStatus, SSL, TencentSSL};
use tokio::time::{Duration, sleep};
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct Domain {
    name: String,
    original_name: String,
    ssl_info: Option<ApplyStatus>,
    dns_info: Option<DnsInfo>,
}

#[derive(Debug, Clone)]
pub struct DnsInfo {
    pub dns_status: u8,
    pub dns_record_id: Option<u64>,
}

impl Domain {
    pub fn new(name: &str) -> Self {
        let original_name = name.split('.').skip(1).collect::<Vec<&str>>().join(".");
        Domain {
            name: name.to_string(),
            original_name,
            ssl_info: None,
            dns_info: None,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn original_name(&self) -> String {
        self.original_name.clone()
    }

    pub fn ssl_certificate_id(&self) -> Option<String> {
        match &self.ssl_info {
            Some(info) => Some(info.certificate_id.clone()),
            None => None,
        }
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

    pub async fn apply_ssl<Client: SSL + Sync>(
        &mut self,
        ssl_client: &Client,
        dv_auth_method: &str,
    ) -> crate::Result<()> {
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

    pub async fn check_ssl_status<Client: SSL + Sync>(
        &mut self,
        ssl_client: &Client,
    ) -> crate::Result<Option<ApplyStatus>> {
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

    pub async fn download_ssl_certificate<Client: SSL + Sync>(
        &self,
        ssl_client: &Client,
    ) -> crate::Result<Option<String>> {
        if let Some(certificate_id) = &self.ssl_certificate_id() {
            let certificate_content = ssl_client.download(certificate_id).await?;
            Ok(Some(certificate_content))
        } else {
            Ok(None)
        }
    }

    pub async fn add_dns_record<Client: DNS + Sync>(
        &mut self,
        dns_client: &Client,
        record: &str,
        sub_domain: &str,
    ) -> crate::Result<u64> {
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

    pub async fn modify_dns_record<Client: DNS + Sync>(
        &self,
        dns_client: &Client,
        record: &str,
        record_id: u64,
    ) -> crate::Result<u64> {
        let record_id = dns_client
            .modify_record(record, record_id, &self.name)
            .await?;
        Ok(record_id)
    }

    pub async fn delete_dns_record<Client: DNS + Sync>(
        &self,
        dns_client: &Client,
        record_id: u64,
    ) -> crate::Result<String> {
        let request_id = dns_client.delete_record(record_id, &self.name).await?;
        Ok(request_id)
    }
}

pub async fn auto_update_ssl(mut domain: Domain, config: &TencentCloudConfig) -> crate::Result<()> {
    let ssl_client = TencentSSL::new(&config.secret_id, &config.secret_key)?;
    if domain.ssl_certificate_id().is_none() {
        domain.apply_ssl(&ssl_client, "DNS").await?;
    }

    debug!("Applied SSL certificate for domain: {:?}", domain,);
    if let Some(certificate_id) = domain.clone().ssl_certificate_id() {
        loop {
            let result = ssl_client.check_status(&certificate_id).await?;
            info!(
                "Auto update SSL certificate status for domain {} certificate id {}: {},can_download:{}",
                domain.name, certificate_id, result.status, result.can_download
            );
            debug!("ApplyStatus: {:?}", result);
            if result.can_download {
                let cdn_client = TencentCDN::new(&config.secret_id, &config.secret_key)?;
                let cert_id = &domain
                    .ssl_certificate_id()
                    .expect("certificate id should exists");
                let result = cdn_client.update_ssl(&domain.name(), cert_id).await?;
                info!(
                    "Update SSL certificate for domain {}: {}",
                    domain.name(),
                    result
                );
                break;
            }
            if domain.dns_status() == 0 {
                let dns_client = TencentDNS::new(&config.secret_id, &config.secret_key)?;
                let add_dns_record = domain
                    .add_dns_record(&dns_client, &result.dns_value, &result.dns_key)
                    .await?;
                info!(
                    "Added DNS record for domain {}: record id {}",
                    domain.name(),
                    add_dns_record
                );
            }
            let _ = sleep(Duration::from_mins(6));
        }
    }
    Ok(())
}
