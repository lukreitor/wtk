//! CLI module - Command line interface definitions and handlers.

pub mod commands;

use clap::{Parser, Subcommand};

/// WTK - Windows Token Killer
///
/// CLI proxy that reduces LLM token consumption by 60-90% on Windows.
#[derive(Parser, Debug)]
#[command(name = "wtk")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a command with token-optimized output
    #[command(external_subcommand)]
    Run(Vec<String>),

    /// Rewrite a command for Claude Code hooks (internal use)
    Rewrite {
        /// The command to rewrite
        command: String,
    },

    /// Show token savings statistics
    Gain {
        #[command(flatten)]
        options: GainOptions,
    },

    /// Initialize WTK hooks
    Init {
        #[command(flatten)]
        options: InitOptions,
    },

    /// Discover missed WTK savings opportunities
    Discover,

    /// Show current configuration
    Config,
}

#[derive(clap::Args, Debug)]
pub struct GainOptions {
    /// Show command history
    #[arg(short = 'H', long)]
    pub history: bool,

    /// Show ASCII graph of savings
    #[arg(short, long)]
    pub graph: bool,

    /// Show daily breakdown
    #[arg(short, long)]
    pub daily: bool,

    /// Show weekly breakdown
    #[arg(short, long)]
    pub weekly: bool,

    /// Show monthly breakdown
    #[arg(short, long)]
    pub monthly: bool,

    /// Group by filter
    #[arg(long)]
    pub by_filter: bool,

    /// Output format: text, json, csv
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Filter to current project
    #[arg(short, long)]
    pub project: bool,

    /// Time period: 1d, 7d, 30d, 90d, 1y, all (default: 30d)
    #[arg(short = 'T', long, default_value = "30d")]
    pub period: String,

    /// Number of history entries to show (default: 20)
    #[arg(short = 'n', long, default_value = "20")]
    pub limit: usize,
}

#[derive(clap::Args, Debug)]
pub struct InitOptions {
    /// Install Claude Code hooks
    #[arg(long)]
    pub claude_code: bool,

    /// Install PowerShell hooks
    #[arg(long)]
    pub powershell: bool,

    /// Install CMD hooks
    #[arg(long)]
    pub cmd: bool,

    /// Install all hooks
    #[arg(short, long)]
    pub all: bool,

    /// Global installation
    #[arg(short, long)]
    pub global: bool,
}
