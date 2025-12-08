// src/main.rs

use std::path::Path;
use std::process;

use anyhow::Result;
use clap::Parser;
use csu::cli::args::{Cli, Commands};
use csu::config::get_all_config;
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
    let mut config = match get_all_config(&cli.config) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load config: {}", e);
            anyhow::bail!(e);
        }
    };

    match cli.command {
        Commands::Check => {
            info!(
                "Checking SSL certificate status for domains: {}",
                cli.domains
            );
            // todo
        }
        Commands::Update => {
            info!("Updating SSL certificates for domains: {}", cli.domains);
            // todo
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
