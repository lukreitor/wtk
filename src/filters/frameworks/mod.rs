//! Framework CLI filters (Next.js, Nx, Turbo).

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for Next.js CLI commands.
pub struct NextFilter;

impl Filter for NextFilter {
    fn name(&self) -> &'static str {
        "next"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "next" || cmd.ends_with("next.js") || cmd.contains("next")
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
            "build" => filter_next_build(&stdout, &stderr),
            "dev" => filter_next_dev(&stdout, &stderr),
            "start" => filter_next_start(&stdout, &stderr),
            "lint" => filter_next_lint(&stdout, &stderr),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for Nx CLI commands.
pub struct NxFilter;

impl Filter for NxFilter {
    fn name(&self) -> &'static str {
        "nx"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "nx" || cmd.ends_with("nx.js")
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
            "build" => filter_nx_build(&stdout, &stderr),
            "test" => filter_nx_test(&stdout, &stderr),
            "serve" => filter_nx_serve(&stdout, &stderr),
            "affected" => filter_nx_affected(&stdout, &stderr),
            "graph" => filter_nx_graph(&stdout),
            "run" => filter_nx_run(&stdout, &stderr),
            "run-many" => filter_nx_run_many(&stdout, &stderr),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for Turbo CLI commands.
pub struct TurboFilter;

impl Filter for TurboFilter {
    fn name(&self) -> &'static str {
        "turbo"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "turbo" || cmd == "turbo.exe"
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
            "run" => filter_turbo_run(&stdout, &stderr),
            "build" => filter_turbo_build(&stdout, &stderr),
            "prune" => filter_turbo_prune(&stdout, &stderr),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for Vite CLI commands.
pub struct ViteFilter;

impl Filter for ViteFilter {
    fn name(&self) -> &'static str {
        "vite"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "vite" || cmd.ends_with("vite.js")
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
            "build" => filter_vite_build(&stdout, &stderr),
            "dev" | "" => filter_vite_dev(&stdout, &stderr),
            "preview" => filter_vite_preview(&stdout, &stderr),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

// Next.js filters
fn filter_next_build(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    // Remove ANSI codes
    let ansi_re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    let cleaned = ansi_re.replace_all(&combined, "").to_string();

    let route_re = Regex::new(r"(?m)^[○◐●λ]\s+(/\S+)").unwrap();
    let size_re = Regex::new(r"First Load JS.*?(\d+(?:\.\d+)?\s*[kM]B)").unwrap();
    let time_re = Regex::new(r"in\s+(\d+(?:\.\d+)?m?s)").unwrap();

    let routes: Vec<String> = route_re.captures_iter(&cleaned)
        .map(|c| c[1].to_string())
        .collect();

    let total_size = size_re.captures(&cleaned)
        .map(|c| c[1].to_string())
        .unwrap_or_default();

    let build_time = time_re.captures(&cleaned)
        .map(|c| c[1].to_string())
        .unwrap_or_default();

    if cleaned.contains("Compiled successfully") || cleaned.contains("Build completed") || !routes.is_empty() {
        let mut result = vec!["✓ Build successful".to_string()];

        if !build_time.is_empty() {
            result[0] = format!("✓ Build successful ({})", build_time);
        }

        if !routes.is_empty() {
            result.push(format!("  {} routes", routes.len()));
            for route in routes.iter().take(10) {
                result.push(format!("    {}", route));
            }
            if routes.len() > 10 {
                result.push(format!("    ... +{} more", routes.len() - 10));
            }
        }

        if !total_size.is_empty() {
            result.push(format!("  First Load JS: {}", total_size));
        }

        return result.join("\n");
    }

    if cleaned.contains("Error") || cleaned.contains("Failed") {
        let error_re = Regex::new(r"Error:\s*(.+)").unwrap();
        let errors: Vec<String> = error_re.captures_iter(&cleaned)
            .map(|c| truncate(&c[1], 60))
            .collect();

        let mut result = vec!["✗ Build failed".to_string()];
        for e in errors.iter().take(5) {
            result.push(format!("  {}", e));
        }
        return result.join("\n");
    }

    filter_generic(stdout, stderr)
}

fn filter_next_dev(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let ansi_re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    let cleaned = ansi_re.replace_all(&combined, "").to_string();

    let url_re = Regex::new(r"(https?://localhost:\d+)").unwrap();
    let ready_re = Regex::new(r"Ready in\s+(\d+(?:\.\d+)?m?s)").unwrap();

    if let Some(url_caps) = url_re.captures(&cleaned) {
        let url = &url_caps[1];
        let time = ready_re.captures(&cleaned)
            .map(|c| format!(" ({})", &c[1]))
            .unwrap_or_default();

        return format!("✓ Dev server running: {}{}", url, time);
    }

    if cleaned.contains("Compiling") {
        return "⏳ Compiling...".to_string();
    }

    filter_generic(stdout, stderr)
}

fn filter_next_start(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let url_re = Regex::new(r"(https?://localhost:\d+)").unwrap();

    if let Some(url_caps) = url_re.captures(&combined) {
        return format!("✓ Server running: {}", &url_caps[1]);
    }

    filter_generic(stdout, stderr)
}

fn filter_next_lint(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let ansi_re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    let cleaned = ansi_re.replace_all(&combined, "").to_string();

    if cleaned.contains("No ESLint warnings or errors") {
        return "✓ No lint issues".to_string();
    }

    let error_re = Regex::new(r"(\d+)\s+errors?").unwrap();
    let warning_re = Regex::new(r"(\d+)\s+warnings?").unwrap();

    let errors = error_re.captures(&cleaned).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);
    let warnings = warning_re.captures(&cleaned).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);

