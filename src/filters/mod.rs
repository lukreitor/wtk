//! Filters module - Command output filters for token optimization.

pub mod registry;
pub mod traits;

pub mod git;
pub mod github;
pub mod network;
pub mod node;
pub mod windows;

pub use git::GitFilter;
pub use github::GhFilter;
pub use network::{CurlFilter, ScpFilter, SshFilter};
pub use node::NodePackageFilter;
pub use registry::FilterRegistry;
pub use traits::{Filter, FilterResult};
pub use windows::WindowsSystemFilter;
