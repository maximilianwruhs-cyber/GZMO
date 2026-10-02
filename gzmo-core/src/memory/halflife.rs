//! # Metabolic Half-Life Scoring
//!
//! Pure, deterministic metabolic scoring function for the Living Vault.
//! Provides exponential decay retention modeling modulated by MemRL-Q utility
//! and logarithmic recall reinforcement.
//!
//! ## Mathematical Formulation
//!
//! - Retention factor:
//!   `retention = exp(-ln(2) * age_days / half_life[decay_class])`
//! - Utility boost:
//!   `(1 + utility_gain * utility_q)`
//! - Recall boost:
//!   `(1 + recall_boost_per_hit * ln(1 + recall_count))`
//! - Combined metabolic score:
//!   `score = clamp01(retention * (1 + utility_gain * utility_q) * (1 + recall_boost_per_hit * ln(1 + recall_count)))`
//!
//! Also provides the HOPE/Titan momentum update reference for exponential smoothing:
//! `momentum_update(prev, sample, alpha) = (1 - alpha) * prev + alpha * sample`.

use std::collections::BTreeMap;
use std::f64::consts::LN_2;

use serde::{Deserialize, Serialize};

pub use crate::types::DecayClass;

/// Configuration parameters for metabolic half-life decay and reinforcement boosts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HalflifeParams {
    /// Half-life in days per decay class.
    pub half_life_days: BTreeMap<DecayClass, f64>,
    /// Multiplier for MemRL-Q utility gain (default ~0.5).
    pub utility_gain: f64,
    /// Multiplier for log-scaled lifetime recall count boost (default ~0.1).
    pub recall_boost_per_hit: f64,
}

impl Default for HalflifeParams {
    /// Canonical default half-life durations:
    /// - CuratedVault: 90 days
    /// - Structural: 60 days
    /// - SessionDistill: 21 days
    /// - Episodic: 7 days
    ///
    /// Reinforcement parameters:
    /// - utility_gain: 0.5
    /// - recall_boost_per_hit: 0.1
    fn default() -> Self {
        let mut half_life_days = BTreeMap::new();
        half_life_days.insert(DecayClass::CuratedVault, 90.0);
        half_life_days.insert(DecayClass::Structural, 60.0);
        half_life_days.insert(DecayClass::SessionDistill, 21.0);
        half_life_days.insert(DecayClass::Episodic, 7.0);

        Self {
            half_life_days,
            utility_gain: 0.5,
            recall_boost_per_hit: 0.1,
        }
    }
}

impl HalflifeParams {
    /// Retrieve the half-life in days for the given decay class.
    ///
    /// If the decay class is explicitly registered in `half_life_days`, that value is returned.
    /// Otherwise, falls back to `decay_class.half_life_days()`.
    pub fn half_life_for(&self, decay_class: DecayClass) -> f64 {
        self.half_life_days
            .get(&decay_class)
            .copied()
            .unwrap_or_else(|| decay_class.half_life_days())
    }
}

/// Calculate the pure retention factor in `[0.0..=1.0]`: `exp(-ln(2) * age_days / half_life_days)`.
///
/// Boundary behaviors:
/// - `age_days <= 0.0` yields exactly `1.0`.
/// - `half_life_days` infinite yields `1.0`.
/// - `half_life_days <= 0.0` or NaN yields `0.0`.
pub fn retention(age_days: f64, half_life_days: f64) -> f64 {
    if half_life_days.is_infinite() {
        return 1.0;
    }
    if half_life_days <= 0.0 || half_life_days.is_nan() {
        return 0.0;
    }
    let age = age_days.max(0.0);
    if age == 0.0 {
        return 1.0;
    }
    (-LN_2 * age / half_life_days).exp()
}

/// Clamp a floating point score into the closed unit interval `[0.0..=1.0]`.
/// Non-finite or NaN inputs map deterministically to `0.0`.
pub fn clamp01(val: f64) -> f64 {
    if val.is_nan() || val <= 0.0 {
        0.0
    } else if val >= 1.0 {
        1.0
    } else {
        val
    }
}

