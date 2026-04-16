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

    // Handle --graph option
    if options.graph {
        return show_gain_graph(&db);
    }

    // Handle --history option
    if options.history {
        return show_gain_history(&db);
    }

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
            println!("{}", "📊 WTK Token Savings".bold());
            println!("{}", "═".repeat(60));
            println!();
            println!("  Total commands:    {}", format_number(stats.total_commands).cyan());
            println!("  Input tokens:      {}", format_tokens(stats.total_input).yellow());
            println!("  Output tokens:     {}", format_tokens(stats.total_output).green());
            println!(
                "  Tokens saved:      {} ({:.1}%)",
                format_tokens(stats.total_saved).bright_green(),
                stats.percent
            );
            println!();
            println!("  {}", render_efficiency_bar(stats.percent));
            println!();

            if !stats.by_command.is_empty() {
                println!("{}", "📋 By Command".bold());
                println!("{}", "─".repeat(60));
                println!(
                    "  {:30}  {:>5}  {:>8}  {:>6}",
                    "Command".dimmed(),
                    "Count".dimmed(),
                    "Saved".dimmed(),
                    "Avg%".dimmed()
                );
                println!("{}", "─".repeat(60));
                for (i, cmd) in stats.by_command.iter().take(10).enumerate() {
                    let pct_str = format!("{:.1}%", cmd.percent);
                    let pct_colored = if cmd.percent > 80.0 {
                        pct_str.bright_green()
                    } else if cmd.percent > 60.0 {
                        pct_str.green()
                    } else if cmd.percent > 40.0 {
                        pct_str.yellow()
                    } else {
                        pct_str.red()
                    };
                    println!(
                        "  {:30}  {:>5}  {:>8}  {}",
                        truncate(&cmd.command, 30),
                        cmd.count,
                        format_tokens(cmd.saved_chars),
                        pct_colored
                    );
                }
                println!("{}", "─".repeat(60));
                println!(
                    "  {:30}  {:>5}  {:>8}  {:.1}%",
                    "TOTAL".bold(),
                    stats.total_commands,
                    format_tokens(stats.total_saved).bright_green(),
                    stats.percent
                );
            }
            println!();
        }
    }

    Ok(())
}

/// Show ASCII graph of token savings over 30 days.
fn show_gain_graph(db: &TrackingDb) -> Result<()> {
    let daily = db.get_daily_stats(30)?;

    println!();
    println!("{}", "📈 WTK Token Savings - Last 30 Days".bold());
    println!("{}", "═".repeat(60));
    println!();

    if daily.is_empty() {
        println!("{}", "  No data yet. Run some commands through WTK first!".yellow());
        return Ok(());
    }

    // Find max saved for scaling
    let max_saved = daily.iter().map(|d| d.saved_chars).max().unwrap_or(1);
    let graph_height = 12;
    let graph_width = daily.len().min(30);
    let mid_value = max_saved / 2;

    // Render graph (top to bottom)
    for row in (0..graph_height).rev() {
        let threshold = (max_saved as f64 * (row as f64 + 1.0) / graph_height as f64) as usize;

        // Y-axis label
        if row == graph_height - 1 {
            print!("{:>8} │ ", format_tokens(max_saved).cyan());
        } else if row == graph_height / 2 {
            print!("{:>8} │ ", format_tokens(mid_value).dimmed());
        } else if row == 0 {
            print!("{:>8} │ ", "0".dimmed());
        } else {
            print!("         │ ");
        }

        // Bars (double width for better visibility)
        for day in daily.iter().take(graph_width) {
            if day.saved_chars >= threshold {
                print!("{}", "██".bright_green());
            } else if day.saved_chars >= threshold.saturating_sub(max_saved / graph_height / 2) {
                print!("{}", "▄▄".green());
            } else {
                print!("  ");
            }
        }
        println!();
    }

    // X-axis
    print!("         └─");
    print!("{}", "──".repeat(graph_width));
    println!();

    // Date labels
    if let (Some(first), Some(last)) = (daily.first(), daily.last()) {
        let first_date = &first.date[5..]; // MM-DD
        let last_date = &last.date[5..];
        let padding = (graph_width * 2).saturating_sub(first_date.len() + last_date.len());
        println!("           {}{}{}", first_date, " ".repeat(padding), last_date);
    }

    println!();

    // Summary box
    let total_saved: usize = daily.iter().map(|d| d.saved_chars).sum();
    let total_commands: usize = daily.iter().map(|d| d.commands).sum();
    let total_input: usize = daily.iter().map(|d| d.input_chars).sum();
    let avg_percent = if total_input > 0 {
        (total_saved as f64 / total_input as f64) * 100.0
    } else {
        0.0
    };

    println!("{}", "📊 Summary".bold());
    println!("{}", "─".repeat(40));
    println!("  Total saved:     {}", format_tokens(total_saved).bright_green());
    println!("  Commands:        {}", format_number(total_commands).cyan());
    println!("  Avg efficiency:  {}%", format!("{:.1}", avg_percent).bright_green());
    println!();
    println!("  {}", render_efficiency_bar(avg_percent));
    println!();

    Ok(())
}

/// Show recent command history.
fn show_gain_history(db: &TrackingDb) -> Result<()> {
    let history = db.get_history(20)?;

    println!();
    println!("{}", "📜 WTK Command History".bold());
    println!("{}", "═".repeat(72));
    println!();

    if history.is_empty() {
        println!("{}", "  No commands tracked yet.".yellow());
        return Ok(());
    }

    println!(
        "  {:19}  {:28}  {:>8}  {:>6}",
        "Timestamp".dimmed(),
        "Command".dimmed(),
        "Saved".dimmed(),
        "%".dimmed()
    );
    println!("{}", "─".repeat(72));

    let mut total_saved: usize = 0;
    let mut total_input: usize = 0;

    for entry in &history {
        let time = if entry.timestamp.len() > 19 {
            &entry.timestamp[..19]
        } else {
            &entry.timestamp
        };

        let saved = entry.input_chars.saturating_sub(entry.output_chars);
        total_saved += saved;
        total_input += entry.input_chars;

        let pct_str = format!("{:.1}%", entry.percent);
        let pct_colored = if entry.percent > 80.0 {
            pct_str.bright_green()
        } else if entry.percent > 60.0 {
            pct_str.green()
        } else if entry.percent > 40.0 {
            pct_str.yellow()
        } else {
            pct_str.red()
        };

        println!(
            "  {}  {:28}  {:>8}  {}",
            time.dimmed(),
            truncate(&entry.command, 28),
            format_tokens(saved),
            pct_colored
        );
    }

    // Totals row
    let avg_percent = if total_input > 0 {
        (total_saved as f64 / total_input as f64) * 100.0
    } else {
        0.0
    };

    println!("{}", "─".repeat(72));
    println!(
        "  {:19}  {:28}  {:>8}  {:.1}%",
        "",
        format!("TOTAL ({} commands)", history.len()).bold(),
        format_tokens(total_saved).bright_green(),
        avg_percent
    );
    println!();
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

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        let whole = n / 1_000;
        let frac = (n % 1_000) / 100;
        if frac > 0 {
            format!("{},{:03}", whole, n % 1_000)
        } else {
            format!("{},{:03}", whole, n % 1_000)
        }
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
