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
    let Some(sub) = subargs.first().map(|s| s.as_str()) else {
        eprintln!("{USAGE}");
        bail!("missing corpus subcommand");
    };
    if sub != "ingest-dir" {
        eprintln!("{USAGE}");
        bail!("unknown corpus subcommand: {sub}");
    }
    let rest = &subargs[1..];
    let args = parse_ingest_dir_args(rest).inspect_err(|_| eprintln!("{USAGE}"))?;
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
}
