//! # GZMO Memory Subsystem
//!
//! Semantic vault (SQLite), episodic daily logs, and shell blacklist filter.

pub mod core_pin;
pub mod embeddings;
pub mod episodic;
pub mod evidence_localize;
pub mod felt_use;
pub mod filter;
pub mod honeypot;
pub mod kg_extract;
pub mod kg_promotion;
pub mod lifecycle;
pub mod profile;
pub mod qdrant_recall;
pub mod qdrant_sync;
pub mod recall_rrf;
pub mod rerank;
pub mod ripen;
pub mod scratch;
pub mod vault;

pub use scratch::{
    DistillJob, DistillSource, RecallSnippet, ScratchPayload, ScratchScope, ScratchService,
};
