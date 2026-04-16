//! Filter registry - Central registry for all command filters.

// Core filters
use super::dotnet::DotnetFilter;
use super::git::GitFilter;
use super::github::GhFilter;
use super::network::{CurlFilter, ScpFilter, SshFilter, SftpFilter};
use super::node::NodePackageFilter;
use super::prisma::PrismaFilter;
use super::typescript::TscFilter;
use super::windows::WindowsSystemFilter;

// Language filters
use super::rust::CargoFilter;
use super::golang::{GoFilter, GolangciLintFilter};
use super::python::{PipFilter, PytestFilter, RuffFilter, MypyFilter, PoetryFilter};
use super::java::{MavenFilter, GradleFilter};

// DevOps filters
use super::docker::{DockerFilter, DockerComposeFilter};
use super::kubernetes::{KubectlFilter, HelmFilter};
use super::terraform::TerraformFilter;
use super::cloud::{AzFilter, AwsFilter, GcloudFilter};
use super::ansible::AnsibleFilter;

// Testing & Linting filters
use super::test::{VitestFilter, JestFilter, PlaywrightFilter};
use super::lint::{EslintFilter, PrettierFilter, BiomeFilter};

// Database filters
use super::database::{PsqlFilter, MysqlFilter, SqlcmdFilter, RedisCliFilter, MongoshFilter};

// Package manager filters
use super::winpkg::{WingetFilter, ChocoFilter, ScoopFilter};

// PowerShell filters
use super::powershell::{PowerShellFilter, GetProcessFilter, GetServiceFilter, GetChildItemFilter};

// Framework filters
use super::frameworks::{NextFilter, NxFilter, TurboFilter, ViteFilter};

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
            // TypeScript compiler
            Box::new(TscFilter),
            // .NET CLI
            Box::new(DotnetFilter),
            // Prisma CLI
            Box::new(PrismaFilter),
            // Network tools
            Box::new(CurlFilter),
            Box::new(SshFilter),
            Box::new(ScpFilter),
            Box::new(SftpFilter),
            // Windows system commands
            Box::new(WindowsSystemFilter),

            // Language filters
            Box::new(CargoFilter),
            Box::new(GoFilter),
            Box::new(GolangciLintFilter),
            Box::new(PipFilter),
            Box::new(PytestFilter),
            Box::new(RuffFilter),
            Box::new(MypyFilter),
            Box::new(PoetryFilter),
            Box::new(MavenFilter),
            Box::new(GradleFilter),

            // DevOps filters
            Box::new(DockerFilter),
            Box::new(DockerComposeFilter),
            Box::new(KubectlFilter),
            Box::new(HelmFilter),
            Box::new(TerraformFilter),
            Box::new(AzFilter),
            Box::new(AwsFilter),
            Box::new(GcloudFilter),
            Box::new(AnsibleFilter),

            // Testing & Linting filters
            Box::new(VitestFilter),
            Box::new(JestFilter),
            Box::new(PlaywrightFilter),
            Box::new(EslintFilter),
            Box::new(PrettierFilter),
            Box::new(BiomeFilter),

            // Database filters
            Box::new(PsqlFilter),
            Box::new(MysqlFilter),
            Box::new(SqlcmdFilter),
            Box::new(RedisCliFilter),
            Box::new(MongoshFilter),

            // Package manager filters
            Box::new(WingetFilter),
            Box::new(ChocoFilter),
            Box::new(ScoopFilter),

            // PowerShell filters
            Box::new(PowerShellFilter),
            Box::new(GetProcessFilter),
            Box::new(GetServiceFilter),
            Box::new(GetChildItemFilter),

            // Framework filters
            Box::new(NextFilter),
            Box::new(NxFilter),
            Box::new(TurboFilter),
            Box::new(ViteFilter),
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
