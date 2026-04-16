//! PHP ecosystem filters - composer, artisan, phpunit, pest

use std::process::Command;
use regex::Regex;
use anyhow::Result;
use super::traits::{Filter, FilterResult};

// ============================================================================
// Composer Filter
// ============================================================================

pub struct ComposerFilter;

impl Filter for ComposerFilter {
    fn name(&self) -> &'static str {
        "composer"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "composer" || cmd == "composer.phar"
    }

    fn execute(&self, _command: &str, args: &[String]) -> Result<FilterResult> {
        let start = std::time::Instant::now();

        let output = Command::new("composer")
            .args(args)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);
        let input_chars = combined.len();

        let subcommand = args.first().map(|s| s.as_str()).unwrap_or("");

        let filtered = match subcommand {
            "install" | "update" | "require" | "remove" => filter_composer_install(&stdout, &stderr),
            "outdated" => filter_composer_outdated(&stdout),
            "show" | "info" => filter_composer_show(&stdout),
            "validate" => filter_composer_validate(&stdout, &stderr),
            "dump-autoload" | "dumpautoload" => filter_composer_dump(&stdout),
            "run" | "run-script" => filter_composer_run(&stdout, &stderr),
            "diagnose" => filter_composer_diagnose(&stdout),
            _ => combined.to_string(),
        };

        Ok(FilterResult::new(filtered, input_chars, start.elapsed().as_millis() as u64))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_composer_install(stdout: &str, stderr: &str) -> String {
    let mut lines = Vec::new();
    let mut installed = Vec::new();
    let mut updated = Vec::new();
    let mut removed = Vec::new();

    for line in stdout.lines().chain(stderr.lines()) {
        let trimmed = line.trim();

        // Skip progress bars and verbose output
        if trimmed.is_empty()
            || trimmed.starts_with("Loading composer")
            || trimmed.starts_with("Updating dependencies")
            || trimmed.starts_with("Running composer")
            || trimmed.starts_with("Writing lock file")
            || trimmed.starts_with("Generating autoload")
            || trimmed.contains("Reading ")
            || trimmed.contains("Downloading ")
            || trimmed.contains('%')
        {
            continue;
        }

        if trimmed.starts_with("- Installing ") {
            if let Some(pkg) = trimmed.strip_prefix("- Installing ") {
                installed.push(pkg.split_whitespace().next().unwrap_or(pkg));
            }
        } else if trimmed.starts_with("- Updating ") {
            if let Some(pkg) = trimmed.strip_prefix("- Updating ") {
                updated.push(pkg.split_whitespace().next().unwrap_or(pkg));
            }
        } else if trimmed.starts_with("- Removing ") {
            if let Some(pkg) = trimmed.strip_prefix("- Removing ") {
                removed.push(pkg.split_whitespace().next().unwrap_or(pkg));
            }
        } else if trimmed.contains("error") || trimmed.contains("Error") || trimmed.contains("warning") {
            lines.push(trimmed.to_string());
        }
    }

    let mut result = Vec::new();

    if !installed.is_empty() {
        result.push(format!("Installed({}): {}", installed.len(), installed.join(", ")));
    }
    if !updated.is_empty() {
        result.push(format!("Updated({}): {}", updated.len(), updated.join(", ")));
    }
    if !removed.is_empty() {
        result.push(format!("Removed({}): {}", removed.len(), removed.join(", ")));
    }

    result.extend(lines);

    if result.is_empty() {
        "OK".to_string()
    } else {
        result.join("\n")
    }
}

fn filter_composer_outdated(stdout: &str) -> String {
    let mut outdated = Vec::new();
    let semver_re = Regex::new(r"(\S+)\s+(\S+)\s+(\S+)\s").unwrap();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Color legend") || trimmed.starts_with("!") {
            continue;
        }

        if let Some(caps) = semver_re.captures(trimmed) {
            let pkg = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let current = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let latest = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            if !pkg.is_empty() && current != latest {
                outdated.push(format!("{}: {} -> {}", pkg, current, latest));
            }
        }
    }

    if outdated.is_empty() {
        "All packages up to date".to_string()
    } else {
        format!("Outdated({}):\n{}", outdated.len(), outdated.join("\n"))
    }
}

fn filter_composer_show(stdout: &str) -> String {
    let mut packages = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            packages.push(format!("{} {}", parts[0], parts[1]));
        }
    }

    if packages.is_empty() {
        "No packages".to_string()
    } else {
        format!("Packages({}):\n{}", packages.len(), packages.join("\n"))
    }
}

