//! Self-Improving System — Closed-loop parameter optimization
//!
//! Implements repetition detection, output evaluation, strategy learning,
//! and the `SelfImprovingLoop` orchestration cycle.

pub mod detector;
pub mod evaluator;
pub mod learner;
pub mod r#loop;

#[cfg(test)]
mod tests;

pub use detector::{RepetitionDetector, PatternState};
pub use evaluator::{EvaluationMetrics, OutputEvaluator};
pub use learner::{StrategyLearner, Experience, Outcome};
pub use r#loop::{LoopConfig, ModulationParams, SelfImprovingLoop};

/// Re-export configuration types
pub use crate::strategies::{LlmParams, ModulationStrategy};

/// Default configuration for self-improving system
pub fn default_config() -> LoopConfig {
    LoopConfig {
        history_window: 10,
        similarity_threshold: 0.85,
        min_diversity_score: 0.3,
        max_latency_ms: 5000,
        experience_buffer_size: 1000,
        strategy_update_interval: 50,
    }
}

/// Stats summary for monitoring
#[derive(Debug, Clone)]
pub struct ImprovementStats {
    /// Total generations
    pub total_generations: u64,
    /// Times stuck detected
    pub stuck_count: u64,
    /// Times escaped stuck state
    pub escape_count: u64,
    /// Average diversity score
    pub avg_diversity: f64,
    /// Average latency
    pub avg_latency_ms: f64,
    /// Current strategy effectiveness
    pub strategy_effectiveness: f64,
}

impl Default for ImprovementStats {
    fn default() -> Self {
        Self {
            total_generations: 0,
            stuck_count: 0,
            escape_count: 0,
            avg_diversity: 0.0,
            avg_latency_ms: 0.0,
            strategy_effectiveness: 0.0,
        }
    }
}