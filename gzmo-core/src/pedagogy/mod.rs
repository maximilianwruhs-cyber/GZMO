//! Agentic Teacher pedagogy stack — learner profile, EDF, orchestrator, Trio Model.

pub mod edf;
pub mod graph;
pub mod intent;
pub mod learner;
pub mod orchestrator;
pub mod session;
pub mod trio;

pub use edf::{EdfRecord, EdfStore, StealthMetrics, ZpdPhase};
pub use graph::{PrerequisiteGraph, PrerequisiteNode};
pub use intent::{classify_intent, InteractionIntent};
pub use learner::{LearnerProfile, LearnerStore};
pub use orchestrator::{OrchestratorInput, OrchestratorOutput, PedagogyOrchestrator};
pub use session::PedagogySession;
pub use trio::{TrioMode, TrioState};
