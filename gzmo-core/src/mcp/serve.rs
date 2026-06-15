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
use crate::platform_memory::PlatformMemory;
use crate::wiki::WikiEngine;

#[derive(Clone)]
pub struct GzmoMemoryMcpServer {
    platform: Arc<PlatformMemory>,
    wiki: WikiConfig,
    tool_router: ToolRouter<Self>,
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
    dynamic_only: Option<bool>,
}

#[tool_router]
impl GzmoMemoryMcpServer {
    pub fn new(platform: Arc<PlatformMemory>, wiki: WikiConfig) -> Self {
        Self {
            platform,
            wiki,
            tool_router: Self::tool_router(),
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
            Ok(res) => Ok(CallToolResult::success(vec![Content::text(res.text)])),
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
            Ok(Some(block)) => Ok(CallToolResult::success(vec![Content::text(block)])),
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
}

#[tool_handler]
impl ServerHandler for GzmoMemoryMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "GZMO platform memory — honeypot RAG search/recall across vault and Pi knowledge, plus gzmo_wiki_search over the git-tracked wiki/ markdown layer."
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
    let server = GzmoMemoryMcpServer::new(platform, config.wiki.clone());
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
