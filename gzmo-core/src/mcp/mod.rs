//! # MCP (Model Context Protocol) Integration
//!
//! Manages MCP server child processes and bridges their tools
//! into the GZMO tool registry via JSON-RPC over stdio.

pub mod manager;
pub mod bridge;
