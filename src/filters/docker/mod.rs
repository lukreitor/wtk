//! Docker and docker-compose filters.

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for Docker commands.
pub struct DockerFilter;

impl Filter for DockerFilter {
    fn name(&self) -> &'static str {
        "docker"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "docker" || cmd == "docker.exe"
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
            "ps" => filter_ps(&stdout),
            "images" => filter_images(&stdout),
            "logs" => filter_logs(&stdout, &stderr),
            "build" => filter_build(&stdout, &stderr),
            "run" => filter_run(&stdout, &stderr),
            "exec" => filter_exec(&stdout, &stderr),
            "inspect" => filter_inspect(&stdout),
            "stats" => filter_stats(&stdout),
            "pull" => filter_pull(&stdout, &stderr),
            "push" => filter_push(&stdout, &stderr),
            "compose" => filter_compose(&stdout, &stderr, args),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for docker-compose commands.
pub struct DockerComposeFilter;

impl Filter for DockerComposeFilter {
    fn name(&self) -> &'static str {
        "docker-compose"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "docker-compose" || cmd == "docker-compose.exe"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_compose(&stdout, &stderr, args);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_ps(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.len() <= 1 {
        return "No containers running".to_string();
    }

    let mut result = Vec::new();
    let container_count = lines.len() - 1;
    result.push(format!("{} containers", container_count));

    for line in lines.iter().skip(1).take(10) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 7 {
            let id = &parts[0][..12.min(parts[0].len())];
            let image = truncate(parts[1], 25);
            let status = parts[4..].join(" ");
            let status_short = if status.contains("Up") { "Up" } else { "Exited" };
            result.push(format!("  {} {} ({})", id, image, status_short));
        }
    }

    if container_count > 10 {
        result.push(format!("  ... +{} more", container_count - 10));
    }

    result.join("\n")
}

fn filter_images(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.len() <= 1 {
        return "No images".to_string();
    }

    let mut result = Vec::new();
    let image_count = lines.len() - 1;
    result.push(format!("{} images", image_count));

    for line in lines.iter().skip(1).take(10) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let repo = truncate(parts[0], 30);
            let tag = parts[1];
            let size = parts.last().unwrap_or(&"");
            result.push(format!("  {}:{} ({})", repo, tag, size));
        }
    }

    if image_count > 10 {
        result.push(format!("  ... +{} more", image_count - 10));
    }

    result.join("\n")
}

fn filter_logs(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    // Deduplicate similar log lines
    let mut seen: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    for line in combined.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            // Normalize timestamps and IDs for dedup
            let normalized = normalize_log_line(trimmed);
            *seen.entry(normalized).or_insert(0) += 1;
        }
    }

    let mut result = Vec::new();
    let total_lines = combined.lines().count();

    for (line, count) in seen.iter().take(20) {
        if *count > 1 {
            result.push(format!("{} (x{})", truncate(line, 70), count));
        } else {
            result.push(truncate(line, 80).to_string());
        }
    }

    if seen.len() > 20 {
        result.push(format!("... +{} more unique lines ({} total)", seen.len() - 20, total_lines));
    }

    result.join("\n")
}

