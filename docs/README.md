# GZMO Documentation Index

Canonical operator and architecture docs. Session reports and milestone logs belong in `docs/archive/` (gitignored).

## Start here

| Doc | Purpose |
|-----|---------|
| [ADR-0005-flywheel-over-frozen-topology.md](ADR-0005-flywheel-over-frozen-topology.md) | **Flywheel doctrine** — continuous upgrade outranks frozen topology |
| [CONTINUOUS_UPGRADE.md](CONTINUOUS_UPGRADE.md) | **Upgrade process** — four rings, promote-by-loop, craft backlog |
| [ADR-0004-airgap-living-usp.md](ADR-0004-airgap-living-usp.md) | **USP invariants** — airgap living on one box |
| [AIRGAP_LIVING.md](AIRGAP_LIVING.md) | **Hero path** — single-box airgap living bring-up |
| [KEEP_QUALITY.md](KEEP_QUALITY.md) | **USP quality gate** — `keep-quality-gate.sh` |
| [BRAIN_FEED.md](BRAIN_FEED.md) | **Active Unpark** — satellites that nourish the living vault |
| [OPPORTUNITY_DISCOVERY.md](OPPORTUNITY_DISCOVERY.md) | **What to build next** — Sense→Rank→Bet→Ship→Soak |
| [MCP_LOCAL_ATTACH.md](MCP_LOCAL_ATTACH.md) | **Brand MCP** — stdio / localhost only |
| [SPINE_FOCUS.md](SPINE_FOCUS.md) | Active focus — living first-class; lite bootstrap |
| [PRODUCT_MCP.md](PRODUCT_MCP.md) | **Lite bootstrap** — Cursor/Pi Memory MCP (`gzmo init`) |
| [PRODUCT_PRODUCTION_READINESS.md](PRODUCT_PRODUCTION_READINESS.md) | Lite GREEN gate |
| [LIVING_PRODUCTION_READINESS.md](LIVING_PRODUCTION_READINESS.md) | Living ops GREEN gate (CT101 reference) |
| [LIVING_APPLIANCE.md](LIVING_APPLIANCE.md) | Redis/Qdrant/Neo4j compose pin for living |
| [PRODUCTION_READINESS.md](PRODUCTION_READINESS.md) | Index — living vs lite gates |
| [../README.md](../README.md) | Public README |
| [../MACHINE.md](../MACHINE.md) | What GZMO is (two sentences) |
| [ROADMAP_TO_M5.md](ROADMAP_TO_M5.md) | Milestone roadmap (operator) |
| [INFRASTRUCTURE_OVERVIEW.md](INFRASTRUCTURE_OVERVIEW.md) | Living stack topology, ports, runbook |
| [PORTS.md](PORTS.md) | **Locked** port map (Redis wired; librarian retired) |
| [CT101_PATH_AUTHORITY.md](CT101_PATH_AUTHORITY.md) | Canonical `/opt/gzmo/*` paths (kill `survey_GZMO` drift) |
| [CT101_QDRANT_EMBED_OPS.md](CT101_QDRANT_EMBED_OPS.md) | Embed backfill + Qdrant orphan prune |
| [LOST_KNOWLEDGE_INVENTORY.md](LOST_KNOWLEDGE_INVENTORY.md) | Archaeology index — branch/plan scars recovered |
| [PANTHEON_SKILLS.md](PANTHEON_SKILLS.md) | Ritual/lab pantheon front door (dice/card/story research) |
| [PANTHEON_DEMO.md](PANTHEON_DEMO.md) | Pantheon ritual theater (dice/card/story felt; not Brain Feed) |
| [HSP_DEMO.md](HSP_DEMO.md) | HSP metabolism sonification theater (MIDI/WAV; not Brain Feed) |
| [MUTUAL_DISCOVERY_THEATER.md](MUTUAL_DISCOVERY_THEATER.md) | Pedagogy theater front door (not scout KPI) |
| [PRODUCTION_READINESS.md](PRODUCTION_READINESS.md) | Production gate checklist |

Product non-goals: public multi-tenant MCP webserver, dual overnight writers, SEIP scaffolding, cloud memory SaaS. Lite without overnight is bootstrap only — not a peer roadmap ([ADR-0004](ADR-0004-airgap-living-usp.md)).

## Architecture

