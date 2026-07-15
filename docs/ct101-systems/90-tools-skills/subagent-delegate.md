# Subagent Delegate — Governed Task Delegation

**Source:** `gzmo-core/src/subagent.rs`, `gzmo-core/src/tools/delegate.rs`  
**Parent:** [90-tools-skills/SYSTEM.md](./SYSTEM.md)

---

## Capability

Lets a parent agent spawn **focused child agents** with isolated scratch scope, bounded iterations, and summary-only return — avoiding full tool-log blow-up in the parent context. Roles: `reviewer`, `architect`, `developer`, `librarian` (prompt-driven).

**CT101:** Subagents are primarily used from workstation `gzmo chat` when `[subagent]` is enabled. The daemon's production `ToolRegistry` does not register `delegate_task` today.

---

## How it works

### Tool surface

```19:72:github-clone/GZMO/gzmo-core/src/tools/delegate.rs
impl ToolHandler for DelegateTaskTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "delegate_task".to_string(),
            description: "Delegate a focused sub-task to an isolated sub-agent. \
                Returns a short summary only (not full tool logs). \
                // ...
        }
    }
    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        // ...
        let spec = SubagentSpec {
            role: role.to_string(),
            brief: brief.to_string(),
            max_iterations,
            depth: self.depth.saturating_add(1),
            parent_session: self.session_id.clone(),
        };
        match self.runner.spawn(spec).await {
            Ok(result) => Ok(serde_json::to_string_pretty(&result)?),
            // ...
        }
    }
}
```

### SubagentRunner lifecycle

```67:95:github-clone/GZMO/gzmo-core/src/subagent.rs
impl SubagentRunner {
    pub fn new(/* config, scratch, gateway, vault, system_prompt */) -> Self {
        let mut tools = ToolRegistry::new();
        tools.register(Box::new(FileReadTool));
        tools.register(Box::new(FileWriteTool));
        tools.register(Box::new(DirListTool));
        tools.register(Box::new(FileSearchTool));
        tools.register(Box::new(ShellExecTool::default()));
        if let Some(ref v) = vault {
            tools.register(Box::new(MemoryRecordTool { vault: Arc::clone(v) }));
            tools.register(Box::new(MemorySearchTool::new(Arc::clone(v))));
        }
        // ...
    }
```

Spawn path (`spawn`):

1. Guard: `enabled`, `depth <= max_depth`, `concurrent < max_concurrent`
2. Allocate `ScratchScope::Sub { session_id, task_id }`
3. `tokio::spawn` → `run_agent_loop` with reduced tool set
4. Truncate response to `summary_max_tokens`
5. Cleanup registry entry; return `SubagentResult { status, summary, llm_calls, tool_calls }`

### Cancellation

```97:116:github-clone/GZMO/gzmo-core/src/subagent.rs
    pub async fn cancel_all(&self, session_id: &str) {
        // set cancel flag, abort JoinHandles, clear scratch scopes
    }
```

---

## Interfaces

| Interface | Default / CT101 |
|-----------|-----------------|
| `[subagent]` in `gzmo.toml` | `enabled=true`, `max_concurrent=2`, `max_depth=2` |
| `context_budget_tokens` | 32768 hot budget for sub-agent loop |
| `summary_max_tokens` | 800 (truncation via `truncate_chars`) |
| Scratch scope | `ScratchScope::Sub { session_id, task_id }` |
| Depth | Parent passes `depth`; tool increments by 1 — blocks re-delegation in role prompt |

Config struct:

```1363:1377:github-clone/GZMO/gzmo-core/src/config.rs
pub struct SubagentConfig {
    pub enabled: bool,
    pub max_concurrent: usize,
    pub max_depth: u8,
    pub context_budget_tokens: usize,
    pub summary_max_tokens: usize,
}
```

---

## THINKING nodes

> **THINKING — subagent.rs:tool subset**
> - *Reviewed:* Subagents get FS + shell + memory only — no web, MCP, or delegate.
> - *Insight:* Prevents runaway delegation trees and caps external API spend.
> - *Risk / limitation:* Reviewer role cannot `web_search` without parent pre-fetching context.
> - *Enhancement:* Role-based tool profiles (reviewer gets read-only, developer gets write). [GZMO-next]

> **THINKING — subagent.rs:spawn concurrency gate**
> - *Reviewed:* Per-session `max_concurrent` with `RunningSub` registry + abort on cancel.
> - *Insight:* Protects gateway Obolus budget from parallel sub-agent storms.
> - *Risk / limitation:* Global limit across sessions not enforced — only per `parent_session`.
> - *Enhancement:* Global subagent pool with priority queue. [GZMO-next]

> **THINKING — subagent.rs:role_system_prompt**
> - *Reviewed:* Injects "Do not delegate further sub-tasks" + summary token cap.
> - *Insight:* Hard prompt guard complements `max_depth` numeric guard.
> - *Risk / limitation:* LLM may still attempt `delegate_task` if tool were registered on child.
> - *Enhancement:* Omit `delegate_task` from child registry explicitly (already omitted). [CT101-safe]

> **THINKING — tools/delegate.rs:max_iterations cap**
> - *Reviewed:* `min(15)` on user-provided max_iterations; default 5.
> - *Insight:* Bounds worst-case LLM round-trips per delegation.
> - *Risk / limitation:* Complex review tasks may truncate before completion without parent retry.
> - *Enhancement:* Return partial status `SubStatus::Incomplete` with resume handle. [GZMO-next]

---

## Advancement

| CT101 | GZMO-next |
|-------|-----------|
| Chat-only delegation on workstation | Lab `kurator` spawn patterns for discovery fixer (separate pipeline) |
| Summary-only return | Structured JSON findings schema for orchestrator ingestion |
| Inline `SubagentRunner` in gzmo-core | Optional cloud subagent runtime via Cursor SDK |

---

## Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Role-based tool profiles for subagents | [GZMO-next] |
| 2 | Register `delegate_task` in daemon for orchestrator review steps | [CT101-safe] |
| 3 | Subagent Synapse events (`subagent_spawn`, `subagent_done`) | [GZMO-next] |
| 4 | Resume/cancel API exposed to operator UI | [GZMO-next] |
| 5 | Obolus preflight per `delegate_task` call | [CT101-safe] |
