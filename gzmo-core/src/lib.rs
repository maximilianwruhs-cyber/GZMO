//! # GZMO Core
//!
//! The complete cognitive engine for the GZMO sovereign agent.
//! All logic — config, LLM gateway, tools, memory, identity,
//! daemon, dreams, MCP, and stealth discovery — lives here.

pub mod config;
pub mod types;
pub mod gateway;
pub mod agent_loop;
pub mod context;
pub mod session;
pub mod tools;
pub mod memory;
pub mod identity;
pub mod daemon;
pub mod dreams;
pub mod mcp;
pub mod stealth;
pub mod orchestrator;
pub mod watcher;
pub mod scanner;
pub mod skills;
