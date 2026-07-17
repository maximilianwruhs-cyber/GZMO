//! GZMO platform memory MCP server (stdio) — exposes `gzmo_memory_*` + ops tools.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use rmcp::{
    handler::server::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};
use tracing::info;

use crate::config::{GzmoConfig, WikiConfig};
use crate::health::{collect_health_probes, format_report};
use crate::platform_memory::PlatformMemory;
use crate::wiki::WikiEngine;

#[derive(Clone)]
pub struct GzmoMemoryMcpServer {
    platform: Arc<PlatformMemory>,
    config: Arc<GzmoConfig>,
    wiki: WikiConfig,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SearchParams {
    query: String,
    #[serde(default)]
    limit: Option<u64>,
    /// When false, search without writing session scratch (default true).
    #[serde(default)]
    write_scratch: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct WikiSearchParams {
    query: String,
    #[serde(default)]
    limit: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ProfileParams {
    #[serde(default)]
    dynamic_only: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ChainParams {
    fact_id: String,
}

fn discovery_data_dir() -> PathBuf {
    std::env::var("PI_MENTOR_DISCOVERY_DATA")
        .or_else(|_| std::env::var("DISCOVERY_DATA_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from("/home/maximilian/gzmo_skills/data/pi-mentor-discovery")
        })
}

fn read_discovery_status_json(data_dir: &Path) -> serde_json::Value {
    let state_path = data_dir.join("state.json");
    let metrics_path = data_dir.join("logs/cycle-metrics.jsonl");
    let lock_path = data_dir.join(".cycle.lock");

    let state = std::fs::read_to_string(&state_path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .unwrap_or(serde_json::json!({"error": "state.json missing"}));

    let mut last_metrics = serde_json::Value::Null;
    if let Ok(text) = std::fs::read_to_string(&metrics_path) {
        if let Some(line) = text.lines().rev().find(|l| !l.trim().is_empty()) {
            last_metrics = serde_json::from_str(line).unwrap_or(serde_json::Value::Null);
        }
    }

    let lock_held = if lock_path.exists() {
        // Best-effort: if another process holds exclusive flock, open+flock -n fails.
        // Without nix crate we only report file presence + recent metrics event.
        true
    } else {
        false
    };

    let bash_calls = last_metrics.get("bash_calls").cloned().unwrap_or(serde_json::json!(null));
    let probe_required_failed = last_metrics
        .get("probe_required_failed")
        .cloned()
        .unwrap_or(serde_json::json!(null));
    let last_event = last_metrics
        .get("event")
        .cloned()
        .unwrap_or(serde_json::json!(null));

    serde_json::json!({
        "data_dir": data_dir.display().to_string(),
        "session_id": state.get("session_id"),
        "session_status": state.get("session_status"),
        "discovery_pillar": state.get("discovery_pillar"),
        "cycle": state.get("cycle"),
        "session_duration_min": state.get("session_duration_min"),
        "session_started_at": state.get("session_started_at"),
        "last_report": state.get("last_report"),
        "last_published_report": state.get("last_published_report"),
        "published": state.get("published"),
        "stale_cycle_count": state.get("stale_cycle_count"),
        "plan_probe_index": state.get("plan_probe_index"),
        "lock_file_present": lock_held,
        "last_metrics": {
            "ts": last_metrics.get("ts"),
            "eval_status": last_metrics.get("eval_status"),
            "bash_calls": bash_calls,
            "mentor_calls": last_metrics.get("mentor_calls"),
            "probe_required_failed": probe_required_failed,
            "probe_id": last_metrics.get("probe_id"),
            "event": last_event,
        }
    })
}

#[tool_router]
impl GzmoMemoryMcpServer {
    pub fn new(platform: Arc<PlatformMemory>, config: Arc<GzmoConfig>) -> Self {
        let wiki = config.wiki.clone();
        Self {
            platform,
            config,
            wiki,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Clear session scratch for a new user turn (call before search/recall).")]
    async fn gzmo_memory_turn_start(&self) -> Result<CallToolResult, McpError> {
        self.platform.turn_start().await;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "turn-start: scratch cleared (session {})",
            self.platform.session_id()
        ))]))
    }

    #[tool(description = "Search GZMO honeypot/vault memory and optional Pi knowledge collection; writes session scratch by default.")]
    async fn gzmo_memory_search(
        &self,
        Parameters(args): Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let limit = args.limit.unwrap_or(5) as usize;
        let write_scratch = args.write_scratch.unwrap_or(true);
        match self
            .platform
            .memory_search(&args.query, limit, write_scratch)
            .await
        {
            Ok(res) => Ok(CallToolResult::success(vec![Content::text(res.text)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Report vault path, fact counts, session id, and scratch backend — use to verify living CT101 attach.")]
    async fn gzmo_memory_status(&self) -> Result<CallToolResult, McpError> {
        match self.platform.status().await {
            Ok(st) => match serde_json::to_string_pretty(&st) {
                Ok(json) => Ok(CallToolResult::success(vec![Content::text(json)])),
                Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
            },
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Return the [RECALL] scratch block for this session.")]
    async fn gzmo_memory_recall_pull(&self) -> Result<CallToolResult, McpError> {
        match self.platform.memory_recall_pull().await {
            Ok(Some(block)) => Ok(CallToolResult::success(vec![Content::text(block)])),
            Ok(None) => Ok(CallToolResult::success(vec![Content::text(
                "(no scratch recall for this session)".to_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Provenance / supersession chain for a honeypot fact id.")]
    async fn gzmo_memory_chain(
        &self,
        Parameters(args): Parameters<ChainParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.platform.memory_chain(&args.fact_id) {
            Ok(chain) if chain.is_empty() => Ok(CallToolResult::success(vec![Content::text(
                format!("(no honeypot chain for id {})", args.fact_id),
            )])),
            Ok(chain) => {
                let mut out = String::new();
                for (i, (content, is_latest, graph_rel)) in chain.iter().enumerate() {
                    let tag = if *is_latest { "latest" } else { "superseded" };
                    let rel = graph_rel.as_deref().unwrap_or("-");
                    out.push_str(&format!("[{i}] ({tag}, rel={rel}) {content}\n"));
                }
                Ok(CallToolResult::success(vec![Content::text(out)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Search the git-tracked wiki/ markdown layer (entity/concept/source pages). Emit-only: reads pages directly, no honeypot writes.")]
    async fn gzmo_wiki_search(
        &self,
        Parameters(args): Parameters<WikiSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        if !self.wiki.enabled {
            return Ok(CallToolResult::error(vec![Content::text(
                "Wiki layer disabled in [wiki] config.".to_string(),
            )]));
        }
        let limit = args.limit.unwrap_or(5) as usize;
        let engine = WikiEngine::new(self.wiki.clone());
        let hits = engine.search(&args.query, limit);
        if hits.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "No wiki pages matched '{}'.",
                args.query
            ))]));
        }
        let mut out = format!("Wiki search '{}' — {} hit(s):\n", args.query, hits.len());
        for h in hits {
            out.push_str(&format!(
                "\n## {} ({})\n{}\n",
                h.title, h.path, h.snippet
            ));
        }
        Ok(CallToolResult::success(vec![Content::text(out)]))
    }

    #[tool(description = "Return cached static+dynamic GZMO operator profile from honeypot.")]
    async fn gzmo_memory_profile(
        &self,
        Parameters(args): Parameters<ProfileParams>,
    ) -> Result<CallToolResult, McpError> {
        let dynamic_only = args.dynamic_only.unwrap_or(false);
        match self.platform.memory_profile(dynamic_only) {
            Ok(profile) => match serde_json::to_string_pretty(&profile) {
                Ok(json) => Ok(CallToolResult::success(vec![Content::text(json)])),
                Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
            },
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Living-instance health probes (LLM, Qdrant, honeypot drift, Redis, Neo4j). Read-only ops gate.")]
    async fn gzmo_ops_health(&self) -> Result<CallToolResult, McpError> {
        let results = collect_health_probes(self.config.as_ref(), None).await;
        let report = format_report(&results);
        let failed: Vec<&str> = results
            .iter()
            .filter(|r| !r.ok && r.name != "sovereign")
            .map(|r| r.name)
            .collect();
        if failed.is_empty() {
            Ok(CallToolResult::success(vec![Content::text(report)]))
        } else {
            Ok(CallToolResult::error(vec![Content::text(format!(
                "{report}\nFailed probes: {}",
                failed.join(", ")
            ))]))
        }
    }

    #[tool(description = "Pi mentor discovery session status (state.json + last cycle metrics: bash_calls, publish). Read-only.")]
    async fn gzmo_discovery_status(&self) -> Result<CallToolResult, McpError> {
        let dir = discovery_data_dir();
        let status = read_discovery_status_json(&dir);
        match serde_json::to_string_pretty(&status) {
            Ok(json) => Ok(CallToolResult::success(vec![Content::text(json)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }
}

#[tool_handler]
impl ServerHandler for GzmoMemoryMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "GZMO living stack MCP — memory (turn_start/search/recall), gzmo_ops_health, gzmo_discovery_status. Verify vault_facts ~60k and discovery bash_calls > 0."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

/// Run the MCP server on stdio until the client disconnects.
/// Honors `GZMO_SESSION_ID` for stable scratch across tool calls.
pub async fn run_mcp_serve(config: &GzmoConfig) -> Result<()> {
    let session_id = std::env::var("GZMO_SESSION_ID")
        .ok()
        .filter(|s| !s.is_empty());
    let platform = Arc::new(PlatformMemory::open(config, session_id).await?);
    info!(
        session = %platform.session_id(),
        vault = %config.memory.vault_db.display(),
        "GZMO memory MCP server starting (stdio)"
    );
    let server = GzmoMemoryMcpServer::new(platform, Arc::new(config.clone()));
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
