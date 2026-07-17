//! Trio Model interaction modes.

use serde::{Deserialize, Serialize};

/// Which leg of the Trio Model is active for the current exchange.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrioMode {
    /// Student ↔ GenAI (default teaching).
    #[default]
    StudentGenAi,
    /// Educator ↔ GenAI (meta-pedagogy, curriculum design).
    EducatorGenAi,
    /// Third Eye — observing the Student↔Educator or Student↔GenAI process.
    ThirdEye,
}

#[derive(Debug, Clone, Default)]
pub struct TrioState {
    pub mode: TrioMode,
}

impl TrioState {
    pub fn prompt_line(&self) -> &'static str {
        match self.mode {
            TrioMode::StudentGenAi => "Trio: Student–GenAI collaborative learning.",
            TrioMode::EducatorGenAi => "Trio: Educator–GenAI instructional design.",
            TrioMode::ThirdEye => "Trio: Third Eye — observe and reflect on the learning process.",
        }
    }
}
