# MCP Manager & Tool Bridge

**System:** [70-mcp-layer](./SYSTEM.md)  
**Sources:** `gzmo-core/src/mcp/manager.rs`, `gzmo-core/src/mcp/bridge.rs`

---

## Capability

`McpManager` spawns configured MCP servers as **child processes** (stdio transport), completes MCP handshake, paginates tool discovery, and registers each tool as a **`ToolHandler`** with prefixed names `mcp__{server}__{tool}`. `McpToolBridge` forwards agent tool calls to JSON-RPC `call_tool` on the live peer.

---

## How it works

### Connect and discover

```42:130:gzmo-core/src/mcp/manager.rs
    pub async fn connect(&mut self, config: McpServerConfig) -> Result<usize> {
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args);
        for (k, v) in &config.env { cmd.env(k, v); }
        let transport = TokioChildProcess::new(cmd)?;
        let client: McpClient = ().serve(transport).await?;
        let tools = peer.list_all_tools().await?;
        // bridges: prefixed_name = mcp__{name}__{sanitized_tool}
        self.servers.push(ConnectedServer { config, client, peer, bridges });
```

### Tool bridge execute

```51:104:gzmo-core/src/mcp/bridge.rs
    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let params = CallToolRequestParams { name: self.mcp_tool_name.clone().into(), arguments, /* ... */ };
        let result = self.peer.call_tool(params).await?;
        // extract text from content blocks; bail if is_error
    }
```

`McpServerConfig` holds `name`, `command`, `args`, `env` map — parsed from `[[mcp_servers]]` in gzmo.toml.

---

## Interfaces

| Kind | Detail |
|------|--------|
| Config | `[[mcp_servers]]` array in gzmo.toml |
| Tool naming | `mcp__memory__create_entities` (server name must stay `memory` for dreams.rs) |
| Lifecycle | `register_all_tools(&mut ToolRegistry)` at daemon boot; `shutdown()` on exit |
| Crate | `rmcp` — TokioChildProcess stdio transport |
| Consumers | Agent loop, DreamEngine deep phase, ingest KG batch |

---

## THINKING nodes

> **THINKING — manager.rs:connect**
> - *Reviewed:* Owned Command per server; env injection; list_all_tools pagination.
> - *Insight:* Each server is isolated process — crash doesn't take down daemon.
> - *Risk / limitation:* No automatic reconnect if child dies mid-run; tools stay registered but calls fail.
> - *Enhancement:* Watchdog reconnect with register refresh [CT101-safe].

> **THINKING — bridge.rs:execute**
> - *Reviewed:* Maps JSON args to MCP params; text extraction with structured fallback.
> - *Insight:* Agent sees unified ToolDef regardless of MCP origin.
> - *Risk / limitation:* Large KG responses may exceed agent context — no truncation here.
> - *Enhancement:* Truncate MCP tool output with hash pointer [CT101-safe].

> **THINKING — bridge.rs:prefixed_name**
> - *Reviewed:* `-` and `.` → `_` in tool names for registry compatibility.
> - *Insight:* Stable naming for dreams.rs string literals (`mcp__memory__*`).
> - *Risk / limitation:* Rename server in TOML breaks hardcoded dream paths.
> - *Enhancement:* Const for server name in shared config module [GZMO-next].

---

## Advancement

- **CT101:** Health probe: `McpManager::server_count()` + test `list_tools` latency.
- **GZMO-next:** Dynamic MCP server list from file watch.

---

## Enhancement backlog

1. **[CT101-safe]** MCP child restart on call_tool transport error.
2. **[CT101-safe]** Output size cap on bridge execute (64KB default).
3. **[CT101-safe]** Startup log: each discovered tool name + server version.
4. **[GZMO-next]** MCP server definitions in separate `mcp-servers.toml`.
5. **[GZMO-next]** Shared `MCP_SERVER_MEMORY_NAME` constant used by dreams + config.
