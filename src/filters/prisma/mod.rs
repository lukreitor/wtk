//! Prisma CLI filter.

use anyhow::Result;
use regex::Regex;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

/// Filter for Prisma CLI commands.
pub struct PrismaFilter;

impl Filter for PrismaFilter {
    fn name(&self) -> &'static str {
        "prisma"
    }

    fn matches(&self, command: &str) -> bool {
        matches!(command.to_lowercase().as_str(),
            "prisma" | "prisma.cmd" | "npx"
        )
    }

    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult> {
        // For npx, check if it's running prisma
        if command == "npx" {
            if !args.first().map(|a| a.contains("prisma")).unwrap_or(false) {
                anyhow::bail!("Not a prisma command");
            }
        }

        let subcommand = if command == "npx" {
            args.get(1).map(|s| s.as_str()).unwrap_or("")
        } else {
            args.first().map(|s| s.as_str()).unwrap_or("")
        };

        let start = Instant::now();

        let output = Command::new(command)
            .args(args)
            .output()?;

        let exec_time_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = stdout.len() + stderr.len();

        let filtered = match subcommand {
            "generate" => filter_generate(&stdout, &stderr),
            "migrate" => filter_migrate(&stdout, &stderr, args),
            "db" => filter_db(&stdout, &stderr, args),
            "studio" => filter_studio(&stdout),
            "format" => filter_format(&stdout),
            "validate" => filter_validate(&stdout, &stderr),
            "push" => filter_push(&stdout, &stderr),
            _ => filter_generic(&stdout, &stderr),
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        85
    }
}

/// Filter prisma generate output.
fn filter_generate(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);

    // Remove ASCII art and decorative lines
    let cleaned = remove_ascii_art(&combined);

    let mut result = Vec::new();

    for line in cleaned.lines() {
        let trimmed = line.trim();

        // Skip empty lines and decorative
        if trimmed.is_empty() ||
           trimmed.starts_with("✔") ||
           trimmed.starts_with("Prisma schema") {
            continue;
        }

        // Capture key info
        if trimmed.contains("Generated") ||
           trimmed.contains("generated") ||
           trimmed.contains("client") {
            result.push(trimmed.to_string());
        }

        // Errors
        if trimmed.contains("Error") || trimmed.contains("error") {
            result.push(format!("✗ {}", trimmed));
        }
    }

    if result.is_empty() {
        "✓ prisma client generated".to_string()
    } else {
        result.join("\n")
    }
}

/// Filter prisma migrate output.
fn filter_migrate(stdout: &str, stderr: &str, args: &[String]) -> String {
    let combined = format!("{}\n{}", stdout, stderr);
    let cleaned = remove_ascii_art(&combined);

    let subsubcmd = args.iter()
        .skip_while(|a| *a != "migrate")
        .nth(1)
        .map(|s| s.as_str())
        .unwrap_or("");

    let mut result = Vec::new();

    match subsubcmd {
        "dev" | "deploy" => {
            let migration_re = Regex::new(r"Applying migration `(\d+_.+)`").unwrap();
            let applied_re = Regex::new(r"(\d+)\s+migration").unwrap();

            let mut applied = Vec::new();

            for line in cleaned.lines() {
                let trimmed = line.trim();

                if let Some(caps) = migration_re.captures(trimmed) {
                    applied.push(caps[1].to_string());
                }

                if trimmed.contains("already in sync") ||
                   trimmed.contains("up to date") {
                    return "✓ database already in sync".to_string();
                }

                if trimmed.contains("Error") || trimmed.contains("error") {
                    result.push(format!("✗ {}", trimmed));
                }
            }

            if !applied.is_empty() {
                result.insert(0, format!("✓ applied {} migrations", applied.len()));
                for m in applied.iter().take(5) {
                    result.push(format!("  {}", m));
                }
                if applied.len() > 5 {
                    result.push(format!("  ... +{} more", applied.len() - 5));
                }
            }
        }
        "status" => {
            let mut pending = Vec::new();
            let mut applied_count = 0;

            for line in cleaned.lines() {
                let trimmed = line.trim();

                if trimmed.contains("not yet applied") {
                    if let Some(name) = trimmed.split_whitespace().next() {
                        pending.push(name.to_string());
                    }
                }

                if trimmed.contains("have been applied") {
                    if let Some(num) = trimmed.split_whitespace().next() {
                        applied_count = num.parse().unwrap_or(0);
                    }
                }
            }

            if !pending.is_empty() {
                result.push(format!("⚠ {} pending, {} applied", pending.len(), applied_count));
                for p in pending.iter().take(5) {
                    result.push(format!("  → {}", p));
                }
            } else {
                result.push(format!("✓ {} migrations applied, none pending", applied_count));
            }
        }
        "reset" => {
            if cleaned.contains("Reset") || cleaned.contains("reset") {
                result.push("✓ database reset".to_string());
            }
        }
        _ => {}
    }

    if result.is_empty() {
        filter_generic(stdout, stderr)
    } else {
        result.join("\n")
    }
}

