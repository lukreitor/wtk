//! GitHub CLI filters (gh).

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for GitHub CLI commands.
pub struct GhFilter;

impl Filter for GhFilter {
    fn name(&self) -> &'static str {
        "gh"
    }

    fn matches(&self, command: &str) -> bool {
        command == "gh"
    }

    fn execute(&self, _command: &str, args: &[String]) -> Result<FilterResult> {
        let subcommand = args.first().map(|s| s.as_str()).unwrap_or("");
        let sub_subcommand = args.get(1).map(|s| s.as_str()).unwrap_or("");

        let start = Instant::now();

        let output = Command::new("gh")
            .args(args)
            .output()?;

        let exec_time_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = match (subcommand, sub_subcommand) {
            ("pr", "view") => filter_pr_view(&stdout),
            ("pr", "list") => filter_pr_list(&stdout),
            ("pr", "checks") => filter_pr_checks(&stdout),
            ("pr", "status") => filter_pr_status(&stdout),
            ("issue", "list") => filter_issue_list(&stdout),
            ("issue", "view") => filter_issue_view(&stdout),
            ("run", "list") => filter_run_list(&stdout),
            ("run", "view") => filter_run_view(&stdout),
            ("api", _) => filter_api_output(&stdout),
            _ => filter_generic_gh(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        90
    }
}

/// Filter gh pr view output.
fn filter_pr_view(stdout: &str) -> String {
    let mut result = Vec::new();

    let title_re = Regex::new(r"^title:\s*(.+)$").unwrap();
    let state_re = Regex::new(r"^state:\s*(\w+)").unwrap();
    let author_re = Regex::new(r"^author:\s*(\S+)").unwrap();
    let url_re = Regex::new(r"^url:\s*(.+)$").unwrap();

    let mut title = String::new();
    let mut state = String::new();
    let mut author = String::new();
    let mut url = String::new();
    let mut body_lines = Vec::new();
    let mut in_body = false;

    for line in stdout.lines() {
        if let Some(caps) = title_re.captures(line) {
            title = caps[1].to_string();
        } else if let Some(caps) = state_re.captures(line) {
            state = caps[1].to_string();
        } else if let Some(caps) = author_re.captures(line) {
            author = caps[1].to_string();
        } else if let Some(caps) = url_re.captures(line) {
            url = caps[1].to_string();
        } else if line.starts_with("--") {
            in_body = true;
        } else if in_body && body_lines.len() < 5 {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                body_lines.push(trimmed.to_string());
            }
        }
    }

    // Build compact output
    if !title.is_empty() {
        let state_emoji = match state.to_uppercase().as_str() {
            "OPEN" => "🟢",
            "CLOSED" => "🔴",
            "MERGED" => "🟣",
            _ => "⚪",
        };
        result.push(format!("{} {} ({})", state_emoji, title, author));
    }

    if !body_lines.is_empty() {
        result.push(body_lines.join(" ").chars().take(200).collect::<String>());
    }

    if !url.is_empty() {
        result.push(url);
    }

    if result.is_empty() {
        stdout.lines().take(10).collect::<Vec<_>>().join("\n")
    } else {
        result.join("\n")
    }
}

/// Filter gh pr list output.
fn filter_pr_list(stdout: &str) -> String {
    let mut result = Vec::new();
    let mut count = 0;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        count += 1;
        if count <= 10 {
            // Compact: show number, title (truncated), author
            let parts: Vec<&str> = trimmed.split('\t').collect();
            if parts.len() >= 3 {
                let num = parts[0];
                let title = if parts[1].len() > 50 {
                    format!("{}...", &parts[1][..47])
                } else {
                    parts[1].to_string()
                };
                result.push(format!("#{} {}", num, title));
            } else {
                result.push(trimmed.to_string());
            }
        }
    }

    if count > 10 {
        result.push(format!("... +{} more PRs", count - 10));
    }

    if result.is_empty() {
        "No open PRs".to_string()
    } else {
        result.join("\n")
    }
}

