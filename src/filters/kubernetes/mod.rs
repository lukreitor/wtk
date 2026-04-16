//! Kubernetes CLI filters (kubectl, helm).

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for kubectl commands.
pub struct KubectlFilter;

impl Filter for KubectlFilter {
    fn name(&self) -> &'static str {
        "kubectl"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "kubectl" || cmd == "kubectl.exe"
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
            "get" => filter_get(&stdout, &stderr, args),
            "describe" => filter_describe(&stdout),
            "logs" => filter_logs(&stdout, &stderr),
            "apply" => filter_apply(&stdout, &stderr),
            "delete" => filter_delete(&stdout, &stderr),
            "exec" => filter_exec(&stdout, &stderr),
            "port-forward" => filter_port_forward(&stdout, &stderr),
            "rollout" => filter_rollout(&stdout, &stderr),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for Helm commands.
pub struct HelmFilter;

impl Filter for HelmFilter {
    fn name(&self) -> &'static str {
        "helm"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "helm" || cmd == "helm.exe"
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
            "install" => filter_helm_install(&stdout, &stderr),
            "upgrade" => filter_helm_upgrade(&stdout, &stderr),
            "list" => filter_helm_list(&stdout),
            "status" => filter_helm_status(&stdout),
            "uninstall" => filter_helm_uninstall(&stdout, &stderr),
            "template" => filter_helm_template(&stdout),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_get(stdout: &str, stderr: &str, args: &[String]) -> String {
    if !stderr.is_empty() && stdout.is_empty() {
        return filter_generic(stdout, stderr);
    }

    let lines: Vec<&str> = stdout.lines().collect();

    if lines.is_empty() {
        return "No resources found".to_string();
    }

    // Detect resource type from args
    let resource_type = args.get(1).map(|s| s.as_str()).unwrap_or("resources");

    let mut result = Vec::new();
    let count = lines.len() - 1; // exclude header

    result.push(format!("{} {}", count, resource_type));

    // Parse based on resource type
    for line in lines.iter().skip(1).take(15) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            match resource_type {
                "pods" | "pod" | "po" if parts.len() >= 5 => {
                    let name = truncate(parts[0], 35);
                    let ready = parts[1];
                    let status = parts[2];
                    let restarts = parts[3];
                    result.push(format!("  {} {} {} R:{}", name, ready, status, restarts));
                }
                "services" | "service" | "svc" if parts.len() >= 5 => {
                    let name = truncate(parts[0], 25);
                    let svc_type = parts[1];
                    let cluster_ip = parts[2];
                    let ports = parts[4];
                    result.push(format!("  {} {} {} {}", name, svc_type, cluster_ip, truncate(ports, 20)));
                }
                "deployments" | "deployment" | "deploy" if parts.len() >= 4 => {
                    let name = truncate(parts[0], 30);
                    let ready = parts[1];
                    let available = parts[3];
                    result.push(format!("  {} ready:{} avail:{}", name, ready, available));
                }
                "nodes" | "node" if parts.len() >= 5 => {
                    let name = truncate(parts[0], 25);
                    let status = parts[1];
                    let version = parts.last().unwrap_or(&"");
                    result.push(format!("  {} {} {}", name, status, version));
                }
                _ => {
                    let name = truncate(parts[0], 40);
                    let status = parts.get(1).unwrap_or(&"");
                    result.push(format!("  {} {}", name, status));
                }
            }
        }
    }

    if count > 15 {
        result.push(format!("  ... +{} more", count - 15));
    }

    result.join("\n")
}

fn filter_describe(stdout: &str) -> String {
    let name_re = Regex::new(r"^Name:\s*(.+)$").unwrap();
    let ns_re = Regex::new(r"^Namespace:\s*(.+)$").unwrap();
    let status_re = Regex::new(r"^Status:\s*(.+)$").unwrap();
    let event_re = Regex::new(r"^\s*\w+\s+\d+[smh]\s+").unwrap();

    let mut name = String::new();
    let mut namespace = String::new();
    let mut status = String::new();
    let mut events = Vec::new();

    for line in stdout.lines() {
        if let Some(caps) = name_re.captures(line) {
            name = caps[1].to_string();
        }
        if let Some(caps) = ns_re.captures(line) {
            namespace = caps[1].to_string();
        }
        if let Some(caps) = status_re.captures(line) {
            status = caps[1].to_string();
        }
        if event_re.is_match(line) {
            events.push(truncate(line.trim(), 70));
        }
    }

    let mut result = Vec::new();

    if !name.is_empty() {
        result.push(format!("{}/{}", namespace, name));
    }
    if !status.is_empty() {
        result.push(format!("Status: {}", status));
    }

    if !events.is_empty() {
        result.push(format!("\nRecent events ({}):", events.len()));
        for e in events.iter().take(5) {
            result.push(format!("  {}", e));
        }
        if events.len() > 5 {
            result.push(format!("  ... +{} more", events.len() - 5));
        }
    }

    if result.is_empty() {
        if stdout.len() > 500 {
            format!("{}... ({} chars)", &stdout[..500], stdout.len())
        } else {
            stdout.to_string()
        }
    } else {
        result.join("\n")
    }
}

fn filter_logs(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    // Deduplicate similar log lines
    let mut seen: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    for line in combined.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let normalized = normalize_log_line(trimmed);
            *seen.entry(normalized).or_insert(0) += 1;
        }
    }

    let mut result = Vec::new();
    let total_lines = combined.lines().count();

    for (line, count) in seen.iter().take(25) {
        if *count > 1 {
            result.push(format!("{} (x{})", truncate(line, 70), count));
        } else {
            result.push(truncate(line, 80).to_string());
        }
    }

    if seen.len() > 25 {
        result.push(format!("... +{} unique lines ({} total)", seen.len() - 25, total_lines));
    }

    result.join("\n")
}

