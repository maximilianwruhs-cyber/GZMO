//! Pedagogy session state — ops mode, Trio Model, flipped-classroom prep.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::PedagogyConfig;
use crate::pedagogy::learner::LearnerStore;
use crate::pedagogy::trio::TrioMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PedagogySession {
    /// When true, bypass pedagogy orchestrator and use execution-first agent loop.
    #[serde(default)]
    pub ops_mode: bool,
    #[serde(default)]
    pub trio_mode: TrioMode,
    /// Flipped classroom: topic being prepped asynchronously before Socratic sync.
    #[serde(default)]
    pub learn_prep_topic: Option<String>,
    #[serde(default)]
    pub learn_prep_notes: Option<String>,
    /// Turns since last teachback checkpoint.
    #[serde(default)]
    pub turns_since_teachback: u32,
    /// Prior turn asked for teachback; next student message is the response.
    #[serde(default)]
    pub awaiting_teachback: bool,
    /// Daemon autopoietic triggers: low-tension Socratic dialogue, /dice follow-up loop.
    #[serde(default = "default_auto_triggers_enabled")]
    pub auto_triggers_enabled: bool,
    /// Active persona name (from /transform). None = default GZMO voice.
    #[serde(default)]
    pub persona_name: Option<String>,
    /// Turns since persona was activated. Incremented each turn while persona is active.
    #[serde(default)]
    pub persona_turns_active: u32,
    /// Max turns before persona auto-expires. 0 = disabled (persist until cleared).
    #[serde(default = "default_persona_ttl")]
    pub persona_ttl: u32,
}

/// Default persona TTL: 10 turns before auto-expire.
fn default_persona_ttl() -> u32 {
    10
}

fn default_auto_triggers_enabled() -> bool {
    true
}

impl Default for PedagogySession {
    fn default() -> Self {
        Self {
            ops_mode: false,
            trio_mode: TrioMode::StudentGenAi,
            learn_prep_topic: None,
            learn_prep_notes: None,
            turns_since_teachback: 0,
            awaiting_teachback: false,
            auto_triggers_enabled: true,
            persona_name: None,
            persona_turns_active: 0,
            persona_ttl: 10,
        }
    }
}

impl PedagogySession {
    pub async fn load(config: &PedagogyConfig) -> Result<Self> {
        LearnerStore::new(config).ensure_layout().await?;
        let path = config.session_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = tokio::fs::read_to_string(&path)
            .await
            .with_context(|| format!("read session {:?}", path))?;
        serde_json::from_str(&raw).context("parse session.json")
    }

    pub async fn save(&self, config: &PedagogyConfig) -> Result<()> {
        let path = config.session_path();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let json = serde_json::to_string_pretty(self)?;
        tokio::fs::write(&path, json).await?;
        Ok(())
    }

    pub fn toggle_ops(&mut self) -> bool {
        self.ops_mode = !self.ops_mode;
        self.ops_mode
    }

    pub fn toggle_auto_triggers(&mut self) -> bool {
        self.auto_triggers_enabled = !self.auto_triggers_enabled;
        self.auto_triggers_enabled
    }

    pub fn set_learn_prep(&mut self, topic: &str) {
        self.learn_prep_topic = Some(topic.to_string());
        self.learn_prep_notes = None;
        self.trio_mode = TrioMode::StudentGenAi;
    }

    pub fn clear_learn_prep(&mut self) {
        self.learn_prep_topic = None;
        self.learn_prep_notes = None;
    }

    /// Activate a persona, resetting the turn counter.
    pub fn set_persona(&mut self, name: &str) {
        self.persona_name = Some(name.to_string());
        self.persona_turns_active = 0;
    }

    /// Clear the active persona.
    pub fn clear_persona(&mut self) {
        self.persona_name = None;
        self.persona_turns_active = 0;
    }

    /// Increment persona turn counter. Returns:
    /// - `None` if no persona is active
    /// - `Some(true)` if persona just expired (turns >= ttl)
    /// - `Some(false)` if persona is approaching expiry (turns == ttl - 2, warning zone)
    /// - `Some(false)` if persona is still active and not in warning zone
    pub fn tick_persona(&mut self) -> Option<bool> {
        if self.persona_name.is_none() || self.persona_ttl == 0 {
            return None;
        }
        self.persona_turns_active += 1;
        if self.persona_turns_active >= self.persona_ttl {
            let name = self.persona_name.take().unwrap_or_default();
            self.persona_turns_active = 0;
            tracing::info!(persona = %name, "Persona auto-expired after {} turns", self.persona_ttl);
            return Some(true);
        }
        Some(self.persona_turns_active == self.persona_ttl.saturating_sub(2))
    }

    /// True if persona is in the warning zone (2 turns from expiry).
    pub fn persona_expiry_warning(&self) -> bool {
        self.persona_name.is_some()
            && self.persona_ttl > 0
            && self.persona_turns_active >= self.persona_ttl.saturating_sub(2)
            && self.persona_turns_active < self.persona_ttl
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persona_activates_and_ticks() {
        let mut s = PedagogySession::default();
        assert!(s.persona_name.is_none());
        s.set_persona("Heaviside");
        assert_eq!(s.persona_name.as_deref(), Some("Heaviside"));
        assert_eq!(s.persona_turns_active, 0);

        // Tick 1 — no warning, no expiry
        assert_eq!(s.tick_persona(), Some(false));
        assert!(s.persona_expiry_warning() == false);
    }

    #[test]
    fn persona_warns_at_ttl_minus_2() {
        let mut s = PedagogySession::default();
        s.set_persona("Rick");
        s.persona_ttl = 5;

        // Ticks 1-2: normal
        s.tick_persona(); // turn 1
        s.tick_persona(); // turn 2
        assert!(!s.persona_expiry_warning());

        // Tick 3: turn == ttl-2 → warning
        assert_eq!(s.tick_persona(), Some(true));  // true = warning zone
        assert!(s.persona_expiry_warning());

        // Tick 4: still in warning zone
        assert_eq!(s.tick_persona(), Some(false));
        assert!(s.persona_expiry_warning());
    }

    #[test]
    fn persona_expires_at_ttl() {
        let mut s = PedagogySession::default();
        s.set_persona("Batman");
        s.persona_ttl = 3;

        s.tick_persona(); // turn 1
        s.tick_persona(); // turn 2 (warning)
        let expired = s.tick_persona(); // turn 3 = ttl → expires
        assert_eq!(expired, Some(true));
        assert!(s.persona_name.is_none());
        assert_eq!(s.persona_turns_active, 0);
    }

    #[test]
    fn persona_ttl_zero_never_expires() {
        let mut s = PedagogySession::default();
        s.set_persona("Grothendieck");
        s.persona_ttl = 0;

        // tick_persona returns None when ttl=0
        assert_eq!(s.tick_persona(), None);
        assert!(s.persona_name.is_some());
    }

    #[test]
    fn clear_persona_resets_state() {
        let mut s = PedagogySession::default();
        s.set_persona("Heaviside");
        s.tick_persona();
        s.tick_persona();
        assert!(s.persona_name.is_some());
        assert!(s.persona_turns_active > 0);

        s.clear_persona();
        assert!(s.persona_name.is_none());
        assert_eq!(s.persona_turns_active, 0);
    }

    #[test]
    fn no_persona_tick_returns_none() {
        let mut s = PedagogySession::default();
        assert_eq!(s.tick_persona(), None);
    }
}
