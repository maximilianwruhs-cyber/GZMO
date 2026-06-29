//! # GZMO Core
//!
//! The complete cognitive engine for the GZMO sovereign agent.
//! All logic — config, LLM gateway, tools, memory, identity,
//! daemon, dreams, MCP, and stealth discovery — lives here.

pub mod compliance;
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
pub mod cycle_guard;
pub mod dreams;
pub mod dreams_md;
pub mod spark;
pub mod spark_anchor_refresh;
pub mod spark_schedule;
pub mod spark_distill_bridge;
pub mod ingest;
pub mod ingest_prep;
pub mod wiki;
pub mod wiki_md;
pub mod session_distill;
pub mod pi_session;
pub mod mcp;
pub mod stealth;
pub mod dice_loop;
pub mod orchestrator;
pub mod watcher;
pub mod scanner;
pub mod skills;
pub mod health;
pub mod synapse;
pub mod synapse_reader;
pub mod kurator_monitor;
pub mod kurator_spawn;
pub mod spawn_gate;
pub mod spawn_prime_budget;
pub mod discovery_code_implementer;
pub mod discovery_plan_agent;
pub mod discovery_execute;
pub mod discovery_acceptance_gate;
pub mod discovery_fixer;
pub mod discovery_git_context;
pub mod remediation_snapshot;
pub mod spawn_polling;
pub mod remediation_tracker;
pub mod synapse_writer;
pub mod bibliothek;
pub mod kg_reconcile;
pub mod platform_search;
pub mod text_util;
pub mod subagent;
pub mod context_compress;
pub mod pedagogy;
pub mod self_improving;
pub mod strategies;
pub mod pi_recent_discoveries;
pub mod mentor_client;
pub mod obolus;
