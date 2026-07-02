//! Modulation strategies for LLM parameter control.
//!
//! Shared types used by the self-improving loop and gateway chaos overrides.

use serde::{Deserialize, Serialize};

/// LLM sampling parameters for a single generation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LlmParams {
    pub temperature: f32,
    pub max_tokens: u32,
    pub top_p: f32,
}

impl Default for LlmParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 512,
            top_p: 0.9,
        }
    }
}

/// Named modulation strategy with baseline and exploration bounds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModulationStrategy {
    pub name: String,
    pub baseline: LlmParams,
    pub exploration_boost: f32,
    pub min_temperature: f32,
    pub max_temperature: f32,
}

impl Default for ModulationStrategy {
    fn default() -> Self {
        Self {
            name: "balanced".to_string(),
            baseline: LlmParams::default(),
            exploration_boost: 0.35,
            min_temperature: 0.3,
            max_temperature: 1.5,
        }
    }
}

impl ModulationStrategy {
    /// Apply exploration level (0.0–1.0) on top of baseline temperature.
    pub fn params_at_exploration(&self, exploration: f64) -> LlmParams {
        let boost = (exploration.clamp(0.0, 1.0) * f64::from(self.exploration_boost)) as f32;
        let temperature = (self.baseline.temperature + boost)
            .clamp(self.min_temperature, self.max_temperature);
        LlmParams {
            temperature,
            max_tokens: self.baseline.max_tokens,
            top_p: if exploration > 0.5 {
                0.95
            } else {
                self.baseline.top_p
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exploration_raises_temperature() {
        let strategy = ModulationStrategy::default();
        let low = strategy.params_at_exploration(0.0);
        let high = strategy.params_at_exploration(1.0);
        assert!(high.temperature > low.temperature);
    }
}
