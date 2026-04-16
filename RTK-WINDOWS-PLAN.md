# WTK Roadmap - Windows Token Killer

## Mission

**WTK > RTK** - Be the definitive Windows-first LLM token optimizer with:
- **Broader coverage** (200+ vs 100+ commands)
- **Higher savings** (60-90% vs 50-70%)
- **Simpler installation** (one-click vs manual setup)
- **Native Windows support** (no WSL required)

---

## Current Features (v0.3.0)

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
| **Database** | psql, mysql, sqlcmd, redis-cli, mongosh | Done |
| **Windows CMD** | ipconfig, netstat, tasklist, ping, systeminfo, sc, wmic, netsh, tree, where, reg, dism, sfc, hostname, getmac, arp, route | Done |
| **PowerShell** | Get-Process, Get-Service, Get-ChildItem | Done |
| **Package Mgrs** | winget, choco, scoop | Done |
| **Network** | curl, ssh, scp, sftp | Done |
| **Frameworks** | next, nx, turbo, vite | Done |
| **Prisma** | prisma | Done |

### Meta Commands
- `wtk gain` - Token savings summary
- `wtk gain --graph` - ASCII graph with period selection (-T)
- `wtk gain --history` - Command history with period/limit (-T, -n)
- `wtk discover` - Find missed WTK opportunities in shell history
- `wtk init` - Install hooks (--claude-code, --powershell, --cmd)
- `wtk config` - Show configuration

---

## Roadmap (Planned Features)

### Phase 1: More Windows Native (v0.3.0)

#### CMD Commands (High Priority)
| Command | Description | Est. Savings |
|---------|-------------|--------------|
| `wmic` | WMI queries (deprecated but still used) | 80% |
| `diskpart` | Disk management | 70% |
| `netsh` | Network shell | 75% |
| `dism` | Deployment Image Servicing | 85% |
| `sfc` | System File Checker | 70% |
| `bcdedit` | Boot config | 75% |
| `reg query` | Registry queries | 70% |
| `certutil` | Certificate management | 75% |
| `fsutil` | File system utilities | 70% |
| `icacls` | File permissions | 70% |
| `attrib` | File attributes | 65% |
| `tree` | Directory tree | 60% |
| `where` | Find executables | 60% |
| `findstr` | Pattern search | 65% |
| `robocopy` | Robust file copy | 70% |

#### PowerShell Cmdlets (High Priority)
| Cmdlet | Description | Est. Savings |
|--------|-------------|--------------|
| `Get-NetAdapter` | Network adapters | 75% |
| `Get-NetIPAddress` | IP addresses | 75% |
| `Get-EventLog` | Event logs (verbose!) | 90% |
| `Get-WinEvent` | Windows events | 90% |
| `Get-HotFix` | Windows updates | 75% |
| `Get-Volume` | Disk volumes | 70% |
| `Get-Disk` | Disk info | 70% |
| `Get-PSDrive` | Drives | 65% |
| `Get-Module` | Installed modules | 75% |
| `Get-Command` | Available commands | 80% |
| `Get-History` | Command history | 60% |
| `Get-Alias` | Aliases | 70% |
| `Get-Content` | File content | 50% |
| `Get-ScheduledTask` | Task scheduler | 75% |
| `Get-LocalUser` | Local users | 70% |
| `Get-LocalGroup` | Local groups | 70% |
| `Get-Acl` | Permissions | 75% |
| `Get-ItemProperty` | Registry values | 70% |
| `Test-NetConnection` | Network test | 70% |
| `Test-Path` | Path validation | 50% |
| `Select-String` | Pattern search | 65% |
| `Measure-Object` | Stats | 60% |
| `Format-Table` | Format output | 50% |
| `Format-List` | Format output | 50% |
| `ConvertTo-Json` | JSON output | 40% |

### Phase 2: More DevOps Tools (v0.4.0)

| Tool | Description | Est. Savings |
|------|-------------|--------------|
| `vagrant` | VM management | 75% |
| `packer` | Image building | 80% |
| `pulumi` | IaC | 80% |
| `cdktf` | CDK for Terraform | 80% |
| `serverless` | Serverless framework | 75% |
| `sam` | AWS SAM CLI | 75% |
| `flyctl` | Fly.io CLI | 70% |
| `railway` | Railway CLI | 70% |
| `render` | Render CLI | 70% |
| `vercel` | Vercel CLI | 70% |
| `netlify` | Netlify CLI | 70% |
| `heroku` | Heroku CLI | 75% |
| `cf` | Cloud Foundry CLI | 75% |
| `oc` | OpenShift CLI | 75% |
| `eksctl` | EKS CLI | 75% |
| `gke` | GKE commands | 75% |
| `aks` | AKS commands | 75% |
| `minikube` | Local Kubernetes | 70% |
| `kind` | Kubernetes in Docker | 70% |
| `k3s` | Lightweight k8s | 70% |
| `skaffold` | Kubernetes dev | 75% |
| `tilt` | Local dev | 70% |
| `argocd` | GitOps | 75% |
| `istioctl` | Service mesh | 75% |
| `linkerd` | Service mesh | 75% |

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

### Phase 4: More Databases (v0.6.0)

| Database | Tools | Est. Savings |
|----------|-------|--------------|
| **SQLite** | sqlite3 | 70% |
| **CouchDB** | curl (CouchDB) | 70% |
| **Cassandra** | cqlsh | 75% |
| **Neo4j** | cypher-shell | 70% |
| **InfluxDB** | influx | 75% |
| **Elasticsearch** | curl (ES) | 70% |
| **DynamoDB** | aws dynamodb | 75% |
| **Cosmos DB** | az cosmosdb | 75% |
| **Firestore** | gcloud firestore | 75% |
| **Supabase** | supabase | 70% |
| **PlanetScale** | pscale | 70% |
| **Neon** | neonctl | 70% |
| **Turso** | turso | 70% |

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
| Commands supported | 100+ | **200+** |
| PowerShell cmdlets | 0 | **25+** |
| Windows package mgrs | 0 | **3** (winget/choco/scoop) |
| Windows system commands | 5 | **15+** |
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
| Commands supported | 200+ | **500+** |
| Average savings | 70% | **80%** |
| Windows coverage | 85% | **99%** |
| Installation steps | 1 | **1** |
| GitHub stars | - | **1000+** |

---

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| v0.3.0 | 2026-04-16 | Phase 1 CMD filters: wmic, netsh, tree, where, sc, reg, dism, sfc, getmac, arp, route |
| v0.2.4 | 2026-04-16 | Time period options (-T), history limit (-n) |
| v0.2.3 | 2026-04-16 | Ansible filter, SFTP support, `wtk discover` |
| v0.2.2 | 2026-04-16 | One-click installer, gain improvements |
| v0.2.1 | 2026-04-16 | Fix git status, enhance history output |
| v0.2.0 | 2026-04-15 | 50+ filters, Windows native, Claude Code hooks |
| v0.1.0 | 2026-04-14 | Initial release |

---

**WTK: Windows Token Killer** - Making LLMs affordable on Windows.
