//! Frontend-facing memory API (`gzmo_memory_*`) — same hot/cold path as [`AgentSession`].

use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::agent_session::AgentSession;
use crate::config::{
    EmbeddingsConfig, GzmoConfig, PlatformSearchConfig, QdrantConfig, RedisConfig, RerankConfig,
};
use crate::memory::embeddings;
use crate::memory::felt_use::{self, FeltUseKind};
use crate::memory::profile::{GzmoProfile, ProfileOptions};
use crate::memory::scratch::{RecallSnippet, ScratchScope, ScratchService};
use crate::memory::vault::SqliteVault;
use crate::platform_search::platform_cross_search;
use crate::session::SessionManager;
use crate::tools::{ToolDef, ToolHandler};

/// Vault + hot session for operator frontends (pi-rust, scripts, MCP later).
pub struct PlatformMemory {
    pub vault: Arc<SqliteVault>,
    pub session: AgentSession,
    vault_path: String,
    platform_search: PlatformSearchConfig,
    qdrant: QdrantConfig,
    embeddings: EmbeddingsConfig,
    redis: RedisConfig,
    rerank: RerankConfig,
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
    pub vault_path: String,
    pub vault_facts: usize,
    pub honeypot_latest: usize,
    pub scratch_backend: String,
    pub scratch_has_recall: bool,
    /// `owner` when served over the control-plane socket; `in-process` otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_plane: Option<String>,
}

/// Living vault floor. Lab/product/empty vaults use `GZMO_ALLOW_LAB_VAULT`,
/// `GZMO_PRODUCT`, or a path under `~/.gzmo/`.
pub const LIVING_VAULT_MIN_FACTS: usize = 10_000;

