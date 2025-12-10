pub mod cdn;
pub mod cli;
pub mod dns;
pub mod domain;
pub mod error;
pub mod ssl;

pub type Result<T> = std::result::Result<T, error::AppError>;

use crate::cdn::{CDN, TencentCDN};
use crate::dns::{DNS, TencentDNS};
use crate::ssl::{SSL, TencentSSL};
use futures::StreamExt;
use reqwest::{Client, StatusCode, Url};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::info;

use crate::domain::Domain;

pub async fn parse_domains(client: &Client, domains: Vec<Domain>) -> Option<Vec<Domain>> {
    let domains: Vec<Domain> = domains
        .into_iter()
        .filter(|domain| !domain.name.trim().is_empty())
        .collect();

    if domains.is_empty() {
        info!("The domains file is empty.");
        return None;
    }

    let (input_tx, input_rx) = mpsc::unbounded_channel();
    let (output_tx, output_rx) = mpsc::unbounded_channel();
    let domains_clone = domains.clone();
    tokio::spawn(async move {
        for domain in domains_clone {
            let _ = input_tx.send(domain);
        }
    });
    let domains_len = domains.len();
    let input_rx_stream = UnboundedReceiverStream::new(input_rx);
    let c_output = output_tx.clone();

    input_rx_stream
        .for_each_concurrent(domains_len, |domain| {
            let output_tx = c_output.clone();
            async move {
                let is_valid = is_domain_valid(client, &domain.name).await;
                if is_valid {
                    let _ = output_tx.send(domain);
                }
            }
        })
        .await;

    drop(output_tx);
    drop(c_output);

    let output_rx_stream = UnboundedReceiverStream::new(output_rx);
    let valid_domains = output_rx_stream.collect().await;

    Some(valid_domains)
}

async fn is_domain_valid(client: &Client, domain: &str) -> bool {
    let url_string = format!("https://{}", domain);

    let url = match Url::parse(&url_string) {
        Ok(u) => u,
        Err(_) => return false,
    };

    match client.get(url).send().await {
        Ok(response) => {
            info!("Domain check for {}: HTTP {}", domain, response.status());
            response.status() != StatusCode::NOT_FOUND
        }
        Err(e) => {
            info!("Domain check failed for {}: {}", domain, e);
            false
        }
    }
}

pub fn ssl_client(provider: &str, secret_id: &str, secret_key: &str) -> Result<Arc<dyn SSL>> {
    match provider {
        "tencent" => {
            let ssl_client = TencentSSL::new(secret_id, secret_key)?;
            Ok(Arc::new(ssl_client))
        }
        _ => panic!("invalid ssl cloud provider"),
    }
}

pub fn dns_client(provider: &str, secret_id: &str, secret_key: &str) -> Result<Arc<dyn DNS>> {
    match provider {
        "tencent" => {
            let dns_client = TencentDNS::new(secret_id, secret_key)?;
            Ok(Arc::new(dns_client))
        }
        _ => panic!("invalid dns cloud provider"),
    }
}

pub fn cdn_client(provider: &str, secret_id: &str, secret_key: &str) -> Result<Arc<dyn CDN>> {
    match provider {
        "tencent" => {
            let cdn_client = TencentCDN::new(secret_id, secret_key)?;
            Ok(Arc::new(cdn_client))
        }
        _ => panic!("invalid cdn cloud provider"),
    }
}
