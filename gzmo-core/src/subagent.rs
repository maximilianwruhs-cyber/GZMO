//! SubagentRunner Lite — governed delegation with isolated scratch + context budgets.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{info, warn};
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
use crate::tools::web::WebSearchTool;
use crate::tools::web_browse::WebBrowseTool;
use crate::tools::ToolRegistry;
use crate::types::{Message, Role};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentSpec {
    pub role: String,
    pub brief: String,
    pub max_iterations: usize,
    pub depth: u8,
    pub parent_session: String,
    #[serde(default)]
    pub working_dir: Option<String>,
    #[serde(default)]
    pub shell_extra_commands: Vec<String>,
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
    #[serde(default)]
    pub hit_max_iterations: bool,
    #[serde(default)]
    pub written_paths: Vec<String>,
}

struct RunningSub {
    task_id: String,
    handle: JoinHandle<()>,
}

pub struct SubagentRunner {
    config: SubagentConfig,
    compress_config: crate::config::ContextCompressConfig,
    ccr: crate::context_compress::CcrStore,
    scratch: Arc<ScratchService>,
    gateway: Arc<dyn LlmGateway>,
    tools: Arc<ToolRegistry>,
    vault: Option<Arc<SqliteVault>>,
    system_prompt_base: String,
    serpapi_key: String,
    registry: Arc<RwLock<HashMap<String, Vec<RunningSub>>>>,
    cancel_flags: Arc<Mutex<HashMap<String, bool>>>,
}

