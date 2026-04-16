//! Node.js ecosystem filters (npm, pnpm, yarn).

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Unified filter for npm/pnpm/yarn commands.
pub struct NodePackageFilter;

impl Filter for NodePackageFilter {
    fn name(&self) -> &'static str {
        "node-pkg"
    }

    fn matches(&self, command: &str) -> bool {
        matches!(command, "npm" | "pnpm" | "yarn" | "bun" | "npx")
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
            "install" | "i" | "add" => filter_install_output(&stdout, &stderr),
            "run" | "exec" => filter_run_output(&stdout, &stderr),
            "test" | "t" => filter_test_output(&stdout, &stderr),
            "ls" | "list" => filter_list_output(&stdout),
            "outdated" => filter_outdated_output(&stdout),
            "build" => filter_build_output(&stdout, &stderr),
            _ => {
                // Passthrough with basic cleanup
                if !stderr.is_empty() && stdout.is_empty() {
                    filter_generic(&stderr)
                } else if !stderr.is_empty() {
                    format!("{}\n{}", filter_generic(&stdout), filter_generic(&stderr))
                } else {
                    filter_generic(&stdout)
                }
            }
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        90
    }
}

/// Filter npm install output - show only summary.
fn filter_install_output(stdout: &str, stderr: &str) -> String {
    let mut result = Vec::new();

    // Look for summary line like "added 123 packages in 4s"
    let summary_re = Regex::new(r"(?:added|removed|changed|up to date)\s+\d+\s+package").unwrap();
    let time_re = Regex::new(r"in\s+[\d.]+[ms]*s?").unwrap();

    for line in stdout.lines().chain(stderr.lines()) {
        let trimmed = line.trim();

        // Skip npm warnings and notices
        if trimmed.starts_with("npm WARN") ||
           trimmed.starts_with("npm notice") ||
           trimmed.is_empty() {
            continue;
        }

        // Capture summary lines
        if summary_re.is_match(trimmed) {
            result.push(trimmed.to_string());
        }

        // Capture audit summary
        if trimmed.contains("vulnerabilities") && trimmed.contains("packages") {
            result.push(format!("⚠ {}", trimmed));
        }
    }

    // If no summary found, show minimal output
    if result.is_empty() {
        // Check for errors
        for line in stderr.lines() {
            if line.contains("ERR!") || line.contains("error") {
                result.push(line.trim().to_string());
            }
        }
        if result.is_empty() {
            result.push("✓ installed".to_string());
        }
    }

    result.join("\n")
}

/// Filter npm run output - show errors and key output.
fn filter_run_output(stdout: &str, stderr: &str) -> String {
    let mut result = Vec::new();
    let mut in_error = false;

    for line in stdout.lines().chain(stderr.lines()) {
        let trimmed = line.trim();

        // Skip npm lifecycle noise
        if trimmed.starts_with("> ") && trimmed.contains("@") {
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }

        // Capture errors
        if trimmed.contains("error") || trimmed.contains("Error") ||
           trimmed.contains("ERR!") || trimmed.starts_with("ERROR") {
            in_error = true;
            result.push(trimmed.to_string());
        } else if in_error && (trimmed.starts_with("at ") || trimmed.starts_with("  ")) {
            // Stack trace - limit
            if result.len() < 10 {
                result.push(trimmed.to_string());
            }
        } else {
            in_error = false;
        }
    }

    if result.is_empty() {
        "✓ completed".to_string()
    } else {
        result.join("\n")
    }
}

