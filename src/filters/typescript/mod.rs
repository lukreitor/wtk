//! TypeScript compiler filter (tsc).

use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for TypeScript compiler output.
pub struct TscFilter;

impl Filter for TscFilter {
    fn name(&self) -> &'static str {
        "tsc"
    }

    fn matches(&self, command: &str) -> bool {
        matches!(command.to_lowercase().as_str(), "tsc" | "tsc.cmd" | "tsc.exe" | "npx")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        // For npx, check if it's running tsc
        if command == "npx" {
            if !args.first().map(|a| a.contains("tsc")).unwrap_or(false) {
                anyhow::bail!("Not a tsc command");
            }
        }

        let start = Instant::now();

        let output = Command::new(command)
            .args(args)
            .output()?;

        let exec_time_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = format!("{}\n{}", stdout, stderr);
        let input_chars = combined.len();

        let filtered = filter_tsc_output(&combined);

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_tsc_output(output: &str) -> String {
    // Error format: file(line,col): error TSxxxx: message
    let error_re = Regex::new(r"^(.+?)\((\d+),(\d+)\):\s*(error|warning)\s+(TS\d+):\s*(.+)$").unwrap();

    let mut errors_by_file: HashMap<String, Vec<TscError>> = HashMap::new();
    let mut error_count = 0;
    let mut warning_count = 0;

    for line in output.lines() {
        let trimmed = line.trim();

        if let Some(caps) = error_re.captures(trimmed) {
            let file = caps[1].to_string();
            let line_num: u32 = caps[2].parse().unwrap_or(0);
            let col: u32 = caps[3].parse().unwrap_or(0);
            let severity = &caps[4];
            let code = caps[5].to_string();
            let message = caps[6].to_string();

            if severity == "error" {
                error_count += 1;
            } else {
                warning_count += 1;
            }

            let err = TscError {
                line: line_num,
                col,
                code,
                message,
                is_error: severity == "error",
            };

            errors_by_file
                .entry(shorten_path(&file))
                .or_default()
                .push(err);
        }
    }

    // Build compact output
    let mut result = Vec::new();

    if error_count == 0 && warning_count == 0 {
        // Check for success message
        if output.contains("Successfully") || output.is_empty() {
            return "✓ compiled successfully".to_string();
        }
        // Unknown output, truncate
        let lines: Vec<&str> = output.lines().take(10).collect();
        return lines.join("\n");
    }

    // Summary
    if error_count > 0 {
        result.push(format!("✗ {} errors, {} warnings", error_count, warning_count));
    } else {
        result.push(format!("⚠ {} warnings", warning_count));
    }

    // Group by file
    let mut files: Vec<_> = errors_by_file.into_iter().collect();
    files.sort_by(|a, b| {
        // Sort by error count (descending)
        let a_errs = a.1.iter().filter(|e| e.is_error).count();
        let b_errs = b.1.iter().filter(|e| e.is_error).count();
        b_errs.cmp(&a_errs)
    });

    // Show top files with errors
    let mut shown_errors = 0;
    for (file, errors) in files.iter().take(5) {
        let err_count = errors.iter().filter(|e| e.is_error).count();
        let warn_count = errors.len() - err_count;

        result.push(format!("\n{} ({} err, {} warn)", file, err_count, warn_count));

        // Group by error code
        let mut by_code: HashMap<&str, Vec<&TscError>> = HashMap::new();
        for err in errors {
            by_code.entry(&err.code).or_default().push(err);
        }

        for (code, code_errors) in by_code.iter().take(3) {
            if shown_errors >= 15 {
                break;
            }

            let first = code_errors[0];
            if code_errors.len() > 1 {
                result.push(format!("  {}: {} (x{}) L{}",
                    code,
                    truncate_msg(&first.message, 40),
                    code_errors.len(),
                    first.line
                ));
            } else {
                result.push(format!("  {}: {} L{}",
                    code,
                    truncate_msg(&first.message, 45),
                    first.line
                ));
            }
            shown_errors += 1;
        }

        if by_code.len() > 3 {
            result.push(format!("  ... +{} more error types", by_code.len() - 3));
        }
    }

    if files.len() > 5 {
        result.push(format!("\n... +{} more files with errors", files.len() - 5));
    }

    result.join("\n")
}

struct TscError {
    line: u32,
    col: u32,
    code: String,
    message: String,
    is_error: bool,
}

fn shorten_path(path: &str) -> String {
    // Remove common prefixes and shorten
    let path = path.replace("\\", "/");

    // Remove node_modules prefix if present
    if let Some(idx) = path.find("node_modules/") {
        return format!("nm/{}", &path[idx + 13..]);
    }

    // Remove src/ prefix
    if let Some(idx) = path.find("src/") {
        return path[idx..].to_string();
    }

    // If path is too long, show just the filename
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
