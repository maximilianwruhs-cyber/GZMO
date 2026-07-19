# GZMO Documentation Index

Canonical operator and architecture docs. Session reports and milestone logs belong in `docs/archive/` (gitignored).

## Start here

| Doc | Purpose |
|-----|---------|
| [PRODUCT_MCP.md](PRODUCT_MCP.md) | **Outsider product** — Cursor/Pi local memory MCP (`gzmo init`) |
| [PRODUCT_PRODUCTION_READINESS.md](PRODUCT_PRODUCTION_READINESS.md) | **Product GREEN gate** — laptop Memory MCP readiness |
| [LIVING_PRODUCTION_READINESS.md](LIVING_PRODUCTION_READINESS.md) | **Living GREEN gate** — CT101 metabolism readiness |
| [PRODUCTION_READINESS.md](PRODUCTION_READINESS.md) | Index — living vs product gates |
| [../README.md](../README.md) | Public product README (MCP-first) |
| [../MACHINE.md](../MACHINE.md) | What GZMO is (two sentences) |
| [ROADMAP_TO_M5.md](ROADMAP_TO_M5.md) | Milestone roadmap (operator) |
| [INFRASTRUCTURE_OVERVIEW.md](INFRASTRUCTURE_OVERVIEW.md) | Living stack topology, ports, runbook |
| [PRODUCTION_READINESS.md](PRODUCTION_READINESS.md) | Production gate checklist |

Product non-goals (v1): multi-host living install, overnight serve as required path, SEIP scaffolding, cloud memory SaaS.

## Architecture

| Doc | Purpose |
|-----|---------|
| [ARCHITECTURE_GZMO_PLATFORM.md](ARCHITECTURE_GZMO_PLATFORM.md) | Platform spine, operator model |
| [MEMORY_ARCHITECTURE_SPEC.md](MEMORY_ARCHITECTURE_SPEC.md) | Vault, honeypot, recall tiers |
| [GZMO_SYSTEM_ARCHITECTURE_INGEST.md](GZMO_SYSTEM_ARCHITECTURE_INGEST.md) | IngestEngine pipeline |
| [WIKI_LAYER.md](WIKI_LAYER.md) | Git-tracked markdown wiki layer (see also `../WIKI.md`) |
| [CORE_MECHANICS_AUDIT_20260605.md](CORE_MECHANICS_AUDIT_20260605.md) | Core mechanics audit |
| [CEILING_ROADMAP.md](CEILING_ROADMAP.md) | Long-term ceiling |
| [SPINE_FOCUS.md](SPINE_FOCUS.md) | **Active product focus** — two pillars, Keep/Park, vault owner |
| [STACK_OPPORTUNITY_MAP.md](STACK_OPPORTUNITY_MAP.md) | Portfolio map (Keep/Park/Later + nightburst spike inventory) |

## Chaos engine (ρ homeostasis)

| Doc | Purpose |
|-----|---------|
| [CHAOS_RHO_CONTROL_MODEL.md](CHAOS_RHO_CONTROL_MODEL.md) | **Canonical** engineering spec (shipped law) |
| [CHAOS_RHO_IMPLEMENTATION_HANDOFF.md](CHAOS_RHO_IMPLEMENTATION_HANDOFF.md) | Completed work inventory + verify tiers |
| [CHAOS_RHO_REMAINING_IMPLEMENTATION_HANDOFF.md](CHAOS_RHO_REMAINING_IMPLEMENTATION_HANDOFF.md) | **Agent brief** — remaining tasks (start here for new work) |
| [LIMIT_CYCLE_SPECS_MATH_MAP.md](LIMIT_CYCLE_SPECS_MATH_MAP.md) | Proposal lineage → equations + lab verdicts |
| [TOTO_GZMO_IMPACT_RESEARCH_BRIEF.md](TOTO_GZMO_IMPACT_RESEARCH_BRIEF.md) | Research protocol — Toto-2.0-4m impact evaluation |
| [TOTO_GZMO_IMPACT_RESEARCH_REPORT.md](TOTO_GZMO_IMPACT_RESEARCH_REPORT.md) | **Verdict: IMPACT NO** — Toto vs ρ baselines (2026-06-08) |

## GZMO-next + Little Tools Lab

| Doc | Purpose |
|-----|---------|
| [GZMO_NEXT_RUNBOOK.md](GZMO_NEXT_RUNBOOK.md) | Workstation next instance (scheduler + lab recipes) |
| [CT101_BOUNDARY.md](CT101_BOUNDARY.md) | Legacy vs next boundary (production cutover 2026-07-15) |
| [SHELL_SANDBOX_AND_DISCOVERY.md](SHELL_SANDBOX_AND_DISCOVERY.md) | Strict shell / Docker isolate / discovery→honeypot path |
| [STRETCH_ITEMS_HANDOFF.md](STRETCH_ITEMS_HANDOFF.md) | Agent brief — gVisor, discovery recipe, vault migrate, Observatory auth |
| [../../little-tools-lab/docs/ENHANCEMENT_AUDIT_2026-07.md](../../little-tools-lab/docs/ENHANCEMENT_AUDIT_2026-07.md) | Enhancement audit — critical notes + prioritized backlog |

## Operations

| Doc | Purpose |
|-----|---------|
| [PI_LIVING_STACK.md](PI_LIVING_STACK.md) | **Recovered** Pi × Redis × Headroom/CCR living topology |
| [PI_UPGRADE_RUNBOOK.md](PI_UPGRADE_RUNBOOK.md) | Pi upgrade checklist (stop attach breakages) |
| [HEADROOM_CCR.md](HEADROOM_CCR.md) | Headroom-inspired CCR on Redis (branch-only today) |
| [PI_OPERATOR_GUIDE.md](PI_OPERATOR_GUIDE.md) | Pi onboarding (historical; see living stack + upgrade runbook) |
| [PI_GZMO_MEMORY_INTEGRATION.md](PI_GZMO_MEMORY_INTEGRATION.md) | Living MCP attach (CT101) |
| [REBOOT_STARTUP.md](REBOOT_STARTUP.md) | Cold-start after reboot |
| [EVAL_TIERS.md](EVAL_TIERS.md) | Eval tier definitions |

## Research (non-canonical)

Ad-hoc research notes live in `docs/research/` and may be archived over time.

## Archive

`docs/archive/` holds local session reports (`ANTIGRAVITY_*`, `M4_*`, etc.) and is **not tracked by git**.
