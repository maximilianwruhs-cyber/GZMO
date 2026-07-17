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
use crate::config::{GzmoConfig, SubagentConfig, ToolsConfig};
use crate::context::ContextConfig;
use crate::gateway::LlmGateway;
use crate::memory::scratch::{ScratchScope, ScratchService};
use crate::memory::vault::SqliteVault;
use crate::text_util::truncate_chars;
use crate::tools::profile::{register_for_profile, CapabilityProfile, ToolRegisterOpts};
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubagentStructured {
    #[serde(default)]
    pub findings: Vec<String>,
    #[serde(default)]
    pub evidence: Vec<String>,
    #[serde(default)]
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentResult {
    pub task_id: String,
    pub status: SubStatus,
    pub summary: String,
    pub llm_calls: usize,
    pub tool_calls: usize,
    #[serde(default)]
    pub structured: SubagentStructured,
    /// Capability profile used for this spawn.
    #[serde(default)]
    pub profile: String,
}

struct RunningSub {
    task_id: String,
    handle: JoinHandle<()>,
}

pub struct SubagentRunner {
    config: SubagentConfig,
    tools_cfg: ToolsConfig,
    gzmo_config: Option<GzmoConfig>,
    scratch: Arc<ScratchService>,
    gateway: Arc<dyn LlmGateway>,
    vault: Option<Arc<SqliteVault>>,
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
        Self::with_tools_config(
            config,
            ToolsConfig::default(),
            scratch,
            gateway,
            vault,
            system_prompt_base,
            None,
        )
    }

    pub fn with_tools_config(
        config: SubagentConfig,
        tools_cfg: ToolsConfig,
        scratch: Arc<ScratchService>,
        gateway: Arc<dyn LlmGateway>,
        vault: Option<Arc<SqliteVault>>,
        system_prompt_base: String,
        gzmo_config: Option<GzmoConfig>,
    ) -> Self {
        Self {
            config,
            tools_cfg,
            gzmo_config,
            scratch,
            gateway,
            vault,
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

    fn role_system_prompt(&self, role: &str, brief: &str, profile: CapabilityProfile) -> String {
        format!(
            "{}\n\n---\nYou are a focused sub-agent (role: {role}, tools profile: {}). \
            Complete only the task in the user message. \
            Do not delegate further sub-tasks. \
            End with a JSON block (and nothing after it) of the form:\n\
            ```json\n{{\n  \"summary\": \"...\",\n  \"findings\": [\"...\"],\n  \
            \"evidence\": [\"...\"],\n  \"next_actions\": [\"...\"]\n}}\n```\n\
            Keep summary under {} tokens.\n\nTask:\n{brief}",
            self.system_prompt_base,
            profile.as_str(),
            self.config.summary_max_tokens,
        )
    }

    fn build_tools_for_role(&self, role: &str) -> Result<(CapabilityProfile, Arc<ToolRegistry>)> {
        let profile = CapabilityProfile::for_subagent_role(role);
        let mut tools = ToolRegistry::new();
        register_for_profile(
            &mut tools,
            profile,
            &self.tools_cfg,
            ToolRegisterOpts {
                vault: self.vault.clone(),
                scratch: Some(Arc::clone(&self.scratch)),
                scratch_scope: None,
                serpapi_key: None,
                workflow: None,
                gzmo_config: self.gzmo_config.clone(),
            },
        )?;
        Ok((profile, Arc::new(tools)))
    }

    pub async fn spawn(&self, spec: SubagentSpec) -> Result<SubagentResult> {
        if !self.config.enabled {
            bail!("Subagents disabled in [subagent] config");
        }
        if spec.depth > self.config.max_depth {
            bail!(
                "Subagent depth {} exceeds max {}",
                spec.depth,
                self.config.max_depth
            );
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

        let (profile, tools) = self.build_tools_for_role(&spec.role)?;
        let task_id = Uuid::new_v4().to_string();
        let scope = ScratchScope::Sub {
            session_id: spec.parent_session.clone(),
            task_id: task_id.clone(),
        };
        let _ = self.scratch.clear(&scope).await;

        let (tx, rx) = tokio::sync::oneshot::channel();
        let gateway = Arc::clone(&self.gateway);
        let scratch = Arc::clone(&self.scratch);
        let session_id = spec.parent_session.clone();
        let role = spec.role.clone();
        let brief = spec.brief.clone();
        let max_iterations = spec.max_iterations.min(60);
        let summary_max = self.config.summary_max_tokens;
        let context_budget = self.config.context_budget_tokens;
        let system = self.role_system_prompt(&role, &brief, profile);
        let cancel_flags = Arc::clone(&self.cancel_flags);
        let parent_session = spec.parent_session.clone();
        let profile_str = profile.as_str().to_string();

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
                        structured: SubagentStructured::default(),
                        profile: profile_str.clone(),
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

                let response: AgentResponse = run_agent_loop(
                    gateway.as_ref(),
                    tools.as_ref(),
                    &mut messages,
                    &loop_config,
                )
                .await?;

                let (summary, structured) = parse_structured_reply(&response.text, summary_max);

                Ok(SubagentResult {
                    task_id: task_id.clone(),
                    status: SubStatus::Done,
                    summary,
                    llm_calls: response.llm_calls,
                    tool_calls: response.tool_results.len(),
                    structured,
                    profile: profile_str,
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

#[derive(Debug, Deserialize)]
struct StructuredReply {
    #[serde(default)]
    summary: String,
    #[serde(default)]
    findings: Vec<String>,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default)]
    next_actions: Vec<String>,
}

fn parse_structured_reply(text: &str, summary_max: usize) -> (String, SubagentStructured) {
    if let Some(json_str) = extract_json_block(text) {
        if let Ok(parsed) = serde_json::from_str::<StructuredReply>(json_str) {
            let summary = if parsed.summary.is_empty() {
                truncate_chars(text, summary_max * 4)
            } else {
                truncate_chars(&parsed.summary, summary_max * 4)
            };
            return (
                summary,
                SubagentStructured {
                    findings: parsed.findings,
                    evidence: parsed.evidence,
                    next_actions: parsed.next_actions,
                },
            );
        }
    }
    (
        truncate_chars(text, summary_max * 4),
        SubagentStructured::default(),
    )
}

fn extract_json_block(text: &str) -> Option<&str> {
    if let Some(start) = text.find("```json") {
        let after = &text[start + 7..];
        let after = after.strip_prefix('\n').unwrap_or(after);
        let end = after.find("```")?;
        return Some(after[..end].trim());
    }
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end > start {
        Some(text[start..=end].trim())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_json_fence() {
        let text = r#"Done.

```json
{
  "summary": "Fixed the bug",
  "findings": ["null deref"],
  "evidence": ["cargo test ok"],
  "next_actions": ["merge"]
}
```
"#;
        let (summary, structured) = parse_structured_reply(text, 200);
        assert_eq!(summary, "Fixed the bug");
        assert_eq!(structured.findings, vec!["null deref".to_string()]);
        assert_eq!(structured.next_actions, vec!["merge".to_string()]);
    }
}