/// Filter gh pr checks output.
fn filter_pr_checks(stdout: &str) -> String {
    let mut passed = 0;
    let mut failed = 0;
    let mut pending = 0;
    let mut failed_checks = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();

        if trimmed.contains("pass") || trimmed.contains("✓") {
            passed += 1;
        } else if trimmed.contains("fail") || trimmed.contains("✗") || trimmed.contains("X") {
            failed += 1;
            // Extract check name
            let parts: Vec<&str> = trimmed.split('\t').collect();
            if !parts.is_empty() {
                failed_checks.push(parts[0].to_string());
            }
        } else if trimmed.contains("pending") || trimmed.contains("...") {
            pending += 1;
        }
    }

    let mut result = Vec::new();

    if failed > 0 {
        result.push(format!("✗ {} failed, {} passed", failed, passed));
        for check in failed_checks.iter().take(5) {
            result.push(format!("  - {}", check));
        }
    } else if pending > 0 {
        result.push(format!("⏳ {} pending, {} passed", pending, passed));
    } else if passed > 0 {
        result.push(format!("✓ {} checks passed", passed));
    } else {
        result.push("No checks".to_string());
    }

    result.join("\n")
}

/// Filter gh pr status output.
fn filter_pr_status(stdout: &str) -> String {
    let mut result = Vec::new();
    let mut section = "";

    for line in stdout.lines() {
        let trimmed = line.trim();

        if trimmed.contains("Created by you") {
            section = "mine";
            result.push("My PRs:".to_string());
        } else if trimmed.contains("Requesting a code review") {
            section = "review";
            result.push("Review requested:".to_string());
        } else if trimmed.starts_with('#') && !section.is_empty() {
            if result.len() < 10 {
                result.push(format!("  {}", trimmed));
            }
        }
    }

    if result.is_empty() {
        "No relevant PRs".to_string()
    } else {
        result.join("\n")
    }
}

/// Filter gh issue list output.
fn filter_issue_list(stdout: &str) -> String {
    let mut result = Vec::new();
    let mut count = 0;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        count += 1;
        if count <= 10 {
            let parts: Vec<&str> = trimmed.split('\t').collect();
            if parts.len() >= 2 {
                let num = parts[0];
                let title = if parts[1].len() > 50 {
                    format!("{}...", &parts[1][..47])
                } else {
                    parts[1].to_string()
                };
                result.push(format!("#{} {}", num, title));
            } else {
                result.push(trimmed.to_string());
            }
        }
    }

    if count > 10 {
        result.push(format!("... +{} more issues", count - 10));
    }

    if result.is_empty() {
        "No open issues".to_string()
    } else {
        result.join("\n")
    }
}

/// Filter gh issue view output.
fn filter_issue_view(stdout: &str) -> String {
    let mut result = Vec::new();
    let title_re = Regex::new(r"^title:\s*(.+)$").unwrap();
    let state_re = Regex::new(r"^state:\s*(\w+)").unwrap();

    let mut title = String::new();
    let mut state = String::new();
    let mut body_preview = String::new();

    for line in stdout.lines() {
        if let Some(caps) = title_re.captures(line) {
            title = caps[1].to_string();
        } else if let Some(caps) = state_re.captures(line) {
            state = caps[1].to_string();
        } else if line.starts_with("--") && body_preview.is_empty() {
            // Start of body
        } else if body_preview.len() < 200 && !title.is_empty() {
            body_preview.push_str(line.trim());
            body_preview.push(' ');
        }
    }

    if !title.is_empty() {
        let emoji = if state.to_uppercase() == "OPEN" { "🟢" } else { "🔴" };
        result.push(format!("{} {}", emoji, title));
    }

    if !body_preview.is_empty() {
        result.push(body_preview.chars().take(200).collect());
    }

    if result.is_empty() {
        stdout.lines().take(5).collect::<Vec<_>>().join("\n")
    } else {
        result.join("\n")
    }
}

