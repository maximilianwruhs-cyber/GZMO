# Living external attach — plug-and-play for non-Cursor hosts

**Date:** 2026-07-22  
**Status:** Research / recommend (no implementation in this note)  
**USP lens:** nutrient · Brain Feed · airgap living — not ecosystem tourism  
**Doctrine:** [ADR-0003](../docs/ADR-0003-one-instance-metabolism.md) · [ADR-0004](../docs/ADR-0004-airgap-living-usp.md) · [ADR-0005](../docs/ADR-0005-flywheel-over-frozen-topology.md)

---

## 1. Problem statement (Hermes lesson)

Cursor/Pi reach **living** CT101 via one installer path. Hermes (and any host outside Cursor/Pi merge targets) did not.

| Surface | What happened (2026-07-22) |
|---------|----------------------------|
| **Cursor / Pi / global MCP** | `bash scripts/install-shared-mcp.sh` → server `gzmo-living` → `scripts/pi-gzmo-mcp-serve.sh` → `ssh ct101` + `GZMO_CONFIG=/opt/gzmo/gzmo.toml` + `gzmo mcp-serve` |
| **Hermes** | Stayed on label `gzmo-memory` → local `~/.gzmo/gzmo.toml` with `GZMO_PRODUCT=1` / `GZMO_ALLOW_LAB_VAULT=1` (~783 facts). Hand-rolled SSH recipes in Hermes skill docs omitted living config / used wrong remote env. Never called living tools. |

**Measured same day (CLI, not MCP soak claim):**

```text
living (CT101): vault_path=/opt/gzmo/data/vault.db  vault_facts≈61750  honeypot≈39835
lab   (~/.gzmo): vault_path=…/.gzmo/data/vault.db   vault_facts≈783
```

Hermes skill text also taught a false positive: `timeout 5 … mcp-serve` ending in `"connection closed: initialized request"` as “SUCCESS” — that is a truncated handshake, not living attach proof. And it claimed MCP needs `~/.gzmo/SOUL.md`; CLI explicitly exempts `McpServe` from identity load (`gzmo-cli/src/main.rs`).

**Lesson:** Living attach is **not** “any stdio MCP to any `gzmo mcp-serve`.” It is a **labeled contract** (config path + vault floor + server name + optional SSH wrapper). Cursor gets that for free; external hosts reinvent it badly.

---

## 2. Current attach surface (as-is)

### 2.1 Two profiles, two labels

| Label | Profile | Config | Installer | Overnight writer |
|-------|---------|--------|-----------|------------------|
| `gzmo-memory` | Lite bootstrap | `~/.gzmo/gzmo.toml` + `GZMO_PRODUCT=1` (+ usually `GZMO_ALLOW_LAB_VAULT=1`) | `scripts/install-product-mcp.sh` / `install-gzmo.sh` | **No** |
| `gzmo-living` | Living USP / ops | Living `GZMO_CONFIG` (CT101: `/opt/gzmo/gzmo.toml`) | On-box: fragment from `install-living-airgap.sh`; ops: `install-shared-mcp.sh` | Sole writer on claimed host ([ADR-0003](../docs/ADR-0003-one-instance-metabolism.md)) |

Brand contract: **stdio MCP** only — public HTTP MCP out ([MCP_LOCAL_ATTACH.md](../docs/MCP_LOCAL_ATTACH.md), [ADR-0004](../docs/ADR-0004-airgap-living-usp.md)).

### 2.2 Official paths today

```text
A) Airgap on-box (USP hero)
   install-living-airgap.sh
   → ~/.gzmo-living/gzmo.toml + mcp-living.fragment.json
   → local: gzmo mcp-serve  env GZMO_CONFIG=<living toml>
   Docs: AIRGAP_LIVING.md · LIVING_APPLIANCE.md

B) Ops remote (operator LAN — not brand USP story)
   install-shared-mcp.sh
   → merges Cursor + Pi + ~/.config/mcp/mcp.json
   → gzmo-living command = scripts/pi-gzmo-mcp-serve.sh
   → ssh $CT101_SSH_HOST → GZMO_CONFIG=/opt/gzmo/gzmo.toml  …/gzmo mcp-serve
   Docs: PI_GZMO_MEMORY_INTEGRATION.md · MCP_LOCAL_ATTACH.md

C) Lite stranger
   install-product-mcp.sh → gzmo-memory @ ~/.gzmo
   Docs: PRODUCT_MCP.md
```

