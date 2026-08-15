//! # eml-core
//!
//! Exp-Minus-Log symbolic computation engine.
//!
//! The EML operator `eml(x, y) = exp(x) - ln(y)` is a universal primitive
//! from which all elementary functions can be synthesised (analogous to NAND
//! in boolean logic).  This crate provides:
//!
//! - [`ComplexBall`] with a first-order (non-rigorous) error radius
//! - [`EmlExpr`] tree, constant-fold, and `Display`/parse
//! - RPN compiler + zero-copy executor
//!
//! Workspace R&D: nothing in `gzmo-core` calls this crate. [`pipeline_hooks`]
//! are local demos, not living memory organs.

pub mod complex_ball;
pub mod emitter;
pub mod executor;
pub mod pipeline_hooks;
pub mod rpn;

pub use complex_ball::{ComplexBall, EmlError, RADIUS_IS_RIGOROUS};
pub use emitter::{EmlExpr, ParseError};
pub use executor::{execute, ExecError};
pub use pipeline_hooks::{
    eval_honeypot_confidence, eval_memory_decay, honeypot_confidence_expr, memory_decay_expr,
};
pub use rpn::{RpnInstruction, RpnProgram};

// ---------------------------------------------------------------------------
// High-level convenience: synthesise common functions as EML expression trees.
// ---------------------------------------------------------------------------

pub mod synth {
    use crate::emitter::EmlExpr;

    /// `exp(x) = eml(x, 1)`
    pub fn exp(x: EmlExpr) -> EmlExpr {
        EmlExpr::eml(x, EmlExpr::c(1.0))
    }

    /// `ln(x) = eml(1, eml(eml(1, x), 1))`
    pub fn ln(x: EmlExpr) -> EmlExpr {
        let a = EmlExpr::eml(EmlExpr::c(1.0), x);       // eml(1, x)  = e - ln(x)
        let b = EmlExpr::eml(a, EmlExpr::c(1.0));        // eml(a, 1) = exp(a) = e^e / x
        EmlExpr::eml(EmlExpr::c(1.0), b) // eml(1, b)  = ln(x)
    }

    /// `x - y = eml(ln(x), exp(y))`
    pub fn sub(x: EmlExpr, y: EmlExpr) -> EmlExpr {
        EmlExpr::eml(ln(x), exp(y))
    }

    /// `-x = 0 - x`
    pub fn neg(x: EmlExpr) -> EmlExpr {
        sub(EmlExpr::c(0.0), x)
    }

    /// `x + y = eml(ln(x), exp(-y))`
    pub fn add(x: EmlExpr, y: EmlExpr) -> EmlExpr {
        EmlExpr::eml(ln(x), exp(neg(y)))
    }

    /// `x * y = exp(ln(x) + ln(y))`
    pub fn mul(x: EmlExpr, y: EmlExpr) -> EmlExpr {
        let ln_sum = add(ln(x), ln(y));
        EmlExpr::eml(ln_sum, EmlExpr::c(1.0))
    }

    /// `x / y = exp(ln(x) - ln(y))`
    pub fn div(x: EmlExpr, y: EmlExpr) -> EmlExpr {
        let ln_diff = sub(ln(x), ln(y));
        EmlExpr::eml(ln_diff, EmlExpr::c(1.0))
    }

    /// `sqrt(x) = exp(ln(x) / 2)`
    pub fn sqrt(x: EmlExpr) -> EmlExpr {
        let half = div(EmlExpr::c(1.0), EmlExpr::c(2.0));  // 0.5
        let ln_half = mul(ln(x), half);                     // ln(x) * 0.5
        EmlExpr::eml(ln_half, EmlExpr::c(1.0))               // exp(ln(x)/2)
    }

    /// `square(x) = x * x`
    pub fn square(x: EmlExpr) -> EmlExpr {
        mul(x.clone(), x)
    }

    /// `pow(x, y) = exp(ln(x) * y)`
    pub fn pow(x: EmlExpr, y: EmlExpr) -> EmlExpr {
        let lnx_mul_y = mul(ln(x), y);
        EmlExpr::eml(lnx_mul_y, EmlExpr::c(1.0))
    }

    /// `inv(x) = 1/x`
    pub fn inv(x: EmlExpr) -> EmlExpr {
        div(EmlExpr::c(1.0), x)
    }
}

#[cfg(test)]
mod tests {
    use crate::{execute, synth, ComplexBall, EmlExpr};

    fn eval(expr: EmlExpr, args: &[f64]) -> ComplexBall {
        let prog = expr.compile();
        let balls: Vec<_> = args.iter().map(|&v| ComplexBall::from_real(v)).collect();
        execute(&prog, &balls).unwrap()
    }

    #[test]
    fn test_exp() {
        let r = eval(synth::exp(EmlExpr::v(0)), &[2.0]);
        assert!((r.center.re - 2.0f64.exp()).abs() < 1e-12);
    }

    #[test]
    fn test_ln() {
        let r = eval(synth::ln(EmlExpr::v(0)), &[5.0]);
        assert!((r.center.re - 5.0f64.ln()).abs() < 1e-12);
    }

    #[test]
    fn test_add() {
        let r = eval(synth::add(EmlExpr::v(0), EmlExpr::v(1)), &[42.0, 7.0]);
        assert!((r.center.re - 49.0).abs() < 1e-12);
    }

