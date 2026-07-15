# Neo4j Memory Server — KG MCP Child

**System:** [70-mcp-layer](./SYSTEM.md)  
**Sources:** `gzmo.toml` `[[mcp_servers]]`, dream/ingest MCP call sites

---

## Capability

The **memory** MCP server is a uvx-spawned **`mcp-neo4j-memory`** process connecting to the Neo4j sidecar on CT101. It exposes graph CRUD tools (`create_entities`, `create_relations`, `read_graph`, etc.) prefixed as **`mcp__memory__*`**. Live graph: **~63k nodes**. Resurrects DreamEngine deep-phase KG writes and ingest relation promotion.

---

## How it works

### TOML spawn configuration

Production entry in `gzmo.toml` (paths on CT101):

```toml
[[mcp_servers]]
name = "memory"
command = "/home/gzmo/.local/bin/uvx"
args = ["--from", "/home/gzmo/github-clone/mcp-neo4j-memory-gzmo", "mcp-neo4j-memory"]

[mcp_servers.env]
NEO4J_URL = "bolt://192.168.31.202:7687"
NEO4J_USERNAME = "neo4j"
NEO4J_DATABASE = "neo4j"
# NEO4J_PASSWORD via env — do not commit secrets to docs
```

Comments in TOML require:
- Absolute `uvx` path (non-interactive daemon PATH lacks `~/.local/bin`)
- Server `name = "memory"` to match `dreams.rs` / boot `read_graph` tool names

### Daemon boot sequence

1. `McpManager::connect` spawns uvx child with Neo4j env
2. Handshake → discover tools → `register_all_tools`
3. Dream/ingest call `mcp__memory__create_entities` in batches (`KG_BATCH_SIZE = 20`)
4. `kg_promotion` sanitizes relation types before MCP payload

### Failure mode

MCP spawn failure (missing uvx, bolt down) → KG writes skip; **vault ingest continues**. Gate scripts may block deploy if MCP required for dream.

---

## Interfaces

| Kind | Detail |
|------|--------|
| Sidecar | Docker Neo4j on CT101 — bolt **7687**, HTTP 7474 |
| Package | Local path `/home/gzmo/github-clone/mcp-neo4j-memory-gzmo` via uvx `--from` |
| Prereq | `uv` installed; Neo4j sidecar healthy |
| Tool prefix | `mcp__memory__` — **do not rename** server without updating dreams.rs |
| Reconcile | `kg_reconcile` cron — graph ↔ vault alignment |
| Live scale | 63,572 nodes (2026-07-14 probe) |

---

## THINKING nodes

> **THINKING — gzmo.toml mcp_servers spawn**
> - *Reviewed:* uvx absolute path; bolt URL to LAN sidecar IP.
> - *Insight:* Child process model keeps Neo4j driver out of gzmo-core binary.
> - *Risk / limitation:* uvx fetch on first spawn adds boot latency; airgap needs prepull.
> - *Enhancement:* Pre-bake venv in CT101 image [CT101-safe].

> **THINKING — name = memory contract**
> - *Reviewed:* TOML comments tie prefix to dreams.rs string literals.
> - *Insight:* Fragile cross-file contract — rename breaks deep dream silently.
> - *Risk / limitation:* No compile-time check on tool name strings in dreams.
> - *Enhancement:* Shared constants or startup assert tool exists [GZMO-next].

> **THINKING — Neo4j vs vault authority**
> - *Reviewed:* Vault/honeypot authoritative; Neo4j is provenance graph mirror.
> - *Insight:* 63k nodes > 37k honeypot — includes entities/relations not in latest honeypot.
> - *Risk / limitation:* Drift until kg_reconcile runs.
> - *Enhancement:* Reconcile diff counts in health [CT101-safe].

---

## Advancement

- **CT101:** Move Neo4j password to `/opt/gzmo/.env` loaded by systemd — not inline TOML.
- **GZMO-next:** Optional in-process neo4rs driver behind same ToolHandler interface.

---

## Enhancement backlog

1. **[CT101-safe]** External env file for `NEO4J_PASSWORD` (systemd EnvironmentFile).
2. **[CT101-safe]** Boot self-test: `mcp__memory__read_graph` with limit 1.
3. **[CT101-safe]** kg_reconcile staleness metric vs 63k node baseline.
4. **[GZMO-next]** Compile-time or startup validation of expected MCP tool set.
5. **[GZMO-next]** Graph write batching coalesced in daemon vs per-truth MCP calls.