    if errors > 0 || warnings > 0 {
        return format!("⚠ {} errors, {} warnings", errors, warnings);
    }

    filter_generic(stdout, stderr)
}

// Nx filters
fn filter_nx_build(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let ansi_re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    let cleaned = ansi_re.replace_all(&combined, "").to_string();

    let success_re = Regex::new(r"Successfully ran target build for project (\S+)").unwrap();
    let time_re = Regex::new(r"in\s+(\d+(?:\.\d+)?m?s)").unwrap();

    if let Some(caps) = success_re.captures(&cleaned) {
        let project = &caps[1];
        let time = time_re.captures(&cleaned)
            .map(|c| format!(" ({})", &c[1]))
            .unwrap_or_default();

        return format!("✓ Built {}{}", project, time);
    }

    if cleaned.contains("Successfully ran target") {
        let time = time_re.captures(&cleaned)
            .map(|c| format!(" ({})", &c[1]))
            .unwrap_or_default();
        return format!("✓ Build complete{}", time);
    }

    if cleaned.contains("Failed") {
        return "✗ Build failed".to_string();
    }

    filter_generic(stdout, stderr)
}

fn filter_nx_test(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let ansi_re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    let cleaned = ansi_re.replace_all(&combined, "").to_string();

    let passed_re = Regex::new(r"(\d+) passed").unwrap();
    let failed_re = Regex::new(r"(\d+) failed").unwrap();
    let time_re = Regex::new(r"in\s+(\d+(?:\.\d+)?m?s)").unwrap();

    let passed = passed_re.captures(&cleaned).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);
    let failed = failed_re.captures(&cleaned).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);
    let time = time_re.captures(&cleaned).map(|c| c[1].to_string()).unwrap_or_default();

    if passed > 0 || failed > 0 {
        if failed > 0 {
            return format!("✗ {} passed, {} failed ({})", passed, failed, time);
        }
        return format!("✓ {} passed ({})", passed, time);
    }

    filter_generic(stdout, stderr)
}

fn filter_nx_serve(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let url_re = Regex::new(r"(https?://localhost:\d+)").unwrap();

    if let Some(url_caps) = url_re.captures(&combined) {
        return format!("✓ Serving: {}", &url_caps[1]);
    }

    if combined.contains("Watching for file changes") {
        return "✓ Server running (watching)".to_string();
    }

    filter_generic(stdout, stderr)
}

fn filter_nx_affected(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let ansi_re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    let cleaned = ansi_re.replace_all(&combined, "").to_string();

    let project_re = Regex::new(r"- (\S+)").unwrap();
    let projects: Vec<String> = project_re.captures_iter(&cleaned)
        .map(|c| c[1].to_string())
        .collect();

    if !projects.is_empty() {
        let mut result = vec![format!("{} affected projects", projects.len())];
        for p in projects.iter().take(15) {
            result.push(format!("  {}", p));
        }
        if projects.len() > 15 {
            result.push(format!("  ... +{} more", projects.len() - 15));
        }
        return result.join("\n");
    }

    if cleaned.contains("No affected projects") {
        return "✓ No affected projects".to_string();
    }

    filter_generic(stdout, stderr)
}

fn filter_nx_graph(stdout: &str) -> String {
    if stdout.contains("Graph generated") || stdout.contains("View") {
        return "✓ Graph generated".to_string();
    }
    filter_generic(stdout, "")
}

fn filter_nx_run(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let ansi_re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    let cleaned = ansi_re.replace_all(&combined, "").to_string();

    if cleaned.contains("Successfully ran") {
        let target_re = Regex::new(r"Successfully ran target (\S+)").unwrap();
        if let Some(caps) = target_re.captures(&cleaned) {
            return format!("✓ Ran {}", &caps[1]);
        }
        return "✓ Run complete".to_string();
    }

    filter_generic(stdout, stderr)
}