/// Compute the un-clamped raw metabolic score:
/// `retention * (1 + utility_gain * utility_q) * (1 + recall_boost_per_hit * ln(1 + recall_count))`
pub fn raw_metabolic_score(
    age_days: f64,
    decay_class: DecayClass,
    utility_q: f64,
    recall_count: u32,
    params: &HalflifeParams,
) -> f64 {
    let half_life = params.half_life_for(decay_class);
    let ret = retention(age_days, half_life);

    let u_gain = params.utility_gain.max(0.0);
    let u_q = utility_q.max(0.0);
    let utility_factor = 1.0 + u_gain * u_q;

    let r_boost = params.recall_boost_per_hit.max(0.0);
    let recall_factor = 1.0 + r_boost * (recall_count as f64).ln_1p();

    ret * utility_factor * recall_factor
}

/// Deterministic metabolic score in `[0..=1]`.
///
/// # Arguments
/// - `age_days`: Elapsed time in days since fact promotion or origin.
/// - `decay_class`: Memory tier governing base decay half-life.
/// - `utility_q`: MemRL-Q style utility, typically in `[0..=1]`.
/// - `recall_count`: Lifetime successful recalls.
/// - `params`: Decay parameters configuring half-lives and gain coefficients.
pub fn metabolic_score(
    age_days: f64,
    decay_class: DecayClass,
    utility_q: f64,
    recall_count: u32,
    params: &HalflifeParams,
) -> f64 {
    clamp01(raw_metabolic_score(
        age_days,
        decay_class,
        utility_q,
        recall_count,
        params,
    ))
}