fn filter_composer_validate(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}{}", stdout, stderr);

    if combined.contains("is valid") {
        "Valid".to_string()
    } else {
        let errors: Vec<&str> = combined.lines()
            .filter(|l| l.contains("error") || l.contains("warning") || l.contains("Error"))
            .collect();

        if errors.is_empty() {
            combined.trim().to_string()
        } else {
            errors.join("\n")
        }
    }
}

fn filter_composer_dump(stdout: &str) -> String {
    if stdout.contains("Generated") || stdout.contains("autoload") {
        "Autoload generated".to_string()
    } else {
        stdout.trim().to_string()
    }
}

fn filter_composer_run(stdout: &str, stderr: &str) -> String {
    // For script runs, keep output but remove composer boilerplate
    let combined = format!("{}{}", stdout, stderr);
    let lines: Vec<&str> = combined.lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with("> ") && !t.is_empty()
        })
        .collect();

    if lines.is_empty() {
        "OK".to_string()
    } else {
        lines.join("\n")
    }
}

fn filter_composer_diagnose(stdout: &str) -> String {
    let mut issues = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.contains("FAIL") || trimmed.contains("WARNING") || trimmed.contains("ERROR") {
            issues.push(trimmed.to_string());
        }
    }

    if issues.is_empty() {
        "All checks passed".to_string()
    } else {
        format!("Issues({}):\n{}", issues.len(), issues.join("\n"))
    }
}

// ============================================================================
// Laravel Artisan Filter
// ============================================================================

pub struct ArtisanFilter;

impl Filter for ArtisanFilter {
    fn name(&self) -> &'static str {
        "artisan"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "php" || cmd == "artisan"
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        let start = std::time::Instant::now();

        // Check if this is an artisan command
        let is_artisan = command.to_lowercase() == "artisan"
            || args.iter().any(|a| a.contains("artisan"));

        if !is_artisan {
            // Not an artisan command, pass through
            let output = Command::new(command)
                .args(args)
                .output()?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{}{}", stdout, stderr);

            return Ok(FilterResult::new(combined.to_string(), combined.len(), start.elapsed().as_millis() as u64));
        }

        let output = Command::new(command)
            .args(args)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);
        let input_chars = combined.len();

        // Find the artisan subcommand
        let subcommand = args.iter()
            .skip_while(|a| a.contains("artisan") || a.starts_with('-'))
            .next()
            .map(|s| s.as_str())
            .unwrap_or("");

        let filtered = match subcommand {
            "migrate" => filter_artisan_migrate(&stdout, &stderr),
            "migrate:status" => filter_artisan_migrate_status(&stdout),
            "make:model" | "make:controller" | "make:migration" | "make:seeder"
            | "make:factory" | "make:middleware" | "make:request" | "make:resource"
            | "make:command" | "make:event" | "make:job" | "make:listener"
            | "make:mail" | "make:notification" | "make:policy" | "make:provider"
            | "make:rule" | "make:test" => filter_artisan_make(&stdout),
            "route:list" => filter_artisan_routes(&stdout),
            "config:cache" | "config:clear" | "cache:clear" | "view:clear"
            | "route:cache" | "route:clear" | "optimize" | "optimize:clear" => {
                filter_artisan_cache(&stdout)
            }
            "serve" => filter_artisan_serve(&stdout, &stderr),
            "test" => filter_artisan_test(&stdout, &stderr),
            "tinker" => combined.to_string(), // Interactive, pass through
            "list" => filter_artisan_list(&stdout),
            "queue:work" | "queue:listen" => filter_artisan_queue(&stdout, &stderr),
            "schedule:run" => filter_artisan_schedule(&stdout),
            "db:seed" => filter_artisan_seed(&stdout),
            _ => combined.to_string(),
        };

        Ok(FilterResult::new(filtered, input_chars, start.elapsed().as_millis() as u64))
    }

    fn priority(&self) -> u8 {
        90 // Higher priority to catch php artisan commands
    }
}

fn filter_artisan_migrate(stdout: &str, stderr: &str) -> String {
    let mut migrations = Vec::new();
    let mut errors = Vec::new();

    for line in stdout.lines().chain(stderr.lines()) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.contains("Migrating:") || trimmed.contains("Migrated:") {
            if let Some(name) = trimmed.split(':').nth(1) {
                migrations.push(name.trim().to_string());
            }
        } else if trimmed.contains("error") || trimmed.contains("Error") || trimmed.contains("SQLSTATE") {
            errors.push(trimmed.to_string());
        }
    }

    let mut result = Vec::new();

    if !migrations.is_empty() {
        // Deduplicate (Migrating + Migrated same migration)
        let unique: Vec<_> = migrations.iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        result.push(format!("Migrated({}): {}", unique.len(), unique.into_iter().cloned().collect::<Vec<_>>().join(", ")));
    }

    if !errors.is_empty() {
        result.extend(errors);
    }

    if result.is_empty() {
        "Nothing to migrate".to_string()
    } else {
        result.join("\n")
    }
}

