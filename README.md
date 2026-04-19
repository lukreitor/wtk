<p align="center">
  <img src="https://img.shields.io/badge/🔪_WTK-Windows_Token_Killer-blue?style=for-the-badge" alt="WTK">
</p>

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
  <img src="https://img.shields.io/badge/commands-200+-green.svg" alt="200+ Commands">
</p>

---

## Real-World Token Savings

```
┌─────────────────────────────────────────────────────────────────┐
│                    Measured Token Savings                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Total commands:    47                                          │
│   Input tokens:      261.5K                                      │
│   Output tokens:     13.1K                                       │
│   Tokens saved:      248.3K (95.0%)                              │
│                                                                  │
│   Efficiency: ████████████████████████░  95.0%                   │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│   tasklist ×6         147.0K saved    (98.5% reduction)         │
│   Get-Process ×1       38.7K saved    (99.5% reduction)         │
│   Get-Service ×1       22.2K saved    (99.4% reduction)         │
│   systeminfo ×2        11.2K saved    (98.3% reduction)         │
│   ipconfig ×6           9.7K saved    (91.7% reduction)         │
│   netstat -an ×1        7.1K saved    (96.1% reduction)         │
│   git status ×10        2.8K saved    (72.3% reduction)         │
│   git log ×1            2.2K saved    (85.8% reduction)         │
│   ping ×5               1.5K saved    (91.9% reduction)         │
└─────────────────────────────────────────────────────────────────┘
```

> **Note**: Savings measured from real WTK usage. PowerShell cmdlets like `Get-Process` and `Get-Service` show 99%+ reduction due to their extremely verbose default output.

---

## Why WTK Over RTK?

RTK is great on Unix/Linux but **doesn't work properly on Windows**:

| Feature | RTK (Unix-first) | WTK (Windows-first) |
|---------|:----------------:|:-------------------:|
| **Windows Native** | ❌ WSL required | ✅ Full native support |
| **PowerShell cmdlets** | ❌ None | ✅ Get-Process, Get-Service, etc |
| **CMD commands** | ❌ None | ✅ ipconfig, tasklist, netstat |
| **winget/choco/scoop** | ❌ None | ✅ All 3 package managers |
| **Windows path handling** | ❌ Breaks on `C:\` | ✅ Native path support |
| **Command filters** | 100+ | **200+ (50 filter classes)** |
| **Binary size** | ~5MB | ~5MB (Rust optimized) |
| **Claude Code Hooks** | ✅ | ✅ + Windows shell hooks |

### The Problem with RTK on Windows

```
# RTK on Windows - these fail or return unfiltered:
rtk ipconfig        # ❌ No filter
rtk Get-Process     # ❌ No filter
rtk tasklist        # ❌ No filter
rtk winget list     # ❌ No filter

# WTK - all work natively:
wtk ipconfig        # ✅ 91.7% reduction
wtk Get-Process     # ✅ 72.1% reduction
wtk tasklist        # ✅ 98.5% reduction
wtk winget list     # ✅ 80% reduction
```

**WTK is the Windows-native solution** for LLM token optimization.

---

## Installation

<details>
<summary><b>📦 From Releases (Recommended)</b></summary>

### PowerShell (Admin)
```powershell
# Download and install
$url = "https://github.com/Lukreitor/wtk/releases/latest/download/wtk-windows-x64.zip"
Invoke-WebRequest -Uri $url -OutFile "$env:TEMP\wtk.zip"
Expand-Archive -Path "$env:TEMP\wtk.zip" -DestinationPath "$env:LOCALAPPDATA\wtk" -Force

# Add to PATH
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*wtk*") {
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$env:LOCALAPPDATA\wtk", "User")
}

# Verify
wtk --version
```

### CMD
```batch
:: Download manually from GitHub releases
:: Extract to %LOCALAPPDATA%\wtk
:: Add to PATH via System Properties > Environment Variables
```

</details>

<details>
<summary><b>🔨 From Source</b></summary>

```bash
# Prerequisites: Rust 1.75+, Visual Studio Build Tools
git clone https://github.com/Lukreitor/wtk.git
cd wtk

# Build
cargo build --release

# Install
cargo install --path .

# Verify
wtk --version
```

</details>

<details>
<summary><b>⚡ One-Line Install (PowerShell)</b></summary>

```powershell
irm https://raw.githubusercontent.com/Lukreitor/wtk/master/install.ps1 | iex
```

</details>

---

## Quick Start

### 1. Initialize Claude Code Hooks

```bash
wtk init --claude-code
```

This automatically rewrites commands to use WTK. **100% transparent, zero manual effort.**

### 2. Use WTK Commands

```bash
# Git
wtk git status
wtk git log --oneline -10
wtk git diff

# Build tools
wtk cargo build
wtk npm install
wtk dotnet build

