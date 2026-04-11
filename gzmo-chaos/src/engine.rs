/// Engine state machine — energy, phases, death/rebirth mechanics
use crate::chaos::Phase;

const ENERGY_MIN: f64 = 0.0;
const ENERGY_MAX: f64 = 100.0;
const REGEN_BASE: f64 = 1.0;
const REBIRTH_ENERGY: f64 = 30.0;
const INBOX_ENERGY: f64 = 20.0;

#[derive(Debug, Clone)]
pub struct EngineState {
    pub tick: u64,
    pub energy: f64,
    pub phase: Phase,
    pub alive: bool,
    pub deaths: u32,
}

impl EngineState {
    pub fn new() -> Self {
        Self {
            tick: 0,
            energy: ENERGY_MAX,
            phase: Phase::Idle,
            alive: true,
            deaths: 0,
        }
    }

    /// Process a heartbeat tick. Returns whether a rebirth occurred.
    /// `thought_drain_mod` is the cognitive load multiplier from incubating thoughts (1.0 = baseline).
    pub fn tick_heartbeat(
        &mut self,
        tension: f64,
        gravity: f64,
        friction: f64,
        chaos_roll: f64,
        thought_drain_mod: f64,
    ) -> bool {
        self.tick += 1;
        self.phase = Phase::from_tension(tension);

        if !self.alive {
            // Dead engines tick silently — chaos still runs externally
            return false;
        }

        // Drain scaled for 174 BPM (~3 ticks/sec), amplified by cognitive load
        let drain = gravity * friction * 0.1 * self.phase.drain_multiplier() * thought_drain_mod;

        // Inverse regen curve: stronger when depleted, zero at full
        let regen = REGEN_BASE * (1.0 - (self.energy / ENERGY_MAX));

        let net = match self.phase {
            Phase::Idle => regen - drain,
            Phase::Build => (regen * 0.3) - drain,
            Phase::Drop => -drain, // No regen in DROP — pure hemorrhage
        };

        self.energy = (self.energy + net).clamp(ENERGY_MIN, ENERGY_MAX);

        // Death check
        if self.energy <= ENERGY_MIN {
            self.alive = false;
            self.deaths += 1;

            // Spontaneous rebirth: 30% chance (chaos roll > 0.7)
            if chaos_roll > 0.7 {
                self.alive = true;
                self.energy = REBIRTH_ENERGY;
                return true;
            }
        }

        false
    }

    /// Process an inbox drop — energy injection, can resurrect
    pub fn apply_inbox_drop(&mut self) -> bool {
        self.tick += 1;
        self.energy = (self.energy + INBOX_ENERGY).min(ENERGY_MAX);

        let resurrected = !self.alive;
        if resurrected {
            self.alive = true;
        }

        resurrected
    }

    /// Inject or drain energy from a skill feedback event
    pub fn apply_energy_delta(&mut self, delta: f64) {
        self.energy = (self.energy + delta).clamp(ENERGY_MIN, ENERGY_MAX);
        if self.energy <= ENERGY_MIN && self.alive {
            self.alive = false;
            self.deaths += 1;
        }
    }

    /// Inject tension directly (clamped to 0-100)
    pub fn apply_tension_delta(&mut self, current_tension: f64, delta: f64) -> f64 {
        (current_tension + delta).clamp(0.0, 100.0)
    }
}

impl Default for EngineState {
    fn default() -> Self {
        Self::new()
    }
}
