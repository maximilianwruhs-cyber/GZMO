//! Slim view of gzmo-next.toml — only the slices the scheduler needs.
//!
//! Deliberately not gzmo-core's GzmoConfig: this crate must stay a cron
//! runner with no engine coupling. Unknown TOML keys are ignored.

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct SchedulerConfig {
    #[serde(default)]
    pub memory: MemorySection,
    #[serde(default)]
    pub skills: SkillsSection,
    #[serde(default)]
    pub dreams: DreamsSection,
    #[serde(default)]
    pub session_distill: SessionDistillSection,
    #[serde(default)]
    pub spark: SparkSection,
    #[serde(default)]
    pub qdrant: QdrantSection,
    #[serde(default)]
    pub ingest: IngestSection,
    #[serde(default)]
    pub wiki: WikiSection,
    #[serde(default)]
    pub kg_reconcile: KgReconcileSection,
    #[serde(default)]
    pub pedagogy: PedagogySection,
    #[serde(default)]
    pub cabinet: CabinetSection,
    #[serde(default)]
    pub assembly: AssemblySection,
    #[serde(default)]
    pub librarian: LibrarianSection,
    #[serde(default)]
    pub engine: EngineSection,
    #[serde(default)]
    pub embeddings: EmbeddingsSection,
}

