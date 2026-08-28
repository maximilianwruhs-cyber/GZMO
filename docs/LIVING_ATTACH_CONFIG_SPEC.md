# Living attach config spec

**Status:** Implementable config contract (map [#151](https://github.com/maximilianwruhs-cyber/GZMO/issues/151), 2026-08-10)  
**USP:** nutrient · Brain Feed · airgap living — not ecosystem tourism  
**Operator pamphlet:** [EXTERNAL_LIVING_ATTACH.md](./EXTERNAL_LIVING_ATTACH.md) (happy path + never-do)  
**Doctrine:** [ADR-0003](./adr/ADR-0003-one-instance-metabolism.md) · [ADR-0004](./adr/ADR-0004-airgap-living-usp.md) · [ADR-0007](./adr/ADR-0007-one-product-living.md)

This document is the **source of truth for homes, install order, refuse conditions, and GREEN proofs**. Agents must not invent alternate trees (especially under `~/tmp/`) or treat `~/.hermes.toml` as Hermes config.

Living attach is **not** “any stdio `gzmo mcp-serve`.” It is: server name **`gzmo-living`** + living `GZMO_CONFIG` (or the official SSH wrapper) + vault proof.

---

## 1. Purpose

Future agents set up living attach **fail-closed** and **single-home** across:

| Surface | Role |
|---------|------|
| OpenClaw | Workstation Telegram operator workspace + takeaway |
| Hermes | Emit+paste living stanza into Hermes `config.yaml` |
| Cursor / Pi / global shared MCP | JSON merge homes via `install-shared-mcp.sh` |
| Airgap | **Alternate** topology (mutually exclusive with ops SSH as “the” attach) |

Per-surface scripts remain. This spec owns **order + refuse**; there is no mega-installer.

---

## 2. Homes table

| Surface | Authoritative home | Env / override | Notes |
|---------|-------------------|----------------|-------|
| OpenClaw workspace | `~/.openclaw/workspace` | `OPENCLAW_WORKSPACE` — **explicit operator override only**; agents must not invent overrides | Sync/install via `scripts/install-openclaw-living-attach.sh` / `scripts/sync-openclaw-workspace.sh` |
| Hermes MCP | `$HERMES_HOME/config.yaml` | `HERMES_HOME` (default `$HOME/.hermes`) | Key: `mcp_servers.gzmo-living`. **YAML only.** |
| Cursor MCP | `~/.cursor/mcp.json` | — | Merged by `scripts/install-shared-mcp.sh` |
| Pi MCP | `~/.pi/agent/mcp.json` | — | Same installer |
| Global shared MCP | `~/.config/mcp/mcp.json` | — | Same installer |
| Airgap living | `~/.gzmo-living/` (config + fragment per installer) | Living `GZMO_CONFIG` on-box | `scripts/install-living-airgap.sh` |

**Not homes (refuse / ignore as config):**

| Path | Why |
|------|-----|
| `~/tmp/openclaw-workspace` | Non-canonical scar; not a valid OpenClaw attach home |
| `~/.hermes.toml` | **Not** a Hermes load path (see research below) |
| `~/.pi/agent/mcp-cache.json`, Cursor approvals JSON, in-repo sample `mcp.json` trees | Caches / samples — not installer merge homes |
| `~/.gzmo/mcp.json` | Product restore fragment source for shared installer — **not** a living merge destination |

---

## 3. Ops SSH happy-path order (default topology)

Default living topology for the workstation operator LAN: **ops SSH** via `scripts/pi-gzmo-mcp-serve.sh` (remote `GZMO_CONFIG=/opt/gzmo/gzmo.toml`).

```bash
# 1) Prove living (fail-closed; read-only; never starts gzmo-serve)
bash scripts/living-attach-check.sh

# 2) Merge gzmo-living into known JSON client homes
bash scripts/install-shared-mcp.sh

# 3) OpenClaw living attach → ~/.openclaw/workspace only
bash scripts/install-openclaw-living-attach.sh
# Nutrient write (enqueue only): bash scripts/openclaw-takeaway.sh 'durable fact'

# 4) Hermes (if installed — else skip this chapter; N/A, not a global refuse)
bash scripts/emit-living-mcp-fragment.sh --format hermes
# Paste under mcp_servers.gzmo-living in $HERMES_HOME/config.yaml
# (default ~/.hermes/config.yaml). Emit+paste only — do not thrash Hermes home.

# 5) GREEN proofs (section 6)
```

Repo-owned dry-run examples: [docs/examples/hermes-gzmo-living.yaml](./examples/hermes-gzmo-living.yaml).

---

## 4. Airgap alternate (mutually exclusive)

When the agent runs **on the living host**, use the airgap USP path:

```bash
bash scripts/install-living-airgap.sh
# Prefer local living-attach-check mode / local GZMO_CONFIG as documented in AIRGAP_LIVING.md
```

Do **not** treat ops SSH OpenClaw/shared MCP as the same attach topology stacked on top of airgap. See [AIRGAP_LIVING.md](./AIRGAP_LIVING.md) and [ADR-0004](./adr/ADR-0004-airgap-living-usp.md).

---

## 5. Refuse matrix

| Refuse | Trigger |
|--------|---------|
| Wrong OpenClaw tree | Inventing `~/tmp/openclaw-workspace` or any home other than `~/.openclaw/workspace` without **explicit** `OPENCLAW_WORKSPACE` |
| Hermes TOML scar | Writing living stanza to `~/.hermes.toml` |
| Wrong MCP label | Claiming living with only `gzmo-memory` |
| PRODUCT / LAB-ALLOW | `GZMO_PRODUCT=1` or `GZMO_ALLOW_LAB_VAULT=1` on living path |
| Dual-writer | Workstation `gzmo-serve` active while CT101 owns overnight ([ADR-0003](./adr/ADR-0003-one-instance-metabolism.md)) |
| Missing vault proof | `living-attach-check.sh` fail / vault not living / fact floor |
| Hand-rolled SSH | `ssh … mcp-serve` without official wrapper / living `GZMO_CONFIG` |

**Hermes carve-out:** if Hermes is not installed (`hermes` missing / no Hermes home), the Hermes chapter is **N/A** — not a global kit refuse.

Guardrails already baked into scripts: `living-attach-check.sh`, `emit-living-mcp-fragment.sh`, `pi-gzmo-mcp-serve.sh` (see EXTERNAL pamphlet).

---

## 6. GREEN proof

Claim living attach success only when **all** of the following pass:

1. `bash scripts/living-attach-check.sh` exits 0  
2. `gzmo_memory_status` / CLI `memory status --json`: `vault_path` under living data (CT101 reference: `/opt/gzmo/data/vault.db`) and not-empty floor (`vault_facts` ≥ 100; CT101 denser Keep census ~800 vault / ~600 latest honeypot — **not** the old warehouse 10k/60k)  
3. Dual-writer inactive: workstation `gzmo-serve` **inactive** while CT101 owns overnight  
4. MCP server name **`gzmo-living`** present in each **configured** client (OpenClaw MCP registration; Cursor/Pi/global JSON homes that this host uses; Hermes `config.yaml` **if** Hermes chapter applies)

Telegram E2E round-trips are **ops soak**, not attach proof.

`"connection closed: initialized request"` after `timeout 5 … mcp-serve` is a **truncated handshake**, not living attach.

---

## 7. Scar quarantine

The spec **mandates** quarantine of these scars (physical delete is operator follow-on):

| Scar | Mandate |
|------|---------|
| `~/tmp/openclaw-workspace` | Not a valid attach home; do not sync/install there; delete/quarantine |
| `~/.hermes.toml` | Non-load-path; do not write living stanza there; delete/quarantine |
| `~/tmp/telegram-integration-handoff.md` and similar folklore | Do not treat as attach truth; prefer this spec + EXTERNAL |

---

## 8. Out of scope

- `/character` / OpenClaw character plugin integration  
- Telegram E2E as attach proof  
- Mega-installer or silent rewrites of Hermes home (`--apply` deferred)  
- Enabling workstation `gzmo-serve` / any second overnight writer  
- Public / WAN HTTP MCP as the living SKU  
- Claiming overnight soak GREEN from one attach probe  

---

## 9. Coexistence: `gzmo-living` + `gzmo-memory`

On ops boxes, **both** labels may appear in Cursor/global JSON homes (inventory 2026-08-10).

| Rule | Detail |
|------|--------|
| Allowed | `gzmo-living` present and GREEN proves living vault |
| Refuse | Claiming living attach while only `gzmo-memory` is configured / used |
| Legacy label | `gzmo-memory` on `~/.gzmo` is incomplete install / telescope scratch — not a second product ([ADR-0007](./adr/ADR-0007-one-product-living.md), [PRODUCT_MCP.md](./PRODUCT_MCP.md)) |

Pi may have living-only; absence of `gzmo-memory` there is fine.

---

## 10. Research pointers

| Topic | Note |
|-------|------|
| Hermes load path | [research/hermes-living-stanza-load-path-2026-08-10.md](../research/hermes-living-stanza-load-path-2026-08-10.md) — loads **only** `$HERMES_HOME/config.yaml`; `~/.hermes.toml` ignored |
| Shared MCP inventory | [research/shared-mcp-client-inventory-2026-08-10.md](../research/shared-mcp-client-inventory-2026-08-10.md) |
| Earlier attach scars | [research/living-external-attach-plug-and-play-2026-07-22.md](../research/living-external-attach-plug-and-play-2026-07-22.md) |

Wayfinder map: [#151 Living attach config — single-home fail-closed spec](https://github.com/maximilianwruhs-cyber/GZMO/issues/151).

---

## Related

- [EXTERNAL_LIVING_ATTACH.md](./EXTERNAL_LIVING_ATTACH.md) — operator pamphlet  
- [OPENCLAW_WORKSPACE_CONTRACT.md](./OPENCLAW_WORKSPACE_CONTRACT.md) — OpenClaw plane map  
- [AIRGAP_LIVING.md](./AIRGAP_LIVING.md) — on-box USP  
- [PI_GZMO_MEMORY_INTEGRATION.md](./ops/PI_GZMO_MEMORY_INTEGRATION.md) — ops SSH living  
- Skill: `skills/workflows/living-attach/SKILL.md`