### 2.3 Env / gates that matter

| Knob | Living | Lite / lab |
|------|--------|------------|
| `GZMO_CONFIG` | `/opt/gzmo/gzmo.toml` or appliance living home | `~/.gzmo/gzmo.toml` |
| `GZMO_PRODUCT` | **Must not** be `1` on living | `1` |
| `GZMO_ALLOW_LAB_VAULT` | **Must not** paper over wrong vault | Allows &lt;10k facts |
| `GZMO_LIVING` | Set by shared installer (`1`) — marker only today | Absent |
| `GZMO_OPS_MCP` | Optional ops tools (`gzmo_ops_health`, `gzmo_discovery_status`) | Stripped by product installer |
| Vault floor | `PlatformMemory::open` refuses &lt;10k facts unless lab/product/`~/.gzmo` (`LIVING_VAULT_MIN_FACTS`) | Small vaults OK |

Sources: `gzmo-core/src/platform_memory.rs`, `gzmo-core/src/mcp/serve.rs`, `scripts/install-*.sh`.

### 2.4 Health probe (attach truth)

```text
gzmo_memory_status  →  vault_path contains living data dir
                     →  vault_facts ~60k on CT101 reference
                     →  never ~/.gzmo when claiming living
```

Ops extras (gated): `GZMO_OPS_MCP=1` → `gzmo_ops_health` / `gzmo_discovery_status`.  
Checks already in-repo: `living-mcp-attach-check.sh`, `mcp-attach-check.sh`, `living-readiness-gate.sh` (label soft HOLD / mislabel FAIL).

### 2.5 Appliance / anybox / shared MCP (existing docs)

| Doc / script | Role |
|--------------|------|
| [LIVING_APPLIANCE.md](../docs/LIVING_APPLIANCE.md) | Sidecar pin + labeled MCP table |
| [AOS_CUSTOMER_EDITION.md](../docs/AOS_CUSTOMER_EDITION.md) | Sketch one-curl living CE (daemon + sidecars) — not external-agent attach |
| [AIRGAP_LIVING.md](../docs/AIRGAP_LIVING.md) | USP one-box topology |
| `living-host-mutex.sh` | Claims `ct101` \| `workstation` \| `appliance` |
| `config/shared-mcp-memory.json` | Fragment template (paths currently operator-machine-hardcoded) |
| Brain Feed | Nutrient into living vault — **no** attach installer of its own ([BRAIN_FEED.md](../docs/BRAIN_FEED.md)) |

No first-class “Hermes / Claude Code / generic agent” install target exists. Shared installer only knows three JSON homes: Cursor, Pi, global `~/.config/mcp/mcp.json`.

---

## 3. Gap analysis — Cursor/Pi free vs external reinvent

| Capability | Cursor / Pi get for free | External hosts reinvent |
|------------|--------------------------|-------------------------|
| Correct **label** (`gzmo-living` vs `gzmo-memory`) | `install-shared-mcp.sh` + mislabel migration | Manual YAML/JSON; Hermes kept lite name |
| Correct **remote env** (`GZMO_CONFIG=/opt/gzmo/gzmo.toml`) | Baked into `pi-gzmo-mcp-serve.sh` | Hand-rolled `ssh … gzmo mcp-serve` without config → wrong CWD / HOME scars |
| Merge into client config | Three known `mcp.json` paths | Hermes `config.yaml` (and others) never touched by GZMO scripts |
| Attach check | `living-mcp-attach-check.sh` | None for `~/.hermes` |
| Health semantics | Docs: status → ~60k under `/opt/gzmo` | Skill doc: “connection closed” = success |
| Ops tools | Optional `GZMO_OPS_MCP` on living fragment | Confused with product surface |
| Airgap on-box fragment | `~/.gzmo-living/mcp-living.fragment.json` | Must copy by hand into foreign config dialects |

**Root gap:** the productized “plug” is **Cursor/Pi-shaped**, not **stdio-MCP-host-shaped**. The binary contract (`gzmo mcp-serve` + `GZMO_CONFIG`) is already host-agnostic; the **installer + docs + checks** are not.