#[derive(Debug, Deserialize)]
pub struct LibrarianSection {
    #[serde(default = "default_librarian_url")]
    pub url: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct EngineSection {
    #[serde(default)]
    pub local: EngineLocalSection,
}

#[derive(Debug, Deserialize)]
pub struct EngineLocalSection {
    #[serde(default = "default_engine_url")]
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingsSection {
    #[serde(default = "default_embed_url")]
    pub url: String,
    #[serde(default = "default_embed_model")]
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct MemorySection {
    #[serde(default = "default_vault_db")]
    pub vault_db: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct SkillsSection {
    #[serde(default = "default_dreams_path")]
    pub dreams_path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct DreamsSection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_dream_hour")]
    pub cron_hour: u32,
    #[serde(default)]
    pub cron_minute: u32,
}

#[derive(Debug, Deserialize)]
pub struct SessionDistillSection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub daemon_scheduled: bool,
    #[serde(default = "default_distill_hour")]
    pub cron_hour: u32,
    #[serde(default = "default_distill_minute")]
    pub cron_minute: u32,
}

#[derive(Debug, Deserialize)]
pub struct SparkSection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_spark_hours")]
    pub cron_hours: Vec<u32>,
    #[serde(default = "default_spark_minute")]
    pub cron_minute: u32,
}

#[derive(Debug, Deserialize)]
pub struct QdrantSection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub sync_enabled: bool,
    #[serde(default = "default_qdrant_sync_hour")]
    pub sync_cron_hour: u32,
    #[serde(default = "default_qdrant_sync_minute")]
    pub sync_cron_minute: u32,
}

impl Default for QdrantSection {
    fn default() -> Self {
        Self {
            enabled: true,
            sync_enabled: true,
            sync_cron_hour: default_qdrant_sync_hour(),
            sync_cron_minute: default_qdrant_sync_minute(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct IngestSection {
    #[serde(default)]
    pub batch_enabled: bool,
    #[serde(default = "default_inbox")]
    pub inbox_path: PathBuf,
    #[serde(default = "default_ingest_hour")]
    pub cron_hour: u32,
    #[serde(default)]
    pub cron_minute: u32,
}

impl Default for IngestSection {
    fn default() -> Self {
        Self {
            batch_enabled: false,
            inbox_path: default_inbox(),
            cron_hour: default_ingest_hour(),
            cron_minute: 0,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WikiSection {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_wiki_backend")]
    pub backend: String,
    #[serde(default = "default_wiki_push_hour")]
    pub push_cron_hour: u32,
    #[serde(default = "default_wiki_push_minute")]
    pub push_cron_minute: u32,
}

impl Default for WikiSection {
    fn default() -> Self {
        Self {
            enabled: false,
            backend: default_wiki_backend(),
            push_cron_hour: default_wiki_push_hour(),
            push_cron_minute: default_wiki_push_minute(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct KgReconcileSection {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_kg_hour")]
    pub cron_hour: u32,
    #[serde(default = "default_kg_minute")]
    pub cron_minute: u32,
    #[serde(default = "default_true")]
    pub dry_run: bool,
}

impl Default for KgReconcileSection {
    fn default() -> Self {
        Self {
            enabled: false,
            cron_hour: default_kg_hour(),
            cron_minute: default_kg_minute(),
            dry_run: true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PedagogySection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 0 = Sunday … 6 = Saturday (chrono weekday number).
    #[serde(default)]
    pub cron_weekday: u32,
    #[serde(default = "default_ped_hour")]
    pub cron_hour: u32,
    #[serde(default)]
    pub cron_minute: u32,
}

impl Default for PedagogySection {
    fn default() -> Self {
        Self {
            enabled: true,
            cron_weekday: 0,
            cron_hour: default_ped_hour(),
            cron_minute: 0,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CabinetSection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub cron_weekday: u32,
    #[serde(default = "default_cab_hour")]
    pub cron_hour: u32,
    #[serde(default = "default_cab_minute")]
    pub cron_minute: u32,
}

impl Default for CabinetSection {
    fn default() -> Self {
        Self {
            enabled: true,
            cron_weekday: 0,
            cron_hour: default_cab_hour(),
            cron_minute: default_cab_minute(),
        }
    }
}

fn default_inbox() -> PathBuf {
    PathBuf::from("../data-next/inbox")
}
fn default_ingest_hour() -> u32 {
    2
}
fn default_wiki_backend() -> String {
    "local".into()
}
fn default_wiki_push_hour() -> u32 {
    5
}
fn default_wiki_push_minute() -> u32 {
    30
}
fn default_kg_hour() -> u32 {
    4
}
fn default_kg_minute() -> u32 {
    30
}
fn default_ped_hour() -> u32 {
    6
}
fn default_cab_hour() -> u32 {
    6
}
fn default_cab_minute() -> u32 {
    30
}

fn default_qdrant_sync_hour() -> u32 {
    1
}
fn default_qdrant_sync_minute() -> u32 {
    45
}

#[derive(Debug, Default, Deserialize)]
pub struct AssemblySection {
    #[serde(default)]
    pub distill: Backend,
    #[serde(default)]
    pub dream: Backend,
    #[serde(default)]
    pub spark: Backend,
    #[serde(default)]
    pub ops_health: Backend,
    #[serde(default)]
    pub config_handoff: Backend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    #[default]
    Inline,
    Lab,
}

impl Default for MemorySection {
    fn default() -> Self {
        Self {
            vault_db: default_vault_db(),
        }
    }
}

impl Default for SkillsSection {
    fn default() -> Self {
        Self {
            dreams_path: default_dreams_path(),
        }
    }
}

impl Default for DreamsSection {
    fn default() -> Self {
        Self {
            enabled: true,
            cron_hour: default_dream_hour(),
            cron_minute: 0,
        }
    }
}

impl Default for SessionDistillSection {
    fn default() -> Self {
        Self {
            enabled: true,
            daemon_scheduled: true,
            cron_hour: default_distill_hour(),
            cron_minute: default_distill_minute(),
        }
    }
}

impl Default for SparkSection {
    fn default() -> Self {
        Self {
            enabled: true,
            cron_hours: default_spark_hours(),
            cron_minute: default_spark_minute(),
        }
    }
}

fn default_true() -> bool {
    true
}
fn default_vault_db() -> PathBuf {
    PathBuf::from("data/vault.db")
}
fn default_dreams_path() -> PathBuf {
    PathBuf::from("DREAMS.md")
}
fn default_dream_hour() -> u32 {
    1
}
fn default_distill_hour() -> u32 {
    2
}
fn default_distill_minute() -> u32 {
    15
}
fn default_spark_hours() -> Vec<u32> {
    vec![3, 22]
}
fn default_spark_minute() -> u32 {
    30
}
fn default_librarian_url() -> String {
    "http://localhost:8000/v1".into()
}
fn default_engine_url() -> String {
    "http://localhost:8000/v1".into()
}
fn default_embed_url() -> String {
    "http://192.168.31.110:8081/v1".into()
}
fn default_embed_model() -> String {
    "gzmo-embed".into()
}

impl Default for LibrarianSection {
    fn default() -> Self {
        Self {
            url: default_librarian_url(),
        }
    }
}

impl Default for EngineLocalSection {
    fn default() -> Self {
        Self {
            url: default_engine_url(),
        }
    }
}

impl Default for EmbeddingsSection {
    fn default() -> Self {
        Self {
            url: default_embed_url(),
            model: default_embed_model(),
        }
    }
}

impl SchedulerConfig {
    /// OpenAI-compatible base URL for recipe env (…/v1).
    pub fn librarian_url(&self) -> &str {
        self.librarian.url.trim()
    }

    pub fn llm_url(&self) -> &str {
        let u = self.engine.local.url.trim();
        if u.is_empty() {
            self.librarian_url()
        } else {
            u
        }
    }

    pub fn embed_url(&self) -> &str {
        self.embeddings.url.trim()
    }

    pub fn embed_model(&self) -> &str {
        self.embeddings.model.trim()
    }

    /// `data-next/dream-stats.json` next to the vault.
    pub fn dream_stats_path(&self) -> PathBuf {
        self.memory
            .vault_db
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("dream-stats.json")
    }

    /// `data-next/recall-report.json` next to the vault.
    pub fn recall_report_path(&self) -> PathBuf {
        self.memory
            .vault_db
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("recall-report.json")
    }

    /// Load from the file `GZMO_CONFIG` points at; relative paths are anchored
    /// to the config file's directory (same convention as gzmo-core).
    pub fn load() -> Result<(Self, PathBuf)> {
        let path = PathBuf::from(
            std::env::var("GZMO_CONFIG")
                .context("GZMO_CONFIG must point at the instance config (e.g. gzmo-next.toml)")?,
        );
        let raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let mut cfg: SchedulerConfig =
            toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;

        let base = path.parent().unwrap_or_else(|| Path::new("."));
        let resolve = |p: &PathBuf| -> PathBuf {
            if p.is_absolute() {
                p.clone()
            } else {
                base.join(p)
            }
        };
        cfg.memory.vault_db = resolve(&cfg.memory.vault_db);
        cfg.skills.dreams_path = resolve(&cfg.skills.dreams_path);
        cfg.ingest.inbox_path = resolve(&cfg.ingest.inbox_path);

        cfg.assert_all_lab()?;
        Ok((cfg, path))
    }

    pub fn data_next_dir(&self) -> PathBuf {
        self.memory
            .vault_db
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    }

    pub fn wiki_push_meta_path(&self) -> PathBuf {
        self.data_next_dir().join("wiki-push-latest.json")
    }

    pub fn pedagogy_meta_path(&self) -> PathBuf {
        self.data_next_dir().join("pedagogy-smoke-meta.json")
    }

    pub fn cabinet_meta_path(&self) -> PathBuf {
        self.data_next_dir().join("cabinet-feed-meta.json")
    }

    pub fn ingest_meta_path(&self) -> PathBuf {
        self.data_next_dir().join("ingest-smoke-meta.json")
    }

    pub fn kg_meta_path(&self) -> PathBuf {
        self.data_next_dir().join("kg-reconcile-smoke-meta.json")
    }

    /// Guardrail: this binary only drives lab recipes. If any loop is inline,
    /// someone pointed a legacy config at the scheduler — refuse to start.
    fn assert_all_lab(&self) -> Result<()> {
        let a = &self.assembly;
        let loops = [
            ("distill", a.distill),
            ("dream", a.dream),
            ("spark", a.spark),
            ("ops_health", a.ops_health),
            ("config_handoff", a.config_handoff),
        ];
        let inline: Vec<&str> = loops
            .iter()
            .filter(|(_, b)| *b == Backend::Inline)
            .map(|(name, _)| *name)
            .collect();
        if !inline.is_empty() {
            bail!(
                "gzmo-scheduler requires [assembly] all-lab; inline loops found: {}. \
                 Use `gzmo daemon` for inline engines.",
                inline.join(", ")
            );
        }
        Ok(())
    }

    /// Sibling `<stem>-fused.toml` next to the instance config — the only file
    /// gzmo-handoff.sh --apply may write (never the live config).
    pub fn handoff_apply_target(config_path: &Path) -> Option<PathBuf> {
        let stem = config_path.file_stem()?.to_string_lossy().into_owned();
        Some(config_path.with_file_name(format!("{stem}-fused.toml")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_inline_loops() {
        let cfg: SchedulerConfig = toml::from_str(
            r#"
            [assembly]
            distill = "lab"
            dream = "inline"
            spark = "lab"
            ops_health = "lab"
            config_handoff = "lab"
            "#,
        )
        .unwrap();
        let err = cfg.assert_all_lab().unwrap_err().to_string();
        assert!(err.contains("dream"));
    }

    #[test]
    fn accepts_all_lab() {
        let cfg: SchedulerConfig = toml::from_str(
            r#"
            [assembly]
            distill = "lab"
            dream = "lab"
            spark = "lab"
            ops_health = "lab"
            config_handoff = "lab"
            "#,
        )
        .unwrap();
        assert!(cfg.assert_all_lab().is_ok());
    }

    #[test]
    fn fused_target_is_sibling() {
        let t =
            SchedulerConfig::handoff_apply_target(Path::new("/x/config/gzmo-next.toml")).unwrap();
        assert_eq!(t, PathBuf::from("/x/config/gzmo-next-fused.toml"));
    }
}
