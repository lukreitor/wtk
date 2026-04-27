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
  <img src="https://img.shields.io/badge/commands-300+-green.svg" alt="200+ Commands">
</p>

---

## Real-World Token Savings

```
┌─────────────────────────────────────────────────────────────────┐
│                    Measured Token Savings                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Total commands:    924                                         │
│   Input tokens:      9.0M                                        │
│   Output tokens:     420.6K                                      │
│   Tokens saved:      8.6M (95.3%)                                │
│                                                                  │
│   Efficiency: ████████████████████████░  95.3%                   │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│   find (recursive) ×2     6.4M saved    (99.9% reduction)        │
│   tasklist ×12          310.6K saved    (98.6% reduction)        │
│   grep -r ×1            139.2K saved    (97.5% reduction)        │
│   find (deep) ×3        193.9K saved    (97.1% reduction)        │
│   find -maxdepth 3 ×1    58.6K saved    (98.3% reduction)        │
│   find (compact) ×3     130.1K saved    (96.5% reduction)        │
│   find (small) ×1        42.1K saved    (97.9% reduction)        │
│   Get-Process ×1         38.7K saved    (99.5% reduction)        │
│   Get-Service ×1         22.2K saved    (99.4% reduction)        │
│   ipconfig ×12           19.4K saved    (91.7% reduction)        │
└─────────────────────────────────────────────────────────────────┘
```

> **Note**: Numbers from real `wtk gain` over 30 days, 9 active days, 924 commands.
> `find` recursive scans dominate — single Unix-style recursive find on a large tree saves 60K-3M+ tokens each.
> PowerShell cmdlets like `Get-Process` and `Get-Service` show 99%+ reduction due to their extremely verbose default output.

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
| **PHP/C/C++ filters** | ❌ None | ✅ composer, artisan, make, cmake, gcc |
| **Ansible** | ❌ None | ✅ ansible, ansible-playbook, galaxy, vault |
| **PaaS/Serverless** | ❌ None | ✅ vercel, netlify, railway, flyctl, heroku |
| **Cloud DBs** | ❌ None | ✅ supabase, pscale, neonctl, turso |
| **Command filters** | 100+ | **300+ (60+ filter classes)** |
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
wtk composer install    # PHP
wtk make                # C/C++
wtk cmake --build .     # C/C++

# DevOps
wtk docker ps
wtk kubectl get pods
wtk terraform plan
wtk ansible-playbook site.yml
wtk pulumi up
wtk vagrant status
wtk argocd app list

# PaaS / Serverless
wtk vercel deploy
wtk serverless deploy
wtk flyctl status

# Databases
wtk supabase db push
wtk pscale branch list
wtk neonctl branches list
wtk turso db list

# Search & file discovery
wtk grep -r "fn main" src/
wtk rg "TODO" --type rust
wtk find . -name "*.rs" -type f
wtk fd "\.ts$" src/
wtk env

# Windows
wtk ipconfig /all
wtk Get-Process
wtk Get-ScheduledTask
wtk winget list
wtk robocopy src dst /MIR
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
| `robocopy` | **80%** | Summary: files copied/skipped/failed |
| `findstr` | **70%** | Matches only, no decorations |
| `icacls` | **70%** | Compact ACL list |
| `certutil` | **75%** | Key fields only |
| `diskpart` | **65%** | Compact volume/disk list |
| `fsutil` | **65%** | Key filesystem info |

### PowerShell Cmdlets (Measured)
| Cmdlet | Savings | Description |
|--------|:-------:|-------------|
| `Get-Process` | **99.5%** | Top 5 by CPU, process count |
| `Get-Service` | **99.4%** | Running/stopped count, top services |
| `Get-EventLog` | **85%** | Error/warning/info counts, recent |
| `Get-ChildItem` | **65%** | Dirs/files count, first 15 items |
| `Get-ComputerInfo` | **70%** | OS, version, memory only |
| `Get-Module` | **75%** | Loaded modules compact list |
| `Get-Command` | **80%** | Filtered command list |
| `Get-ScheduledTask` | **85%** | Active tasks only |
| `Get-LocalUser` | **80%** | User list compact |
| `Get-Acl` | **70%** | Key permissions only |
| `Test-NetConnection` | **65%** | Success/fail + latency |
| `Select-String` | **70%** | Matches with file:line |

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
| `composer install/update` | PHP | **85%** |
| `php artisan` | Laravel/PHP | **80%** |
| `phpunit/pest` | PHP Testing | **90%** |
| `make/cmake` | C/C++ Build | **80%** |
| `gcc/g++/clang` | C/C++ Compile | **75%** |
| `ninja` | C/C++ Build | **80%** |

