//! Filter traits and types.

use anyhow::Result;

/// Result of a filtered command execution.
#[derive(Debug, Clone)]
pub struct FilterResult {
    /// The filtered output
    pub output: String,
    /// Number of characters in the original output
    pub input_chars: usize,
    /// Number of characters in the filtered output
    pub output_chars: usize,
    /// Execution time in milliseconds
    pub exec_time_ms: u64,
}

impl FilterResult {
    /// Create a new FilterResult.
    pub fn new(output: String, input_chars: usize, exec_time_ms: u64) -> Self {
        let output_chars = output.len();
        Self {
            output,
            input_chars,
            output_chars,
            exec_time_ms,
        }
    }

    /// Calculate the percentage of tokens saved.
    pub fn savings_percent(&self) -> f64 {
        if self.input_chars == 0 {
            0.0
        } else {
            ((self.input_chars - self.output_chars) as f64 / self.input_chars as f64) * 100.0
        }
    }
}

/// Trait for command filters.
pub trait Filter: Send + Sync {
    /// Get the filter name (e.g., "git-status").
    fn name(&self) -> &'static str;

    /// Check if this filter matches the given command.
    fn matches(&self, command: &str) -> bool;

    /// Execute the command and return filtered output.
    fn execute(&self, command: &str, args: &[String]) -> Result<FilterResult>;

    /// Get the filter priority (higher = checked first).
    fn priority(&self) -> u8 {
        50
    }
}
