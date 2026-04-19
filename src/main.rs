//! WTK - Windows Token Killer
//!
//! CLI proxy that reduces LLM token consumption by 60-90% on Windows.

mod cli;
mod compress;
mod config;
mod filters;
mod hooks;
mod tracking;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::cli::{Cli, Commands};

fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Execute command
    match cli.command {
        Commands::Run(command) => {
            cli::commands::run_command(&command)?;
        }
        Commands::Rewrite { command } => {
            cli::commands::rewrite_command(command.as_deref())?;
        }
        Commands::Gain { options } => {
            cli::commands::show_gain(options)?;
        }
        Commands::Init { options } => {
            cli::commands::init(options)?;
        }
        Commands::Discover => {
            cli::commands::discover()?;
        }
        Commands::Config => {
            cli::commands::show_config()?;
        }
    }

    Ok(())
}

fn init_logging() {
    let filter = EnvFilter::try_from_env("WTK_LOG")
        .unwrap_or_else(|_| EnvFilter::new("warn"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}
