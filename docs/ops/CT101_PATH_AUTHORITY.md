# CT101 Path Authority

**Status:** Canonical living paths (2026-07-19)  
**Related:** [CT101_DEPLOY.md](./CT101_DEPLOY.md), [CT101_BOUNDARY.md](./CT101_BOUNDARY.md), [DISCOVERY_LIVING_WIRE.md](../DISCOVERY_LIVING_WIRE.md), [LOST_KNOWLEDGE_INVENTORY.md](../LOST_KNOWLEDGE_INVENTORY.md)

Three eras still appear in docs, Pi handoffs, and scripts. Only one is public living authority.

## Authority table

| Role | Canonical path | Notes |
|------|----------------|-------|
| Runtime root | `/opt/gzmo/` | systemd `WorkingDirectory` |
| Config | `/opt/gzmo/gzmo.toml` | Always set `GZMO_CONFIG` to this |
| Secrets | `/opt/gzmo/.env` | Never commit; never copy into `~/.pi/agent/` |
| Data / vault | `/opt/gzmo/data/` | `vault.db`, Synapse, distill-queue fallback, mentor sock |
| Mentor socket | `/opt/gzmo/data/gzmo_mentor.sock` | Living teach/ping |
| Release tree | `/opt/gzmo/current` | Symlink → checkout (may still resolve to `survey_GZMO` **directory name**) |
| Binary | `/opt/gzmo/current/target/release/gzmo` | Build **on CT101** (glibc) |
| Scripts | `/opt/gzmo/current/scripts/` | Prefer this over hardcoded `survey_GZMO` |
| Skills (product/next) | repo `skills/` via config | Workstation next uses `GZMO/skills/` |
| Skills (discovery/Pi bridge) | `gzmo_skills/` sibling or CT101 copy | See [SKILLS_BRIDGE.md](../SKILLS_BRIDGE.md) |

**Public name rule:** always say `/opt/gzmo/current`. The symlink target may be a directory literally named `survey_GZMO` — that is an implementation detail, not the SoT name.

## Forbidden / stale

| Path pattern | Why it bites |
|--------------|--------------|
| `~/Projects/_foundation-audit/survey_GZMO` | Old workstation home; pollutes CT101 when exported as `GZMO_ROOT` |
| Bare `survey_GZMO` without `/opt/gzmo/current` | Agents attach wrong tree → wrong vault / OpenRouter teach fallback |
| Workstation `data-next/` as living vault | Lab only — [ADR-0003](./adr/) / [CT101_BOUNDARY.md](./CT101_BOUNDARY.md) |
| Product `~/.gzmo` while expecting living Neo4j/Qdrant | Product MCP vs living attach collision — [PI_LIVING_STACK.md](./PI_LIVING_STACK.md) |

## Pi / discovery attach checklist

```bash
# Living
export GZMO_CONFIG=/opt/gzmo/gzmo.toml
export GZMO_ROOT=/opt/gzmo/current
# binary
/opt/gzmo/current/target/release/gzmo health
```

Discovery entry scripts on CT101 must **hardcode** `GZMO_ROOT=/opt/gzmo/current` (or accept `current` \| legacy symlink) — never inherit a workstation path. See [DISCOVERY_LIVING_WIRE.md](../DISCOVERY_LIVING_WIRE.md).

## Workstation clone (this repo)

| Role | Path |
|------|------|
| Dev checkout | `/home/gzmo/github-clone/GZMO` |
| Lab next config | `config/gzmo-next.toml` + `data-next/` (gitignored) |
| Deploy source | rsync → `ct101:/opt/gzmo/current/` then build there |

## Quick verify

```bash
ssh ct101 'readlink -f /opt/gzmo/current; test -f /opt/gzmo/gzmo.toml && echo CONFIG_OK'
ssh ct101 'GZMO_CONFIG=/opt/gzmo/gzmo.toml /opt/gzmo/current/target/release/gzmo health'
```
