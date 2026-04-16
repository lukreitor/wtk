//! Windows system command filters.

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for Windows system commands.
pub struct WindowsSystemFilter;

impl Filter for WindowsSystemFilter {
    fn name(&self) -> &'static str {
        "windows"
    }

    fn matches(&self, command: &str) -> bool {
        matches!(command.to_lowercase().as_str(),
            "ipconfig" | "ipconfig.exe" |
            "netstat" | "netstat.exe" |
            "tasklist" | "tasklist.exe" |
            "systeminfo" | "systeminfo.exe" |
            "whoami" | "whoami.exe" |
            "ping" | "ping.exe" |
            "nslookup" | "nslookup.exe" |
            "tracert" | "tracert.exe" |
            // Phase 1 additions
            "wmic" | "wmic.exe" |
            "netsh" | "netsh.exe" |
            "tree" | "tree.exe" |
            "where" | "where.exe" |
            "sc" | "sc.exe" |
            "reg" | "reg.exe" |
            "dism" | "dism.exe" |
            "sfc" | "sfc.exe" |
            "hostname" | "hostname.exe" |
            "getmac" | "getmac.exe" |
            "arp" | "arp.exe" |
            "route" | "route.exe" |
            // Phase 1 remaining
            "diskpart" | "diskpart.exe" |
            "bcdedit" | "bcdedit.exe" |
            "certutil" | "certutil.exe" |
            "fsutil" | "fsutil.exe" |
            "icacls" | "icacls.exe" |
            "attrib" | "attrib.exe" |
            "findstr" | "findstr.exe" |
            "robocopy" | "robocopy.exe"
        )
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let cmd_lower = command.to_lowercase();
        let cmd_name = cmd_lower.trim_end_matches(".exe");

        let start = Instant::now();

        let output = Command::new(command)
            .args(args)
            .output()?;

        let exec_time_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = match cmd_name {
            "ipconfig" => filter_ipconfig(&stdout, args),
            "netstat" => filter_netstat(&stdout, args),
            "tasklist" => filter_tasklist(&stdout),
            "systeminfo" => filter_systeminfo(&stdout),
            "whoami" => filter_whoami(&stdout, args),
            "ping" => filter_ping(&stdout),
            "nslookup" => filter_nslookup(&stdout),
            "tracert" => filter_tracert(&stdout),
            // Phase 1 additions
            "wmic" => filter_wmic(&stdout, args),
            "netsh" => filter_netsh(&stdout, args),
            "tree" => filter_tree(&stdout),
            "where" => filter_where(&stdout),
            "sc" => filter_sc(&stdout, args),
            "reg" => filter_reg(&stdout),
            "dism" => filter_dism(&stdout),
            "sfc" => filter_sfc(&stdout),
            "hostname" => stdout.trim().to_string(),
            "getmac" => filter_getmac(&stdout),
            "arp" => filter_arp(&stdout),
            "route" => filter_route(&stdout),
            // Phase 1 remaining
            "diskpart" => filter_diskpart(&stdout),
            "bcdedit" => filter_bcdedit(&stdout),
            "certutil" => filter_certutil(&stdout, args),
            "fsutil" => filter_fsutil(&stdout),
            "icacls" => filter_icacls(&stdout),
            "attrib" => filter_attrib(&stdout),
            "findstr" => filter_findstr(&stdout),
            "robocopy" => filter_robocopy(&stdout),
            _ => stdout.clone(),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        70
    }
}