/// Filter gh run list output.
fn filter_run_list(stdout: &str) -> String {
    let mut result = Vec::new();
    let mut count = 0;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        count += 1;
        if count <= 8 {
            // Parse: STATUS TITLE WORKFLOW BRANCH TIME ID
            let parts: Vec<&str> = trimmed.split('\t').collect();
            if parts.len() >= 4 {
                let status = parts[0];
                let title = if parts[1].len() > 40 {
                    format!("{}...", &parts[1][..37])
                } else {
                    parts[1].to_string()
                };
                let emoji = match status {
                    s if s.contains("success") || s.contains("✓") => "✓",
                    s if s.contains("failure") || s.contains("X") => "✗",
                    s if s.contains("pending") || s.contains("...") => "⏳",
                    s if s.contains("cancelled") => "⊘",
                    _ => "•",
                };
                result.push(format!("{} {}", emoji, title));
            } else {
                result.push(trimmed.to_string());
            }
        }
    }

    if count > 8 {
        result.push(format!("... +{} more runs", count - 8));
    }

    if result.is_empty() {
        "No workflow runs".to_string()
    } else {
        result.join("\n")
    }
}

/// Filter gh run view output.
fn filter_run_view(stdout: &str) -> String {
    let mut result = Vec::new();
    let mut jobs = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();

        // Capture status line
        if trimmed.starts_with("✓") || trimmed.starts_with("X") || trimmed.starts_with("*") {
            if result.is_empty() {
                result.push(trimmed.to_string());
            } else {
                jobs.push(trimmed.to_string());
            }
        }
    }

    // Add job summary
    if jobs.len() > 5 {
        for job in jobs.iter().take(5) {
            result.push(format!("  {}", job));
        }
        result.push(format!("  ... +{} more jobs", jobs.len() - 5));
    } else {
        for job in &jobs {
            result.push(format!("  {}", job));
        }
    }

    if result.is_empty() {
        stdout.lines().take(10).collect::<Vec<_>>().join("\n")
    } else {
        result.join("\n")
    }
}

/// Filter gh api output (JSON).
fn filter_api_output(stdout: &str) -> String {
    let trimmed = stdout.trim();

    if trimmed.is_empty() {
        return "✓ empty response".to_string();
    }

    // Try to parse JSON
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            return summarize_json_value(&value, 0);
        }
    }

    // Not JSON, truncate
    if trimmed.len() > 500 {
        format!("{}... ({} chars)", &trimmed[..500], trimmed.len())
    } else {
        trimmed.to_string()
    }
}

fn summarize_json_value(value: &serde_json::Value, depth: usize) -> String {
    if depth > 2 {
        return "...".to_string();
    }

    match value {
        serde_json::Value::Object(map) => {
            let keys: Vec<String> = map.keys()
                .take(8)
                .map(|k| k.to_string())
                .collect();
            if map.len() > 8 {
                format!("{{{}... +{}}}", keys.join(", "), map.len() - 8)
            } else {
                format!("{{{}}}", keys.join(", "))
            }
        }
        serde_json::Value::Array(arr) => {
            format!("[{} items]", arr.len())
        }
        serde_json::Value::String(s) => {
            if s.len() > 50 {
                format!("\"{}...\"", &s[..47])
            } else {
                format!("\"{}\"", s)
            }
        }
        _ => value.to_string()
    }
}

/// Generic gh output filter.
fn filter_generic_gh(stdout: &str, stderr: &str) -> String {
    let combined = if !stderr.is_empty() && stdout.is_empty() {
        stderr
    } else {
        stdout
    };

    let lines: Vec<&str> = combined.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.len() > 20 {
        let mut result: Vec<String> = lines.iter().take(15).map(|s| s.to_string()).collect();
        result.push(format!("... +{} more lines", lines.len() - 15));
        result.join("\n")
    } else {
        lines.join("\n")
    }
}
