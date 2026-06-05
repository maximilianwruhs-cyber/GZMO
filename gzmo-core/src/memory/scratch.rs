//! Ephemeral scratch cache (Redis or in-memory) + distill job queue.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::config::{ContextMemoryConfig, RedisConfig};
use crate::context::estimate_text_tokens;
use crate::types::{Message, Role};

const SCRATCH_PREFIX: &str = "gzmo:scratch:";

/// Which agent owns this scratch pad.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScratchScope {
    Main { session_id: String },
    Sub { session_id: String, task_id: String },
    Orch { job: String, step: String },
}

impl ScratchScope {
    pub fn redis_key(&self) -> String {
        match self {
            Self::Main { session_id } => format!("{SCRATCH_PREFIX}main:{session_id}"),
            Self::Sub { session_id, task_id } => {
                format!("{SCRATCH_PREFIX}sub:{session_id}:{task_id}")
            }
            Self::Orch { job, step } => format!("{SCRATCH_PREFIX}orch:{job}:{step}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallSnippet {
    pub content: String,
    pub score: f32,
    #[serde(default)]
    pub fact_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScratchPayload {
    pub snippets: Vec<RecallSnippet>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DistillSource {
    MainArchive,
    SubArchive { task_id: String, role: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillJob {
    pub session_id: String,
    pub transcript: String,
    pub source: DistillSource,
}

enum ScratchBackend {
    Redis(redis::aio::ConnectionManager),
    Memory(Arc<RwLock<HashMap<String, ScratchPayload>>>),
}

/// Scratch read/write + distill enqueue with Redis fallback.
pub struct ScratchService {
    backend: ScratchBackend,
    redis_enabled: bool,
    distill_queue: String,
    distill_fallback_dir: PathBuf,
    scratch_max_tokens: usize,
    chars_per_token: f64,
}

impl ScratchService {
    pub async fn from_config(redis_cfg: &RedisConfig, ctx_mem: &ContextMemoryConfig) -> Self {
        let scratch_max_tokens = ctx_mem.scratch_max_tokens;
        let distill_queue = redis_cfg.distill_queue.clone();
        let distill_fallback_dir = redis_cfg.distill_fallback_dir.clone();

        if redis_cfg.enabled {
            match redis::Client::open(redis_cfg.url.as_str()) {
                Ok(client) => match client.get_connection_manager().await {
                    Ok(conn) => {
                        debug!(url = %redis_cfg.url, "ScratchService using Redis");
                        return Self {
                            backend: ScratchBackend::Redis(conn),
                            redis_enabled: true,
                            distill_queue,
                            distill_fallback_dir,
                            scratch_max_tokens,
                            chars_per_token: 3.5,
                        };
                    }
                    Err(e) => {
                        warn!("Redis connection failed, using in-memory scratch: {e}");
                    }
                },
                Err(e) => {
                    warn!("Redis client open failed, using in-memory scratch: {e}");
                }
            }
        }

        Self {
            backend: ScratchBackend::Memory(Arc::new(RwLock::new(HashMap::new()))),
            redis_enabled: false,
            distill_queue,
            distill_fallback_dir,
            scratch_max_tokens,
            chars_per_token: 3.5,
        }
    }

    pub fn uses_redis(&self) -> bool {
        self.redis_enabled
    }

    pub async fn clear(&self, scope: &ScratchScope) -> Result<()> {
        let key = scope.redis_key();
        match &self.backend {
            ScratchBackend::Redis(conn) => {
                let mut conn = conn.clone();
                let _: () = conn.del(&key).await.context("redis DEL scratch")?;
            }
            ScratchBackend::Memory(map) => {
                map.write().await.remove(&key);
            }
        }
        Ok(())
    }

    pub async fn write(&self, scope: &ScratchScope, snippets: Vec<RecallSnippet>) -> Result<()> {
        if snippets.is_empty() {
            return self.clear(scope).await;
        }
        let payload = ScratchPayload {
            snippets,
            updated_at: Utc::now(),
        };
        let key = scope.redis_key();
        let json = serde_json::to_string(&payload)?;
        match &self.backend {
            ScratchBackend::Redis(conn) => {
                let mut conn = conn.clone();
                let _: () = conn.set(&key, json).await.context("redis SET scratch")?;
            }
            ScratchBackend::Memory(map) => {
                map.write().await.insert(key, payload);
            }
        }
        Ok(())
    }

    pub async fn read(&self, scope: &ScratchScope) -> Result<Option<ScratchPayload>> {
        let key = scope.redis_key();
        match &self.backend {
            ScratchBackend::Redis(conn) => {
                let mut conn = conn.clone();
                let val: Option<String> = conn.get(&key).await.context("redis GET scratch")?;
                Ok(val
                    .map(|s| serde_json::from_str(&s))
                    .transpose()
                    .context("parse scratch JSON")?)
            }
            ScratchBackend::Memory(map) => Ok(map.read().await.get(&key).cloned()),
        }
    }

    /// Format scratch for injection as a meta system block.
    pub async fn format_for_inject(&self, scope: &ScratchScope) -> Result<Option<String>> {
        let Some(payload) = self.read(scope).await? else {
            return Ok(None);
        };
        if payload.snippets.is_empty() {
            return Ok(None);
        }

        let mut lines = vec!["[RECALL]".to_string()];
        let mut used = estimate_text_tokens("[RECALL]\n", self.chars_per_token);

        for snip in &payload.snippets {
            let line = if let Some(ref id) = snip.fact_id {
                format!("- [{:.2}] ({}) {}", snip.score, id, snip.content)
            } else {
                format!("- [{:.2}] {}", snip.score, snip.content)
            };
            let cost = estimate_text_tokens(&line, self.chars_per_token);
            if used + cost > self.scratch_max_tokens {
                break;
            }
            used += cost;
            lines.push(line);
        }

        Ok(Some(lines.join("\n")))
    }

    pub async fn enqueue_distill(&self, job: DistillJob) -> Result<()> {
        let json = serde_json::to_string(&job)?;
        if self.redis_enabled {
            if let ScratchBackend::Redis(conn) = &self.backend {
                let mut conn = conn.clone();
                let _: usize = conn
                    .lpush(&self.distill_queue, json)
                    .await
                    .context("redis LPUSH distill")?;
                return Ok(());
            }
        }
        self.enqueue_distill_file(&json).await
    }

    async fn enqueue_distill_file(&self, json: &str) -> Result<()> {
        tokio::fs::create_dir_all(&self.distill_fallback_dir)
            .await
            .context("create distill fallback dir")?;
        let id = uuid::Uuid::new_v4();
        let path = self.distill_fallback_dir.join(format!("{id}.json"));
        tokio::fs::write(&path, json)
            .await
            .context("write distill fallback job")?;
        debug!(path = %path.display(), "Distill job queued to file");
        Ok(())
    }

    /// Pop one job: Redis BRPOP or oldest file in fallback dir.
    pub async fn pop_distill_job(&self, timeout_secs: f64) -> Result<Option<DistillJob>> {
        if self.redis_enabled {
            if let ScratchBackend::Redis(conn) = &self.backend {
                let mut conn = conn.clone();
                let result: Option<(String, String)> = redis::cmd("BRPOP")
                    .arg(&self.distill_queue)
                    .arg(timeout_secs)
                    .query_async(&mut conn)
                    .await
                    .context("redis BRPOP distill")?;
                if let Some((_, json)) = result {
                    let job: DistillJob = serde_json::from_str(&json)?;
                    return Ok(Some(job));
                }
            }
        }
        self.pop_distill_file().await
    }

    async fn pop_distill_file(&self) -> Result<Option<DistillJob>> {
        let mut entries = tokio::fs::read_dir(&self.distill_fallback_dir)
            .await
            .ok();
        let Some(ref mut rd) = entries else {
            return Ok(None);
        };
        let mut paths = Vec::new();
        while let Some(entry) = rd.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                paths.push(path);
            }
        }
        paths.sort();
        let Some(path) = paths.into_iter().next() else {
            return Ok(None);
        };
        let json = tokio::fs::read_to_string(&path).await?;
        let _ = tokio::fs::remove_file(&path).await;
        let job: DistillJob = serde_json::from_str(&json)?;
        Ok(Some(job))
    }
}

/// Build a transcript string from archived messages for distill pipeline.
pub fn messages_to_transcript(messages: &[Message]) -> String {
    let mut out = String::new();
    for msg in messages {
        if msg.is_meta && msg.role != Role::Tool {
            continue;
        }
        let role = match msg.role {
            Role::User => "User",
            Role::Assistant => "Assistant",
            Role::System => "System",
            Role::Tool => "Tool",
        };
        out.push_str(&format!("{role}: {}\n\n", msg.content));
    }
    out
}
