use crate::{
    domain::Domain,
    ssl::{CertificateInfo, SSL, check_ssl_certificate},
};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

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
