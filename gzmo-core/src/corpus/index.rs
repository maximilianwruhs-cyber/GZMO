//! Deterministic chunking + Qdrant vector upsert for corpus passages.

use std::sync::Arc;

use anyhow::Result;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::corpus::store::CorpusPassage;
use crate::memory::embeddings::Embedder;
use crate::memory::qdrant_recall::QdrantRecall;

/// Split `text` into overlapping chunks of at most `max_len` characters.
///
/// Operates on `Vec<char>` so chunk boundaries never split a UTF-8 code
/// point. Prefers to break on a paragraph boundary (`\n\n`), then on
/// whitespace, and only falls back to a hard character cut when neither is
/// available inside the window. Pure and deterministic: identical input
/// always yields identical output.
pub fn chunk_text(text: &str, max_len: usize, overlap: usize) -> Vec<String> {
    if max_len == 0 {
        return Vec::new();
    }
    let normalized = text.replace("\r\n", "\n");
    let chars: Vec<char> = normalized.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }
    if chars.len() <= max_len {
        let trimmed = normalized.trim();
        return if trimmed.is_empty() {
            Vec::new()
        } else {
            vec![trimmed.to_string()]
        };
    }

    let mut chunks = Vec::new();
    let mut start = 0usize;
    while start < chars.len() {
        let ideal_end = (start + max_len).min(chars.len());
        let end = if ideal_end >= chars.len() {
            ideal_end
        } else {
            find_break_point(&chars, start, ideal_end)
        };
        let chunk: String = chars[start..end].iter().collect();
        let trimmed = chunk.trim();
        if !trimmed.is_empty() {
            chunks.push(trimmed.to_string());
        }
        if end >= chars.len() {
            break;
        }
        // Back up by `overlap` chars for continuity, but always make
        // forward progress so we can never loop forever.
        start = end.saturating_sub(overlap).max(start + 1);
    }
    chunks
}

/// Find the best break point in `chars[start..ideal_end]`, preferring a
/// paragraph boundary, then trailing whitespace, else the raw char cut.
fn find_break_point(chars: &[char], start: usize, ideal_end: usize) -> usize {
    let mut best_para: Option<usize> = None;
    for i in start..ideal_end.saturating_sub(1) {
        if chars[i] == '\n' && chars[i + 1] == '\n' {
            best_para = Some(i + 2);
        }
    }
    if let Some(p) = best_para {
        if p > start && p <= ideal_end {
            return p;
        }
    }
    let mut j = ideal_end;
    while j > start {
        if chars[j - 1].is_whitespace() {
            return j;
        }
        j -= 1;
    }
    ideal_end
}

/// Build the Qdrant payload for a passage. Mirrors the `path`/`chunk`/`text`
/// shape already read by `platform_search`'s knowledge-collection search, plus
/// the stable `passage_id` so hits can be joined back to `corpus_passages`.
pub fn passage_payload(passage: &CorpusPassage) -> serde_json::Value {
    serde_json::json!({
        "passage_id": passage.id,
        "path": passage.source_path,
        "chunk": passage.chunk_index,
        "text": passage.content,
    })
}

/// Derive a deterministic Qdrant point UUID from a passage's stable string
/// id. Qdrant point ids must be an unsigned integer or a UUID, so the
/// `sha256:<hash>:<chunk_idx>` passage id cannot be used directly. Hashing
/// keeps the mapping stable and idempotent: re-ingesting the same passage id
/// always upserts the same point instead of duplicating it.
pub fn passage_point_id(passage_id: &str) -> Uuid {
    let digest = Sha256::digest(passage_id.as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    Uuid::from_bytes(bytes)
}

/// Embeds and upserts corpus passages into the configured Qdrant collection.
pub struct CorpusIndexer {
    embedder: Arc<Embedder>,
    qdrant: Arc<QdrantRecall>,
}

impl CorpusIndexer {
    pub fn new(embedder: Arc<Embedder>, qdrant: Arc<QdrantRecall>) -> Self {
        Self { embedder, qdrant }
    }

    /// Embed and upsert every passage, aborting on the first failure so
    /// partial vector-index writes are surfaced to the caller instead of
    /// being silently swallowed.
    pub async fn index_passages(&self, passages: &[CorpusPassage]) -> Result<usize> {
        let mut indexed = 0usize;
        for passage in passages {
            let vector = self.embedder.embed(&passage.content).await?;
            let point_id = passage_point_id(&passage.id);
            self.qdrant
                .upsert_point(point_id, &vector, passage_payload(passage))
                .await?;
            indexed += 1;
        }
        Ok(indexed)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn chunks_are_stable_and_overlap() {
        let text = "alpha ".repeat(500);
        let first = super::chunk_text(&text, 1200, 150);
        let second = super::chunk_text(&text, 1200, 150);
        assert_eq!(first, second);
        assert!(first.len() > 1);
        assert!(first[0].len() <= 1200);
    }

    #[test]
    fn short_text_is_a_single_chunk() {
        let chunks = super::chunk_text("just one short passage", 1200, 150);
        assert_eq!(chunks, vec!["just one short passage".to_string()]);
    }

    #[test]
    fn empty_text_yields_no_chunks() {
        assert!(super::chunk_text("", 1200, 150).is_empty());
        assert!(super::chunk_text("   \n\n  ", 1200, 150).is_empty());
    }

    #[test]
    fn passage_payload_carries_path_chunk_text_and_passage_id() {
        let passage = super::super::store::CorpusPassage {
            id: "sha256:abc:2".into(),
            source_path: "notes/lantern.md".into(),
            chunk_index: 2,
            content: "cobalt finch".into(),
            content_sha256: "abc".into(),
        };
        let payload = super::passage_payload(&passage);
        assert_eq!(payload["passage_id"], "sha256:abc:2");
        assert_eq!(payload["path"], "notes/lantern.md");
        assert_eq!(payload["chunk"], 2);
        assert_eq!(payload["text"], "cobalt finch");
    }

    #[test]
    fn passage_point_id_is_deterministic_and_distinct() {
        let a = super::passage_point_id("sha256:abc:0");
        let b = super::passage_point_id("sha256:abc:0");
        let c = super::passage_point_id("sha256:abc:1");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[tokio::test]
    async fn index_passages_fails_fast_when_embedder_is_unreachable() {
        use crate::config::{EmbeddingsConfig, QdrantConfig, RedisConfig};
        use crate::memory::embeddings::Embedder;
        use crate::memory::qdrant_recall::QdrantRecall;

        let embeddings_cfg = EmbeddingsConfig {
            enabled: true,
            url: "http://127.0.0.1:1".to_string(),
            ..Default::default()
        };
        let redis_cfg = RedisConfig {
            enabled: false,
            ..Default::default()
        };
        let embedder = Embedder::from_config(&embeddings_cfg, &redis_cfg).expect("embedder");
        let qdrant_cfg = QdrantConfig {
            url: "http://127.0.0.1:1".to_string(),
            ..Default::default()
        };
        let qdrant = QdrantRecall::from_config(&qdrant_cfg).expect("qdrant");
        let indexer = super::CorpusIndexer::new(embedder, qdrant);
        let passages = vec![super::super::store::CorpusPassage {
            id: "sha256:abc:0".into(),
            source_path: "notes.md".into(),
            chunk_index: 0,
            content: "unreachable embedder should fail fast".into(),
            content_sha256: "abc".into(),
        }];
        let result = indexer.index_passages(&passages).await;
        assert!(result.is_err());
    }
}