/// Filter ipconfig output.
fn filter_ipconfig(stdout: &str, args: &[String]) -> String {
    let is_all = args.iter().any(|a| a.to_lowercase() == "/all");

    let mut result = Vec::new();
    let mut current_adapter = String::new();
    let mut ipv4 = String::new();
    let mut ipv6 = String::new();
    let mut gateway = String::new();
    let mut dns = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();

        // Adapter header
        if line.starts_with("Ethernet adapter") || line.starts_with("Wireless") ||
           line.starts_with("Adaptador") {
            // Save previous adapter
            if !current_adapter.is_empty() && !ipv4.is_empty() {
                result.push(format_adapter(&current_adapter, &ipv4, &ipv6, &gateway, &dns));
            }
            current_adapter = trimmed.trim_end_matches(':').to_string();
            ipv4.clear();
            ipv6.clear();
            gateway.clear();
            dns.clear();
        }

        // IPv4
        if trimmed.contains("IPv4") || trimmed.contains("Endereço IPv4") {
            if let Some(ip) = extract_value(trimmed) {
                ipv4 = ip;
            }
        }

        // IPv6 (only show link-local)
        if trimmed.contains("IPv6") && trimmed.contains("fe80") {
            if let Some(ip) = extract_value(trimmed) {
                ipv6 = ip.split('%').next().unwrap_or(&ip).to_string();
            }
        }

        // Gateway
        if trimmed.contains("Gateway") || trimmed.contains("Padrão") {
            if let Some(gw) = extract_value(trimmed) {
                if !gw.is_empty() {
                    gateway = gw;
                }
            }
        }

        // DNS (only with /all)
        if is_all && (trimmed.contains("DNS Servers") || trimmed.contains("Servidores DNS")) {
            if let Some(d) = extract_value(trimmed) {
                dns.push(d);
            }
        }
    }

    // Don't forget last adapter
    if !current_adapter.is_empty() && !ipv4.is_empty() {
        result.push(format_adapter(&current_adapter, &ipv4, &ipv6, &gateway, &dns));
    }

    if result.is_empty() {
        "No active adapters".to_string()
    } else {
        result.join("\n")
    }
}

fn format_adapter(name: &str, ipv4: &str, ipv6: &str, gateway: &str, dns: &[String]) -> String {
    let mut parts = vec![format!("{}: {}", truncate_adapter_name(name), ipv4)];

    if !gateway.is_empty() {
        parts.push(format!("  gw: {}", gateway));
    }

    if !dns.is_empty() {
        parts.push(format!("  dns: {}", dns.join(", ")));
    }

    parts.join("\n")
}

fn truncate_adapter_name(name: &str) -> String {
    if name.len() > 30 {
        format!("{}...", &name[..27])
    } else {
        name.to_string()
    }
}

fn extract_value(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() >= 2 {
        let value = parts[1..].join(":").trim().to_string();
        if !value.is_empty() {
            return Some(value);
        }
    }
    None
}

/// Filter netstat output.
fn filter_netstat(stdout: &str, args: &[String]) -> String {
    let mut result = Vec::new();
    let mut listen_count = 0;
    let mut established_count = 0;
    let mut other_count = 0;

    let is_listening = args.iter().any(|a| a.to_lowercase().contains("l"));
    let show_all = args.iter().any(|a| a == "-a" || a == "-an" || a == "-ano");

    let mut connections: Vec<String> = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();

        // Skip headers
        if trimmed.starts_with("Proto") || trimmed.starts_with("Active") ||
           trimmed.is_empty() || trimmed.starts_with("Conexões") {
            continue;
        }

        // Parse connection line
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 4 {
            let state = parts.get(3).unwrap_or(&"");

            if state.contains("LISTEN") {
                listen_count += 1;
                if connections.len() < 10 && (is_listening || show_all) {
                    let local = parts.get(1).unwrap_or(&"");
                    connections.push(format!("LISTEN {}", local));
                }
            } else if state.contains("ESTABLISHED") {
                established_count += 1;
                if connections.len() < 10 {
                    let local = parts.get(1).unwrap_or(&"");
                    let remote = parts.get(2).unwrap_or(&"");
                    connections.push(format!("ESTAB {} → {}", local, remote));
                }
            } else {
                other_count += 1;
            }
        }
    }

    // Summary
    result.push(format!("Connections: {} listening, {} established, {} other",
        listen_count, established_count, other_count));

    // Show sample connections
    if !connections.is_empty() {
        for conn in connections.iter().take(10) {
            result.push(format!("  {}", conn));
        }
        let total = listen_count + established_count + other_count;
        if total > 10 {
            result.push(format!("  ... +{} more", total - 10));
        }
    }

    result.join("\n")
}