---

## 4. Footguns (do not ship around them)

1. **Name collision** — wiring living bridge under `gzmo-memory` (installer migrates this for Cursor/Pi; Hermes skill still says `gzmo:` / `gzmo-memory`).
2. **`GZMO_ALLOW_LAB_VAULT=1` on a “living” story** — silences the ≥10k refuse; Hermes lived happily on 783 facts.
3. **SSH without `GZMO_CONFIG=/opt/gzmo/gzmo.toml`** — remote binary inherits wrong HOME/config; CT101 scars from maximilian lab paths.
4. **Treating handshake stderr as attach proof** — `"connection closed: initialized request"` ≠ tools + vault.
5. **`GZMO_PRODUCT=1` on living** — `living-mcp-attach-check.sh` FAILs this; product installer sets it for lite only.
6. **Ops tools without `GZMO_OPS_MCP=1`** — clear deny string; not a connection failure.
7. **Public / WAN MCP** — rejected by ADR-0004; not a shortcut for Hermes.
8. **Second overnight writer** to “make attach easier” — forbidden (ADR-0003/0005). Attach is read/search; daemon stays sole writer.
9. **Hardcoded paths in `shared-mcp-memory.json`** — fragment assumes this workstation’s clone layout; strangers need generated paths.
10. **SOUL.md folklore** — MCP serve does not require it; do not gate external attach on persona files.

---

## 5. Recommended plug-and-play contract (minimal)

One sentence: **Any MCP host that can spawn a stdio command can attach living by running one wrapper with one env map, then proving `gzmo_memory_status` against the living vault.**

### 5.1 Config stanza (canonical)

**On living box (USP):**

```json
{
  "mcpServers": {
    "gzmo-living": {
      "command": "/usr/local/bin/gzmo",
      "args": ["mcp-serve"],
      "env": { "GZMO_CONFIG": "/opt/gzmo/gzmo.toml" }
    }
  }
}
```

**Ops SSH (operator LAN only):**

```json
{
  "mcpServers": {
    "gzmo-living": {
      "command": "/path/to/GZMO/scripts/pi-gzmo-mcp-serve.sh",
      "args": [],
      "env": { "GZMO_LIVING": "1" }
    }
  }
}
```

(Hermes YAML equivalent: same command/args/env under `mcp_servers.gzmo-living` — **not** under `gzmo-memory`.)

### 5.2 Env contract

| Required | Forbidden on living claim |
|----------|---------------------------|
| `GZMO_CONFIG` → living toml (on-box) **or** wrapper that sets it remotely | `GZMO_PRODUCT=1` |
| Absolute path to trusted `gzmo` / wrapper | `GZMO_ALLOW_LAB_VAULT=1` “to make it work” |
| Optional: `GZMO_OPS_MCP=1` for ops probes | Pointing lite label at living SSH |
| Optional: `CT101_SSH_HOST` / `CT101_GZMO_BIN` overrides | WAN-exposed MCP port |

### 5.3 Health checks (pass/fail)

1. Process starts; MCP tools list includes `gzmo_memory_status` / `gzmo_memory_search`.
2. `gzmo_memory_status`: `vault_path` under living data dir (CT101: `/opt/gzmo/data/vault.db`); `vault_facts` ≥ 10k (reference ~60k).
3. Soft: `honeypot_latest` non-trivial; ops health only if `GZMO_OPS_MCP=1`.
4. FAIL if path is `~/.gzmo` while operator believes they are on living.

CLI equivalent (no MCP):  
`ssh ct101 'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml …/gzmo memory status --json'`

### 5.4 Forbidden paths

- Dual overnight writer / enabling workstation `gzmo serve` while CT101 claimed  
- Public multi-tenant MCP URL as product SKU  
- Replacing Brain Feed / Keep quality with “more agent ecosystems”  
- Overnight LoRA / Arena DNA into `gzmo-daemon` by default  
- Claiming soak GREEN from a single attach probe  

### 5.5 Airgap-honest constraint

Preferred USP: agent **on the living box**, local stdio, no SSH. SSH bridge is **ops topology** ([MCP_LOCAL_ATTACH.md](../docs/MCP_LOCAL_ATTACH.md)) — document it as such so external hosts do not treat LAN SSH as the brand.