    #[test]
    fn test_sub() {
        let r = eval(synth::sub(EmlExpr::v(0), EmlExpr::v(1)), &[100.0, 23.0]);
        assert!((r.center.re - 77.0).abs() < 1e-12);
    }

    #[test]
    fn test_mul() {
        let r = eval(synth::mul(EmlExpr::v(0), EmlExpr::v(1)), &[6.0, 7.0]);
        assert!((r.center.re - 42.0).abs() < 1e-9);
    }

    #[test]
    fn test_div() {
        let r = eval(synth::div(EmlExpr::v(0), EmlExpr::v(1)), &[100.0, 4.0]);
        assert!((r.center.re - 25.0).abs() < 1e-9);
    }

    #[test]
    fn test_precision_drift_reported() {
        let r = eval(synth::exp(EmlExpr::v(0)), &[1.0]);
        assert!(r.radius > 0.0);
        assert!(r.radius < 1e-14);
    }

    #[test]
    fn test_sqrt() {
        let r = eval(synth::sqrt(EmlExpr::v(0)), &[16.0]);
        assert!((r.center.re - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_pow() {
        let r = eval(synth::pow(EmlExpr::v(0), EmlExpr::v(1)), &[2.0, 10.0]);
        assert!((r.center.re - 1024.0).abs() < 1e-8);
    }

    #[test]
    fn test_square() {
        let r = eval(synth::square(EmlExpr::v(0)), &[7.0]);
        assert!((r.center.re - 49.0).abs() < 1e-10);
    }

    #[test]
    fn test_inv() {
        let r = eval(synth::inv(EmlExpr::v(0)), &[4.0]);
        assert!((r.center.re - 0.25).abs() < 1e-12);
    }

    #[test]
    fn test_compile_arity() {
        let expr = synth::add(EmlExpr::v(0), EmlExpr::v(1));
        let prog = expr.compile();
        assert_eq!(prog.arity, 2);
    }

    #[test]
    fn test_compile_arity_v0() {
        let expr = synth::exp(EmlExpr::v(0));
        let prog = expr.compile();
        assert_eq!(prog.arity, 1);
    }

    #[test]
    fn test_compile_arity_const() {
        let expr = EmlExpr::c(42.0);
        let prog = expr.compile();
        assert_eq!(prog.arity, 0);
    }

    #[test]
    fn test_compile_arity_sparse() {
        let expr = synth::add(EmlExpr::v(0), EmlExpr::v(3));
        let prog = expr.compile();
        assert_eq!(prog.arity, 4);
    }

    #[test]
    fn test_execute_empty_args_error() {
        let prog = synth::exp(EmlExpr::v(0)).compile();
        let err = execute(&prog, &[]).unwrap_err();
        assert!(matches!(err, crate::executor::ExecError::MissingArguments { expected: 1, got: 0 }));
    }

    #[test]
    fn test_execute_const_empty_args_success() {
        let prog = EmlExpr::c(42.0).compile();
        let r = execute(&prog, &[]).unwrap();
        assert!((r.center.re - 42.0).abs() < 1e-12);
    }

    #[test]
    fn test_exp_overflow_nan_handled() {
        let prog = synth::exp(EmlExpr::v(0)).compile();
        let ball = ComplexBall::from_real(1000.0);
        let err = execute(&prog, &[ball]).unwrap_err();
        assert!(matches!(
            err,
            crate::ExecError::Eml(crate::EmlError::Overflow)
        ));
    }

    #[test]
    fn test_nan_propagation_error() {
        let nan_ball = ComplexBall::from_real(f64::NAN);
        let one = ComplexBall::from_real(1.0);
        assert_eq!(ComplexBall::eml(nan_ball, one), Err(crate::EmlError::NanResult));
    }

    #[test]
    fn test_ln_zero_is_ieee_continuation() {
        let zero = ComplexBall::from_real(0.0);
        let one = ComplexBall::from_real(1.0);
        let ball = ComplexBall::eml(one, zero).unwrap();
        assert!(!ball.is_finite());
        let err = execute(&EmlExpr::eml(EmlExpr::c(1.0), EmlExpr::c(0.0)).compile(), &[])
            .unwrap_err();
        assert!(matches!(err, crate::ExecError::Eml(crate::EmlError::Overflow)));
    }

    #[test]
    fn test_neg() {
        let r = eval(synth::neg(EmlExpr::v(0)), &[5.0]);
        assert!((r.center.re + 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_fold_const_add() {
        let expr = synth::add(EmlExpr::c(1.0), EmlExpr::c(2.0));
        let folded = expr.fold();
        match folded {
            EmlExpr::Const(v) => assert!((v - 3.0).abs() < 1e-9, "folded {v}"),
            other => panic!("expected Const, got {other}"),
        }
        let prog = expr.compile();
        assert_eq!(prog.arity, 0);
        assert_eq!(prog.instructions.len(), 1);
    }

    #[test]
    fn test_parse_roundtrip() {
        let expr = synth::exp(EmlExpr::v(0));
        let parsed = EmlExpr::parse(&expr.to_string()).unwrap();
        assert_eq!(parsed, expr);
        let nested = EmlExpr::parse("eml(eml(1, x0), 1)").unwrap();
        assert_eq!(
            nested,
            EmlExpr::eml(EmlExpr::eml(EmlExpr::c(1.0), EmlExpr::v(0)), EmlExpr::c(1.0))
        );
    }

    #[test]
    fn test_radius_is_not_rigorous() {
        const { assert!(!crate::RADIUS_IS_RIGOROUS) };
    }
}
