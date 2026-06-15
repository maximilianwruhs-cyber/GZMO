//! Agentic Teacher pedagogy stack — learner profile, EDF, orchestrator, Trio Model.

pub mod edf;
pub mod graph;
pub mod intent;
pub mod learner;
pub mod low_tension_opening;
pub mod low_tension_persist;
pub mod orchestrator;
pub mod session;
pub mod trio;

pub use edf::{EdfRecord, EdfStore, StealthMetrics, ZpdPhase};
pub use graph::{PrerequisiteGraph, PrerequisiteNode};
pub use intent::{classify_intent, InteractionIntent};
pub use learner::{LearnerProfile, LearnerStore};
pub use low_tension_opening::{build_opening, LowTensionOpening};
pub use low_tension_persist::{persist_socratic_dialogue, prior_opening_hints};
pub use orchestrator::{OrchestratorInput, OrchestratorOutput, PedagogyOrchestrator};
pub use session::PedagogySession;
pub use trio::{TrioMode, TrioState};
