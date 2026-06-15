//! GZMO platform memory MCP server (stdio) — exposes `gzmo_memory_*` tools.

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
use crate::mentor_client::MentorAction;
use crate::platform_memory::PlatformMemory;
use crate::wiki::WikiEngine;

#[derive(Clone)]
pub struct GzmoMemoryMcpServer {
    platform: Arc<PlatformMemory>,
    wiki: WikiConfig,
    mentor_socket_path: std::path::PathBuf,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct TeachParams {
    /// The user message / question to ask the Socratic mentor.
    message: String,
    /// Optional conversation history turns to maintain Socratic dialog context.
    #[serde(default)]
    conversation: Option<Vec<McpMentorTurn>>,
    /// S/A/B/C discovery pillar (Pi mutual discovery sessions)
    #[serde(default)]
    discovery_pillar: Option<String>,
    /// Pillar learn topic (matches gzmo_mentor_learn_start)
    #[serde(default)]
    learn_topic: Option<String>,
    /// Current probe id e.g. S03, or short action summary
    #[serde(default)]
    probe_context: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct McpMentorTurn {
    /// Role of the turn (either 'user' or 'assistant'/'gzmo'/'mentor').
    role: String,
    /// Message content of the turn.
    content: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SearchParams {
    query: String,
    #[serde(default)]
    limit: Option<u64>,
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
    pub dynamic_only: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct RetrieveParams {
    hash: String,
}

#[tool_router]
impl GzmoMemoryMcpServer {
    pub fn new(platform: Arc<PlatformMemory>, wiki: WikiConfig, mentor_socket_path: std::path::PathBuf) -> Self {
        Self {
            platform,
            wiki,
            mentor_socket_path,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Ping the GZMO Socratic mentor API daemon socket to verify status/liveness.")]
    async fn gzmo_mentor_ping(&self) -> Result<CallToolResult, McpError> {
        let req = crate::mentor_client::MentorRequest {
            method: "ping".to_string(),
            message: String::new(),
            conversation: Vec::new(),
            ..Default::default()
        };
        match crate::mentor_client::client_request(&self.mentor_socket_path, &req).await {
            Ok(resp) => {
                if resp.ok {
                    let text = resp.response.unwrap_or_else(|| "pong".to_string());
                    Ok(CallToolResult::success(vec![Content::text(text)]))
                } else {
                    let err = resp.error.unwrap_or_else(|| "ping failed".to_string());
                    Ok(CallToolResult::error(vec![Content::text(err)]))
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to connect to mentor socket: {e}"
            ))])),
        }
    }

    #[tool(description = "Send a Socratic teaching query to the GZMO pedagogy orchestrator over the daemon socket.")]
    async fn gzmo_mentor_teach(
        &self,
        Parameters(args): Parameters<TeachParams>,
    ) -> Result<CallToolResult, McpError> {
        let conversation = args.conversation.unwrap_or_default().into_iter().map(|t| {
            crate::mentor_client::MentorTurn {
                role: t.role,
                content: t.content,
            }
        }).collect();

        let req = crate::mentor_client::MentorRequest {
            method: "teach".to_string(),
            message: args.message,
            conversation,
            discovery_pillar: args.discovery_pillar,
            learn_topic: args.learn_topic,
            probe_context: args.probe_context,
        };

        match crate::mentor_client::client_request(&self.mentor_socket_path, &req).await {
            Ok(resp) => {
                if resp.ok {
                    if resp.action == Some(MentorAction::DelegateExec) {
                        let hint = resp.delegate_hint.unwrap_or_else(|| {
                            "Ops intent detected; execute with the caller's shell/tools.".to_string()
                        });
                        let payload = resp.delegate_payload.unwrap_or_default();
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "delegate_exec\nhint: {hint}\npayload: {payload}"
                        ))]));
                    }
                    if let Some(text) = resp.response {
                        Ok(CallToolResult::success(vec![Content::text(text)]))
                    } else {
                        let err = resp.error.unwrap_or_else(|| "not a mentor turn".to_string());
                        Ok(CallToolResult::success(vec![Content::text(format!(
                            "(Skipped: {err})"
                        ))]))
                    }
                } else {
                    let err = resp.error.unwrap_or_else(|| "teach failed".to_string());
                    Ok(CallToolResult::error(vec![Content::text(err)]))
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to connect to mentor socket: {e}"
            ))])),
        }
    }

    #[tool(description = "Search GZMO honeypot/vault memory and optional Pi knowledge collection; writes session scratch.")]
    async fn gzmo_memory_search(
        &self,
        Parameters(args): Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let limit = args.limit.unwrap_or(5) as usize;
        match self
            .platform
            .memory_search(&args.query, limit, true)
            .await
        {
            Ok(res) => {
                let text = crate::context_compress::compress_for_context_with_ccr(
                    &res.text,
                    self.platform.compress_cfg.recall_compress_budget,
                    &self.platform.compress_cfg,
                    &self.platform.ccr,
                    self.platform.session_id(),
                    true,
                )
                .await
                .text;
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Report vault fact count, session id, and scratch backend state.")]
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
            Ok(Some(block)) => {
                let text = crate::context_compress::compress_for_context_with_ccr(
                    &block,
                    self.platform.compress_cfg.recall_compress_budget,
                    &self.platform.compress_cfg,
                    &self.platform.ccr,
                    self.platform.session_id(),
                    true,
                )
                .await
                .text;
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Ok(None) => Ok(CallToolResult::success(vec![Content::text(
                "(no scratch recall for this session)".to_string(),
            )])),
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
        let text = crate::context_compress::compress_for_context_with_ccr(
            &out,
            1500,
            &self.platform.compress_cfg,
            &self.platform.ccr,
            self.platform.session_id(),
            true,
        )
        .await
        .text;
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Return cached static+dynamic GZMO operator profile from honeypot.")]
    async fn gzmo_memory_profile(
        &self,
        Parameters(args): Parameters<ProfileParams>,
    ) -> Result<CallToolResult, McpError> {
        let dynamic_only = args.dynamic_only.unwrap_or(false);
        match self.platform.memory_profile(dynamic_only) {
            Ok(profile) => match serde_json::to_string_pretty(&profile) {
                Ok(json) => {
                    let text = crate::context_compress::compress_for_context_with_ccr(
                        &json,
                        self.platform.compress_cfg.tool_output_max_tokens,
                        &self.platform.compress_cfg,
                        &self.platform.ccr,
                        self.platform.session_id(),
                        true,
                    )
                    .await
                    .text;
                    Ok(CallToolResult::success(vec![Content::text(text)]))
                }
                Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
            },
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Retrieve full original content by CCR hash from a prior compressed tool/MCP response.")]
    async fn gzmo_retrieve_context(
        &self,
        Parameters(args): Parameters<RetrieveParams>,
    ) -> Result<CallToolResult, McpError> {
        let session_id = self.platform.session_id();
        match self.platform.ccr.retrieve(session_id, &args.hash).await {
            Ok(Some(original)) => Ok(CallToolResult::success(vec![Content::text(original)])),
            Ok(None) => Ok(CallToolResult::error(vec![Content::text(format!(
                "CCR hash '{}' not found or expired for session '{}'.",
                args.hash, session_id
            ))])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }
}

#[tool_handler]
impl ServerHandler for GzmoMemoryMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "GZMO platform memory and Socratic mentor — honeypot RAG search/recall, wiki search, and mentor Socratic teaching."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

/// Run the MCP server on stdio until the client disconnects.
pub async fn run_mcp_serve(config: &GzmoConfig) -> Result<()> {
    let platform = Arc::new(PlatformMemory::open(config, None).await?);
    info!(
        session = %platform.session_id(),
        "GZMO memory MCP server starting (stdio)"
    );
    let server = GzmoMemoryMcpServer::new(
        platform,
        config.wiki.clone(),
        config.pedagogy.mentor_socket_path(),
    );
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
