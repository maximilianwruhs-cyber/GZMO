# 03 — MCP attach and control-plane contracts

## Scope

Living **attach** contracts for writer vs client: process owner claim (flock + Unix socket), outbound `gzmo mcp-serve` tool surface, inbound `McpManager`/`McpToolBridge` children, `PlatformMemory` → RRF/utility path, brand labels (`gzmo-living` vs legacy `gzmo-memory` / `~/.gzmo`), and dual-writer refusal points (scripts vs core).

Non-goals (per ticket): deep recall algorithm internals beyond the attach→RRF/utility seam; side-store drift beyond naming inbound Neo4j child as optional.

## Contract inventory

### 1. Control-plane claim (flock, socket, owner roles)

**Doctrine (Observed, Doc-dated ADR-0006 accepted 2026-08-16):** one vault owner, two socket clients. Owners = `gzmo serve` / `gzmo daemon`; clients = CLI `gzmo memory *` and MCP. No HTTP; no MCP-as-control-plane. Protocol = one NDJSON request per connection. Host mutex (`living-host-mutex`) remains **host placement**; process ownership is the flock (`docs/ADR-0006-owner-control-plane.md`).

**Core claim path (Observed):**

| Symbol | Path | Role |
|--------|------|------|
| `VaultWriteLock::try_acquire` | `gzmo-core/src/control_plane/lock.rs` | Exclusive `fs2` flock on `{vault_db}.write.lock`; writes PID; second serve/daemon fails closed |
| `vault_write_lock_path` | same | Lock path is **separate** from SQLite WAL (comment L1–2) |
| `claim_owner` | `gzmo-core/src/control_plane/mod.rs` | `try_acquire` → `PlatformMemory::open_as_owner` → `spawn_server` → `OwnerClaim` |
| `OwnerClaim` | same | Holds lock + socket path + accept-loop join handle; `Drop` aborts server and removes socket file |
| `resolved_socket` / `ControlPlaneConfig::resolved_socket` | `control_plane/mod.rs`, `config.rs` | Default `{vault_db.parent()}/gzmo.sock` unless `control_plane.socket_path` set |
| `bind_socket` / `spawn_server` / `dispatch` | `control_plane/server.rs` | Unix listener; dispatches to process-local `PlatformMemory` |
| CLI owners | `gzmo-cli/src/serve_cmd.rs` L161, `daemon_cmd.rs` L96 | Both call `claim_owner` |

**Owner protocol methods (Observed, `server.rs` `dispatch`):** `ping`, `memory.search`, `memory.recall`, `memory.status`, `memory.turn_start`, `memory.chain`, `memory.profile`. Status forced `control_plane = "owner"` (`VIA_OWNER`).

**Client attach policy (Observed, `attach.rs` `attach_memory`):**

1. If `offline` and socket live → **refuse** (`refuse --offline while the socket is live`).
2. If `offline` and socket dead → `MemoryAttach::Local`.
3. If socket live → `MemoryAttach::Owner(ControlPlaneClient)`.
4. If socket dead and (`GZMO_CONTROL_PLANE` off **or** lite `~/.gzmo` **or** not `/opt/gzmo` living path) → `Local`.
5. Living vault (`is_living_vault`: path starts with `/opt/gzmo`) + dead socket + clients enabled → **hard-fail** (start daemon / `--offline` / `GZMO_CONTROL_PLANE=0`).

**Client switch (Observed):** `clients_enabled()` reads `GZMO_CONTROL_PLANE`; `0`/`false`/`off` disables socket prefer (`client.rs`). Comment: owner never honors this for the flock.

**Consumers of `attach_memory` (Observed):** `mcp/serve.rs` `run_mcp_serve`, `gzmo-cli` `memory_cmd.rs`, `chat.rs`.

**Host-level mutex (Observed, script, not flock):** `scripts/living-host-mutex.sh` claim/release/status for hosts `ct101|workstation|appliance`; writes `data-next/living-host/claim.json` (ADR-0005). Claiming `ct101` exits 3 if workstation `gzmo-serve` is active. This is **placement doctrine**, orthogonal to process flock.

