use num_complex::Complex64;
use thiserror::Error;

/// First-order linearization plus one ULP-scale term. Not a rigorous enclosure.
pub const RADIUS_IS_RIGOROUS: bool = false;

/// A complex number with a tracked error radius (ComplexBall arithmetic).
///
/// Propagates precision bounds during EML operations so the user can
/// see where drift accumulates in deep expression trees.
///
/// The radius is a first-order sketch (`RADIUS_IS_RIGOROUS == false`), not
/// interval arithmetic. Do not treat a small radius as a proof of enclosure.
///
/// Infinite centers are allowed as IEEE continuation so `synth::neg` / `add`
/// can pass `ln(0) = -inf` as an intermediate. [`crate::execute`] refuses a
/// non-finite *final* result as [`EmlError::Overflow`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComplexBall {
    pub center: Complex64,
    pub radius: f64,
}

impl ComplexBall {
    pub fn exact(val: Complex64) -> Self {
        Self {
            center: val,
            radius: 0.0,
        }
    }

    pub fn from_real(val: f64) -> Self {
        Self::exact(Complex64::new(val, 0.0))
    }

    pub fn is_finite(&self) -> bool {
        self.center.re.is_finite() && self.center.im.is_finite()
    }

    /// `eml(x, y) = exp(x) - ln(y)`
    ///
    /// Error bound (first-order, not rigorous):
    ///   Δresult ≈ |exp(x)|·Δx + (1/|y|)·Δy  + 1 ULP
    pub fn eml(x: Self, y: Self) -> Result<Self, EmlError> {
        if has_nan(x) || has_nan(y) {
            return Err(EmlError::NanResult);
        }

        if is_real(x) && is_real(y) && y.center.re > 0.0 {
            return eml_real(x, y);
        }
        eml_complex(x, y)
    }
}

fn has_nan(b: ComplexBall) -> bool {
    b.center.re.is_nan() || b.center.im.is_nan()
}

fn is_real(b: ComplexBall) -> bool {
    b.center.im == 0.0
}

fn first_order_radius(exp_scale: f64, y_norm: f64, dx: f64, dy: f64) -> f64 {
    let inv_scale = if y_norm == 0.0 { f64::INFINITY } else { 1.0 / y_norm };
    (exp_scale * dx) + (inv_scale * dy) + 1e-15
}

fn classify_center(center: Complex64, radius: f64) -> Result<ComplexBall, EmlError> {
    if center.re.is_nan() || center.im.is_nan() {
        return Err(EmlError::NanResult);
    }
    Ok(ComplexBall { center, radius })
}

fn eml_real(x: ComplexBall, y: ComplexBall) -> Result<ComplexBall, EmlError> {
    let exp_x = x.center.re.exp();
    let ln_y = y.center.re.ln();
    let re = exp_x - ln_y;
    let radius = first_order_radius(exp_x.abs(), y.center.re.abs(), x.radius, y.radius);
    classify_center(Complex64::new(re, 0.0), radius)
}

fn eml_complex(x: ComplexBall, y: ComplexBall) -> Result<ComplexBall, EmlError> {
    let exp_x = x.center.exp();
    let ln_y = y.center.ln();
    let center = exp_x - ln_y;
    let radius = first_order_radius(exp_x.norm(), y.center.norm(), x.radius, y.radius);
    classify_center(center, radius)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum EmlError {
    #[error("EML produced NaN")]
    NanResult,
    #[error("EML overflow: result is infinite")]
    Overflow,
}