| Doc | Purpose |
|-----|---------|
| [ARCHITECTURE_GZMO_PLATFORM.md](ARCHITECTURE_GZMO_PLATFORM.md) | Platform spine, operator model |
| [MEMORY_ARCHITECTURE_SPEC.md](MEMORY_ARCHITECTURE_SPEC.md) | Vault, honeypot, recall tiers |
| [GZMO_SYSTEM_ARCHITECTURE_INGEST.md](GZMO_SYSTEM_ARCHITECTURE_INGEST.md) | IngestEngine pipeline |
| [WIKI_LAYER.md](WIKI_LAYER.md) | Git-tracked markdown wiki layer (see also `../WIKI.md`) |
| [CORE_MECHANICS_AUDIT_20260605.md](CORE_MECHANICS_AUDIT_20260605.md) | Core mechanics audit |
| [CEILING_ROADMAP.md](CEILING_ROADMAP.md) | Long-term ceiling |
| [SPINE_FOCUS.md](SPINE_FOCUS.md) | **Active product focus** — airgap living USP, Unpark queue |
| [UNPARK_ROADMAP.md](UNPARK_ROADMAP.md) | Sequenced Unpark waves after keep-quality soak GREEN |
| [AOS_CUSTOMER_EDITION.md](AOS_CUSTOMER_EDITION.md) | Wave 4.1 CE sketch (on top of C only) |
| [OKCP_MARKETPLACE.md](OKCP_MARKETPLACE.md) | Wave 4.2 marketplace notes |
| [WIKI_OBSERVATORY_MIND.md](WIKI_OBSERVATORY_MIND.md) | Wave 4.3 demable mind notes |
| [PORTABLE_GZMO_CORE_RFC.md](PORTABLE_GZMO_CORE_RFC.md) | Wave 4.4 portable-core RFC |
| [STACK_OPPORTUNITY_MAP.md](STACK_OPPORTUNITY_MAP.md) | Portfolio map (Keep/Unpark/Later + nightburst spike inventory) |

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
| [SYNAPSE_EVENT_OWNERSHIP.md](SYNAPSE_EVENT_OWNERSHIP.md) | Pi vs daemon bus ownership (dedupe rules) |
| [DISTILL_COLD_CHAIN.md](DISTILL_COLD_CHAIN.md) | Distill ingress + SubArchive episodic skip |
| [SPAWN_GATE.md](SPAWN_GATE.md) | Autospawn + Redis Prime budget |
| [OBOLUS_GOVERNANCE.md](OBOLUS_GOVERNANCE.md) | T0–T2 energy gates |
| [DISCOVERY_KB_FEEDBACK_LOOP.md](DISCOVERY_KB_FEEDBACK_LOOP.md) | Discovery ↔ vault/honeypot loop |
| [DISCOVERY_LIFECYCLE.md](DISCOVERY_LIFECYCLE.md) | Scout vs implement (discover-only default) |
| [CT101_CLOUD_ROUTING.md](CT101_CLOUD_ROUTING.md) | OpenRouter / active_mode vs background |
| [AGENT_HOME_SECRETS.md](AGENT_HOME_SECRETS.md) | Never store passwords in `~/.pi/agent/` |
| [CHAOS_LIVING_VS_RITUAL.md](CHAOS_LIVING_VS_RITUAL.md) | Living chaos-free vs chat/TUI ritual |
| [PANTHEON_SKILLS.md](PANTHEON_SKILLS.md) | Pantheon packs — ritual/lab only |
| [MUTUAL_DISCOVERY_THEATER.md](MUTUAL_DISCOVERY_THEATER.md) | Mutual-discovery theater vs Forum-1 scout |
| [SKILLS_BRIDGE.md](SKILLS_BRIDGE.md) | Rust vs shell vs `gzmo_skills` |
| [HERDR_METABOLISM.md](HERDR_METABOLISM.md) | herdr takeaway → distill ritual (Park) |
| [OBOLUS_ARENA_BOUNDARY.md](OBOLUS_ARENA_BOUNDARY.md) | Daemon gate vs Obolus product vs Arena lab |
| [VM200_RETRIEVAL_BENCH.md](VM200_RETRIEVAL_BENCH.md) | Embed/rerank latency gates on VM200 |
| [PI_OPERATOR_GUIDE.md](PI_OPERATOR_GUIDE.md) | Pi onboarding (historical; see living stack + upgrade runbook) |
| [PI_GZMO_MEMORY_INTEGRATION.md](PI_GZMO_MEMORY_INTEGRATION.md) | Living MCP attach (CT101) |
| [REBOOT_STARTUP.md](REBOOT_STARTUP.md) | Cold-start after reboot |
| [EVAL_TIERS.md](EVAL_TIERS.md) | Eval tier definitions |

## Research (non-canonical)

Ad-hoc research notes live in `docs/research/` and may be archived over time.

| Doc | Purpose |
|-----|---------|
| [research/CT101_STACK_FUTURE_2026-07.md](research/CT101_STACK_FUTURE_2026-07.md) | **Futures** — MCP vs Pi vs compose appliance vs other (2026-07) |

| Path | Topic |
|------|-------|
| [research/pantheon/](research/pantheon/) | Dice / card / story / Dozen — archive contracts (front door: [PANTHEON_SKILLS.md](PANTHEON_SKILLS.md)) |
| [research/mutual-discovery/](research/mutual-discovery/) | Verified LINKs + Socratic modes (front door: [MUTUAL_DISCOVERY_THEATER.md](MUTUAL_DISCOVERY_THEATER.md)) |

## Archive

`docs/archive/` holds local session reports (`ANTIGRAVITY_*`, `M4_*`, etc.) and is **not tracked by git**.