fn filter_apply(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let created_re = Regex::new(r"(\S+) created").unwrap();
    let configured_re = Regex::new(r"(\S+) configured").unwrap();
    let unchanged_re = Regex::new(r"(\S+) unchanged").unwrap();

    let created: Vec<String> = created_re.captures_iter(&combined).map(|c| c[1].to_string()).collect();
    let configured: Vec<String> = configured_re.captures_iter(&combined).map(|c| c[1].to_string()).collect();
    let unchanged: Vec<String> = unchanged_re.captures_iter(&combined).map(|c| c[1].to_string()).collect();

    let mut result = vec![format!("✓ {} created, {} configured, {} unchanged",
        created.len(), configured.len(), unchanged.len())];

    for c in created.iter().take(5) {
        result.push(format!("  + {}", truncate(c, 50)));
    }
    for c in configured.iter().take(5) {
        result.push(format!("  ~ {}", truncate(c, 50)));
    }

    result.join("\n")
}

fn filter_delete(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let deleted_re = Regex::new(r#""?(\S+)"? deleted"#).unwrap();

    let deleted: Vec<String> = deleted_re.captures_iter(&combined).map(|c| c[1].to_string()).collect();

    if !deleted.is_empty() {
        let mut result = vec![format!("✓ {} deleted", deleted.len())];
        for d in deleted.iter().take(10) {
            result.push(format!("  - {}", truncate(d, 50)));
        }
        if deleted.len() > 10 {
            result.push(format!("  ... +{} more", deleted.len() - 10));
        }
        result.join("\n")
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_exec(stdout: &str, stderr: &str) -> String {
    filter_generic(stdout, stderr)
}

fn filter_port_forward(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let forward_re = Regex::new(r"Forwarding from ([^ ]+) -> (\d+)").unwrap();

    if let Some(caps) = forward_re.captures(&combined) {
        format!("✓ forwarding {} → {}", &caps[1], &caps[2])
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_rollout(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("successfully rolled out") {
        "✓ rollout complete".to_string()
    } else if combined.contains("Waiting") {
        "⏳ rollout in progress...".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

// === Helm filters ===

fn filter_helm_install(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let name_re = Regex::new(r"NAME:\s*(\S+)").unwrap();
    let status_re = Regex::new(r"STATUS:\s*(\S+)").unwrap();

    let name = name_re.captures(&combined).map(|c| c[1].to_string()).unwrap_or_default();
    let status = status_re.captures(&combined).map(|c| c[1].to_string()).unwrap_or_default();

    if !name.is_empty() {
        format!("✓ {} installed ({})", name, status)
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_helm_upgrade(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let name_re = Regex::new(r"NAME:\s*(\S+)").unwrap();
    let rev_re = Regex::new(r"REVISION:\s*(\d+)").unwrap();

    let name = name_re.captures(&combined).map(|c| c[1].to_string()).unwrap_or_default();
    let rev = rev_re.captures(&combined).map(|c| c[1].to_string()).unwrap_or_default();

    if !name.is_empty() {
        format!("✓ {} upgraded to revision {}", name, rev)
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_helm_list(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.len() <= 1 {
        return "No releases".to_string();
    }

    let mut result = vec![format!("{} releases", lines.len() - 1)];

    for line in lines.iter().skip(1).take(10) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let name = truncate(parts[0], 25);
            let ns = parts[1];
            let rev = parts[2];
            let status = parts[4];
            result.push(format!("  {} ({}) rev:{} {}", name, ns, rev, status));
        }
    }

    result.join("\n")
}

fn filter_helm_status(stdout: &str) -> String {
    let name_re = Regex::new(r"NAME:\s*(\S+)").unwrap();
    let ns_re = Regex::new(r"NAMESPACE:\s*(\S+)").unwrap();
    let status_re = Regex::new(r"STATUS:\s*(\S+)").unwrap();
    let rev_re = Regex::new(r"REVISION:\s*(\d+)").unwrap();

    let name = name_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();
    let ns = ns_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();
    let status = status_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();
    let rev = rev_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();

    if !name.is_empty() {
        format!("{} ({}) rev:{} status:{}", name, ns, rev, status)
    } else {
        if stdout.len() > 300 {
            format!("{}...", &stdout[..300])
        } else {
            stdout.to_string()
        }
    }
}

fn filter_helm_uninstall(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let release_re = Regex::new(r#"release "(\S+)" uninstalled"#).unwrap();

    if let Some(caps) = release_re.captures(&combined) {
        format!("✓ {} uninstalled", &caps[1])
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_helm_template(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().collect();
    let resource_re = Regex::new(r"^kind:\s*(\S+)").unwrap();

    let mut resources: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    for line in &lines {
        if let Some(caps) = resource_re.captures(line) {
            *resources.entry(caps[1].to_string()).or_insert(0) += 1;
        }
    }

    let mut result = vec![format!("{} manifests ({} lines)", resources.values().sum::<u32>(), lines.len())];

    for (kind, count) in resources.iter() {
        result.push(format!("  {}: {}", kind, count));
    }

    result.join("\n")
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
    let ts_re = Regex::new(r"^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}[.\d]*Z?\s*").unwrap();
    ts_re.replace(line, "").to_string()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
