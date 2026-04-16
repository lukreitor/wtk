//! Python CLI filters (pip, pytest, ruff, mypy, etc.).

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for pip commands.
pub struct PipFilter;

impl Filter for PipFilter {
    fn name(&self) -> &'static str {
        "pip"
    }

    fn matches(&self, command: &str) -> bool {
        matches!(command.to_lowercase().as_str(),
            "pip" | "pip.exe" | "pip3" | "pip3.exe")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let subcommand = args.first().map(|s| s.as_str()).unwrap_or("");

        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = match subcommand {
            "install" => filter_pip_install(&stdout, &stderr),
            "list" => filter_pip_list(&stdout),
            "freeze" => filter_pip_freeze(&stdout),
            "show" => filter_pip_show(&stdout),
            "check" => filter_pip_check(&stdout, &stderr),
            "uninstall" => filter_pip_uninstall(&stdout, &stderr),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for pytest.
pub struct PytestFilter;

impl Filter for PytestFilter {
    fn name(&self) -> &'static str {
        "pytest"
    }

    fn matches(&self, command: &str) -> bool {
        command.to_lowercase().contains("pytest")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_pytest(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for ruff.
pub struct RuffFilter;

impl Filter for RuffFilter {
    fn name(&self) -> &'static str {
        "ruff"
    }

    fn matches(&self, command: &str) -> bool {
        command.to_lowercase().contains("ruff")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_ruff(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for mypy.
pub struct MypyFilter;

impl Filter for MypyFilter {
    fn name(&self) -> &'static str {
        "mypy"
    }

    fn matches(&self, command: &str) -> bool {
        command.to_lowercase().contains("mypy")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_mypy(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for poetry.
pub struct PoetryFilter;

impl Filter for PoetryFilter {
    fn name(&self) -> &'static str {
        "poetry"
    }

    fn matches(&self, command: &str) -> bool {
        command.to_lowercase().contains("poetry")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let subcommand = args.first().map(|s| s.as_str()).unwrap_or("");

        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = match subcommand {
            "install" => filter_poetry_install(&stdout, &stderr),
            "add" => filter_poetry_add(&stdout, &stderr),
            "update" => filter_poetry_update(&stdout, &stderr),
            "show" => filter_poetry_show(&stdout),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

// === Pip filters ===

fn filter_pip_install(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let installed_re = Regex::new(r"Successfully installed (.+)").unwrap();
    let already_re = Regex::new(r"Requirement already satisfied").unwrap();

    if let Some(caps) = installed_re.captures(&combined) {
        let packages: Vec<&str> = caps[1].split_whitespace().collect();
        let mut result = vec![format!("✓ installed {} packages", packages.len())];
        for p in packages.iter().take(5) {
            result.push(format!("  {}", p));
        }
        if packages.len() > 5 {
            result.push(format!("  ... +{} more", packages.len() - 5));
        }
        result.join("\n")
    } else if already_re.is_match(&combined) {
        "✓ requirements already satisfied".to_string()
    } else if combined.contains("ERROR") {
        filter_generic(stdout, stderr)
    } else {
        "✓ pip install completed".to_string()
    }
}

fn filter_pip_list(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.is_empty() && !l.starts_with("Package") && !l.starts_with("---"))
        .collect();

    if lines.len() > 20 {
        let mut result = vec![format!("{} packages installed", lines.len())];
        for l in lines.iter().take(15) {
            result.push(l.to_string());
        }
        result.push(format!("... +{} more", lines.len() - 15));
        result.join("\n")
    } else {
        format!("{} packages installed\n{}", lines.len(), lines.join("\n"))
    }
}

fn filter_pip_freeze(stdout: &str) -> String {
    let count = stdout.lines().filter(|l| !l.is_empty()).count();
    format!("{} packages frozen", count)
}

fn filter_pip_show(stdout: &str) -> String {
    let name_re = Regex::new(r"Name: (.+)").unwrap();
    let version_re = Regex::new(r"Version: (.+)").unwrap();
    let summary_re = Regex::new(r"Summary: (.+)").unwrap();

    let name = name_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();
    let version = version_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();
    let summary = summary_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();

    format!("{} v{}\n{}", name, version, truncate(&summary, 60))
}

fn filter_pip_check(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("No broken requirements") {
        "✓ no broken requirements".to_string()
    } else {
        let issues = combined.lines().filter(|l| l.contains("has requirement")).count();
        format!("⚠ {} dependency issues", issues)
    }
}

fn filter_pip_uninstall(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let removed_re = Regex::new(r"Successfully uninstalled (.+)").unwrap();

    if let Some(caps) = removed_re.captures(&combined) {
        format!("✓ uninstalled {}", &caps[1])
    } else {
        filter_generic(stdout, stderr)
    }
}

// === Pytest filter ===

fn filter_pytest(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let summary_re = Regex::new(r"(\d+) passed").unwrap();
    let failed_re = Regex::new(r"(\d+) failed").unwrap();
    let skipped_re = Regex::new(r"(\d+) skipped").unwrap();
    let error_re = Regex::new(r"(\d+) error").unwrap();
    let time_re = Regex::new(r"in ([\d.]+)s").unwrap();
    let failed_test_re = Regex::new(r"FAILED (.+?) -").unwrap();

    let passed = summary_re.captures(&combined).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);
    let failed = failed_re.captures(&combined).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);
    let skipped = skipped_re.captures(&combined).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);
    let errors = error_re.captures(&combined).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);
    let time = time_re.captures(&combined).map(|c| c[1].to_string()).unwrap_or_default();

    let mut result = Vec::new();

    if failed > 0 || errors > 0 {
        result.push(format!("✗ {} passed, {} failed, {} errors ({}s)", passed, failed, errors, time));

        // List failed tests
        let mut failed_tests: Vec<String> = Vec::new();
        for caps in failed_test_re.captures_iter(&combined) {
            failed_tests.push(caps[1].to_string());
        }

        for t in failed_tests.iter().take(10) {
            result.push(format!("  ✗ {}", shorten_test_name(t)));
        }
        if failed_tests.len() > 10 {
            result.push(format!("  ... +{} more", failed_tests.len() - 10));
        }
    } else if passed > 0 {
        result.push(format!("✓ {} passed, {} skipped ({}s)", passed, skipped, time));
    } else if combined.contains("no tests ran") {
        result.push("⚠ no tests ran".to_string());
    }

    if result.is_empty() {
        filter_generic(stdout, stderr)
    } else {
        result.join("\n")
    }
}

// === Ruff filter ===

fn filter_ruff(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let issue_re = Regex::new(r"^(.+?):(\d+):(\d+):\s*(\w+)\s+(.+)$").unwrap();

    let mut issues_by_code: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for line in combined.lines() {
        if let Some(caps) = issue_re.captures(line) {
            let file = shorten_path(&caps[1]);
            let line_num = &caps[2];
            let code = &caps[4];
            let msg = &caps[5];

            issues_by_code
                .entry(code.to_string())
                .or_default()
                .push(format!("{}:{} {}", file, line_num, truncate(msg, 40)));
        }
    }

    if issues_by_code.is_empty() {
        if combined.contains("All checks passed") || combined.trim().is_empty() {
            "✓ ruff clean".to_string()
        } else {
            filter_generic(stdout, stderr)
        }
    } else {
        let total: usize = issues_by_code.values().map(|v| v.len()).sum();
        let mut result = vec![format!("⚠ {} issues ({} rules)", total, issues_by_code.len())];

        for (code, issues) in issues_by_code.iter().take(5) {
            result.push(format!("\n{} ({}):", code, issues.len()));
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

// === Mypy filter ===

fn filter_mypy(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let error_re = Regex::new(r"^(.+?):(\d+):\s*error:\s*(.+)$").unwrap();
    let note_re = Regex::new(r"Found (\d+) error").unwrap();

    let mut errors = Vec::new();

    for line in combined.lines() {
        if let Some(caps) = error_re.captures(line) {
            let file = shorten_path(&caps[1]);
            let line_num = &caps[2];
            let msg = &caps[3];
            errors.push(format!("{}:{} {}", file, line_num, truncate(msg, 45)));
        }
    }

    if errors.is_empty() {
        if combined.contains("Success") {
            "✓ mypy clean".to_string()
        } else {
            filter_generic(stdout, stderr)
        }
    } else {
        let mut result = vec![format!("✗ {} type errors", errors.len())];
        for e in errors.iter().take(10) {
            result.push(format!("  {}", e));
        }
        if errors.len() > 10 {
            result.push(format!("  ... +{} more", errors.len() - 10));
        }
        result.join("\n")
    }
}

// === Poetry filters ===

fn filter_poetry_install(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let install_re = Regex::new(r"Installing\s+(\S+)\s+\(").unwrap();
    let update_re = Regex::new(r"Updating\s+(\S+)").unwrap();

    let installs: Vec<String> = install_re.captures_iter(&combined)
        .map(|c| c[1].to_string())
        .collect();
    let updates: Vec<String> = update_re.captures_iter(&combined)
        .map(|c| c[1].to_string())
        .collect();

    if !installs.is_empty() || !updates.is_empty() {
        format!("✓ {} installed, {} updated", installs.len(), updates.len())
    } else if combined.contains("No dependencies") || combined.contains("Installing dependencies") {
        "✓ dependencies installed".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_poetry_add(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let adding_re = Regex::new(r"Using version (.+) for (.+)").unwrap();

    if let Some(caps) = adding_re.captures(&combined) {
        format!("✓ added {} {}", &caps[2], &caps[1])
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_poetry_update(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let update_re = Regex::new(r"Updating\s+(\S+)\s+\((.+)\s+->\s+(.+)\)").unwrap();

    let mut updates = Vec::new();
    for caps in update_re.captures_iter(&combined) {
        updates.push(format!("{} {} → {}", &caps[1], &caps[2], &caps[3]));
    }

    if !updates.is_empty() {
        let mut result = vec![format!("✓ {} packages updated", updates.len())];
        for u in updates.iter().take(5) {
            result.push(format!("  {}", u));
        }
        if updates.len() > 5 {
            result.push(format!("  ... +{} more", updates.len() - 5));
        }
        result.join("\n")
    } else {
        "✓ dependencies up to date".to_string()
    }
}

fn filter_poetry_show(stdout: &str) -> String {
    let count = stdout.lines().filter(|l| !l.is_empty()).count();
    format!("{} packages", count)
}

// === Helpers ===

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

fn shorten_test_name(name: &str) -> String {
    if name.len() > 60 {
        if let Some(test) = name.split("::").last() {
            return test.to_string();
        }
    }
    name.to_string()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