# DevOps
wtk docker ps
wtk kubectl get pods
wtk terraform plan

# Windows
wtk ipconfig /all
wtk Get-Process
wtk winget list
```

### 3. Track Your Savings

```bash
# Summary
wtk gain

# ASCII graph (30 days default)
wtk gain --graph

# Different time periods
wtk gain --graph -T 7d      # Last week
wtk gain --graph -T 90d     # Last 3 months
wtk gain --graph -T all     # All time

# Command history with options
wtk gain --history          # Last 20 commands, 30 days
wtk gain --history -T 1d    # Last 24 hours
wtk gain --history -n 50    # Show 50 entries

# Find missed savings
wtk discover
```

---

## Token Savings by Category

### Windows Native (Measured)
| Command | Measured Savings | Description |
|---------|:----------------:|-------------|
| `tasklist` | **98.5%** | Top 10 by memory, process count |
| `systeminfo` | **98.3%** | Key system info only |
| `netstat -an` | **96.1%** | Connection summary, top connections |
| `ping` | **91.9%** | Success/fail summary, avg latency |
| `ipconfig` | **91.7%** | Active adapters with IPs only |
| `tracert` | **75%** | Condensed hop list |

### PowerShell Cmdlets (Measured)
| Cmdlet | Savings | Description |
|--------|:-------:|-------------|
| `Get-Process` | **99.5%** | Top 5 by CPU, process count |
| `Get-Service` | **99.4%** | Running/stopped count, top services |
| `Get-EventLog` | **85%** | Error/warning/info counts, recent |
| `Get-ChildItem` | **65%** | Dirs/files count, first 15 items |
| `Get-ComputerInfo` | **70%** | OS, version, memory only |

### Version Control (Measured)
| Command | Savings | Description |
|---------|:-------:|-------------|
| `git log` | **85.8%** | Short hash + message only |
| `git status` | **73.7%** | Compact M/A/D/? format |
| `git diff` | **80%** | Condensed diff |
| `gh pr view` | **87%** | Key PR info only |
| `gh run list` | **82%** | Compact workflow list |

### Build & Languages
| Command | Type | Savings |
|---------|------|:-------:|
| `cargo build/test` | Rust | **90%** |
| `go build/test` | Go | **90%** |
| `npm/pnpm/yarn` | Node.js | **85%** |
| `pip/poetry` | Python | **80%** |
| `mvn/gradle` | Java | **90%** |
| `dotnet build` | .NET | **85%** |

### Test Runners
| Command | Savings |
|---------|:-------:|
| `vitest` | **99%** |
| `jest` | **95%** |
| `playwright` | **94%** |
| `pytest` | **90%** |

### DevOps & Cloud
| Command | Type | Savings |
|---------|------|:-------:|
| `docker ps/images` | Containers | **85%** |
| `kubectl get` | Kubernetes | **85%** |
| `terraform plan` | IaC | **90%** |
| `az/aws/gcloud` | Cloud CLIs | **80%** |

### Package Managers
| Command | Savings |
|---------|:-------:|
| `winget list` | **80%** |
| `choco list` | **80%** |
| `scoop list` | **75%** |

### Databases
| Command | Savings |
|---------|:-------:|
| `psql` | **75%** |
| `mysql` | **75%** |
| `sqlcmd` | **75%** |
| `redis-cli` | **80%** |

---

## Gain Graph

```
$ wtk gain --graph -T 7d

WTK Token Savings - Last 7 Days
════════════════════════════════════════════════════════════

 136.7K │ ████████████████████████████████
        │ ████████████████████████████████
 102.5K │ ████████████████████████████████
        │ ████████████████████████████████
  68.3K │ ████████████████████████████████
        │ ████████████████████████████████
  34.2K │ ████████████████████████████████
        │ ████████████████████████████████
       0│────────────────────────────────
         04-10                       04-16

Summary (Last 7 Days)
────────────────────────────────────
  Total saved:     136.7K
  Commands:        23
  Avg efficiency:  96.8%

Periods: -T 1d | -T 7d | -T 30d | -T 90d | -T 1y | -T all
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Claude Code                              │
├─────────────────────────────────────────────────────────────────┤
│  PreToolUse Hook                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Input:  git status                                        │  │
│  │  WTK:    wtk rewrite  (reads from stdin)                         │  │
│  │  Output: {"updatedInput": {"command": "wtk git status"}}  │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                           WTK                                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐     │
│  │  Filter  │   │  Filter  │   │  Filter  │   │  Filter  │     │
│  │   Git    │   │  Docker  │   │  Cargo   │   │   ...    │     │
│  └────┬─────┘   └────┬─────┘   └────┬─────┘   └────┬─────┘     │
│       │              │              │              │            │
│       └──────────────┴──────────────┴──────────────┘            │
│                              │                                   │
│                       ┌──────┴──────┐                           │
│                       │   Registry  │                           │
│                       │  (50+ filters)                          │
│                       └──────┬──────┘                           │
│                              │                                   │
│                       ┌──────┴──────┐                           │
│                       │  Tracking   │                           │
│                       │  (SQLite)   │                           │
│                       └─────────────┘                           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    Compressed Output
                    (60-90% smaller)
