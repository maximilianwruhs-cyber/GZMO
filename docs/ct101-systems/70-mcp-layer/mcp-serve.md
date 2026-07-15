# MCP Serve — Platform Memory Server

**System:** [70-mcp-layer](./SYSTEM.md)  
**Sources:** `gzmo-core/src/mcp/serve.rs`, `gzmo-cli/src/mcp_serve_cmd.rs`

---

## Capability

`gzmo mcp-serve` runs a **stdio MCP server** exposing GZMO platform memory to external clients (Pi agent, Cursor on workstation): honeypot RRF search, session scratch recall, operator profile, wiki search, and vault status. Uses `PlatformMemory` — same recall path as the daemon against **60k vault / 37k honeypot** on CT101 when pointed at production data dir.

---

## How it works

### Server tools

```57:137:gzmo-core/src/mcp/serve.rs
    #[tool(description = "Search GZMO honeypot/vault memory...")]
    async fn gzmo_memory_search(&self, Parameters(args): Parameters<SearchParams>) -> Result<CallToolResult, McpError> {
        self.platform.memory_search(&args.query, limit, true).await
    }

    async fn gzmo_memory_status(&self) -> Result<CallToolResult, McpError> { /* vault counts, scratch backend */ }
    async fn gzmo_memory_recall_pull(&self) -> Result<CallToolResult, McpError> { /* [RECALL] block */ }
    async fn gzmo_wiki_search(/* ... */) -> Result<CallToolResult, McpError> { /* WikiEngine */ }
    async fn gzmo_memory_profile(/* dynamic_only */) -> Result<CallToolResult, McpError> { /* profile cache */ }
```

### Entry point

```154:165:gzmo-core/src/mcp/serve.rs
pub async fn run_mcp_serve(config: &GzmoConfig) -> Result<()> {
    let platform = Arc::new(PlatformMemory::open(config, None).await?);
    let server = GzmoMemoryMcpServer::new(platform, config.wiki.clone());
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
}
```

CLI wrapper:

```7:9:gzmo-cli/src/mcp_serve_cmd.rs
pub async fn run(config: &GzmoConfig) -> Result<()> {
    run_mcp_serve(config).await
}
```

Search writes recall snippets to Redis scratch when `write_scratch=true`.

---

## Interfaces

| Kind | Detail |
|------|--------|
| CLI | `gzmo mcp-serve` (stdio — designed for MCP client subprocess) |
| Tools | `gzmo_memory_search`, `gzmo_memory_status`, `gzmo_memory_recall_pull`, `gzmo_memory_profile`, `gzmo_wiki_search` |
| Config | Full `GzmoConfig` — `[memory]`, `[embeddings]`, `[rerank]`, `[qdrant]`, `[redis]`, `[wiki]` |
| Deploy | Typically workstation Pi bridge; can run on CT101 with production `gzmo.toml` |
| Docs | [PI_GZMO_MEMORY_INTEGRATION.md](../../PI_GZMO_MEMORY_INTEGRATION.md) |

---

## THINKING nodes

> **THINKING — serve.rs:GzmoMemoryMcpServer**
> - *Reviewed:* rmcp `tool_router` macro; PlatformMemory + WikiEngine split.
> - *Insight:* Outbound MCP surface mirrors internal agent memory tools — Pi parity.
> - *Risk / limitation:* Each search opens full vault stack — heavy for rapid Pi polling.
> - *Enhancement:* Connection pooling / long-lived PlatformMemory singleton [CT101-safe].

> **THINKING — serve.rs:memory_search scratch write**
> - *Reviewed:* Search with write_scratch=true populates Redis [RECALL] for session.
> - *Insight:* Pi gets same scratch inject path as daemon agent loop.
> - *Risk / limitation:* Remote client must share session_id semantics for scratch scope.
> - *Enhancement:* Explicit session_id param on MCP tools [GZMO-next].

> **THINKING — mcp_serve_cmd.rs**
> - *Reviewed:* Thin CLI delegate — no extra flags.
> - *Insight:* Config file path determines which vault (CT101 vs data-next) is served.
> - *Risk / limitation:* Mis-pointed cwd serves wrong vault silently.
> - *Enhancement:* Log resolved vault_db path at MCP startup [CT101-safe].

---

## Advancement

- **CT101:** Run `mcp-serve` only when needed; primary ops stay in daemon.
- **GZMO-next:** HTTP/SSE transport option for MCP beyond stdio (lab).

---

## Enhancement backlog

1. **[CT101-safe]** Startup log: vault path, honeypot count, redis live/degraded.
2. **[CT101-safe]** Optional read-only mode flag (search/status only, no scratch write).
3. **[CT101-safe]** Rate limit on memory_search for Pi clients.
4. **[GZMO-next]** session_id parameter on search/recall_pull tools.
5. **[GZMO-next]** Streamed search results for large recall payloads.
