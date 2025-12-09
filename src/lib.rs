pub mod cdn;
pub mod cli;
pub mod config;
pub mod dns;
pub mod domain;
pub mod error;
pub mod ssl;

pub type Result<T> = std::result::Result<T, error::AppError>;

use futures::StreamExt;
use reqwest::{Client, StatusCode, Url};
use std::fs::File;
use std::io::Read;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{error, info};

pub async fn parse_domains(client: &Client, domains_file: &str) -> Option<Vec<String>> {
    let mut file = match File::open(domains_file) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to open domains file {}: {}", domains_file, e);
            return None;
        }
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to read domains file {}: {}", domains_file, e);
            return None;
        }
    };

    // 2. Prepare the data: Split lines and collect into a Vec<String>
    let domains: Vec<String> = contents
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_lowercase())
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
                let is_valid = is_domain_valid(client, &domain).await;
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
