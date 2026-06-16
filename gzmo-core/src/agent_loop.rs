//! # Agentic Loop
//!
//! The core cognitive cycle: prompt → LLM → tool dispatch → result injection → LLM.
//! Runs until the LLM produces a final text response with no tool calls,
//! or hits the maximum iteration limit (safety valve).
//!
//! Uses SSE streaming: text tokens appear in the terminal in real-time.
//! Tool calls are silently buffered and dispatched after the stream ends.

use std::io::Write;

use anyhow::Result;
use tracing::{debug, info, warn};

use std::sync::Arc;

use crate::context::{self, ContextConfig};
use crate::gateway::{LlmGateway, LlmResponse, ToolDeclaration};
use crate::memory::scratch::{messages_to_transcript, DistillJob, DistillSource, ScratchScope, ScratchService};
use crate::tools::{ToolDef, ToolRegistry, ToolResult};
use crate::types::{Message, MessageToolCall, MessageToolCallFunction, Role};


/// Configuration for the agentic loop.
pub struct AgentLoopConfig {
    /// Maximum tool-call iterations before forcing a text response.
    /// Prevents infinite loops if the LLM keeps requesting tools.
    pub max_iterations: usize,
    /// If true, log full tool results (can be verbose).
    pub verbose_tool_output: bool,
    /// Context window management configuration.
    /// Controls token budget and pruning behavior.
    pub context: ContextConfig,
    /// Optional callback for streaming tokens. When set, replaces the
    /// default spinner + stderr output so the TUI can intercept tokens.
    pub on_chunk: Option<std::sync::Arc<dyn Fn(String) + Send + Sync>>,
    /// Scratch cache + archive/distill hooks (optional).
    pub memory: Option<AgentMemoryContext>,
}

/// Per-loop scratch scope and distill session id.
#[derive(Clone)]
pub struct AgentMemoryContext {
    pub scratch: Arc<ScratchService>,
    pub session_id: String,
    pub scope: ScratchScope,
    pub compress_cfg: crate::config::ContextCompressConfig,
    pub ccr: crate::context_compress::CcrStore,
}

impl Default for AgentLoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            verbose_tool_output: false,
            context: ContextConfig::default(),
            on_chunk: None,
            memory: None,
        }
    }
}