fn filter_artisan_migrate_status(stdout: &str) -> String {
    let mut pending = Vec::new();
    let mut ran = 0;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.contains("Pending") || trimmed.contains("No") {
            if let Some(name) = trimmed.split_whitespace().last() {
                pending.push(name.to_string());
            }
        } else if trimmed.contains("Ran") || trimmed.contains("Yes") {
            ran += 1;
        }
    }

    let mut result = format!("Ran: {}", ran);
    if !pending.is_empty() {
        result.push_str(&format!(", Pending({}): {}", pending.len(), pending.join(", ")));
    }
    result
}

fn filter_artisan_make(stdout: &str) -> String {
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.contains("created successfully") || trimmed.contains("Created") {
            return trimmed.to_string();
        }
    }
    stdout.trim().to_string()
}

fn filter_artisan_routes(stdout: &str) -> String {
    let mut routes = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('+') || trimmed.starts_with('|') && trimmed.contains("Method") {
            continue;
        }

        // Extract method and URI from table format
        if trimmed.starts_with('|') {
            let parts: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 4 {
                let method = parts.get(1).unwrap_or(&"");
                let uri = parts.get(2).unwrap_or(&"");
                if !method.is_empty() && !uri.is_empty() && *method != "Method" {
                    routes.push(format!("{} {}", method, uri));
                }
            }
        }
    }

    if routes.is_empty() {
        stdout.trim().to_string()
    } else {
        format!("Routes({}):\n{}", routes.len(), routes.join("\n"))
    }
}

fn filter_artisan_cache(stdout: &str) -> String {
    let actions: Vec<&str> = stdout.lines()
        .filter(|l| {
            let t = l.trim().to_lowercase();
            t.contains("cleared") || t.contains("cached") || t.contains("compiled") || t.contains("removed")
        })
        .map(|l| l.trim())
        .collect();

    if actions.is_empty() {
        "OK".to_string()
    } else {
        actions.join(", ")
    }
}

fn filter_artisan_serve(stdout: &str, stderr: &str) -> String {
    // Extract just the server URL
    let combined = format!("{}{}", stdout, stderr);
    for line in combined.lines() {
        if line.contains("http://") || line.contains("https://") {
            return line.trim().to_string();
        }
    }
    combined.trim().to_string()
}

fn filter_artisan_test(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}{}", stdout, stderr);
    let mut result = Vec::new();
    let mut in_failure = false;

    for line in combined.lines() {
        let trimmed = line.trim();

        // Capture test summary
        if trimmed.contains("Tests:") || trimmed.contains("PASS") || trimmed.contains("FAIL") {
            result.push(trimmed.to_string());
        }
        // Capture failures
        else if trimmed.starts_with("FAILED") || trimmed.contains("FAILURES!") {
            in_failure = true;
            result.push(trimmed.to_string());
        }
        else if in_failure && !trimmed.is_empty() && !trimmed.starts_with("Time:") {
            result.push(trimmed.to_string());
        }
    }

    if result.is_empty() {
        "Tests passed".to_string()
    } else {
        result.join("\n")
    }
}

fn filter_artisan_list(stdout: &str) -> String {
    let commands: Vec<&str> = stdout.lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with("Laravel") && !t.starts_with("Usage:")
            && !t.starts_with("Options:") && !t.starts_with("Available")
            && !t.starts_with('-')
        })
        .take(30) // Limit output
        .collect();

    format!("Commands({}):\n{}", commands.len(), commands.join("\n"))
}

fn filter_artisan_queue(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}{}", stdout, stderr);
    let mut processed = 0;
    let mut failed = 0;

    for line in combined.lines() {
        if line.contains("Processed:") || line.contains("Processing:") {
            processed += 1;
        }
        if line.contains("Failed:") || line.contains("failed") {
            failed += 1;
        }
    }

    format!("Queue: processed={}, failed={}", processed, failed)
}

fn filter_artisan_schedule(stdout: &str) -> String {
    let ran: Vec<&str> = stdout.lines()
        .filter(|l| l.contains("Running") || l.contains("scheduled"))
        .collect();

    if ran.is_empty() {
        "No scheduled tasks ran".to_string()
    } else {
        format!("Ran({}):\n{}", ran.len(), ran.join("\n"))
    }
}

