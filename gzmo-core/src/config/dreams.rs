use serde::Deserialize;

use super::defaults::*;

// ─── Dreams ─────────────────────────────────────────────────────────────

/// Settings for the autoDream consolidation engine.
///
/// These knobs are the front line against hallucinated knowledge entering
/// permanent memory. When `verify` is on, every extracted entity/relation is
/// fact-checked against the source log before it is written to the knowledge
/// graph; anything scoring below `min_confidence` (or unsupported by a quotable
/// span) is dropped instead of promoted.
#[derive(Debug, Deserialize, Clone)]
pub struct DreamsConfig {
    /// Master switch — when false, daemon skips nightly consolidation (CLI `gzmo dream` still runs).
    #[serde(default = "default_dream_enabled")]
    pub enabled: bool,

    /// If true, run a second LLM pass that fact-checks each extracted claim
    /// against the source before writing it to the knowledge graph / vault.
    #[serde(default = "default_dream_verify")]
    pub verify: bool,

    /// Minimum verified confidence (0.0–1.0) required to promote a claim.
    /// Matches the vault's quarantine threshold so the graph and vault agree.
    #[serde(default = "default_dream_min_confidence")]
    pub min_confidence: f64,

    /// Temperature for the verification pass. Kept low for near-deterministic
    /// fact-checking, independent of the engine's creative default.
    #[serde(default = "default_dream_verify_temperature")]
    pub verify_temperature: f32,

    /// Near-deterministic decoding for the extraction pass (mirrors
    /// `verify_temperature`): keeps KG extract independent of engine/chaos temp.
    #[serde(default = "default_dream_verify_temperature")]
    pub extract_temperature: f32,

    /// Hour (UTC) when the daemon runs consolidation for **yesterday's** episodic log.
    #[serde(default = "default_dream_cron_hour")]
    pub cron_hour: u32,

    /// Minute (UTC) within `cron_hour` for the nightly dream tick.
    #[serde(default = "default_dream_cron_minute")]
    pub cron_minute: u32,

    /// Supported claims must include a quotable evidence span (≥12 chars).
    #[serde(default = "default_kg_require_evidence")]
    pub require_evidence: bool,

    /// Abort if Neo4j MCP writes fewer nodes/edges than verified (no silent partial promote).
    #[serde(default = "default_kg_strict")]
    pub strict_kg: bool,

    /// Max chars per REM/verify chunk for large daily logs (paragraph-aware split).
    #[serde(default = "default_pipeline_chunk_chars")]
    pub chunk_chars: usize,

    /// Drop episodic `### 🧠 INTERNAL` sections whose body matches any of these (case-insensitive).
    #[serde(default = "default_dream_exclude_episodic_substrings")]
    pub exclude_episodic_substrings: Vec<String>,

    /// Skip REM/KG when filtered episodic text is shorter than this (ops-only days).
    #[serde(default = "default_dream_min_consolidation_chars")]
    pub min_consolidation_chars: usize,

    /// M3: fold honeypot distillates + vector associations into REM (not episodic-only).
    #[serde(default = "default_dream_honeypot_rem_enabled")]
    pub honeypot_rem_enabled: bool,

    /// Anchor facts sampled from honeypot for association expansion.
    #[serde(default = "default_honeypot_rem_anchor_limit")]
    pub honeypot_rem_anchor_limit: usize,

    /// Similar honeypot facts retrieved per anchor (vector decay search).
    #[serde(default = "default_honeypot_rem_associate_k")]
    pub honeypot_rem_associate_k: usize,
}

impl DreamsConfig {
    pub fn kg_gate(&self) -> crate::memory::kg_extract::KgGateConfig {
        crate::memory::kg_extract::KgGateConfig {
            verify: self.verify,
            min_confidence: self.min_confidence,
            verify_temperature: self.verify_temperature,
            extract_temperature: self.extract_temperature,
            require_evidence: self.require_evidence,
            strict_kg: self.strict_kg,
        }
    }
}

impl Default for DreamsConfig {
    fn default() -> Self {
        Self {
            enabled: default_dream_enabled(),
            verify: default_dream_verify(),
            min_confidence: default_dream_min_confidence(),
            verify_temperature: default_dream_verify_temperature(),
            extract_temperature: default_dream_verify_temperature(),
            cron_hour: default_dream_cron_hour(),
            cron_minute: default_dream_cron_minute(),
            require_evidence: default_kg_require_evidence(),
            strict_kg: default_kg_strict(),
            chunk_chars: default_pipeline_chunk_chars(),
            exclude_episodic_substrings: default_dream_exclude_episodic_substrings(),
            min_consolidation_chars: default_dream_min_consolidation_chars(),
            honeypot_rem_enabled: true,
            honeypot_rem_anchor_limit: default_honeypot_rem_anchor_limit(),
            honeypot_rem_associate_k: default_honeypot_rem_associate_k(),
        }
    }
}
