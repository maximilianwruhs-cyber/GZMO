//! `gzmo corpus ingest-dir` — native corpus ingest into the separately
//! indexed FTS5 + Qdrant knowledge-collection store (see `gzmo_core::corpus`),
//! distinct from the promoted-fact vault used by `gzmo ingest-dir`.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Result};

use gzmo_core::config::GzmoConfig;
use gzmo_core::corpus::{CorpusIngestOptions, CorpusIngestReceipt, CorpusService};
use gzmo_core::memory::embeddings::Embedder;
use gzmo_core::memory::qdrant_recall::QdrantRecall;
use gzmo_core::memory::scratch::ScratchService;
use gzmo_core::memory::vault::SqliteVault;

const USAGE: &str = "Usage:\n  \
     gzmo corpus ingest-dir <path> [--json] [--defer-distill]\n\n\
     Ingests every .md/.txt file under <path> into a separately indexed \
     corpus (SQLite FTS5 + Qdrant knowledge collection) distinct from the \
     promoted-fact vault used by `gzmo ingest-dir`, then creates a session \
     transcript and enqueues a distill job unless --defer-distill is given.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestDirArgs {
    pub path: PathBuf,
    pub json: bool,
    pub defer_distill: bool,
}

/// Parse `gzmo corpus ingest-dir` subargs (everything after `ingest-dir`).
pub fn parse_ingest_dir_args(args: &[String]) -> Result<IngestDirArgs> {
    let mut path: Option<PathBuf> = None;
    let mut json = false;
    let mut defer_distill = false;
    for arg in args {
        match arg.as_str() {
            "--json" => json = true,
            "--defer-distill" => defer_distill = true,
            other if !other.starts_with('-') && path.is_none() => {
                path = Some(PathBuf::from(other));
            }
            other => bail!("unknown argument: {other}"),
        }
    }
    let path = path.ok_or_else(|| anyhow::anyhow!("missing <path> for corpus ingest-dir"))?;
    Ok(IngestDirArgs {
        path,
        json,
        defer_distill,
    })
}

/// Render the ingest receipt for CLI output (`--json` or human-readable).
pub fn render_receipt(receipt: &CorpusIngestReceipt, json: bool) -> Result<String> {
    if json {
        Ok(serde_json::to_string_pretty(receipt)?)
    } else {
        Ok(format!(
            "Corpus ingest complete: {} file(s), {} passage(s) ({} FTS, {} vector). \
             Session {} ({}).",
            receipt.source_files,
            receipt.passages,
            receipt.fts_indexed,
            receipt.vector_indexed,
            receipt.distill_session_id,
            if receipt.distill_enqueued {
                "distill enqueued"
            } else {
                "distill deferred"
            }
        ))
    }
}

async fn build_service(config: &GzmoConfig) -> Result<CorpusService> {
    if let Some(parent) = config.memory.vault_db.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    let vault = SqliteVault::open(&config.memory.vault_db)?;
    let embedder = Embedder::from_config(&config.embeddings, &config.redis)?;
    let qdrant = QdrantRecall::from_config(&config.qdrant)?
        .with_collection(config.platform_search.knowledge_collection.clone());
    let scratch = ScratchService::from_config(&config.redis, &config.context_memory).await;
    CorpusService::new(
        vault,
        embedder,
        Arc::new(qdrant),
        &config.session_distill.sessions_dir,
        scratch,
    )
}

async fn execute(config: &GzmoConfig, args: &IngestDirArgs) -> Result<CorpusIngestReceipt> {
    let service = build_service(config).await?;
    let options = CorpusIngestOptions {
        enqueue_distill: !args.defer_distill,
    };
    service.ingest_dir(&args.path, options).await
}