async fn build_windowed_messages(
    messages: &[Message],
    config: &ContextConfig,
    memory: Option<&AgentMemoryContext>,
) -> Result<Vec<Message>> {
    let prune = context::prune_with_archive(messages, config);

    if let Some(mem) = memory {
        if !prune.archived.is_empty() {
            let transcript = messages_to_transcript(&prune.archived);
            let source = match &mem.scope {
                ScratchScope::Sub { task_id, .. } => DistillSource::SubArchive {
                    task_id: task_id.clone(),
                    role: "subagent".to_string(),
                },
                _ => DistillSource::MainArchive,
            };
            let job = DistillJob {
                session_id: mem.session_id.clone(),
                transcript,
                source,
            };
            if let Err(e) = mem.scratch.enqueue_distill(job).await {
                warn!(error = %e, "Failed to enqueue distill job");
            }
        }

        if let Some(recall) = mem.scratch.format_for_inject(&mem.scope, &mem.compress_cfg).await? {
            let mut windowed = prune.windowed;
            if !windowed.is_empty() {
                windowed.insert(
                    1,
                    Message {
                        role: Role::System,
                        content: recall,
                        is_meta: true,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                );
            }
            return Ok(windowed);
        }
    }

    Ok(prune.windowed)
}

/// The result of a complete agentic loop execution.
#[derive(Debug)]
pub struct AgentResponse {
    /// The final text response from the LLM.
    pub text: String,
    /// Total LLM calls made (including tool-call rounds).
    pub llm_calls: usize,
    /// All tool calls that were executed during the loop.
    pub tool_results: Vec<ToolResult>,
}

/// Convert our ToolDef into the LLM-compatible ToolDeclaration format.
fn to_declarations(defs: &[ToolDef]) -> Vec<ToolDeclaration> {
    defs.iter()
        .map(|d| ToolDeclaration {
            r#type: "function".to_string(),
            function: crate::gateway::ToolFunction {
                name: d.name.clone(),
                description: d.description.clone(),
                parameters: d.parameters.clone(),
            },
        })
        .collect()
}

/// Execute the full agentic loop.
///
/// Takes a conversation history, available tools, and the LLM gateway.
/// Runs the cycle until the LLM produces a text response or hits max iterations.
pub async fn run_agent_loop(
    gateway: &dyn LlmGateway,
    tools: &ToolRegistry,
    messages: &mut Vec<Message>,
    config: &AgentLoopConfig,
) -> Result<AgentResponse> {
    let tool_defs = tools.definitions();
    let declarations = to_declarations(&tool_defs);
    let mut total_calls = 0usize;
    let mut all_results = Vec::new();

    for iteration in 0..config.max_iterations {
        debug!(iteration, messages = messages.len(), "Agent loop iteration");

        // ─── Call LLM ────────────────────────────────────────────
        // Stream text tokens in real-time.
        // If a custom on_chunk is provided (TUI mode), use it directly.
        // Otherwise, use the default spinner + stderr output (REPL mode).
        let (on_chunk, spinner_cleanup): (Box<dyn Fn(String) + Send>, Option<(std::sync::Arc<std::sync::atomic::AtomicBool>, std::thread::JoinHandle<()>)>) =
            if let Some(ref callback) = config.on_chunk {
                let cb = callback.clone();
                (Box::new(move |text: String| { cb(text); }), None)
            } else {
                let spinning = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
                let spin_flag = spinning.clone();
                let spinner_handle = std::thread::spawn(move || {
                    const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                    let mut i = 0usize;
                    while spin_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        eprint!("\r  {} \x1b[2m⚙ cogitating...\x1b[0m", FRAMES[i % FRAMES.len()]);
                        let _ = std::io::stderr().flush();
                        std::thread::sleep(std::time::Duration::from_millis(80));
                        i += 1;
                    }
                });
                let spin_stop = spinning.clone();
                let cb = Box::new(move |text: String| {
                    if spin_stop.swap(false, std::sync::atomic::Ordering::Relaxed) {
                        eprint!("\r                              \r");
                    }
                    eprint!("{}", text);
                    let _ = std::io::stderr().flush();
                });
                (cb, Some((spinning, spinner_handle)))
            };

        let windowed = build_windowed_messages(messages, &config.context, config.memory.as_ref())
            .await?;

        let response = gateway
            .complete_streaming(&windowed, &declarations, on_chunk)
            .await;
        // Ensure spinner is dead (REPL mode only)
        if let Some((flag, handle)) = spinner_cleanup {
            flag.store(false, std::sync::atomic::Ordering::Relaxed);
            let _ = handle.join();
        }
        let response = response?;
        // Newline after streamed text (REPL mode only — TUI handles its own rendering)
        if config.on_chunk.is_none() && matches!(&response, LlmResponse::Text(_)) {
            eprintln!();
        }
        total_calls += 1;

        match response {
            // ─── Final text response — loop complete ─────────────
            LlmResponse::Text(text) => {
                if let Some(ref mem) = config.memory {
                    let _ = mem.scratch.clear(&mem.scope).await;
                }
                info!(
                    iterations = iteration + 1,
                    llm_calls = total_calls,
                    tools_executed = all_results.len(),
                    "Agent loop complete"
                );
                return Ok(AgentResponse {
                    text,
                    llm_calls: total_calls,
                    tool_results: all_results,
                });
            }

            // ─── Tool calls — dispatch and feed results back ─────
            LlmResponse::ToolCalls(calls) => {
                info!(
                    calls = calls.len(),
                    iteration,
                    "LLM requested tool calls"
                );

                // Add the assistant's tool-call message to history
                // with proper structured tool_calls (OpenAI-compatible format)
                let structured_calls: Vec<MessageToolCall> = calls
                    .iter()
                    .map(|c| MessageToolCall {
                        id: c.id.clone(),
                        r#type: "function".to_string(),
                        function: MessageToolCallFunction {
                            name: c.function_name.clone(),
                            arguments: serde_json::to_string(&c.arguments).unwrap_or_default(),
                        },
                    })
                    .collect();

                messages.push(Message {
                    role: Role::Assistant,
                    // Content can be empty for tool-call-only assistant messages
                    content: String::new(),
                    is_meta: true,
                    tool_calls: Some(structured_calls),
                    tool_call_id: None,
                });

                // Execute each tool call
                for call in &calls {
                    info!(
                        tool = %call.function_name,
                        call_id = %call.id,
                        "Dispatching tool"
                    );

                    let result = tools.dispatch(call).await;

                    if config.verbose_tool_output {
                        debug!(
                            tool = %call.function_name,
                            success = result.success,
                            output_len = result.output.len(),
                            "Tool result"
                        );
                    }

                    let output = if let Some(ref mem) = config.memory {
                        crate::context_compress::compress_for_context_with_ccr(
                            &result.output,
                            mem.compress_cfg.tool_output_max_tokens,
                            &mem.compress_cfg,
                            &mem.ccr,
                            &mem.session_id,
                            true,
                        )
                        .await
                        .text
                    } else {
                        result.output.clone()
                    };

                    // Inject tool result back into conversation
                    // with proper tool_call_id linking to the parent
                    messages.push(Message {
                        role: Role::Tool,
                        content: output,
                        is_meta: true,
                        tool_calls: None,
                        tool_call_id: Some(call.id.clone()),
                    });

                    all_results.push(result);
                }
            }
        }
    }

    // Safety valve: max iterations reached
    warn!(
        max = config.max_iterations,
        "Agent loop hit max iterations — forcing text response"
    );

    // Ask the LLM to summarize and stop. Must be Role::User — Qwen/llama.cpp chat
    // templates reject a trailing system message ("System message must be at the beginning").
    messages.push(Message {
        role: Role::User,
        content: "You have reached the maximum number of tool calls. Provide your final answer now based on the information gathered so far. Do not request any more tools.".to_string(),
        is_meta: false, tool_calls: None, tool_call_id: None,
    });

    let windowed = build_windowed_messages(messages, &config.context, config.memory.as_ref()).await?;
    let final_response = gateway.complete(&windowed, &[]).await?;
    total_calls += 1;

    let text = match final_response {
        LlmResponse::Text(t) => t,
        LlmResponse::ToolCalls(_) => {
            "I was unable to complete the task within the allowed number of tool calls.".to_string()
        }
    };

    if let Some(ref mem) = config.memory {
        let _ = mem.scratch.clear(&mem.scope).await;
    }

    Ok(AgentResponse {
        text,
        llm_calls: total_calls,
        tool_results: all_results,
    })
}
