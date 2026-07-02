//! Self-Improving Loop — Orchestrates the full feedback cycle

use super::detector::{PatternState, RepetitionDetector};
use super::evaluator::{EvaluationMetrics, OutputEvaluator};
use super::learner::{Experience, Outcome, StrategyLearner};
use std::time::Duration;

/// Configuration for the self-improving loop
#[derive(Debug, Clone)]
pub struct LoopConfig {
    pub history_window: usize,
    pub similarity_threshold: f64,
    pub min_diversity_score: f64,
    pub max_latency_ms: u64,
    pub experience_buffer_size: usize,
    pub strategy_update_interval: usize,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            history_window: 10,
            similarity_threshold: 0.85,
            min_diversity_score: 0.3,
            max_latency_ms: 5000,
            experience_buffer_size: 1000,
            strategy_update_interval: 50,
        }
    }
}

/// Core modulation parameters
#[derive(Debug, Clone, Copy)]
pub struct ModulationParams {
    pub temperature: f32,
    pub max_tokens: u32,
    pub top_p: f32,
}

impl Default for ModulationParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 512,
            top_p: 0.9,
        }
    }
}

/// State of the self-improving loop
#[derive(Debug, Clone)]
pub struct LoopState {
    pub generation_count: u64,
    pub stuck_count: u64,
    pub escape_count: u64,
    pub last_params: ModulationParams,
    pub last_metrics: Option<EvaluationMetrics>,
    pub current_pattern: PatternState,
    pub exploration_level: f64,
}

impl LoopState {
    pub fn new() -> Self {
        Self {
            generation_count: 0,
            stuck_count: 0,
            escape_count: 0,
            last_params: ModulationParams::default(),
            last_metrics: None,
            current_pattern: PatternState::Novel,
            exploration_level: 0.0,
        }
    }
}

impl Default for LoopState {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a single generation cycle
#[derive(Debug, Clone)]
pub struct CycleResult {
    pub output: String,
    pub params: ModulationParams,
    pub metrics: EvaluationMetrics,
    pub pattern_state: PatternState,
    pub was_stuck: bool,
    pub exploration_applied: bool,
}

/// Self-improving feedback loop
pub struct SelfImprovingLoop {
    pub config: LoopConfig,
    pub detector: RepetitionDetector,
    pub evaluator: OutputEvaluator,
    pub learner: StrategyLearner,
    pub state: LoopState,
    pub params: ModulationParams,
    base_temperature: f32,
}

impl SelfImprovingLoop {
    pub fn new(config: LoopConfig) -> Self {
        let base_temp = 0.7;
        Self {
            config: config.clone(),
            detector: RepetitionDetector::with_config(
                config.history_window,
                config.similarity_threshold,
                0.95,
                3,
            ),
            evaluator: OutputEvaluator::new(),
            learner: StrategyLearner::new(),
            state: LoopState::new(),
            params: ModulationParams::default(),
            base_temperature: base_temp,
        }
    }

    /// Configure initial temperature
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.base_temperature = temp;
        self.params.temperature = temp;
        self
    }

    /// Execute one generation cycle
    pub fn cycle<F>(&mut self, prompt: &str, mut generate: F) -> CycleResult
    where
        F: FnMut(&str, ModulationParams) -> (String, Duration),
    {
        // 1. Detect current pattern state
        let pattern_state = self.detector.current_state();
        let was_stuck = pattern_state.needs_exploration();
        
        if was_stuck {
            self.state.stuck_count += 1;
        }

        // 2. Adjust modulation based on pattern state
        let exploration = pattern_state.exploration_level();
        self.params = self.adjust_params(exploration);

        // 3. Generate output
        let start = std::time::Instant::now();
        let (output, _latency) = generate(prompt, self.params);
        let actual_latency = start.elapsed();

        // 4. Evaluate output
        let metrics = self.evaluator.evaluate(&output, actual_latency);

        // 5. Update detector
        let new_state = self.detector.add_output(output.clone());
        let escaped_stuck = was_stuck && !new_state.needs_exploration();
        
        if escaped_stuck {
            self.state.escape_count += 1;
        }

        // 6. Record experience for learning
        let experience = Experience {
            temperature: self.params.temperature,
            max_tokens: self.params.max_tokens,
            top_p: self.params.top_p,
            diversity_score: metrics.lexical_diversity,
            latency_ms: metrics.latency_ms,
            stuck_detected: was_stuck,
        };
        self.learner.record_experience(experience);

        // 7. Update state
        self.state.generation_count += 1;
        self.state.last_params = self.params;
        self.state.last_metrics = Some(metrics);
        self.state.current_pattern = new_state;
        self.state.exploration_level = exploration;

        CycleResult {
            output,
            params: self.params,
            metrics,
            pattern_state: new_state,
            was_stuck,
            exploration_applied: exploration > 0.0,
        }
    }