pub async fn run(config: &GzmoConfig, subargs: Vec<String>) -> Result<()> {
    // Usage errors (missing/unknown subcommand, missing path, unknown flag)
    // exit 2, matching `session_cmd.rs`'s explicit `std::process::exit(2)`
    // convention and the brief's "Unknown flags exit 2" requirement. This
    // must be a real process exit, not a `bail!`-propagated `Err` — under
    // `#[tokio::main] async fn main() -> Result<()>`, an `Err` return only
    // yields exit code 1.
    let Some(sub) = subargs.first().map(|s| s.as_str()) else {
        eprintln!("{USAGE}");
        eprintln!("error: missing corpus subcommand");
        std::process::exit(2);
    };
    if sub != "ingest-dir" {
        eprintln!("{USAGE}");
        eprintln!("error: unknown corpus subcommand: {sub}");
        std::process::exit(2);
    }
    let rest = &subargs[1..];
    let args = match parse_ingest_dir_args(rest) {
        Ok(args) => args,
        Err(err) => {
            eprintln!("{USAGE}");
            eprintln!("error: {err}");
            std::process::exit(2);
        }
    };
    // A failure here (unreachable embedder/Qdrant, vector-index mismatch,
    // etc.) propagates as `Err` — `main()` returns `Result<()>` under
    // `#[tokio::main]`, so this naturally produces a nonzero exit status
    // without needing an explicit `std::process::exit` call.
    let receipt = execute(config, &args).await?;
    println!("{}", render_receipt(&receipt, args.json)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ingest_dir_args_requires_a_path() {
        let err = parse_ingest_dir_args(&[]).unwrap_err();
        assert!(err.to_string().contains("missing <path>"));
    }

    #[test]
    fn parse_ingest_dir_args_parses_path_and_flags() {
        let args = parse_ingest_dir_args(&[
            "./corpus".to_string(),
            "--json".to_string(),
            "--defer-distill".to_string(),
        ])
        .unwrap();
        assert_eq!(args.path, PathBuf::from("./corpus"));
        assert!(args.json);
        assert!(args.defer_distill);
    }

    #[test]
    fn parse_ingest_dir_args_defaults_flags_to_false() {
        let args = parse_ingest_dir_args(&["./corpus".to_string()]).unwrap();
        assert!(!args.json);
        assert!(!args.defer_distill);
    }

    #[test]
    fn parse_ingest_dir_args_rejects_unknown_flag() {
        let err =
            parse_ingest_dir_args(&["./corpus".to_string(), "--bogus".to_string()]).unwrap_err();
        assert!(err.to_string().contains("unknown argument"));
    }

    fn sample_receipt() -> CorpusIngestReceipt {
        CorpusIngestReceipt {
            schema: "gzmo.corpus.ingest/v1".to_string(),
            source_files: 2,
            passages: 5,
            fts_indexed: 5,
            vector_indexed: 5,
            distill_session_id: "abcd1234".to_string(),
            distill_enqueued: true,
        }
    }

    #[test]
    fn render_receipt_json_round_trips_all_fields() {
        let receipt = sample_receipt();
        let rendered = render_receipt(&receipt, true).unwrap();
        let parsed: CorpusIngestReceipt = serde_json::from_str(&rendered).unwrap();
        assert_eq!(parsed.schema, receipt.schema);
        assert_eq!(parsed.source_files, receipt.source_files);
        assert_eq!(parsed.passages, receipt.passages);
        assert_eq!(parsed.fts_indexed, receipt.fts_indexed);
        assert_eq!(parsed.vector_indexed, receipt.vector_indexed);
        assert_eq!(parsed.distill_session_id, receipt.distill_session_id);
        assert_eq!(parsed.distill_enqueued, receipt.distill_enqueued);
    }

    #[test]
    fn render_receipt_plain_is_human_readable() {
        let receipt = sample_receipt();
        let rendered = render_receipt(&receipt, false).unwrap();
        assert!(rendered.contains("2 file(s)"));
        assert!(rendered.contains("5 passage(s)"));
        assert!(rendered.contains("abcd1234"));
        assert!(rendered.contains("distill enqueued"));
    }

    #[test]
    fn render_receipt_plain_shows_deferred_distill() {
        let mut receipt = sample_receipt();
        receipt.distill_enqueued = false;
        let rendered = render_receipt(&receipt, false).unwrap();
        assert!(rendered.contains("distill deferred"));
    }

    /// Behavioral proof (not just a parser message) that a real runtime/index
    /// failure (unreachable embedder + Qdrant) surfaces from `run()` as `Err`,
    /// which is what causes a nonzero process exit under `#[tokio::main]
    /// async fn main() -> Result<()>` — no explicit `std::process::exit` call
    /// is needed or wanted for this path.
    #[tokio::test]
    async fn run_propagates_index_failure_as_err_for_nonzero_exit() {
        use gzmo_core::config::{EmbeddingsConfig, GzmoConfig, QdrantConfig};

        let tmp = std::env::temp_dir().join(format!(
            "gzmo-corpus-cmd-run-fail-{}",
            std::process::id()
        ));
        let corpus_dir = tmp.join("corpus");
        std::fs::create_dir_all(&corpus_dir).expect("create corpus dir");
        std::fs::write(corpus_dir.join("note.md"), "# Note\n\nSome corpus content.")
            .expect("write corpus file");

        let mut config = GzmoConfig::default();
        config.memory.vault_db = tmp.join("vault.db");
        config.session_distill.sessions_dir = tmp.join("sessions");
        config.embeddings = EmbeddingsConfig {
            enabled: true,
            url: "http://127.0.0.1:1".to_string(),
            ..Default::default()
        };
        config.qdrant = QdrantConfig {
            url: "http://127.0.0.1:1".to_string(),
            ..Default::default()
        };

        let result = run(
            &config,
            vec!["ingest-dir".to_string(), corpus_dir.to_string_lossy().to_string()],
        )
        .await;

        assert!(
            result.is_err(),
            "expected unreachable embedder/Qdrant to surface as Err from run()"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
