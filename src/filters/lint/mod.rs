//! Linter and formatter filters (ESLint, Prettier, Biome).

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for ESLint.
pub struct EslintFilter;

impl Filter for EslintFilter {
    fn name(&self) -> &'static str {
        "eslint"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd.contains("eslint")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_eslint(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for Prettier.
pub struct PrettierFilter;

impl Filter for PrettierFilter {
    fn name(&self) -> &'static str {
        "prettier"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd.contains("prettier")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_prettier(&stdout, &stderr, args);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for Biome.
pub struct BiomeFilter;

impl Filter for BiomeFilter {
    fn name(&self) -> &'static str {
        "biome"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd.contains("biome")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_biome(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_eslint(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let problem_re = Regex::new(r"(\d+) problems? \((\d+) errors?, (\d+) warnings?\)").unwrap();
    let file_re = Regex::new(r"^(/[^\s]+|[A-Z]:\\[^\s]+)$").unwrap();
    let issue_re = Regex::new(r"^\s*(\d+):(\d+)\s+(error|warning)\s+(.+?)\s+(\S+)$").unwrap();

    let mut issues_by_file: std::collections::HashMap<String, Vec<(String, String, String)>> = std::collections::HashMap::new();
    let mut current_file = String::new();
    let mut total_errors = 0u32;
    let mut total_warnings = 0u32;

    for line in combined.lines() {
        if let Some(caps) = file_re.captures(line) {
            current_file = shorten_path(&caps[1]);
        } else if let Some(caps) = issue_re.captures(line) {
            let line_num = &caps[1];
            let severity = &caps[3];
            let msg = &caps[4];
            let rule = &caps[5];

            if severity == "error" {
                total_errors += 1;
            } else {
                total_warnings += 1;
            }

            if !current_file.is_empty() {
                issues_by_file
                    .entry(current_file.clone())
                    .or_default()
                    .push((line_num.to_string(), rule.to_string(), truncate(msg, 40)));
            }
        } else if let Some(caps) = problem_re.captures(line) {
            total_errors = caps[2].parse().unwrap_or(0);
            total_warnings = caps[3].parse().unwrap_or(0);
        }
    }

    if issues_by_file.is_empty() && total_errors == 0 && total_warnings == 0 {
        "✓ eslint clean".to_string()
    } else {
        let mut result = vec![format!("⚠ {} errors, {} warnings ({} files)",
            total_errors, total_warnings, issues_by_file.len())];

        for (file, issues) in issues_by_file.iter().take(5) {
            result.push(format!("\n{} ({} issues)", file, issues.len()));
            for (line, rule, msg) in issues.iter().take(3) {
                result.push(format!("  L{}: {} ({})", line, msg, rule));
            }
            if issues.len() > 3 {
                result.push(format!("  ... +{} more", issues.len() - 3));
            }
        }

        if issues_by_file.len() > 5 {
            result.push(format!("\n... +{} more files", issues_by_file.len() - 5));
        }

        result.join("\n")
    }
}

fn filter_prettier(stdout: &str, stderr: &str, args: &[String]) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let is_check = args.iter().any(|a| a == "--check" || a == "-c");

    if is_check {
        let unformatted_re = Regex::new(r"Checking formatting\.\.\.").unwrap();
        let file_re = Regex::new(r"^\[warn\]\s*(.+)$").unwrap();

        let mut unformatted_files: Vec<String> = Vec::new();
        for line in combined.lines() {
            if let Some(caps) = file_re.captures(line) {
                unformatted_files.push(shorten_path(&caps[1]));
            }
        }

        if unformatted_files.is_empty() {
            "✓ All files formatted".to_string()
        } else {
            let mut result = vec![format!("⚠ {} files need formatting", unformatted_files.len())];
            for f in unformatted_files.iter().take(10) {
                result.push(format!("  {}", f));
            }
            if unformatted_files.len() > 10 {
                result.push(format!("  ... +{} more", unformatted_files.len() - 10));
            }
            result.join("\n")
        }
    } else {
        // Write mode - just list files that were formatted
        let files: Vec<&str> = combined.lines()
            .filter(|l| !l.is_empty() && !l.starts_with("["))
            .collect();

        if files.is_empty() {
            "✓ formatted".to_string()
        } else {
            format!("✓ formatted {} files", files.len())
        }
    }
}

fn filter_biome(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let checked_re = Regex::new(r"Checked (\d+) files?").unwrap();
    let error_re = Regex::new(r"Found (\d+) errors?").unwrap();
    let warning_re = Regex::new(r"Found (\d+) warnings?").unwrap();

    let checked = checked_re.captures(&combined).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);
    let errors = error_re.captures(&combined).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);
    let warnings = warning_re.captures(&combined).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);

    if errors > 0 || warnings > 0 {
        format!("⚠ {} errors, {} warnings ({} files checked)", errors, warnings, checked)
    } else if checked > 0 {
        format!("✓ {} files checked", checked)
    } else if combined.contains("Fixed") {
        "✓ fixed".to_string()
    } else {
        filter_generic(stdout, stderr)
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

fn shorten_path(path: &str) -> String {
    let path = path.replace("\\", "/");
    if path.len() > 50 {
        if let Some(name) = path.split('/').last() {
            return name.to_string();
        }
    }
    path
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
