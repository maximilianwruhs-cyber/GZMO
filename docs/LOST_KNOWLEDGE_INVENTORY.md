# Lost Knowledge Inventory

**Recovered:** 2026-07-19  
**Trigger:** Operator knowledge lived in Pi attach, Cursor plans, and unmerged branches — never as durable `main` docs.

Already recovered earlier this session (do not re-litigate):

| Doc | Topic |
|-----|-------|
| [PI_LIVING_STACK.md](./PI_LIVING_STACK.md) | Pi × Redis scratch × distill × Headroom/CCR |
| [PI_UPGRADE_RUNBOOK.md](./PI_UPGRADE_RUNBOOK.md) | Stop upgrade-break attach |
| [HEADROOM_CCR.md](./HEADROOM_CCR.md) | CCR on Redis (`feat/context-compress-headroom`) |

## Root cause

Almost all deep operator contracts were written on **`origin/feat/context-compress-headroom`** and never merged. `main` kept thin capability notes under `docs/ct101-systems/` while Cursor plans and `~/.pi/agent/` carried the living scars. Path eras (`survey_GZMO` vs `/opt/gzmo/current`) compounded the amnesia.

## Recovered onto `main` (this pass)

| Doc | Source |
|-----|--------|
| [PORTS.md](./PORTS.md) | Branch (locked port map; Redis **is** wired) |
| [SYNAPSE_EVENT_OWNERSHIP.md](./SYNAPSE_EVENT_OWNERSHIP.md) | Branch |
| [DISTILL_COLD_CHAIN.md](./DISTILL_COLD_CHAIN.md) | Branch |
| [DISCOVERY_KB_FEEDBACK_LOOP.md](./DISCOVERY_KB_FEEDBACK_LOOP.md) | Branch |
| [OBOLUS_GOVERNANCE.md](./OBOLUS_GOVERNANCE.md) | Branch |
| [OBOLUS_ENERGY.md](./OBOLUS_ENERGY.md) | Branch |
| [OBOLUS_EFFICIENCY.md](./OBOLUS_EFFICIENCY.md) | Branch |
| [OBOLUS_ROUTING.md](./OBOLUS_ROUTING.md) | Branch |
| [SPAWN_GATE.md](./SPAWN_GATE.md) | Branch |
| [FORUM_ROMANUM_SCHEMA.md](./FORUM_ROMANUM_SCHEMA.md) | Branch |
| [PEDAGOGY_LEARNING_BUS_SCHEMA.md](./PEDAGOGY_LEARNING_BUS_SCHEMA.md) | Branch |
| [SKILL_GOLDEN_STANDARD.md](./SKILL_GOLDEN_STANDARD.md) | Branch (path notes may still say `survey_GZMO`) |
| [DEFERRED_WORK_HANDOFF.md](./DEFERRED_WORK_HANDOFF.md) | Branch (TUI/daemon pedagogy parity backlog) |
| [CT101_PATH_AUTHORITY.md](./CT101_PATH_AUTHORITY.md) | **New** — single path table |
| [AGENT_HOME_SECRETS.md](./AGENT_HOME_SECRETS.md) | **New** — never store passwords in agent homes |
| [CHAOS_LIVING_VS_RITUAL.md](./CHAOS_LIVING_VS_RITUAL.md) | **New** — living park vs chat/TUI ritual |
| [SKILLS_BRIDGE.md](./SKILLS_BRIDGE.md) | **New** — Rust vs shell vs `gzmo_skills` |
| [CT101_CLOUD_ROUTING.md](./CT101_CLOUD_ROUTING.md) | **New** — OpenRouter / GLM / 402 ops |
| [HERDR_METABOLISM.md](./HERDR_METABOLISM.md) | **New** — herdr takeaway ritual (Park spike) |
| [CT101_QDRANT_EMBED_OPS.md](./CT101_QDRANT_EMBED_OPS.md) | **New** — embed backfill + orphan prune |
| [OBOLUS_ARENA_BOUNDARY.md](./OBOLUS_ARENA_BOUNDARY.md) | **New** — core gate vs product vs Arena lab |
| [VM200_RETRIEVAL_BENCH.md](./VM200_RETRIEVAL_BENCH.md) | Branch (paths updated) |
| [DISCOVERY_LIFECYCLE.md](./DISCOVERY_LIFECYCLE.md) | **New** — scout vs implement (from Cursor plan) |

Also fixed: Redis “not wired” lie in [INFRASTRUCTURE_OVERVIEW.md](./INFRASTRUCTURE_OVERVIEW.md).

## Still branch-only / plan-only (P2)

Cherry-pick or promote when needed; do not invent:

| Item | Where | Suggested action |
|------|-------|------------------|
| Card forge / dice / Würfel / pantheon packs | Branch `CARD_*`, `DICE_*`, `PANTHEON_*`, `WUERFEL_*` | **Parked packaging plan:** [PANTHEON_THEATER_PACKAGING_PARK.md](./PANTHEON_THEATER_PACKAGING_PARK.md) |
| Pi mutual-discovery session packs | Branch `PI_MUTUAL_*`, `PI_GUIDED_*` | Same park doc — promote only when unparked |
| ARCH-DIR-001 / zero-bloat reviews | Branch | Promote if sovereignty debate reopens |
| Cutover scar detail | Plans `*cutover*` | Covered enough by CT101_BOUNDARY / ADR-0003 |
| HSP redesign | Sibling HSP + plans | Leave in HSP repo |
| Little Tools Lab pieces | `little-tools-lab/docs/` | Index pointer only |
| IpW / Forge overnight spikes | scripts + plans | Stay Park — see [OBOLUS_ARENA_BOUNDARY.md](./OBOLUS_ARENA_BOUNDARY.md) |

Full branch-only list (49 files):  
`git diff --diff-filter=A --name-only main...origin/feat/context-compress-headroom -- docs/`

## Cursor plans worth keeping as scars

Under `~/.cursor/plans/` (not git):

- `discovery_probe-first_restore_*`, `restore_living_mentor_*` — wrong `GZMO_CONFIG` / path pollution
- `chaos_diagnosis_review_*` — `CHAOS_STATE.json` is write-only telemetry
- `ct101_glm_5.2_cloud_*`, `ct101_openrouter_verify_*` — living cloud cognition
- `lived_gzmo_cutover_*`, `workstation_gzmo_cutover_*` — dual-writer near-miss

## Ops actions (not docs)

1. **Rotate** Neo4j (and any SSH) credentials that appeared in `~/.pi/agent/MEMORY_REFERENCE.md` — see [AGENT_HOME_SECRETS.md](./AGENT_HOME_SECRETS.md).
2. Scrub agent-home files of plaintext passwords; keep only env-var names.
3. Prefer `/opt/gzmo/current` + `/opt/gzmo/gzmo.toml` in every living script (never inherit workstation `survey_GZMO` paths).
