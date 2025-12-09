use tracing::{debug, info};

use crate::config::TencentCloudConfig;
use crate::dns::{DNS, TencentDNS};
use crate::ssl::{ApplyStatus, SSL, TencentSSL};

#[derive(Debug, Clone)]
pub struct Domain {
    name: String,
    original_name: String,
    ssl_certificate_id: Option<String>,
    apply_status: Option<ApplyStatus>,
    dns_status: u8,
}

impl Domain {
    pub fn new(name: &str) -> Self {
        let original_name = name.split('.').skip(1).collect::<Vec<&str>>().join(".");
        Domain {
            name: name.to_string(),
            original_name,
            ssl_certificate_id: None,
            apply_status: None,
            dns_status: 0,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn original_name(&self) -> String {
        self.original_name.clone()
    }

    pub fn ssl_certificate_id(&self) -> Option<String> {
        self.ssl_certificate_id.clone()
    }

    pub fn set_ssl_certificate_id(&mut self, certificate_id: &str) {
        self.ssl_certificate_id = Some(certificate_id.to_string());
    }

    pub fn set_apply_status(&mut self, status: ApplyStatus) {
        self.apply_status = Some(status);
    }

    pub fn set_dns_status(&mut self, status: u8) {
        self.dns_status = status;
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
        self.ssl_certificate_id = Some(certificate_id);
        Ok(())
    }

    pub async fn check_ssl_status<Client: SSL + Sync>(
        &mut self,
        ssl_client: &Client,
    ) -> crate::Result<Option<ApplyStatus>> {
        if let Some(certificate_id) = &self.ssl_certificate_id {
            let result = ssl_client.check_status(certificate_id).await?;
            info!(
                "Checked SSL certificate status for domain {}: {},can_download:{}",
                self.name, result.status, result.can_download
            );
            self.apply_status = Some(result.clone());
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    pub async fn download_ssl_certificate<Client: SSL + Sync>(
        &self,
        ssl_client: &Client,
    ) -> crate::Result<Option<String>> {
        if let Some(certificate_id) = &self.ssl_certificate_id {
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
        self.dns_status = 1;
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
    if domain.ssl_certificate_id.is_none() {
        domain.apply_ssl(&ssl_client, "DNS").await?;
    }

    debug!("Applied SSL certificate for domain: {:?}", domain,);
    if let Some(certificate_id) = domain.clone().ssl_certificate_id {
        let result = ssl_client.check_status(&certificate_id).await?;
        info!(
            "Auto update SSL certificate status for domain {} certificate id {}: {},can_download:{}",
            domain.name, certificate_id, result.status, result.can_download
        );
        debug!("ApplyStatus: {:?}", result);
        if domain.dns_status == 0 {
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

        if result.can_download {
            let certificate_content = ssl_client.download(&certificate_id).await?;
            info!(
                "Downloaded SSL certificate for domain {}: content length {}",
                domain.name,
                certificate_content.len()
            );
        }
    }
    Ok(())
}