/// Filter tasklist output.
fn filter_tasklist(stdout: &str) -> String {
    let mut processes: Vec<(String, u64, usize)> = Vec::new(); // name, pid, mem_kb
    let mem_re = Regex::new(r"(\d[\d,.]*)\s*K").unwrap();

    for line in stdout.lines() {
        let trimmed = line.trim();

        // Skip headers
        if trimmed.starts_with("Image Name") || trimmed.starts_with("=") ||
           trimmed.starts_with("Nome da Imagem") || trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 5 {
            let name = parts[0].to_string();
            let pid: u64 = parts[1].parse().unwrap_or(0);

            // Parse memory (handle different locales)
            let mem_str = parts[4..].join(" ");
            let mem_kb: usize = if let Some(caps) = mem_re.captures(&mem_str) {
                caps[1].replace(",", "").replace(".", "").parse().unwrap_or(0)
            } else {
                0
            };

            processes.push((name, pid, mem_kb));
        }
    }

    // Sort by memory (descending)
    processes.sort_by(|a, b| b.2.cmp(&a.2));

    let mut result = Vec::new();
    result.push(format!("Processes: {} total", processes.len()));

    // Top 10 by memory
    result.push("Top by memory:".to_string());
    for (name, pid, mem) in processes.iter().take(10) {
        let mem_mb = *mem as f64 / 1024.0;
        result.push(format!("  {} ({}) {:.1} MB", name, pid, mem_mb));
    }

    if processes.len() > 10 {
        result.push(format!("  ... +{} more", processes.len() - 10));
    }

    result.join("\n")
}

/// Filter systeminfo output.
fn filter_systeminfo(stdout: &str) -> String {
    let mut result = Vec::new();

    let keys = [
        "Host Name", "Nome do host",
        "OS Name", "Nome do sistema",
        "OS Version", "Versão do sistema",
        "System Type", "Tipo de sistema",
        "Total Physical Memory", "Memória física total",
        "Available Physical Memory", "Memória física disponível",
        "Domain", "Domínio",
        "Logon Server", "Servidor de logon",
    ];

    for line in stdout.lines() {
        for key in &keys {
            if line.contains(key) {
                if let Some(value) = extract_value(line) {
                    let short_key = key.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
                    result.push(format!("{}: {}", short_key, value));
                }
                break;
            }
        }
    }

    if result.is_empty() {
        "Unable to parse system info".to_string()
    } else {
        result.join("\n")
    }
}

/// Filter whoami output.
fn filter_whoami(stdout: &str, args: &[String]) -> String {
    let is_all = args.iter().any(|a| a.to_lowercase() == "/all");
    let is_groups = args.iter().any(|a| a.to_lowercase() == "/groups");

    if is_all || is_groups {
        // For /all or /groups, summarize
        let mut groups = Vec::new();

        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.contains("\\") && !trimmed.starts_with("User") &&
               !trimmed.starts_with("Usuário") {
                // It's a group or user line
                let name = trimmed.split_whitespace().next().unwrap_or(trimmed);
                if groups.len() < 10 {
                    groups.push(name.to_string());
                }
            }
        }

        if groups.len() > 10 {
            groups.push(format!("... +{} more", groups.len() - 10));
        }

        if groups.is_empty() {
            stdout.lines().next().unwrap_or("").to_string()
        } else {
            groups.join("\n")
        }
    } else {
        // Simple whoami - just return as is
        stdout.trim().to_string()
    }
}

