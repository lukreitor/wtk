//! Go CLI filter.

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for Go commands.
pub struct GoFilter;

impl Filter for GoFilter {
    fn name(&self) -> &'static str {
        "go"
    }

    fn matches(&self, command: &str) -> bool {
        matches!(command.to_lowercase().as_str(), "go" | "go.exe")
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
            "test" => filter_test(&stdout, &stderr),
            "vet" => filter_vet(&stdout, &stderr),
            "mod" => filter_mod(&stdout, &stderr, args),
            "get" => filter_get(&stdout, &stderr),
            "run" => filter_run(&stdout, &stderr),
            "fmt" => filter_fmt(&stdout, &stderr),
            "install" => filter_install(&stdout, &stderr),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for golangci-lint.
pub struct GolangciLintFilter;

impl Filter for GolangciLintFilter {
    fn name(&self) -> &'static str {
        "golangci-lint"
    }

    fn matches(&self, command: &str) -> bool {
        command.to_lowercase().contains("golangci-lint")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();

        let output = Command::new(command)
            .args(args)
            .output()?;

        let exec_time_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_golangci_lint(&stdout, &stderr);

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_build(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let error_re = Regex::new(r"^(.+?):(\d+):(\d+):\s*(.+)$").unwrap();

    let mut errors = Vec::new();

    for line in combined.lines() {
        if let Some(caps) = error_re.captures(line) {
            let file = shorten_path(&caps[1]);
            let line_num = &caps[2];
            let msg = &caps[4];
            errors.push(format!("{}:{} {}", file, line_num, truncate(msg, 50)));
        }
    }

    if !errors.is_empty() {
        let mut result = vec![format!("✗ {} errors", errors.len())];
        for e in errors.iter().take(10) {
            result.push(format!("  {}", e));
        }
        if errors.len() > 10 {
            result.push(format!("  ... +{} more", errors.len() - 10));
        }
        result.join("\n")
    } else if combined.is_empty() || combined.trim().is_empty() {
        "✓ build succeeded".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_test(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let pass_re = Regex::new(r"ok\s+(\S+)\s+([\d.]+)s").unwrap();
    let fail_re = Regex::new(r"FAIL\s+(\S+)").unwrap();
    let skip_re = Regex::new(r"\[no test files\]").unwrap();

    let mut passed = Vec::new();
    let mut failed = Vec::new();
    let mut skipped = 0;

    for line in combined.lines() {
        if let Some(caps) = pass_re.captures(line) {
            passed.push(format!("{} ({}s)", &caps[1], &caps[2]));
        }
        if let Some(caps) = fail_re.captures(line) {
            failed.push(caps[1].to_string());
        }
        if skip_re.is_match(line) {
            skipped += 1;
        }
    }

    let mut result = Vec::new();

    if !failed.is_empty() {
        result.push(format!("✗ {} failed, {} passed", failed.len(), passed.len()));
        for f in failed.iter().take(10) {
            result.push(format!("  ✗ {}", f));
        }
    } else if !passed.is_empty() {
        result.push(format!("✓ {} passed ({} skipped)", passed.len(), skipped));
    } else if combined.contains("no test files") {
        result.push("⚠ no test files".to_string());
    }

    if result.is_empty() {
        filter_generic(stdout, stderr)
    } else {
        result.join("\n")
    }
}

fn filter_vet(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let issue_re = Regex::new(r"^(.+?):(\d+):(\d+):\s*(.+)$").unwrap();

    let mut issues = Vec::new();

    for line in combined.lines() {
        if let Some(caps) = issue_re.captures(line) {
            let file = shorten_path(&caps[1]);
            let line_num = &caps[2];
            let msg = &caps[4];
            issues.push(format!("{}:{} {}", file, line_num, truncate(msg, 50)));
        }
    }

    if !issues.is_empty() {
        let mut result = vec![format!("⚠ {} issues", issues.len())];
        for i in issues.iter().take(10) {
            result.push(format!("  {}", i));
        }
        if issues.len() > 10 {
            result.push(format!("  ... +{} more", issues.len() - 10));
        }
        result.join("\n")
    } else {
        "✓ vet clean".to_string()
    }
}

fn filter_mod(stdout: &str, stderr: &str, args: &[String]) -> String {
    let combined = format!("{}\n{}", stdout, stderr);
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "tidy" => {
            if combined.is_empty() || combined.trim().is_empty() {
                "✓ go mod tidy".to_string()
            } else {
                filter_generic(stdout, stderr)
            }
        }
        "download" => {
            let download_re = Regex::new(r"go: downloading\s+(\S+)").unwrap();
            let count = download_re.captures_iter(&combined).count();
            if count > 0 {
                format!("✓ downloaded {} modules", count)
            } else {
                "✓ modules up to date".to_string()
            }
        }
        "init" => "✓ go.mod initialized".to_string(),
        "verify" => {
            if combined.contains("all modules verified") {
                "✓ all modules verified".to_string()
            } else {
                filter_generic(stdout, stderr)
            }
        }
        _ => filter_generic(stdout, stderr),
    }
}

fn filter_get(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let get_re = Regex::new(r"go: (downloading|added|upgraded)\s+(\S+)").unwrap();

    let mut actions = Vec::new();
    for caps in get_re.captures_iter(&combined) {
        actions.push(format!("{} {}", &caps[1], &caps[2]));
    }

    if !actions.is_empty() {
        let mut result = vec![format!("✓ {} packages", actions.len())];
        for a in actions.iter().take(5) {
            result.push(format!("  {}", a));
        }
        if actions.len() > 5 {
            result.push(format!("  ... +{} more", actions.len() - 5));
        }
        result.join("\n")
    } else if combined.is_empty() {
        "✓ go get completed".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_run(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let lines: Vec<&str> = combined.lines()
        .filter(|l| !l.trim().is_empty())
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
    } else {
        // gofmt lists files that were changed
        let count = combined.lines().count();
        format!("✓ formatted {} files", count)
    }
}

fn filter_install(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.is_empty() || combined.trim().is_empty() {
        "✓ installed".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_golangci_lint(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let issue_re = Regex::new(r"^(.+?):(\d+):(\d+):\s*(.+?)\s*\((\w+)\)$").unwrap();

    let mut issues_by_linter: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for line in combined.lines() {
        if let Some(caps) = issue_re.captures(line) {
            let file = shorten_path(&caps[1]);
            let line_num = &caps[2];
            let msg = &caps[4];
            let linter = &caps[5];

            issues_by_linter
                .entry(linter.to_string())
                .or_default()
                .push(format!("{}:{} {}", file, line_num, truncate(msg, 40)));
        }
    }

    if issues_by_linter.is_empty() {
        "✓ golangci-lint clean".to_string()
    } else {
        let total: usize = issues_by_linter.values().map(|v| v.len()).sum();
        let mut result = vec![format!("⚠ {} issues ({} linters)", total, issues_by_linter.len())];

        for (linter, issues) in issues_by_linter.iter().take(5) {
            result.push(format!("\n{} ({}):", linter, issues.len()));
            for i in issues.iter().take(3) {
                result.push(format!("  {}", i));
            }
            if issues.len() > 3 {
                result.push(format!("  ... +{} more", issues.len() - 3));
            }
        }

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

fn shorten_path(path: &str) -> String {
    let path = path.replace("\\", "/");
    if path.len() > 40 {
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
