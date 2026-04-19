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
pub fn rewrite_command(command: Option<&str>) -> Result<()> {
    let registry = FilterRegistry::new();

    let cmd_str = if let Some(cmd) = command {
        cmd.to_string()
    } else {
        // Claude Code hook mode: reads JSON from stdin
        use std::io::Read;
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;

        if input.trim().is_empty() {
            return Ok(());
        }

        let hook_input: serde_json::Value = match serde_json::from_str(&input) {
            Ok(v) => v,
            Err(_) => return Ok(()),
        };

        match hook_input
            .get("tool_input")
            .and_then(|ti| ti.get("command"))
            .and_then(|c| c.as_str())
        {
            Some(cmd) => cmd.to_string(),
            None => return Ok(()),
        }
    };

    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }

    if registry.find_filter(parts[0]).is_some() {
        let response = serde_json::json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "updatedInput": {
                    "command": format!("wtk {}", cmd_str)
                }
            }
        });
        println!("{}", serde_json::to_string(&response)?);
    }
    // If no filter, output nothing (passthrough)

    Ok(())
}

/// Parse period string to days
fn parse_period(period: &str) -> u32 {
    match period.to_lowercase().as_str() {
        "1d" | "24h" | "today" => 1,
        "3d" => 3,
        "7d" | "1w" | "week" => 7,
        "14d" | "2w" => 14,
        "30d" | "1m" | "month" => 30,
        "90d" | "3m" => 90,
        "180d" | "6m" => 180,
        "365d" | "1y" | "year" => 365,
        "all" => 9999,
        _ => {
            // Try to parse as number of days
            period.trim_end_matches('d').parse().unwrap_or(30)
        }
    }
}

/// Get period label for display
fn period_label(period: &str) -> &str {
    match period.to_lowercase().as_str() {
        "1d" | "24h" | "today" => "Last 24 Hours",
        "3d" => "Last 3 Days",
        "7d" | "1w" | "week" => "Last 7 Days",
        "14d" | "2w" => "Last 2 Weeks",
        "30d" | "1m" | "month" => "Last 30 Days",
        "90d" | "3m" => "Last 3 Months",
        "180d" | "6m" => "Last 6 Months",
        "365d" | "1y" | "year" => "Last Year",
        "all" => "All Time",
        _ => "Custom Period",
    }
}