/// Filter ping output.
fn filter_ping(stdout: &str) -> String {
    let mut result = Vec::new();
    let mut success = 0;
    let mut fail = 0;

    let time_re = Regex::new(r"time[=<](\d+)ms").unwrap();
    let ttl_re = Regex::new(r"TTL=(\d+)").unwrap();
    let stats_re = Regex::new(r"(\d+)%\s*(loss|perda)").unwrap();

    let mut times: Vec<u32> = Vec::new();
    let mut ttl = 0;

    for line in stdout.lines() {
        let trimmed = line.trim();

        // Reply lines
        if trimmed.contains("Reply") || trimmed.contains("Resposta") {
            success += 1;
            if let Some(caps) = time_re.captures(trimmed) {
                times.push(caps[1].parse().unwrap_or(0));
            }
            if ttl == 0 {
                if let Some(caps) = ttl_re.captures(trimmed) {
                    ttl = caps[1].parse().unwrap_or(0);
                }
            }
        }

        // Timeout
        if trimmed.contains("timed out") || trimmed.contains("esgotado") {
            fail += 1;
        }

        // Statistics line
        if let Some(caps) = stats_re.captures(trimmed) {
            // Already have stats
        }
    }

    // Build summary
    let total = success + fail;
    if total > 0 {
        let avg_time = if !times.is_empty() {
            times.iter().sum::<u32>() / times.len() as u32
        } else {
            0
        };

        let loss_pct = (fail as f64 / total as f64) * 100.0;

        if fail == 0 {
            result.push(format!("✓ {}/{} ok, avg {}ms, TTL={}", success, total, avg_time, ttl));
        } else {
            result.push(format!("⚠ {}/{} ok ({:.0}% loss), avg {}ms", success, total, loss_pct, avg_time));
        }
    } else {
        result.push("✗ no response".to_string());
    }

    result.join("\n")
}

/// Filter nslookup output.
fn filter_nslookup(stdout: &str) -> String {
    let mut result = Vec::new();
    let mut addresses = Vec::new();

    let addr_re = Regex::new(r"Address(?:es)?:\s*(.+)").unwrap();
    let name_re = Regex::new(r"Name:\s*(.+)").unwrap();

    let mut found_name = String::new();

    for line in stdout.lines() {
        let trimmed = line.trim();

        if let Some(caps) = name_re.captures(trimmed) {
            found_name = caps[1].to_string();
        }

        if let Some(caps) = addr_re.captures(trimmed) {
            let addr = caps[1].trim();
            // Skip server address (usually first)
            if addresses.is_empty() && addr.contains("127.0.0.1") {
                continue;
            }
            if !addr.contains("#") {
                addresses.push(addr.to_string());
            }
        }

        // Direct address lines
        if trimmed.starts_with("192.") || trimmed.starts_with("10.") ||
           trimmed.starts_with("172.") || trimmed.chars().all(|c| c.is_numeric() || c == '.') {
            addresses.push(trimmed.to_string());
        }
    }

    if !found_name.is_empty() {
        result.push(format!("Name: {}", found_name));
    }

    if !addresses.is_empty() {
        for addr in addresses.iter().take(5) {
            result.push(format!("  → {}", addr));
        }
        if addresses.len() > 5 {
            result.push(format!("  ... +{} more", addresses.len() - 5));
        }
    }

    if result.is_empty() {
        stdout.lines().take(5).collect::<Vec<_>>().join("\n")
    } else {
        result.join("\n")
    }
}

/// Filter tracert output.
fn filter_tracert(stdout: &str) -> String {
    let mut result = Vec::new();
    let mut hops = Vec::new();

    let hop_re = Regex::new(r"^\s*(\d+)\s+").unwrap();

    for line in stdout.lines() {
        let trimmed = line.trim();

        // Skip header
        if trimmed.contains("Tracing route") || trimmed.contains("over a maximum") ||
           trimmed.contains("Rastreando") {
            continue;
        }

        // Hop lines
        if let Some(caps) = hop_re.captures(trimmed) {
            let hop_num: u32 = caps[1].parse().unwrap_or(0);

            // Extract host/IP
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            let host = parts.last().unwrap_or(&"*");

            if trimmed.contains("*") && trimmed.matches('*').count() >= 3 {
                hops.push(format!("{:>2}. * timeout", hop_num));
            } else {
                // Find first time
                let time = parts.iter()
                    .find(|p| p.ends_with("ms"))
                    .map(|t| t.to_string())
                    .unwrap_or_default();

                hops.push(format!("{:>2}. {} {}", hop_num, host, time));
            }
        }
    }

    if hops.len() > 15 {
        for hop in hops.iter().take(10) {
            result.push(hop.clone());
        }
        result.push("...".to_string());
        for hop in hops.iter().rev().take(3).rev() {
            result.push(hop.clone());
        }
    } else {
        result = hops;
    }

    if result.is_empty() {
        stdout.lines().take(10).collect::<Vec<_>>().join("\n")
    } else {
        result.join("\n")
    }
}

