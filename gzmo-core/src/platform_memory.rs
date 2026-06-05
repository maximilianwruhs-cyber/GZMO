//! Frontend-facing memory API (`gzmo_memory_*`) — same hot/cold path as [`AgentSession`].

use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::agent_session::AgentSession;
use crate::config::GzmoConfig;
use crate::memory::embeddings;
use crate::memory::profile::{GzmoProfile, ProfileOptions};
use crate::memory::scratch::{RecallSnippet, ScratchScope, ScratchService};
use crate::memory::vault::SqliteVault;
use crate::session::SessionManager;
use crate::tools::{ToolDef, ToolHandler};

/// Vault + hot session for operator frontends (pi-rust, scripts, MCP later).
pub struct PlatformMemory {
    pub vault: Arc<SqliteVault>,
    pub session: AgentSession,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHit {
    pub content: String,
    pub score: f32,
    /// Source archive file the recalled fact was promoted from (honeypot provenance).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fact_id: Option<uuid::Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    pub query: String,
    pub hits: usize,
    pub items: Vec<MemoryHit>,
    pub text: String,
    pub scratch_written: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatusReport {
    pub session_id: String,
    pub vault_facts: usize,
    pub scratch_backend: String,
    pub scratch_has_recall: bool,
}

impl PlatformMemory {
    /// Bind an already-open vault and session (e.g. legacy `gzmo chat` harness).
    pub fn from_parts(vault: Arc<SqliteVault>, session: AgentSession) -> Self {
        Self { vault, session }
    }

    pub async fn open(config: &GzmoConfig, session_id: Option<String>) -> Result<Self> {
        let sid = session_id.unwrap_or_else(|| {
            std::env::var("GZMO_SESSION_ID")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(SessionManager::new_session_id)
        });

        let vault = embeddings::open_vault_with_embeddings(
            &config.memory.vault_db,
            &config.embeddings,
            &config.rerank,
            &config.qdrant,
        )
        .await?;

        let session =
            AgentSession::new_main(&config.redis, &config.context_memory, sid).await;

        Ok(Self {
            vault: Arc::new(vault),
            session,
        })
    }

    pub fn session_id(&self) -> &str {
        self.session.session_id()
    }

    pub fn scratch_scope(&self) -> ScratchScope {
        self.session.main_scope()
    }

    pub async fn turn_start(&self) {
        self.session.turn_start().await;
    }

    /// `gzmo_memory_search` — recall into honeypot/vault and write scratch for this session.
    pub async fn memory_search(
        &self,
        query: &str,
        limit: usize,
        write_scratch: bool,
    ) -> Result<MemorySearchResult> {
        let limit = limit.clamp(1, 20);
        let (text, results) = memory_search_core(&self.vault, query, limit).await?;
        let scratch_written = write_scratch && !results.is_empty();
        let hits = results.len();
        let items: Vec<MemoryHit> = results
            .iter()
            .map(|(fact, score)| {
                let ev_text = self.vault.get_evidence_text(fact.id).ok().flatten();
                MemoryHit {
                    content: fact.content.clone(),
                    score: *score as f32,
                    source_file: self.vault.honeypot_source_file(fact.id).ok().flatten(),
                    fact_id: Some(fact.id),
                    evidence_text: ev_text,
                }
            })
            .collect();

        if scratch_written {
            let snippets: Vec<RecallSnippet> = results
                .iter()
                .map(|(fact, score)| RecallSnippet {
                    content: fact.content.clone(),
                    score: *score as f32,
                    fact_id: Some(fact.id.to_string()),
                    evidence_text: self.vault.get_evidence_text(fact.id).ok().flatten(),
                })
                .collect();
            self.session.scratch().write(&self.scratch_scope(), snippets).await?;
        }

        Ok(MemorySearchResult {
            query: query.to_string(),
            hits,
            items,
            text,
            scratch_written,
        })
    }

    /// `gzmo_memory_recall_pull` — formatted `[RECALL]` block for the next model turn.
    pub async fn memory_recall_pull(&self) -> Result<Option<String>> {
        self.session
            .scratch()
            .format_for_inject(&self.scratch_scope())
            .await
    }

    /// Cached static+dynamic profile for session injection (Spec §5).
    pub fn memory_profile(&self, dynamic_only: bool) -> Result<GzmoProfile> {
        self.vault.build_profile(ProfileOptions {
            dynamic_only,
            ..ProfileOptions::default()
        })
    }

    pub async fn status(&self) -> Result<MemoryStatusReport> {
        let vault_facts = self.vault.count().unwrap_or(0);
        let scratch_backend = if self.session.uses_redis() {
            "redis"
        } else {
            "in-memory"
        };
        let scratch_has_recall = self
            .session
            .scratch()
            .read(&self.scratch_scope())
            .await?
            .map(|p| !p.snippets.is_empty())
            .unwrap_or(false);

        Ok(MemoryStatusReport {
            session_id: self.session.session_id().to_string(),
            vault_facts,
            scratch_backend: scratch_backend.to_string(),
            scratch_has_recall,
        })
    }
}

/// RRF recall + formatted text (no scratch write).
pub async fn memory_search_core(
    vault: &SqliteVault,
    query: &str,
    limit: usize,
) -> Result<(String, Vec<(crate::types::SemanticFact, f64)>)> {
    let results = vault.search_recall(query, limit).await?;
    if results.is_empty() {
        return Ok((
            format!("No relevant memories found for query: '{query}'"),
            Vec::new(),
        ));
    }
    let mut out = String::new();
    out.push_str(&format!("Honeypot recall for '{query}':\n\n"));
    for (fact, score) in &results {
        let dt = fact.created_at.format("%Y-%m-%d").to_string();
        out.push_str(&format!(
            "- [{}] (Score: {:.2}) {}\n",
            dt, score, fact.content
        ));
    }
    Ok((out, results))
}

/// Shared search + scratch write (used by `gzmo_memory_search` tools).
pub async fn memory_search_into_scratch(
    vault: &SqliteVault,
    scratch: Arc<ScratchService>,
    scope: &ScratchScope,
    query: &str,
    limit: usize,
) -> Result<String> {
    let (text, results) = memory_search_core(vault, query, limit).await?;
    if !results.is_empty() {
        let snippets: Vec<RecallSnippet> = results
            .iter()
            .map(|(fact, score)| RecallSnippet {
                content: fact.content.clone(),
                score: *score as f32,
                fact_id: Some(fact.id.to_string()),
                evidence_text: vault.get_evidence_text(fact.id).ok().flatten(),
            })
            .collect();
        scratch.write(scope, snippets).await?;
    }
    Ok(text)
}

/// Tool surface for frontends: `gzmo_memory_search`.
pub struct GzmoMemorySearchTool {
    platform: Arc<PlatformMemory>,
}

impl GzmoMemorySearchTool {
    pub fn new(platform: Arc<PlatformMemory>) -> Self {
        Self { platform }
    }
}

#[async_trait]
impl ToolHandler for GzmoMemorySearchTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "gzmo_memory_search".to_string(),
            description: "Search GZMO honeypot/vault memory and store hits in session scratch for recall injection.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" },
                    "limit": { "type": "integer", "description": "Max results (1-20, default 5)" }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let query = args["query"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'query'"))?;
        let limit = args["limit"].as_u64().unwrap_or(5) as usize;
        let res = self.platform.memory_search(query, limit, true).await?;
        Ok(res.text)
    }
}

