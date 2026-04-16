<p align="center">
  <h1 align="center">WTK - Windows Token Killer</h1>
  <p align="center">
    <strong>CLI proxy that reduces LLM token consumption by 60-90% on Windows</strong>
  </p>
  <p align="center">
    <a href="https://github.com/Lukreitor/wtk/actions"><img src="https://github.com/Lukreitor/wtk/workflows/CI/badge.svg" alt="CI Status"></a>
    <a href="https://github.com/Lukreitor/wtk/releases"><img src="https://img.shields.io/github/v/release/Lukreitor/wtk" alt="Release"></a>
    <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.75+-orange.svg" alt="Rust 1.75+"></a>
    <img src="https://img.shields.io/badge/platform-Windows-blue.svg" alt="Platform: Windows">
  </p>
</p>

---

## Overview

**WTK (Windows Token Killer)** is a high-performance CLI proxy designed specifically for Windows that filters and compresses command outputs before they reach your LLM context. It achieves **60-90% token savings** on common development operations through smart filtering, grouping, truncation, and deduplication.

### Why WTK?

- **Windows-First**: Built from the ground up for Windows with native PowerShell and CMD support
- **Deterministic Hooks**: Integrates with Claude Code via hooks (not CLAUDE.md), ensuring 100% consistent command rewriting
- **200+ Commands**: Supports git, npm, dotnet, docker, kubectl, terraform, PowerShell cmdlets, and more
- **Zero Configuration**: Run `wtk init --claude-code` and you're done

### Key Features

| Feature | Description |
|---------|-------------|
| **Smart Filtering** | Removes noise, boilerplate, and verbose output |
| **Grouping** | Aggregates similar items (errors by type, files by directory) |
| **Truncation** | Preserves relevant context while cutting redundancy |
| **Deduplication** | Collapses repeated log lines with counts |
| **Gain Tracking** | Real-time statistics on token savings |

---

## Installation

### From Releases (Recommended)

