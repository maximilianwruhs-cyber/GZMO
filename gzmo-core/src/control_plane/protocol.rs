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