// ============================================================================
// Phase 1: Additional Windows CMD Filters
// ============================================================================

/// Filter wmic output - WMI queries are extremely verbose.
fn filter_wmic(stdout: &str, args: &[String]) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.is_empty() {
        return "No data".to_string();
    }

    // Get the query type from args
    let query_type = args.first().map(|s| s.to_lowercase()).unwrap_or_default();

    let mut result = Vec::new();
    result.push(format!("{} results:", lines.len().saturating_sub(1)));

    // Show header and first few rows
    for line in lines.iter().take(11) {
        let truncated = if line.len() > 100 {
            format!("{}...", &line[..97])
        } else {
            line.to_string()
        };
        result.push(truncated);
    }

    if lines.len() > 11 {
        result.push(format!("... +{} more rows", lines.len() - 11));
    }

    result.join("\n")
}

/// Filter netsh output - network shell commands.
fn filter_netsh(stdout: &str, args: &[String]) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.is_empty() {
        return "No output".to_string();
    }

    let mut result = Vec::new();

    // Check subcommand
    let subcmd = args.iter()
        .take(2)
        .map(|s| s.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");

    if subcmd.contains("wlan") && subcmd.contains("show") {
        // WLAN info - extract key details
        for line in &lines {
            if line.contains("SSID") || line.contains("Signal") ||
               line.contains("State") || line.contains("Authentication") {
                result.push(line.trim().to_string());
            }
        }
        if result.is_empty() {
            result = lines.iter().take(10).map(|s| s.to_string()).collect();
        }
    } else if subcmd.contains("interface") {
        // Interface info
        for line in lines.iter().take(15) {
            result.push(line.to_string());
        }
        if lines.len() > 15 {
            result.push(format!("... +{} more", lines.len() - 15));
        }
    } else {
        // Generic - show summary
        result.push(format!("{} lines of output", lines.len()));
        for line in lines.iter().take(10) {
            result.push(line.to_string());
        }
        if lines.len() > 10 {
            result.push(format!("... +{} more", lines.len() - 10));
        }
    }

    result.join("\n")
}

/// Filter tree output - directory trees can be huge.
fn filter_tree(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.is_empty() {
        return "Empty".to_string();
    }

    let mut dirs = 0;
    let mut files = 0;

    for line in &lines {
        if line.contains("───") || line.contains("---") {
            if line.contains('.') {
                files += 1;
            } else {
                dirs += 1;
            }
        }
    }

    let mut result = vec![format!("{} dirs, {} files", dirs, files)];

    // Show first 20 lines
    for line in lines.iter().take(20) {
        result.push(line.to_string());
    }

    if lines.len() > 20 {
        result.push(format!("... +{} more items", lines.len() - 20));
    }

    result.join("\n")
}

/// Filter where output - find executables.
fn filter_where(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.is_empty() {
        return "Not found".to_string();
    }

    let mut result = vec![format!("{} matches:", lines.len())];

    for line in lines.iter().take(10) {
        result.push(format!("  {}", line));
    }

    if lines.len() > 10 {
        result.push(format!("  ... +{} more", lines.len() - 10));
    }

    result.join("\n")
}

/// Filter sc output - service control.
fn filter_sc(stdout: &str, args: &[String]) -> String {
    let subcmd = args.first().map(|s| s.to_lowercase()).unwrap_or_default();

    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.is_empty() {
        return "No output".to_string();
    }

    let mut result = Vec::new();

    if subcmd == "query" || subcmd == "queryex" {
        // Service query - extract key info
        let mut services = 0;
        let mut running = 0;
        let mut stopped = 0;

        for line in &lines {
            if line.contains("SERVICE_NAME") {
                services += 1;
            }
            if line.contains("RUNNING") {
                running += 1;
            }
            if line.contains("STOPPED") {
                stopped += 1;
            }
        }

        result.push(format!("{} services ({} running, {} stopped)", services, running, stopped));

        // Show first few service names
        let mut count = 0;
        for line in &lines {
            if line.contains("SERVICE_NAME") && count < 10 {
                if let Some(name) = line.split(':').nth(1) {
                    result.push(format!("  {}", name.trim()));
                    count += 1;
                }
            }
        }

        if services > 10 {
            result.push(format!("  ... +{} more", services - 10));
        }
    } else {
        // Other sc commands - show truncated
        for line in lines.iter().take(10) {
            result.push(line.to_string());
        }
        if lines.len() > 10 {
            result.push(format!("... +{} more", lines.len() - 10));
        }
    }

    result.join("\n")
}

