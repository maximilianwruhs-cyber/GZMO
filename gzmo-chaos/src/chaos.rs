//! Deterministic Chaos Engine — f64 Lorenz Attractor + Logistic Map
//!
//! The Lorenz system is a set of three coupled nonlinear ODEs that produce
//! deterministic yet unpredictable trajectories in 3D phase space.
//! Hardware telemetry drives the attractor's control parameters.

/// Engine phase derived from global hardware tension.
/// Compile-time checked — impossible to misspell, exhaustive matching.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Phase {
    Idle,
    Build,
    Drop,
}

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Phase::Idle => write!(f, "Idle"),
            Phase::Build => write!(f, "Build"),
            Phase::Drop => write!(f, "Drop"),
        }
    }
}

impl Phase {
    pub fn from_tension(t: f64) -> Self {
        let t = if t.is_finite() { t } else { 0.0 };
        if t < 30.0 {
            Phase::Idle
        } else if t < 70.0 {
            Phase::Build
        } else {
            Phase::Drop
        }
    }

    /// Energy drain multiplier per phase
    pub fn drain_multiplier(&self) -> f64 {
        match self {
            Phase::Idle => 0.3,
            Phase::Build => 1.5,
            Phase::Drop => 3.0,
        }
    }

    /// Lorenz sigma parameter — controls convection intensity
    /// Higher sigma = more aggressive orbital divergence
    pub fn lorenz_sigma(&self) -> f64 {
        match self {
            Phase::Idle => 8.0,   // Below standard (10) — calmer orbits
            Phase::Build => 10.0, // Standard Lorenz — onset of chaos
            Phase::Drop => 14.0,  // Above standard — extreme sensitivity
        }
    }
}

/// 3D Lorenz Attractor: dx/dt = σ(y-x), dy/dt = x(ρ-z)-y, dz/dt = xy-βz
pub struct LorenzAttractor {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    sigma: f64,
    rho: f64,
    beta: f64,
    dt: f64,
}

impl LorenzAttractor {
    pub fn new(seed: f64) -> Self {
        // Lorenz's historical initial conditions, perturbed by seed
        let seed = if seed.is_finite() { seed } else { 0.506 };
        Self {
            x: seed,
            y: seed + 0.001,
            z: seed + 0.002,
            sigma: 10.0,     // Standard Prandtl number
            rho: 28.0,       // Standard Rayleigh number
            beta: 8.0 / 3.0, // Geometric factor
            dt: 0.005,       // Integration timestep (small for stability)
        }
    }

    /// Advance the attractor one step via 4th-order Runge-Kutta
    pub fn step(&mut self) -> (f64, f64, f64) {
        let (x, y, z) = (self.x, self.y, self.z);
        let (s, r, b, dt) = (self.sigma, self.rho, self.beta, self.dt);

        // RK4 integration for numerical stability
        let k1x = s * (y - x);
        let k1y = x * (r - z) - y;
        let k1z = x * y - b * z;

        let x2 = x + 0.5 * dt * k1x;
        let y2 = y + 0.5 * dt * k1y;
        let z2 = z + 0.5 * dt * k1z;
        let k2x = s * (y2 - x2);
        let k2y = x2 * (r - z2) - y2;
        let k2z = x2 * y2 - b * z2;

        let x3 = x + 0.5 * dt * k2x;
        let y3 = y + 0.5 * dt * k2y;
        let z3 = z + 0.5 * dt * k2z;
        let k3x = s * (y3 - x3);
        let k3y = x3 * (r - z3) - y3;
        let k3z = x3 * y3 - b * z3;

        let x4 = x + dt * k3x;
        let y4 = y + dt * k3y;
        let z4 = z + dt * k3z;
        let k4x = s * (y4 - x4);
        let k4y = x4 * (r - z4) - y4;
        let k4z = x4 * y4 - b * z4;

        self.x = x + (dt / 6.0) * (k1x + 2.0 * k2x + 2.0 * k3x + k4x);
        self.y = y + (dt / 6.0) * (k1y + 2.0 * k2y + 2.0 * k3y + k4y);
        self.z = z + (dt / 6.0) * (k1z + 2.0 * k2z + 2.0 * k3z + k4z);

        (self.x, self.y, self.z)
    }

