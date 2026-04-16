//! Windows package manager filters (winget, choco, scoop).

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for Winget commands.
pub struct WingetFilter;

impl Filter for WingetFilter {
    fn name(&self) -> &'static str {
        "winget"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "winget" || cmd == "winget.exe"
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
            "install" => filter_winget_install(&stdout, &stderr),
            "upgrade" => filter_winget_upgrade(&stdout, &stderr),
            "list" => filter_winget_list(&stdout),
            "search" => filter_winget_search(&stdout),
            "show" => filter_winget_show(&stdout),
            "uninstall" => filter_winget_uninstall(&stdout, &stderr),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for Chocolatey commands.
pub struct ChocoFilter;

impl Filter for ChocoFilter {
    fn name(&self) -> &'static str {
        "choco"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "choco" || cmd == "choco.exe" || cmd == "chocolatey"
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
            "install" => filter_choco_install(&stdout, &stderr),
            "upgrade" => filter_choco_upgrade(&stdout, &stderr),
            "list" => filter_choco_list(&stdout),
            "search" => filter_choco_search(&stdout),
            "info" => filter_choco_info(&stdout),
            "uninstall" => filter_choco_uninstall(&stdout, &stderr),
            "outdated" => filter_choco_outdated(&stdout),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for Scoop commands.
pub struct ScoopFilter;

impl Filter for ScoopFilter {
    fn name(&self) -> &'static str {
        "scoop"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "scoop" || cmd == "scoop.exe" || cmd.ends_with("scoop.ps1")
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
            "install" => filter_scoop_install(&stdout, &stderr),
            "update" => filter_scoop_update(&stdout, &stderr),
            "list" => filter_scoop_list(&stdout),
            "search" => filter_scoop_search(&stdout),
            "info" => filter_scoop_info(&stdout),
            "uninstall" => filter_scoop_uninstall(&stdout, &stderr),
            "status" => filter_scoop_status(&stdout),
            "bucket" => filter_scoop_bucket(&stdout, args),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

// Winget filters
fn filter_winget_install(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("Successfully installed") {
        let pkg_re = Regex::new(r"Successfully installed\s+(.+)").unwrap();
        if let Some(caps) = pkg_re.captures(&combined) {
            return format!("✓ installed {}", caps[1].trim());
        }
        return "✓ installed".to_string();
    }

    if combined.contains("No package found") {
        return "✗ package not found".to_string();
    }

    if combined.contains("already installed") {
        return "✓ already installed".to_string();
    }

    filter_generic(stdout, stderr)
}

fn filter_winget_upgrade(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let upgrade_re = Regex::new(r"(\d+) package\(?s?\)? (?:have|has) upgrades? available").unwrap();
    let success_re = Regex::new(r"Successfully installed|upgraded").unwrap();

    if let Some(caps) = upgrade_re.captures(&combined) {
        return format!("⚠ {} packages have upgrades available", &caps[1]);
    }

    if success_re.is_match(&combined) {
        return "✓ upgraded".to_string();
    }

    if combined.contains("No installed package found") || combined.contains("No applicable upgrade") {
        return "✓ all packages up to date".to_string();
    }

    filter_generic(stdout, stderr)
}

fn filter_winget_list(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    // Skip header lines
    let pkg_lines: Vec<&str> = lines.iter()
        .skip_while(|l| !l.contains("---"))
        .skip(1)
        .copied()
        .collect();

    if pkg_lines.is_empty() {
        return "No packages installed".to_string();
    }

    let mut result = vec![format!("{} packages installed", pkg_lines.len())];

    for line in pkg_lines.iter().take(15) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            result.push(format!("  {}", truncate(parts[0], 40)));
        }
    }

    if pkg_lines.len() > 15 {
        result.push(format!("  ... +{} more", pkg_lines.len() - 15));
    }

    result.join("\n")
}

fn filter_winget_search(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    let pkg_lines: Vec<&str> = lines.iter()
        .skip_while(|l| !l.contains("---"))
        .skip(1)
        .copied()
        .collect();

    if pkg_lines.is_empty() {
        return "No packages found".to_string();
    }

    let mut result = vec![format!("{} packages found", pkg_lines.len())];

    for line in pkg_lines.iter().take(10) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            result.push(format!("  {} ({})", parts[0], parts.get(1).unwrap_or(&"")));
        }
    }

    if pkg_lines.len() > 10 {
        result.push(format!("  ... +{} more", pkg_lines.len() - 10));
    }

    result.join("\n")
}

