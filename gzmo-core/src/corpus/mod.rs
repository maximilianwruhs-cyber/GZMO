//! Native corpus ingest: separately indexed folder passages (FTS5 + Qdrant),
//! distinct from the promoted-fact vault. Upstream prerequisite for
//! `docs/superpowers/specs/2026-08-20-gzmo-demo-design.md`.

pub mod index;
pub mod store;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::corpus::index::{chunk_text, CorpusIndexer};
use crate::corpus::store::{CorpusPassage, CorpusStore};
use crate::memory::embeddings::Embedder;
use crate::memory::qdrant_recall::QdrantRecall;
use crate::memory::scratch::{messages_to_transcript, DistillJob, DistillSource, ScratchService};
use crate::memory::vault::SqliteVault;
use crate::session::SessionManager;
use crate::types::{Message, Role};

/// JSON receipt schema tag for `gzmo corpus ingest-dir --json`.
pub const CORPUS_INGEST_SCHEMA: &str = "gzmo.corpus.ingest/v1";

/// Character length/overlap used for deterministic passage chunking.
const CHUNK_MAX_LEN: usize = 1200;
const CHUNK_OVERLAP: usize = 150;

/// Options controlling one `CorpusService::ingest_dir` call.
#[derive(Debug, Clone, Copy)]
pub struct CorpusIngestOptions {
    /// Enqueue a `DistillJob` for the created session (default: true).
    /// `--defer-distill` on the CLI sets this to false.
    pub enqueue_distill: bool,
}

impl Default for CorpusIngestOptions {
    fn default() -> Self {
        Self {
            enqueue_distill: true,
        }
    }
}

/// JSON receipt returned by `gzmo corpus ingest-dir --json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusIngestReceipt {
    pub schema: String,
    pub source_files: usize,
    pub passages: usize,
    pub fts_indexed: usize,
    pub vector_indexed: usize,
    pub distill_session_id: String,
    pub distill_enqueued: bool,
}

/// Orchestrates native corpus ingest: chunk → FTS store → Qdrant vectors →
/// session transcript → optional distill enqueue.
pub struct CorpusService {
    store: CorpusStore,
    indexer: CorpusIndexer,
    sessions: SessionManager,
    scratch: ScratchService,
}

impl CorpusService {
    pub fn new(
        vault: SqliteVault,
        embedder: Arc<Embedder>,
        qdrant: Arc<QdrantRecall>,
        sessions_dir: impl AsRef<Path>,
        scratch: ScratchService,
    ) -> Result<Self> {
        let store = CorpusStore::new(vault)?;
        let indexer = CorpusIndexer::new(embedder, qdrant);
        let sessions = SessionManager::new(sessions_dir);
        Ok(Self {
            store,
            indexer,
            sessions,
            scratch,
        })
    }

    /// Ingest every `.md`/`.txt` file under `root` (recursively) into the
    /// separate corpus FTS store and the configured Qdrant knowledge
    /// collection, then create a normal GZMO session transcript and
    /// (by default) enqueue a `DistillJob` for it.
    pub async fn ingest_dir(
        &self,
        root: impl AsRef<Path>,
        options: CorpusIngestOptions,
    ) -> Result<CorpusIngestReceipt> {
        let root = root.as_ref();
        let canonical_root = root
            .canonicalize()
            .with_context(|| format!("Corpus ingest root not found: {}", root.display()))?;

        let mut files = collect_corpus_files(&canonical_root)?;
        files.sort();

        if files.is_empty() {
            bail!(
                "No .md/.txt files found under {}",
                canonical_root.display()
            );
        }

        // Reject any file that resolves (e.g. via a symlink) outside the
        // root *before* any side effects (FTS/vector writes, session save).
        for file in &files {
            let canon = file
                .canonicalize()
                .with_context(|| format!("Failed to canonicalize {}", file.display()))?;
            if !canon.starts_with(&canonical_root) {
                bail!(
                    "Refusing to ingest {} — escapes root {}",
                    file.display(),
                    canonical_root.display()
                );
            }
        }

        let mut passages = Vec::new();
        let mut transcript_sections = Vec::new();
        for file in &files {
            let content = std::fs::read_to_string(file)
                .with_context(|| format!("Failed to read {}", file.display()))?;
            let content_sha256 = format!("{:x}", Sha256::digest(content.as_bytes()));
            let rel_path = file
                .strip_prefix(&canonical_root)
                .unwrap_or(file)
                .to_string_lossy()
                .replace('\\', "/");

            let chunks = chunk_text(&content, CHUNK_MAX_LEN, CHUNK_OVERLAP);
            for (idx, chunk) in chunks.into_iter().enumerate() {
                passages.push(CorpusPassage {
                    id: format!("sha256:{content_sha256}:{idx}"),
                    source_path: rel_path.clone(),
                    chunk_index: idx,
                    content: chunk,
                    content_sha256: content_sha256.clone(),
                });
            }
            transcript_sections.push(format!("## {rel_path}\n\n{content}"));
        }

        for passage in &passages {
            self.store.upsert(passage)?;
        }
        let fts_indexed = passages.len();

        let vector_indexed = self.indexer.index_passages(&passages).await?;
        if vector_indexed != passages.len() {
            bail!(
                "Vector index count mismatch: expected {} got {}",
                passages.len(),
                vector_indexed
            );
        }

        let session_id = SessionManager::new_session_id();
        let created_at = Utc::now();
        let messages = vec![Message {
            role: Role::User,
            content: format!(
                "Corpus ingest of {} — {} source file(s), {} passage(s).\n\n{}",
                canonical_root.display(),
                files.len(),
                passages.len(),
                transcript_sections.join("\n\n---\n\n")
            ),
            is_meta: false,
            tool_calls: None,
            tool_call_id: None,
        }];
        self.sessions
            .save(&session_id, None, &messages, created_at)
            .await?;

        let mut distill_enqueued = false;
        if options.enqueue_distill {
            let transcript = messages_to_transcript(&messages);
            self.scratch
                .enqueue_distill(DistillJob {
                    session_id: session_id.clone(),
                    transcript,
                    source: DistillSource::MainArchive,
                })
                .await?;
            distill_enqueued = true;
        }

        Ok(CorpusIngestReceipt {
            schema: CORPUS_INGEST_SCHEMA.to_string(),
            source_files: files.len(),
            passages: passages.len(),
            fts_indexed,
            vector_indexed,
            distill_session_id: session_id,
            distill_enqueued,
        })
    }
}

