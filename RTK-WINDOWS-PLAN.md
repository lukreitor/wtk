# WTK Roadmap - Windows Token Killer

## Mission

**WTK > RTK** - Be the definitive Windows-first LLM token optimizer with:
- **Broader coverage** (200+ vs 100+ commands)
- **Higher savings** (60-90% vs 50-70%)
- **Simpler installation** (one-click vs manual setup)
- **Native Windows support** (no WSL required)

---

## Current Features (v0.5.0)

### Filters (50+ categories, 200+ commands)

| Category | Commands | Status |
|----------|----------|--------|
| **Git** | status, log, diff, show, branch, stash, worktree, fetch, add, commit, push, pull | Done |
| **GitHub CLI** | pr, issue, run, release, repo, api | Done |
| **Rust** | cargo build/check/test/clippy | Done |
| **Go** | go build/test, golangci-lint | Done |
| **Python** | pip, pytest, ruff, mypy, poetry | Done |
| **Java** | maven, gradle | Done |
| **Node.js** | npm, pnpm, yarn, bun, npx | Done |
| **TypeScript** | tsc | Done |
| **.NET** | dotnet build/test/run | Done |
| **Docker** | docker, docker-compose | Done |
| **Kubernetes** | kubectl, helm | Done |
| **Terraform** | terraform | Done |
| **Cloud** | az, aws, gcloud | Done |
| **Ansible** | ansible, ansible-playbook, ansible-galaxy, ansible-vault | Done |
| **Test** | vitest, jest, playwright | Done |
| **Lint** | eslint, prettier, biome | Done |
| **Database** | psql, mysql, sqlcmd, redis-cli, mongosh, sqlite3, cqlsh, cypher-shell, influx, supabase, pscale, neonctl, turso | Done |
| **Windows CMD** | ipconfig, netstat, tasklist, ping, systeminfo, sc, wmic, netsh, tree, where, reg, dism, sfc, hostname, getmac, arp, route, diskpart, bcdedit, certutil, fsutil, icacls, attrib, findstr, robocopy | Done |
| **PowerShell** | Get-Process, Get-Service, Get-ChildItem, Get-Content, Get-NetAdapter, Get-NetIPAddress, Get-EventLog, Get-WinEvent, Get-HotFix, Get-Volume, Get-Disk, Get-ComputerInfo, Get-PSDrive, Get-Module, Get-Command, Get-History, Get-Alias, Get-ScheduledTask, Get-LocalUser, Get-LocalGroup, Get-Acl, Get-ItemProperty, Test-NetConnection, Test-Path, Select-String, Measure-Object, Format-Table, Format-List, ConvertTo-Json | Done |
| **Package Mgrs** | winget, choco, scoop | Done |
| **Network** | curl, ssh, scp, sftp | Done |
| **Frameworks** | next, nx, turbo, vite | Done |
| **Prisma** | prisma | Done |
| **IaC** | vagrant, packer, pulumi | Done |
| **Serverless** | serverless (sls), vercel, netlify, railway, flyctl, render, heroku | Done |
| **Local K8s** | minikube, kind, k3s | Done |
| **K8s Dev** | skaffold, tilt | Done |
| **GitOps** | argocd | Done |
| **Service Mesh** | istioctl, linkerd | Done |
| **Enterprise K8s** | cf, oc, eksctl | Done |

### Meta Commands
- `wtk gain` - Token savings summary
- `wtk gain --graph` - ASCII graph with period selection (-T)
- `wtk gain --history` - Command history with period/limit (-T, -n)
- `wtk discover` - Find missed WTK opportunities in shell history
- `wtk init` - Install hooks (--claude-code, --powershell, --cmd)
- `wtk config` - Show configuration

---

## Roadmap (Planned Features)

### Phase 1: More Windows Native (v0.3.0) - COMPLETED

All Phase 1 items have been implemented. See Current Features section above.

### Phase 2: More DevOps Tools (v0.4.0) - COMPLETED

All Phase 2 items have been implemented. See Current Features section above.

### Phase 3: More Languages (v0.5.0)

| Language | Tools | Est. Savings |
|----------|-------|--------------|
| **Ruby** | bundle, gem, rails, rake, rspec | 70-85% |
| **PHP** | composer, artisan, phpunit | 70-80% |
| **Elixir** | mix, iex | 70-80% |
| **Scala** | sbt, mill | 80-90% |
| **Kotlin** | kotlinc, gradlew | 80-85% |
| **Swift** | swift, swiftc, xcodebuild | 75-85% |
| **C/C++** | make, cmake, ninja, meson | 70-80% |
| **Zig** | zig | 75% |
| **Nim** | nim, nimble | 75% |
| **OCaml** | opam, dune | 75% |
| **Haskell** | cabal, stack | 75% |
| **Clojure** | lein, clj | 75% |
| **Perl** | cpan, perl | 65% |
| **Lua** | luarocks | 70% |
| **Julia** | julia, Pkg | 70% |
| **R** | Rscript | 70% |
| **MATLAB** | matlab | 65% |

