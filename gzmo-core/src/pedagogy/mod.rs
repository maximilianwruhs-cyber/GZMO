//! Agentic Teacher pedagogy stack — learner profile, EDF, orchestrator, Trio Model.

pub mod edf;
pub mod graph;
pub mod intent;
pub mod knowledge_snapshot;
pub mod learner;
pub mod low_tension_opening;
pub mod low_tension_persist;
pub mod orchestrator;
pub mod orchestrator_v2;
pub mod session;
pub mod trio;

pub use edf::{EdfRecord, EdfStore, StealthMetrics, ZpdPhase};
pub use graph::{PrerequisiteGraph, PrerequisiteNode};
pub use intent::{classify_intent, InteractionIntent};
pub use learner::{LearnerProfile, LearnerStore};
pub use knowledge_snapshot::{
    compute_knowledge_delta, delta_to_json, empty_knowledge_delta, empty_knowledge_state,
    knowledge_state_for_cycle_start, knowledge_state_from_handoff_env,
    knowledge_state_from_handoff_path, knowledge_state_from_handoff_value,
    knowledge_state_from_vault, snapshot_to_json, vault_metrics_from_path, KnowledgeDelta,
    KnowledgeStateSnapshot, VaultKnowledgeMetrics,
};
pub use low_tension_opening::{build_opening, LowTensionOpening};
pub use low_tension_persist::{persist_socratic_dialogue, prior_opening_hints};
pub use orchestrator::{OrchestratorInput, OrchestratorOutput, PedagogyOrchestrator};
pub use orchestrator_v2::{OrchestratorInputV2, OrchestratorOutputV2, SimplifiedOrchestrator};
pub use session::PedagogySession;
pub use trio::{TrioMode, TrioState};
