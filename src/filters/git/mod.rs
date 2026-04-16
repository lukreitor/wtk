//! Git command filters.

mod status;
mod log;
mod diff;

use anyhow::Result;
use std::process::Command;
use std::time::Instant;

use super::{Filter, FilterResult};

pub use status::filter_status_output;
pub use log::filter_log_output;
pub use diff::filter_diff_output;

/// Unified filter for all git commands.
pub struct GitFilter;

impl Filter for GitFilter {
    fn name(&self) -> &'static str {
        "git"
    }

    fn matches(&self, command: &str) -> bool {
        command == "git"
    }

    fn execute(&self, _command: &str, args: &[String]) -> Result<FilterResult> {
        let subcommand = args.first().map(|s| s.as_str()).unwrap_or("");

        let start = Instant::now();

        // Execute git command
        let output = Command::new("git")
            .args(args)
            .output()?;

        let exec_time_ms = start.elapsed().as_millis() as u64;
        let raw_output = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let input_chars = raw_output.len() + stderr.len();

        // Apply appropriate filter based on subcommand
        let filtered = match subcommand {
            "status" => filter_status_output(&raw_output),
            "log" => filter_log_output(&raw_output),
            "diff" | "show" => filter_diff_output(&raw_output),
            // Passthrough for other commands with minimal filtering
            _ => {
                // For non-filtered commands, still capture output but don't filter
                if !stderr.is_empty() && raw_output.is_empty() {
                    stderr
                } else if !stderr.is_empty() {
                    format!("{}\n{}", raw_output, stderr)
                } else {
                    raw_output.clone()
                }
            }
        };

        Ok(FilterResult::new(filtered, input_chars, exec_time_ms))
    }

    fn priority(&self) -> u8 {
        100 // High priority for git commands
    }
}
