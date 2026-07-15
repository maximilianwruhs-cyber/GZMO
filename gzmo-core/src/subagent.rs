//! SubagentRunner Lite — governed delegation with isolated scratch + context budgets.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::info;
use uuid::Uuid;

use crate::agent_loop::{run_agent_loop, AgentLoopConfig, AgentMemoryContext, AgentResponse};
use crate::config::SubagentConfig;
use crate::context::ContextConfig;
use crate::gateway::LlmGateway;
use crate::memory::scratch::{ScratchScope, ScratchService};
use crate::text_util::truncate_chars;
use crate::memory::vault::SqliteVault;
use crate::tools::fs::{DirListTool, FileReadTool, FileSearchTool, FileWriteTool};
use crate::tools::memory::{MemoryRecordTool, MemorySearchTool};
use crate::tools::shell::ShellExecTool;
use crate::tools::ToolRegistry;
use crate::types::{Message, Role};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentSpec {
    pub role: String,
    pub brief: String,
    pub max_iterations: usize,
    pub depth: u8,
    pub parent_session: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubStatus {
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentResult {
    pub task_id: String,
    pub status: SubStatus,
    pub summary: String,
    pub llm_calls: usize,
    pub tool_calls: usize,
}

struct RunningSub {
    task_id: String,
    handle: JoinHandle<()>,
}

pub struct SubagentRunner {
    config: SubagentConfig,
    scratch: Arc<ScratchService>,
    gateway: Arc<dyn LlmGateway>,
    tools: Arc<ToolRegistry>,
    system_prompt_base: String,
    registry: Arc<RwLock<HashMap<String, Vec<RunningSub>>>>,
    cancel_flags: Arc<Mutex<HashMap<String, bool>>>,
}

impl SubagentRunner {
    pub fn new(
        config: SubagentConfig,
        scratch: Arc<ScratchService>,
        gateway: Arc<dyn LlmGateway>,
        vault: Option<Arc<SqliteVault>>,
        system_prompt_base: String,
    ) -> Self {
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

        Self {
            config,
            scratch,
            gateway,
            tools: Arc::new(tools),
            system_prompt_base,
            registry: Arc::new(RwLock::new(HashMap::new())),
            cancel_flags: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn cancel_all(&self, session_id: &str) {
        {
            let mut flags = self.cancel_flags.lock().await;
            flags.insert(session_id.to_string(), true);
        }
        let mut reg = self.registry.write().await;
        if let Some(subs) = reg.remove(session_id) {
            for sub in subs {
                sub.handle.abort();
                let scope = ScratchScope::Sub {
                    session_id: session_id.to_string(),
                    task_id: sub.task_id,
                };
                let _ = self.scratch.clear(&scope).await;
            }
        }
        let mut flags = self.cancel_flags.lock().await;
        flags.remove(session_id);
        info!(session_id, "Cancelled all subagents for session");
    }

    fn role_system_prompt(&self, role: &str, brief: &str) -> String {
        format!(
            "{}\n\n---\nYou are a focused sub-agent (role: {role}). \
            Complete only the task in the user message. \
            Do not delegate further sub-tasks. \
            Your final reply must be a concise summary under {} tokens.\n\nTask:\n{brief}",
            self.system_prompt_base,
            self.config.summary_max_tokens,
        )
    }

    pub async fn spawn(&self, spec: SubagentSpec) -> Result<SubagentResult> {
        if !self.config.enabled {
            bail!("Subagents disabled in [subagent] config");
        }
        if spec.depth > self.config.max_depth {
            bail!("Subagent depth {} exceeds max {}", spec.depth, self.config.max_depth);
        }

        {
            let reg = self.registry.read().await;
            let count = reg.get(&spec.parent_session).map(|v| v.len()).unwrap_or(0);
            if count >= self.config.max_concurrent {
                bail!(
                    "Max concurrent subagents ({}) reached for session",
                    self.config.max_concurrent
                );
            }
        }

        let task_id = Uuid::new_v4().to_string();
        let scope = ScratchScope::Sub {
            session_id: spec.parent_session.clone(),
            task_id: task_id.clone(),
        };
        let _ = self.scratch.clear(&scope).await;

        let (tx, rx) = tokio::sync::oneshot::channel();
        let gateway = Arc::clone(&self.gateway);
        let tools = Arc::clone(&self.tools);
        let scratch = Arc::clone(&self.scratch);
        let session_id = spec.parent_session.clone();
        let role = spec.role.clone();
        let brief = spec.brief.clone();
        let max_iterations = spec.max_iterations.min(60);
        let summary_max = self.config.summary_max_tokens;
        let context_budget = self.config.context_budget_tokens;
        let system = self.role_system_prompt(&role, &brief);
        let cancel_flags = Arc::clone(&self.cancel_flags);
        let parent_session = spec.parent_session.clone();

        let task_id_spawn = task_id.clone();
        let task_id_for_reg = task_id.clone();
        let parent_for_reg = spec.parent_session.clone();

        let handle = tokio::spawn(async move {
            let result: Result<SubagentResult> = async {
                if cancel_flags
                    .lock()
                    .await
                    .get(&parent_session)
                    .copied()
                    .unwrap_or(false)
                {
                    return Ok(SubagentResult {
                        task_id: task_id.clone(),
                        status: SubStatus::Cancelled,
                        summary: "Subagent cancelled.".to_string(),
                        llm_calls: 0,
                        tool_calls: 0,
                    });
                }

                let mut messages = vec![
                    Message {
                        role: Role::System,
                        content: system,
                        is_meta: true,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    Message {
                        role: Role::User,
                        content: brief,
                        is_meta: false,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                ];

                let loop_config = AgentLoopConfig {
                    max_iterations,
                    verbose_tool_output: false,
                    context: ContextConfig::with_hot_budget(context_budget),
                    on_chunk: None,
                    memory: Some(AgentMemoryContext {
                        scratch: Arc::clone(&scratch),
                        session_id: session_id.clone(),
                        scope: scope.clone(),
                    }),
                };

                let response: AgentResponse =
                    run_agent_loop(gateway.as_ref(), tools.as_ref(), &mut messages, &loop_config)
                        .await?;

                let summary = truncate_chars(&response.text, summary_max * 4);

                Ok(SubagentResult {
                    task_id: task_id.clone(),
                    status: SubStatus::Done,
                    summary,
                    llm_calls: response.llm_calls,
                    tool_calls: response.tool_results.len(),
                })
            }
            .await;

            let _ = tx.send(result);
        });

        {
            let mut reg = self.registry.write().await;
            reg.entry(parent_for_reg.clone())
                .or_default()
                .push(RunningSub {
                    task_id: task_id_spawn,
                    handle,
                });
        }

        let result = match rx.await {
            Ok(r) => r?,
            Err(_) => bail!("Subagent result channel closed"),
        };

        {
            let mut reg = self.registry.write().await;
            if let Some(subs) = reg.get_mut(&spec.parent_session) {
                subs.retain(|s| s.task_id != task_id_for_reg);
                if subs.is_empty() {
                    reg.remove(&spec.parent_session);
                }
            }
        }

        Ok(result)
    }
}