/// Recursively collect `.md`/`.txt` files under `root`.
///
/// `DirEntry::file_type()` reports the type of the entry itself and does not
/// follow symlinks, so a symlinked file would otherwise be silently skipped
/// here instead of being caught by `ingest_dir`'s root-escape check. Symlink
/// entries are resolved via `Path::metadata` (which does follow links) so
/// they are collected like any other file and then validated for escape.
fn collect_corpus_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in
            std::fs::read_dir(&dir).with_context(|| format!("Failed to read dir {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;
            let (is_dir, is_file) = if file_type.is_symlink() {
                match path.metadata() {
                    Ok(meta) => (meta.is_dir(), meta.is_file()),
                    Err(_) => (false, false), // broken symlink: skip
                }
            } else {
                (file_type.is_dir(), file_type.is_file())
            };
            if is_dir {
                stack.push(path);
            } else if is_file {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("txt") {
                        out.push(path);
                    }
                }
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    /// Minimal hand-rolled HTTP/1.1 mock: ignores the request entirely and
    /// always replies with `body` and `status_line`. Avoids adding a mocking
    /// dependency (e.g. httpmock) purely for this one integration test —
    /// matches the existing repo convention of pure-function unit tests plus
    /// real network behavior (see `index_passages_fails_fast_when_embedder_is_unreachable`).
    async fn spawn_canned_http_server(status_line: &'static str, body: String) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind mock listener");
        let addr = listener.local_addr().expect("mock listener addr");
        tokio::spawn(async move {
            loop {
                let (mut socket, _) = match listener.accept().await {
                    Ok(pair) => pair,
                    Err(_) => break,
                };
                let body = body.clone();
                tokio::spawn(async move {
                    let mut buf = [0u8; 4096];
                    let _ = tokio::time::timeout(
                        std::time::Duration::from_millis(200),
                        socket.read(&mut buf),
                    )
                    .await;
                    let response = format!(
                        "{status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                    let _ = socket.shutdown().await;
                });
            }
        });
        addr
    }

    struct TestHarness {
        tmp_dir: PathBuf,
        service: CorpusService,
        corpus_dir: PathBuf,
    }

    impl Drop for TestHarness {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.tmp_dir);
        }
    }

    async fn build_harness(embed_status: &'static str, qdrant_status: &'static str) -> TestHarness {
        use crate::config::{ContextMemoryConfig, EmbeddingsConfig, QdrantConfig, RedisConfig};

        let tmp_dir = std::env::temp_dir().join(format!("gzmo-corpus-ingest-test-{}", uuid::Uuid::new_v4()));
        let corpus_dir = tmp_dir.join("corpus");
        std::fs::create_dir_all(&corpus_dir).expect("mkdir corpus");

        let embed_addr =
            spawn_canned_http_server(embed_status, serde_json::json!({"data": [{"embedding": [0.1, 0.2, 0.3]}]}).to_string())
                .await;
        let qdrant_addr = spawn_canned_http_server(qdrant_status, "{}".to_string()).await;

        let embeddings_cfg = EmbeddingsConfig {
            enabled: true,
            url: format!("http://{embed_addr}/v1"),
            ..Default::default()
        };
        let redis_cfg = RedisConfig {
            enabled: false,
            distill_fallback_dir: tmp_dir.join("distill-queue"),
            ..Default::default()
        };
        let embedder = Embedder::from_config(&embeddings_cfg, &redis_cfg).expect("embedder");

        let qdrant_cfg = QdrantConfig {
            url: format!("http://{qdrant_addr}"),
            collection: "knowledge".into(),
            ..Default::default()
        };
        let qdrant = QdrantRecall::from_config(&qdrant_cfg).expect("qdrant");

        let scratch = ScratchService::from_config(&redis_cfg, &ContextMemoryConfig::default()).await;

        let vault_path = tmp_dir.join("vault.db");
        let vault = SqliteVault::open(&vault_path).expect("vault");
        let sessions_dir = tmp_dir.join("sessions");

        let service = CorpusService::new(vault, embedder, qdrant, &sessions_dir, scratch)
            .expect("corpus service");

        TestHarness {
            tmp_dir,
            service,
            corpus_dir,
        }
    }

    #[tokio::test]
    async fn ingest_dir_indexes_files_and_enqueues_distill_by_default() {
        let harness = build_harness("HTTP/1.1 200 OK", "HTTP/1.1 200 OK").await;
        std::fs::write(
            harness.corpus_dir.join("orion-lantern.md"),
            "The calibration phrase is cobalt finch 731.",
        )
        .unwrap();
        std::fs::create_dir_all(harness.corpus_dir.join("sub")).unwrap();
        std::fs::write(
            harness.corpus_dir.join("sub").join("notes.txt"),
            "A second passage lives here.",
        )
        .unwrap();

        let receipt = harness
            .service
            .ingest_dir(&harness.corpus_dir, CorpusIngestOptions::default())
            .await
            .expect("ingest_dir should succeed");

        assert_eq!(receipt.schema, CORPUS_INGEST_SCHEMA);
        assert_eq!(receipt.source_files, 2);
        assert_eq!(receipt.passages, 2);
        assert_eq!(receipt.fts_indexed, 2);
        assert_eq!(receipt.vector_indexed, 2);
        assert!(receipt.distill_enqueued);
        assert!(!receipt.distill_session_id.is_empty());
    }

    #[tokio::test]
    async fn ingest_dir_can_defer_distill() {
        let harness = build_harness("HTTP/1.1 200 OK", "HTTP/1.1 200 OK").await;
        std::fs::write(harness.corpus_dir.join("a.md"), "solo passage content").unwrap();

        let receipt = harness
            .service
            .ingest_dir(
                &harness.corpus_dir,
                CorpusIngestOptions {
                    enqueue_distill: false,
                },
            )
            .await
            .expect("ingest_dir should succeed");

        assert!(!receipt.distill_enqueued);
    }

    #[tokio::test]
    async fn ingest_dir_fails_when_qdrant_upsert_errors() {
        let harness = build_harness("HTTP/1.1 200 OK", "HTTP/1.1 500 Internal Server Error").await;
        std::fs::write(harness.corpus_dir.join("a.md"), "content that will fail to vector-index").unwrap();

        let result = harness
            .service
            .ingest_dir(&harness.corpus_dir, CorpusIngestOptions::default())
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn ingest_dir_rejects_empty_directory() {
        let harness = build_harness("HTTP/1.1 200 OK", "HTTP/1.1 200 OK").await;
        let result = harness
            .service
            .ingest_dir(&harness.corpus_dir, CorpusIngestOptions::default())
            .await;
        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn ingest_dir_rejects_symlink_escaping_root() {
        let harness = build_harness("HTTP/1.1 200 OK", "HTTP/1.1 200 OK").await;
        let outside_dir = std::env::temp_dir().join(format!("gzmo-corpus-outside-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&outside_dir).expect("mkdir outside");
        let secret = outside_dir.join("secret.md");
        std::fs::write(&secret, "outside the root").unwrap();

        std::fs::write(harness.corpus_dir.join("inside.md"), "inside the root").unwrap();
        std::os::unix::fs::symlink(&secret, harness.corpus_dir.join("escape.md")).unwrap();

        let result = harness
            .service
            .ingest_dir(&harness.corpus_dir, CorpusIngestOptions::default())
            .await;

        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&outside_dir);
    }
}
