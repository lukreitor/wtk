//! Rust/Cargo CLI filter.

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for Cargo commands.
pub struct CargoFilter;

impl Filter for CargoFilter {
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn matches(&self, command: &str) -> bool {
        matches!(command.to_lowercase().as_str(), "cargo" | "cargo.exe")
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
            "build" => filter_build(&stdout, &stderr),
            "check" => filter_check(&stdout, &stderr),
            "test" => filter_test(&stdout, &stderr),
            "clippy" => filter_clippy(&stdout, &stderr),
            "run" => filter_run(&stdout, &stderr),
            "fmt" => filter_fmt(&stdout, &stderr),
            "doc" => filter_doc(&stdout, &stderr),
            "clean" => "✓ cleaned".to_string(),
            "update" => filter_update(&stdout, &stderr),
            "install" => filter_install(&stdout, &stderr),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_build(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let error_re = Regex::new(r"error\[E\d+\]:\s*(.+)").unwrap();
    let warning_re = Regex::new(r"warning:\s*(.+)").unwrap();
    let compiling_re = Regex::new(r"Compiling\s+(\S+)\s+v").unwrap();
    let finished_re = Regex::new(r"Finished\s+`?(\w+)`?\s+.*in\s+([\d.]+)s").unwrap();

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut compiled = Vec::new();
    let mut finish_info = String::new();

    for line in combined.lines() {
        if let Some(caps) = error_re.captures(line) {
            errors.push(caps[1].to_string());
        }
        if let Some(caps) = warning_re.captures(line) {
            let msg = &caps[1];
            if !msg.contains("generated") && !msg.contains("warnings") {
                warnings.push(msg.to_string());
            }
        }
        if let Some(caps) = compiling_re.captures(line) {
            compiled.push(caps[1].to_string());
        }
        if let Some(caps) = finished_re.captures(line) {
            finish_info = format!("{} in {}s", &caps[1], &caps[2]);
        }
    }

    let mut result = Vec::new();

    if !errors.is_empty() {
        result.push(format!("✗ {} errors", errors.len()));
        for e in errors.iter().take(10) {
            result.push(format!("  {}", truncate(e, 70)));
        }
        if errors.len() > 10 {
            result.push(format!("  ... +{} more", errors.len() - 10));
        }
    } else if !warnings.is_empty() {
        result.push(format!("✓ built ({}) - {} warnings", finish_info, warnings.len()));
        for w in warnings.iter().take(5) {
            result.push(format!("  ⚠ {}", truncate(w, 60)));
        }
        if warnings.len() > 5 {
            result.push(format!("  ... +{} more warnings", warnings.len() - 5));
        }
    } else if !finish_info.is_empty() {
        result.push(format!("✓ built {} ({} crates)", finish_info, compiled.len()));
    } else if combined.contains("Compiling") {
        result.push(format!("✓ compiled {} crates", compiled.len()));
    }

    if result.is_empty() {
        filter_generic(stdout, stderr)
    } else {
        result.join("\n")
    }
}

fn filter_check(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("error[E") {
        filter_build(stdout, stderr)
    } else if combined.contains("warning:") {
        let warning_count = combined.matches("warning:").count();
        format!("✓ check passed ({} warnings)", warning_count)
    } else if combined.contains("Finished") {
        "✓ check passed".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_test(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let result_re = Regex::new(r"test result: (\w+)\.\s*(\d+) passed;\s*(\d+) failed;\s*(\d+) ignored").unwrap();
    let failed_re = Regex::new(r"---- (\S+) stdout ----").unwrap();

    let mut failed_tests = Vec::new();
    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_ignored = 0;

    for line in combined.lines() {
        if let Some(caps) = result_re.captures(line) {
            total_passed += caps[2].parse::<u32>().unwrap_or(0);
            total_failed += caps[3].parse::<u32>().unwrap_or(0);
            total_ignored += caps[4].parse::<u32>().unwrap_or(0);
        }
        if let Some(caps) = failed_re.captures(line) {
            failed_tests.push(caps[1].to_string());
        }
    }

    let mut result = Vec::new();

    if total_failed > 0 {
        result.push(format!("✗ {} passed, {} failed, {} ignored", total_passed, total_failed, total_ignored));
        for t in failed_tests.iter().take(10) {
            result.push(format!("  ✗ {}", t));
        }
        if failed_tests.len() > 10 {
            result.push(format!("  ... +{} more", failed_tests.len() - 10));
        }
    } else if total_passed > 0 {
        result.push(format!("✓ {} passed, {} ignored", total_passed, total_ignored));
    } else if combined.contains("running 0 tests") {
        result.push("⚠ no tests found".to_string());
    }

    if result.is_empty() {
        filter_generic(stdout, stderr)
    } else {
        result.join("\n")
    }
}

fn filter_clippy(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let warning_re = Regex::new(r"warning: (.+)").unwrap();
    let error_re = Regex::new(r"error\[E\d+\]: (.+)").unwrap();
    let help_re = Regex::new(r"help: (.+)").unwrap();

    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    for line in combined.lines() {
        if let Some(caps) = error_re.captures(line) {
            errors.push(caps[1].to_string());
        } else if let Some(caps) = warning_re.captures(line) {
            let msg = &caps[1];
            if !msg.contains("generated") && !msg.contains("warnings") {
                warnings.push(msg.to_string());
            }
        }
    }

    let mut result = Vec::new();

    if !errors.is_empty() {
        result.push(format!("✗ {} errors, {} warnings", errors.len(), warnings.len()));
        for e in errors.iter().take(5) {
            result.push(format!("  ✗ {}", truncate(e, 60)));
        }
    } else if !warnings.is_empty() {
        result.push(format!("⚠ {} clippy warnings", warnings.len()));
        for w in warnings.iter().take(10) {
            result.push(format!("  {}", truncate(w, 65)));
        }
        if warnings.len() > 10 {
            result.push(format!("  ... +{} more", warnings.len() - 10));
        }
    } else {
        result.push("✓ clippy clean".to_string());
    }

    result.join("\n")
}

fn filter_run(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    // Remove compilation output, keep program output
    let lines: Vec<&str> = combined.lines()
        .filter(|l| {
            !l.trim().starts_with("Compiling") &&
            !l.trim().starts_with("Finished") &&
            !l.trim().starts_with("Running") &&
            !l.trim().is_empty()
        })
        .collect();

    if lines.len() > 30 {
        let mut result: Vec<String> = lines.iter().take(25).map(|s| s.to_string()).collect();
        result.push(format!("... +{} more lines", lines.len() - 25));
        result.join("\n")
    } else {
        lines.join("\n")
    }
}

fn filter_fmt(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.is_empty() || combined.trim().is_empty() {
        "✓ formatted".to_string()
    } else if combined.contains("Diff in") {
        let count = combined.matches("Diff in").count();
        format!("⚠ {} files need formatting", count)
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_doc(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("Finished") {
        let doc_re = Regex::new(r"Documenting\s+(\S+)").unwrap();
        let count = doc_re.captures_iter(&combined).count();
        format!("✓ documented {} crates", count)
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_update(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let update_re = Regex::new(r"Updating\s+(\S+)\s+v([\d.]+)\s+->\s+v([\d.]+)").unwrap();

    let mut updates = Vec::new();
    for caps in update_re.captures_iter(&combined) {
        updates.push(format!("{} {} → {}", &caps[1], &caps[2], &caps[3]));
    }

    if !updates.is_empty() {
        let mut result = vec![format!("✓ {} updates", updates.len())];
        for u in updates.iter().take(10) {
            result.push(format!("  {}", u));
        }
        if updates.len() > 10 {
            result.push(format!("  ... +{} more", updates.len() - 10));
        }
        result.join("\n")
    } else if combined.contains("Unchanged") || combined.is_empty() {
        "✓ dependencies up to date".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_install(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let installed_re = Regex::new(r"Installed package `(\S+) v([\d.]+)`").unwrap();

    if let Some(caps) = installed_re.captures(&combined) {
        format!("✓ installed {} v{}", &caps[1], &caps[2])
    } else if combined.contains("already") {
        "✓ already installed".to_string()
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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
