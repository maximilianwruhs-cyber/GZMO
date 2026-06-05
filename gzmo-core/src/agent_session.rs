//! Hot-memory session lifecycle for any frontend (REPL harness, future operator clients).
//!
//! Owns scratch scope, turn boundaries (clear / cancel subs), and [`AgentLoopConfig`] wiring.

use std::sync::Arc;

use crate::agent_loop::{AgentLoopConfig, AgentMemoryContext};
use crate::config::{ContextMemoryConfig, RedisConfig};
use crate::context::ContextConfig;
use crate::memory::scratch::{ScratchScope, ScratchService};
use crate::subagent::SubagentRunner;

/// Per-conversation hot memory: scratch pad, archive @ threshold, optional subagent governor.
pub struct AgentSession {
    scratch: Arc<ScratchService>,
    session_id: String,
    context: ContextConfig,
    subagent: Option<Arc<SubagentRunner>>,
}

impl AgentSession {
    /// Main session scope (`scratch:main:{session_id}`).
    pub async fn new_main(
        redis: &RedisConfig,
        context_memory: &ContextMemoryConfig,
        session_id: String,
    ) -> Self {
        let scratch = Arc::new(ScratchService::from_config(redis, context_memory).await);
        Self {
            scratch,
            session_id: session_id.clone(),
            context: ContextConfig::from_memory_config(context_memory),
            subagent: None,
        }
    }

    pub fn scratch(&self) -> Arc<ScratchService> {
        Arc::clone(&self.scratch)
    }

    pub fn uses_redis(&self) -> bool {
        self.scratch.uses_redis()
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn set_session_id(&mut self, session_id: String) {
        self.session_id = session_id;
    }

    pub fn main_scope(&self) -> ScratchScope {
        ScratchScope::Main {
            session_id: self.session_id.clone(),
        }
    }

    pub fn attach_subagent_runner(&mut self, runner: Arc<SubagentRunner>) {
        self.subagent = Some(runner);
    }

    /// Start of a user turn: drop stale recall and cancel in-flight subagents.
    pub async fn turn_start(&self) {
        if let Some(ref runner) = self.subagent {
            runner.cancel_all(&self.session_id).await;
        }
        let _ = self.scratch.clear(&self.main_scope()).await;
    }

    pub fn memory_context(&self) -> AgentMemoryContext {
        AgentMemoryContext {
            scratch: Arc::clone(&self.scratch),
            session_id: self.session_id.clone(),
            scope: self.main_scope(),
        }
    }

    pub fn loop_config(
        &self,
        max_iterations: usize,
        verbose_tool_output: bool,
        on_chunk: Option<Arc<dyn Fn(String) + Send + Sync>>,
    ) -> AgentLoopConfig {
        AgentLoopConfig {
            max_iterations,
            verbose_tool_output,
            context: self.context.clone(),
            on_chunk,
            memory: Some(self.memory_context()),
        }
    }
}
