//! Filters module - Command output filters for token optimization.

pub mod registry;
pub mod traits;

pub mod git;
// pub mod node;
// pub mod dotnet;
// pub mod docker;
// pub mod windows;

pub use git::GitFilter;
pub use registry::FilterRegistry;
pub use traits::{Filter, FilterResult};
