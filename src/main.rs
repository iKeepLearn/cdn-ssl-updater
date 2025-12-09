// src/main.rs

use std::path::Path;
use std::process;

use clap::Parser;
use csu::Result;
use csu::cli::args::{Cli, Commands};
use csu::cli::command::{check_ssl_remin_days, update_ssl_certificate};
use csu::config::get_all_config;
use csu::error::AppError;
use reqwest::Client;
use tabled::Table;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if !Path::new(&cli.domains).exists() {
        error!("Domains file does not exist: {}", cli.domains);
        process::exit(1);
    }
    // 加载配置
    let config = match get_all_config(&cli.config) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load config: {}", e);
            return Err(AppError::ConfigError(e.to_string()));
        }
    };
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let valid_domains = match csu::parse_domains(&client, &cli.domains).await {
        Some(domains) => domains,
        None => {
            error!("No valid domains found in file: {}", cli.domains);
            process::exit(1);
        }
    };

    println!("Valid domains: {:?}", valid_domains);

    match cli.command {
        Commands::Check => {
            info!(
                "Checking SSL certificate status for domains: {}",
                cli.domains
            );
            let info = check_ssl_remin_days(valid_domains).await?;
            let table = Table::new(&info).to_string();
            println!("=== 域名列表 ===");
            println!("{}", table);
        }
        Commands::Update => {
            info!("Updating SSL certificates for domains: {}", cli.domains);
            update_ssl_certificate(valid_domains, &config.tencent_cloud).await?;
        }
        Commands::ForceUpdate => {
            info!(
                "Force updating SSL certificates for domains: {}",
                cli.domains
            );
            // todo
        }
        Commands::Version => {
            println!("CDN SSL Auto Updater version 1.0.0");
        }
    }
    Ok(())
}