fn filter_build(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let step_re = Regex::new(r"Step (\d+)/(\d+)").unwrap();
    let success_re = Regex::new(r"Successfully built ([a-f0-9]+)").unwrap();
    let tagged_re = Regex::new(r"Successfully tagged (.+)").unwrap();
    let error_re = Regex::new(r"error|Error|ERROR").unwrap();

    let mut total_steps = 0;
    let mut current_step = 0;
    let mut image_id = String::new();
    let mut image_tag = String::new();
    let mut has_error = false;

    for line in combined.lines() {
        if let Some(caps) = step_re.captures(line) {
            current_step = caps[1].parse().unwrap_or(0);
            total_steps = caps[2].parse().unwrap_or(0);
        }
        if let Some(caps) = success_re.captures(line) {
            image_id = caps[1][..12.min(caps[1].len())].to_string();
        }
        if let Some(caps) = tagged_re.captures(line) {
            image_tag = caps[1].to_string();
        }
        if error_re.is_match(line) && !line.contains("errorhandling") {
            has_error = true;
        }
    }

    if has_error {
        format!("✗ build failed at step {}/{}", current_step, total_steps)
    } else if !image_id.is_empty() {
        if !image_tag.is_empty() {
            format!("✓ built {} ({}) [{} steps]", image_tag, image_id, total_steps)
        } else {
            format!("✓ built {} [{} steps]", image_id, total_steps)
        }
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_run(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    // Check if it's a container ID output
    if combined.trim().len() == 64 && combined.trim().chars().all(|c| c.is_ascii_hexdigit()) {
        let id = &combined.trim()[..12];
        return format!("✓ container started: {}", id);
    }

    filter_generic(stdout, stderr)
}

fn filter_exec(stdout: &str, stderr: &str) -> String {
    filter_generic(stdout, stderr)
}

fn filter_inspect(stdout: &str) -> String {
    // For inspect, extract key info from JSON
    let state_re = Regex::new(r#""Status":\s*"(\w+)""#).unwrap();
    let name_re = Regex::new(r#""Name":\s*"/([^"]+)""#).unwrap();
    let ip_re = Regex::new(r#""IPAddress":\s*"([^"]+)""#).unwrap();

    let state = state_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();
    let name = name_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();
    let ip = ip_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();

    if !name.is_empty() {
        let mut result = vec![format!("Container: {}", name)];
        if !state.is_empty() {
            result.push(format!("  Status: {}", state));
        }
        if !ip.is_empty() {
            result.push(format!("  IP: {}", ip));
        }
        result.join("\n")
    } else {
        // Truncate raw JSON
        if stdout.len() > 500 {
            format!("{}... ({} chars)", &stdout[..500], stdout.len())
        } else {
            stdout.to_string()
        }
    }
}

fn filter_stats(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.len() <= 1 {
        return "No running containers".to_string();
    }

    let mut result = Vec::new();
    result.push(format!("{} containers", lines.len() - 1));

    for line in lines.iter().skip(1).take(10) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 7 {
            let name = truncate(parts[1], 20);
            let cpu = parts[2];
            let mem = parts[6];
            result.push(format!("  {} CPU:{} MEM:{}", name, cpu, mem));
        }
    }

    result.join("\n")
}

fn filter_pull(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let digest_re = Regex::new(r"Digest: sha256:([a-f0-9]+)").unwrap();
    let status_re = Regex::new(r"Status: (.+)").unwrap();

    let digest = digest_re.captures(&combined).map(|c| c[1][..12].to_string()).unwrap_or_default();
    let status = status_re.captures(&combined).map(|c| c[1].to_string()).unwrap_or_default();

    if !status.is_empty() {
        if !digest.is_empty() {
            format!("✓ {} ({})", status, digest)
        } else {
            format!("✓ {}", status)
        }
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_push(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let digest_re = Regex::new(r"digest: sha256:([a-f0-9]+)").unwrap();

    if let Some(caps) = digest_re.captures(&combined) {
        format!("✓ pushed (sha256:{})", &caps[1][..12])
    } else if combined.contains("Pushed") {
        "✓ pushed".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_compose(stdout: &str, stderr: &str, args: &[String]) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let subcommand = args.iter()
        .skip_while(|a| a.starts_with('-') || *a == "compose")
        .next()
        .map(|s| s.as_str())
        .unwrap_or("");

    match subcommand {
        "up" => {
            let created_re = Regex::new(r"Creating (\S+)").unwrap();
            let started_re = Regex::new(r"Started (\S+)").unwrap();

            let created: Vec<String> = created_re.captures_iter(&combined)
                .map(|c| c[1].to_string())
                .collect();
            let started: Vec<String> = started_re.captures_iter(&combined)
                .map(|c| c[1].to_string())
                .collect();

            if !created.is_empty() || !started.is_empty() {
                format!("✓ {} created, {} started", created.len(), started.len())
            } else if combined.contains("Running") {
                "✓ services running".to_string()
            } else {
                filter_generic(stdout, stderr)
            }
        }
        "down" => {
            let stopped_re = Regex::new(r"Stopped|Stopping").unwrap();
            let removed_re = Regex::new(r"Removed|Removing").unwrap();

            let stopped = stopped_re.captures_iter(&combined).count();
            let removed = removed_re.captures_iter(&combined).count();

            format!("✓ {} stopped, {} removed", stopped, removed)
        }
        "ps" => filter_ps(stdout),
        "logs" => filter_logs(stdout, stderr),
        "build" => filter_build(stdout, stderr),
        _ => filter_generic(stdout, stderr),
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

fn normalize_log_line(line: &str) -> String {
    // Remove timestamps and container IDs for dedup
    let ts_re = Regex::new(r"^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}[.\d]*Z?\s*").unwrap();
    let id_re = Regex::new(r"[a-f0-9]{12,}").unwrap();

    let mut normalized = ts_re.replace(line, "").to_string();
    normalized = id_re.replace_all(&normalized, "ID").to_string();

    normalized
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
