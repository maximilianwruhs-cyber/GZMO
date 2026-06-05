//! # GZMO Memory Subsystem
//!
//! Semantic vault (SQLite), episodic daily logs, and shell blacklist filter.

pub mod honeypot;
pub mod lifecycle;
pub mod profile;
pub mod qdrant_recall;
pub mod recall_rrf;
pub mod vault;
pub mod episodic;
pub mod filter;
pub mod kg_promotion;
pub mod evidence_localize;
pub mod kg_extract;
pub mod vault_backend;
pub mod embeddings;
pub mod rerank;
pub mod qdrant_sync;
pub mod scratch;

pub use scratch::{DistillJob, DistillSource, RecallSnippet, ScratchPayload, ScratchScope, ScratchService};