### 2. Outbound MCP tool surface vs README

**Entry (Observed):** `gzmo mcp-serve` → `run_mcp_serve` (`mcp/serve.rs`). Attaches via `attach_memory(..., offline=false)` then either `MemoryFront::Owner` or in-process `PlatformMemory::open`. Session from `GZMO_SESSION_ID`. Stdio only (`server.serve(stdio())`).

**Product tools (Observed, `#[tool]` on `GzmoMemoryMcpServer`):**

| Tool | Behavior |
|------|----------|
| `gzmo_memory_turn_start` | Clear session scratch |
| `gzmo_memory_search` | Search; default `limit=5`, `write_scratch=true` |
| `gzmo_memory_status` | Vault path, counts, scratch backend JSON |
| `gzmo_memory_recall_pull` | `[RECALL]` scratch block |
| `gzmo_memory_chain` | Supersession chain by `fact_id` |
| `gzmo_memory_profile` | Cached operator profile (`dynamic_only`) |
| `gzmo_wiki_search` | Wiki markdown search if `[wiki].enabled`; else error |

**Ops gate (Observed):** `gzmo_ops_health`, `gzmo_discovery_status` call `ops_mcp_denied()` unless `GZMO_OPS_MCP=1` (or `true`). Denied message directs product tools only. `ServerHandler::get_info` instructions change with gate.

**README alignment (Observed):** `README.md` MCP tools table lists the seven product tools + ops gate note — matches `serve.rs`. Brand path names `gzmo-living` → status/search (`MCP_LOCAL_ATTACH.md`, README L34).

**Doctrine surface (Observed):** Brand attach = stdio client spawns `gzmo mcp-serve`; public HTTP/SSE MCP out of brand (`docs/MCP_LOCAL_ATTACH.md`, ADR-0004). Historical `~/.gzmo` + `gzmo-memory` documented as incomplete install (`docs/PRODUCT_MCP.md`).

### 3. Inbound manager / bridge — essential vs optional

**Inbound stack (Observed):**

- `McpManager` (`mcp/manager.rs`): spawns child MCP servers from config, handshake, `list_all_tools`, builds bridges, `register_all_tools`, `ensure_healthy` reconnect, `shutdown`.
- `McpToolBridge` (`mcp/bridge.rs`): registers as `ToolHandler` with prefixed name `mcp__{server}__{tool}` (`-`/`.` → `_`).
- Config: `[[mcp_servers]]` / `GzmoConfig.mcp_servers` / `active_mcp_servers()` (`config.rs`).

**Boot sites (Observed):** `daemon_cmd.rs` (dream MCP), `chat.rs`, `tui/runner.rs`, `cli_mcp.rs` `McpSession`, health path when a server named `memory` is active.

**Essential vs optional distinction (Observed + [INFERENCE]):**

| Surface | Essential to living **client attach**? | Notes |
|---------|----------------------------------------|-------|
| Outbound `gzmo mcp-serve` / label `gzmo-living` | **Yes** — brand attach contract | ADR-0004 / `MCP_LOCAL_ATTACH.md` |
| Owner control plane (`serve`/`daemon` flock+socket) | **Yes** for living vault under `/opt/gzmo` when clients enabled | `attach_memory` hard-fail |
| Inbound `[[mcp_servers]]` children via `McpManager` | **No** for external Cursor/Pi attach | Optional agent/daemon tools |
| Neo4j child named `memory` (`mcp__memory__*`) | **Optional** sidecar path for dreams/KG/health | `config/shared-mcp-memory.json` ships both `memory` (Neo4j uvx) and `gzmo-living`; health probes `mcp__memory__read_graph`; dreams/KG hardcode `mcp__memory__*` |

[INFERENCE]: External attach does not require Neo4j MCP child; living metabolism may use it when configured. Shared MCP JSON co-locates both for operator workstation topology, not as a single essential attach primitive.

### 4. `PlatformMemory` → RRF / utility / living min facts

**Open / floor (Observed, `platform_memory.rs`):**