```

### Project Structure

```
wtk/
├── src/
│   ├── main.rs              # Entry point
│   ├── cli/                 # CLI (clap)
│   │   ├── mod.rs           # Command definitions
│   │   └── commands.rs      # Command handlers
│   ├── filters/             # 50+ command filters
│   │   ├── git/             # Git operations
│   │   ├── docker/          # Docker/Compose
│   │   ├── kubernetes/      # kubectl/helm
│   │   ├── cloud/           # az/aws/gcloud
│   │   ├── rust/            # Cargo
│   │   ├── golang/          # Go
│   │   ├── python/          # pip/pytest/ruff
│   │   ├── java/            # Maven/Gradle
│   │   ├── node/            # npm/pnpm/yarn
│   │   ├── windows/         # CMD/PowerShell
│   │   ├── winpkg/          # winget/choco/scoop
│   │   ├── database/        # psql/mysql/redis
│   │   ├── terraform/       # Terraform
│   │   ├── test/            # vitest/jest/playwright
│   │   ├── lint/            # eslint/prettier
│   │   ├── frameworks/      # next/nx/vite
│   │   └── registry.rs      # Filter registry
│   ├── hooks/               # Hook installers
│   ├── tracking/            # SQLite tracking
│   └── compress/            # Compression utils
├── tests/                   # Integration tests
└── .github/workflows/       # CI/CD
```

---

## Configuration

### Config File

`~/.config/wtk/config.toml`:

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

## Contributing

```bash
# Clone
git clone https://github.com/Lukreitor/wtk.git
cd wtk

# Build
cargo build

# Test
cargo test

# Run with debug
WTK_LOG=debug cargo run -- git status
```

### Adding a New Filter

1. Create `src/filters/<category>/mod.rs`
2. Implement `Filter` trait
3. Register in `src/filters/registry.rs`
4. Add tests

---

## Acknowledgments

Inspired by [RTK (Rust Token Killer)](https://github.com/rtk-ai/rtk).

---

## Changelog

### v0.6.1 (2026-04-19)

**🐛 Bug Fixes**
- **hooks**: Fix `wtk init --claude-code` generating wrong matcher format
  - Was: `"matcher": { "tool_name": "Bash" }` (invalid object)
  - Now: `"matcher": "Bash"` (correct string format per Claude Code spec)
- **rewrite**: Fix `wtk rewrite` to read from stdin (Claude Code hook protocol)
  - Was: required positional `<COMMAND>` argument — caused all hook invocations to fail
  - Now: reads JSON from stdin when invoked without args, extracts `tool_input.command`
  - Backwards compatible: still accepts optional positional arg for manual testing

---

### v0.6.0 (2026-04-17)

**🐛 Bug Fixes**
- **hooks**: Fix Claude Code hook installation to use absolute path with forward slashes
  - Changed from relative `wtk` to absolute `C:/Users/.../.cargo/bin/wtk.exe`
  - Added path normalization with `canonicalize()` + UNC prefix removal
  - Claude Code now correctly invokes wtk on Windows
  - Resolves hook execution failures on system paths

**🔧 Improvements**
- Improved hook installer compatibility with Windows path formats
- Better cross-platform path handling in hook configuration

---

## What's Coming

### v0.3.0 (Next Release)
- [ ] **SSH session filtering** - Filter output from remote commands
- [ ] **WSL integration** - Seamless filtering for WSL commands
- [ ] **Custom filter plugins** - Load user-defined filters from TOML/Lua
- [ ] **Filter chaining** - Pipe multiple filters together

### v0.4.0
- [ ] **Real-time streaming** - Filter output as it arrives (not just at completion)
- [ ] **GitHub Copilot CLI integration** - `gh copilot` command filtering
- [ ] **VS Code extension** - Direct integration with VS Code terminal
- [ ] **More PowerShell cmdlets** - Get-Event, Get-Counter, Get-WmiObject

### Planned Features
- **Filter debugging mode** - See exactly what gets filtered and why
- **Output diff view** - Compare raw vs filtered output
- **AI-assisted filter creation** - Generate filters from example outputs
- **Cloud CLI improvements** - Better az/aws/gcloud filtering with profile awareness
- **Container log deduplication** - Smart dedup for repetitive container logs

### Community Contributions Welcome
- New filter implementations (see [CONTRIBUTING.md](CONTRIBUTING.md))
- Windows command coverage expansions
- Performance optimizations
- Documentation improvements

---

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Built with Rust for Windows developers</sub>
</p>
