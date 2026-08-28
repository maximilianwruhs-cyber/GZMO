# System 120 — Two-Stack Boundary

**Parent:** [CT101 Capability Index](../INDEX.md)  
**Authority:** [CT101_BOUNDARY.md](../../ops/CT101_BOUNDARY.md)  
**Infrastructure:** [CT101_INFRASTRUCTURE_REPORT.md](../../reports/CT101_INFRASTRUCTURE_REPORT.md) §12

---

## Role

Defines and enforces the **parallel development model**: CT101 runs frozen legacy `gzmo-daemon` (inline gzmo-core engines); GZMO-next on the workstation runs `gzmo-scheduler` (thin lab recipe runner). Little Tools Lab proves parity via **beat-gates** — it does **not** authorize per-loop graft onto CT101.

**Cutover:** Single migration when full lab assembly is S3 stack-ready — not loop-by-loop.

---

## Capability matrix

| Subsystem | Report | Function |
|-----------|--------|----------|
| **Assembly guard** | [assembly-guard.md](./assembly-guard.md) | `AssemblyConfig::effective()` forces Inline on non-`next` instances |
| **GZMO-next scheduler** | [gzmo-next-scheduler.md](./gzmo-next-scheduler.md) | Cron runner spawning lab recipes only |
| **Beat gates** | [beat-gates.md](./beat-gates.md) | Lab vs legacy comparison; emits `beat-meta.json` |

---

## Two-stack comparison

| | **CT101 (legacy)** | **GZMO-next (workstation)** |
|--|-------------------|----------------------------|
| Process | `gzmo-daemon` | `gzmo-scheduler` |
| Config | `/opt/gzmo/gzmo.toml` | `GZMO/config/gzmo-next.toml` |
| Data | `/opt/gzmo/data/` | `GZMO/data-next/` |
| `[assembly]` | All **Inline** (forced) | All **lab** (required) |
| Change policy | Frozen — legacy hotfixes | Active development |
| Env marker | `GZMO_INSTANCE` unset or `legacy` | `GZMO_INSTANCE=next` |

---

## Boundary enforcement flow

```mermaid
flowchart TB
  TOML["gzmo.toml assembly=lab?"]
  ENV["GZMO_INSTANCE=next?"]
  TOML --> Guard[assembly.rs effective()]
  ENV --> Guard
  Guard -->|legacy CT101| Inline[Force Inline engines]
  Guard -->|next| Lab[Run lab scripts]
  Lab --> Beat[beat-gate.sh compare]
  Beat --> Meta[beat-meta.json]
  Meta -.->|does NOT authorize| CT101
  Inline --> Daemon[gzmo-daemon cognition]
```

---

## Cross-dependencies

| System | Link |
|--------|------|
| [20-daemon-core](../20-daemon-core/SYSTEM.md) | `daemon_cmd.rs` resolves assembly backends at boot |
| [100-discovery-automation](../100-discovery-automation/SYSTEM.md) | Sidecar-only policy mirrors boundary intent |
| Little Tools Lab | Recipe source for scheduler + beat-gates |

---

## Consolidated enhancement backlog

| Rank | Enhancement | Tag |
|------|-------------|-----|
| 1 | CI beat-gate on all four loops (config, ops, cognition, knowledge) | [GZMO-next] |
| 2 | Document S3 cutover checklist in GZMO_NEXT_RUNBOOK | [GZMO-next] |
| 3 | Lint rule: fail if CT101 `gzmo.toml` contains `assembly = "lab"` | [CT101-safe] |
| 4 | Unified `gzmo instance status` command showing effective backends | [GZMO-next] |
| 5 | Automated diff report CT101 vs gzmo-next vault sizes pre-cutover | [GZMO-next] |

---

*Generated 2026-07-14 from `assembly.rs`, `gzmo-scheduler/`, `beat-gate.sh`, CT101_BOUNDARY.md.*