/// Filter prisma db output.
fn filter_db(stdout: &str, stderr: &str, args: &[String]) -> String {
    let combined = format!("{}\n{}", stdout, stderr);
    let cleaned = remove_ascii_art(&combined);

    let subsubcmd = args.iter()
        .skip_while(|a| *a != "db")
        .nth(1)
        .map(|s| s.as_str())
        .unwrap_or("");

    match subsubcmd {
        "push" => filter_push(&cleaned, ""),
        "pull" => {
            if cleaned.contains("schema.prisma") || cleaned.contains("introspected") {
                "✓ schema pulled from database".to_string()
            } else if cleaned.contains("Error") {
                filter_generic(stdout, stderr)
            } else {
                "✓ db pull completed".to_string()
            }
        }
        "seed" => {
            if cleaned.contains("seed") && !cleaned.contains("Error") {
                "✓ database seeded".to_string()
            } else {
                filter_generic(stdout, stderr)
            }
        }
        _ => filter_generic(stdout, stderr),
    }
}

/// Filter prisma push output.
fn filter_push(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);
    let cleaned = remove_ascii_art(&combined);

    let mut changes = Vec::new();

    for line in cleaned.lines() {
        let trimmed = line.trim();

        if trimmed.contains("Created") ||
           trimmed.contains("Added") ||
           trimmed.contains("Altered") ||
           trimmed.contains("Dropped") {
            changes.push(trimmed.to_string());
        }

        if trimmed.contains("in sync") {
            return "✓ database in sync".to_string();
        }

        if trimmed.contains("Error") || trimmed.contains("error") {
            return format!("✗ {}", trimmed);
        }
    }

    if !changes.is_empty() {
        let mut result = vec![format!("✓ pushed {} changes", changes.len())];
        for c in changes.iter().take(5) {
            result.push(format!("  {}", c));
        }
        if changes.len() > 5 {
            result.push(format!("  ... +{} more", changes.len() - 5));
        }
        result.join("\n")
    } else {
        "✓ db push completed".to_string()
    }
}

/// Filter prisma studio output.
fn filter_studio(stdout: &str) -> String {
    let port_re = Regex::new(r"localhost:(\d+)").unwrap();

    if let Some(caps) = port_re.captures(stdout) {
        format!("✓ Prisma Studio running at http://localhost:{}", &caps[1])
    } else {
        "✓ Prisma Studio starting...".to_string()
    }
}

/// Filter prisma format output.
fn filter_format(stdout: &str) -> String {
    let cleaned = remove_ascii_art(stdout);

    if cleaned.contains("formatted") || cleaned.contains("Formatted") {
        "✓ schema formatted".to_string()
    } else if cleaned.contains("already") {
        "✓ schema already formatted".to_string()
    } else {
        "✓ prisma format completed".to_string()
    }
}

/// Filter prisma validate output.
fn filter_validate(stdout: &str, stderr: &str) -> String {
    let combined = format!("{}\n{}", stdout, stderr);
    let cleaned = remove_ascii_art(&combined);

    let mut errors = Vec::new();

    for line in cleaned.lines() {
        let trimmed = line.trim();

        if trimmed.contains("Error") ||
           trimmed.contains("error") ||
           trimmed.contains("✖") {
            errors.push(trimmed.to_string());
        }
    }

    if errors.is_empty() {
        "✓ schema valid".to_string()
    } else {
        let mut result = vec![format!("✗ {} validation errors", errors.len())];
        for e in errors.iter().take(5) {
            result.push(format!("  {}", e));
        }
        result.join("\n")
    }
}

/// Generic prisma output filter.
fn filter_generic(stdout: &str, stderr: &str) -> String {
    let combined = if !stderr.is_empty() && stdout.is_empty() {
        stderr.to_string()
    } else if !stderr.is_empty() {
        format!("{}\n{}", stdout, stderr)
    } else {
        stdout.to_string()
    };

    let cleaned = remove_ascii_art(&combined);

    let lines: Vec<&str> = cleaned.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.len() > 15 {
        let mut result: Vec<String> = lines.iter().take(12).map(|s| s.to_string()).collect();
        result.push(format!("... +{} more lines", lines.len() - 12));
        result.join("\n")
    } else {
        lines.join("\n")
    }
}

/// Remove Prisma ASCII art and decorative elements.
fn remove_ascii_art(text: &str) -> String {
    let art_patterns = [
        "╔", "╗", "║", "╚", "╝", "═",
        "┌", "┐", "│", "└", "┘", "─",
        "Prisma ",
        "◭",
    ];

    text.lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Skip empty lines
            if trimmed.is_empty() {
                return false;
            }
            // Skip lines that are mostly decorative
            if art_patterns.iter().any(|p| trimmed.starts_with(p)) {
                return false;
            }
            // Skip lines that are just version info
            if trimmed.contains("@prisma/client") && trimmed.contains("✔") {
                return true; // Keep this one
            }
            true
        })
        .collect::<Vec<_>>()
        .join("\n")
}
