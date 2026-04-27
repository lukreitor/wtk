//! Filter traits and types.

use anyhow::Result;

/// Result of a filtered command execution.
#[derive(Debug, Clone)]
pub struct FilterResult {
    /// The filtered output
    pub output: String,
    /// Bytes in the original (raw, pre-filter) output
    pub input_chars: usize,
    /// Bytes in the filtered output
    pub output_chars: usize,
    /// Execution time in milliseconds
    pub exec_time_ms: u64,
    /// Raw pre-filter text — stored when present so the tracking layer can
    /// run a real tokenizer (e.g. cl100k_base) on it. Filters that use the
    /// `with_raw` constructor populate this; legacy `new` callers leave it
    /// `None` (tokenizer mode then falls back to the bytes/4 heuristic).
    pub raw_input: Option<String>,
}

impl FilterResult {
    /// Create a new FilterResult from a precomputed `input_chars` count.
    /// Use `with_raw` instead when the raw text is still available — that
    /// enables real-tokenizer counting in the tracking layer.
    pub fn new(output: String, input_chars: usize, exec_time_ms: u64) -> Self {
        let output_chars = output.len();
        Self {
            output,
            input_chars,
            output_chars,
            exec_time_ms,
            raw_input: None,
        }
    }

    /// Create a FilterResult that retains the raw pre-filter text for
    /// downstream token counting. `input_chars` is computed from `raw`'s
    /// byte length to stay consistent with the `new` constructor.
    pub fn with_raw(output: String, raw: String, exec_time_ms: u64) -> Self {
        let input_chars = raw.len();
        let output_chars = output.len();
        Self {
            output,
            input_chars,
            output_chars,
            exec_time_ms,
            raw_input: Some(raw),
        }
    }

    /// Calculate the percentage of bytes saved.
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