/// Filter reg output - registry queries.
fn filter_reg(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.is_empty() {
        return "No data".to_string();
    }

    let mut result = vec![format!("{} entries:", lines.len())];

    for line in lines.iter().take(15) {
        let truncated = if line.len() > 80 {
            format!("{}...", &line[..77])
        } else {
            line.to_string()
        };
        result.push(truncated);
    }

    if lines.len() > 15 {
        result.push(format!("... +{} more", lines.len() - 15));
    }

    result.join("\n")
}

/// Filter dism output - deployment image servicing.
fn filter_dism(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty() && !l.contains("===="))
        .collect();

    if lines.is_empty() {
        return "No output".to_string();
    }

    let mut result = Vec::new();

    // Look for key info
    for line in &lines {
        if line.contains("Version") || line.contains("State") ||
           line.contains("Feature") || line.contains("Package") ||
           line.contains("Image") || line.contains(":") {
            if result.len() < 15 {
                result.push(line.trim().to_string());
            }
        }
    }

    if result.is_empty() {
        result = lines.iter().take(10).map(|s| s.to_string()).collect();
    }

    if lines.len() > result.len() {
        result.push(format!("... +{} more lines", lines.len() - result.len()));
    }

    result.join("\n")
}

/// Filter sfc output - system file checker.
fn filter_sfc(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    // Extract key status
    let mut result = Vec::new();

    for line in &lines {
        let lower = line.to_lowercase();
        if lower.contains("verification") || lower.contains("integrity") ||
           lower.contains("found") || lower.contains("repaired") ||
           lower.contains("could not") || lower.contains("windows resource") {
            result.push(line.trim().to_string());
        }
    }

    if result.is_empty() {
        result = lines.iter().take(5).map(|s| s.to_string()).collect();
    }

    result.join("\n")
}

/// Filter getmac output - MAC addresses.
fn filter_getmac(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty() && !l.contains("===") && !l.contains("Physical"))
        .collect();

    if lines.is_empty() {
        return "No adapters".to_string();
    }

    let mac_re = Regex::new(r"([0-9A-Fa-f]{2}[:-]){5}[0-9A-Fa-f]{2}").unwrap();

    let mut result = vec![format!("{} adapters:", lines.len())];

    for line in &lines {
        if let Some(mac) = mac_re.find(line) {
            result.push(format!("  {}", mac.as_str()));
        }
    }

    result.join("\n")
}

/// Filter arp output - ARP table.
fn filter_arp(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    let mut entries = 0;
    let mut dynamic = 0;
    let mut static_entries = 0;

    for line in &lines {
        if line.contains("dynamic") || line.contains("dinâmico") {
            entries += 1;
            dynamic += 1;
        } else if line.contains("static") || line.contains("estático") {
            entries += 1;
            static_entries += 1;
        }
    }

    let mut result = vec![format!("ARP: {} entries ({} dynamic, {} static)", entries, dynamic, static_entries)];

    // Show first few
    let mut count = 0;
    for line in &lines {
        if (line.contains("dynamic") || line.contains("static") ||
            line.contains("dinâmico") || line.contains("estático")) && count < 10 {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                result.push(format!("  {} → {}", parts[0], parts[1]));
                count += 1;
            }
        }
    }

    if entries > 10 {
        result.push(format!("  ... +{} more", entries - 10));
    }

    result.join("\n")
}