    /// Adjust parameters based on exploration level (0.0-1.0)
    pub fn adjust_params(&self, exploration: f64) -> ModulationParams {
        let temp_boost = (exploration * 0.5) as f32; // Max +0.5 temperature when fully exploring
        let new_temp = (self.base_temperature + temp_boost).min(1.5);
        
        // When stuck/exploring, increase max_tokens for more variety
        let token_boost = if exploration > 0.5 { 256 } else { 0 };
        
        ModulationParams {
            temperature: new_temp,
            max_tokens: self.params.max_tokens + token_boost,
            top_p: 0.95, // More diverse sampling when exploring
        }
    }

    /// Get current statistics
    pub fn stats(&self) -> LoopStats {
        LoopStats {
            generation_count: self.state.generation_count,
            stuck_count: self.state.stuck_count,
            escape_count: self.state.escape_count,
            escape_rate: if self.state.stuck_count > 0 {
                self.state.escape_count as f64 / self.state.stuck_count as f64
            } else {
                0.0
            },
            current_exploration: self.state.exploration_level,
            pattern_state: self.state.current_pattern,
        }
    }

    /// Get optimal settings from learner
    pub fn optimal_settings(&self) -> ModulationParams {
        let s = self.learner.optimal_settings();
        ModulationParams {
            temperature: s.temperature,
            max_tokens: s.max_tokens,
            top_p: s.top_p,
        }
    }

    /// Check if should update strategy
    pub fn should_update(&self) -> bool {
        self.learner.should_update()
    }

    /// Apply learned optimal settings
    pub fn apply_optimal(&mut self) {
        self.params = self.optimal_settings();
    }

    /// Reset the loop state
    pub fn reset(&mut self) {
        self.state = LoopState::new();
        self.params = ModulationParams::default();
    }
}

/// Statistics for monitoring
#[derive(Debug, Clone)]
pub struct LoopStats {
    pub generation_count: u64,
    pub stuck_count: u64,
    pub escape_count: u64,
    pub escape_rate: f64,
    pub current_exploration: f64,
    pub pattern_state: PatternState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let params = ModulationParams::default();
        assert!(params.temperature > 0.0 && params.temperature <= 1.0);
        assert!(params.max_tokens > 0);
    }

    #[test]
    fn test_cycle_execution() {
        let mut loop_sys = SelfImprovingLoop::new(LoopConfig::default());
        
        let result = loop_sys.cycle("test prompt", |_prompt, params| {
            let temp = params.temperature;
            let output = format!("Output at temp {}", temp);
            (output, Duration::from_millis(100))
        });

        assert!(!result.output.is_empty());
        assert_eq!(loop_sys.state.generation_count, 1);
    }

    #[test]
    fn test_adjust_params() {
        let loop_sys = SelfImprovingLoop::new(LoopConfig::default())
            .with_temperature(0.5);
        
        // No exploration
        let params_normal = loop_sys.adjust_params(0.0);
        assert!((params_normal.temperature - 0.5).abs() < 0.01);
        
        // Full exploration
        let params_explore = loop_sys.adjust_params(1.0);
        assert!(params_explore.temperature > 0.9);
    }
}
