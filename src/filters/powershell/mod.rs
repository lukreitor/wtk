//! PowerShell cmdlet filters.

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for PowerShell commands that produce verbose/structured output.
pub struct PowerShellFilter;

impl Filter for PowerShellFilter {
    fn name(&self) -> &'static str {
        "powershell"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "powershell" || cmd == "powershell.exe" || cmd == "pwsh" || cmd == "pwsh.exe"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        // Try to detect the cmdlet being run
        let cmdlet = extract_cmdlet(args);
        let filtered = filter_by_cmdlet(&cmdlet, &stdout, &stderr);

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        70 // Lower priority - let more specific filters run first
    }
}

/// Filter for Get-Process cmdlet.
pub struct GetProcessFilter;

impl Filter for GetProcessFilter {
    fn name(&self) -> &'static str {
        "Get-Process"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd.contains("get-process") || cmd == "gps" || cmd == "ps"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();

        // For direct cmdlet execution
        let output = if command.to_lowercase().contains("get-process") {
            Command::new("powershell")
                .args(["-Command", &format!("{} {}", command, args.join(" "))])
                .output()?
        } else {
            Command::new(command).args(args).output()?
        };

        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_get_process(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for Get-Service cmdlet.
pub struct GetServiceFilter;

impl Filter for GetServiceFilter {
    fn name(&self) -> &'static str {
        "Get-Service"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd.contains("get-service") || cmd == "gsv"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();

        let output = if command.to_lowercase().contains("get-service") {
            Command::new("powershell")
                .args(["-Command", &format!("{} {}", command, args.join(" "))])
                .output()?
        } else {
            Command::new(command).args(args).output()?
        };

        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_get_service(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for Get-ChildItem (dir/ls equivalent).
pub struct GetChildItemFilter;

impl Filter for GetChildItemFilter {
    fn name(&self) -> &'static str {
        "Get-ChildItem"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd.contains("get-childitem") || cmd == "gci" || cmd == "dir" || cmd == "ls"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_get_childitem(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn extract_cmdlet(args: &[String]) -> String {
    // Look for -Command or cmdlet name in args
    let mut in_command = false;
    for arg in args {
        if in_command {
            // Extract first word as cmdlet
            let words: Vec<&str> = arg.split_whitespace().collect();
            if !words.is_empty() {
                return words[0].to_lowercase();
            }
        }
        if arg == "-Command" || arg == "-c" {
            in_command = true;
        }
    }

    // Check if any arg looks like a cmdlet (Verb-Noun pattern)
    for arg in args {
        if arg.contains('-') && !arg.starts_with('-') {
            let parts: Vec<&str> = arg.split('-').collect();
            if parts.len() == 2 && parts[0].chars().next().map_or(false, |c| c.is_uppercase()) {
                return arg.to_lowercase();
            }
        }
    }

    String::new()
}

fn filter_by_cmdlet(cmdlet: &str, stdout: &str, stderr: &str) -> String {
    match cmdlet {
        s if s.contains("get-process") => filter_get_process(stdout, stderr),
        s if s.contains("get-service") => filter_get_service(stdout, stderr),
        s if s.contains("get-childitem") => filter_get_childitem(stdout, stderr),
        s if s.contains("get-content") => filter_get_content(stdout, stderr),
        s if s.contains("get-netipaddress") => filter_get_netipaddress(stdout, stderr),
        s if s.contains("get-netadapter") => filter_get_netadapter(stdout, stderr),
        s if s.contains("get-disk") => filter_get_disk(stdout, stderr),
        s if s.contains("get-volume") => filter_get_volume(stdout, stderr),
        s if s.contains("get-eventlog") || s.contains("get-winevent") => filter_get_eventlog(stdout, stderr),
        s if s.contains("get-hotfix") => filter_get_hotfix(stdout, stderr),
        s if s.contains("get-computerinfo") => filter_get_computerinfo(stdout, stderr),
        _ => filter_generic(stdout, stderr),
    }
}

fn filter_get_process(stdout: &str, stderr: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    // Skip header lines
    let processes: Vec<&str> = lines.iter()
        .skip_while(|l| l.contains("Handles") || l.contains("---"))
        .copied()
        .collect();

    if processes.is_empty() {
        return "No processes found".to_string();
    }

    let mut result = vec![format!("{} processes", processes.len())];

    // Group by CPU usage (top consumers)
    let mut process_data: Vec<(String, f64, f64)> = Vec::new(); // name, cpu, mem

    for line in &processes {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 8 {
            let name = parts.last().unwrap_or(&"").to_string();
            let cpu: f64 = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let mem: f64 = parts.get(5).and_then(|s| s.replace(',', "").parse().ok()).unwrap_or(0.0);
            process_data.push((name, cpu, mem));
        }
    }

    // Sort by CPU and show top 10
    process_data.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    result.push("Top by CPU:".to_string());
    for (name, cpu, mem) in process_data.iter().take(5) {
        result.push(format!("  {} CPU:{:.1}% MEM:{:.0}K", truncate(name, 20), cpu, mem));
    }

    if !stderr.is_empty() {
        result.push(format!("⚠ {}", truncate(stderr.lines().next().unwrap_or(""), 50)));
    }

    result.join("\n")
}

fn filter_get_service(stdout: &str, stderr: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    // Skip header lines
    let services: Vec<&str> = lines.iter()
        .skip_while(|l| l.contains("Status") || l.contains("---"))
        .copied()
        .collect();

    if services.is_empty() {
        return "No services found".to_string();
    }

    let mut running = 0;
    let mut stopped = 0;

    for line in &services {
        if line.contains("Running") {
            running += 1;
        } else if line.contains("Stopped") {
            stopped += 1;
        }
    }

    let mut result = vec![format!("{} services ({} running, {} stopped)", services.len(), running, stopped)];

    // Show first few running services
    result.push("Running:".to_string());
    let mut shown = 0;
    for line in &services {
        if line.contains("Running") && shown < 5 {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                result.push(format!("  {}", parts.get(1).unwrap_or(&"")));
            }
            shown += 1;
        }
    }

    if running > 5 {
        result.push(format!("  ... +{} more running", running - 5));
    }

    if !stderr.is_empty() {
        result.push(format!("⚠ {}", truncate(stderr.lines().next().unwrap_or(""), 50)));
    }

    result.join("\n")
}

fn filter_get_childitem(stdout: &str, stderr: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    // Skip header lines
    let items: Vec<&str> = lines.iter()
        .skip_while(|l| l.contains("Mode") || l.contains("---") || l.contains("Directory:"))
        .copied()
        .collect();

    if items.is_empty() {
        return "Empty directory".to_string();
    }

    let mut dirs = 0;
    let mut files = 0;

    for line in &items {
        if line.starts_with('d') {
            dirs += 1;
        } else {
            files += 1;
        }
    }

    let mut result = vec![format!("{} items ({} dirs, {} files)", items.len(), dirs, files)];

    for line in items.iter().take(15) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let name = parts[4..].join(" ");
            let mode = parts.first().unwrap_or(&"");
            let prefix = if mode.starts_with('d') { "📁" } else { "📄" };
            result.push(format!("  {} {}", prefix, truncate(&name, 50)));
        }
    }

    if items.len() > 15 {
        result.push(format!("  ... +{} more", items.len() - 15));
    }

    if !stderr.is_empty() {
        result.push(format!("⚠ {}", truncate(stderr.lines().next().unwrap_or(""), 50)));
    }

    result.join("\n")
}

fn filter_get_content(stdout: &str, stderr: &str) -> String {
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.is_empty() {
        return "Empty file".to_string();
    }

    let mut result = vec![format!("{} lines", lines.len())];

    for line in lines.iter().take(20) {
        result.push(truncate(line, 80));
    }

    if lines.len() > 20 {
        result.push(format!("... +{} more lines", lines.len() - 20));
    }

    if !stderr.is_empty() {
        result.push(format!("⚠ {}", truncate(stderr.lines().next().unwrap_or(""), 50)));
    }

    result.join("\n")
}

fn filter_get_netipaddress(stdout: &str, stderr: &str) -> String {
    let ip_re = Regex::new(r"IPAddress\s*:\s*(\S+)").unwrap();
    let iface_re = Regex::new(r"InterfaceAlias\s*:\s*(.+)").unwrap();

    let ips: Vec<String> = ip_re.captures_iter(stdout)
        .map(|c| c[1].to_string())
        .filter(|ip| !ip.starts_with("fe80") && !ip.starts_with("::1") && ip != "127.0.0.1")
        .collect();

    let interfaces: Vec<String> = iface_re.captures_iter(stdout)
        .map(|c| c[1].trim().to_string())
        .collect();

    if ips.is_empty() {
        return filter_generic(stdout, stderr);
    }

    let mut result = vec![format!("{} IP addresses", ips.len())];
    for (i, ip) in ips.iter().take(10).enumerate() {
        let iface = interfaces.get(i).map(|s| s.as_str()).unwrap_or("");
        result.push(format!("  {} ({})", ip, truncate(iface, 20)));
    }

    result.join("\n")
}

fn filter_get_netadapter(stdout: &str, stderr: &str) -> String {
    let name_re = Regex::new(r"Name\s*:\s*(.+)").unwrap();
    let status_re = Regex::new(r"Status\s*:\s*(\S+)").unwrap();

    let names: Vec<String> = name_re.captures_iter(stdout)
        .map(|c| c[1].trim().to_string())
        .collect();

    let statuses: Vec<String> = status_re.captures_iter(stdout)
        .map(|c| c[1].to_string())
        .collect();

    if names.is_empty() {
        return filter_generic(stdout, stderr);
    }

    let up_count = statuses.iter().filter(|s| *s == "Up").count();

    let mut result = vec![format!("{} adapters ({} up)", names.len(), up_count)];
    for (name, status) in names.iter().zip(statuses.iter()).take(10) {
        let icon = if status == "Up" { "✓" } else { "✗" };
        result.push(format!("  {} {} ({})", icon, truncate(name, 30), status));
    }

    result.join("\n")
}

fn filter_get_disk(stdout: &str, stderr: &str) -> String {
    let number_re = Regex::new(r"Number\s*:\s*(\d+)").unwrap();
    let size_re = Regex::new(r"Size\s*:\s*(\S+)").unwrap();
    let model_re = Regex::new(r"FriendlyName\s*:\s*(.+)").unwrap();

    let numbers: Vec<String> = number_re.captures_iter(stdout).map(|c| c[1].to_string()).collect();
    let sizes: Vec<String> = size_re.captures_iter(stdout).map(|c| c[1].to_string()).collect();
    let models: Vec<String> = model_re.captures_iter(stdout).map(|c| c[1].trim().to_string()).collect();

    if numbers.is_empty() {
        return filter_generic(stdout, stderr);
    }

    let mut result = vec![format!("{} disks", numbers.len())];
    for i in 0..numbers.len().min(10) {
        let model = models.get(i).map(|s| s.as_str()).unwrap_or("Unknown");
        let size = sizes.get(i).map(|s| s.as_str()).unwrap_or("?");
        result.push(format!("  Disk {}: {} ({})", numbers[i], truncate(model, 30), size));
    }

    result.join("\n")
}

fn filter_get_volume(stdout: &str, stderr: &str) -> String {
    let letter_re = Regex::new(r"DriveLetter\s*:\s*(\w)").unwrap();
    let size_re = Regex::new(r"Size\s*:\s*(\S+)").unwrap();
    let remaining_re = Regex::new(r"SizeRemaining\s*:\s*(\S+)").unwrap();

    let letters: Vec<String> = letter_re.captures_iter(stdout).map(|c| c[1].to_string()).collect();
    let sizes: Vec<String> = size_re.captures_iter(stdout).map(|c| c[1].to_string()).collect();
    let remaining: Vec<String> = remaining_re.captures_iter(stdout).map(|c| c[1].to_string()).collect();

    if letters.is_empty() {
        return filter_generic(stdout, stderr);
    }

    let mut result = vec![format!("{} volumes", letters.len())];
    for i in 0..letters.len().min(10) {
        let size = sizes.get(i).map(|s| s.as_str()).unwrap_or("?");
        let free = remaining.get(i).map(|s| s.as_str()).unwrap_or("?");
        result.push(format!("  {}:\\ {} ({} free)", letters[i], size, free));
    }

    result.join("\n")
}

fn filter_get_eventlog(stdout: &str, stderr: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty() && !l.contains("---") && !l.contains("Index"))
        .collect();

    if lines.is_empty() {
        return "No events found".to_string();
    }

    let mut errors = 0;
    let mut warnings = 0;
    let mut info = 0;

    for line in &lines {
        if line.contains("Error") {
            errors += 1;
        } else if line.contains("Warning") {
            warnings += 1;
        } else if line.contains("Information") {
            info += 1;
        }
    }

    let mut result = vec![format!("{} events ({} errors, {} warnings, {} info)",
        lines.len(), errors, warnings, info)];

    // Show recent errors first
    result.push("Recent:".to_string());
    for line in lines.iter().take(5) {
        result.push(format!("  {}", truncate(line, 70)));
    }

    if lines.len() > 5 {
        result.push(format!("  ... +{} more", lines.len() - 5));
    }

    if !stderr.is_empty() {
        result.push(format!("⚠ {}", truncate(stderr.lines().next().unwrap_or(""), 50)));
    }

    result.join("\n")
}

fn filter_get_hotfix(stdout: &str, stderr: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty() && !l.contains("---") && !l.contains("Source"))
        .collect();

    if lines.is_empty() {
        return "No hotfixes found".to_string();
    }

    let mut result = vec![format!("{} hotfixes installed", lines.len())];

    for line in lines.iter().take(10) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            result.push(format!("  {} ({})", parts.get(1).unwrap_or(&""), parts.last().unwrap_or(&"")));
        }
    }

    if lines.len() > 10 {
        result.push(format!("  ... +{} more", lines.len() - 10));
    }

    if !stderr.is_empty() {
        result.push(format!("⚠ {}", truncate(stderr.lines().next().unwrap_or(""), 50)));
    }

    result.join("\n")
}