/// Filter route output - routing table.
fn filter_route(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty() && !l.contains("==="))
        .collect();

    let mut routes = 0;

    for line in &lines {
        // Count actual route entries (start with IP)
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            if parts[0].chars().next().map_or(false, |c| c.is_numeric()) {
                routes += 1;
            }
        }
    }

    let mut result = vec![format!("{} routes", routes)];

    // Show key routes (default gateway, etc)
    for line in &lines {
        if line.contains("0.0.0.0") || line.contains("default") ||
           line.contains("On-link") {
            if result.len() < 12 {
                let truncated = if line.len() > 70 {
                    format!("{}...", &line[..67])
                } else {
                    line.to_string()
                };
                result.push(truncated);
            }
        }
    }

    if routes > 10 {
        result.push(format!("... +{} more routes", routes - 10));
    }

    result.join("\n")
}

// ============================================================================
// Phase 1 Remaining: CMD Filters
// ============================================================================

/// Filter diskpart output - disk management.
fn filter_diskpart(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.is_empty() {
        return "No output".to_string();
    }

    let mut result = Vec::new();
    let mut disks = 0;
    let mut volumes = 0;
    let mut partitions = 0;

    for line in &lines {
        if line.contains("Disk ###") || line.contains("Disco ###") {
            disks += 1;
        }
        if line.contains("Volume ###") {
            volumes += 1;
        }
        if line.contains("Partition ###") || line.contains("Partição") {
            partitions += 1;
        }
    }

    if disks > 0 || volumes > 0 || partitions > 0 {
        result.push(format!("{} disks, {} volumes, {} partitions", disks, volumes, partitions));
    }

    // Show first relevant lines
    for line in lines.iter().take(15) {
        if !line.contains("DISKPART>") && !line.contains("Microsoft") {
            result.push(line.to_string());
        }
    }

    if lines.len() > 15 {
        result.push(format!("... +{} more lines", lines.len() - 15));
    }

    if result.is_empty() {
        lines.iter().take(10).map(|s| s.to_string()).collect::<Vec<_>>().join("\n")
    } else {
        result.join("\n")
    }
}

/// Filter bcdedit output - boot configuration.
fn filter_bcdedit(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.is_empty() {
        return "No boot entries".to_string();
    }

    let mut result = Vec::new();
    let mut entries = 0;

    for line in &lines {
        if line.contains("identifier") || line.contains("identificador") {
            entries += 1;
        }
    }

    result.push(format!("{} boot entries", entries));

    // Extract key info
    for line in &lines {
        if line.contains("identifier") || line.contains("device") ||
           line.contains("path") || line.contains("description") ||
           line.contains("osdevice") || line.contains("default") {
            if result.len() < 15 {
                let truncated = if line.len() > 70 {
                    format!("{}...", &line[..67])
                } else {
                    line.to_string()
                };
                result.push(truncated);
            }
        }
    }

    if entries > 5 {
        result.push(format!("... +{} more entries", entries - 5));
    }

    result.join("\n")
}

/// Filter certutil output - certificate management.
fn filter_certutil(stdout: &str, args: &[String]) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty() && !l.contains("===="))
        .collect();

    if lines.is_empty() {
        return "No output".to_string();
    }

    let subcmd = args.first().map(|s| s.to_lowercase()).unwrap_or_default();

    let mut result = Vec::new();

    if subcmd.contains("store") || subcmd.contains("verify") {
        // Certificate store - count certs
        let mut certs = 0;
        for line in &lines {
            if line.contains("Serial Number") || line.contains("Número de Série") {
                certs += 1;
            }
        }
        result.push(format!("{} certificates", certs));

        // Show key info
        for line in &lines {
            if line.contains("Subject") || line.contains("Issuer") ||
               line.contains("NotAfter") || line.contains("Serial") {
                if result.len() < 12 {
                    result.push(line.trim().to_string());
                }
            }
        }
    } else {
        // Generic - truncate
        for line in lines.iter().take(15) {
            result.push(line.to_string());
        }
    }

    if lines.len() > result.len() {
        result.push(format!("... +{} more lines", lines.len() - result.len()));
    }

    result.join("\n")
}

