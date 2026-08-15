use num_complex::Complex64;

/// A complex number with a tracked error radius (ComplexBall arithmetic).
///
/// Propagates precision bounds during EML operations so the user can
/// see where drift accumulates in deep expression trees.
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

    /// `eml(x, y) = exp(x) - ln(y)`
    ///
    /// Error bound propagation:
    ///   Δresult ≈ |exp(x)|·Δx + (1/|y|)·Δy  + 1 ULPs
    pub fn eml(x: Self, y: Self) -> Result<Self, EmlError> {
        let exp_x = x.center.exp();

        // ln(0) = -inf is a valid IEEE 754 value — let it propagate.
        let ln_y = y.center.ln();
        let center = exp_x - ln_y;

        if center.re.is_nan() || center.im.is_nan() {
            return Err(EmlError::NanResult);
        }

        // propagated error bound
        let exp_scale = exp_x.norm();
        let inv_scale = 1.0 / y.center.norm();
        let radius = (exp_scale * x.radius) + (inv_scale * y.radius) + 1e-15;

        Ok(Self { center, radius })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmlError {
    NanResult,
    Overflow,
}

impl core::fmt::Display for EmlError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NanResult => write!(f, "EML produced NaN"),
            Self::Overflow => write!(f, "EML overflow: result is infinite"),
        }
    }
}