fn filter_winget_show(stdout: &str) -> String {
    let name_re = Regex::new(r"(?m)^(?:Name|Found)\s*[:\|]\s*(.+)$").unwrap();
    let version_re = Regex::new(r"(?m)^Version\s*[:\|]\s*(.+)$").unwrap();
    let publisher_re = Regex::new(r"(?m)^Publisher\s*[:\|]\s*(.+)$").unwrap();

    let name = name_re.captures(stdout).map(|c| c[1].trim().to_string()).unwrap_or_default();
    let version = version_re.captures(stdout).map(|c| c[1].trim().to_string()).unwrap_or_default();
    let publisher = publisher_re.captures(stdout).map(|c| c[1].trim().to_string()).unwrap_or_default();

    if !name.is_empty() {
        let mut result = vec![name];
        if !version.is_empty() {
            result.push(format!("  Version: {}", version));
        }
        if !publisher.is_empty() {
            result.push(format!("  Publisher: {}", publisher));
        }
        result.join("\n")
    } else {
        filter_generic(stdout, "")
    }
}

fn filter_winget_uninstall(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("Successfully uninstalled") {
        return "✓ uninstalled".to_string();
    }

    if combined.contains("No installed package found") {
        return "✗ package not found".to_string();
    }

    filter_generic(stdout, stderr)
}

// Chocolatey filters
fn filter_choco_install(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let success_re = Regex::new(r"(\d+)/(\d+) packages? installed").unwrap();
    let already_re = Regex::new(r"already installed").unwrap();

    if let Some(caps) = success_re.captures(&combined) {
        return format!("✓ {}/{} packages installed", &caps[1], &caps[2]);
    }

    if already_re.is_match(&combined) {
        return "✓ already installed".to_string();
    }

    if combined.contains("Failures") {
        return "✗ installation failed".to_string();
    }

    filter_generic(stdout, stderr)
}

fn filter_choco_upgrade(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let success_re = Regex::new(r"(\d+)/(\d+) packages? upgraded").unwrap();

    if let Some(caps) = success_re.captures(&combined) {
        return format!("✓ {}/{} packages upgraded", &caps[1], &caps[2]);
    }

    if combined.contains("0 packages upgraded") || combined.contains("can upgrade 0") {
        return "✓ all packages up to date".to_string();
    }

    filter_generic(stdout, stderr)
}

fn filter_choco_list(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    // Last line usually has count
    let count_re = Regex::new(r"(\d+) packages? installed").unwrap();
    let count = count_re.captures(stdout).and_then(|c| c[1].parse::<usize>().ok()).unwrap_or(lines.len());

    let mut result = vec![format!("{} packages installed", count)];

    for line in lines.iter().take(15) {
        if !line.contains("packages installed") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                result.push(format!("  {} v{}", parts[0], parts[1]));
            }
        }
    }

    if count > 15 {
        result.push(format!("  ... +{} more", count - 15));
    }

    result.join("\n")
}

fn filter_choco_search(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    let count_re = Regex::new(r"(\d+) packages? found").unwrap();
    let count = count_re.captures(stdout).and_then(|c| c[1].parse::<usize>().ok()).unwrap_or(0);

    if count == 0 {
        return "No packages found".to_string();
    }

    let mut result = vec![format!("{} packages found", count)];

    for line in lines.iter().take(10) {
        if !line.contains("packages found") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                result.push(format!("  {} v{}", parts[0], parts[1]));
            }
        }
    }

    if count > 10 {
        result.push(format!("  ... +{} more", count - 10));
    }

    result.join("\n")
}

fn filter_choco_info(stdout: &str) -> String {
    let title_re = Regex::new(r"Title:\s*(.+)").unwrap();
    let version_re = Regex::new(r"Version:\s*(.+)").unwrap();
    let downloads_re = Regex::new(r"Downloads:\s*(.+)").unwrap();

    let title = title_re.captures(stdout).map(|c| c[1].trim().to_string()).unwrap_or_default();
    let version = version_re.captures(stdout).map(|c| c[1].trim().to_string()).unwrap_or_default();
    let downloads = downloads_re.captures(stdout).map(|c| c[1].trim().to_string()).unwrap_or_default();

    if !title.is_empty() {
        let mut result = vec![title];
        if !version.is_empty() {
            result.push(format!("  Version: {}", version));
        }
        if !downloads.is_empty() {
            result.push(format!("  Downloads: {}", downloads));
        }
        result.join("\n")
    } else {
        filter_generic(stdout, "")
    }
}

fn filter_choco_uninstall(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("has been successfully uninstalled") || combined.contains("uninstalled 1/") {
        return "✓ uninstalled".to_string();
    }

    if combined.contains("Cannot uninstall") || combined.contains("not installed") {
        return "✗ package not installed".to_string();
    }

    filter_generic(stdout, stderr)
}

fn filter_choco_outdated(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty() && l.contains("|"))
        .collect();

    if lines.len() <= 1 {
        return "✓ all packages up to date".to_string();
    }

    let outdated_count = lines.len() - 1; // Exclude header
    let mut result = vec![format!("⚠ {} packages outdated", outdated_count)];

    for line in lines.iter().skip(1).take(10) {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 3 {
            result.push(format!("  {} {} → {}", parts[0].trim(), parts[1].trim(), parts[2].trim()));
        }
    }

    if outdated_count > 10 {
        result.push(format!("  ... +{} more", outdated_count - 10));
    }

    result.join("\n")
}

