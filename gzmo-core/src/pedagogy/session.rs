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

    pub fn set_learn_prep(&mut self, topic: &str) {
        self.learn_prep_topic = Some(topic.to_string());
        self.learn_prep_notes = None;
        self.trio_mode = TrioMode::StudentGenAi;
    }

    pub fn clear_learn_prep(&mut self) {
        self.learn_prep_topic = None;
        self.learn_prep_notes = None;
    }
}