- `LIVING_VAULT_MIN_FACTS = 100`.
- `open` / `open_inner(as_owner=false)` refuses vaults with `facts < 100` unless `allow_lab_or_product_vault` (`GZMO_ALLOW_LAB_VAULT`, `GZMO_PRODUCT`, or path under `~/.gzmo`).
- `open_as_owner` bypasses floor for bootstrap owners.
- Living proof comment: vault under `/opt/gzmo` (or airgap living home) + not-empty floor; lab/product/`~/.gzmo` bypass.

**Search path (Observed):**

```
gzmo_memory_search / control memory.search
  → PlatformMemory::memory_search[_scoped]
  → platform_cross_search (platform_search.rs)
  → memory_search_core → SqliteVault::search_recall
  → recall_rrf (vault.rs) → rrf_fuse → optional rerank → apply_utility_select
  → apply_utility_boost (recall_rrf.rs, UTILITY_POOL_LAMBDA = 0.05)
```

After hits: optional scratch write; `felt_use::touch_hits` with `Cited` if scratch written else `Glance` (Glance utility weight 0 — `felt_use.rs`).

**Cross-search (Observed):** Always runs vault RRF core; optionally merges Pi knowledge Qdrant collection when `platform_search.include_knowledge_collection` and qdrant+embeddings enabled.

**Status via marker (Observed):** Local status sets `control_plane: "in-process"`; owner dispatch overwrites to `"owner"`.

### 5. Brand labels

| Label | Role | Evidence |
|-------|------|----------|
| `gzmo-living` | **Brand** living attach | `MCP_LOCAL_ATTACH.md` table; `docs/examples/gzmo-living.mcp.json`; `config/shared-mcp-memory.json`; `emit-living-mcp-fragment.sh` emits only this label; README |
| `gzmo-memory` | **Legacy** scratch / product-incomplete | `config/product-mcp-memory.json`; `PRODUCT_MCP.md`; `install-product-mcp.sh` sets `GZMO_PRODUCT=1` + `GZMO_ALLOW_LAB_VAULT=1` |
| `~/.gzmo` | Incomplete install / lite vault path | README L22; `is_lite_vault`; product init path |
| `/opt/gzmo` | Living vault path test | `is_living_vault`; living hard-fail attach |
| Wrapper `pi-gzmo-mcp-serve.sh` | SSH stdio bridge to CT101 `mcp-serve` only | Never starts local `gzmo-serve`; refuses `GZMO_PRODUCT` / `GZMO_ALLOW_LAB_VAULT` |

**Attach checks (Observed):** `living-mcp-attach-check.sh` — soft HOLD if `gzmo-living` missing; FAIL if living mislabeled as `gzmo-memory`. `mcp-attach-check.sh` / product path for legacy.

### 6. Dual-writer refusal — scripts vs core

**Two layers (Observed):**

**A. Process ownership (core flock)** — mechanical same-vault mutex:

- `VaultWriteLock::try_acquire` fails if another serve/daemon holds lock.
- Living attach refuses second long-lived in-process open when owner socket is required/up (ADR-0006 intent: MCP no longer opens second vault handle when owner up).
- Does **not** by itself know “workstation vs CT101” host topology.

**B. Host / overnight dual-writer (scripts + doctrine)** — ADR-0003/0005:

| Script / path | Refusal behavior |
|---------------|------------------|
| `install-living-airgap.sh` | `die` if user `gzmo-serve` active |
| `airgap-overnight-soak.sh` | `REFUSE: gzmo-serve active` |
| `airgap-living-install-smoke.sh` | FAIL dual-writer row; checks installer refuse text |
| `living-host-mutex.sh` | claim ct101 fails if serve active; checklist stop writers elsewhere |
| `herdr-living-enqueue.sh` | `refused_dual_writer` if serve active; no `--now` |
| `openclaw-takeaway.sh` | wraps herdr enqueue; never list includes `gzmo_serve_start`, `session_close_--now` |
| `herdr-gzmo-metabolism/.../session-close.sh` | `--living` refuses `--now` |
| `core-crystallize.sh` | skips apply with `refused_dual_writer` |
| `brain-feed-check.sh`, `brain-intel-promote.sh`, `aos-ce-smoke.sh`, `ct101-living-probe.sh` | dual-writer FAIL/risk flags |
| `pi-gzmo-mcp-serve.sh` / `emit-living-mcp-fragment.sh` / `install-openclaw-living-attach.sh` | refuse PRODUCT/LAB-ALLOW on living bridge; emit never starts serve |

