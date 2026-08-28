use serde::Deserialize;

use super::defaults::*;

// ─── Wiki layer ────────────────────────────────────────────────────────────

/// Settings for the git-tracked markdown wiki layer (`WikiEngine`).
///
/// The wiki is a browsable, compounding markdown synthesis layer that sits
/// between raw RAG retrieval and `DREAMS.md`. Pages are derived from already
/// verified vault facts, so retrieval is **emit-only**: `WikiEngine::search`
/// greps over `wiki/*.md` and pages are never re-ingested into the honeypot
/// (which would create circular facts). See `WIKI.md` and `docs/WIKI_LAYER.md`.
#[derive(Debug, Deserialize, Clone)]
pub struct WikiConfig {
    #[serde(default = "default_wiki_enabled")]
    pub enabled: bool,

    /// `"local"` = on-disk WikiEngine; `"okforge"` = OKCP push to forge repo.
    #[serde(default = "default_wiki_backend")]
    pub backend: String,

    #[serde(default = "default_wiki_directory")]
    pub directory: String,

    #[serde(default = "default_wiki_index_path")]
    pub index_path: String,

    #[serde(default = "default_wiki_log_path")]
    pub log_path: String,

    #[serde(default = "default_wiki_schema_path")]
    pub schema_path: String,

    /// When true, `IngestEngine` emits a `wiki/sources/` page on promotion.
    #[serde(default = "default_wiki_emit_on_ingest")]
    pub emit_on_ingest: bool,

    /// Hook `wiki-okforge-push` after distill recipe (GZMO-next).
    #[serde(default)]
    pub emit_after_distill: bool,

    /// Hook `wiki-okforge-push` after dream recipe (GZMO-next).
    #[serde(default)]
    pub emit_after_dream: bool,

    /// Daemon "Knowledge Gardener" sync loop (UTC hour/minute).
    #[serde(default = "default_wiki_sync_cron_hour")]
    pub sync_cron_hour: u32,
    #[serde(default = "default_wiki_sync_cron_minute")]
    pub sync_cron_minute: u32,

    /// Daemon weekly lint loop (UTC weekday 0=Sun, hour).
    #[serde(default = "default_wiki_lint_cron_dow")]
    pub lint_cron_dow: u32,
    #[serde(default = "default_wiki_lint_cron_hour")]
    pub lint_cron_hour: u32,

    /// Catch-up push cron (UTC) when recipe hooks miss.
    #[serde(default = "default_wiki_push_cron_hour")]
    pub push_cron_hour: u32,
    #[serde(default = "default_wiki_push_cron_minute")]
    pub push_cron_minute: u32,

    #[serde(default)]
    pub okforge: Option<WikiOkforgeConfig>,
}

/// OKForge OKCP target for `[wiki.okforge]`.
#[derive(Debug, Deserialize, Clone)]
pub struct WikiOkforgeConfig {
    #[serde(default = "default_okforge_url")]
    pub url: String,
    #[serde(default = "default_okforge_owner")]
    pub owner: String,
    #[serde(default = "default_okforge_repo")]
    pub repo: String,
    #[serde(default = "default_okforge_token_env")]
    pub token_env: String,
    #[serde(default = "default_okforge_agent_id")]
    pub agent_id: String,
    #[serde(default = "default_true")]
    pub auto_commit: bool,
    #[serde(default)]
    pub open_pr: bool,
}

impl Default for WikiOkforgeConfig {
    fn default() -> Self {
        Self {
            url: default_okforge_url(),
            owner: default_okforge_owner(),
            repo: default_okforge_repo(),
            token_env: default_okforge_token_env(),
            agent_id: default_okforge_agent_id(),
            auto_commit: true,
            open_pr: false,
        }
    }
}

impl Default for WikiConfig {
    fn default() -> Self {
        Self {
            enabled: default_wiki_enabled(),
            backend: default_wiki_backend(),
            directory: default_wiki_directory(),
            index_path: default_wiki_index_path(),
            log_path: default_wiki_log_path(),
            schema_path: default_wiki_schema_path(),
            emit_on_ingest: default_wiki_emit_on_ingest(),
            emit_after_distill: false,
            emit_after_dream: false,
            sync_cron_hour: default_wiki_sync_cron_hour(),
            sync_cron_minute: default_wiki_sync_cron_minute(),
            lint_cron_dow: default_wiki_lint_cron_dow(),
            lint_cron_hour: default_wiki_lint_cron_hour(),
            push_cron_hour: default_wiki_push_cron_hour(),
            push_cron_minute: default_wiki_push_cron_minute(),
            okforge: None,
        }
    }
}

impl WikiConfig {
    /// Absolute-ish paths relative to the agent working directory.
    pub fn entities_dir(&self) -> String {
        format!("{}/entities", self.directory)
    }
    pub fn concepts_dir(&self) -> String {
        format!("{}/concepts", self.directory)
    }
    pub fn sources_dir(&self) -> String {
        format!("{}/sources", self.directory)
    }
}
