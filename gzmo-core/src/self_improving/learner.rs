//! Strategy Learning — Adjusts modulation parameters based on outcomes

use std::collections::VecDeque;

/// A single experience entry
#[derive(Debug, Clone)]
pub struct Experience {
    pub temperature: f32,
    pub max_tokens: u32,
    pub top_p: f32,
    pub diversity_score: f64,
    pub latency_ms: u64,
    pub stuck_detected: bool,
}

/// Outcome of a generation
#[derive(Debug, Clone)]
pub struct Outcome {
    pub success: bool,
    pub diversity: f64,
    pub latency_ms: u64,
    pub repetition_detected: bool,
}

/// Learns optimal modulation strategy from experience
pub struct StrategyLearner {
    experiences: VecDeque<Experience>,
    max_experiences: usize,
    optimal_temperature: f32,
    exploration_rate: f32,
}

impl StrategyLearner {
    pub fn new() -> Self {
        Self {
            experiences: VecDeque::with_capacity(1000),
            max_experiences: 1000,
            optimal_temperature: 0.7,
            exploration_rate: 0.1,
        }
    }

    pub fn record_experience(&mut self, exp: Experience) {
        self.experiences.push_back(exp);
        if self.experiences.len() > self.max_experiences {
            self.experiences.pop_front();
        }
    }

    pub fn analyze(&self) -> StrategyAnalysis {
        if self.experiences.is_empty() {
            return StrategyAnalysis::default();
        }

        let successful: Vec<_> = self.experiences.iter()
            .filter(|e| !e.stuck_detected && e.diversity_score > 0.5)
            .collect();

        if successful.is_empty() {
            return StrategyAnalysis::default();
        }

        let avg_temp = successful.iter()
            .map(|e| e.temperature)
            .sum::<f32>() / successful.len() as f32;

        let diversity_by_temp: Vec<_> = successful.iter()
            .map(|e| (e.temperature, e.diversity_score))
            .collect();

        StrategyAnalysis {
            recommended_temperature: avg_temp,
            recommended_max_tokens: 512,
            confidence: successful.len() as f64 / self.experiences.len() as f64,
            diversity_correlation: self.correlation_temperature_diversity(),
        }
    }

    pub fn optimal_settings(&self) -> OptimalSettings {
        let analysis = self.analyze();
        
        OptimalSettings {
            temperature: analysis.recommended_temperature,
            max_tokens: analysis.recommended_max_tokens,
            top_p: 0.9,
        }
    }

    pub fn should_update(&self) -> bool {
        self.experiences.len() >= 50 && self.experiences.len() % 50 == 0
    }

    fn correlation_temperature_diversity(&self) -> f64 {
        if self.experiences.len() < 10 {
            return 0.0;
        }

        let temps: Vec<f64> = self.experiences.iter()
            .map(|e| e.temperature as f64)
            .collect();
        let diversities: Vec<f64> = self.experiences.iter()
            .map(|e| e.diversity_score)
            .collect();

        correlation(&temps, &diversities)
    }
}

impl Default for StrategyLearner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct StrategyAnalysis {
    pub recommended_temperature: f32,
    pub recommended_max_tokens: u32,
    pub confidence: f64,
    pub diversity_correlation: f64,
}

impl Default for StrategyAnalysis {
    fn default() -> Self {
        Self {
            recommended_temperature: 0.7,
            recommended_max_tokens: 512,
            confidence: 0.0,
            diversity_correlation: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OptimalSettings {
    pub temperature: f32,
    pub max_tokens: u32,
    pub top_p: f32,
}

fn correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.len() < 2 {
        return 0.0;
    }

    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;

    let numerator: f64 = x.iter().zip(y.iter())
        .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
        .sum();

    let sum_sq_x: f64 = x.iter().map(|xi| (xi - mean_x).powi(2)).sum();
    let sum_sq_y: f64 = y.iter().map(|yi| (yi - mean_y).powi(2)).sum();

    let denominator = (sum_sq_x * sum_sq_y).sqrt();
    
    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        assert!((correlation(&x, &y) - 1.0).abs() < 0.01);

        let y_neg = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        assert!((correlation(&x, &y_neg) - (-1.0)).abs() < 0.01);
    }

    #[test]
    fn test_empty_analysis() {
        let learner = StrategyLearner::new();
        let analysis = learner.analyze();
        assert_eq!(analysis.confidence, 0.0);
    }
}