/// Filter fsutil output - file system utilities.
fn filter_fsutil(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.is_empty() {
        return "No output".to_string();
    }

    let mut result = Vec::new();

    // Extract key info lines
    for line in &lines {
        if line.contains(":") || line.contains("=") {
            if result.len() < 15 {
                let truncated = if line.len() > 70 {
                    format!("{}...", &line[..67])
                } else {
                    line.to_string()
                };
                result.push(truncated);
            }
        }
    }

    if result.is_empty() {
        result = lines.iter().take(10).map(|s| s.to_string()).collect();
    }

    if lines.len() > result.len() {
        result.push(format!("... +{} more", lines.len() - result.len()));
    }

    result.join("\n")
}

/// Filter icacls output - file permissions.
fn filter_icacls(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.is_empty() {
        return "No output".to_string();
    }

    let mut files = 0;
    let mut result = Vec::new();

    // Count files processed
    for line in &lines {
        if line.contains("Successfully processed") || line.contains("processados com êxito") {
            result.push(line.trim().to_string());
        }
        if !line.starts_with(' ') && !line.contains("Successfully") {
            files += 1;
        }
    }

    if files > 0 && result.is_empty() {
        result.push(format!("{} files", files));
    }

    // Show first few permission entries
    for line in lines.iter().take(10) {
        if !result.contains(&line.to_string()) {
            let truncated = if line.len() > 70 {
                format!("{}...", &line[..67])
            } else {
                line.to_string()
            };
            result.push(truncated);
        }
    }

    if lines.len() > 10 {
        result.push(format!("... +{} more", lines.len() - 10));
    }

    result.join("\n")
}

/// Filter attrib output - file attributes.
fn filter_attrib(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.is_empty() {
        return "No files".to_string();
    }

    let mut result = vec![format!("{} files", lines.len())];

    // Show first few with attributes
    for line in lines.iter().take(15) {
        let truncated = if line.len() > 70 {
            format!("{}...", &line[..67])
        } else {
            line.to_string()
        };
        result.push(truncated);
    }

    if lines.len() > 15 {
        result.push(format!("... +{} more", lines.len() - 15));
    }

    result.join("\n")
}

/// Filter findstr output - pattern search.
fn filter_findstr(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.is_empty() {
        return "No matches".to_string();
    }

    let mut result = vec![format!("{} matches", lines.len())];

    // Group by file if possible
    let mut current_file = String::new();
    let mut file_matches = 0;

    for line in &lines {
        if let Some(colon_pos) = line.find(':') {
            let file = &line[..colon_pos];
            if file != current_file {
                if !current_file.is_empty() && file_matches > 1 {
                    // Previous file had multiple matches
                }
                current_file = file.to_string();
                file_matches = 0;
            }
            file_matches += 1;
        }

        if result.len() < 15 {
            let truncated = if line.len() > 80 {
                format!("{}...", &line[..77])
            } else {
                line.to_string()
            };
            result.push(truncated);
        }
    }

    if lines.len() > 14 {
        result.push(format!("... +{} more matches", lines.len() - 14));
    }

    result.join("\n")
}

/// Filter robocopy output - robust file copy.
fn filter_robocopy(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.is_empty() {
        return "No output".to_string();
    }

    let mut result = Vec::new();

    // Extract summary stats
    let mut dirs_total = 0;
    let mut files_total = 0;
    let mut bytes_total = 0u64;

    for line in &lines {
        // Look for summary lines
        if line.contains("Dirs :") || line.contains("Pastas :") {
            result.push(line.trim().to_string());
        }
        if line.contains("Files :") || line.contains("Arquivos :") {
            result.push(line.trim().to_string());
        }
        if line.contains("Bytes :") {
            result.push(line.trim().to_string());
        }
        if line.contains("Times :") || line.contains("Speed :") {
            result.push(line.trim().to_string());
        }
        if line.contains("Ended :") || line.contains("Started :") {
            result.push(line.trim().to_string());
        }
    }

    // If no summary found, show truncated output
    if result.is_empty() {
        for line in lines.iter().take(15) {
            if !line.contains("----") && !line.contains("ROBOCOPY") {
                result.push(line.to_string());
            }
        }
        if lines.len() > 15 {
            result.push(format!("... +{} more lines", lines.len() - 15));
        }
    }

    result.join("\n")
}