// Scoop filters
fn filter_scoop_install(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("was installed successfully") {
        let pkg_re = Regex::new(r"'([^']+)' was installed").unwrap();
        if let Some(caps) = pkg_re.captures(&combined) {
            return format!("✓ installed {}", &caps[1]);
        }
        return "✓ installed".to_string();
    }

    if combined.contains("is already installed") {
        return "✓ already installed".to_string();
    }

    if combined.contains("Couldn't find manifest") {
        return "✗ package not found".to_string();
    }

    filter_generic(stdout, stderr)
}

fn filter_scoop_update(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let updated_re = Regex::new(r"(\d+) apps? (?:was|were) updated").unwrap();

    if let Some(caps) = updated_re.captures(&combined) {
        return format!("✓ {} apps updated", &caps[1]);
    }

    if combined.contains("Everything is up to date") || combined.contains("Latest versions installed") {
        return "✓ all apps up to date".to_string();
    }

    filter_generic(stdout, stderr)
}

fn filter_scoop_list(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    if lines.is_empty() {
        return "No apps installed".to_string();
    }

    // Skip header if present
    let apps: Vec<&str> = lines.iter()
        .skip_while(|l| l.contains("Name") && l.contains("Version"))
        .skip_while(|l| l.contains("----"))
        .copied()
        .collect();

    let app_count = if apps.is_empty() { lines.len() } else { apps.len() };
    let mut result = vec![format!("{} apps installed", app_count)];

    let display_lines = if apps.is_empty() { &lines } else { &apps };
    for line in display_lines.iter().take(15) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            if parts.len() >= 2 {
                result.push(format!("  {} v{}", parts[0], parts[1]));
            } else {
                result.push(format!("  {}", parts[0]));
            }
        }
    }

    if app_count > 15 {
        result.push(format!("  ... +{} more", app_count - 15));
    }

    result.join("\n")
}

fn filter_scoop_search(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    if lines.is_empty() || stdout.contains("No matches found") {
        return "No apps found".to_string();
    }

    let mut result = vec![format!("{} apps found", lines.len())];

    for line in lines.iter().take(10) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            result.push(format!("  {}", parts[0]));
        }
    }

    if lines.len() > 10 {
        result.push(format!("  ... +{} more", lines.len() - 10));
    }

    result.join("\n")
}

fn filter_scoop_info(stdout: &str) -> String {
    let name_re = Regex::new(r"(?m)^Name\s*:\s*(.+)$").unwrap();
    let version_re = Regex::new(r"(?m)^Version\s*:\s*(.+)$").unwrap();
    let bucket_re = Regex::new(r"(?m)^Bucket\s*:\s*(.+)$").unwrap();

    let name = name_re.captures(stdout).map(|c| c[1].trim().to_string()).unwrap_or_default();
    let version = version_re.captures(stdout).map(|c| c[1].trim().to_string()).unwrap_or_default();
    let bucket = bucket_re.captures(stdout).map(|c| c[1].trim().to_string()).unwrap_or_default();

    if !name.is_empty() {
        let mut result = vec![name];
        if !version.is_empty() {
            result.push(format!("  Version: {}", version));
        }
        if !bucket.is_empty() {
            result.push(format!("  Bucket: {}", bucket));
        }
        result.join("\n")
    } else {
        filter_generic(stdout, "")
    }
}

fn filter_scoop_uninstall(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("was uninstalled") {
        return "✓ uninstalled".to_string();
    }

    if combined.contains("isn't installed") {
        return "✗ app not installed".to_string();
    }

    filter_generic(stdout, stderr)
}

fn filter_scoop_status(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    if stdout.contains("Everything is up to date") || lines.is_empty() {
        return "✓ all apps up to date".to_string();
    }

    // Count apps with updates
    let update_lines: Vec<&str> = lines.iter()
        .skip_while(|l| l.contains("Name") || l.contains("----"))
        .copied()
        .collect();

    if update_lines.is_empty() {
        return "✓ all apps up to date".to_string();
    }

    let mut result = vec![format!("⚠ {} apps have updates", update_lines.len())];

    for line in update_lines.iter().take(10) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            result.push(format!("  {} {} → {}", parts[0], parts[1], parts[2]));
        }
    }

    if update_lines.len() > 10 {
        result.push(format!("  ... +{} more", update_lines.len() - 10));
    }

    result.join("\n")
}

fn filter_scoop_bucket(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "list" => {
            let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
            if lines.is_empty() {
                "No buckets added".to_string()
            } else {
                let mut result = vec![format!("{} buckets", lines.len())];
                for line in lines.iter().take(10) {
                    result.push(format!("  {}", line.trim()));
                }
                result.join("\n")
            }
        }
        "add" => {
            if stdout.contains("was added successfully") {
                "✓ bucket added".to_string()
            } else {
                filter_generic(stdout, "")
            }
        }
        "rm" => {
            if stdout.contains("was removed") {
                "✓ bucket removed".to_string()
            } else {
                filter_generic(stdout, "")
            }
        }
        _ => filter_generic(stdout, ""),
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
