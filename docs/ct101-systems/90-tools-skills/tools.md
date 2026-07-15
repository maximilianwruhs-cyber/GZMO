# Tools — Pluggable Agent Tool Registry

**Source:** `gzmo-core/src/tools/*.rs`, registration in `gzmo-cli/src/daemon_cmd.rs`  
**Parent:** [90-tools-skills/SYSTEM.md](./SYSTEM.md)

---

## Capability

Gives the LLM a typed, schema-documented toolkit: read/write filesystem, run allowlisted shell commands, search the web, inspect/kill processes, and read/write the native memory vault. On CT101, the daemon registers a **production subset** (no `web_read`, no `delegate_task`) plus all connected MCP tools.

---

## How it works

### Registry and dispatch

```45:73:github-clone/GZMO/gzmo-core/src/tools/mod.rs
pub struct ToolRegistry {
    handlers: HashMap<String, Box<dyn ToolHandler>>,
}
// ...
    pub async fn dispatch(&self, call: &ToolCall) -> ToolResult {
        match self.handlers.get(&call.function_name) {
            Some(handler) => match handler.execute(call.arguments.clone()).await {
                Ok(output) => ToolResult { call_id: call.id.clone(), success: true, output },
                Err(e) => ToolResult { call_id: call.id.clone(), success: false, output: format!("Tool error: {e}") },
            },
            None => ToolResult { call_id: call.id.clone(), success: false, output: format!("Unknown tool: {}", call.function_name) },
        }
    }
```

Each tool implements `ToolHandler`: `definition()` returns JSON-schema `ToolDef`; `execute()` runs async.

### CT101 daemon registration

```133:163:github-clone/GZMO/gzmo-cli/src/daemon_cmd.rs
    let mut dream_tools = ToolRegistry::new();
    dream_tools.register(Box::new(FileReadTool));
    dream_tools.register(Box::new(FileWriteTool));
    dream_tools.register(Box::new(DirListTool));
    dream_tools.register(Box::new(FileSearchTool));
    dream_tools.register(Box::new(ShellExecTool::default()));
    dream_tools.register(Box::new(WebSearchTool::default()));
    dream_tools.register(Box::new(SysMetricsTool));
    dream_tools.register(Box::new(SysKillTool));
    dream_tools.register(Box::new(MemoryRecordTool { vault: Arc::clone(&dream_vault) }));
    dream_tools.register(Box::new(MemorySearchTool::with_orchestrator_scratch(/* ... */));
    // MCP tools appended:
    dream_mcp.register_all_tools(&mut dream_tools);
```

### Module inventory

| Module | Tool name(s) | Behavior |
|--------|--------------|----------|
| `fs.rs` | `file_read`, `file_write`, `dir_list`, `file_search` | Async tokio FS; grep-backed search; 8k/5k truncation |
| `shell.rs` | `shell_exec` | Allowlist on first binary token; 30s timeout; stdout 6k cap |
| `web.rs` | `web_search` | SerpAPI if key, else DuckDuckGo HTML parse |
| `web_browse.rs` | `web_read` | HTTP fetch + HTML strip (chat/TUI only on workstation) |
| `sysadmin.rs` | `sys_metrics`, `sys_kill` | `sysinfo` telemetry; kill by PID (blocks self-PID) |
| `memory.rs` | `memory_record`, `memory_search` | Vault store/search; quarantine if confidence < 0.85 |
| `delegate.rs` | `delegate_task` | Spawns `SubagentRunner` (see [subagent-delegate.md](./subagent-delegate.md)) |

### Shell security model

```86:105:github-clone/GZMO/gzmo-core/src/tools/shell.rs
        let first_token = command
            .split_whitespace()
            .find(|t| !t.contains('='))
            .unwrap_or("");
        let binary_name = first_token.rsplit('/').next().unwrap_or(first_token);

        if !SAFE_COMMAND_PREFIXES.iter().any(|safe| binary_name == *safe) {
            tracing::warn!(command = %command, binary = %binary_name, "Blocked: not in allowlist");
            return Ok(format!(
                "ERROR: Command '{}' is not in the safe command allowlist. \
                // ...
            ));
        }
```

