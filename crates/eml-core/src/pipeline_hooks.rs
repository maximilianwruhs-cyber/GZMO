//! Demo EML trees for a sigmoid and an exponential decay.
//!
//! These are crate-local examples. Nothing in `gzmo-core` honeypot / spark /
//! dream calls them. Evaluation is still `f64` via `ComplexBall.center.re`.

use crate::complex_ball::ComplexBall;
use crate::emitter::EmlExpr;
use crate::executor::{execute, ExecError};
use crate::synth;

/// Returns an EML symbolic expression for Honeypot Anomaly Confidence.
///
/// Formulation:
/// `confidence(score, threshold, k) = 1 / (1 + exp(-k * (score - threshold)))`
/// where:
/// - `v(0)` = raw anomaly score
/// - `v(1)` = anomaly threshold
/// - `v(2)` = steepness parameter k
pub fn honeypot_confidence_expr() -> EmlExpr {
    // delta = score - threshold = sub(v(0), v(1))
    let delta = synth::sub(EmlExpr::v(0), EmlExpr::v(1));
    // scaled = -k * delta = sub(0, k * delta)
    let k_delta = synth::mul(EmlExpr::v(2), delta);
    let neg_k_delta = synth::sub(EmlExpr::c(0.0), k_delta);
    // denom = 1 + exp(-k * delta)
    let exp_term = synth::exp(neg_k_delta);
    let denom = synth::add(EmlExpr::c(1.0), exp_term);
    // 1 / denom
    synth::div(EmlExpr::c(1.0), denom)
}

/// Evaluates Honeypot confidence using the compiled EML RPN engine.
pub fn eval_honeypot_confidence(score: f64, threshold: f64, k: f64) -> Result<f64, ExecError> {
    let expr = honeypot_confidence_expr();
    let prog = expr.compile();
    let inputs = [
        ComplexBall::from_real(score),
        ComplexBall::from_real(threshold),
        ComplexBall::from_real(k),
    ];
    let result = execute(&prog, &inputs)?;
    Ok(result.center.re)
}

/// Returns an EML symbolic expression for Spark/Dream memory decay.
///
/// Formulation:
/// `decay(M_0, lambda, elapsed) = M_0 * exp(-lambda * elapsed)`
/// where:
/// - `v(0)` = initial memory weight (M_0)
/// - `v(1)` = decay rate (lambda)
/// - `v(2)` = elapsed time in seconds
pub fn memory_decay_expr() -> EmlExpr {
    let exponent = synth::mul(EmlExpr::v(1), EmlExpr::v(2));
    let neg_exponent = synth::sub(EmlExpr::c(0.0), exponent);
    let decay_factor = synth::exp(neg_exponent);
    synth::mul(EmlExpr::v(0), decay_factor)
}

/// Evaluates Spark/Dream memory decay using compiled EML RPN.
pub fn eval_memory_decay(initial_weight: f64, lambda: f64, elapsed: f64) -> Result<f64, ExecError> {
    let expr = memory_decay_expr();
    let prog = expr.compile();
    let inputs = [
        ComplexBall::from_real(initial_weight),
        ComplexBall::from_real(lambda),
        ComplexBall::from_real(elapsed),
    ];
    let result = execute(&prog, &inputs)?;
    Ok(result.center.re)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_honeypot_confidence_at_threshold() {
        // At score == threshold (delta = 0), sigmoid is exactly 0.5
        let conf = eval_honeypot_confidence(10.0, 10.0, 1.0).unwrap();
        assert!((conf - 0.5).abs() < 1e-4, "Expected ~0.5 at threshold, got {}", conf);
    }

    #[test]
    fn test_honeypot_confidence_high_score() {
        // Above threshold, confidence approaches 1.0
        let conf = eval_honeypot_confidence(15.0, 10.0, 1.0).unwrap();
        assert!(conf > 0.99, "Expected high confidence, got {}", conf);
    }

    #[test]
    fn test_memory_decay_half_life() {
        // M_0 = 1.0, lambda = 0.693147 (ln(2)), elapsed = 1.0 -> expected M ~ 0.5
        let decay = eval_memory_decay(1.0, std::f64::consts::LN_2, 1.0).unwrap();
        assert!((decay - 0.5).abs() < 1e-4, "Expected ~0.5 at half-life, got {}", decay);
    }

    #[test]
    fn test_memory_decay_zero_elapsed() {
        let decay = eval_memory_decay(100.0, 0.05, 0.0).unwrap();
        assert!((decay - 100.0).abs() < 1e-4, "Expected 100.0, got {}", decay);
    }
}
