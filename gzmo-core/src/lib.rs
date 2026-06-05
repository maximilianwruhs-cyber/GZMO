//! # GZMO Core
//!
//! The complete cognitive engine for the GZMO sovereign agent.
//! All logic — config, LLM gateway, tools, memory, identity,
//! daemon, dreams, MCP, and stealth discovery — lives here.

pub mod config;
pub mod types;
pub mod gateway;
pub mod agent_loop;
pub mod agent_session;
pub mod platform_memory;
pub mod context;
pub mod session;
pub mod tools;
pub mod memory;
pub mod identity;
pub mod daemon;
pub mod dreams;
pub mod dreams_md;
pub mod spark;
pub mod spark_schedule;
pub mod ingest;
pub mod ingest_prep;
pub mod session_distill;
pub mod mcp;
pub mod stealth;
pub mod orchestrator;
pub mod watcher;
pub mod scanner;
pub mod skills;
pub mod health;
pub mod synapse;
pub mod text_util;
pub mod subagent;