/// Tool surface: `gzmo_memory_status`.
pub struct GzmoMemoryStatusTool {
    platform: Arc<PlatformMemory>,
}

impl GzmoMemoryStatusTool {
    pub fn new(platform: Arc<PlatformMemory>) -> Self {
        Self { platform }
    }
}

#[async_trait]
impl ToolHandler for GzmoMemoryStatusTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "gzmo_memory_status".to_string(),
            description: "Report vault fact count, session id, and scratch backend state.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
            }),
        }
    }

    async fn execute(&self, _args: serde_json::Value) -> Result<String> {
        let st = self.platform.status().await?;
        Ok(serde_json::to_string_pretty(&st)?)
    }
}

/// Tool surface: `gzmo_memory_recall_pull`.
pub struct GzmoMemoryRecallPullTool {
    platform: Arc<PlatformMemory>,
}

impl GzmoMemoryRecallPullTool {
    pub fn new(platform: Arc<PlatformMemory>) -> Self {
        Self { platform }
    }
}

#[async_trait]
impl ToolHandler for GzmoMemoryRecallPullTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "gzmo_memory_recall_pull".to_string(),
            description: "Return the [RECALL] scratch block for this session (inject into the next LLM turn).".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
            }),
        }
    }

    async fn execute(&self, _args: serde_json::Value) -> Result<String> {
        match self.platform.memory_recall_pull().await? {
            Some(block) => Ok(block),
            None => Ok("(no scratch recall for this session)".to_string()),
        }
    }
}

/// Tool surface: `gzmo_memory_profile`.
pub struct GzmoMemoryProfileTool {
    platform: Arc<PlatformMemory>,
}

impl GzmoMemoryProfileTool {
    pub fn new(platform: Arc<PlatformMemory>) -> Self {
        Self { platform }
    }
}

#[async_trait]
impl ToolHandler for GzmoMemoryProfileTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "gzmo_memory_profile".to_string(),
            description: "Return cached static+dynamic GZMO operator profile from honeypot.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "dynamic_only": { "type": "boolean", "description": "Omit static structural facts" }
                },
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let dynamic_only = args["dynamic_only"].as_bool().unwrap_or(false);
        let profile = self.platform.memory_profile(dynamic_only)?;
        Ok(serde_json::to_string_pretty(&profile)?)
    }
}
