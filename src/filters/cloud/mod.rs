//! Cloud CLI filters (Azure, AWS, GCloud).

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for Azure CLI commands.
pub struct AzFilter;

impl Filter for AzFilter {
    fn name(&self) -> &'static str {
        "az"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "az" || cmd == "az.cmd" || cmd == "az.exe"
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
            "login" => filter_az_login(&stdout, &stderr),
            "account" => filter_az_account(&stdout, args),
            "group" => filter_az_group(&stdout, args),
            "vm" => filter_az_vm(&stdout, args),
            "webapp" => filter_az_webapp(&stdout, args),
            "storage" => filter_az_storage(&stdout, args),
            "aks" => filter_az_aks(&stdout, args),
            "acr" => filter_az_acr(&stdout, args),
            _ => filter_json_output(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for AWS CLI commands.
pub struct AwsFilter;

impl Filter for AwsFilter {
    fn name(&self) -> &'static str {
        "aws"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "aws" || cmd == "aws.cmd" || cmd == "aws.exe"
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
            "configure" => filter_aws_configure(&stdout, &stderr),
            "s3" => filter_aws_s3(&stdout, &stderr, args),
            "ec2" => filter_aws_ec2(&stdout, args),
            "lambda" => filter_aws_lambda(&stdout, args),
            "iam" => filter_aws_iam(&stdout, args),
            "ecs" => filter_aws_ecs(&stdout, args),
            "eks" => filter_aws_eks(&stdout, args),
            "rds" => filter_aws_rds(&stdout, args),
            "sts" => filter_aws_sts(&stdout),
            _ => filter_json_output(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter for Google Cloud CLI commands.
pub struct GcloudFilter;

impl Filter for GcloudFilter {
    fn name(&self) -> &'static str {
        "gcloud"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "gcloud" || cmd == "gcloud.cmd" || cmd == "gcloud.exe"
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
            "auth" => filter_gcloud_auth(&stdout, &stderr),
            "config" => filter_gcloud_config(&stdout, args),
            "projects" => filter_gcloud_projects(&stdout, args),
            "compute" => filter_gcloud_compute(&stdout, args),
            "container" => filter_gcloud_container(&stdout, args),
            "functions" => filter_gcloud_functions(&stdout, args),
            "run" => filter_gcloud_run(&stdout, args),
            "sql" => filter_gcloud_sql(&stdout, args),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

// Azure CLI filters
fn filter_az_login(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("You have logged in") || stdout.contains("\"tenantId\"") {
        let tenant_re = Regex::new(r#""tenantId":\s*"([^"]+)""#).unwrap();
        let user_re = Regex::new(r#""user":\s*\{[^}]*"name":\s*"([^"]+)""#).unwrap();

        let tenant = tenant_re.captures(&combined).map(|c| c[1].to_string()).unwrap_or_default();
        let user = user_re.captures(&combined).map(|c| c[1].to_string()).unwrap_or_default();

        let mut result = vec!["✓ logged in".to_string()];
        if !user.is_empty() {
            result.push(format!("  User: {}", user));
        }
        if !tenant.is_empty() {
            result.push(format!("  Tenant: {}", truncate(&tenant, 20)));
        }
        return result.join("\n");
    }

    filter_generic(stdout, stderr)
}

fn filter_az_account(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "list" => {
            let name_re = Regex::new(r#""name":\s*"([^"]+)""#).unwrap();
            let names: Vec<String> = name_re.captures_iter(stdout)
                .map(|c| c[1].to_string())
                .collect();

            if names.is_empty() {
                "No subscriptions found".to_string()
            } else {
                let mut result = vec![format!("{} subscriptions", names.len())];
                for name in names.iter().take(10) {
                    result.push(format!("  {}", truncate(name, 50)));
                }
                result.join("\n")
            }
        }
        "show" => {
            let name_re = Regex::new(r#""name":\s*"([^"]+)""#).unwrap();
            let state_re = Regex::new(r#""state":\s*"([^"]+)""#).unwrap();

            let name = name_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();
            let state = state_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();

            if !name.is_empty() {
                format!("{} ({})", name, state)
            } else {
                filter_json_output(stdout, "")
            }
        }
        "set" => "✓ subscription set".to_string(),
        _ => filter_json_output(stdout, ""),
    }
}

fn filter_az_group(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "list" => {
            let name_re = Regex::new(r#""name":\s*"([^"]+)""#).unwrap();
            let names: Vec<String> = name_re.captures_iter(stdout)
                .map(|c| c[1].to_string())
                .collect();

            if names.is_empty() {
                "No resource groups".to_string()
            } else {
                let mut result = vec![format!("{} resource groups", names.len())];
                for name in names.iter().take(15) {
                    result.push(format!("  {}", name));
                }
                if names.len() > 15 {
                    result.push(format!("  ... +{} more", names.len() - 15));
                }
                result.join("\n")
            }
        }
        "create" => {
            if stdout.contains("\"provisioningState\": \"Succeeded\"") {
                "✓ resource group created".to_string()
            } else {
                filter_json_output(stdout, "")
            }
        }
        "delete" => "✓ resource group deleted".to_string(),
        _ => filter_json_output(stdout, ""),
    }
}

fn filter_az_vm(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "list" => {
            let name_re = Regex::new(r#""name":\s*"([^"]+)""#).unwrap();
            let names: Vec<String> = name_re.captures_iter(stdout)
                .map(|c| c[1].to_string())
                .collect();

            if names.is_empty() {
                "No VMs found".to_string()
            } else {
                let mut result = vec![format!("{} VMs", names.len())];
                for name in names.iter().take(10) {
                    result.push(format!("  {}", name));
                }
                result.join("\n")
            }
        }
        "create" => {
            if stdout.contains("\"provisioningState\"") {
                "✓ VM created".to_string()
            } else {
                filter_json_output(stdout, "")
            }
        }
        "start" => "✓ VM started".to_string(),
        "stop" => "✓ VM stopped".to_string(),
        "delete" => "✓ VM deleted".to_string(),
        _ => filter_json_output(stdout, ""),
    }
}

fn filter_az_webapp(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "list" => {
            let name_re = Regex::new(r#""name":\s*"([^"]+)""#).unwrap();
            let names: Vec<String> = name_re.captures_iter(stdout)
                .map(|c| c[1].to_string())
                .collect();

            if names.is_empty() {
                "No web apps found".to_string()
            } else {
                let mut result = vec![format!("{} web apps", names.len())];
                for name in names.iter().take(10) {
                    result.push(format!("  {}", name));
                }
                result.join("\n")
            }
        }
        "create" => "✓ web app created".to_string(),
        "deploy" => "✓ deployed".to_string(),
        _ => filter_json_output(stdout, ""),
    }
}

fn filter_az_storage(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "account" => {
            let name_re = Regex::new(r#""name":\s*"([^"]+)""#).unwrap();
            let names: Vec<String> = name_re.captures_iter(stdout)
                .map(|c| c[1].to_string())
                .collect();

            if names.is_empty() {
                "No storage accounts".to_string()
            } else {
                format!("{} storage accounts", names.len())
            }
        }
        "blob" | "container" => filter_json_output(stdout, ""),
        _ => filter_json_output(stdout, ""),
    }
}

fn filter_az_aks(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "list" => {
            let name_re = Regex::new(r#""name":\s*"([^"]+)""#).unwrap();
            let names: Vec<String> = name_re.captures_iter(stdout)
                .map(|c| c[1].to_string())
                .collect();

            if names.is_empty() {
                "No AKS clusters".to_string()
            } else {
                let mut result = vec![format!("{} AKS clusters", names.len())];
                for name in names.iter().take(10) {
                    result.push(format!("  {}", name));
                }
                result.join("\n")
            }
        }
        "create" => "✓ AKS cluster created".to_string(),
        "get-credentials" => "✓ credentials configured".to_string(),
        _ => filter_json_output(stdout, ""),
    }
}

fn filter_az_acr(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "list" => {
            let name_re = Regex::new(r#""name":\s*"([^"]+)""#).unwrap();
            let names: Vec<String> = name_re.captures_iter(stdout)
                .map(|c| c[1].to_string())
                .collect();

            format!("{} container registries", names.len())
        }
        "login" => "✓ logged into ACR".to_string(),
        _ => filter_json_output(stdout, ""),
    }
}

// AWS CLI filters
fn filter_aws_configure(_stdout: &str, stderr: &str) -> String {
    if stderr.is_empty() {
        "✓ configured".to_string()
    } else {
        filter_generic("", stderr)
    }
}

fn filter_aws_s3(stdout: &str, stderr: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "ls" => {
            let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
            if lines.is_empty() {
                "No buckets/objects".to_string()
            } else {
                let mut result = vec![format!("{} items", lines.len())];
                for line in lines.iter().take(15) {
                    result.push(format!("  {}", truncate(line, 60)));
                }
                if lines.len() > 15 {
                    result.push(format!("  ... +{} more", lines.len() - 15));
                }
                result.join("\n")
            }
        }
        "cp" | "mv" | "sync" => {
            if stderr.is_empty() || stdout.contains("copy:") || stdout.contains("upload:") {
                "✓ completed".to_string()
            } else {
                filter_generic(stdout, stderr)
            }
        }
        "rm" => "✓ deleted".to_string(),
        "mb" => "✓ bucket created".to_string(),
        "rb" => "✓ bucket deleted".to_string(),
        _ => filter_generic(stdout, stderr),
    }
}

fn filter_aws_ec2(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "describe-instances" => {
            let id_re = Regex::new(r#""InstanceId":\s*"([^"]+)""#).unwrap();
            let ids: Vec<String> = id_re.captures_iter(stdout)
                .map(|c| c[1].to_string())
                .collect();

            if ids.is_empty() {
                "No instances".to_string()
            } else {
                let mut result = vec![format!("{} instances", ids.len())];
                for id in ids.iter().take(10) {
                    result.push(format!("  {}", id));
                }
                result.join("\n")
            }
        }
        "run-instances" => "✓ instance launched".to_string(),
        "start-instances" => "✓ instance started".to_string(),
        "stop-instances" => "✓ instance stopped".to_string(),
        "terminate-instances" => "✓ instance terminated".to_string(),
        _ => filter_json_output(stdout, ""),
    }
}

fn filter_aws_lambda(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "list-functions" => {
            let name_re = Regex::new(r#""FunctionName":\s*"([^"]+)""#).unwrap();
            let names: Vec<String> = name_re.captures_iter(stdout)
                .map(|c| c[1].to_string())
                .collect();

            if names.is_empty() {
                "No functions".to_string()
            } else {
                let mut result = vec![format!("{} functions", names.len())];
                for name in names.iter().take(10) {
                    result.push(format!("  {}", name));
                }
                result.join("\n")
            }
        }
        "invoke" => "✓ invoked".to_string(),
        "create-function" => "✓ function created".to_string(),
        "update-function-code" => "✓ function updated".to_string(),
        _ => filter_json_output(stdout, ""),
    }
}

fn filter_aws_iam(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "list-users" => {
            let name_re = Regex::new(r#""UserName":\s*"([^"]+)""#).unwrap();
            let names: Vec<String> = name_re.captures_iter(stdout)
                .map(|c| c[1].to_string())
                .collect();

            format!("{} users", names.len())
        }
        "list-roles" => {
            let name_re = Regex::new(r#""RoleName":\s*"([^"]+)""#).unwrap();
            let names: Vec<String> = name_re.captures_iter(stdout)
                .map(|c| c[1].to_string())
                .collect();

            format!("{} roles", names.len())
        }
        _ => filter_json_output(stdout, ""),
    }
}

fn filter_aws_ecs(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "list-clusters" => {
            let arn_re = Regex::new(r#""arn:aws:ecs:[^"]+/([^"]+)""#).unwrap();
            let names: Vec<String> = arn_re.captures_iter(stdout)
                .map(|c| c[1].to_string())
                .collect();

            format!("{} clusters", names.len())
        }
        "list-services" => {
            let count = stdout.matches("arn:aws:ecs").count();
            format!("{} services", count)
        }
        _ => filter_json_output(stdout, ""),
    }
}

fn filter_aws_eks(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "list-clusters" => {
            let name_re = Regex::new(r#""([^"]+)""#).unwrap();
            let names: Vec<String> = name_re.captures_iter(stdout)
                .map(|c| c[1].to_string())
                .filter(|s| !s.contains("clusters"))
                .collect();

            format!("{} EKS clusters", names.len())
        }
        "update-kubeconfig" => "✓ kubeconfig updated".to_string(),
        _ => filter_json_output(stdout, ""),
    }
}

fn filter_aws_rds(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "describe-db-instances" => {
            let id_re = Regex::new(r#""DBInstanceIdentifier":\s*"([^"]+)""#).unwrap();
            let ids: Vec<String> = id_re.captures_iter(stdout)
                .map(|c| c[1].to_string())
                .collect();

            format!("{} DB instances", ids.len())
        }
        _ => filter_json_output(stdout, ""),
    }
}

fn filter_aws_sts(stdout: &str) -> String {
    let account_re = Regex::new(r#""Account":\s*"([^"]+)""#).unwrap();
    let arn_re = Regex::new(r#""Arn":\s*"([^"]+)""#).unwrap();

    let account = account_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();
    let arn = arn_re.captures(stdout).map(|c| c[1].to_string()).unwrap_or_default();

    if !account.is_empty() {
        format!("Account: {} ({})", account, truncate(&arn, 40))
    } else {
        filter_json_output(stdout, "")
    }
}

// Google Cloud filters
fn filter_gcloud_auth(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("You are now logged in") || combined.contains("Credentials saved") {
        return "✓ authenticated".to_string();
    }

    if combined.contains("You are now logged in as") {
        let user_re = Regex::new(r"logged in as \[([^\]]+)\]").unwrap();
        if let Some(caps) = user_re.captures(&combined) {
            return format!("✓ logged in as {}", &caps[1]);
        }
    }

    filter_generic(stdout, stderr)
}

fn filter_gcloud_config(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "list" => {
            let lines: Vec<&str> = stdout.lines()
                .filter(|l| !l.trim().is_empty() && !l.starts_with('['))
                .collect();

            let mut result = vec![format!("{} config values", lines.len())];
            for line in lines.iter().take(10) {
                result.push(format!("  {}", line.trim()));
            }
            result.join("\n")
        }
        "set" => "✓ config set".to_string(),
        _ => filter_generic(stdout, ""),
    }
}

fn filter_gcloud_projects(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "list" => {
            let lines: Vec<&str> = stdout.lines()
                .filter(|l| !l.trim().is_empty() && !l.contains("PROJECT_ID"))
                .collect();

            if lines.is_empty() {
                "No projects".to_string()
            } else {
                let mut result = vec![format!("{} projects", lines.len())];
                for line in lines.iter().take(10) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if !parts.is_empty() {
                        result.push(format!("  {}", parts[0]));
                    }
                }
                result.join("\n")
            }
        }
        _ => filter_generic(stdout, ""),
    }
}

fn filter_gcloud_compute(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "instances" => {
            let lines: Vec<&str> = stdout.lines()
                .filter(|l| !l.trim().is_empty() && !l.contains("NAME"))
                .collect();

            if lines.is_empty() {
                "No instances".to_string()
            } else {
                let mut result = vec![format!("{} instances", lines.len())];
                for line in lines.iter().take(10) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        result.push(format!("  {} ({})", parts[0], parts.last().unwrap_or(&"")));
                    }
                }
                result.join("\n")
            }
        }
        _ => filter_generic(stdout, ""),
    }
}

fn filter_gcloud_container(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "clusters" => {
            let lines: Vec<&str> = stdout.lines()
                .filter(|l| !l.trim().is_empty() && !l.contains("NAME"))
                .collect();

            if lines.is_empty() {
                "No GKE clusters".to_string()
            } else {
                let mut result = vec![format!("{} GKE clusters", lines.len())];
                for line in lines.iter().take(10) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if !parts.is_empty() {
                        result.push(format!("  {}", parts[0]));
                    }
                }
                result.join("\n")
            }
        }
        _ => filter_generic(stdout, ""),
    }
}

fn filter_gcloud_functions(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "list" => {
            let lines: Vec<&str> = stdout.lines()
                .filter(|l| !l.trim().is_empty() && !l.contains("NAME"))
                .collect();

            format!("{} functions", lines.len())
        }
        "deploy" => "✓ function deployed".to_string(),
        _ => filter_generic(stdout, ""),
    }
}

fn filter_gcloud_run(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "services" => {
            let lines: Vec<&str> = stdout.lines()
                .filter(|l| !l.trim().is_empty() && !l.contains("SERVICE"))
                .collect();

            format!("{} Cloud Run services", lines.len())
        }
        "deploy" => {
            if stdout.contains("Service URL:") {
                let url_re = Regex::new(r"Service URL:\s*(https://[^\s]+)").unwrap();
                if let Some(caps) = url_re.captures(stdout) {
                    return format!("✓ deployed: {}", &caps[1]);
                }
            }
            "✓ deployed".to_string()
        }
        _ => filter_generic(stdout, ""),
    }
}

fn filter_gcloud_sql(stdout: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "instances" => {
            let lines: Vec<&str> = stdout.lines()
                .filter(|l| !l.trim().is_empty() && !l.contains("NAME"))
                .collect();

            format!("{} SQL instances", lines.len())
        }
        _ => filter_generic(stdout, ""),
    }
}

fn filter_json_output(stdout: &str, stderr: &str) -> String {
    // For JSON output, extract key counts
    let object_count = stdout.matches('{').count();
    let array_items = stdout.matches('[').count();

    if object_count > 5 {
        // Complex JSON, summarize
        let key_re = Regex::new(r#""([^"]+)":\s*"#).unwrap();
        let keys: std::collections::HashSet<String> = key_re.captures_iter(stdout)
            .map(|c| c[1].to_string())
            .take(20)
            .collect();

        if !keys.is_empty() {
            let mut result = vec![format!("{} objects", object_count)];
            result.push(format!("  Keys: {}", keys.iter().take(5).cloned().collect::<Vec<_>>().join(", ")));
            return result.join("\n");
        }
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