### Test Runners
| Command | Savings |
|---------|:-------:|
| `vitest` | **99%** |
| `jest` | **95%** |
| `playwright` | **94%** |
| `pytest` | **90%** |
| `phpunit` | **90%** |
| `pest` | **90%** |

### DevOps & Cloud
| Command | Type | Savings |
|---------|------|:-------:|
| `docker ps/images` | Containers | **85%** |
| `kubectl get` | Kubernetes | **85%** |
| `terraform plan` | IaC | **90%** |
| `az/aws/gcloud` | Cloud CLIs | **80%** |
| `ansible/ansible-playbook` | Automation | **85%** |
| `ansible-galaxy/vault` | Ansible | **80%** |
| `pulumi up/preview` | IaC | **85%** |
| `vagrant status/up` | VMs | **80%** |
| `packer build` | Images | **80%** |
| `vercel/netlify` | PaaS Deploy | **75%** |
| `railway/flyctl/render` | PaaS Deploy | **75%** |
| `heroku` | PaaS | **75%** |
| `minikube/kind/k3s` | Local K8s | **80%** |
| `skaffold/tilt` | K8s Dev | **80%** |
| `argocd` | GitOps | **85%** |
| `istioctl/linkerd` | Service Mesh | **80%** |
| `eksctl` | EKS | **80%** |

### Package Managers
| Command | Savings |
|---------|:-------:|
| `winget list` | **80%** |
| `choco list` | **80%** |
| `scoop list` | **75%** |

### Databases
| Command | Type | Savings |
|---------|------|:-------:|
| `psql` | PostgreSQL | **75%** |
| `mysql` | MySQL | **75%** |
| `sqlcmd` | SQL Server | **75%** |
| `redis-cli` | Redis | **80%** |
| `sqlite3` | SQLite | **70%** |
| `supabase` | Supabase | **80%** |
| `pscale` | PlanetScale | **80%** |
| `neonctl` | Neon (Postgres) | **80%** |
| `turso` | Turso (LibSQL) | **80%** |
| `influx` | InfluxDB | **75%** |
| `cqlsh` | Cassandra | **75%** |
| `cypher-shell` | Neo4j | **75%** |

### Search & File Discovery
| Command | Savings | Description |
|---------|:-------:|-------------|
| `grep` | **80%** | Grouped by file: N matches per file, 3 lines context |
| `rg` / `ripgrep` | **80%** | Same as grep — file:line:content format |
| `find` | **75%** | Grouped by dir: N files per dir (uses Unix find on Windows) |
| `fd` / `fdfind` | **75%** | Compact file list grouped by directory |
| `env` / `printenv` | **55-85%** | Hides system vars, masks `*_KEY/*_SECRET/*_TOKEN`, formats PATH |

### Network & SSH
| Command | Savings |
|---------|:-------:|
| `sftp/psftp` | **70%** | Compact transfer summary |
| `ssh` | **65%** | Passthrough with error filter |
| `curl` | **70%** | Headers + body compact |

### Infrastructure Automation
| Command | Savings |
|---------|:-------:|
| `serverless/sls` | **85%** | Deployment summary only |

---

## Gain Graph

