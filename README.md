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
│                    30-Minute Claude Code Session                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   WITHOUT WTK                          WITH WTK                  │
│   ═══════════                          ════════                  │
│   Input:  118,247 tokens               Input:  23,891 tokens     │
│   ████████████████████████████████     ██████░░░░░░░░░░░░░░░░░░  │
│                                                                  │
│                     SAVED: 94,356 tokens (79.8%)                 │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│   git status ×45      8,234 → 1,647    (80.0% saved)            │
│   npm run build ×12   7,102 → 2,025    (71.5% saved)            │
│   docker ps ×23       5,412 → 1,140    (78.9% saved)            │
│   Get-Process ×18     4,821 → 1,345    (72.1% saved)            │
│   cargo build ×8      3,890 →   389    (90.0% saved)            │
└─────────────────────────────────────────────────────────────────┘
```

---

## Why WTK?

| Feature | RTK | WTK |
|---------|:---:|:---:|
| **Windows Native** | ❌ WSL only | ✅ Full support |
| **Commands** | 100+ | **200+** |
| **PowerShell** | ❌ | ✅ 25+ cmdlets |
| **Windows Hooks** | ❌ Broken | ✅ Native |
| **winget/choco/scoop** | ❌ | ✅ |
| **Claude Code Hook** | ✅ | ✅ |

**WTK is the Windows-first solution** for LLM token optimization.

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

### Version Control (50-85%)
| Command | Subcommands | Savings |
|---------|-------------|---------|
| `git` | status, log, diff, show, branch, stash | 50-85% |
| `gh` | pr, issue, run, release, repo, api | 70-87% |

### Build & Languages (70-92%)
| Command | Type | Savings |
|---------|------|---------|
| `cargo` | Rust | 80-90% |
| `go` | Go | 80-90% |
| `npm/pnpm/yarn` | Node.js | 70-85% |
| `pip/poetry` | Python | 70-80% |
| `mvn/gradle` | Java | 85-92% |
| `dotnet` | .NET | 70-85% |

### Test Runners (90-99%)
| Command | Savings |
|---------|---------|
| `vitest` | 99% |
| `jest` | 95% |
| `playwright` | 94% |
| `pytest` | 90% |
| `cargo test` | 90% |

### DevOps & Cloud (70-90%)
| Command | Type | Savings |
|---------|------|---------|
| `docker` | Containers | 75-85% |
| `kubectl/helm` | Kubernetes | 70-85% |
| `terraform` | IaC | 80-90% |
| `ansible` | Configuration | 75-85% |
| `az/aws/gcloud` | Cloud CLIs | 75-80% |

### Windows Native (70-85%)
| Command | Type | Savings |
|---------|------|---------|
| `ipconfig/netstat/tasklist` | CMD | 75-80% |
| `Get-Process/Get-Service` | PowerShell | 70-75% |
| `winget/choco/scoop` | Package Mgrs | 75-85% |

### Databases (70-85%)
| Command | Savings |
|---------|---------|
| `psql` | 75% |
| `mysql` | 75% |
| `sqlcmd` | 75% |
| `redis-cli` | 80% |
| `mongosh` | 70% |

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
│  │  WTK:    wtk rewrite "git status"                         │  │
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

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Built with Rust for Windows developers</sub>
</p>
