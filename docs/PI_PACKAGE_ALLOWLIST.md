# Pi package allowlist (thin kit)

**Status:** Active (2026-07-22)  
**USP:** nutrient · Brain Feed · airgap living — not ecosystem tourism  
**Doctrine:** Pi = **thin remote**; living memory/work via **MCP** (`gzmo-living`); upgrades = [PI_UPGRADE_RUNBOOK.md](PI_UPGRADE_RUNBOOK.md) smoke  
**Related:** [PI_LIVING_STACK.md](PI_LIVING_STACK.md) · [EXTERNAL_LIVING_ATTACH.md](EXTERNAL_LIVING_ATTACH.md) · ADR-0003 (no dual-writer)

Pi’s store rewards collecting packages. Every package loads in-process. Fat → slow → weird breaks → “upgrade to fix” → fatter → dies.

**Rule:** allowlist, not wishlist. When it feels fat, amputate with `scripts/pi-thin-diet.sh`.

---

## Doctrine (do not confuse)

| Layer | Role |
|-------|------|
| **Pi packages** | Thin TUI / QoL / one MCP adapter / one gzmo-pi / one subagent pack |
| **MCP (`gzmo-living`)** | Living vault, search, recall — CT101 / airgap host |
| **GZMO CLI / daemon** | Metabolism, overnight writer (CT101 only; ADR-0003) |

Never install a second memory stack inside Pi. Never start workstation `gzmo-serve` to “make Pi better.”

---

## Core (must-have)

Install with `bash scripts/pi-thin-diet.sh --apply-core` (or manually).

| Spec | Why |
|------|-----|
| `npm:pi-mcp-adapter` | Reads `~/.pi/agent/mcp.json` — living/product MCP attach |
| **One** of `npm:gzmo-pi` **or** `git:github.com/maximilianwruhs-cyber/gzmo-pi` | GZMO Pi surface — **not both** |
| `npm:hsp-pi` *(optional)* | Audio / HSP bridge when you use it |
| `npm:pi-subagents` | **Keep one** subagent stack only |

Default when both gzmo-pi sources are present: **prefer git**, remove npm (`--prefer-gzmo-pi git|npm` to override).

---

## Recommended QoL (opt-in)

Install only what you will use. Default diet does **not** auto-install these.

```bash
bash scripts/pi-thin-diet.sh --apply-recommended --with spark,plan,ask,permissions,web,skillful
# optional extras:
bash scripts/pi-thin-diet.sh --apply-recommended --with lens,fff,plannotator,compact
```

| Flag / package | Spec | Notes |
|----------------|------|-------|
| `spark` | `npm:pi-spark` | TUI polish (unrelated to GZMO spark/ripen) |
| `plan` | `npm:pi-plan-mode` | Read-only explore + plan file |
| `ask` | `npm:@eko24ive/pi-ask` | Real `ask_user` clarification |
| `permissions` | `npm:@gotgenes/pi-permission-system` | Tool permission gates |
| `web` | **one of** `npm:pi-web-access` **or** `npm:@demigodmode/pi-web-agent` | Pick one; default `pi-web-access` (`--web-agent demigod`) |
| `skillful` | `npm:pi-skillful` | Better `/skill:` UX for large banks |
| `lens` | `npm:pi-lens` | Optional LSP / lint feedback |
| `fff` | `npm:@ff-labs/pi-fff` | Optional fuzzy file/content search |
| `plannotator` | `npm:@plannotator/pi-extension` | Optional plan/PR annotation UI |
| `compact` | `npm:pi-mega-compact` | Optional stronger compaction (situational) |

---

## Opt-in methodology (not default)

| Spec | Status |
|------|--------|
| `git:github.com/obra/superpowers` | **Opt-in only** — operator must explicitly request. Not core. Not recommended default. |

Superpowers is a strong gated build methodology (brainstorm → plan → TDD → subagent exec) with a first-class Pi package and session bootstrap. It conflicts with this kit’s defaults: thin remote, no auto-Socratic theater, ship-until-done. Matt process skills already cover grill/TDD/debug/review without bootstrap injection.

If you deliberately opt in: install the package, hide Matt process twins so two methodologies do not fight, set `SUPERPOWERS_DISABLE_TELEMETRY=1`, and put a ship-mode override in `system.md` (user ship/build instructions win). Treat that as a doctrine fork — not a casual add.

---

## Deny list (do not install; purge if present)

Competing memory, mega harnesses, duplicate subagent stacks, and toys. Living memory is MCP — not another Pi memory package.

| Category | Specs (safe names only) |
|----------|-------------------------|
| **Competing memory** | `npm:pi-memory`, `npm:@samfp/pi-memory`, `npm:pi-hermes-memory`, `npm:@mariozechner/pi-memory` |
| **Mega / duplicate harnesses** | `npm:pi-crew`, `npm:pi-workflow-engine`, `npm:pi-orchestrator`, `npm:pi-swarm` |
| **Duplicate subagents** | Anything claiming multi-agent teams beyond `npm:pi-subagents` (e.g. alternate `*-subagents` stacks) — keep **only** `pi-subagents` |
| **Toys / noise** | Store candy that is games or custom-UI demos as packages (prefer built-in examples if needed) |

```bash
bash scripts/pi-thin-diet.sh --purge-denied --dry-run
bash scripts/pi-thin-diet.sh --purge-denied
```

---

## Operator loop (after `pi update`)

```bash
bash scripts/pi-thin-diet.sh --check
bash scripts/install-shared-mcp.sh          # living mcp.json
bash scripts/living-attach-check.sh        # vault proof; never starts gzmo-serve
```

See [PI_UPGRADE_RUNBOOK.md](PI_UPGRADE_RUNBOOK.md).

---

## Anti-patterns

| Don’t | Do |
|-------|----|
| Install both npm + git `gzmo-pi` | Pick one; diet removes the duplicate |
| Install `pi-hermes-memory` / `pi-memory` for “recall” | Use `gzmo-living` MCP + `pi-gzmo-memory.sh` |
| Install `pi-web-access` **and** `@demigodmode/pi-web-agent` | Pick one web pack |
| Treat the store survey as a shopping list | Re-read this allowlist |
| Enable workstation `gzmo-serve` for Pi attach | ADR-0003 — CT101 owns overnight writer |