fn filter_artisan_seed(stdout: &str) -> String {
    let seeded: Vec<&str> = stdout.lines()
        .filter(|l| l.contains("Seeding:") || l.contains("Seeded:") || l.contains("Database seeding"))
        .collect();

    if seeded.is_empty() {
        "Seeding complete".to_string()
    } else {
        seeded.join("\n")
    }
}

// ============================================================================
// PHPUnit Filter
// ============================================================================

pub struct PhpunitFilter;

impl Filter for PhpunitFilter {
    fn name(&self) -> &'static str {
        "phpunit"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "phpunit" || cmd.contains("phpunit")
    }

    fn execute(&self, _command: &str, args: &[String]) -> Result<FilterResult> {
        let start = std::time::Instant::now();

        let output = Command::new("phpunit")
            .args(args)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);
        let input_chars = combined.len();

        let filtered = filter_phpunit_output(&stdout, &stderr);

        Ok(FilterResult::new(filtered, input_chars, start.elapsed().as_millis() as u64))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_phpunit_output(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}{}", stdout, stderr);
    let mut result = Vec::new();
    let mut failures = Vec::new();
    let mut in_failure = false;

    for line in combined.lines() {
        let trimmed = line.trim();

        // Skip progress dots and empty lines
        if trimmed.is_empty() || trimmed.chars().all(|c| c == '.' || c == 'F' || c == 'E' || c == 'S' || c == 'I') {
            continue;
        }

        // Capture summary line
        if trimmed.starts_with("OK (") || trimmed.starts_with("FAILURES!")
            || trimmed.starts_with("Tests:") || trimmed.starts_with("Time:")
            || trimmed.contains("tests,") || trimmed.contains("assertions")
        {
            result.push(trimmed.to_string());
        }
        // Capture failure details
        else if trimmed.starts_with("1)") || trimmed.starts_with("2)") || trimmed.starts_with("3)")
            || trimmed.starts_with("FAILED") || trimmed.contains("Failed asserting")
        {
            in_failure = true;
            failures.push(trimmed.to_string());
        }
        else if in_failure && (trimmed.starts_with("---") || trimmed.starts_with("+++") || trimmed.starts_with("@@")) {
            // Skip diff markers
            continue;
        }
        else if in_failure && !trimmed.is_empty() {
            failures.push(trimmed.to_string());
        }
    }

    if !failures.is_empty() {
        result.push("--- Failures ---".to_string());
        result.extend(failures.into_iter().take(20)); // Limit failure output
    }

    if result.is_empty() {
        "Tests passed".to_string()
    } else {
        result.join("\n")
    }
}

// ============================================================================
// Pest Filter (Modern PHP Testing)
// ============================================================================

pub struct PestFilter;

impl Filter for PestFilter {
    fn name(&self) -> &'static str {
        "pest"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.to_lowercase();
        cmd == "pest" || cmd.contains("pest")
    }

    fn execute(&self, _command: &str, args: &[String]) -> Result<FilterResult> {
        let start = std::time::Instant::now();

        let output = Command::new("pest")
            .args(args)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);
        let input_chars = combined.len();

        let filtered = filter_pest_output(&stdout, &stderr);

        Ok(FilterResult::new(filtered, input_chars, start.elapsed().as_millis() as u64))
    }

    fn priority(&self) -> u8 {
        85
    }
}

fn filter_pest_output(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}{}", stdout, stderr);
    let mut result = Vec::new();
    let mut failures = Vec::new();

    for line in combined.lines() {
        let trimmed = line.trim();

        // Skip decorative lines
        if trimmed.is_empty() || trimmed.starts_with("⟶") || trimmed.chars().all(|c| c == '─' || c == ' ') {
            continue;
        }

        // Capture summary
        if trimmed.contains("Tests:") || trimmed.contains("Passed:") || trimmed.contains("Failed:")
            || trimmed.contains("PASS") || trimmed.contains("FAIL") || trimmed.contains("Duration:")
        {
            result.push(trimmed.to_string());
        }
        // Capture failures
        else if trimmed.contains("FAILED") || trimmed.contains("✕") || trimmed.contains("Error:") {
            failures.push(trimmed.to_string());
        }
    }

    if !failures.is_empty() {
        result.push("--- Failures ---".to_string());
        result.extend(failures.into_iter().take(15));
    }

    if result.is_empty() {
        "Tests passed".to_string()
    } else {
        result.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composer_filter_matches() {
        let filter = ComposerFilter;
        assert!(filter.matches("composer"));
        assert!(filter.matches("Composer"));
        assert!(!filter.matches("npm"));
    }

    #[test]
    fn test_phpunit_filter_matches() {
        let filter = PhpunitFilter;
        assert!(filter.matches("phpunit"));
        assert!(filter.matches("./vendor/bin/phpunit"));
    }
}