/// HOPE/Titan momentum reference — exponential smoothing with forgetting gate `alpha` in `(0, 1]`.
///
/// Computes `(1 - alpha) * prev_m + alpha * sample`.
///
/// Boundary conditions:
/// - `alpha <= 0.0` (or NaN) returns `prev_m`.
/// - `alpha >= 1.0` returns `sample`.
pub fn momentum_update(prev_m: f64, sample: f64, alpha: f64) -> f64 {
    if alpha.is_nan() || alpha <= 0.0 {
        prev_m
    } else if alpha >= 1.0 {
        sample
    } else {
        (1.0 - alpha) * prev_m + alpha * sample
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monotonicity_score_decreases_as_age_grows() {
        let params = HalflifeParams::default();
        let ages = [0.0, 1.0, 7.0, 14.0, 30.0, 60.0, 90.0, 180.0, 365.0];

        for &class in &[
            DecayClass::CuratedVault,
            DecayClass::Structural,
            DecayClass::SessionDistill,
            DecayClass::Episodic,
        ] {
            let mut prev_score = metabolic_score(ages[0], class, 0.0, 0, &params);
            assert_eq!(prev_score, 1.0, "Score at age 0 with 0 boosts must be 1.0");

            for &age in &ages[1..] {
                let score = metabolic_score(age, class, 0.0, 0, &params);
                assert!(
                    score < prev_score,
                    "Monotonicity failed for {:?} at age {}: prev={}, current={}",
                    class,
                    age,
                    prev_score,
                    score
                );
                prev_score = score;
            }
        }
    }

    #[test]
    fn class_ordering_at_equal_age() {
        let params = HalflifeParams::default();
        // At equal positive age and equal baseline (no boost saturation):
        // CuratedVault (90d) > Structural (60d) > SessionDistill (21d) > Episodic (7d)
        let test_ages = [1.0, 7.0, 14.0, 30.0, 60.0];

        for &age in &test_ages {
            let score_curated = metabolic_score(age, DecayClass::CuratedVault, 0.0, 0, &params);
            let score_structural = metabolic_score(age, DecayClass::Structural, 0.0, 0, &params);
            let score_distill = metabolic_score(age, DecayClass::SessionDistill, 0.0, 0, &params);
            let score_episodic = metabolic_score(age, DecayClass::Episodic, 0.0, 0, &params);

            assert!(
                score_curated > score_structural,
                "CuratedVault ({}) must exceed Structural ({}) at age {}",
                score_curated,
                score_structural,
                age
            );
            assert!(
                score_structural > score_distill,
                "Structural ({}) must exceed SessionDistill ({}) at age {}",
                score_structural,
                score_distill,
                age
            );
            assert!(
                score_distill > score_episodic,
                "SessionDistill ({}) must exceed Episodic ({}) at age {}",
                score_distill,
                score_episodic,
                age
            );

            // Also holds for raw_metabolic_score under non-zero boosts:
            let raw_curated = raw_metabolic_score(age, DecayClass::CuratedVault, 0.5, 3, &params);
            let raw_structural = raw_metabolic_score(age, DecayClass::Structural, 0.5, 3, &params);
            let raw_distill = raw_metabolic_score(age, DecayClass::SessionDistill, 0.5, 3, &params);
            let raw_episodic = raw_metabolic_score(age, DecayClass::Episodic, 0.5, 3, &params);

            assert!(raw_curated > raw_structural);
            assert!(raw_structural > raw_distill);
            assert!(raw_distill > raw_episodic);
        }
    }

    #[test]
    fn utility_and_recall_boosts_are_non_negative_effects() {
        let params = HalflifeParams::default();
        let age = 30.0;
        let class = DecayClass::CuratedVault;

        let baseline = metabolic_score(age, class, 0.0, 0, &params);

        // Utility boost tests
        let with_util_low = metabolic_score(age, class, 0.2, 0, &params);
        let with_util_high = metabolic_score(age, class, 0.8, 0, &params);
        assert!(
            with_util_low >= baseline,
            "Low utility should not decrease score"
        );
        assert!(
            with_util_high > with_util_low,
            "Higher utility should increase score"
        );

        // Recall boost tests
        let with_recall_1 = metabolic_score(age, class, 0.0, 1, &params);
        let with_recall_10 = metabolic_score(age, class, 0.0, 10, &params);
        assert!(
            with_recall_1 >= baseline,
            "Recall count 1 should not decrease score"
        );
        assert!(
            with_recall_10 > with_recall_1,
            "Higher recall count should increase score"
        );

        // Both combined
        let combined = metabolic_score(age, class, 0.5, 5, &params);
        assert!(combined > with_util_low);
        assert!(combined > with_recall_1);
    }

    #[test]
    fn age_zero_retention_factor_is_exactly_one() {
        let params = HalflifeParams::default();

        for &class in &[
            DecayClass::CuratedVault,
            DecayClass::Structural,
            DecayClass::SessionDistill,
            DecayClass::Episodic,
        ] {
            let half_life = params.half_life_for(class);
            let ret = retention(0.0, half_life);
            assert_eq!(
                ret, 1.0,
                "Retention at age 0 must be exactly 1.0 for {:?}",
                class
            );

            // With zero boosts, metabolic score at age 0 is exactly 1.0
            let score_zero_boost = metabolic_score(0.0, class, 0.0, 0, &params);
            assert_eq!(score_zero_boost, 1.0);

            // Raw metabolic score at age 0 equals boost terms only
            let u_q = 0.4;
            let recall = 3;
            let raw = raw_metabolic_score(0.0, class, u_q, recall, &params);
            let expected_boost = (1.0 + params.utility_gain * u_q)
                * (1.0 + params.recall_boost_per_hit * (recall as f64).ln_1p());
            assert_eq!(
                raw.to_bits(),
                expected_boost.to_bits(),
                "Raw score at age 0 must bit-identically match boost terms only"
            );
        }
    }

    #[test]
    fn determinism_same_inputs_twice_bit_identical() {
        let params = HalflifeParams::default();

        let s1 = metabolic_score(24.5, DecayClass::SessionDistill, 0.65, 4, &params);
        let s2 = metabolic_score(24.5, DecayClass::SessionDistill, 0.65, 4, &params);
        assert_eq!(
            s1.to_bits(),
            s2.to_bits(),
            "Determinism violation in metabolic_score"
        );

        let r1 = raw_metabolic_score(45.123, DecayClass::CuratedVault, 0.9, 12, &params);
        let r2 = raw_metabolic_score(45.123, DecayClass::CuratedVault, 0.9, 12, &params);
        assert_eq!(
            r1.to_bits(),
            r2.to_bits(),
            "Determinism violation in raw_metabolic_score"
        );

        let m1 = momentum_update(0.42, 0.88, 0.15);
        let m2 = momentum_update(0.42, 0.88, 0.15);
        assert_eq!(
            m1.to_bits(),
            m2.to_bits(),
            "Determinism violation in momentum_update"
        );
    }

    #[test]
    fn momentum_update_boundaries_alpha_zero_and_one() {
        let prev = 0.3541;
        let sample = 0.9127;

        // alpha = 0.0 -> returns prev
        assert_eq!(
            momentum_update(prev, sample, 0.0).to_bits(),
            prev.to_bits(),
            "alpha=0 must return prev"
        );

        // alpha < 0.0 -> returns prev
        assert_eq!(
            momentum_update(prev, sample, -0.5).to_bits(),
            prev.to_bits(),
            "negative alpha must return prev"
        );

        // alpha = 1.0 -> returns sample
        assert_eq!(
            momentum_update(prev, sample, 1.0).to_bits(),
            sample.to_bits(),
            "alpha=1 must return sample"
        );

        // alpha > 1.0 -> returns sample
        assert_eq!(
            momentum_update(prev, sample, 1.5).to_bits(),
            sample.to_bits(),
            "alpha > 1 must return sample"
        );

        // Intermediate value
        let smoothed = momentum_update(10.0, 20.0, 0.3);
        let expected = 0.7 * 10.0 + 0.3 * 20.0; // 7.0 + 6.0 = 13.0
        assert!((smoothed - expected).abs() < 1e-12);

        // NaN handling
        assert_eq!(
            momentum_update(prev, sample, f64::NAN).to_bits(),
            prev.to_bits(),
            "NaN alpha must return prev"
        );
    }

    #[test]
    fn retention_exact_half_at_half_life() {
        let half_life = 90.0;
        let ret = retention(half_life, half_life);
        assert!(
            (ret - 0.5).abs() < 1e-12,
            "Retention at 1 half-life must be 0.5"
        );

        let ret_2 = retention(2.0 * half_life, half_life);
        assert!(
            (ret_2 - 0.25).abs() < 1e-12,
            "Retention at 2 half-lives must be 0.25"
        );
    }

    #[test]
    fn clamp01_boundary_and_nan_behavior() {
        assert_eq!(clamp01(0.0), 0.0);
        assert_eq!(clamp01(1.0), 1.0);
        assert_eq!(clamp01(-0.5), 0.0);
        assert_eq!(clamp01(1.5), 1.0);
        assert_eq!(clamp01(f64::NAN), 0.0);
        assert_eq!(clamp01(0.5), 0.5);
    }

    #[test]
    fn negative_age_clamped_to_zero() {
        let params = HalflifeParams::default();
        let score_neg = metabolic_score(-5.0, DecayClass::CuratedVault, 0.0, 0, &params);
        let score_zero = metabolic_score(0.0, DecayClass::CuratedVault, 0.0, 0, &params);
        assert_eq!(score_neg, score_zero);
    }

    #[test]
    fn infinite_half_life_retention_is_one() {
        let ret = retention(1000.0, f64::INFINITY);
        assert_eq!(ret, 1.0);
    }

    #[test]
    fn halflife_params_serde_roundtrip() {
        let params = HalflifeParams::default();
        let json = serde_json::to_string(&params).expect("Serialization failed");
        let decoded: HalflifeParams = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(params, decoded);
    }
}
