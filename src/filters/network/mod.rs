//! Network command filters (curl, ssh, plink, scp).

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for curl commands.
pub struct CurlFilter;

impl Filter for CurlFilter {
    fn name(&self) -> &'static str {
        "curl"
    }

    fn matches(&self, command: &str) -> bool {
        command == "curl"
    }

    fn execute(&self, _command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();

        let output = Command::new("curl")
            .args(args)
            .output()?;

        let exec_time_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_curl_output(&stdout, &stderr, args);

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        80
    }
}

fn filter_curl_output(stdout: &str, stderr: &str, args: &[String]) -> String {
    let mut result = Vec::new();

    // Check if verbose mode
    let is_verbose = args.iter().any(|a| a == "-v" || a == "--verbose");
    let is_headers = args.iter().any(|a| a == "-I" || a == "--head" || a == "-i" || a == "--include");

    // Filter stderr (progress, headers in verbose)
    if is_verbose {
        for line in stderr.lines() {
            let trimmed = line.trim();

            // Skip progress bars
            if trimmed.contains('%') && trimmed.contains('[') {
                continue;
            }

            // Keep important headers
            if trimmed.starts_with('>') || trimmed.starts_with('<') {
                let header = trimmed.trim_start_matches(|c| c == '>' || c == '<' || c == ' ');
                // Only keep key headers
                if header.starts_with("HTTP/") ||
                   header.starts_with("Content-Type") ||
                   header.starts_with("Content-Length") ||
                   header.starts_with("Location") ||
                   header.starts_with("Status") {
                    result.push(header.to_string());
                }
            }
        }
    }

    // Process stdout (body)
    let body = stdout.trim();

    if body.is_empty() {
        if !result.is_empty() {
            return result.join("\n");
        }
        // Check for error in stderr
        if stderr.contains("Could not resolve") || stderr.contains("Connection refused") {
            return format!("✗ {}", stderr.lines().find(|l| l.contains("curl")).unwrap_or("connection failed"));
        }
        return "✓ empty response".to_string();
    }

    // Detect JSON and compact it
    if body.starts_with('{') || body.starts_with('[') {
        let json_summary = summarize_json(body);
        result.push(json_summary);
    } else if is_headers {
        // Headers only request
        for line in body.lines().take(15) {
            result.push(line.to_string());
        }
        if body.lines().count() > 15 {
            result.push(format!("... +{} more headers", body.lines().count() - 15));
        }
    } else {
        // Text/HTML - truncate
        let lines: Vec<&str> = body.lines().collect();
        if lines.len() > 20 {
            for line in lines.iter().take(15) {
                result.push(line.to_string());
            }
            result.push(format!("... +{} more lines ({} chars)", lines.len() - 15, body.len()));
        } else {
            result.push(body.to_string());
        }
    }

    result.join("\n")
}

fn summarize_json(json: &str) -> String {
    // Try to parse and summarize JSON
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json) {
        match &value {
            serde_json::Value::Object(map) => {
                let keys: Vec<&String> = map.keys().take(10).collect();
                let summary = keys.iter().map(|k| k.as_str()).collect::<Vec<_>>().join(", ");
                if map.len() > 10 {
                    format!("{{{}... +{} keys}} ({} chars)", summary, map.len() - 10, json.len())
                } else {
                    format!("{{{}}} ({} chars)", summary, json.len())
                }
            }
            serde_json::Value::Array(arr) => {
                format!("[{} items] ({} chars)", arr.len(), json.len())
            }
            _ => {
                if json.len() > 200 {
                    format!("{}... ({} chars)", &json[..200], json.len())
                } else {
                    json.to_string()
                }
            }
        }
    } else {
        // Invalid JSON, truncate
        if json.len() > 500 {
            format!("{}... ({} chars)", &json[..500], json.len())
        } else {
            json.to_string()
        }
    }
}

/// Filter for SSH/plink commands.
pub struct SshFilter;

impl Filter for SshFilter {
    fn name(&self) -> &'static str {
        "ssh"
    }

    fn matches(&self, command: &str) -> bool {
        matches!(command, "ssh" | "plink" | "plink.exe")
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

        let filtered = filter_ssh_output(&stdout, &stderr);

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        80
    }
}

fn filter_ssh_output(stdout: &str, stderr: &str) -> String {
    let mut result = Vec::new();

    // Process stderr for connection info
    for line in stderr.lines() {
        let trimmed = line.trim();

        // Keep important messages
        if trimmed.contains("Authenticated") ||
           trimmed.contains("Connection") ||
           trimmed.contains("Permission denied") ||
           trimmed.contains("error") ||
           trimmed.contains("Warning") {
            result.push(trimmed.to_string());
        }
    }

    // Process stdout (command output)
    let stdout_lines: Vec<&str> = stdout.lines().collect();
    if stdout_lines.len() > 30 {
        for line in stdout_lines.iter().take(25) {
            result.push(line.to_string());
        }
        result.push(format!("... +{} more lines", stdout_lines.len() - 25));
    } else {
        for line in stdout_lines {
            result.push(line.to_string());
        }
    }

    if result.is_empty() {
        "✓ completed".to_string()
    } else {
        result.join("\n")
    }
}

/// Filter for SCP commands.
pub struct ScpFilter;

impl Filter for ScpFilter {
    fn name(&self) -> &'static str {
        "scp"
    }

    fn matches(&self, command: &str) -> bool {
        matches!(command, "scp" | "pscp" | "pscp.exe")
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

        let filtered = filter_scp_output(&stdout, &stderr);

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        80
    }
}

fn filter_scp_output(stdout: &str, stderr: &str) -> String {
    let mut result = Vec::new();

    // Look for file transfer info
    let file_re = Regex::new(r"(\S+)\s+\d+%\s+\d+[KMG]?B?\s+[\d.]+[KMG]?B/s").unwrap();

    for line in stderr.lines().chain(stdout.lines()) {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Error messages
        if trimmed.contains("Permission denied") ||
           trimmed.contains("No such file") ||
           trimmed.contains("error") {
            result.push(format!("✗ {}", trimmed));
        }

        // Final status (100%)
        if trimmed.contains("100%") {
            if let Some(caps) = file_re.captures(trimmed) {
                result.push(format!("✓ {}", caps.get(0).unwrap().as_str()));
            } else {
                result.push(format!("✓ {}", trimmed));
            }
        }
    }

    if result.is_empty() {
        "✓ transfer completed".to_string()
    } else {
        result.join("\n")
    }
}