**Core does not** systemctl-check workstation `gzmo-serve` when opening MCP; that remains script/ops. Core **does** enforce one flock owner and living socket attach policy.

## Gaps and drift

1. **`docs/ct101-systems/70-mcp-layer/mcp-serve.md` is stale vs `serve.rs` (Observed):**
   - Documents only `PlatformMemory::open` startup; live code uses `attach_memory` Owner/Local.
   - Tool list omits `gzmo_memory_turn_start`, `gzmo_memory_chain`, and ops tools.
   - Still cites “60k vault / 37k honeypot” warehouse numbers; living floor in code is `LIVING_VAULT_MIN_FACTS = 100`.

2. **Two “memory” names (Observed):** inbound Neo4j MCP server key `memory` (`mcp__memory__*`) vs outbound brand `gzmo-living` / legacy `gzmo-memory`. Easy operator confusion; contracts differ (child KG tools vs platform attach).

3. **Host mutex is soft state file (Observed):** `living-host-mutex.sh` writes JSON under `data-next/`; not a distributed lock. Enforcement depends on scripts reading serve activity + operator checklist. Process flock only covers same-vault local owners.

4. **`mcp-serve.md` “Enhancement: long-lived PlatformMemory singleton”** partially superseded by owner socket path (ADR-0006 implemented).

5. **Wiki + ops tools are not on the control-plane NDJSON protocol (Observed):** wiki search and ops/discovery run only in MCP process local config/files; owner socket methods are memory-* only. [INFERENCE]: with Owner front, wiki still uses local `WikiConfig` in the MCP process (not via socket).

## Evidence status

| Area | Status |
|------|--------|
| control_plane flock/socket/attach | Observed in source |
| mcp serve tool surface + ops gate | Observed in source; README match Observed |
| inbound manager/bridge + Neo4j child | Observed config + manager/bridge + daemon/chat wiring |
| PlatformMemory → RRF/utility/min facts | Observed call chain in source |
| Brand labels | Observed docs + configs + scripts |
| Dual-writer scripts | Observed multiple scripts; host live serve state Unreachable from this session |
| Live CT101 socket/vault proof | Unreachable (research host is Windows workstation outside living box) |
| ADR-0010 clean-sheet onebox | Horizon context only — not treated as living attach replacement |

## Sources

- `gzmo-core/src/control_plane/{mod,lock,attach,client,server,protocol}.rs`
- `gzmo-core/src/mcp/{mod,serve,manager,bridge}.rs`
- `gzmo-core/src/platform_memory.rs`, `platform_search.rs`
- `gzmo-core/src/memory/{vault.rs,recall_rrf.rs,felt_use.rs}`
- `gzmo-core/src/config.rs` (`ControlPlaneConfig`, `mcp_servers`)
- `gzmo-cli/src/{serve_cmd,daemon_cmd,memory_cmd,chat,cli_mcp}.rs`
- `docs/ADR-0006-owner-control-plane.md`, `ADR-0003`, `ADR-0004`, `ADR-0005`, `ADR-0007`
- `docs/MCP_LOCAL_ATTACH.md`, `docs/PRODUCT_MCP.md`, `docs/ARCHITECTURE_GZMO_PLATFORM.md`
- `docs/ct101-systems/70-mcp-layer/{mcp-serve,mcp-manager-bridge}.md`
- `README.md` MCP tools section
- `config/{shared-mcp-memory,product-mcp-memory}.json`, `docs/examples/gzmo-living.mcp.json`
- `scripts/{pi-gzmo-mcp-serve,emit-living-mcp-fragment,living-mcp-attach-check,living-host-mutex,install-living-airgap,herdr-living-enqueue,openclaw-takeaway}.sh`
- `integrations/herdr-gzmo-metabolism/scripts/session-close.sh`