impl SubagentRunner {
    pub fn new(
        config: SubagentConfig,
        compress_config: crate::config::ContextCompressConfig,
        ccr: crate::context_compress::CcrStore,
        scratch: Arc<ScratchService>,
        gateway: Arc<dyn LlmGateway>,
        vault: Option<Arc<SqliteVault>>,
        system_prompt_base: String,
        serpapi_key: String,
    ) -> Self {
        let mut tools = ToolRegistry::new();
        tools.register(Box::new(FileReadTool::default()));
        tools.register(Box::new(FileWriteTool));
        tools.register(Box::new(DirListTool));
        tools.register(Box::new(FileSearchTool::default()));
        tools.register(Box::new(ShellExecTool::default()));
        if let Some(ref v) = vault {
            tools.register(Box::new(MemoryRecordTool { vault: Arc::clone(v) }));
            tools.register(Box::new(MemorySearchTool::new(Arc::clone(v))));
        }

        Self {
            config,
            compress_config,
            ccr,
            scratch,
            gateway,
            tools: Arc::new(tools),
            vault,
            system_prompt_base,
            serpapi_key,
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
        let mut prompt = format!(
            "{}\n\n---\nYou are a focused sub-agent (role: {role}). \
            Complete only the task in the user message. \
            Do not delegate further sub-tasks. \
            Your final reply must be a concise summary under {} tokens.\n\nTask:\n{brief}",
            self.system_prompt_base,
            self.config.summary_max_tokens,
        );
        if crate::discovery_code_implementer::is_discovery_agent_brief(brief) {
            prompt.push_str(
                "\n\nTool discipline: use the file_write tool for all file outputs. \
                Never embed <tool_call>, <function=file_write>, or fenced code blocks as a substitute for file_write. \
                Do not narrate tool calls — invoke them.",
            );
        }
        prompt
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

        let mut sub_tools = ToolRegistry::new();
        sub_tools.register(Box::new(FileReadTool::new_with_compress(
            self.compress_config.clone(),
            self.ccr.clone(),
            spec.parent_session.clone(),
        )));
        sub_tools.register(Box::new(FileWriteTool));
        sub_tools.register(Box::new(DirListTool));
        sub_tools.register(Box::new(FileSearchTool::new_with_compress(
            self.compress_config.clone(),
            self.ccr.clone(),
            spec.parent_session.clone(),
        )));
        sub_tools.register(Box::new(
            ShellExecTool::new_with_compress_and_extra(
                std::time::Duration::from_secs(30),
                spec.working_dir.clone(),
                self.compress_config.clone(),
                self.ccr.clone(),
                spec.parent_session.clone(),
                {
                    let mut extras = self.config.shell_extra_commands.clone();
                    extras.extend(spec.shell_extra_commands.clone());
                    extras.sort();
                    extras.dedup();
                    extras
                },
            ),
        ));
        if let Some(ref v) = self.vault {
            sub_tools.register(Box::new(MemoryRecordTool { vault: Arc::clone(v) }));
            sub_tools.register(Box::new(MemorySearchTool::new(Arc::clone(v))));
        }
        let is_discovery_agent = crate::discovery_code_implementer::is_discovery_agent_brief(&spec.brief);
        if is_discovery_agent {
            crate::discovery_plan_agent::log_spawn_brief(&spec.parent_session, &spec.brief);
            if crate::discovery_plan_agent::is_discovery_agent_brief(&spec.brief) {
                crate::discovery_plan_agent::assert_discovery_parent_session(
                    "discovery-plan:",
                    &spec.parent_session,
                    "plan agent",
                );
            } else if spec.brief.contains("execute workstream") {
                crate::discovery_plan_agent::assert_discovery_parent_session(
                    "discovery-fix:",
                    &spec.parent_session,
                    "execute workstream",
                );
            }
        }
        if !is_discovery_agent {
            sub_tools.register(Box::new(WebBrowseTool::new_with_compress(
                self.compress_config.clone(),
                self.ccr.clone(),
                spec.parent_session.clone(),
            )));
            if self.serpapi_key.is_empty() {
                sub_tools.register(Box::new(WebSearchTool::new_with_compress(
                    String::new(),
                    self.compress_config.clone(),
                    self.ccr.clone(),
                    spec.parent_session.clone(),
                )));
            } else {
                sub_tools.register(Box::new(WebSearchTool::new_with_compress(
                    self.serpapi_key.clone(),
                    self.compress_config.clone(),
                    self.ccr.clone(),
                    spec.parent_session.clone(),
                )));
            }
        }
        let tools = Arc::new(sub_tools);
        let scratch = Arc::clone(&self.scratch);
        let session_id = spec.parent_session.clone();
        let role = spec.role.clone();
        let brief = spec.brief.clone();
        let max_iterations = if is_discovery_agent {
            spec.max_iterations
        } else {
            spec.max_iterations.min(15)
        };
        let discovery_write = if is_discovery_agent {
            crate::discovery_code_implementer::discovery_agent_write_config(&spec.brief, max_iterations)
        } else {
            None
        };
        let summary_max = self.config.summary_max_tokens;
        let context_budget = self.config.context_budget_tokens;
        let system = self.role_system_prompt(&role, &brief);
        let cancel_flags = Arc::clone(&self.cancel_flags);
        let compress_cfg = self.compress_config.clone();
        let ccr = self.ccr.clone();

        let task_id_spawn = task_id.clone();
        let task_id_cleanup = task_id.clone();
        let parent_for_reg = spec.parent_session.clone();
        let parent_session = parent_for_reg.clone();
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
                        hit_max_iterations: false,
                        written_paths: Vec::new(),
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
                        compress_cfg: compress_cfg.clone(),
                        ccr: ccr.clone(),
                    }),
                    write_phase_at: discovery_write.as_ref().map(|c| c.write_phase_at),
                    write_phase_message: discovery_write.as_ref().map(|c| c.write_phase_message.clone()),
                    require_file_write_before_done: is_discovery_agent,
                    require_file_write_prompt: discovery_write
                        .as_ref()
                        .map(|c| c.require_file_write_prompt.clone()),
                };

                let process = if is_discovery_agent {
                    if spec.brief.contains("Discovery code implementer") {
                        crate::obolus::kurator_process_label("discovery_code_implement")
                    } else if crate::discovery_plan_agent::is_discovery_agent_brief(&spec.brief) {
                        crate::obolus::kurator_process_label("discovery_plan")
                    } else {
                        crate::obolus::kurator_process_label("discovery_fix")
                    }
                } else {
                    crate::obolus::kurator_process_label("session_triage")
                };
                let _obolus_ctx = crate::obolus::CallContextGuard::new(crate::obolus::ObolusCallContext {
                    process,
                    task_kind: Some("chat".into()),
                    caller: "subagent".into(),
                    correlation_id: Some(parent_session.clone()),
                    action_id: Some(task_id.clone()),
                });

                let response: AgentResponse =
                    run_agent_loop(gateway.as_ref(), tools.as_ref(), &mut messages, &loop_config)
                        .await?;

                let summary = if compress_cfg.enabled {
                    let cv = crate::context_compress::compress_for_context(
                        &response.text,
                        summary_max,
                        &compress_cfg,
                    );
                    cv.text
                } else {
                    truncate_chars(&response.text, summary_max * 4)
                };

                Ok(SubagentResult {
                    task_id: task_id.clone(),
                    status: SubStatus::Done,
                    summary,
                    llm_calls: response.llm_calls,
                    tool_calls: response.tool_results.len(),
                    hit_max_iterations: response.hit_max_iterations,
                    written_paths: response.written_paths,
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
            if let Some(subs) = reg.get_mut(&parent_for_reg) {
                subs.retain(|s| s.task_id != task_id_cleanup);
                if subs.is_empty() {
                    reg.remove(&spec.parent_session);
                }
            }
        }

        Ok(result)
    }
}
