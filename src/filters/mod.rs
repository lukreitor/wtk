//! Filters module - Command output filters for token optimization.

pub mod registry;
pub mod traits;

// Core filters
pub mod dotnet;
pub mod git;
pub mod github;
pub mod network;
pub mod node;
pub mod prisma;
pub mod typescript;
pub mod windows;

// Language filters
pub mod rust;
pub mod golang;
pub mod python;
pub mod java;

// DevOps filters
pub mod docker;
pub mod kubernetes;
pub mod terraform;
pub mod cloud;
pub mod ansible;

// Testing & Linting
pub mod test;
pub mod lint;

// Database filters
pub mod database;

// Package managers
pub mod winpkg;

// PowerShell
pub mod powershell;

// Frameworks
pub mod frameworks;

// Core exports
pub use dotnet::DotnetFilter;
pub use git::GitFilter;
pub use github::GhFilter;
pub use network::{CurlFilter, ScpFilter, SshFilter, SftpFilter};
pub use node::NodePackageFilter;
pub use prisma::PrismaFilter;
pub use registry::FilterRegistry;
pub use traits::{Filter, FilterResult};
pub use typescript::TscFilter;
pub use windows::WindowsSystemFilter;

// Language exports
pub use rust::CargoFilter;
pub use golang::{GoFilter, GolangciLintFilter};
pub use python::{PipFilter, PytestFilter, RuffFilter, MypyFilter, PoetryFilter};
pub use java::{MavenFilter, GradleFilter};

// DevOps exports
pub use docker::{DockerFilter, DockerComposeFilter};
pub use kubernetes::{KubectlFilter, HelmFilter};
pub use terraform::TerraformFilter;
pub use cloud::{AzFilter, AwsFilter, GcloudFilter};
pub use ansible::AnsibleFilter;

// Testing & Linting exports
pub use test::{VitestFilter, JestFilter, PlaywrightFilter};
pub use lint::{EslintFilter, PrettierFilter, BiomeFilter};

// Database exports
pub use database::{PsqlFilter, MysqlFilter, SqlcmdFilter, RedisCliFilter, MongoshFilter};

// Package manager exports
pub use winpkg::{WingetFilter, ChocoFilter, ScoopFilter};

// PowerShell exports
pub use powershell::{PowerShellFilter, GetProcessFilter, GetServiceFilter, GetChildItemFilter};

// Framework exports
pub use frameworks::{NextFilter, NxFilter, TurboFilter, ViteFilter};