fn filter_nx_run_many(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let ansi_re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    let cleaned = ansi_re.replace_all(&combined, "").to_string();

    let success_re = Regex::new(r"Successfully ran targets? .+ for (\d+) projects?").unwrap();
    let failed_re = Regex::new(r"Failed (\d+)").unwrap();

    let succeeded = success_re.captures(&cleaned).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);
    let failed = failed_re.captures(&cleaned).and_then(|c| c[1].parse::<u32>().ok()).unwrap_or(0);

    if succeeded > 0 || failed > 0 {
        if failed > 0 {
            return format!("⚠ {} succeeded, {} failed", succeeded, failed);
        }
        return format!("✓ {} projects succeeded", succeeded);
    }

    filter_generic(stdout, stderr)
}

// Turbo filters
fn filter_turbo_run(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let ansi_re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    let cleaned = ansi_re.replace_all(&combined, "").to_string();

    let tasks_re = Regex::new(r"(\d+) successful, (\d+) total").unwrap();
    let cached_re = Regex::new(r"(\d+) cached").unwrap();
    let time_re = Regex::new(r"Tasks:\s+.+\nDuration:\s+(.+)").unwrap();

    if let Some(caps) = tasks_re.captures(&cleaned) {
        let successful: u32 = caps[1].parse().unwrap_or(0);
        let total: u32 = caps[2].parse().unwrap_or(0);
        let cached = cached_re.captures(&cleaned)
            .and_then(|c| c[1].parse::<u32>().ok())
            .unwrap_or(0);
        let duration = time_re.captures(&cleaned)
            .map(|c| c[1].trim().to_string())
            .unwrap_or_default();

        let mut result = format!("✓ {}/{} tasks", successful, total);
        if cached > 0 {
            result = format!("{} ({} cached)", result, cached);
        }
        if !duration.is_empty() {
            result = format!("{} [{}]", result, duration);
        }
        return result;
    }

    if cleaned.contains("FULL TURBO") {
        return "✓ FULL TURBO - all cached".to_string();
    }

    filter_generic(stdout, stderr)
}

fn filter_turbo_build(stdout: &str, stderr: &str) -> String {
    filter_turbo_run(stdout, stderr)
}

fn filter_turbo_prune(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("Created") || combined.contains("Pruned") {
        return "✓ Pruned monorepo".to_string();
    }

    filter_generic(stdout, stderr)
}

// Vite filters
fn filter_vite_build(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let ansi_re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    let cleaned = ansi_re.replace_all(&combined, "").to_string();

    let chunks_re = Regex::new(r"(\d+) modules transformed").unwrap();
    let time_re = Regex::new(r"built in\s+(\d+(?:\.\d+)?m?s)").unwrap();
    let size_re = Regex::new(r"(\d+(?:\.\d+)?\s*[kM]B)").unwrap();

    if cleaned.contains("built") || cleaned.contains("Build completed") {
        let modules = chunks_re.captures(&cleaned)
            .map(|c| format!("{} modules", &c[1]))
            .unwrap_or_default();
        let time = time_re.captures(&cleaned)
            .map(|c| c[1].to_string())
            .unwrap_or_default();

        let mut result = "✓ Build complete".to_string();
        if !time.is_empty() {
            result = format!("✓ Build complete ({})", time);
        }
        if !modules.is_empty() {
            result = format!("{} - {}", result, modules);
        }
        return result;
    }

    filter_generic(stdout, stderr)
}

fn filter_vite_dev(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let ansi_re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    let cleaned = ansi_re.replace_all(&combined, "").to_string();

    let url_re = Regex::new(r"Local:\s+(https?://[^\s]+)").unwrap();
    let network_re = Regex::new(r"Network:\s+(https?://[^\s]+)").unwrap();

    if let Some(url_caps) = url_re.captures(&cleaned) {
        let local_url = &url_caps[1];
        let network = network_re.captures(&cleaned)
            .map(|c| format!("\n  Network: {}", &c[1]))
            .unwrap_or_default();

        return format!("✓ Dev server\n  Local: {}{}", local_url, network);
    }

    if cleaned.contains("ready in") {
        let time_re = Regex::new(r"ready in\s+(\d+(?:\.\d+)?m?s)").unwrap();
        let time = time_re.captures(&cleaned)
            .map(|c| format!(" ({})", &c[1]))
            .unwrap_or_default();
        return format!("✓ Dev server ready{}", time);
    }

    filter_generic(stdout, stderr)
}

fn filter_vite_preview(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let url_re = Regex::new(r"Local:\s+(https?://[^\s]+)").unwrap();

    if let Some(url_caps) = url_re.captures(&combined) {
        return format!("✓ Preview: {}", &url_caps[1]);
    }

    filter_generic(stdout, stderr)
}

fn filter_generic(stdout: &str, stderr: &str) -> String {
    let combined = if !stderr.is_empty() && stdout.is_empty() {
        stderr.to_string()
    } else if !stderr.is_empty() {
        format!("{}\n{}", stdout, stderr)
    } else {
        stdout.to_string()
    };

    // Remove ANSI codes
    let ansi_re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    let cleaned = ansi_re.replace_all(&combined, "").to_string();

    let lines: Vec<&str> = cleaned.lines()
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