```
$ wtk gain --graph

📈 WTK Token Savings - Last 30 Days
════════════════════════════════════════════════════════════

    6.9M │           ██
         │           ██
    5.2M │           ██
         │           ██
         │           ██
    3.5M │           ██
         │           ██
         │           ██
    1.7M │           ██
         │           ██
         │           ██
       0 │ ▄▄    ▄▄  ██
         └───────────────────
           04-16        04-27

📊 Summary (Last 30 Days)
──────────────────────────────────────────────────
  Period:          Last 30 Days
  Days with data:  9
  Total saved:     8.6M
  Total input:     9.0M
  Commands:        925
  Avg efficiency:  95.3%

  Efficiency: ████████████████████████░  95.3%

📅 Other periods:
  → -T 1d (24h) | -T 7d (week) | -T 90d (3 months) | -T all
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
│   ├── filters/             # 60+ command filters
│   │   ├── git/             # Git operations
│   │   ├── docker/          # Docker/Compose
│   │   ├── kubernetes/      # kubectl/helm
│   │   ├── cloud/           # az/aws/gcloud
│   │   ├── rust/            # Cargo
│   │   ├── golang/          # Go
│   │   ├── python/          # pip/pytest/ruff
│   │   ├── java/            # Maven/Gradle
│   │   ├── node/            # npm/pnpm/yarn
│   │   ├── php/             # composer/artisan/phpunit/pest
│   │   ├── cpp/             # make/cmake/gcc/g++/clang/ninja
│   │   ├── windows/         # CMD commands (25+)
│   │   ├── powershell/      # PowerShell cmdlets (29+)
│   │   ├── winpkg/          # winget/choco/scoop
│   │   ├── database/        # psql/mysql/redis/sqlite3/supabase/pscale/neon/turso
│   │   ├── terraform/       # Terraform
│   │   ├── devops/          # vagrant/packer/pulumi/serverless/vercel/argocd/istio
│   │   ├── ansible/         # ansible/ansible-playbook/galaxy/vault
│   │   ├── network/         # curl/ssh/sftp/psftp
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
# Token counting strategy:
#   "bytes"  — fast, default. Counts UTF-8 bytes; gain reports show
#              "Chars saved" + a chars÷4 heuristic estimate.
#   "cl100k" — real BPE tokenizer (OpenAI cl100k_base, ~95% Claude proxy).
#              Adds ~5-30ms per command. New rows store actual token counts;
#              gain reports show real "Tokens saved" alongside chars.
tokenizer = "bytes"

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
| `WTK_TOKENIZER` | Token counting: `bytes` (heuristic) or `cl100k` (real BPE). Overrides config. | `bytes` |

### Real-token mode (cl100k)

By default WTK measures **byte counts**, then estimates tokens as `chars ÷ 4` —
the same heuristic RTK uses. Fast but imprecise: file paths and code can be
±30% off. To get real LLM token counts (within ~5% of Claude's tokenizer):

```bash
# Per-call
WTK_TOKENIZER=cl100k wtk find . -name "*.rs"

# Persistent for the shell
export WTK_TOKENIZER=cl100k

# Or via config.toml [tracking] tokenizer = "cl100k"
```

Adds ~5-30ms per filter call (BPE encode of raw stdout/stderr). Reports show
both "Chars saved" and "Tokens saved" when DB has tokenized rows; the
`Active tokenizer:` line at the bottom of `wtk gain` confirms the live mode.

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

### v0.8.0 (2026-04-27)

**✨ Features**
- **tokenizer**: Optional real BPE tokenizer (`cl100k_base`) for accurate
  LLM token counts. Default remains the fast `bytes` heuristic. Activate via
  `WTK_TOKENIZER=cl100k`, the `tokenizer` key in `~/.config/wtk/config.toml`,
  or per-shell export. Reports show both chars and tokens when DB has
  tokenized rows, with coverage indicator (`N/M rows, cl100k`).
- **gain**: Display now shows the active tokenizer at the bottom of the
  summary, with a tip to enable cl100k when running in heuristic mode.

**🗄️ Database**
- Forward-only migration on next open: adds `tokens_input`, `tokens_output`,
  and `tokenizer_kind` columns. Existing rows keep NULL for these — queries
  use COALESCE / IS NOT NULL to handle the mixed state. No data loss.

**🐛 Honesty Fixes**
- `wtk gain` no longer labels byte counts as "tokens" without qualification.
  The `≈ X tokens` line is now `~X (heuristic: chars÷4)` with explicit hint.
- Renamed internal `format_tokens` → `format_count` to match its actual job.
- Graph summary uses `Chars saved` / `Input chars` instead of generic
  `Total saved` / `Total input`.

**📦 Filters Migrated to Token-Aware API**
- `find` / `fd`, `grep` / `rg`, `env`, `git`, all `windows/*` (tasklist,
  ipconfig, ...), all PowerShell cmdlets (`Get-Process`, `Get-Service`,
  `Get-ChildItem`, generic). Other filters fall back to the byte heuristic
  when cl100k is active — they still record chars correctly.

---

### v0.7.0 (2026-04-27)

**✨ Features**
- **filters**: Add `grep` / `rg` / `ripgrep` filter — group matches by file, configurable context
- **filters**: Add `find` / `fd` / `fdfind` filter — group results by directory, compact listings
- **filters**: Add `env` / `printenv` filter — hide system vars, mask `*_KEY` / `*_SECRET` / `*_TOKEN`, format PATH

**📊 Real-World Stats**
- 924 commands tracked, 8.6M tokens saved, 95.3% avg efficiency
- Recursive `find` scans top savers (60K–6.4M tokens per call)

---

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
