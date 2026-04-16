//! Filter registry - Central registry for all command filters.

use super::git::GitFilter;
use super::github::GhFilter;
use super::network::{CurlFilter, ScpFilter, SshFilter};
use super::node::NodePackageFilter;
use super::windows::WindowsSystemFilter;
use super::Filter;

/// Central registry for all command filters.
pub struct FilterRegistry {
    filters: Vec<Box<dyn Filter>>,
}

impl FilterRegistry {
    /// Create a new filter registry with all registered filters.
    pub fn new() -> Self {
        let mut filters: Vec<Box<dyn Filter>> = vec![
            // Git filter (handles all git subcommands)
            Box::new(GitFilter),
            // GitHub CLI
            Box::new(GhFilter),
            // Node.js ecosystem (npm, pnpm, yarn, bun, npx)
            Box::new(NodePackageFilter),
            // Network tools
            Box::new(CurlFilter),
            Box::new(SshFilter),
            Box::new(ScpFilter),
            // Windows system commands
            Box::new(WindowsSystemFilter),
        ];

        // Sort by priority (descending)
        filters.sort_by(|a, b| b.priority().cmp(&a.priority()));

        Self { filters }
    }

    /// Find a filter that matches the given command.
    pub fn find_filter(&self, command: &str) -> Option<&dyn Filter> {
        self.filters
            .iter()
            .find(|f| f.matches(command))
            .map(|f| f.as_ref())
    }

    /// Get all registered filters.
    pub fn all(&self) -> &[Box<dyn Filter>] {
        &self.filters
    }

    /// Get the number of registered filters.
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }
}

impl Default for FilterRegistry {
    fn default() -> Self {
        Self::new()
    }
}
