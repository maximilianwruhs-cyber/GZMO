# Subsystem — Agent Loop

**Source:** `gzmo-core/src/agent_loop.rs`  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

The core cognitive cycle: **prompt → LLM (streaming) → tool dispatch → result injection → repeat** until the model returns final text or hits `max_iterations`. Powers orchestrator jobs, watchers, and interactive REPL/TUI modes.

---

## 2. How it works

### Configuration

```24:47:gzmo-core/src/agent_loop.rs
pub struct AgentLoopConfig {
    pub max_iterations: usize,
    pub verbose_tool_output: bool,
    pub context: ContextConfig,
    pub on_chunk: Option<std::sync::Arc<dyn Fn(String) + Send + Sync>>,
    pub memory: Option<AgentMemoryContext>,
}

pub struct AgentMemoryContext {
    pub scratch: Arc<ScratchService>,
    pub session_id: String,
    pub scope: ScratchScope,
}
```

### Context window + distill enqueue

When messages are pruned, archived turns enqueue distill jobs:

```61:106:gzmo-core/src/agent_loop.rs
async fn build_windowed_messages(
    messages: &[Message],
    config: &ContextConfig,
    memory: Option<&AgentMemoryContext>,
) -> Result<Vec<Message>> {
    let prune = context::prune_with_archive(messages, config);
    if let Some(mem) = memory {
        if !prune.archived.is_empty() {
            let job = DistillJob { session_id, transcript, source };
            mem.scratch.enqueue_distill(job).await?;
        }
        if let Some(recall) = mem.scratch.format_for_inject(&mem.scope).await? {
            // insert recall as System message at index 1
        }
    }
    Ok(prune.windowed)
}
```

### Main loop

```149:284:gzmo-core/src/agent_loop.rs
    for iteration in 0..config.max_iterations {
        let windowed = build_windowed_messages(messages, &config.context, config.memory.as_ref()).await?;
        let response = gateway.complete_streaming(&windowed, &declarations, on_chunk).await?;
        match response {
            LlmResponse::Text(text) => {
                // clear scratch scope, return AgentResponse
            }
            LlmResponse::ToolCalls(calls) => {
                // push assistant tool_calls message
                for call in &calls {
                    let result = tools.dispatch(call).await;
                    messages.push(Message { role: Role::Tool, ... });
                }
            }
        }
    }
    // Safety valve: force final text response
```

### Safety valve

When max iterations hit, inject system message forcing answer without tools:

```287:318:gzmo-core/src/agent_loop.rs
    messages.push(Message {
        role: Role::System,
        content: "You have reached the maximum number of tool calls. Provide your final answer now...".to_string(),
        ...
    });
    let final_response = gateway.complete_streaming(&windowed, &[], on_chunk).await?;
```

Orchestrator calls via `run_step_inner` → `run_agent_loop` with scratch scope per job/step.

---

## 3. Interfaces

| Interface | Typical value |
|-----------|---------------|
| Entry point | `run_agent_loop(gateway, tools, messages, config)` |
| Max iterations | 5 (orchestrator), 10 (default), per-step override in pipeline |
| Streaming | SSE via `LlmGateway::complete_streaming` |
| Tool format | OpenAI-compatible `ToolDeclaration` |
| Context budget | `[context_memory]` → `ContextConfig` |

---

## 4. THINKING nodes

> **THINKING — agent_loop.rs:streaming dispatch**
> - *Reviewed:* Tool calls buffered during SSE stream; dispatched after stream ends.
> - *Insight:* User sees tokens in real-time; tool execution is post-stream batch.
> - *Risk / limitation:* Long streams delay first tool execution.
> - *Enhancement:* Early tool-call detection in stream parser. [GZMO-next]

> **THINKING — agent_loop.rs:archive distill**
> - *Reviewed:* Pruned messages enqueue distill jobs to Redis/file queue.
> - *Insight:* Background conversations feed session-distill without blocking loop.
> - *Risk / limitation:* High-volume orchestrator jobs could flood distill queue.
> - *Enhancement:* Rate limit distill enqueue per scope. [CT101-safe]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| TUI/REPL | Same loop with `on_chunk` callback for terminal UI |
| Subagents | `ScratchScope::Sub` for nested agent distill source |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Distill enqueue rate limiting | [CT101-safe] |
| 2 | Parallel tool dispatch where safe | [GZMO-next] |
| 3 | Token budget telemetry per iteration | [CT101-safe] |
| 4 | Cancellation token for long-running loops | [GZMO-next] |
