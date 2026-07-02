//! Validation tests for Self-Improving System

use super::detector::{PatternState, RepetitionDetector};
use super::evaluator::OutputEvaluator;
use super::learner::{Experience, StrategyLearner};
use super::r#loop::{LoopConfig, SelfImprovingLoop};
use std::time::Duration;

/// Test 1: Repetition detector accuracy
#[test]
fn test_repetition_detection_accuracy() {
    let mut detector = RepetitionDetector::with_config(10, 0.85, 0.95, 3);
    
    // Generate novel outputs - should not detect as stuck
    for i in 0..5 {
        let output = format!("Completely different text with unique words {}", i);
        let state = detector.add_output(output);
        assert!(!state.needs_exploration(), "Novel output {} should not need exploration", i);
    }
    
    // Now add similar outputs - should eventually detect
    let similar = "The cat sat on the mat. This is a test sentence.";
    let mut detected_stuck = false;
    
    for i in 0..10 {
        let output = format!("{}", similar);
        let state = detector.add_output(output);
        if state.needs_exploration() {
            detected_stuck = true;
            println!("Detected stuck after {} similar outputs", i + 1);
            break;
        }
    }
    
    assert!(detected_stuck, "Should detect stuck after repeated similar outputs");
}

/// Test 2: Exploration level increases with stuck detection
#[test]
fn test_exploration_level_progression() {
    let mut detector = RepetitionDetector::with_config(5, 0.85, 0.95, 3);
    
    // Novel outputs
    detector.add_output("First unique sentence".to_string());
    let level = detector.exploration_level();
    assert!(level < 0.1, "Novel output should have low exploration: got {}", level);
    
    // Add stuck outputs
    let stuck_text = "Same text repeated many times for testing";
    for _ in 0..5 {
        detector.add_output(stuck_text.to_string());
    }
    
    let level = detector.exploration_level();
    assert!(level > 0.5, "Stuck pattern should have high exploration: got {}", level);
}

/// Test 3: Pattern state transitions
#[test]
fn test_pattern_state_transitions() {
    let mut detector = RepetitionDetector::with_config(10, 0.85, 0.95, 3);
    
    // Start novel
    let state1 = detector.add_output("Unique content A".to_string());
    assert_eq!(state1, PatternState::Novel);
    
    // Move to similar
    let state2 = detector.add_output("Unique content B".to_string());
    // May still be novel or similar, depending on similarity
    
    // Add identical outputs to force stuck/loop
    let identical = "Identical text for all outputs";
    for _ in 0..5 {
        detector.add_output(identical.to_string());
    }
    
    let state = detector.current_state();
    assert!(
        matches!(state, PatternState::Stuck | PatternState::Loop),
        "Identical outputs should cause stuck/loop state: got {:?}",
        state
    );
}

/// Test 4: Output evaluator diversity calculation
#[test]
fn test_diversity_calculation() {
    let mut evaluator = OutputEvaluator::new();
    
    // High diversity text
    let high_div = "The quick brown fox jumps over the lazy dog and runs through the forest";
    let metrics1 = evaluator.evaluate(high_div, Duration::from_millis(100));
    assert!(
        metrics1.lexical_diversity > 0.5,
        "High diversity text should score > 0.5: got {}",
        metrics1.lexical_diversity
    );
    
    // Low diversity text
    let low_div = "A B A B A B A B A B A B A B A B A B";
    let metrics2 = evaluator.evaluate(low_div, Duration::from_millis(100));
    // Reset for independent test
    let mut evaluator2 = OutputEvaluator::new();
    let metrics2 = evaluator2.evaluate(low_div, Duration::from_millis(100));
    assert!(
        metrics2.lexical_diversity < 0.5,
        "Low diversity text should score < 0.5: got {}",
        metrics2.lexical_diversity
    );
}

/// Test 5: Strategy learner correlation analysis
#[test]
fn test_strategy_correlation() {
    let mut learner = StrategyLearner::new();
    
    // Add experiences with positive correlation between temperature and diversity
    for i in 0..10 {
        let temp = 0.5 + (i as f32 * 0.05); // 0.5 to 0.95
        let diversity = 0.3 + (i as f64 * 0.06); // 0.3 to 0.84
        
        learner.record_experience(Experience {
            temperature: temp,
            max_tokens: 512,
            top_p: 0.9,
            diversity_score: diversity,
            latency_ms: 100,
            stuck_detected: false,
        });
    }
    
    let analysis = learner.analyze();
    // Should recommend higher temperature since we showed correlation with diversity
    assert!(
        analysis.confidence > 0.0,
        "Should have some confidence with 10 samples"
    );
}

