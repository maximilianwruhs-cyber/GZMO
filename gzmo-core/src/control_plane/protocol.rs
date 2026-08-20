//! One JSON object per line (NDJSON), one request per connection.

use serde::{Deserialize, Serialize};

use crate::memory::profile::GzmoProfile;
use crate::platform_memory::{MemorySearchResult, MemoryStatusReport};

pub const VIA_OWNER: &str = "owner";
pub const VIA_IN_PROCESS: &str = "in-process";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlRequest {
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write_scratch: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fact_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamic_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingBody {
    pub pid: u32,
    pub vault_path: String,
    pub socket_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainEntry {
    pub content: String,
    pub latest: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_rel: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResponse {
    pub ok: bool,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ping: Option<PingBody>,
    /// `items[].kind`/`retrieval_channels` (corpus_passage vs promoted_fact,
    /// fts/vector) ride along transparently — `MemoryHit` derives
    /// Serialize/Deserialize, so no bespoke NDJSON mapping is needed here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search: Option<MemorySearchResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<MemoryStatusReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recall: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain: Option<Vec<ChainEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<GzmoProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_start: Option<String>,
}

impl ControlResponse {
    pub fn err(method: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            ok: false,
            method: method.into(),
            error: Some(error.into()),
            via: Some(VIA_OWNER.to_string()),
            ping: None,
            search: None,
            status: None,
            recall: None,
            chain: None,
            profile: None,
            turn_start: None,
        }
    }

    pub fn ok_method(method: impl Into<String>) -> Self {
        Self {
            ok: true,
            method: method.into(),
            error: None,
            via: Some(VIA_OWNER.to_string()),
            ping: None,
            search: None,
            status: None,
            recall: None,
            chain: None,
            profile: None,
            turn_start: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform_memory::{MemoryHit, MemoryHitKind, RetrievalChannel};

    /// Pure NDJSON round-trip (no socket): a labeled corpus_passage hit and a
    /// promoted_fact hit must both survive `ControlResponse` serialization
    /// with their `kind`/`retrieval_channels` intact and distinguishable.
    #[test]
    fn control_response_search_round_trips_labeled_hits() {
        let mut resp = ControlResponse::ok_method("memory.search");
        resp.search = Some(MemorySearchResult {
            query: "cobalt finch".into(),
            hits: 2,
            items: vec![
                MemoryHit {
                    kind: MemoryHitKind::CorpusPassage,
                    retrieval_channels: vec![RetrievalChannel::Fts, RetrievalChannel::Vector],
                    content: "[corpus:orion-lantern.md#chunk0] cobalt finch 731".into(),
                    score: 1.2,
                    source_file: Some("orion-lantern.md".into()),
                    fact_id: None,
                    evidence_text: None,
                },
                MemoryHit {
                    kind: MemoryHitKind::PromotedFact,
                    retrieval_channels: Vec::new(),
                    content: "operator prefers dark roast".into(),
                    score: 0.9,
                    source_file: None,
                    fact_id: Some(uuid::Uuid::new_v4()),
                    evidence_text: Some("evidence".into()),
                },
            ],
            text: "Platform recall for 'cobalt finch'".into(),
            scratch_written: false,
        });

        let line = serde_json::to_string(&resp).expect("encode NDJSON line");
        let decoded: ControlResponse = serde_json::from_str(&line).expect("decode NDJSON line");
        let items = &decoded.search.expect("search body").items;

        assert_eq!(items[0].kind, MemoryHitKind::CorpusPassage);
        assert_eq!(
            items[0].retrieval_channels,
            vec![RetrievalChannel::Fts, RetrievalChannel::Vector]
        );
        assert_eq!(items[1].kind, MemoryHitKind::PromotedFact);
        assert!(items[1].fact_id.is_some());
        assert!(line.contains("corpus_passage"));
        assert!(line.contains("promoted_fact"));
    }
}
