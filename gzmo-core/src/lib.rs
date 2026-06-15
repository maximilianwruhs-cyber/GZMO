//! # GZMO Core
//!
//! The complete cognitive engine for the GZMO sovereign agent.
//! All logic — config, LLM gateway, tools, memory, identity,
//! daemon, dreams, MCP, and stealth discovery — lives here.

pub mod agent_loop;
pub mod agent_session;
pub mod config;
pub mod context;
pub mod daemon;
pub mod dreams;
pub mod dreams_md;
pub mod gateway;
pub mod health;
pub mod identity;
pub mod ingest;
pub mod ingest_prep;
pub mod kg_reconcile;
pub mod mcp;
pub mod memory;
pub mod orchestrator;
pub mod platform_memory;
pub mod platform_search;
pub mod scanner;
pub mod session;
pub mod session_distill;
pub mod skills;
pub mod spark;
pub mod spark_schedule;
pub mod stealth;
pub mod subagent;
pub mod synapse;
pub mod synapse_reader;
pub mod text_util;
pub mod tools;
pub mod types;
pub mod watcher;
pub mod wiki;
pub mod wiki_md;
