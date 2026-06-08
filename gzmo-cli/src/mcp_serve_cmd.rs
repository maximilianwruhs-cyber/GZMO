//! `gzmo mcp-serve` — stdio MCP server for platform memory tools.

use anyhow::Result;
use gzmo_core::config::GzmoConfig;
use gzmo_core::mcp::serve::run_mcp_serve;

pub async fn run(config: &GzmoConfig) -> Result<()> {
    run_mcp_serve(config).await
}
