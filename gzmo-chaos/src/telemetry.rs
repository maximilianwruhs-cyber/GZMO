//! Telemetry system for empirical validation of chaos parameters
//!
//! Tracks the actual impact of all "magic numbers" to enable
//! data-driven optimization and replace arbitrary constants.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, trace};

/// A tracked parameter with its metadata and history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedParameter {
    pub name: String,
    pub current_value: f64,
    pub default_value: f64,
    pub valid_range: (f64, f64),
    /// How this parameter was derived (formula, constant, empirical)
    pub derivation: ParameterDerivation,
    /// History of values and their measured outcomes
    pub observations: Vec<ParameterObservation>,
    /// Whether this parameter has been empirically validated
    pub validated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterDerivation {
    /// Aesthetic/arbitrary choice (needs validation)
    Arbitrary { rationale: String },
    /// Mathematically derived from first principles
    Mathematical { formula: String },
    /// Empirically tuned through A/B testing
    Empirical { experiment_id: String, optimal_value: f64 },
    /// Adaptive based on runtime conditions
    Adaptive { formula: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterObservation {
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    pub value: f64,
    /// Measured outcome metric (e.g., output diversity, latency)
    pub outcome: f64,
    /// Context when observation was made
    pub context: ObservationContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationContext {
    pub tick: u64,
    pub task_category: String,
    pub llm_params: crate::pulse::LlmParams,
    pub output_metrics: Option<OutputMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputMetrics {
    /// Lexical diversity of generated output
    pub lexical_diversity: f64,
    /// Response latency in milliseconds
    pub latency_ms: f64,
    /// Token count
    pub tokens_generated: u32,
    /// Task success (if applicable)
    pub task_success: Option<bool>,
}

/// Registry of all tracked parameters
pub struct TelemetryRegistry {
    parameters: Arc<Mutex<HashMap<String, TrackedParameter>>>,
    start_time: Instant,
}

impl TelemetryRegistry {
    pub fn new() -> Self {
        Self {
            parameters: Arc::new(Mutex::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Register a parameter for tracking
    pub fn register(
        &self,
        name: &str,
        current_value: f64,
        default_value: f64,
        valid_range: (f64, f64),
        derivation: ParameterDerivation,
    ) {
        let mut params = self.parameters.lock().unwrap();
        params.insert(
            name.to_string(),
            TrackedParameter {
                name: name.to_string(),
                current_value,
                default_value,
                valid_range,
                derivation,
                observations: Vec::new(),
                validated: false,
            },
        );
        trace!("Registered parameter: {}", name);
    }

    /// Update a parameter value and record observation
    pub fn record_observation(
        &self,
        name: &str,
        value: f64,
        outcome: f64,
        context: ObservationContext,
    ) {
        let mut params = self.parameters.lock().unwrap();
        if let Some(param) = params.get_mut(name) {
            param.current_value = value;
            param.observations.push(ParameterObservation {
                timestamp: Instant::now(),
                value,
                outcome,
                context,
            });
            
            // Mark as validated if we have enough observations
            if param.observations.len() >= 30 {
                param.validated = true;
            }
        }
    }

    /// Get parameter statistics
    pub fn get_statistics(&self, name: &str) -> Option<ParameterStatistics> {
        let params = self.parameters.lock().unwrap();
        let param = params.get(name)?;
        
        if param.observations.is_empty() {
            return None;
        }
        
        let values: Vec<f64> = param.observations.iter().map(|o| o.value).collect();
        let outcomes: Vec<f64> = param.observations.iter().map(|o| o.outcome).collect();
        
        Some(ParameterStatistics {
            name: name.to_string(),
            sample_size: param.observations.len(),
            value_mean: mean(&values),
            value_std: std_dev(&values),
            outcome_mean: mean(&outcomes),
            outcome_std: std_dev(&outcomes),
            correlation: correlation(&values, &outcomes),
            validated: param.validated,
        })
    }

    /// Generate report of all magic numbers and their validation status
    pub fn generate_validation_report(&self) -> ValidationReport {
        let params = self.parameters.lock().unwrap();
        let mut arbitrary = Vec::new();
        let mut mathematical = Vec::new();
        let mut empirical = Vec::new();
        let mut adaptive = Vec::new();
        let mut needs_validation = Vec::new();

        for (name, param) in params.iter() {
            match &param.derivation {
                ParameterDerivation::Arbitrary { .. } => {
                    arbitrary.push(name.clone());
                    if !param.validated {
                        needs_validation.push(name.clone());
                    }
                }
                ParameterDerivation::Mathematical { .. } => mathematical.push(name.clone()),
                ParameterDerivation::Empirical { .. } => empirical.push(name.clone()),
                ParameterDerivation::Adaptive { .. } => adaptive.push(name.clone()),
            }
        }

        ValidationReport {
            total_parameters: params.len(),
            arbitrary_count: arbitrary.len(),
            mathematical_count: mathematical.len(),
            empirical_count: empirical.len(),
            adaptive_count: adaptive.len(),
            needs_validation_count: needs_validation.len(),
            needs_validation_names: needs_validation,
            uptime: self.start_time.elapsed(),
        }
    }

    /// Export all data for external analysis
    pub fn export_data(&self) -> TelemetryExport {
        let params = self.parameters.lock().unwrap();
        TelemetryExport {
            parameters: params.values().cloned().collect(),
            exported_at: Instant::now(),
            uptime: self.start_time.elapsed(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ParameterStatistics {
    pub name: String,
    pub sample_size: usize,
    pub value_mean: f64,
    pub value_std: f64,
    pub outcome_mean: f64,
    pub outcome_std: f64,
    /// Pearson correlation coefficient between parameter and outcome
    pub correlation: f64,
    pub validated: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
    pub total_parameters: usize,
    pub arbitrary_count: usize,
    pub mathematical_count: usize,
    pub empirical_count: usize,
    pub adaptive_count: usize,
    pub needs_validation_count: usize,
    pub needs_validation_names: Vec<String>,
    pub uptime: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryExport {
    pub parameters: Vec<TrackedParameter>,
    #[serde(skip, default = "Instant::now")]
    pub exported_at: Instant,
    pub uptime: Duration,
}

/// Register all GZMO magic numbers for tracking
pub fn register_gzmo_parameters(registry: &TelemetryRegistry) {
    // PulseLoop constants
    registry.register(
        "tick_interval_ms",
        344.0, // 174 BPM
        344.0,
        (50.0, 5000.0),
        ParameterDerivation::Arbitrary {
            rationale: "174 BPM (Drum & Bass tempo) - aesthetic choice, not empirically validated".to_string(),
        },
    );

    registry.register(
        "llm_temp_min",
        0.3,
        0.3,
        (0.0, 1.0),
        ParameterDerivation::Arbitrary {
            rationale: "Low temperature floor - not tested against output quality".to_string(),
        },
    );

    registry.register(
        "llm_temp_max",
        1.2,
        1.2,
        (1.0, 2.0),
        ParameterDerivation::Arbitrary {
            rationale: "High temperature ceiling - not tested against output quality".to_string(),
        },
    );

    // Phase thresholds
    registry.register(
        "phase_idle_threshold",
        30.0,
        30.0,
        (0.0, 100.0),
        ParameterDerivation::Arbitrary {
            rationale: "Tension < 30 = Idle phase - arbitrary cutoff".to_string(),
        },
    );

    registry.register(
        "phase_build_threshold",
        70.0,
        70.0,
        (0.0, 100.0),
        ParameterDerivation::Arbitrary {
            rationale: "Tension < 70 = Build phase - arbitrary cutoff".to_string(),
        },
    );

    // Thought Cabinet
    registry.register(
        "thought_absorb_threshold",
        0.82, // 18% chance
        0.82,
        (0.0, 1.0),
        ParameterDerivation::Arbitrary {
            rationale: "18% absorption rate - aesthetic choice, not from memory research".to_string(),
        },
    );

    registry.register(
        "max_thought_slots",
        7.0,
        7.0,
        (1.0, 20.0),
        ParameterDerivation::Arbitrary {
            rationale: "7 slots - Disco Elysium reference, not from cognitive science".to_string(),
        },
    );

    // Confidence threshold
    registry.register(
        "honeypot_confidence_threshold",
        0.85,
        0.85,
        (0.0, 1.0),
        ParameterDerivation::Arbitrary {
            rationale: "0.85 confidence for honeypot - no calibration data".to_string(),
        },
    );

    info!("Registered {} GZMO parameters for empirical validation", 8);
}

// Statistical helper functions
fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let m = mean(values);
    let variance = values.iter().map(|v| {
        let diff = v - m;
        diff * diff
    }).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

fn correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.len() < 2 {
        return 0.0;
    }
    
    let mean_x = mean(x);
    let mean_y = mean(y);
    let std_x = std_dev(x);
    let std_y = std_dev(y);
    
    if std_x == 0.0 || std_y == 0.0 {
        return 0.0;
    }
    
    let covariance: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| {
        (xi - mean_x) * (yi - mean_y)
    }).sum::<f64>() / (x.len() - 1) as f64;
    
    covariance / (std_x * std_y)
}

impl Default for TelemetryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_calculation() {
        let registry = TelemetryRegistry::new();
        registry.register(
            "test_param",
            0.5,
            0.5,
            (0.0, 1.0),
            ParameterDerivation::Arbitrary {
                rationale: "Test".to_string(),
            },
        );

        // Record some observations
        for i in 0..10 {
            registry.record_observation(
                "test_param",
                0.5 + i as f64 * 0.05,
                100.0 - i as f64 * 5.0,
                ObservationContext {
                    tick: i as u64,
                    task_category: "test".to_string(),
                    llm_params: crate::pulse::LlmParams::default(),
                    output_metrics: None,
                },
            );
        }

        let stats = registry.get_statistics("test_param").unwrap();
        assert_eq!(stats.sample_size, 10);
        assert!(stats.correlation.abs() > 0.9); // Strong negative correlation
    }
}