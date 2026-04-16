//! Filters module - Command output filters for token optimization.

pub mod registry;
pub mod traits;

pub mod dotnet;
pub mod git;
pub mod github;
pub mod network;
pub mod node;
pub mod prisma;
pub mod typescript;
pub mod windows;

pub use dotnet::DotnetFilter;
pub use git::GitFilter;
pub use github::GhFilter;
pub use network::{CurlFilter, ScpFilter, SshFilter};
pub use node::NodePackageFilter;
pub use prisma::PrismaFilter;
pub use registry::FilterRegistry;
pub use traits::{Filter, FilterResult};
pub use typescript::TscFilter;
pub use windows::WindowsSystemFilter;
