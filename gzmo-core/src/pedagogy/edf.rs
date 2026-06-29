//! Evidence–Decision–Feedback framework and stealth assessment metrics.

use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::PedagogyConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZpdPhase {
    /// "I Do" — high support, modeling.
    IDo,
    /// "We Do" — guided co-construction.
    WeDo,
    /// "You Do" — fading, autonomy challenge.
    YouDo,
}

impl ZpdPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IDo => "i_do",
            Self::WeDo => "we_do",
            Self::YouDo => "you_do",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StealthMetrics {
    /// Prompt-Structure Uptake — logical structured inquiry.
    pub psu: f64,
    /// Scaffolded Depth and Revision — productive hesitation vs guessing.
    pub sdr: f64,
    /// Logic Validation and Debugging — systematic reasoning on failure.
    pub lvd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdfRecord {
    pub timestamp: DateTime<Utc>,
    pub user_input: String,
    pub evidence: String,
    pub decision: String,
    pub zpd_phase: ZpdPhase,
    pub hint_level: u8,
    pub stealth: StealthMetrics,
    pub tutor_response_preview: String,
    #[serde(default)]
    pub leakage_detected: bool,
    #[serde(default)]
    pub leakage_retries: u8,
    #[serde(default)]
    pub compute_used: bool,
}

pub struct EdfStore {
    path: std::path::PathBuf,
}

impl EdfStore {
    pub fn new(config: &PedagogyConfig) -> Self {
        Self {
            path: std::path::PathBuf::from(&config.edf_log_path),
        }
    }

    pub async fn append(&self, record: &EdfRecord) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let line = serde_json::to_string(record)?;
        use tokio::io::AsyncWriteExt;
        let mut f = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await
            .with_context(|| format!("open EDF log {:?}", self.path))?;
        // Write line + newline atomically in single call to prevent interleaving
        let mut buf = line.into_bytes();
        buf.push(b'\n');
        f.write_all(&buf).await?;
        Ok(())
    }

    pub fn log_path(&self) -> &Path {
        &self.path
    }
}
