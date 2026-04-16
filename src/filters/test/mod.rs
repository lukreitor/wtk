//! Test runner filters (Vitest, Jest, Playwright).

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for Vitest.
pub struct VitestFilter;

impl Filter for VitestFilter {
    fn name(&self) -> &'static str {
        "vitest"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd.contains("vitest")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_vitest(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for Jest.
pub struct JestFilter;

impl Filter for JestFilter {
    fn name(&self) -> &'static str {
        "jest"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd.contains("jest")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_jest(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for Playwright.
pub struct PlaywrightFilter;

impl Filter for PlaywrightFilter {
    fn name(&self) -> &'static str {
        "playwright"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd.contains("playwright")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_playwright(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_vitest(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    // Remove ANSI codes
    let ansi_re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    let cleaned = ansi_re.replace_all(&combined, "").to_string();

    let summary_re = Regex::new(r"Tests\s+(\d+)\s+passed").unwrap();
    let failed_re = Regex::new(r"(\d+)\s+failed").unwrap();
    let skipped_re = Regex::new(r"(\d+)\s+skipped").unwrap();
    let time_re = Regex::new(r"Duration\s+([\d.]+[ms]*s?)").unwrap();
    let failed_test_re = Regex::new(r"FAIL\s+(.+)").unwrap();

    let passed = summary_re.captures(&cleaned).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);
    let failed = failed_re.captures(&cleaned).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);
    let skipped = skipped_re.captures(&cleaned).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);
    let duration = time_re.captures(&cleaned).map(|c| c[1].to_string()).unwrap_or_default();

    let mut result = Vec::new();

    if failed > 0 {
        result.push(format!("✗ {} passed, {} failed, {} skipped ({})", passed, failed, skipped, duration));

        // List failed tests
        let mut failed_tests: Vec<String> = Vec::new();
        for caps in failed_test_re.captures_iter(&cleaned) {
            failed_tests.push(caps[1].trim().to_string());
        }

        for t in failed_tests.iter().take(10) {
            result.push(format!("  ✗ {}", truncate(t, 60)));
        }
        if failed_tests.len() > 10 {
            result.push(format!("  ... +{} more", failed_tests.len() - 10));
        }
    } else if passed > 0 {
        result.push(format!("✓ {} passed, {} skipped ({})", passed, skipped, duration));
    } else if cleaned.contains("no test") {
        result.push("⚠ no tests found".to_string());
    }

    if result.is_empty() {
        filter_generic(stdout, stderr)
    } else {
        result.join("\n")
    }
}

fn filter_jest(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    // Remove ANSI codes
    let ansi_re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    let cleaned = ansi_re.replace_all(&combined, "").to_string();

    let summary_re = Regex::new(r"Tests:\s+(?:(\d+) failed,\s+)?(?:(\d+) skipped,\s+)?(\d+) passed,\s+(\d+) total").unwrap();
    let time_re = Regex::new(r"Time:\s+([\d.]+)\s*s").unwrap();
    let failed_test_re = Regex::new(r"FAIL\s+(.+)").unwrap();

    let (failed, skipped, passed, total) = if let Some(caps) = summary_re.captures(&cleaned) {
        (
            caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok()).unwrap_or(0),
            caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok()).unwrap_or(0),
            caps[3].parse::<u32>().unwrap_or(0),
            caps[4].parse::<u32>().unwrap_or(0),
        )
    } else {
        (0, 0, 0, 0)
    };

    let duration = time_re.captures(&cleaned).map(|c| format!("{}s", &c[1])).unwrap_or_default();

    let mut result = Vec::new();

    if failed > 0 {
        result.push(format!("✗ {} passed, {} failed, {} skipped ({})", passed, failed, skipped, duration));

        // List failed tests
        let mut failed_tests: Vec<String> = Vec::new();
        for caps in failed_test_re.captures_iter(&cleaned) {
            failed_tests.push(caps[1].trim().to_string());
        }

        for t in failed_tests.iter().take(10) {
            result.push(format!("  ✗ {}", truncate(t, 60)));
        }
        if failed_tests.len() > 10 {
            result.push(format!("  ... +{} more", failed_tests.len() - 10));
        }
    } else if passed > 0 || total > 0 {
        result.push(format!("✓ {}/{} passed, {} skipped ({})", passed, total, skipped, duration));
    } else if cleaned.contains("no tests") {
        result.push("⚠ no tests found".to_string());
    }

    if result.is_empty() {
        filter_generic(stdout, stderr)
    } else {
        result.join("\n")
    }
}

fn filter_playwright(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    // Remove ANSI codes
    let ansi_re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    let cleaned = ansi_re.replace_all(&combined, "").to_string();

    let summary_re = Regex::new(r"(\d+) passed").unwrap();
    let failed_re = Regex::new(r"(\d+) failed").unwrap();
    let skipped_re = Regex::new(r"(\d+) skipped").unwrap();
    let time_re = Regex::new(r"\(?([\d.]+[ms]*s?)\)?$").unwrap();
    let failed_test_re = Regex::new(r"\d+\)\s+\[.+?\]\s+›\s+(.+)").unwrap();

    let passed = summary_re.captures(&cleaned).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);
    let failed = failed_re.captures(&cleaned).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);
    let skipped = skipped_re.captures(&cleaned).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);
    let duration = time_re.captures(&cleaned).map(|c| c[1].to_string()).unwrap_or_default();

    let mut result = Vec::new();

    if failed > 0 {
        result.push(format!("✗ {} passed, {} failed, {} skipped ({})", passed, failed, skipped, duration));

        // List failed tests
        let mut failed_tests: Vec<String> = Vec::new();
        for caps in failed_test_re.captures_iter(&cleaned) {
            failed_tests.push(caps[1].trim().to_string());
        }

        for t in failed_tests.iter().take(10) {
            result.push(format!("  ✗ {}", truncate(t, 60)));
        }
        if failed_tests.len() > 10 {
            result.push(format!("  ... +{} more", failed_tests.len() - 10));
        }
    } else if passed > 0 {
        result.push(format!("✓ {} passed, {} skipped ({})", passed, skipped, duration));
    } else if cleaned.contains("no tests") {
        result.push("⚠ no tests found".to_string());
    }

    if result.is_empty() {
        filter_generic(stdout, stderr)
    } else {
        result.join("\n")
    }
}

fn filter_generic(stdout: &str, stderr: &str) -> String {
    let combined = if !stderr.is_empty() && stdout.is_empty() {
        stderr.to_string()
    } else if !stderr.is_empty() {
        format!("{}\n{}", stdout, stderr)
    } else {
        stdout.to_string()
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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
