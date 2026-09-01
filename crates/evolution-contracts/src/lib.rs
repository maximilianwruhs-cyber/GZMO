//! Pure domain contracts for GZMO evolution artifacts.
//!
//! No filesystem, network, database, or process side effects.

pub mod audit;
pub mod candidate;
pub mod evaluation;
pub mod policy;
pub mod promotion;

pub use audit::*;
pub use candidate::*;
pub use evaluation::*;
pub use policy::*;
pub use promotion::*;


pub const CANDIDATE_SCHEMA: &str = "gzmo.evolution.candidate/v1";
pub const ENVELOPE_SCHEMA: &str = "gzmo.evolution.envelope/v1";
pub const EVALUATION_SCHEMA: &str = "gzmo.evolution.evaluation/v1";
pub const PROMOTION_SCHEMA: &str = "gzmo.evolution.promotion/v1";
pub const AUDIT_SCHEMA: &str = "gzmo.evolution.audit/v1";
