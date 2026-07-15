# Subsystem — Sidecar Neo4j

**Source:** `swap/templates/database-cluster-compose.yml`, `gzmo-core/src/health.rs`  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Runs Neo4j 5 Community as the shared knowledge graph backend for MCP memory tools. Dream, spark, ingest, and session-distill engines write entities/relations via `KgPromoter`; KG reconcile canonicalizes ontology daily.

**Live (2026-07-14):** Container `sidecar-neo4j` up 6 days; **63,572** nodes, **64,224** relations; HTTP **7474**, Bolt **7687**.

---

## 2. How it works

Compose service (credentials via `NEO4J_AUTH` env — set in deployment secrets, not committed):

```28:46:swap/templates/database-cluster-compose.yml
  neo4j:
    image: neo4j:5-community
    container_name: sidecar-neo4j
    restart: always
    ports:
      - "7474:7474" # HTTP Console
      - "7687:7687" # Bolt Protocol
    environment:
      - NEO4J_AUTH=neo4j/<secret>
      - NEO4J_server_memory_heap_initial__size=1500m
      - NEO4J_server_memory_heap_max__size=1500m
      - NEO4J_server_memory_pagecache_size=1500m
      - NEO4J_dbms_security_procedures_unrestricted=apoc.*,gds.*
      - NEO4J_dbms_security_procedures_allowlist=apoc.*,gds.*
    volumes:
      - neo4j_data:/data
      - neo4j_logs:/logs
      - neo4j_import:/var/lib/neo4j/import
      - neo4j_plugins:/plugins
```

Memory budget: **~4.5 GiB** JVM (heap + pagecache) on 8 GiB CT101 — coexists with ~487 MiB daemon.

Bolt TCP probe at startup:

```86:103:gzmo-core/src/health.rs
pub fn probe_neo4j_bolt(bolt_url: &str) -> ProbeResult {
    let host_port = bolt_url
        .trim()
        .strip_prefix("bolt://")
        .or_else(|| bolt_url.strip_prefix("bolt+s://"))
        .unwrap_or(bolt_url);
    // TcpStream::connect_timeout 3s
}
```

MCP graph smoke test when memory server registered:

```217:235:gzmo-core/src/health.rs
pub async fn probe_mcp_memory(tools: &ToolRegistry) -> ProbeResult {
    if !tools.has_tool("mcp__memory__read_graph") {
        return ProbeResult::fail("mcp_memory", "mcp__memory__read_graph not registered");
    }
    // dispatch read_graph → pass/fail
}
```

---

## 3. Interfaces

| Interface | Value |
|-----------|-------|
| HTTP browser | `http://192.168.31.202:7474` |
| Bolt | `bolt://192.168.31.202:7687` |
| MCP package | `/opt/gzmo/mcp-neo4j-memory-gzmo/` |
| Config | `[mcp_servers]` entry `memory` with `NEO4J_URL`, credentials in `/opt/gzmo/.env` |
| Reconcile cron | `[kg_reconcile]` — default daily via daemon loop |

---

## 4. THINKING nodes

> **THINKING — database-cluster-compose.yml:neo4j memory**
> - *Reviewed:* 1500m heap + 1500m pagecache on 8 GiB LXC.
> - *Insight:* Tight but workable with sidecars + daemon; no headroom for large graph analytics.
> - *Risk / limitation:* OOM kill under bulk import or reconcile storms.
> - *Enhancement:* Dynamic heap sizing from container RAM; monitor with Prometheus. [CT101-safe]

> **THINKING — health.rs:bolt vs mcp probe**
> - *Reviewed:* TCP bolt check is cheap; read_graph is full MCP round-trip.
> - *Insight:* Two-layer validation — transport vs semantic graph access.
> - *Risk / limitation:* MCP server crash after startup passes bolt probe.
> - *Enhancement:* Periodic MCP health in heartbeat CheapChecks. [CT101-safe]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| KG reconcile | Daily ontology fix via `mcp__memory__*` tools |
| GZMO-next | Could migrate to Neo4j Aura or embedded graph — out of CT101 scope |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Move `NEO4J_AUTH` to `.env` file (not compose inline) | [CT101-safe] |
| 2 | Neo4j backup cron to `/opt/gzmo/backups/` | [CT101-safe] |
| 3 | Graph size alerts when nodes > threshold | [CT101-safe] |
| 4 | Neo4j cluster / read replicas | [GZMO-next] |