---

## Interfaces

| Interface | CT101 value |
|-----------|-------------|
| Config | No dedicated `[tools]` section — tools always registered at daemon boot |
| Vault path | `/opt/gzmo/data/vault.db` via `MemoryRecordTool` / `MemorySearchTool` |
| MCP | `[mcp.servers]` in `gzmo.toml` → `McpManager::register_all_tools` |
| SerpAPI | Optional via env/API keys in config (daemon uses `WebSearchTool::default()` = DDG) |
| Scratch scope | Orchestrator jobs: `MemorySearchTool::with_orchestrator_scratch` + per-step scope cell |

---

## THINKING nodes

> **THINKING — tools/mod.rs:ToolRegistry**
> - *Reviewed:* Central `HashMap` dispatch with success/error `ToolResult` envelope.
> - *Insight:* Unknown tools fail soft (success=false string) rather than panicking — agent loop can recover.
> - *Risk / limitation:* No per-tool rate limits or audit trail beyond tracing.
> - *Enhancement:* Emit Synapse `tool_call` events on dispatch. [GZMO-next]

> **THINKING — tools/shell.rs:SAFE_COMMAND_PREFIXES**
> - *Reviewed:* First-token allowlist blocks `rm`, `dd`, etc.; still runs full command via `/bin/sh -c`.
> - *Insight:* Piping (`ls | sh`) or argument injection past first token is possible if first token is allowed.
> - *Risk / limitation:* Host-mode execution on CT101 — comment notes Phase 3 Docker/gVisor not implemented.
> - *Enhancement:* Container-isolated shell or deny `|` / `;` in daemon mode. [GZMO-next]

> **THINKING — tools/memory.rs:confidence gate**
> - *Reviewed:* `memory_record` quarantines facts with confidence < 0.85.
> - *Insight:* Aligns with honeypot curation — low-confidence writes don't pollute recall layer immediately.
> - *Risk / limitation:* `file_write` has no analogous guard — LLM can still write arbitrary paths.
> - *Enhancement:* CT101 path jail for `file_write` under `/opt/gzmo/`. [CT101-safe]

> **THINKING — tools/web.rs:DDG fallback**
> - *Reviewed:* String-scrape HTML parser without full DOM dependency.
> - *Insight:* Zero API key path keeps CT101 cloud-mode research working when SerpAPI unset.
> - *Risk / limitation:* DDG HTML layout changes break parser silently (empty results).
> - *Enhancement:* Health probe for `web_search` in `gzmo health`. [CT101-safe]

> **THINKING — tools/sysadmin.rs:sys_kill**
> - *Reviewed:* PID equality check prevents killing own agent process.
> - *Insight:* Used by orchestrator `sys_janitor` for runaway child cleanup.
> - *Risk / limitation:* Can still kill unrelated user processes if LLM picks wrong PID.
> - *Enhancement:* Allowlist process names or cgroup scope on CT101. [CT101-safe]

---

## Advancement

| CT101 today | GZMO-next / lab |
|-------------|-----------------|
| Inline Rust tools in `gzmo-daemon` | Same registry in `gzmo chat`; lab pieces may expose CLI equivalents |
| MCP as primary extension point | Cursor-style MCP catalog; tool schemas generated from lab CLIs |
| `shell_exec` host allowlist | Replace with recipe subprocess boundaries in Little Tools Lab |

---

## Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Path jail for `file_write` / `file_read` on CT101 | [CT101-safe] |
| 2 | gVisor/Docker sandbox for `shell_exec` | [GZMO-next] |
| 3 | Tool dispatch Synapse events | [GZMO-next] |
| 4 | Wire SerpAPI on daemon `WebSearchTool` when key present | [CT101-safe] |
| 5 | Structured JSON output option for `sys_metrics` ingestion | [GZMO-next] |