fn env_flag_true(name: &str) -> bool {
    std::env::var(name)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Product (`gzmo init` → ~/.gzmo) and explicit lab flags may attach small vaults.
fn allow_lab_or_product_vault(vault_path: &Path) -> bool {
    if env_flag_true("GZMO_ALLOW_LAB_VAULT") || env_flag_true("GZMO_PRODUCT") {
        return true;
    }
    if let Some(home) = std::env::var_os("HOME") {
        let product_root = Path::new(&home).join(".gzmo");
        if vault_path.starts_with(&product_root) {
            return true;
        }
    }
    false
}

impl PlatformMemory {
    /// Bind an already-open vault and session (e.g. legacy `gzmo chat` harness).
    pub fn from_parts(vault: Arc<SqliteVault>, session: AgentSession) -> Self {
        Self {
            vault,
            session,
            vault_path: "(bound)".into(),
            platform_search: PlatformSearchConfig::default(),
            qdrant: QdrantConfig::default(),
            embeddings: EmbeddingsConfig::default(),
            redis: RedisConfig::default(),
            rerank: RerankConfig::default(),
        }
    }

    pub async fn open(config: &GzmoConfig, session_id: Option<String>) -> Result<Self> {
        Self::open_inner(config, session_id, false).await
    }

    /// Owner process may open a small/bootstrap vault. Clients still hit the living floor.
    pub async fn open_as_owner(config: &GzmoConfig, session_id: Option<String>) -> Result<Self> {
        Self::open_inner(config, session_id, true).await
    }

    async fn open_inner(
        config: &GzmoConfig,
        session_id: Option<String>,
        as_owner: bool,
    ) -> Result<Self> {
        let sid = session_id.unwrap_or_else(|| {
            std::env::var("GZMO_SESSION_ID")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(SessionManager::new_session_id)
        });

        let vault = embeddings::open_vault_with_embeddings(
            &config.memory.vault_db,
            &config.embeddings,
            &config.redis,
            &config.rerank,
            &config.qdrant,
        )
        .await?;

        let facts = vault.count().unwrap_or(0);
        let allow_lab = as_owner || allow_lab_or_product_vault(&config.memory.vault_db);
        if facts < LIVING_VAULT_MIN_FACTS && !allow_lab {
            anyhow::bail!(
                "refusing vault attach: {} has only {facts} facts (need ≥{LIVING_VAULT_MIN_FACTS} \
                 for living instance). Point GZMO_CONFIG at the living vault, set \
                 GZMO_ALLOW_LAB_VAULT=1 / GZMO_PRODUCT=1, or run `gzmo init` (~/.gzmo).",
                config.memory.vault_db.display()
            );
        }

        let session = AgentSession::new_main(&config.redis, &config.context_memory, sid).await;

        Ok(Self {
            vault: Arc::new(vault),
            session,
            vault_path: config.memory.vault_db.display().to_string(),
            platform_search: config.platform_search.clone(),
            qdrant: config.qdrant.clone(),
            embeddings: config.embeddings.clone(),
            redis: config.redis.clone(),
            rerank: config.rerank.clone(),
        })
    }

    pub fn vault_path(&self) -> &str {
        &self.vault_path
    }

    fn scope_for(&self, session_id: Option<&str>) -> ScratchScope {
        ScratchScope::Main {
            session_id: session_id
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| self.session_id())
                .to_string(),
        }
    }

    fn session_label<'a>(&'a self, session_id: Option<&'a str>) -> &'a str {
        session_id
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.session_id())
    }

    pub fn session_id(&self) -> &str {
        self.session.session_id()
    }

    pub fn scratch_scope(&self) -> ScratchScope {
        self.session.main_scope()
    }

    pub async fn turn_start(&self) {
        self.turn_start_scoped(None).await;
    }

    /// Clear scratch for `session_id` (or the bound session). Returns the session label.
    pub async fn turn_start_scoped(&self, session_id: Option<&str>) -> String {
        let scope = self.scope_for(session_id);
        let _ = self.session.scratch().clear(&scope).await;
        self.session_label(session_id).to_string()
    }

    /// `gzmo_memory_search` — recall into honeypot/vault and write scratch for this session.
    pub async fn memory_search(
        &self,
        query: &str,
        limit: usize,
        write_scratch: bool,
    ) -> Result<MemorySearchResult> {
        self.memory_search_scoped(None, query, limit, write_scratch)
            .await
    }

    pub async fn memory_search_scoped(
        &self,
        session_id: Option<&str>,
        query: &str,
        limit: usize,
        write_scratch: bool,
    ) -> Result<MemorySearchResult> {
        let limit = limit.clamp(1, 20);
        let (text, items) = platform_cross_search(
            &self.vault,
            &self.platform_search,
            &self.qdrant,
            &self.embeddings,
            &self.redis,
            &self.rerank,
            query,
            limit,
        )
        .await?;
        let scratch_written = write_scratch && !items.is_empty();
        let hits = items.len();

        if scratch_written {
            let snippets: Vec<RecallSnippet> = items
                .iter()
                .map(|hit| RecallSnippet {
                    content: hit.content.clone(),
                    score: hit.score,
                    fact_id: hit.fact_id.map(|id| id.to_string()),
                    evidence_text: hit
                        .fact_id
                        .and_then(|id| self.vault.get_evidence_text(id).ok().flatten())
                        .or_else(|| hit.evidence_text.clone()),
                })
                .collect();
            self.session
                .scratch()
                .write(&self.scope_for(session_id), snippets)
                .await?;
        }

        // Felt Use: Cited when scratch written, else Glance for ranked hits.
        let kind = if scratch_written {
            FeltUseKind::Cited
        } else {
            FeltUseKind::Glance
        };
        felt_use::touch_hits(&self.vault, items.iter().map(|h| h.fact_id.as_ref()), kind);

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
        self.memory_recall_pull_scoped(None).await
    }

    pub async fn memory_recall_pull_scoped(
        &self,
        session_id: Option<&str>,
    ) -> Result<Option<String>> {
        self.session
            .scratch()
            .format_for_inject(&self.scope_for(session_id))
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
        self.status_scoped(None).await
    }

    pub async fn status_scoped(&self, session_id: Option<&str>) -> Result<MemoryStatusReport> {
        let vault_facts = self.vault.count().unwrap_or(0);
        let honeypot_latest = self.vault.count_honeypot_latest().unwrap_or(0);
        let scratch_backend = if self.session.uses_redis() {
            if self.session.scratch().redis_live().await {
                "redis"
            } else {
                "redis (unreachable — in-memory buffer)"
            }
        } else {
            "in-memory"
        };
        let scratch_has_recall = self
            .session
            .scratch()
            .read(&self.scope_for(session_id))
            .await?
            .map(|p| !p.snippets.is_empty())
            .unwrap_or(false);

        Ok(MemoryStatusReport {
            session_id: self.session_label(session_id).to_string(),
            vault_path: self.vault_path.clone(),
            vault_facts,
            honeypot_latest,
            scratch_backend: scratch_backend.to_string(),
            scratch_has_recall,
            control_plane: Some(crate::control_plane::VIA_IN_PROCESS.to_string()),
        })
    }

    /// Provenance / supersession chain for a honeypot fact id.
    pub fn memory_chain(&self, fact_id: &str) -> Result<Vec<(String, bool, Option<String>)>> {
        self.vault.get_memory_chain(fact_id)
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
        felt_use::touch_hits(
            vault,
            results.iter().map(|(f, _)| Some(&f.id)),
            FeltUseKind::Cited,
        );
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
            description: "Search GZMO honeypot/vault + Pi knowledge (when [platform_search] enabled); stores hits in session scratch.".to_string(),
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
            description: "Report vault fact count, session id, and scratch backend state."
                .to_string(),
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
            description: "Return cached static+dynamic GZMO operator profile from honeypot."
                .to_string(),
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