/// Filter npm test output - show failures only.
fn filter_test_output(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);
    let mut result = Vec::new();
    let mut failed_tests = Vec::new();
    let mut summary_line = String::new();

    // Look for test framework patterns
    let fail_re = Regex::new(r"(?i)(FAIL|✗|✖|×|\bfailed?\b)").unwrap();
    let pass_re = Regex::new(r"(?i)(PASS|✓|✔|passed?)").unwrap();
    let summary_re = Regex::new(r"(?i)(\d+\s+(?:passed|failed|skipped|tests?))").unwrap();

    for line in combined.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Capture failed tests
        if fail_re.is_match(trimmed) && !pass_re.is_match(trimmed) {
            failed_tests.push(trimmed.to_string());
        }

        // Capture summary
        if summary_re.is_match(trimmed) &&
           (trimmed.contains("passed") || trimmed.contains("failed") || trimmed.contains("total")) {
            summary_line = trimmed.to_string();
        }
    }

    // Build result
    if !failed_tests.is_empty() {
        result.push(format!("Failed ({}):", failed_tests.len()));
        for test in failed_tests.iter().take(10) {
            result.push(format!("  {}", test));
        }
        if failed_tests.len() > 10 {
            result.push(format!("  ... +{} more", failed_tests.len() - 10));
        }
    }

    if !summary_line.is_empty() {
        result.push(summary_line);
    }

    if result.is_empty() {
        "✓ all tests passed".to_string()
    } else {
        result.join("\n")
    }
}

/// Filter npm ls output - compact tree.
fn filter_list_output(stdout: &str) -> String {
    let mut result = Vec::new();
    let mut depth_counts: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for line in stdout.lines() {
        // Count leading spaces/tree chars to determine depth
        let depth = line.chars().take_while(|c| !c.is_alphanumeric() && *c != '@').count() / 2;

        if depth == 0 {
            // Root level - always show
            result.push(line.to_string());
        } else if depth == 1 {
            // Direct dependencies - show
            *depth_counts.entry(1).or_insert(0) += 1;
            if depth_counts[&1] <= 20 {
                result.push(line.to_string());
            }
        }
        // Skip deeper dependencies
    }

    let total_deps = depth_counts.get(&1).unwrap_or(&0);
    if *total_deps > 20 {
        result.push(format!("... +{} more dependencies", total_deps - 20));
    }

    result.join("\n")
}

/// Filter npm outdated output.
fn filter_outdated_output(stdout: &str) -> String {
    let mut result = Vec::new();
    let mut count = 0;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Header line
        if trimmed.starts_with("Package") || trimmed.starts_with("Name") {
            result.push("Outdated:".to_string());
            continue;
        }

        // Package lines - compact format
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 4 {
            count += 1;
            if count <= 15 {
                result.push(format!("  {} {} → {}", parts[0], parts[1], parts[3]));
            }
        }
    }

    if count > 15 {
        result.push(format!("  ... +{} more", count - 15));
    }

    if result.is_empty() {
        "✓ all up to date".to_string()
    } else {
        result.join("\n")
    }
}

/// Filter build output - show errors and summary.
fn filter_build_output(stdout: &str, stderr: &str) -> String {
    let mut result = Vec::new();
    let mut errors = Vec::new();

    for line in stdout.lines().chain(stderr.lines()) {
        let trimmed = line.trim();

        // Skip empty and noise
        if trimmed.is_empty() ||
           trimmed.starts_with("> ") ||
           trimmed.starts_with("npm WARN") {
            continue;
        }

        // Capture errors
        if trimmed.contains("error") || trimmed.contains("Error") ||
           trimmed.starts_with("ERROR") || trimmed.contains("failed") {
            errors.push(trimmed.to_string());
        }

        // Capture build success/summary
        if trimmed.contains("built") || trimmed.contains("compiled") ||
           trimmed.contains("Bundle") || trimmed.contains("chunks") {
            result.push(trimmed.to_string());
        }
    }

    if !errors.is_empty() {
        result.insert(0, format!("Errors ({}):", errors.len()));
        for err in errors.iter().take(10) {
            result.insert(result.len().min(11), format!("  {}", err));
        }
    }

    if result.is_empty() {
        "✓ build completed".to_string()
    } else {
        result.join("\n")
    }
}

/// Generic filter - remove noise.
fn filter_generic(output: &str) -> String {
    let mut result = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();

        // Skip common noise
        if trimmed.is_empty() ||
           trimmed.starts_with("npm WARN") ||
           trimmed.starts_with("npm notice") ||
           trimmed.starts_with("npm timing") {
            continue;
        }

        result.push(trimmed.to_string());
    }

    // Limit output
    if result.len() > 30 {
        let mut truncated: Vec<String> = result.iter().take(25).cloned().collect();
        truncated.push(format!("... +{} more lines", result.len() - 25));
        return truncated.join("\n");
    }

    result.join("\n")
}