/// Show token savings statistics.
pub fn show_gain(options: GainOptions) -> Result<()> {
    let db = TrackingDb::open()?;
    let days = parse_period(&options.period);

    // Handle --graph option
    if options.graph {
        return show_gain_graph(&db, days, &options.period);
    }

    // Handle --history option
    if options.history {
        return show_gain_history(&db, options.limit, days, &options.period);
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

/// Show ASCII graph of token savings.
fn show_gain_graph(db: &TrackingDb, days: u32, period: &str) -> Result<()> {
    let daily = db.get_daily_stats(days)?;
    let label = period_label(period);

    println!();
    println!("{}", format!("📈 WTK Token Savings - {}", label).bold());
    println!("{}", "═".repeat(60));
    println!();

    if daily.is_empty() {
        println!("{}", "  No data yet. Run some commands through WTK first!".yellow());
        println!();
        println!("  {}", "Available periods:".dimmed());
        println!("    {} 1d, 7d, 30d, 90d, 1y, all", "→".dimmed());
        println!("    {} wtk gain --graph -T 7d", "Example:".dimmed());
        return Ok(());
    }

    // Find max saved for scaling
    let max_saved = daily.iter().map(|d| d.saved_chars).max().unwrap_or(1);
    let graph_height = 12;
    let max_width = if days <= 7 { 7 } else if days <= 30 { 30 } else { 60 };
    let graph_width = daily.len().min(max_width);
    let mid_value = max_saved / 2;
    let quarter_value = max_saved / 4;
    let three_quarter_value = (max_saved * 3) / 4;

    // Render graph (top to bottom)
    for row in (0..graph_height).rev() {
        let threshold = (max_saved as f64 * (row as f64 + 1.0) / graph_height as f64) as usize;

        // Y-axis label - show more labels
        if row == graph_height - 1 {
            print!("{:>8} │ ", format_tokens(max_saved).cyan());
        } else if row == (graph_height * 3) / 4 {
            print!("{:>8} │ ", format_tokens(three_quarter_value).dimmed());
        } else if row == graph_height / 2 {
            print!("{:>8} │ ", format_tokens(mid_value).dimmed());
        } else if row == graph_height / 4 {
            print!("{:>8} │ ", format_tokens(quarter_value).dimmed());
        } else if row == 0 {
            print!("{:>8} │ ", "0".dimmed());
        } else {
            print!("         │ ");
        }

        // Bars
        let bar_width = if days <= 7 { 4 } else if days <= 30 { 2 } else { 1 };
        for day in daily.iter().take(graph_width) {
            let bar = if bar_width == 4 { "████" } else if bar_width == 2 { "██" } else { "█" };
            let half_bar = if bar_width == 4 { "▄▄▄▄" } else if bar_width == 2 { "▄▄" } else { "▄" };
            let space = " ".repeat(bar_width);

            if day.saved_chars >= threshold {
                print!("{}", bar.bright_green());
            } else if day.saved_chars >= threshold.saturating_sub(max_saved / graph_height / 2) {
                print!("{}", half_bar.green());
            } else {
                print!("{}", space);
            }
        }
        println!();
    }

    // X-axis
    print!("         └─");
    let bar_width = if days <= 7 { 4 } else if days <= 30 { 2 } else { 1 };
    print!("{}", "─".repeat(graph_width * bar_width));
    println!();

    // Date labels
    if let (Some(first), Some(last)) = (daily.first(), daily.last()) {
        let first_date = &first.date[5..]; // MM-DD
        let last_date = &last.date[5..];
        let total_width = graph_width * bar_width;
        let padding = total_width.saturating_sub(first_date.len() + last_date.len());
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

    println!("{}", format!("📊 Summary ({})", label).bold());
    println!("{}", "─".repeat(50));
    println!("  Period:          {}", label.cyan());
    println!("  Days with data:  {}", daily.len().to_string().cyan());
    println!("  Total saved:     {}", format_tokens(total_saved).bright_green());
    println!("  Total input:     {}", format_tokens(total_input).yellow());
    println!("  Commands:        {}", format_number(total_commands).cyan());
    println!("  Avg efficiency:  {}%", format!("{:.1}", avg_percent).bright_green());
    println!();
    println!("  {}", render_efficiency_bar(avg_percent));
    println!();

    // Period suggestions
    println!("{}", "📅 Other periods:".dimmed());
    println!("  {} -T 1d (24h) | -T 7d (week) | -T 90d (3 months) | -T all", "→".dimmed());
    println!();

    Ok(())
}

/// Show recent command history.
fn show_gain_history(db: &TrackingDb, limit: usize, days: u32, period: &str) -> Result<()> {
    let history = db.get_history_with_period(limit, days)?;
    let label = period_label(period);

    println!();
    println!("{}", format!("📜 WTK Command History - {}", label).bold());
    println!("{}", "═".repeat(76));
    println!();

    if history.is_empty() {
        println!("{}", "  No commands tracked yet.".yellow());
        println!();
        println!("  {}", "Available options:".dimmed());
        println!("    {} -T 1d | -T 7d | -T 30d | -T all", "Period:".dimmed());
        println!("    {} -n 50 (show 50 entries)", "Limit:".dimmed());
        println!("    {} wtk gain --history -T 7d -n 50", "Example:".dimmed());
        return Ok(());
    }

    println!(
        "  {:19}  {:28}  {:>8}  {:>6}  {:>8}",
        "Timestamp".dimmed(),
        "Command".dimmed(),
        "Saved".dimmed(),
        "%".dimmed(),
        "Filter".dimmed()
    );
    println!("{}", "─".repeat(76));

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

        // Short filter name
        let filter_short = if entry.filter_name.len() > 8 {
            format!("{}...", &entry.filter_name[..5])
        } else {
            entry.filter_name.clone()
        };

        println!(
            "  {}  {:28}  {:>8}  {}  {:>8}",
            time.dimmed(),
            truncate(&entry.command, 28),
            format_tokens(saved),
            pct_colored,
            filter_short.dimmed()
        );
    }

    // Totals row
    let avg_percent = if total_input > 0 {
        (total_saved as f64 / total_input as f64) * 100.0
    } else {
        0.0
    };

    println!("{}", "─".repeat(76));
    println!(
        "  {:19}  {:28}  {:>8}  {:.1}%",
        "",
        format!("TOTAL ({} commands)", history.len()).bold(),
        format_tokens(total_saved).bright_green(),
        avg_percent
    );
    println!();

    // Period suggestions
    println!("{}", "📅 Options:".dimmed());
    println!("  {} -T 1d | -T 7d | -T 90d | -T all    {} -n 50", "Period:".dimmed(), "Limit:".dimmed());
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
    println!();
    println!("{}", "🔍 WTK Discover - Analyzing Shell History".bold());
    println!("{}", "═".repeat(60));
    println!();

    let registry = FilterRegistry::new();
    let mut opportunities: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut total_commands = 0;

    // Read PowerShell history
    if let Some(appdata) = dirs::data_local_dir() {
        let ps_history = appdata
            .join("Microsoft")
            .join("Windows")
            .join("PowerShell")
            .join("PSReadLine")
            .join("ConsoleHost_history.txt");

        if ps_history.exists() {
            if let Ok(content) = std::fs::read_to_string(&ps_history) {
                for line in content.lines().take(1000) {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }

                    // Skip if already using wtk
                    if trimmed.starts_with("wtk ") {
                        continue;
                    }

                    // Get first word (command)
                    let cmd = trimmed.split_whitespace().next().unwrap_or("");

                    // Check if we have a filter for this
                    if registry.find_filter(cmd).is_some() {
                        *opportunities.entry(cmd.to_string()).or_insert(0) += 1;
                        total_commands += 1;
                    }
                }
            }
        }
    }

    if opportunities.is_empty() {
        println!("  {} No missed opportunities found!", "✓".green());
        println!();
        println!("  {}", "Either you're already using WTK for everything,".dimmed());
        println!("  {}", "or your shell history is empty/inaccessible.".dimmed());
        println!();
        return Ok(());
    }

    // Sort by count
    let mut sorted: Vec<_> = opportunities.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    // Calculate estimated savings
    let estimated_savings_per_cmd = 500; // Average chars saved per command
    let total_estimated_savings = total_commands * estimated_savings_per_cmd;

    println!("  Found {} commands that could use WTK:", total_commands.to_string().cyan());
    println!();
    println!("{}", "─".repeat(60));
    println!(
        "  {:25}  {:>8}  {:>15}",
        "Command".dimmed(),
        "Count".dimmed(),
        "Est. Savings".dimmed()
    );
    println!("{}", "─".repeat(60));

    for (cmd, count) in sorted.iter().take(15) {
        let est_savings = *count * estimated_savings_per_cmd;
        println!(
            "  {:25}  {:>8}  {:>15}",
            cmd,
            count.to_string().cyan(),
            format_tokens(est_savings).green()
        );
    }

    if sorted.len() > 15 {
        println!("  ... and {} more commands", sorted.len() - 15);
    }

    println!("{}", "─".repeat(60));
    println!(
        "  {:25}  {:>8}  {:>15}",
        "TOTAL".bold(),
        total_commands.to_string().cyan(),
        format_tokens(total_estimated_savings).bright_green()
    );
    println!();

    // Recommendations
    println!("{}", "💡 Recommendations".bold());
    println!("{}", "─".repeat(60));
    println!("  1. Install Claude Code hooks: {}", "wtk init --claude-code".cyan());
    println!("  2. Or prefix commands manually: {}", "wtk <command>".cyan());
    println!();

    if let Some((top_cmd, _)) = sorted.first() {
        println!("  {} Most used: {} - try: {}",
            "→".yellow(),
            top_cmd.bright_white(),
            format!("wtk {}", top_cmd).cyan()
        );
    }

    println!();
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
