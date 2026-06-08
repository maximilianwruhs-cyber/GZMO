# GZMO Documentation Index

Canonical operator and architecture docs. Session reports and milestone logs belong in `docs/archive/` (gitignored).

## Start here

| Doc | Purpose |
|-----|---------|
| [../MACHINE.md](../MACHINE.md) | What GZMO is (two sentences) |
| [ROADMAP_TO_M5.md](ROADMAP_TO_M5.md) | Milestone roadmap |
| [INFRASTRUCTURE_OVERVIEW.md](INFRASTRUCTURE_OVERVIEW.md) | Stack topology, ports, runbook |
| [PRODUCTION_READINESS.md](PRODUCTION_READINESS.md) | Production gate checklist |

## Architecture

| Doc | Purpose |
|-----|---------|
| [ARCHITECTURE_GZMO_PLATFORM.md](ARCHITECTURE_GZMO_PLATFORM.md) | Platform spine, operator model |
| [MEMORY_ARCHITECTURE_SPEC.md](MEMORY_ARCHITECTURE_SPEC.md) | Vault, honeypot, recall tiers |
| [GZMO_SYSTEM_ARCHITECTURE_INGEST.md](GZMO_SYSTEM_ARCHITECTURE_INGEST.md) | IngestEngine pipeline |
| [WIKI_LAYER.md](WIKI_LAYER.md) | Git-tracked markdown wiki layer (see also `../WIKI.md`) |
| [CORE_MECHANICS_AUDIT_20260605.md](CORE_MECHANICS_AUDIT_20260605.md) | Core mechanics audit |
| [CEILING_ROADMAP.md](CEILING_ROADMAP.md) | Long-term ceiling |

## Chaos engine (ρ homeostasis)

| Doc | Purpose |
|-----|---------|
| [CHAOS_RHO_CONTROL_MODEL.md](CHAOS_RHO_CONTROL_MODEL.md) | **Canonical** engineering spec (shipped law) |
| [CHAOS_RHO_IMPLEMENTATION_HANDOFF.md](CHAOS_RHO_IMPLEMENTATION_HANDOFF.md) | **Step-by-step** handoff — verify, daemon, MASTER phases |
| [LIMIT_CYCLE_SPECS_MATH_MAP.md](LIMIT_CYCLE_SPECS_MATH_MAP.md) | Lore specs → math Rosetta |
| [CHAOS_RHO_HOMEOSTASIS_REVISION_REPORT.md](CHAOS_RHO_HOMEOSTASIS_REVISION_REPORT.md) | Audit + lab + port history |

## Operations

| Doc | Purpose |
|-----|---------|
| [PI_OPERATOR_GUIDE.md](PI_OPERATOR_GUIDE.md) | Pi agent onboarding |
| [REBOOT_STARTUP.md](REBOOT_STARTUP.md) | Cold-start after reboot |
| [EVAL_TIERS.md](EVAL_TIERS.md) | Eval tier definitions |

## Research (non-canonical)

Ad-hoc research notes live in `docs/research/` and may be archived over time.

## Archive

`docs/archive/` holds local session reports (`ANTIGRAVITY_*`, `M4_*`, etc.) and is **not tracked by git**.