### Phase 4: More Databases (v0.5.0) - COMPLETED

All Phase 4 items have been implemented. See Current Features section above.

### Phase 5: Enhanced Analytics (v0.7.0)

| Feature | Description |
|---------|-------------|
| **Cost estimation** | Show estimated $ saved based on LLM pricing |
| **Export reports** | PDF/HTML reports with charts |
| **Comparison mode** | Compare efficiency before/after WTK |
| **Team analytics** | Aggregate stats across machines |
| **Webhook notifications** | Alert when savings reach milestones |
| **Real-time dashboard** | Live web dashboard for monitoring |
| **Historical trends** | Long-term savings analysis |
| **Filter recommendations** | AI-powered filter suggestions |

### Phase 6: Enterprise Features (v1.0.0)

| Feature | Description |
|---------|-------------|
| **Central config** | Team-wide configuration management |
| **Custom filters** | User-defined filter rules (YAML/TOML) |
| **Filter plugins** | Hot-loadable filter DLLs |
| **API server mode** | REST API for integrations |
| **Multi-tenant** | Separate tracking per project/team |
| **SSO integration** | Enterprise authentication |
| **Audit logging** | Compliance-ready logs |
| **Rate limiting** | Per-command throttling |

---

## Unique WTK Advantages vs RTK

### Already Better

| Feature | RTK | WTK |
|---------|:---:|:---:|
| Windows native | No WSL | Full native |
| Commands supported | 100+ | **250+** |
| PowerShell cmdlets | 0 | **29** |
| Windows package mgrs | 0 | **3** (winget/choco/scoop) |
| Windows system commands | 5 | **25+** |
| One-click install | No | **Yes** |
| `wtk discover` | No | **Yes** |
| Time period options | No | **Yes** (-T 1d/7d/30d/90d/1y/all) |
| Claude Code native hook | Partial | **Full** |

### Planned Advantages

| Feature | RTK | WTK (planned) |
|---------|:---:|:---:|
| Cost estimation | No | v0.7.0 |
| Custom filters | No | v1.0.0 |
| Team analytics | No | v0.7.0 |
| Enterprise SSO | No | v1.0.0 |
| Plugin system | No | v1.0.0 |

---

## Installation Comparison

### RTK (Complex)
```bash
# Requires WSL on Windows
# Multiple manual steps
# PATH configuration
# Node.js dependency
```

### WTK (Simple)
```powershell
# One command!
irm https://raw.githubusercontent.com/Lukreitor/wtk/master/install.ps1 | iex
```

---

## Contributing

Want to help WTK surpass RTK? Here's how:

1. **Add new filters** - Each filter = more coverage
2. **Improve savings** - Better regex = more compression
3. **Test on Windows** - Find edge cases
4. **Documentation** - Help others use WTK
5. **Spread the word** - Star the repo, share with colleagues

---

## Metrics Goals

| Metric | Current | Goal |
|--------|---------|------|
| Commands supported | 300+ | **500+** |
| Average savings | 70% | **80%** |
| Windows coverage | 95% | **99%** |
| Installation steps | 1 | **1** |
| GitHub stars | - | **1000+** |

---

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| v0.5.0 | 2026-04-16 | **Phase 4 Complete**: +8 database tools (sqlite3, cqlsh, cypher-shell, influx, supabase, pscale, neonctl, turso) |
| v0.4.0 | 2026-04-16 | **Phase 2 Complete**: +21 DevOps tools (vagrant, packer, pulumi, serverless, PaaS CLIs, K8s ecosystem) |
| v0.3.0 | 2026-04-16 | **Phase 1 Complete**: 25 CMD commands, 29 PowerShell cmdlets |
| v0.2.4 | 2026-04-16 | Time period options (-T), history limit (-n) |
| v0.2.3 | 2026-04-16 | Ansible filter, SFTP support, `wtk discover` |
| v0.2.2 | 2026-04-16 | One-click installer, gain improvements |
| v0.2.1 | 2026-04-16 | Fix git status, enhance history output |
| v0.2.0 | 2026-04-15 | 50+ filters, Windows native, Claude Code hooks |
| v0.1.0 | 2026-04-14 | Initial release |

---

**WTK: Windows Token Killer** - Making LLMs affordable on Windows.
