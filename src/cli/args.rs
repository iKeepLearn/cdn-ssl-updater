// src/cli.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "csu")]
#[command(about = "CDN SSL auto updater", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Domains to manage, each line a domain
    #[arg(short, long)]
    pub domains: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// check ssl certificate status
    Check,
    /// update ssl certificates
    Update,
    /// force update ssl certificates
    ForceUpdate,
    /// Show tool version
    Version,
}