    /// Shift attractor control parameters based on engine phase
    pub fn update_phase(&mut self, phase: &Phase) {
        self.sigma = phase.lorenz_sigma();
    }

    /// Map the Lorenz x-coordinate to [0, 1] for use as a chaos index
    /// The Lorenz attractor orbits roughly in x ∈ [-20, 20]
    pub fn normalized_output(&self) -> f64 {
        ((self.x + 20.0) / 40.0).clamp(0.0, 1.0)
    }

    /// Apply permanent rho mutation from crystallized thoughts
    /// Shifts the attractor's orbital topology
    pub fn apply_rho_mutation(&mut self, rho_mod: f64) {
        let rho_mod = if rho_mod.is_finite() { rho_mod } else { 0.0 };
        self.rho = 28.0 + rho_mod.clamp(-10.0, 10.0); // Base 28.0 + accumulated thought mutations
    }

    /// Apply transient sigma noise from incubating thoughts (cognitive load)
    pub fn apply_cognitive_noise(&mut self, noise: f64) {
        // Phase sigma + cognitive noise from active incubation
        if noise.is_finite() {
            self.sigma = (self.sigma + noise).clamp(1.0, 30.0);
        }
    }

    /// Get current sigma for diagnostics
    pub fn sigma(&self) -> f64 {
        self.sigma
    }

    /// Get current rho for diagnostics
    pub fn rho(&self) -> f64 {
        self.rho
    }
}

/// Classic Logistic Map — fast secondary chaos source
/// Reseeded periodically from Lorenz state for coupled dynamics
pub struct LogisticMap {
    pub r: f64,
    pub x: f64,
}

impl LogisticMap {
    pub fn new(seed: f64) -> Self {
        let seed = if seed.is_finite() { seed } else { 0.506 };
        Self {
            r: 3.99,
            x: seed.clamp(0.0001, 0.9999),
        }
    }

    pub fn next_val(&mut self) -> f64 {
        self.x = self.r * self.x * (1.0 - self.x);
        self.x
    }

    /// Couple the logistic map to the Lorenz attractor
    /// Seeds x from the normalized Lorenz output
    pub fn reseed_from_lorenz(&mut self, lorenz_normalized: f64) {
        if lorenz_normalized.is_finite() {
            self.x = lorenz_normalized.clamp(0.0001, 0.9999);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lorenz_stays_bounded() {
        let mut lorenz = LorenzAttractor::new(0.506);
        for _ in 0..10_000 {
            let (x, y, z) = lorenz.step();
            assert!(x.abs() < 100.0, "x diverged: {x}");
            assert!(y.abs() < 100.0, "y diverged: {y}");
            assert!(z.abs() < 100.0, "z diverged: {z}");
        }
    }

    #[test]
    fn logistic_stays_in_unit_interval() {
        let mut logistic = LogisticMap::new(0.506);
        for _ in 0..10_000 {
            let v = logistic.next_val();
            assert!((0.0..=1.0).contains(&v), "logistic outside [0,1]: {v}");
        }
    }

    #[test]
    fn normalized_output_in_range() {
        let mut lorenz = LorenzAttractor::new(0.506);
        for _ in 0..1_000 {
            lorenz.step();
            let n = lorenz.normalized_output();
            assert!((0.0..=1.0).contains(&n), "normalized outside [0,1]: {n}");
        }
    }

    #[test]
    fn phase_from_tension() {
        assert_eq!(Phase::from_tension(10.0), Phase::Idle);
        assert_eq!(Phase::from_tension(50.0), Phase::Build);
        assert_eq!(Phase::from_tension(80.0), Phase::Drop);
        assert_eq!(Phase::from_tension(f64::NAN), Phase::Idle);
    }

    #[test]
    fn non_finite_seed_uses_default() {
        let lorenz = LorenzAttractor::new(f64::NAN);
        let logistic = LogisticMap::new(f64::NAN);

        assert!(lorenz.x.is_finite());
        assert!(logistic.x.is_finite());
    }
}
