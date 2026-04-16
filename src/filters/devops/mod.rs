//! DevOps tools filters - Phase 2.

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

// ============================================================================
// Vagrant Filter
// ============================================================================

pub struct VagrantFilter;

impl Filter for VagrantFilter {
    fn name(&self) -> &'static str {
        "vagrant"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "vagrant" || cmd == "vagrant.exe"
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
            "status" => filter_vagrant_status(&stdout, &stderr),
            "up" => filter_vagrant_up(&stdout, &stderr),
            "halt" | "suspend" | "destroy" => filter_vagrant_action(&stdout, &stderr, subcommand),
            "ssh-config" => filter_vagrant_ssh_config(&stdout),
            "box" => filter_vagrant_box(&stdout, &stderr, args),
            "global-status" => filter_vagrant_global_status(&stdout),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_vagrant_status(stdout: &str, stderr: &str) -> String {
    let state_re = Regex::new(r"(\S+)\s+(running|poweroff|saved|not created|aborted)\s+\((\w+)\)").unwrap();

    let mut machines = Vec::new();
    for caps in state_re.captures_iter(stdout) {
        let icon = match &caps[2] {
            "running" => "R",
            "poweroff" => "-",
            "saved" => "S",
            "not created" => "x",
            _ => "?"
        };
        machines.push(format!("[{}] {} ({})", icon, &caps[1], &caps[3]));
    }

    if !machines.is_empty() {
        machines.join("\n")
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_vagrant_up(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("Machine booted and ready") {
        "VM ready".to_string()
    } else if combined.contains("Bringing machine") {
        let importing = combined.contains("Importing base box");
        let booting = combined.contains("Booting VM");

        if importing { "Importing box...".to_string() }
        else if booting { "Booting...".to_string() }
        else { "Starting...".to_string() }
    } else if combined.contains("error") || combined.contains("Error") {
        let err_re = Regex::new(r"(?i)error[:\s]+(.+)").unwrap();
        if let Some(caps) = err_re.captures(&combined) {
            format!("X {}", truncate(&caps[1], 60))
        } else {
            "X Failed".to_string()
        }
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_vagrant_action(stdout: &str, stderr: &str, action: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("Gracefully halting") || combined.contains("halted") {
        "VM halted".to_string()
    } else if combined.contains("Saving VM state") || combined.contains("saved") {
        "VM saved".to_string()
    } else if combined.contains("Destroying VM") || combined.contains("destroyed") {
        "VM destroyed".to_string()
    } else {
        format!("{} complete", action)
    }
}

fn filter_vagrant_ssh_config(stdout: &str) -> String {
    let host_re = Regex::new(r"Host\s+(\S+)").unwrap();
    let hostname_re = Regex::new(r"HostName\s+(\S+)").unwrap();
    let port_re = Regex::new(r"Port\s+(\d+)").unwrap();
    let user_re = Regex::new(r"User\s+(\S+)").unwrap();

    let host = host_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();
    let hostname = hostname_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();
    let port = port_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();
    let user = user_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();

    if !host.is_empty() {
        format!("ssh {}@{}:{} (Host: {})", user, hostname, port, host)
    } else {
        truncate(stdout, 200)
    }
}

fn filter_vagrant_box(stdout: &str, _stderr: &str, args: &[String]) -> String {
    let subcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subcmd {
        "list" => {
            let boxes: Vec<&str> = stdout.lines()
                .filter(|l| !l.trim().is_empty() && !l.contains("---"))
                .collect();
            format!("{} boxes\n{}", boxes.len(), boxes.iter().take(10).map(|b| format!("  {}", b)).collect::<Vec<_>>().join("\n"))
        }
        _ => truncate(stdout, 200)
    }
}

fn filter_vagrant_global_status(stdout: &str) -> String {
    let vm_re = Regex::new(r"([a-f0-9]+)\s+(\S+)\s+(\S+)\s+(\S+)\s+(.+)").unwrap();

    let mut vms = Vec::new();
    for caps in vm_re.captures_iter(stdout) {
        let state = &caps[3];
        let icon = match state {
            "running" => "R",
            "poweroff" => "-",
            _ => "?"
        };
        vms.push(format!("[{}] {} - {}", icon, &caps[2], truncate(&caps[5], 40)));
    }

    if !vms.is_empty() {
        format!("{} VMs\n{}", vms.len(), vms.join("\n"))
    } else {
        "No VMs".to_string()
    }
}

// ============================================================================
// Packer Filter
// ============================================================================

pub struct PackerFilter;

impl Filter for PackerFilter {
    fn name(&self) -> &'static str {
        "packer"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "packer" || cmd == "packer.exe"
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
            "build" => filter_packer_build(&stdout, &stderr),
            "validate" => filter_packer_validate(&stdout, &stderr),
            "init" => filter_packer_init(&stdout, &stderr),
            "fmt" => filter_packer_fmt(&stdout),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_packer_build(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let artifact_re = Regex::new(r"--> (\S+): (.+)").unwrap();
    let build_re = Regex::new(r"Build '(\S+)' finished").unwrap();
    let error_re = Regex::new(r"(?i)error[:\s]+(.+)").unwrap();

    let mut artifacts = Vec::new();
    for caps in artifact_re.captures_iter(&combined) {
        artifacts.push(format!("{}: {}", &caps[1], truncate(&caps[2], 50)));
    }

    let builds: Vec<String> = build_re.captures_iter(&combined)
        .map(|c| c[1].to_string())
        .collect();

    if !builds.is_empty() {
        let mut result = vec![format!("{} builds completed", builds.len())];
        for a in artifacts.iter().take(5) {
            result.push(format!("  {}", a));
        }
        result.join("\n")
    } else if let Some(caps) = error_re.captures(&combined) {
        format!("X {}", truncate(&caps[1], 60))
    } else if combined.contains("Starting") || combined.contains("Creating") {
        "Building...".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_packer_validate(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("valid") && !combined.contains("invalid") {
        "Valid".to_string()
    } else if combined.contains("error") || combined.contains("Error") {
        let err_re = Regex::new(r"(?i)error[:\s]+(.+)").unwrap();
        let errors: Vec<String> = err_re.captures_iter(&combined)
            .map(|c| truncate(&c[1], 60))
            .collect();
        format!("X {} errors\n{}", errors.len(), errors.iter().take(5).map(|e| format!("  {}", e)).collect::<Vec<_>>().join("\n"))
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_packer_init(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);
    let installed_re = Regex::new(r"Installed plugin (.+)").unwrap();

    let plugins: Vec<String> = installed_re.captures_iter(&combined)
        .map(|c| c[1].to_string())
        .collect();

    if !plugins.is_empty() {
        format!("{} plugins installed", plugins.len())
    } else if combined.contains("already installed") {
        "Plugins up to date".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_packer_fmt(stdout: &str) -> String {
    let files: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    if files.is_empty() {
        "No files changed".to_string()
    } else {
        format!("{} files formatted", files.len())
    }
}

// ============================================================================
// Pulumi Filter
// ============================================================================

pub struct PulumiFilter;

impl Filter for PulumiFilter {
    fn name(&self) -> &'static str {
        "pulumi"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "pulumi" || cmd == "pulumi.exe"
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
            "preview" => filter_pulumi_preview(&stdout, &stderr),
            "up" => filter_pulumi_up(&stdout, &stderr),
            "destroy" => filter_pulumi_destroy(&stdout, &stderr),
            "stack" => filter_pulumi_stack(&stdout, args),
            "config" => filter_pulumi_config(&stdout),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_pulumi_preview(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let changes_re = Regex::new(r"(\d+) to create, (\d+) to update, (\d+) to delete").unwrap();
    let resource_re = Regex::new(r"\s+([+-~])\s+(\S+)\s+(\S+)").unwrap();

    if let Some(caps) = changes_re.captures(&combined) {
        let mut result = vec![format!("+{} ~{} -{}", &caps[1], &caps[2], &caps[3])];

        for (i, caps) in resource_re.captures_iter(&combined).enumerate() {
            if i >= 10 {
                result.push("  ...more".to_string());
                break;
            }
            result.push(format!("  {} {}", &caps[1], truncate(&caps[3], 50)));
        }
        result.join("\n")
    } else if combined.contains("no changes") {
        "No changes".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_pulumi_up(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let complete_re = Regex::new(r"(\d+) created, (\d+) updated, (\d+) deleted").unwrap();

    if let Some(caps) = complete_re.captures(&combined) {
        format!("+{} ~{} -{}", &caps[1], &caps[2], &caps[3])
    } else if combined.contains("Updating") {
        "Updating...".to_string()
    } else if combined.contains("error") {
        let err_re = Regex::new(r"error: (.+)").unwrap();
        if let Some(caps) = err_re.captures(&combined) {
            format!("X {}", truncate(&caps[1], 60))
        } else {
            "X Failed".to_string()
        }
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_pulumi_destroy(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let deleted_re = Regex::new(r"(\d+) deleted").unwrap();

    if let Some(caps) = deleted_re.captures(&combined) {
        format!("{} deleted", &caps[1])
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_pulumi_stack(stdout: &str, args: &[String]) -> String {
    let subcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subcmd {
        "ls" | "list" => {
            let stacks: Vec<&str> = stdout.lines()
                .filter(|l| !l.trim().is_empty() && !l.contains("NAME"))
                .collect();
            format!("{} stacks\n{}", stacks.len(), stacks.iter().take(10).map(|s| format!("  {}", s)).collect::<Vec<_>>().join("\n"))
        }
        _ => truncate(stdout, 300)
    }
}

fn filter_pulumi_config(stdout: &str) -> String {
    let configs: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty() && !l.contains("KEY"))
        .collect();

    if configs.is_empty() {
        "No config".to_string()
    } else {
        format!("{} config values\n{}", configs.len(), configs.iter().take(10).map(|c| format!("  {}", c)).collect::<Vec<_>>().join("\n"))
    }
}

// ============================================================================
// Serverless Framework Filter
// ============================================================================

pub struct ServerlessFilter;

impl Filter for ServerlessFilter {
    fn name(&self) -> &'static str {
        "serverless"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "serverless" || cmd == "sls" || cmd == "serverless.cmd" || cmd == "sls.cmd"
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
            "deploy" => filter_serverless_deploy(&stdout, &stderr),
            "remove" => filter_serverless_remove(&stdout, &stderr),
            "invoke" => filter_serverless_invoke(&stdout),
            "logs" => filter_serverless_logs(&stdout),
            "info" => filter_serverless_info(&stdout),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_serverless_deploy(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let endpoint_re = Regex::new(r"(GET|POST|PUT|DELETE|PATCH)\s+-\s+(\S+)").unwrap();
    let function_re = Regex::new(r"functions:\s*\n((?:\s+\S+:\s+\S+\n?)+)").unwrap();

    let mut endpoints = Vec::new();
    for caps in endpoint_re.captures_iter(&combined) {
        endpoints.push(format!("{} {}", &caps[1], truncate(&caps[2], 50)));
    }

    if combined.contains("Service deployed") || combined.contains("Deploying") && combined.contains("complete") {
        let mut result = vec!["Deployed".to_string()];
        if !endpoints.is_empty() {
            result.push(format!("{} endpoints:", endpoints.len()));
            for e in endpoints.iter().take(5) {
                result.push(format!("  {}", e));
            }
        }
        result.join("\n")
    } else if combined.contains("error") {
        let err_re = Regex::new(r"Error[:\s]+(.+)").unwrap();
        if let Some(caps) = err_re.captures(&combined) {
            format!("X {}", truncate(&caps[1], 60))
        } else {
            "X Deploy failed".to_string()
        }
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_serverless_remove(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("removed") || combined.contains("Service removed") {
        "Service removed".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_serverless_invoke(stdout: &str) -> String {
    // Show response but truncate if too long
    if stdout.len() > 500 {
        format!("{}...\n({} chars)", &stdout[..500], stdout.len())
    } else {
        stdout.to_string()
    }
}

fn filter_serverless_logs(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().collect();
    if lines.len() > 20 {
        let mut result: Vec<String> = lines.iter().take(15).map(|s| s.to_string()).collect();
        result.push(format!("...+{} more", lines.len() - 15));
        result.join("\n")
    } else {
        stdout.to_string()
    }
}

fn filter_serverless_info(stdout: &str) -> String {
    let service_re = Regex::new(r"service:\s*(\S+)").unwrap();
    let stage_re = Regex::new(r"stage:\s*(\S+)").unwrap();
    let region_re = Regex::new(r"region:\s*(\S+)").unwrap();
    let endpoint_re = Regex::new(r"(GET|POST|PUT|DELETE)\s+-\s+(\S+)").unwrap();

    let service = service_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();
    let stage = stage_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();
    let region = region_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();

    let endpoints: Vec<String> = endpoint_re.captures_iter(stdout)
        .map(|c| format!("{} {}", &c[1], truncate(&c[2], 40)))
        .collect();

    let mut result = Vec::new();
    if !service.is_empty() {
        result.push(format!("{} ({}/{})", service, stage, region));
    }
    if !endpoints.is_empty() {
        result.push(format!("{} endpoints", endpoints.len()));
        for e in endpoints.iter().take(5) {
            result.push(format!("  {}", e));
        }
    }

    if result.is_empty() {
        truncate(stdout, 300)
    } else {
        result.join("\n")
    }
}

// ============================================================================
// PaaS CLIs (Vercel, Netlify, Railway, Fly, Render, Heroku)
// ============================================================================

pub struct VercelFilter;

impl Filter for VercelFilter {
    fn name(&self) -> &'static str { "vercel" }
    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "vercel" || cmd == "vercel.cmd" || cmd == "vc"
    }
    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        execute_paas_filter(command, args, "vercel")
    }
    fn priority(&self) -> u8 { 85 }
}

pub struct NetlifyFilter;

impl Filter for NetlifyFilter {
    fn name(&self) -> &'static str { "netlify" }
    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "netlify" || cmd == "ntl" || cmd == "netlify.cmd"
    }
    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        execute_paas_filter(command, args, "netlify")
    }
    fn priority(&self) -> u8 { 85 }
}

pub struct RailwayFilter;

impl Filter for RailwayFilter {
    fn name(&self) -> &'static str { "railway" }
    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "railway" || cmd == "rail"
    }
    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        execute_paas_filter(command, args, "railway")
    }
    fn priority(&self) -> u8 { 85 }
}

pub struct FlyctlFilter;

impl Filter for FlyctlFilter {
    fn name(&self) -> &'static str { "flyctl" }
    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "flyctl" || cmd == "fly"
    }
    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        execute_paas_filter(command, args, "fly")
    }
    fn priority(&self) -> u8 { 85 }
}

pub struct RenderFilter;

impl Filter for RenderFilter {
    fn name(&self) -> &'static str { "render" }
    fn matches(&self, command: &str) -> bool {
        command.to_lowercase() == "render"
    }
    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        execute_paas_filter(command, args, "render")
    }
    fn priority(&self) -> u8 { 85 }
}

pub struct HerokuFilter;

impl Filter for HerokuFilter {
    fn name(&self) -> &'static str { "heroku" }
    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "heroku" || cmd == "heroku.cmd"
    }
    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        execute_paas_filter(command, args, "heroku")
    }
    fn priority(&self) -> u8 { 85 }
}

fn execute_paas_filter(command: &str, args: &[String], _provider: &str) -> Result<FilterResult> {
    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("");

    let start = Instant::now();
    let output = Command::new(command).args(args).output()?;
    let exec_time_ms = start.elapsed().as_millis() as u64;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let input_chars = stdout.len() + stderr.len();

    let filtered = match subcommand {
        "deploy" | "up" | "push" => filter_paas_deploy(&stdout, &stderr),
        "logs" => filter_paas_logs(&stdout),
        "status" | "info" | "list" | "ls" => filter_paas_status(&stdout),
        "env" | "config" | "secrets" => filter_paas_env(&stdout),
        _ => filter_generic(&stdout, &stderr),
    };

    Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
}

fn filter_paas_deploy(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let url_re = Regex::new(r"https?://\S+").unwrap();

    if combined.contains("Deployed") || combined.contains("deployed") || combined.contains("Production") {
        if let Some(url) = url_re.find(&combined) {
            format!("Deployed: {}", url.as_str())
        } else {
            "Deployed".to_string()
        }
    } else if combined.contains("Building") || combined.contains("Uploading") {
        "Deploying...".to_string()
    } else if combined.contains("error") || combined.contains("Error") {
        let err_re = Regex::new(r"(?i)error[:\s]+(.+)").unwrap();
        if let Some(caps) = err_re.captures(&combined) {
            format!("X {}", truncate(&caps[1], 60))
        } else {
            "X Failed".to_string()
        }
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_paas_logs(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().collect();
    if lines.len() > 30 {
        let mut result: Vec<String> = lines.iter().rev().take(20).rev().map(|s| s.to_string()).collect();
        result.push(format!("...showing last 20 of {} lines", lines.len()));
        result.join("\n")
    } else {
        stdout.to_string()
    }
}

fn filter_paas_status(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.len() > 15 {
        let mut result: Vec<String> = lines.iter().take(12).map(|s| s.to_string()).collect();
        result.push(format!("...+{} more", lines.len() - 12));
        result.join("\n")
    } else {
        lines.join("\n")
    }
}

fn filter_paas_env(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    // Mask values for security
    let mut result = vec![format!("{} env vars", lines.len())];
    for line in lines.iter().take(10) {
        if let Some((key, _)) = line.split_once('=') {
            result.push(format!("  {}=***", key.trim()));
        } else {
            result.push(format!("  {}", truncate(line, 40)));
        }
    }
    if lines.len() > 10 {
        result.push(format!("  ...+{} more", lines.len() - 10));
    }
    result.join("\n")
}

// ============================================================================
// Local Kubernetes (minikube, kind, k3s)
// ============================================================================

pub struct MinikubeFilter;

impl Filter for MinikubeFilter {
    fn name(&self) -> &'static str { "minikube" }
    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "minikube" || cmd == "minikube.exe"
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
            "status" => filter_minikube_status(&stdout, &stderr),
            "start" => filter_minikube_start(&stdout, &stderr),
            "stop" | "delete" => filter_minikube_stop(&stdout, &stderr, subcommand),
            "dashboard" => "Dashboard opened".to_string(),
            "ip" => stdout.trim().to_string(),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }
    fn priority(&self) -> u8 { 85 }
}

fn filter_minikube_status(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let host_re = Regex::new(r"host:\s*(\S+)").unwrap();
    let kubelet_re = Regex::new(r"kubelet:\s*(\S+)").unwrap();
    let apiserver_re = Regex::new(r"apiserver:\s*(\S+)").unwrap();

    let host = host_re.captures(&combined).map(|c| c[1].to_string()).unwrap_or_default();
    let kubelet = kubelet_re.captures(&combined).map(|c| c[1].to_string()).unwrap_or_default();
    let apiserver = apiserver_re.captures(&combined).map(|c| c[1].to_string()).unwrap_or_default();

    if !host.is_empty() {
        format!("host:{} kubelet:{} api:{}", host, kubelet, apiserver)
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_minikube_start(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("Done!") || combined.contains("kubectl is now configured") {
        "Minikube ready".to_string()
    } else if combined.contains("Starting") {
        "Starting...".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_minikube_stop(stdout: &str, stderr: &str, action: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("Stopping") || combined.contains("stopped") {
        "Stopped".to_string()
    } else if combined.contains("Deleting") || combined.contains("deleted") {
        "Deleted".to_string()
    } else {
        format!("{} complete", action)
    }
}

pub struct KindFilter;

impl Filter for KindFilter {
    fn name(&self) -> &'static str { "kind" }
    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "kind" || cmd == "kind.exe"
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
            "create" => filter_kind_create(&stdout, &stderr),
            "delete" => filter_kind_delete(&stdout, &stderr),
            "get" => filter_kind_get(&stdout, args),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }
    fn priority(&self) -> u8 { 85 }
}

fn filter_kind_create(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("ready") || combined.contains("You can now use your cluster") {
        "Cluster ready".to_string()
    } else if combined.contains("Creating cluster") {
        "Creating...".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_kind_delete(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("Deleting cluster") || combined.contains("deleted") {
        "Cluster deleted".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_kind_get(stdout: &str, args: &[String]) -> String {
    let subcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subcmd {
        "clusters" => {
            let clusters: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
            if clusters.is_empty() {
                "No clusters".to_string()
            } else {
                format!("{} clusters: {}", clusters.len(), clusters.join(", "))
            }
        }
        "nodes" => {
            let nodes: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
            format!("{} nodes", nodes.len())
        }
        _ => truncate(stdout, 200)
    }
}

pub struct K3sFilter;

impl Filter for K3sFilter {
    fn name(&self) -> &'static str { "k3s" }
    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "k3s" || cmd == "k3s.exe"
    }
    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = filter_generic(&stdout, &stderr);
        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }
    fn priority(&self) -> u8 { 85 }
}

// ============================================================================
// K8s Dev Tools (Skaffold, Tilt)
// ============================================================================

pub struct SkaffoldFilter;

impl Filter for SkaffoldFilter {
    fn name(&self) -> &'static str { "skaffold" }
    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "skaffold" || cmd == "skaffold.exe"
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
            "build" => filter_skaffold_build(&stdout, &stderr),
            "deploy" => filter_skaffold_deploy(&stdout, &stderr),
            "run" => filter_skaffold_run(&stdout, &stderr),
            "dev" => "Dev mode running...".to_string(),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }
    fn priority(&self) -> u8 { 85 }
}

fn filter_skaffold_build(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let image_re = Regex::new(r"Build \[(\S+)\] succeeded").unwrap();
    let images: Vec<String> = image_re.captures_iter(&combined)
        .map(|c| c[1].to_string())
        .collect();

    if !images.is_empty() {
        format!("{} images built\n{}", images.len(), images.iter().take(5).map(|i| format!("  {}", i)).collect::<Vec<_>>().join("\n"))
    } else if combined.contains("error") {
        let err_re = Regex::new(r"(?i)error[:\s]+(.+)").unwrap();
        if let Some(caps) = err_re.captures(&combined) {
            format!("X {}", truncate(&caps[1], 60))
        } else {
            "X Build failed".to_string()
        }
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_skaffold_deploy(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("Deployments stabilized") || combined.contains("successfully deployed") {
        "Deployed".to_string()
    } else if combined.contains("Deploying") {
        "Deploying...".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_skaffold_run(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("Deployments stabilized") {
        "Build & deploy complete".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

pub struct TiltFilter;

impl Filter for TiltFilter {
    fn name(&self) -> &'static str { "tilt" }
    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "tilt" || cmd == "tilt.exe"
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
            "up" => "Tilt UI running".to_string(),
            "down" => "Tilt stopped".to_string(),
            "ci" => filter_tilt_ci(&stdout, &stderr),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }
    fn priority(&self) -> u8 { 85 }
}

fn filter_tilt_ci(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("SUCCESS") || combined.contains("All resources ready") {
        "CI passed".to_string()
    } else if combined.contains("FAILURE") || combined.contains("error") {
        "CI failed".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

// ============================================================================
// GitOps & Service Mesh (ArgoCD, Istio, Linkerd)
// ============================================================================

pub struct ArgoCDFilter;

impl Filter for ArgoCDFilter {
    fn name(&self) -> &'static str { "argocd" }
    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "argocd" || cmd == "argocd.exe"
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
            "app" => filter_argocd_app(&stdout, &stderr, args),
            "repo" => filter_argocd_repo(&stdout, args),
            "cluster" => filter_argocd_cluster(&stdout, args),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }
    fn priority(&self) -> u8 { 85 }
}

fn filter_argocd_app(stdout: &str, stderr: &str, args: &[String]) -> String {
    let subcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subcmd {
        "list" => {
            let apps: Vec<&str> = stdout.lines()
                .filter(|l| !l.trim().is_empty() && !l.contains("NAME"))
                .collect();
            format!("{} apps\n{}", apps.len(), apps.iter().take(10).map(|a| format!("  {}", truncate(a, 60))).collect::<Vec<_>>().join("\n"))
        }
        "sync" => {
            if stdout.contains("succeeded") || stdout.contains("Synced") {
                "Synced".to_string()
            } else {
                filter_generic(stdout, stderr)
            }
        }
        "get" => {
            let health_re = Regex::new(r"Health Status:\s*(\S+)").unwrap();
            let sync_re = Regex::new(r"Sync Status:\s*(\S+)").unwrap();

            let health = health_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();
            let sync = sync_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();

            if !health.is_empty() {
                format!("Health:{} Sync:{}", health, sync)
            } else {
                truncate(stdout, 200)
            }
        }
        _ => filter_generic(stdout, stderr)
    }
}

fn filter_argocd_repo(stdout: &str, args: &[String]) -> String {
    let subcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subcmd {
        "list" => {
            let repos: Vec<&str> = stdout.lines()
                .filter(|l| !l.trim().is_empty() && !l.contains("REPO"))
                .collect();
            format!("{} repos", repos.len())
        }
        _ => truncate(stdout, 200)
    }
}

fn filter_argocd_cluster(stdout: &str, args: &[String]) -> String {
    let subcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subcmd {
        "list" => {
            let clusters: Vec<&str> = stdout.lines()
                .filter(|l| !l.trim().is_empty() && !l.contains("SERVER"))
                .collect();
            format!("{} clusters", clusters.len())
        }
        _ => truncate(stdout, 200)
    }
}

pub struct IstioFilter;

impl Filter for IstioFilter {
    fn name(&self) -> &'static str { "istioctl" }
    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "istioctl" || cmd == "istioctl.exe"
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
            "analyze" => filter_istio_analyze(&stdout, &stderr),
            "proxy-status" => filter_istio_proxy_status(&stdout),
            "install" => filter_istio_install(&stdout, &stderr),
            "version" => filter_istio_version(&stdout),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }
    fn priority(&self) -> u8 { 85 }
}

fn filter_istio_analyze(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let error_re = Regex::new(r"Error \[(\S+)\]").unwrap();
    let warn_re = Regex::new(r"Warning \[(\S+)\]").unwrap();

    let errors: Vec<String> = error_re.captures_iter(&combined).map(|c| c[1].to_string()).collect();
    let warnings: Vec<String> = warn_re.captures_iter(&combined).map(|c| c[1].to_string()).collect();

    if errors.is_empty() && warnings.is_empty() {
        "No issues found".to_string()
    } else {
        format!("{} errors, {} warnings", errors.len(), warnings.len())
    }
}

fn filter_istio_proxy_status(stdout: &str) -> String {
    let proxies: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty() && !l.contains("NAME"))
        .collect();

    let synced = proxies.iter().filter(|p| p.contains("SYNCED")).count();
    format!("{}/{} proxies synced", synced, proxies.len())
}

fn filter_istio_install(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("Installation complete") {
        "Installed".to_string()
    } else if combined.contains("error") {
        "Install failed".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_istio_version(stdout: &str) -> String {
    let version_re = Regex::new(r"(\d+\.\d+\.\d+)").unwrap();
    if let Some(caps) = version_re.captures(stdout) {
        format!("v{}", &caps[1])
    } else {
        truncate(stdout, 100)
    }
}

pub struct LinkerdFilter;

impl Filter for LinkerdFilter {
    fn name(&self) -> &'static str { "linkerd" }
    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "linkerd" || cmd == "linkerd.exe"
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
            "check" => filter_linkerd_check(&stdout, &stderr),
            "stat" => filter_linkerd_stat(&stdout),
            "install" => filter_linkerd_install(&stdout, &stderr),
            "version" => truncate(stdout.trim(), 100),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }
    fn priority(&self) -> u8 { 85 }
}

fn filter_linkerd_check(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let pass_count = combined.matches("[ok]").count();
    let fail_count = combined.matches("[FAIL]").count();

    if fail_count == 0 {
        format!("{} checks passed", pass_count)
    } else {
        format!("{} passed, {} failed", pass_count, fail_count)
    }
}

fn filter_linkerd_stat(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.len() > 15 {
        let mut result: Vec<String> = lines.iter().take(12).map(|s| s.to_string()).collect();
        result.push(format!("...+{} more", lines.len() - 12));
        result.join("\n")
    } else {
        lines.join("\n")
    }
}

fn filter_linkerd_install(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("successfully") {
        "Installed".to_string()
    } else {
        // Install outputs YAML - just confirm
        if stdout.len() > 100 {
            "Install manifest generated".to_string()
        } else {
            filter_generic(stdout, stderr)
        }
    }
}

// ============================================================================
// Enterprise K8s CLIs (cf, oc, eksctl)
// ============================================================================

pub struct CloudFoundryFilter;

impl Filter for CloudFoundryFilter {
    fn name(&self) -> &'static str { "cf" }
    fn matches(&self, command: &str) -> bool {
        command.to_lowercase() == "cf"
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
            "apps" => filter_cf_apps(&stdout),
            "push" => filter_cf_push(&stdout, &stderr),
            "logs" => filter_paas_logs(&stdout),
            "services" => filter_cf_services(&stdout),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }
    fn priority(&self) -> u8 { 85 }
}

fn filter_cf_apps(stdout: &str) -> String {
    let apps: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty() && !l.contains("name") && !l.contains("Getting apps"))
        .collect();

    if apps.is_empty() {
        "No apps".to_string()
    } else {
        format!("{} apps\n{}", apps.len(), apps.iter().take(10).map(|a| format!("  {}", truncate(a, 60))).collect::<Vec<_>>().join("\n"))
    }
}

fn filter_cf_push(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("App started") || combined.contains("started") {
        let url_re = Regex::new(r"routes:\s*(\S+)").unwrap();
        if let Some(caps) = url_re.captures(&combined) {
            format!("Deployed: {}", &caps[1])
        } else {
            "Deployed".to_string()
        }
    } else if combined.contains("FAILED") {
        "Push failed".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_cf_services(stdout: &str) -> String {
    let services: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty() && !l.contains("name") && !l.contains("Getting services"))
        .collect();
    format!("{} services", services.len())
}

pub struct OpenShiftFilter;

impl Filter for OpenShiftFilter {
    fn name(&self) -> &'static str { "oc" }
    fn matches(&self, command: &str) -> bool {
        command.to_lowercase() == "oc"
    }
    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        // OC is similar to kubectl, reuse similar filtering
        let subcommand = args.first().map(|s| s.as_str()).unwrap_or("");

        let start = Instant::now();
        let output = Command::new(command).args(args).output()?;
        let exec_time_ms = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = match subcommand {
            "get" => filter_oc_get(&stdout),
            "status" => filter_oc_status(&stdout),
            "project" | "projects" => filter_oc_projects(&stdout),
            "new-app" | "new-build" => filter_oc_new(&stdout, &stderr),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }
    fn priority(&self) -> u8 { 85 }
}

fn filter_oc_get(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.len() > 15 {
        let mut result: Vec<String> = lines.iter().take(12).map(|s| s.to_string()).collect();
        result.push(format!("...+{} more", lines.len() - 12));
        result.join("\n")
    } else {
        lines.join("\n")
    }
}

fn filter_oc_status(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .take(15)
        .collect();
    lines.join("\n")
}

fn filter_oc_projects(stdout: &str) -> String {
    let projects: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    format!("{} projects", projects.len())
}

fn filter_oc_new(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("Success") || combined.contains("created") {
        "Created".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

pub struct EksctlFilter;

impl Filter for EksctlFilter {
    fn name(&self) -> &'static str { "eksctl" }
    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "eksctl" || cmd == "eksctl.exe"
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
            "create" => filter_eksctl_create(&stdout, &stderr),
            "delete" => filter_eksctl_delete(&stdout, &stderr),
            "get" => filter_eksctl_get(&stdout),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }
    fn priority(&self) -> u8 { 85 }
}

fn filter_eksctl_create(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("EKS cluster") && combined.contains("is ready") {
        "Cluster ready".to_string()
    } else if combined.contains("creating") {
        "Creating cluster...".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_eksctl_delete(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("deleted") {
        "Cluster deleted".to_string()
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_eksctl_get(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.len() > 10 {
        let mut result: Vec<String> = lines.iter().take(8).map(|s| s.to_string()).collect();
        result.push(format!("...+{} more", lines.len() - 8));
        result.join("\n")
    } else {
        lines.join("\n")
    }
}

// ============================================================================
// Helpers
// ============================================================================

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
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
