//! .NET CLI filter (dotnet).

use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for .NET CLI commands.
pub struct DotnetFilter;

impl Filter for DotnetFilter {
    fn name(&self) -> &'static str {
        "dotnet"
    }

    fn matches(&self, command: &str) -> bool {
        matches!(command.to_lowercase().as_str(), "dotnet" | "dotnet.exe")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let subcommand = args.first().map(|s| s.as_str()).unwrap_or("");

        let start = Instant::now();

        let output = Command::new(command)
            .args(args)
            .output()?;

        let exec_time_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = match subcommand {
            "build" => filter_build_output(&stdout, &stderr),
            "test" => filter_test_output(&stdout, &stderr),
            "restore" => filter_restore_output(&stdout, &stderr),
            "publish" => filter_publish_output(&stdout, &stderr),
            "run" => filter_run_output(&stdout, &stderr),
            "ef" => filter_ef_output(&stdout, &stderr, args),
            "watch" => filter_watch_output(&stdout, &stderr),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter dotnet build output.
fn filter_build_output(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let error_re = Regex::new(r"^(.+?)\((\d+),(\d+)\):\s*(error|warning)\s+(\w+):\s*(.+)$").unwrap();
    let build_re = Regex::new(r"Build (succeeded|FAILED)").unwrap();
    let time_re = Regex::new(r"Time Elapsed\s+([\d:\.]+)").unwrap();

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut build_status = String::new();
    let mut build_time = String::new();

    for line in combined.lines() {
        let trimmed = line.trim();

        if let Some(caps) = error_re.captures(trimmed) {
            let file = shorten_path(&caps[1]);
            let line_num = &caps[2];
            let severity = &caps[4];
            let code = &caps[5];
            let msg = truncate_msg(&caps[6], 50);

            let formatted = format!("{}: {} L{} - {}", code, file, line_num, msg);

            if severity == "error" {
                errors.push(formatted);
            } else {
                warnings.push(formatted);
            }
        }

        if let Some(caps) = build_re.captures(trimmed) {
            build_status = caps[1].to_string();
        }

        if let Some(caps) = time_re.captures(trimmed) {
            build_time = caps[1].to_string();
        }
    }

    let mut result = Vec::new();

    // Status line
    if build_status == "succeeded" {
        if warnings.is_empty() {
            result.push(format!("✓ build succeeded ({})", build_time));
        } else {
            result.push(format!("✓ build succeeded ({}) - {} warnings", build_time, warnings.len()));
        }
    } else if build_status == "FAILED" {
        result.push(format!("✗ build FAILED - {} errors, {} warnings", errors.len(), warnings.len()));
    }

    // Show errors
    if !errors.is_empty() {
        result.push("Errors:".to_string());
        for err in errors.iter().take(10) {
            result.push(format!("  {}", err));
        }
        if errors.len() > 10 {
            result.push(format!("  ... +{} more errors", errors.len() - 10));
        }
    }

    // Show warnings (limited)
    if !warnings.is_empty() && errors.is_empty() {
        result.push("Warnings:".to_string());
        for warn in warnings.iter().take(5) {
            result.push(format!("  {}", warn));
        }
        if warnings.len() > 5 {
            result.push(format!("  ... +{} more warnings", warnings.len() - 5));
        }
    }

    if result.is_empty() {
        filter_generic(stdout, stderr)
    } else {
        result.join("\n")
    }
}

/// Filter dotnet test output.
fn filter_test_output(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let passed_re = Regex::new(r"Passed!\s+-\s+Failed:\s*(\d+),\s*Passed:\s*(\d+),\s*Skipped:\s*(\d+),\s*Total:\s*(\d+)").unwrap();
    let failed_re = Regex::new(r"Failed!\s+-\s+Failed:\s*(\d+),\s*Passed:\s*(\d+)").unwrap();
    let test_failed_re = Regex::new(r"Failed\s+(.+)$").unwrap();
    let error_msg_re = Regex::new(r"Error Message:").unwrap();

    let mut result = Vec::new();
    let mut failed_tests: Vec<String> = Vec::new();
    let mut in_error = false;
    let mut current_error = String::new();

    for line in combined.lines() {
        let trimmed = line.trim();

        // Summary line - passed
        if let Some(caps) = passed_re.captures(trimmed) {
            let failed: u32 = caps[1].parse().unwrap_or(0);
            let passed: u32 = caps[2].parse().unwrap_or(0);
            let skipped: u32 = caps[3].parse().unwrap_or(0);
            let total: u32 = caps[4].parse().unwrap_or(0);

            if failed == 0 {
                result.push(format!("✓ {}/{} passed ({} skipped)", passed, total, skipped));
            } else {
                result.push(format!("✗ {}/{} passed, {} failed", passed, total, failed));
            }
        }

        // Summary line - failed
        if let Some(caps) = failed_re.captures(trimmed) {
            let failed: u32 = caps[1].parse().unwrap_or(0);
            let passed: u32 = caps[2].parse().unwrap_or(0);
            result.push(format!("✗ {} failed, {} passed", failed, passed));
        }

        // Individual failed test
        if let Some(caps) = test_failed_re.captures(trimmed) {
            let test_name = caps[1].trim();
            failed_tests.push(test_name.to_string());
        }

        // Error message capture
        if error_msg_re.is_match(trimmed) {
            in_error = true;
            current_error.clear();
        } else if in_error {
            if trimmed.starts_with("Stack Trace:") || trimmed.is_empty() {
                in_error = false;
            } else {
                current_error = trimmed.to_string();
            }
        }
    }

    // Add failed tests
    if !failed_tests.is_empty() && result.iter().any(|r| r.contains("failed")) {
        result.push("Failed tests:".to_string());
        for test in failed_tests.iter().take(10) {
            result.push(format!("  ✗ {}", truncate_msg(test, 60)));
        }
        if failed_tests.len() > 10 {
            result.push(format!("  ... +{} more", failed_tests.len() - 10));
        }
    }

    if result.is_empty() {
        // Check for no tests
        if combined.contains("No test matches") || combined.contains("0 tests") {
            return "⚠ no tests found".to_string();
        }
        filter_generic(stdout, stderr)
    } else {
        result.join("\n")
    }
}

/// Filter dotnet restore output.
fn filter_restore_output(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let restored_re = Regex::new(r"Restored\s+(.+\.csproj)").unwrap();
    let up_to_date_re = Regex::new(r"up-to-date").unwrap();

    let mut restored = Vec::new();
    let mut has_errors = false;

    for line in combined.lines() {
        let trimmed = line.trim();

        if let Some(caps) = restored_re.captures(trimmed) {
            let proj = shorten_path(&caps[1]);
            restored.push(proj);
        }

        if trimmed.contains("error") || trimmed.contains("Error") {
            has_errors = true;
        }
    }

    if has_errors {
        return filter_generic(stdout, stderr);
    }

    if !restored.is_empty() {
        let mut result = vec![format!("✓ restored {} projects", restored.len())];
        for proj in restored.iter().take(5) {
            result.push(format!("  {}", proj));
        }
        if restored.len() > 5 {
            result.push(format!("  ... +{} more", restored.len() - 5));
        }
        result.join("\n")
    } else if up_to_date_re.is_match(&combined) {
        "✓ all packages up-to-date".to_string()
    } else {
        "✓ restore completed".to_string()
    }
}

/// Filter dotnet publish output.
fn filter_publish_output(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let publish_re = Regex::new(r"(.+) -> (.+)$").unwrap();

    let mut outputs = Vec::new();

    for line in combined.lines() {
        let trimmed = line.trim();

        if let Some(caps) = publish_re.captures(trimmed) {
            let output_path = shorten_path(&caps[2]);
            outputs.push(output_path);
        }
    }

    if !outputs.is_empty() {
        let mut result = vec!["✓ published".to_string()];
        for out in outputs.iter().take(3) {
            result.push(format!("  → {}", out));
        }
        result.join("\n")
    } else {
        filter_build_output(stdout, stderr)
    }
}

/// Filter dotnet run output.
fn filter_run_output(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    // For run, we mostly pass through but trim some noise
    let mut result = Vec::new();

    for line in combined.lines() {
        let trimmed = line.trim();

        // Skip build output noise
        if trimmed.contains("Determining projects to restore") ||
           trimmed.contains("All projects are up-to-date") ||
           trimmed.contains("Build started") ||
           trimmed.is_empty() {
            continue;
        }

        result.push(trimmed.to_string());
    }

    if result.len() > 30 {
        let mut truncated: Vec<String> = result.iter().take(25).cloned().collect();
        truncated.push(format!("... +{} more lines", result.len() - 25));
        truncated.join("\n")
    } else {
        result.join("\n")
    }
}

/// Filter dotnet ef output.
fn filter_ef_output(stdout: &str, stderr: &str, args: &[String]) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let subcommand = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subcommand {
        "migrations" => {
            let mut migrations = Vec::new();
            let migration_re = Regex::new(r"^\s*(\d{14}_\w+)").unwrap();

            for line in combined.lines() {
                if let Some(caps) = migration_re.captures(line) {
                    migrations.push(caps[1].to_string());
                }
            }

            if !migrations.is_empty() {
                let mut result = vec![format!("Migrations ({}):", migrations.len())];
                for m in migrations.iter().rev().take(10) {
                    result.push(format!("  {}", m));
                }
                if migrations.len() > 10 {
                    result.push(format!("  ... +{} more", migrations.len() - 10));
                }
                result.join("\n")
            } else {
                filter_generic(stdout, stderr)
            }
        }
        "database" => {
            if combined.contains("Done") || combined.contains("Applied") {
                "✓ database updated".to_string()
            } else if combined.contains("No migrations") {
                "✓ database up-to-date".to_string()
            } else {
                filter_generic(stdout, stderr)
            }
        }
        _ => filter_generic(stdout, stderr),
    }
}

/// Filter dotnet watch output.
fn filter_watch_output(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let mut result = Vec::new();
    let watch_re = Regex::new(r"watch\s*:\s*(.+)").unwrap();

    for line in combined.lines().rev().take(20) {
        let trimmed = line.trim();

        if let Some(caps) = watch_re.captures(trimmed) {
            result.insert(0, caps[1].to_string());
        } else if trimmed.contains("error") || trimmed.contains("Error") {
            result.insert(0, format!("✗ {}", trimmed));
        }
    }

    if result.is_empty() {
        "watching...".to_string()
    } else {
        result.join("\n")
    }
}

/// Generic filter for dotnet output.
fn filter_generic(stdout: &str, stderr: &str) -> String {
    let combined = if !stderr.is_empty() && stdout.is_empty() {
        stderr
    } else if !stderr.is_empty() {
        &format!("{}\n{}", stdout, stderr)
    } else {
        stdout
    };

    let lines: Vec<&str> = combined.lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() &&
            !t.starts_with("Microsoft") &&
            !t.starts_with("Copyright")
        })
        .collect();

    if lines.len() > 20 {
        let mut result: Vec<String> = lines.iter().take(15).map(|s| s.to_string()).collect();
        result.push(format!("... +{} more lines", lines.len() - 15));
        result.join("\n")
    } else {
        lines.join("\n")
    }
}

fn shorten_path(path: &str) -> String {
    let path = path.replace("\\", "/");

    if path.len() > 50 {
        if let Some(name) = path.split('/').last() {
            return name.to_string();
        }
    }

    path
}

fn truncate_msg(msg: &str, max_len: usize) -> String {
    if msg.len() <= max_len {
        msg.to_string()
    } else {
        format!("{}...", &msg[..max_len - 3])
    }
}
