//! Java build tools filters (Maven, Gradle).

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for Maven commands.
pub struct MavenFilter;

impl Filter for MavenFilter {
    fn name(&self) -> &'static str {
        "maven"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "mvn" || cmd == "mvn.cmd" || cmd == "mvnw" || cmd == "mvnw.cmd"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_maven(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for Gradle commands.
pub struct GradleFilter;

impl Filter for GradleFilter {
    fn name(&self) -> &'static str {
        "gradle"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "gradle" || cmd == "gradle.bat" || cmd == "gradlew" || cmd == "gradlew.bat"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_gradle(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_maven(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let build_success_re = Regex::new(r"BUILD SUCCESS").unwrap();
    let build_failure_re = Regex::new(r"BUILD FAILURE").unwrap();
    let time_re = Regex::new(r"Total time:\s*([\d.:]+)").unwrap();
    let error_re = Regex::new(r"\[ERROR\]\s*(.+)").unwrap();
    let warning_re = Regex::new(r"\[WARNING\]\s*(.+)").unwrap();
    let test_re = Regex::new(r"Tests run:\s*(\d+),\s*Failures:\s*(\d+),\s*Errors:\s*(\d+),\s*Skipped:\s*(\d+)").unwrap();

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut test_results = Vec::new();
    let mut build_time = String::new();

    for line in combined.lines() {
        if let Some(caps) = error_re.captures(line) {
            let msg = &caps[1];
            if !msg.contains("Help 1") && !msg.contains("->") {
                errors.push(truncate(msg, 70));
            }
        }
        if let Some(caps) = warning_re.captures(line) {
            warnings.push(truncate(&caps[1], 60));
        }
        if let Some(caps) = test_re.captures(line) {
            test_results.push((
                caps[1].parse::<u32>().unwrap_or(0),
                caps[2].parse::<u32>().unwrap_or(0),
                caps[3].parse::<u32>().unwrap_or(0),
                caps[4].parse::<u32>().unwrap_or(0),
            ));
        }
        if let Some(caps) = time_re.captures(line) {
            build_time = caps[1].to_string();
        }
    }

    let mut result = Vec::new();

    if build_failure_re.is_match(&combined) {
        result.push(format!("✗ BUILD FAILURE ({})", build_time));
        for e in errors.iter().take(10) {
            result.push(format!("  {}", e));
        }
        if errors.len() > 10 {
            result.push(format!("  ... +{} more errors", errors.len() - 10));
        }
    } else if build_success_re.is_match(&combined) {
        if !test_results.is_empty() {
            let (total_run, total_fail, total_err, total_skip): (u32, u32, u32, u32) = test_results.iter()
                .fold((0, 0, 0, 0), |acc, x| (acc.0 + x.0, acc.1 + x.1, acc.2 + x.2, acc.3 + x.3));

            if total_fail > 0 || total_err > 0 {
                result.push(format!("✗ {} tests, {} failed, {} errors ({})", total_run, total_fail, total_err, build_time));
            } else {
                result.push(format!("✓ BUILD SUCCESS - {} tests passed ({})", total_run, build_time));
            }
        } else {
            result.push(format!("✓ BUILD SUCCESS ({})", build_time));
        }

        if !warnings.is_empty() {
            result.push(format!("  {} warnings", warnings.len()));
        }
    }

    if result.is_empty() {
        filter_generic(stdout, stderr)
    } else {
        result.join("\n")
    }
}

fn filter_gradle(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let build_success_re = Regex::new(r"BUILD SUCCESSFUL").unwrap();
    let build_failure_re = Regex::new(r"BUILD FAILED").unwrap();
    let time_re = Regex::new(r"in\s*([\d.]+[ms]*s?)").unwrap();
    let task_re = Regex::new(r"> Task :(\S+)").unwrap();
    let error_re = Regex::new(r"^e:\s*(.+)$").unwrap();
    let test_re = Regex::new(r"(\d+) tests completed, (\d+) failed").unwrap();

    let mut tasks = Vec::new();
    let mut errors = Vec::new();
    let mut build_time = String::new();
    let mut test_info = (0u32, 0u32);

    for line in combined.lines() {
        if let Some(caps) = task_re.captures(line) {
            tasks.push(caps[1].to_string());
        }
        if let Some(caps) = error_re.captures(line) {
            errors.push(truncate(&caps[1], 70));
        }
        if let Some(caps) = time_re.captures(line) {
            build_time = caps[1].to_string();
        }
        if let Some(caps) = test_re.captures(line) {
            test_info = (
                caps[1].parse().unwrap_or(0),
                caps[2].parse().unwrap_or(0),
            );
        }
    }

    let mut result = Vec::new();

    if build_failure_re.is_match(&combined) {
        result.push(format!("✗ BUILD FAILED ({})", build_time));
        for e in errors.iter().take(10) {
            result.push(format!("  {}", e));
        }
        if errors.len() > 10 {
            result.push(format!("  ... +{} more errors", errors.len() - 10));
        }
    } else if build_success_re.is_match(&combined) {
        if test_info.0 > 0 {
            if test_info.1 > 0 {
                result.push(format!("✗ {} tests, {} failed ({})", test_info.0, test_info.1, build_time));
            } else {
                result.push(format!("✓ BUILD SUCCESSFUL - {} tests ({}) [{} tasks]", test_info.0, build_time, tasks.len()));
            }
        } else {
            result.push(format!("✓ BUILD SUCCESSFUL ({}) [{} tasks]", build_time, tasks.len()));
        }
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

    // Remove Maven/Gradle noise
    let lines: Vec<&str> = combined.lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() &&
            !t.starts_with("[INFO] ---") &&
            !t.starts_with("[INFO] Building") &&
            !t.starts_with("[INFO] Scanning") &&
            !t.starts_with("Downloading") &&
            !t.starts_with("Downloaded")
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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
