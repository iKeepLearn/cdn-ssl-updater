use tracing::info;

use crate::ssl::SSL;
pub struct Domain {
    name: String,
    ssl_certificate_id: Option<String>,
}

impl Domain {
    pub fn new(name: &str) -> Self {
        Domain {
            name: name.to_string(),
            ssl_certificate_id: None,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn ssl_certificate_id(&self) -> Option<String> {
        self.ssl_certificate_id.clone()
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
        &self,
        ssl_client: &Client,
    ) -> crate::Result<Option<(i32, bool)>> {
        if let Some(certificate_id) = &self.ssl_certificate_id {
            let (status, can_download) = ssl_client.check_status(certificate_id).await?;
            info!(
                "Checked SSL certificate status for domain {}: {},can_download:{}",
                self.name, status, can_download
            );
            Ok(Some((status, can_download)))
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
}
