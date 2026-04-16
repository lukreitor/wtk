//! CLI command implementations.

use anyhow::{Context, Result};
use colored::Colorize;
use std::process::{Command, Stdio};

use crate::filters::registry::FilterRegistry;
use crate::hooks;
use crate::tracking::db::TrackingDb;

use super::{GainOptions, InitOptions};

/// Run a command with token-optimized output.
pub fn run_command(args: &[String]) -> Result<()> {
    if args.is_empty() {
        anyhow::bail!("No command provided");
    }

    let command = args[0].clone();
    let command_args: Vec<String> = args[1..].to_vec();

    // Get the filter registry
    let registry = FilterRegistry::new();

    // Find matching filter
    if let Some(filter) = registry.find_filter(&command) {
        tracing::debug!("Using filter: {}", filter.name());

        // Execute with filter
        let result = filter.execute(&command, &command_args)?;

        // Track the result
        let db = TrackingDb::open()?;
        db.track_command(
            &format!("{} {}", command, command_args.join(" ")),
            result.input_chars,
            result.output_chars,
            result.exec_time_ms,
            filter.name(),
        )?;

        // Print filtered output
        println!("{}", result.output);
    } else {
        // No filter found, execute raw command
        tracing::debug!("No filter found for: {}", command);
        execute_raw(&command, &command_args)?;
    }

    Ok(())
}

/// Rewrite a command for Claude Code hooks.
pub fn rewrite_command(command: &str) -> Result<()> {
    let registry = FilterRegistry::new();
    let parts: Vec<&str> = command.split_whitespace().collect();

    if parts.is_empty() {
        return Ok(());
    }

    // Check if we have a filter for this command
    if registry.find_filter(parts[0]).is_some() {
        // Output JSON for Claude Code hook
        let response = serde_json::json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "updatedInput": {
                    "command": format!("wtk {}", command)
                }
            }
        });
        println!("{}", serde_json::to_string(&response)?);
    }
    // If no filter, output nothing (passthrough)

    Ok(())
}

/// Show token savings statistics.
pub fn show_gain(options: GainOptions) -> Result<()> {
    let db = TrackingDb::open()?;
    let stats = db.get_statistics()?;

    match options.format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
        "csv" => {
            println!("command,count,input,output,saved,percent");
            for cmd in &stats.by_command {
                println!(
                    "{},{},{},{},{},{:.1}",
                    cmd.command, cmd.count, cmd.input_chars, cmd.output_chars, cmd.saved_chars, cmd.percent
                );
            }
        }
        _ => {
            // Text format
            println!();
            println!("{}", "WTK Token Savings".bold());
            println!("{}", "═".repeat(60));
            println!();
            println!("Total commands:    {}", stats.total_commands.to_string().cyan());
            println!("Input tokens:      {}", format_tokens(stats.total_input).yellow());
            println!("Output tokens:     {}", format_tokens(stats.total_output).green());
            println!(
                "Tokens saved:      {} ({:.1}%)",
                format_tokens(stats.total_saved).bright_green(),
                stats.percent
            );
            println!();
            println!("{}", render_efficiency_bar(stats.percent));
            println!();

            if !stats.by_command.is_empty() {
                println!("{}", "By Command".bold());
                println!("{}", "─".repeat(60));
                for (i, cmd) in stats.by_command.iter().take(10).enumerate() {
                    println!(
                        "{:>2}. {:30} {:>5}  {:>6}  {:>5.1}%",
                        i + 1,
                        truncate(&cmd.command, 30),
                        cmd.count,
                        format_tokens(cmd.saved_chars),
                        cmd.percent
                    );
                }
            }
        }
    }

    Ok(())
}

/// Initialize WTK hooks.
pub fn init(options: InitOptions) -> Result<()> {
    let mut installed = Vec::new();

    if options.all || options.claude_code {
        hooks::claude_code::install(options.global)
            .context("Failed to install Claude Code hooks")?;
        installed.push("Claude Code");
    }

    if options.all || options.powershell {
        hooks::powershell::install(options.global)
            .context("Failed to install PowerShell hooks")?;
        installed.push("PowerShell");
    }

    if options.all || options.cmd {
        hooks::cmd::install(options.global)
            .context("Failed to install CMD hooks")?;
        installed.push("CMD");
    }

    if installed.is_empty() {
        println!("{}", "No hooks specified. Use --claude-code, --powershell, --cmd, or --all".yellow());
    } else {
        println!("{} Installed hooks: {}", "✓".green(), installed.join(", "));
    }

    Ok(())
}

/// Discover missed WTK savings opportunities.
pub fn discover() -> Result<()> {
    println!("{}", "WTK Discover - Analyzing missed opportunities...".bold());
    println!();
    println!("{}", "This feature will analyze your shell history to find".dimmed());
    println!("{}", "commands that could benefit from WTK filtering.".dimmed());
    println!();
    println!("{}", "Coming soon!".yellow());

    Ok(())
}

/// Show current configuration.
pub fn show_config() -> Result<()> {
    let config = crate::config::load()?;

    println!("{}", "WTK Configuration".bold());
    println!("{}", "═".repeat(60));
    println!();
    println!("{}", toml::to_string_pretty(&config)?);

    Ok(())
}

// Helper functions

fn execute_raw(command: &str, args: &[String]) -> Result<()> {
    let status = Command::new(command)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to execute: {}", command))?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn format_tokens(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn render_efficiency_bar(percent: f64) -> String {
    let filled = (percent / 4.0).round() as usize;
    let empty = 25 - filled.min(25);

    format!(
        "Efficiency: {}{}  {:.1}%",
        "█".repeat(filled).green(),
        "░".repeat(empty).dimmed(),
        percent
    )
}