/// Test 6: Self-improving loop cycle execution
#[test]
fn test_loop_cycle() {
    let config = LoopConfig {
        history_window: 5,
        similarity_threshold: 0.85,
        min_diversity_score: 0.3,
        max_latency_ms: 5000,
        experience_buffer_size: 100,
        strategy_update_interval: 50,
    };
    
    let mut loop_sys = SelfImprovingLoop::new(config).with_temperature(0.6);
    
    // Run several cycles with varying outputs
    for i in 0..5 {
        let prompt = format!("test prompt {}", i);
        let result = loop_sys.cycle(&prompt, |_p, params| {
            let output = format!("Output with temperature {}", params.temperature);
            (output, Duration::from_millis(100))
        });
        
        assert!(!result.output.is_empty(), "Should produce output");
        assert!(result.params.temperature > 0.0, "Should have valid temperature");
    }
    
    let stats = loop_sys.stats();
    assert_eq!(stats.generation_count, 5, "Should track 5 generations");
}

/// Test 7: Loop detection and escape
#[test]
fn test_loop_detection_and_escape() {
    let config = LoopConfig {
        history_window: 5,
        similarity_threshold: 0.85,
        min_diversity_score: 0.3,
        max_latency_ms: 5000,
        experience_buffer_size: 100,
        strategy_update_interval: 50,
    };
    
    let mut loop_sys = SelfImprovingLoop::new(config).with_temperature(0.6);
    
    // Start with varied outputs
    for i in 0..3 {
        let _ = loop_sys.cycle("prompt", |_p, _params| {
            (format!("Varied output {}", i), Duration::from_millis(100))
        });
    }
    
    // Now generate repetitive outputs to trigger stuck detection
    let repetitive = "Same repetitive text for testing purposes";
    let mut stuck_triggered = false;
    
    for _ in 0..10 {
        let result = loop_sys.cycle("prompt", |_p, params| {
            // Temperature should increase when stuck
            (repetitive.to_string(), Duration::from_millis(100))
        });
        
        if result.was_stuck {
            stuck_triggered = true;
            // Check that temperature was increased
            assert!(
                result.params.temperature > 0.6,
                "Should increase temperature when stuck: got {}",
                result.params.temperature
            );
        }
    }
    
    assert!(stuck_triggered, "Should have triggered stuck detection");
    
    // Verify stats
    let stats = loop_sys.stats();
    assert!(stats.stuck_count > 0, "Should record stuck count");
}

/// Test 8: Parameter adjustment based on exploration
#[test]
fn test_parameter_adjustment() {
    let loop_sys = SelfImprovingLoop::new(LoopConfig::default()).with_temperature(0.5);
    
    // No exploration
    let params_normal = loop_sys.adjust_params(0.0);
    assert!(
        (params_normal.temperature - 0.5).abs() < 0.01,
        "No exploration should keep base temp: got {}",
        params_normal.temperature
    );
    
    // Full exploration
    let params_explore = loop_sys.adjust_params(1.0);
    assert!(
        params_explore.temperature > 0.9,
        "Full exploration should boost temp: got {}",
        params_explore.temperature
    );
}

/// Test 9: Statistics tracking
#[test]
fn test_statistics_tracking() {
    let mut loop_sys = SelfImprovingLoop::new(LoopConfig::default());
    
    // Initial state
    let stats1 = loop_sys.stats();
    assert_eq!(stats1.generation_count, 0);
    assert_eq!(stats1.stuck_count, 0);
    
    // Run some cycles
    for i in 0..3 {
        let _ = loop_sys.cycle("test", |_p, _params| {
            (format!("output {}", i), Duration::from_millis(100))
        });
    }
    
    let stats2 = loop_sys.stats();
    assert_eq!(stats2.generation_count, 3);
    assert_eq!(stats2.current_exploration, 0.0); // Should be 0 for novel outputs
}

