//! Ansible filter - compresses ansible-playbook and ansible output.

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use crate::filters::traits::{Filter, FilterResult};

/// Filter for ansible commands.
pub struct AnsibleFilter;

impl Filter for AnsibleFilter {
    fn name(&self) -> &'static str {
        "ansible"
    }

    fn matches(&self, command: &str) -> bool {
        matches!(command, "ansible" | "ansible-playbook" | "ansible-galaxy" | "ansible-vault")
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();

        let output = Command::new(command)
            .args(args)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        let filtered = match command {
            "ansible-playbook" => filter_playbook_output(&combined),
            "ansible-galaxy" => filter_galaxy_output(&combined, args),
            "ansible-vault" => filter_vault_output(&combined),
            _ => filter_ansible_output(&combined, args),
        };

        let exec_time = start.elapsed().as_millis() as u64;

        Ok(FilterResult::with_raw(filtered, combined, exec_time))
    }

    fn priority(&self) -> u8 {
        60
    }
}

fn filter_playbook_output(output: &str) -> String {
    let mut lines = Vec::new();
    let mut play_name = String::new();
    let mut task_results: Vec<(String, String, String)> = Vec::new(); // (task, status, host)
    let mut recap_section = false;
    let mut recap_lines = Vec::new();

    let play_re = Regex::new(r"PLAY \[(.+?)\]").unwrap();
    let task_re = Regex::new(r"TASK \[(.+?)\]").unwrap();
    let status_re = Regex::new(r"^(ok|changed|failed|skipping|fatal|unreachable):\s*\[(.+?)\]").unwrap();
    let recap_re = Regex::new(r"^(.+?)\s+:\s+ok=(\d+)\s+changed=(\d+)\s+unreachable=(\d+)\s+failed=(\d+)").unwrap();

    let mut current_task = String::new();

    for line in output.lines() {
        let trimmed = line.trim();

        // Detect PLAY
        if let Some(caps) = play_re.captures(trimmed) {
            if !play_name.is_empty() && !task_results.is_empty() {
                lines.push(format_play_summary(&play_name, &task_results));
                task_results.clear();
            }
            play_name = caps[1].to_string();
            continue;
        }

        // Detect TASK
        if let Some(caps) = task_re.captures(trimmed) {
            current_task = caps[1].to_string();
            continue;
        }

        // Detect status lines
        if let Some(caps) = status_re.captures(trimmed) {
            let status = &caps[1];
            let host = &caps[2];
            task_results.push((current_task.clone(), status.to_string(), host.to_string()));
            continue;
        }

        // Detect PLAY RECAP
        if trimmed.contains("PLAY RECAP") {
            recap_section = true;
            continue;
        }

        if recap_section {
            if let Some(caps) = recap_re.captures(trimmed) {
                let host = &caps[1];
                let ok: u32 = caps[2].parse().unwrap_or(0);
                let changed: u32 = caps[3].parse().unwrap_or(0);
                let unreachable: u32 = caps[4].parse().unwrap_or(0);
                let failed: u32 = caps[5].parse().unwrap_or(0);

                let status = if failed > 0 || unreachable > 0 {
                    "FAILED"
                } else if changed > 0 {
                    "CHANGED"
                } else {
                    "OK"
                };

                recap_lines.push(format!(
                    "  {} [{}] ok:{} changed:{} failed:{}",
                    host.trim(), status, ok, changed, failed
                ));
            }
        }
    }

    // Output last play
    if !play_name.is_empty() && !task_results.is_empty() {
        lines.push(format_play_summary(&play_name, &task_results));
    }

    // Build output
    let mut result = Vec::new();

    if !lines.is_empty() {
        result.push("Playbook Summary".to_string());
        result.push("─".repeat(50));
        result.extend(lines);
    }

    if !recap_lines.is_empty() {
        result.push(String::new());
        result.push("Recap".to_string());
        result.push("─".repeat(50));
        result.extend(recap_lines);
    }

    if result.is_empty() {
        // Fallback - just show key lines
        output.lines()
            .filter(|l| {
                let t = l.trim();
                t.contains("PLAY [") ||
                t.contains("TASK [") ||
                t.starts_with("ok:") ||
                t.starts_with("changed:") ||
                t.starts_with("failed:") ||
                t.starts_with("fatal:") ||
                t.contains("PLAY RECAP") ||
                t.contains("ok=")
            })
            .take(30)
            .map(|l| l.trim().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        result.join("\n")
    }
}

fn format_play_summary(play: &str, results: &[(String, String, String)]) -> String {
    let ok = results.iter().filter(|(_, s, _)| s == "ok").count();
    let changed = results.iter().filter(|(_, s, _)| s == "changed").count();
    let failed = results.iter().filter(|(_, s, _)| s == "failed" || s == "fatal").count();
    let skipped = results.iter().filter(|(_, s, _)| s == "skipping").count();

    let status = if failed > 0 { "FAILED" } else if changed > 0 { "CHANGED" } else { "OK" };

    format!(
        "Play: {} [{}]\n  Tasks: {} ok, {} changed, {} failed, {} skipped",
        play, status, ok, changed, failed, skipped
    )
}

fn filter_galaxy_output(output: &str, args: &[String]) -> String {
    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("");

    match subcommand {
        "install" => {
            let installed: Vec<&str> = output.lines()
                .filter(|l| l.contains("was installed") || l.contains("is already installed"))
                .collect();

            if installed.is_empty() {
                output.lines().take(10).collect::<Vec<_>>().join("\n")
            } else {
                format!("Installed {} role(s)/collection(s)\n{}", installed.len(), installed.join("\n"))
            }
        }
        "list" => {
            let items: Vec<&str> = output.lines()
                .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                .collect();
            format!("{} installed\n{}", items.len(), items.join("\n"))
        }
        _ => output.lines().take(20).collect::<Vec<_>>().join("\n")
    }
}

fn filter_vault_output(output: &str) -> String {
    // Vault output is usually minimal
    let lines: Vec<&str> = output.lines()
        .filter(|l| !l.trim().is_empty())
        .take(5)
        .collect();

    if lines.is_empty() {
        "Vault operation completed".to_string()
    } else {
        lines.join("\n")
    }
}

fn filter_ansible_output(output: &str, args: &[String]) -> String {
    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("");

    match subcommand {
        "--list-hosts" | "-i" => {
            let hosts: Vec<&str> = output.lines()
                .filter(|l| !l.trim().is_empty() && !l.contains("hosts ("))
                .map(|l| l.trim())
                .collect();
            format!("{} hosts: {}", hosts.len(), hosts.join(", "))
        }
        "--version" => {
            output.lines().take(1).collect::<Vec<_>>().join("")
        }
        _ => {
            // Ad-hoc command output
            let mut results = Vec::new();
            let success_re = Regex::new(r"^(.+?)\s*\|\s*(SUCCESS|CHANGED|FAILED)").unwrap();

            for line in output.lines() {
                if let Some(caps) = success_re.captures(line) {
                    results.push(format!("{}: {}", &caps[1], &caps[2]));
                }
            }

            if results.is_empty() {
                output.lines().take(15).collect::<Vec<_>>().join("\n")
            } else {
                results.join("\n")
            }
        }
    }
}
