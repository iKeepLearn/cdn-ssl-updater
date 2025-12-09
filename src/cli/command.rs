use crate::{
    config::TencentCloudConfig,
    domain::{Domain, auto_update_ssl},
    ssl::{CertificateInfo, SSL, check_ssl_certificate},
};
use futures::StreamExt;
use futures::future::join_all;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::info;

pub async fn check_ssl_remin_days(domains: Vec<String>) -> crate::Result<Vec<CertificateInfo>> {
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
                if let Ok(info) = check_ssl_certificate(&domain) {
                    let _ = output_tx.send(info);
                }
            }
        })
        .await;

    drop(output_tx);
    drop(c_output);

    let output_rx_stream = UnboundedReceiverStream::new(output_rx);
    let info = output_rx_stream.collect().await;

    Ok(info)
}

pub async fn apply_ssl_certificate<Client: SSL + Sync>(
    domains: Vec<String>,
    ssl_client: &Client,
) -> crate::Result<Vec<Domain>> {
    let (input_tx, input_rx) = mpsc::unbounded_channel();
    let (output_tx, output_rx) = mpsc::unbounded_channel();
    let domains_clone = domains.clone();
    tokio::spawn(async move {
        for domain in domains_clone {
            let _ = input_tx.send(Domain::new(&domain));
        }
    });
    let domains_len = domains.len();
    let input_rx_stream = UnboundedReceiverStream::new(input_rx);
    let c_output = output_tx.clone();

    input_rx_stream
        .for_each_concurrent(domains_len, |domain| {
            let output_tx = c_output.clone();
            async move {
                let mut domain = domain;
                if domain.apply_ssl(ssl_client, "DNS").await.is_ok() {
                    let _ = output_tx.send(domain);
                }
            }
        })
        .await;

    drop(output_tx);
    drop(c_output);

    let output_rx_stream = UnboundedReceiverStream::new(output_rx);
    let info = output_rx_stream.collect().await;

    Ok(info)
}

pub async fn update_ssl_certificate(
    domains: Vec<String>,
    config: &TencentCloudConfig,
) -> crate::Result<()> {
    let mut tasks: Vec<JoinHandle<crate::Result<()>>> = Vec::with_capacity(domains.len());

    for domain in domains {
        let config = config.clone();
        let handle: JoinHandle<crate::Result<()>> = tokio::spawn(async move {
            let domain_obj = Domain::new(&domain);
            match auto_update_ssl(domain_obj.clone(), &config).await {
                Ok(_) => {
                    info!("Successfully updated SSL for domain: {}", domain_obj.name());
                    Ok(())
                }
                Err(e) => {
                    info!(
                        "Failed to update SSL for domain: {}: {}",
                        domain_obj.name(),
                        e
                    );
                    Err(e)
                }
            }
        });
        tasks.push(handle);
    }

    let _ = join_all(tasks).await;

    Ok(())
}