Download the latest release from [GitHub Releases](https://github.com/Lukreitor/wtk/releases):

```powershell
# PowerShell
Invoke-WebRequest -Uri "https://github.com/Lukreitor/wtk/releases/latest/download/wtk-windows-x64.zip" -OutFile "wtk.zip"
Expand-Archive -Path "wtk.zip" -DestinationPath "$env:LOCALAPPDATA\wtk"
$env:PATH += ";$env:LOCALAPPDATA\wtk"
```

### From Source

```bash
# Clone the repository
git clone https://github.com/Lukreitor/wtk.git
cd wtk

# Build release binary
cargo build --release

# Install globally
cargo install --path .
```

### Verify Installation

```bash
wtk --version
# wtk 0.2.0
```

---

## Quick Start

### 1. Initialize Claude Code Integration

```bash
wtk init --claude-code
```

This automatically configures Claude Code hooks for command rewriting.

### 2. Use WTK Commands

```bash
# Git commands
wtk git status
wtk git log --oneline -10
wtk git diff

# Package managers
wtk npm install
wtk pnpm build
wtk dotnet build

# Windows system
wtk ipconfig /all
wtk netstat -an
wtk Get-Process
```

### 3. Check Your Savings

```bash
wtk gain
# Shows token savings statistics
```

---

## Command Coverage

### Version Control (50-85% savings)

| Command | Subcommands | Savings |
|---------|-------------|---------|
| `git` | status, log, diff, show, branch, stash, blame | 50-85% |
| `gh` | pr, issue, run, release, repo, gist, api | 70-87% |

### Package Managers (70-90% savings)

| Command | Type | Savings |
|---------|------|---------|
| `npm`, `pnpm`, `yarn`, `bun` | Node.js | 70-85% |
| `pip`, `poetry`, `uv` | Python | 70-80% |
| `dotnet`, `nuget` | .NET | 70-85% |
| `mvn`, `gradle` | Java | 85-92% |
| `cargo` | Rust | 80-90% |
| `go` | Go | 80-90% |
| `winget`, `choco`, `scoop` | Windows | 75-85% |

### Build Tools & Test Runners (70-99% savings)

| Command | Type | Savings |
|---------|------|---------|
| `tsc`, `webpack`, `vite` | Build | 70-90% |
| `vitest`, `jest`, `playwright` | Test | 90-99% |
| `eslint`, `prettier`, `ruff` | Lint | 70-84% |
| `msbuild`, `dotnet build` | .NET Build | 85-90% |

### DevOps & Infrastructure (70-90% savings)

| Command | Type | Savings |
|---------|------|---------|
| `docker`, `docker-compose` | Containers | 75-85% |
| `kubectl`, `helm` | Kubernetes | 70-85% |
| `terraform` | IaC | 80-90% |
| `ansible-playbook` | Automation | 80-85% |
| `az`, `aws`, `gcloud` | Cloud CLIs | 75-80% |

### Windows System (50-85% savings)

| Command | Type | Savings |
|---------|------|---------|
| `ipconfig`, `netstat`, `tasklist` | CMD | 75-80% |
| `Get-Process`, `Get-Service` | PowerShell | 70-75% |
| `Get-EventLog`, `Get-WinEvent` | Logs | 85% |

### SSH & Remote (65-75% savings)

| Command | Type | Savings |
|---------|------|---------|
| `ssh`, `scp`, `sftp` | OpenSSH | 65-70% |
| `plink`, `pscp`, `psftp` | PuTTY | 65-70% |

---

## Claude Code Integration

WTK integrates with Claude Code via **deterministic hooks**, not CLAUDE.md instructions. This ensures 100% consistent behavior.

### How It Works

```
┌─────────────────────────────────────────────────────────────┐
│                      Claude Code                             │
├─────────────────────────────────────────────────────────────┤
│  PreToolUse Hook (Bash)                                      │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ Input:  git status                                       │ │
│  │ WTK:    wtk rewrite "git status"                        │ │
│  │ Output: {"updatedInput": {"command": "wtk git status"}} │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Configuration

After running `wtk init --claude-code`, your `~/.claude/settings.json` will contain:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": { "tool_name": "Bash" },
        "hooks": [{
          "type": "command",
          "command": "wtk rewrite"
        }]
      }
    ]
  }
}
```

---

## Gain Tracking

WTK tracks all command executions and calculates token savings in real-time.

### View Statistics

```bash
# Summary
wtk gain

# Detailed history
wtk gain --history

# By filter
wtk gain --by-filter

# Weekly breakdown
wtk gain --weekly

# Export as JSON
wtk gain --format json
```

### Example Output

```
WTK Token Savings
═══════════════════════════════════════════════════════════════

Total commands:    156
Input tokens:      45.2K
Output tokens:     12.1K
Tokens saved:      33.1K (73.2%)
Efficiency meter: ████████████████████░░░░ 73.2%

By Command
───────────────────────────────────────────────────────────────
  #  Command              Count  Saved    Avg%   Impact
───────────────────────────────────────────────────────────────
 1.  wtk git status          45   8.2K   82.3%  ██████████
 2.  wtk npm run build       12   7.1K   71.5%  ████████░░
 3.  wtk docker ps           23   5.4K   78.9%  ██████░░░░
 4.  wtk Get-Process         18   4.8K   72.1%  █████░░░░░
```

---

## Configuration

### Config File

WTK uses `~/.config/wtk/config.toml` for configuration:

```toml
[tracking]
enabled = true
history_days = 90

[display]
colors = true
max_width = 120

[filters]
ignore_dirs = [".git", "node_modules", "target"]

[hooks]
claude_code = true
powershell = true
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `WTK_CONFIG` | Config file path | `~/.config/wtk/config.toml` |
| `WTK_LOG` | Log level | `warn` |
| `WTK_NO_COLOR` | Disable colors | `false` |

---

## Architecture

```
wtk/
├── src/
│   ├── main.rs              # Entry point
│   ├── cli/                 # CLI commands (clap)
│   ├── filters/             # Command filters
│   │   ├── git/             # Git filters
│   │   ├── node/            # Node.js filters
│   │   ├── dotnet/          # .NET filters
│   │   ├── docker/          # Docker filters
│   │   ├── kubernetes/      # Kubernetes filters
│   │   └── windows/         # Windows CMD/PowerShell
│   ├── hooks/               # Hook installers
│   ├── tracking/            # Gain tracking (SQLite)
│   └── compress/            # Compression algorithms
├── tests/                   # Integration tests
└── benches/                 # Benchmarks
```

---

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone
git clone https://github.com/Lukreitor/wtk.git
cd wtk

# Build
cargo build

# Run tests
cargo test

# Run with debug logging
WTK_LOG=debug cargo run -- git status
```

### Adding a New Filter

1. Create a new file in `src/filters/<category>/`
2. Implement the `Filter` trait
3. Register the filter in `src/filters/registry.rs`
4. Add tests in `tests/`

---

## Comparison with RTK

| Feature | RTK | WTK |
|---------|-----|-----|
| **Platform Focus** | Unix-first | Windows-first |
| **Commands** | ~60 | 200+ |
| **Claude Integration** | CLAUDE.md (advisory) | Hooks (deterministic) |
| **Windows System** | Minimal | 50+ commands |
| **PowerShell** | No | 25+ cmdlets |
| **Java** | No | mvn, gradle, ant |
| **Windows Hooks** | Broken | Native PS + CMD |

---

## License

MIT License - see [LICENSE](LICENSE) for details.

---

## Acknowledgments

Inspired by [RTK (Rust Token Killer)](https://github.com/rtk-ai/rtk).

---

<p align="center">
  <sub>Built with Rust for the Windows developer community</sub>
</p>
