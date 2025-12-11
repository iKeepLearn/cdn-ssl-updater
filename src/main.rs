// src/main.rs

use clap::Parser;
use csu::Result;
use csu::cli::args::{Cli, Commands};
use csu::cli::command::{check_ssl_remin_days, update_ssl_certificate};
use csu::domain::Domain;
use csu::error::AppError;
use csu::ssl::CertificateInfo;
use reqwest::Client;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process;
use tabled::Table;
use tracing::{debug, error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if !Path::new(&cli.domains).exists() {
        error!("Domains file does not exist: {}", cli.domains);
        process::exit(1);
    }

    let file = File::open(&cli.domains)?;
    let reader = BufReader::new(file);

    let domains: Vec<Domain> = match serde_json::from_reader(reader) {
        Ok(value) => value,
        Err(e) => {
            error!("Failed to load domains: {}", e);
            return Err(AppError::ConfigError(e.to_string()));
        }
    };

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let valid_domains = match csu::parse_domains(&client, domains.clone()).await {
        Some(domains) => domains,
        None => {
            error!("No valid domains found in file: {}", cli.domains);
            process::exit(1);
        }
    };

    debug!("Valid domains: {:?}", valid_domains);

    match cli.command {
        Commands::Check => {
            info!(
                "Checking SSL certificate status for domains: {}",
                cli.domains
            );
            let info = check_ssl_remin_days(valid_domains).await?;
            let info: Vec<CertificateInfo> = info
                .into_iter()
                .map(|domain| domain.certificate_info.unwrap_or_default())
                .collect();
            let table = Table::new(&info).to_string();
            println!("=== 域名列表 ===");
            println!("{}", table);
        }
        Commands::Update => {
            info!("Updating SSL certificates for domains: {}", cli.domains);
            let info = check_ssl_remin_days(valid_domains).await?;
            let domains: Vec<Domain> = info
                .into_iter()
                .filter(|domain| match &domain.certificate_info {
                    Some(info) => info.need_update(),
                    None => true,
                })
                .collect();
            update_ssl_certificate(domains).await?;
        }
        Commands::ForceUpdate => {
            info!(
                "Force updating SSL certificates for domains: {}",
                cli.domains
            );
            update_ssl_certificate(domains).await?;
        }
        Commands::Version => {
            println!("CDN SSL Auto Updater version 2.1.0");
        }
    }
    Ok(())
}
