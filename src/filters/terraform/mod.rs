//! Terraform CLI filter.

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for Terraform commands.
pub struct TerraformFilter;

impl Filter for TerraformFilter {
    fn name(&self) -> &'static str {
        "terraform"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "terraform" || cmd == "terraform.exe" || cmd == "tf"
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
            "init" => filter_init(&stdout, &stderr),
            "plan" => filter_plan(&stdout, &stderr),
            "apply" => filter_apply(&stdout, &stderr),
            "destroy" => filter_destroy(&stdout, &stderr),
            "validate" => filter_validate(&stdout, &stderr),
            "fmt" => filter_fmt(&stdout, &stderr),
            "state" => filter_state(&stdout, &stderr, args),
            "output" => filter_output(&stdout),
            "show" => filter_show(&stdout),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_init(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let provider_re = Regex::new(r"- Installed (.+) v([\d.]+)").unwrap();
    let success_re = Regex::new(r"Terraform has been successfully initialized").unwrap();

    let mut providers = Vec::new();
    for caps in provider_re.captures_iter(&combined) {
        providers.push(format!("{} v{}", &caps[1], &caps[2]));
    }

    if success_re.is_match(&combined) {
        let mut result = vec!["✓ initialized".to_string()];
        if !providers.is_empty() {
            result.push(format!("  {} providers installed", providers.len()));
            for p in providers.iter().take(5) {
                result.push(format!("    {}", p));
            }
        }
        result.join("\n")
    } else if combined.contains("Error") {
        filter_generic(stdout, stderr)
    } else {
        "✓ initialized".to_string()
    }
}

fn filter_plan(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let plan_re = Regex::new(r"Plan: (\d+) to add, (\d+) to change, (\d+) to destroy").unwrap();
    let no_changes_re = Regex::new(r"No changes").unwrap();
    let resource_re = Regex::new(r"#\s*(\S+)\s+will be (created|destroyed|updated|replaced)").unwrap();

    let mut resources: Vec<(String, String)> = Vec::new();
    for caps in resource_re.captures_iter(&combined) {
        resources.push((caps[1].to_string(), caps[2].to_string()));
    }

    if let Some(caps) = plan_re.captures(&combined) {
        let add: u32 = caps[1].parse().unwrap_or(0);
        let change: u32 = caps[2].parse().unwrap_or(0);
        let destroy: u32 = caps[3].parse().unwrap_or(0);

        let mut result = vec![format!("Plan: +{} ~{} -{}", add, change, destroy)];

        // Group by action
        let creates: Vec<_> = resources.iter().filter(|(_, a)| a == "created").collect();
        let updates: Vec<_> = resources.iter().filter(|(_, a)| a == "updated" || a == "replaced").collect();
        let destroys: Vec<_> = resources.iter().filter(|(_, a)| a == "destroyed").collect();

        for (r, _) in creates.iter().take(5) {
            result.push(format!("  + {}", truncate(r, 50)));
        }
        if creates.len() > 5 {
            result.push(format!("  ... +{} more to create", creates.len() - 5));
        }

        for (r, _) in updates.iter().take(3) {
            result.push(format!("  ~ {}", truncate(r, 50)));
        }
        if updates.len() > 3 {
            result.push(format!("  ... +{} more to change", updates.len() - 3));
        }

        for (r, _) in destroys.iter().take(3) {
            result.push(format!("  - {}", truncate(r, 50)));
        }
        if destroys.len() > 3 {
            result.push(format!("  ... +{} more to destroy", destroys.len() - 3));
        }

        result.join("\n")
    } else if no_changes_re.is_match(&combined) {
        "✓ No changes. Infrastructure is up-to-date.".to_string()
    } else if combined.contains("Error") {
        let error_re = Regex::new(r"Error: (.+)").unwrap();
        let errors: Vec<String> = error_re.captures_iter(&combined)
            .map(|c| truncate(&c[1], 60))
            .collect();

        let mut result = vec![format!("✗ {} errors", errors.len())];
        for e in errors.iter().take(5) {
            result.push(format!("  {}", e));
        }
        result.join("\n")
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_apply(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let complete_re = Regex::new(r"Apply complete! Resources: (\d+) added, (\d+) changed, (\d+) destroyed").unwrap();
    let creating_re = Regex::new(r"(\S+): Creating\.\.\.").unwrap();
    let created_re = Regex::new(r"(\S+): Creation complete").unwrap();

    if let Some(caps) = complete_re.captures(&combined) {
        let add: u32 = caps[1].parse().unwrap_or(0);
        let change: u32 = caps[2].parse().unwrap_or(0);
        let destroy: u32 = caps[3].parse().unwrap_or(0);
        format!("✓ Apply complete! +{} ~{} -{}", add, change, destroy)
    } else if combined.contains("Error") {
        let error_re = Regex::new(r"Error: (.+)").unwrap();
        if let Some(caps) = error_re.captures(&combined) {
            format!("✗ {}", truncate(&caps[1], 70))
        } else {
            "✗ Apply failed".to_string()
        }
    } else {
        // In progress
        let creating: Vec<String> = creating_re.captures_iter(&combined)
            .map(|c| c[1].to_string())
            .collect();

        if !creating.is_empty() {
            format!("⏳ Creating {} resources...", creating.len())
        } else {
            filter_generic(stdout, stderr)
        }
    }
}

fn filter_destroy(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    let complete_re = Regex::new(r"Destroy complete! Resources: (\d+) destroyed").unwrap();

    if let Some(caps) = complete_re.captures(&combined) {
        format!("✓ Destroyed {} resources", &caps[1])
    } else if combined.contains("Error") {
        filter_generic(stdout, stderr)
    } else {
        "✓ Destroy complete".to_string()
    }
}

fn filter_validate(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.contains("Success!") || combined.contains("valid") {
        "✓ Configuration is valid".to_string()
    } else if combined.contains("Error") {
        let error_re = Regex::new(r"Error: (.+)").unwrap();
        let errors: Vec<String> = error_re.captures_iter(&combined)
            .map(|c| truncate(&c[1], 60))
            .collect();

        let mut result = vec![format!("✗ {} validation errors", errors.len())];
        for e in errors.iter().take(5) {
            result.push(format!("  {}", e));
        }
        result.join("\n")
    } else {
        filter_generic(stdout, stderr)
    }
}

fn filter_fmt(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    if combined.is_empty() || combined.trim().is_empty() {
        "✓ Formatted".to_string()
    } else {
        // Lists files that were formatted
        let files: Vec<&str> = combined.lines().filter(|l| !l.is_empty()).collect();
        format!("✓ Formatted {} files", files.len())
    }
}

fn filter_state(stdout: &str, stderr: &str, args: &[String]) -> String {
    let subsubcmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match subsubcmd {
        "list" => {
            let resources: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
            let mut result = vec![format!("{} resources in state", resources.len())];
            for r in resources.iter().take(15) {
                result.push(format!("  {}", r));
            }
            if resources.len() > 15 {
                result.push(format!("  ... +{} more", resources.len() - 15));
            }
            result.join("\n")
        }
        "show" => {
            if stdout.len() > 500 {
                format!("{}... ({} chars)", &stdout[..500], stdout.len())
            } else {
                stdout.to_string()
            }
        }
        _ => filter_generic(stdout, stderr),
    }
}

fn filter_output(stdout: &str) -> String {
    let output_re = Regex::new(r#"(\w+)\s*=\s*"?([^"\n]+)"?"#).unwrap();

    let mut outputs = Vec::new();
    for caps in output_re.captures_iter(stdout) {
        outputs.push(format!("{} = {}", &caps[1], truncate(&caps[2], 40)));
    }

    if !outputs.is_empty() {
        let mut result = vec![format!("{} outputs", outputs.len())];
        for o in outputs.iter().take(10) {
            result.push(format!("  {}", o));
        }
        if outputs.len() > 10 {
            result.push(format!("  ... +{} more", outputs.len() - 10));
        }
        result.join("\n")
    } else {
        stdout.to_string()
    }
}

fn filter_show(stdout: &str) -> String {
    let resource_re = Regex::new(r"#\s*(\S+):").unwrap();

    let resources: Vec<String> = resource_re.captures_iter(stdout)
        .map(|c| c[1].to_string())
        .collect();

    if !resources.is_empty() {
        let mut result = vec![format!("{} resources in state", resources.len())];
        for r in resources.iter().take(10) {
            result.push(format!("  {}", r));
        }
        if resources.len() > 10 {
            result.push(format!("  ... +{} more", resources.len() - 10));
        }
        result.join("\n")
    } else if stdout.len() > 500 {
        format!("{}... ({} chars)", &stdout[..500], stdout.len())
    } else {
        stdout.to_string()
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