---

## 6. Phased bets (smallest first)

Research recommends; do **not** implement the whole ladder in one PR.

| Phase | Bet | Why smallest / brain profit |
|-------|-----|-----------------------------|
| **P0** | Doc: `docs/EXTERNAL_LIVING_ATTACH.md` (or section in MCP_LOCAL_ATTACH) + Hermes YAML example pointing at **existing** wrapper; fix folklore (SOUL / connection-closed) | Unblocks Hermes without new code; nutrient to living facts |
| **P1** | `scripts/emit-living-mcp-fragment.sh` — print host-agnostic JSON **and** Hermes YAML from env (`GZMO_BIN`, `GZMO_CONFIG` or `GZMO_ATTACH_MODE=ssh`) | One generator; no Cursor-only merge |
| **P2** | Rename/alias wrapper `pi-gzmo-mcp-serve.sh` → `gzmo-living-mcp-serve.sh` (keep Pi name as symlink); document `CT101_*` overrides | Stops “Pi-only” reading of ops path |
| **P3** | `install-shared-mcp.sh --target=cursor\|pi\|global\|hermes\|stdout` | First-class Hermes merge without inventing HTTP MCP |
| **P4** | Extend `living-mcp-attach-check.sh` to optional `HERMES_MCP_CONFIG` / generic path list | Catch lab-vs-living before agents thrash |
| **P5** | Tiny `gzmo living attach-smoke` (or script) that spawns mcp-serve, calls status, asserts path+facts ≥10k | Replaces “timeout 5 connection closed” folklore |

Stop before: multi-tenant gateway, second writer, Mem0-compatible cloud SKU, rewriting foreign agent frameworks.

---

## 7. Explicit non-goals

- Second overnight writer / dual-write vault  
- Claiming overnight soaks from attach work  
- Public HTTP/SSE MCP as default living attach  
- Overnight LoRA / Arena as required attach dependency  
- Replacing lite `gzmo-memory` stranger path (bootstrap stays)  
- Editing plan files / Unpark theater as substitute for living nutrient  
- Ecosystem tourism (support every agent brand before P0–P2 land)

---

## 8. Sources (primary)

| Evidence | Path |
|----------|------|
| Brand attach contract | `docs/MCP_LOCAL_ATTACH.md` |
| Ops SSH living | `docs/PI_GZMO_MEMORY_INTEGRATION.md`, `scripts/pi-gzmo-mcp-serve.sh` |
| Shared installer | `scripts/install-shared-mcp.sh`, `config/shared-mcp-memory.json` |
| Airgap on-box fragment | `scripts/install-living-airgap.sh`, `docs/AIRGAP_LIVING.md` |
| Lite vs living | `docs/PRODUCT_MCP.md`, `docs/SPINE_FOCUS.md`, `docs/LIVING_APPLIANCE.md` |
| Vault floor | `gzmo-core/src/platform_memory.rs` (`LIVING_VAULT_MIN_FACTS`) |
| Ops gate | `gzmo-core/src/mcp/serve.rs` (`GZMO_OPS_MCP`) |
| MCP serve depth | `docs/ct101-systems/70-mcp-layer/mcp-serve.md` |
| One writer / USP | `docs/ADR-0003-*.md`, `ADR-0004-*.md`, `ADR-0005-*.md` |
| Appliance mutex | `scripts/living-host-mutex.sh`, `docs/AOS_CUSTOMER_EDITION.md` |
| Hermes failure (ops scar) | Hermes `~/.hermes/config.yaml` still `gzmo-memory`@`~/.gzmo`; skill `references/gzmo-mcp.md` hand-rolled SSH |

---

## 9. Operator next (if acting)

```bash
# Prove living (ops) — fail-closed agent kit (P0/P1 landed):
bash scripts/living-attach-check.sh
bash scripts/emit-living-mcp-fragment.sh --format hermes
# Docs: docs/EXTERNAL_LIVING_ATTACH.md · docs/examples/

# Cursor path already:
bash scripts/install-shared-mcp.sh
bash scripts/living-mcp-attach-check.sh

# Hermes: paste emitted gzmo-living stanza (do not keep only gzmo-memory).
# Do not thrash ~/.hermes from scripts — emit to stdout / docs/examples only.
```