fn filter_get_computerinfo(stdout: &str, _stderr: &str) -> String {
    let os_re = Regex::new(r"OsName\s*:\s*(.+)").unwrap();
    let version_re = Regex::new(r"OsVersion\s*:\s*(.+)").unwrap();
    let arch_re = Regex::new(r"OsArchitecture\s*:\s*(.+)").unwrap();
    let mem_re = Regex::new(r"OsTotalVisibleMemorySize\s*:\s*(\d+)").unwrap();
    let cpu_re = Regex::new(r"CsProcessors\s*:\s*\{(.+)\}").unwrap();

    let os = os_re.captures(stdout).map(|c| c[1].trim().to_string()).unwrap_or_default();
    let version = version_re.captures(stdout).map(|c| c[1].trim().to_string()).unwrap_or_default();
    let arch = arch_re.captures(stdout).map(|c| c[1].trim().to_string()).unwrap_or_default();
    let mem_kb: u64 = mem_re.captures(stdout).and_then(|c| c[1].parse().ok()).unwrap_or(0);

    let mut result = Vec::new();

    if !os.is_empty() {
        result.push(format!("{} {}", os, arch));
    }
    if !version.is_empty() {
        result.push(format!("  Version: {}", version));
    }
    if mem_kb > 0 {
        let mem_gb = mem_kb as f64 / 1024.0 / 1024.0;
        result.push(format!("  Memory: {:.1} GB", mem_gb));
    }

    if result.is_empty() {
        filter_generic(stdout, "")
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