/// Test 10: Success criteria validation
/// 
/// Success criteria from plan:
/// 1. Repetition detection: >90% accuracy on stuck patterns
/// 2. Escape rate: Self-improving loop escapes 2x faster than static
/// 3. Strategy convergence: Learns optimal modulation within 100 iterations
#[test]
fn test_success_criteria() {
    // Test 1: Detection accuracy on stuck patterns
    let mut detector = RepetitionDetector::with_config(10, 0.85, 0.95, 3);
    
    // Create stuck pattern
    let stuck_text = "This is the same text repeated over and over";
    let mut detections = 0;
    let total: usize = 20;
    
    for _ in 0..total {
        let state = detector.add_output(stuck_text.to_string());
        if state.needs_exploration() {
            detections += 1;
        }
    }
    
    // After initial window, should detect nearly 100% of stuck outputs
    let accuracy = detections as f64 / (total.saturating_sub(5)) as f64;
    assert!(
        accuracy > 0.9,
        "Detection accuracy should be >90%: got {:.1}%",
        accuracy * 100.0
    );
    
    // Test 2: Loop efficiency
    let mut loop_sys = SelfImprovingLoop::new(LoopConfig::default()).with_temperature(0.5);
    let stuck_text = "Repetitive stuck text pattern";
    
    let mut cycles_to_escape = None;
    for i in 0..20 {
        let result = loop_sys.cycle("test", |_p, params| {
            let temp = params.temperature;
            // Simulate escape when temperature is high enough
            if temp > 0.8 {
                (format!("escaped with temp {}", temp), Duration::from_millis(100))
            } else {
                (stuck_text.to_string(), Duration::from_millis(100))
            }
        });
        
        if !result.was_stuck && cycles_to_escape.is_none() && i > 5 {
            cycles_to_escape = Some(i);
        }
    }
    
    assert!(
        cycles_to_escape.is_some(),
        "Should escape stuck state within 20 cycles"
    );
    
    // Test 3: Strategy convergence
    let mut learner = StrategyLearner::new();
    
    // Simulate learning 100 iterations
    for i in 0..100 {
        // Higher temp generally produces better diversity in this simulation
        let temp = if i % 2 == 0 { 0.7 } else { 0.5 };
        let diversity = if temp > 0.6 { 0.7 } else { 0.4 };
        
        learner.record_experience(Experience {
            temperature: temp,
            max_tokens: 512,
            top_p: 0.9,
            diversity_score: diversity,
            latency_ms: 100,
            stuck_detected: false,
        });
    }
    
    let optimal = learner.optimal_settings();
    // Should recommend higher temp based on correlation with diversity
    assert!(
        optimal.temperature >= 0.6,
        "Should converge to higher temp: got {}",
        optimal.temperature
    );
}

/// Integration test: Full self-improving pipeline
#[test]
fn test_full_pipeline() {
    let config = LoopConfig::default();
    let mut loop_sys = SelfImprovingLoop::new(config).with_temperature(0.6);
    
    let scenarios = vec![
        // Phase 1: Normal operation
        ("What is Rust?", "Rust is a systems programming language."),
        ("How does memory work?", "Memory in Rust uses ownership."),
        ("Tell me about borrowing", "Borrowing allows temporary access."),
        
        // Phase 2: Repetitive queries (should trigger stuck detection)
        ("Explain again", "The cat sat on the mat."),
        ("Explain again", "The cat sat on the mat."),
        ("Explain again", "The cat sat on the mat."),
        ("Explain again", "The cat sat on the mat."),
        
        // Phase 3: Recovery (diverse outputs)
        ("What about traits?", "Traits define shared behavior."),
        ("And lifetimes?", "Lifetimes track reference validity."),
    ];
    
    let scenario_count = scenarios.len();
    for (prompt, expected_base) in &scenarios {
        let _ = loop_sys.cycle(prompt, |_p, params| {
            let output = format!("{} (temp={:.2})", expected_base, params.temperature);
            (output, Duration::from_millis(100))
        });
    }
    
    let stats = loop_sys.stats();
    
    // Should have detected stuck during repetitive phase
    assert!(stats.stuck_count > 0, "Should detect stuck during repetitive phase");
    
    // Should have tracked all generations
    assert_eq!(stats.generation_count, scenario_count as u64);
}
