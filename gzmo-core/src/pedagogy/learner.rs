//! Tripartite learner memory — episodic, semantic, procedural tiers.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::PedagogyConfig;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EpisodicLearnerMemory {
    #[serde(default)]
    pub entries: Vec<EpisodicLearnerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicLearnerEntry {
    pub timestamp: DateTime<Utc>,
    pub summary: String,
    pub struggle: Option<String>,
    pub breakthrough: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SemanticLearnerMemory {
    #[serde(default)]
    pub mastery_vectors: Vec<String>,
    #[serde(default)]
    pub interests: Vec<String>,
    #[serde(default)]
    pub misconceptions: Vec<String>,
    #[serde(default)]
    pub accommodations: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProceduralLearnerMemory {
    #[serde(default)]
    pub effective_modalities: Vec<String>,
    #[serde(default)]
    pub ineffective_modalities: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearnerProfile {
    pub learner_id: String,
    #[serde(default)]
    pub episodic: EpisodicLearnerMemory,
    #[serde(default)]
    pub semantic: SemanticLearnerMemory,
    #[serde(default)]
    pub procedural: ProceduralLearnerMemory,
    pub updated_at: Option<DateTime<Utc>>,
}

impl LearnerProfile {
    pub fn default_operator() -> Self {
        Self {
            learner_id: "operator".to_string(),
            updated_at: Some(Utc::now()),
            ..Default::default()
        }
    }

    /// Bounded context block for system prompt injection (cache-friendly suffix).
    pub fn prompt_block(&self, max_chars: usize) -> String {
        let mut block = String::from("\n\n## Learner Profile (Teaching Context)\n");
        if !self.semantic.mastery_vectors.is_empty() {
            block.push_str(&format!(
                "- Mastery: {}\n",
                self.semantic.mastery_vectors.join("; ")
            ));
        }
        if !self.semantic.misconceptions.is_empty() {
            block.push_str(&format!(
                "- Misconceptions to watch: {}\n",
                self.semantic.misconceptions.join("; ")
            ));
        }
        if !self.procedural.effective_modalities.is_empty() {
            block.push_str(&format!(
                "- Works well with: {}\n",
                self.procedural.effective_modalities.join("; ")
            ));
        }
        if let Some(last) = self.episodic.entries.last() {
            block.push_str(&format!("- Last session note: {}\n", last.summary));
        }
        if block.len() > max_chars {
            block.truncate(max_chars);
            block.push_str("…");
        }
        block
    }

    /// Distill a teachback response into semantic mastery notes.
    pub fn record_teachback(&mut self, summary: &str) {
        let trimmed = summary.trim();
        if trimmed.len() < 20 {
            return;
        }
        let note = crate::text_util::truncate_chars(trimmed, 120);
        if !self.semantic.mastery_vectors.iter().any(|m| m == &note) {
            self.semantic.mastery_vectors.push(note);
        }
        if self.semantic.mastery_vectors.len() > 40 {
            let drain = self.semantic.mastery_vectors.len() - 40;
            self.semantic.mastery_vectors.drain(0..drain);
        }
        self.updated_at = Some(Utc::now());
    }

    pub fn record_episode(&mut self, summary: &str, struggle: Option<&str>, breakthrough: Option<&str>) {
        self.episodic.entries.push(EpisodicLearnerEntry {
            timestamp: Utc::now(),
            summary: summary.to_string(),
            struggle: struggle.map(String::from),
            breakthrough: breakthrough.map(String::from),
        });
        if self.episodic.entries.len() > 100 {
            let drain = self.episodic.entries.len() - 100;
            self.episodic.entries.drain(0..drain);
        }
        self.updated_at = Some(Utc::now());
    }
}

pub struct LearnerStore {
    config: PedagogyConfig,
}

impl LearnerStore {
    pub fn new(config: &PedagogyConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub fn pedagogy_config(&self) -> &PedagogyConfig {
        &self.config
    }

    /// Migrate legacy flat `data/learner/profile.json` layout to `data/learner/operator/`.
    pub async fn ensure_layout(&self) -> Result<()> {
        let root = std::path::PathBuf::from(&self.config.learner_data_dir);
        let legacy_profile = root.join("profile.json");
        let operator_dir = root.join("operator");
        if legacy_profile.exists() && !operator_dir.join("profile.json").exists() {
            tokio::fs::create_dir_all(&operator_dir).await?;
            for name in ["profile.json", "session.json"] {
                let src = root.join(name);
                if src.exists() {
                    tokio::fs::rename(&src, operator_dir.join(name)).await?;
                }
            }
            let legacy_episodes = root.join("episodes");
            if legacy_episodes.exists() && !operator_dir.join("episodes").exists() {
                tokio::fs::rename(&legacy_episodes, operator_dir.join("episodes")).await?;
            }
        }
        tokio::fs::create_dir_all(self.config.learner_dir()).await?;
        Ok(())
    }

    pub async fn load(&self) -> Result<LearnerProfile> {
        self.ensure_layout().await?;
        let path = self.config.profile_path();
        if !path.exists() {
            let mut profile = LearnerProfile::default_operator();
            profile.learner_id = self.config.learner_id().to_string();
            return Ok(profile);
        }
        let raw = tokio::fs::read_to_string(&path)
            .await
            .with_context(|| format!("read learner profile {:?}", path))?;
        let mut profile: LearnerProfile =
            serde_json::from_str(&raw).context("parse learner profile.json")?;
        profile.learner_id = self.config.learner_id().to_string();
        Ok(profile)
    }

    pub async fn save(&self, profile: &LearnerProfile) -> Result<()> {
        self.ensure_layout().await?;
        let path = self.config.profile_path();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let json = serde_json::to_string_pretty(profile)?;
        tokio::fs::write(&path, json).await?;
        Ok(())
    }

    pub async fn append_episode_markdown(&self, summary: &str) -> Result<()> {
        let dir = self.config.episodes_dir();
        tokio::fs::create_dir_all(&dir).await?;
        let file = dir.join(format!("{}.md", Utc::now().format("%Y-%m-%d")));
        let line = format!(
            "- {} — {}\n",
            Utc::now().format("%H:%M UTC"),
            summary
        );
        use tokio::io::AsyncWriteExt;
        let mut f = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file)
            .await?;
        f.write_all(line.as_bytes()).await?;
        Ok(())
    }

    pub fn profile_path_buf(&self) -> std::path::PathBuf {
        self.config.profile_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_block_includes_mastery() {
        let mut p = LearnerProfile::default_operator();
        p.semantic.mastery_vectors.push("bash basics".into());
        let block = p.prompt_block(2000);
        assert!(block.contains("bash basics"));
    }
}
