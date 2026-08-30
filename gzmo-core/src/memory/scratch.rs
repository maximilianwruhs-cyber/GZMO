//! Ephemeral scratch cache (Redis or in-memory) + distill job queue.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

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
            Self::Sub {
                session_id,
                task_id,
            } => {
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
    #[serde(default)]
    pub evidence_text: Option<String>,
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

/// Bound the initial/reconnect attempt so a dead Redis can't stall startup
/// or a scratch op on the driver's internal connect timeout.
const REDIS_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
/// Minimum gap between reconnect attempts while Redis is unreachable, so a
/// persistent outage degrades to the in-memory buffer instead of hammering
/// the connect path (and log) on every turn.
const REDIS_RECONNECT_BACKOFF: Duration = Duration::from_secs(15);

/// Redis-backed scratch that lazily (re)establishes its connection.
///
/// `redis::aio::ConnectionManager` already auto-reconnects once it exists, so
/// the failure mode this guards against is the manager never being created —
/// e.g. Redis briefly down during daemon startup. Instead of permanently
/// pinning the service to memory, we retain the client and re-attempt the
/// manager on demand (rate-limited by `next_retry`). While Redis is
/// unreachable, writes are buffered in `fallback` so the current turn's
/// scratch is not lost; reads prefer Redis and fall back to the buffer.
struct RedisBackend {
    client: redis::Client,
    conn: Mutex<Option<redis::aio::ConnectionManager>>,
    next_retry: Mutex<Instant>,
    fallback: RwLock<HashMap<String, ScratchPayload>>,
}

impl RedisBackend {
    fn new(client: redis::Client, conn: Option<redis::aio::ConnectionManager>) -> Self {
        Self {
            client,
            conn: Mutex::new(conn),
            next_retry: Mutex::new(Instant::now()),
            fallback: RwLock::new(HashMap::new()),
        }
    }

    /// Return a live connection manager, lazily (re)connecting if needed.
    /// Reconnect attempts are rate-limited and time-bounded; on a real
    /// (non-backoff) attempt failure we log once so the outage is visible.
    async fn conn(&self) -> Result<redis::aio::ConnectionManager> {
        if let Some(c) = self.conn.lock().await.as_ref() {
            return Ok(c.clone());
        }
        {
            let mut next = self.next_retry.lock().await;
            if Instant::now() < *next {
                anyhow::bail!("redis reconnect backing off");
            }
            *next = Instant::now() + REDIS_RECONNECT_BACKOFF;
        }
        match tokio::time::timeout(REDIS_CONNECT_TIMEOUT, self.client.get_connection_manager())
            .await
        {
            Ok(Ok(conn)) => {
                info!("Redis scratch backend connected");
                *self.conn.lock().await = Some(conn.clone());
                Ok(conn)
            }
            Ok(Err(e)) => {
                warn!(
                    "Redis scratch unreachable (using in-memory buffer, retry in {}s): {e}",
                    REDIS_RECONNECT_BACKOFF.as_secs()
                );
                anyhow::bail!("redis reconnect failed: {e}")
            }
            Err(_) => {
                warn!(
                    "Redis scratch connect timed out (using in-memory buffer, retry in {}s)",
                    REDIS_RECONNECT_BACKOFF.as_secs()
                );
                anyhow::bail!("redis connect timed out")
            }
        }
    }

    /// Drop the cached manager so the next op forces a fresh connect.
    async fn drop_conn(&self) {
        *self.conn.lock().await = None;
    }

    /// True if a command round-trips to Redis right now.
    async fn live(&self) -> bool {
        match self.conn().await {
            Ok(mut c) => {
                let pong: redis::RedisResult<String> = redis::cmd("PING").query_async(&mut c).await;
                if pong.is_err() {
                    self.drop_conn().await;
                }
                pong.is_ok()
            }
            Err(_) => false,
        }
    }
}

enum ScratchBackend {
    Redis(RedisBackend),
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
    #[cfg(test)]
    fn memory(scratch_max_tokens: usize) -> Self {
        Self {
            backend: ScratchBackend::Memory(Arc::new(RwLock::new(HashMap::new()))),
            redis_enabled: false,
            distill_queue: String::new(),
            distill_fallback_dir: PathBuf::new(),
            scratch_max_tokens,
            chars_per_token: 3.5,
        }
    }

    pub async fn from_config(redis_cfg: &RedisConfig, ctx_mem: &ContextMemoryConfig) -> Self {
        let scratch_max_tokens = ctx_mem.scratch_max_tokens;
        let distill_queue = redis_cfg.distill_queue.clone();
        let distill_fallback_dir = redis_cfg.distill_fallback_dir.clone();

        if redis_cfg.enabled {
            match redis::Client::open(redis_cfg.url.as_str()) {
                Ok(client) => {
                    // Try to connect up front, but a startup failure no longer
                    // pins us to memory forever: keep the client and reconnect
                    // lazily (see RedisBackend::conn).
                    let conn = match tokio::time::timeout(
                        REDIS_CONNECT_TIMEOUT,
                        client.get_connection_manager(),
                    )
                    .await
                    {
                        Ok(Ok(conn)) => {
                            info!(url = %redis_cfg.url, "ScratchService using Redis");
                            Some(conn)
                        }
                        Ok(Err(e)) => {
                            error!(
                                url = %redis_cfg.url,
                                "Redis enabled but unreachable at startup; \
                                 buffering scratch in-memory and retrying lazily: {e}"
                            );
                            None
                        }
                        Err(_) => {
                            error!(
                                url = %redis_cfg.url,
                                "Redis enabled but connect timed out at startup; \
                                 buffering scratch in-memory and retrying lazily"
                            );
                            None
                        }
                    };
                    return Self {
                        backend: ScratchBackend::Redis(RedisBackend::new(client, conn)),
                        redis_enabled: true,
                        distill_queue,
                        distill_fallback_dir,
                        scratch_max_tokens,
                        chars_per_token: 3.5,
                    };
                }
                Err(e) => {
                    error!(
                        url = %redis_cfg.url,
                        "Redis client open failed (bad URL?), using in-memory scratch: {e}"
                    );
                }
            }
        } else {
            debug!("Redis disabled in config; using in-memory scratch");
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

    /// Whether Redis is the configured scratch backend (intent). This stays
    /// true across transient outages; use [`redis_live`](Self::redis_live)
    /// for current connectivity.
    pub fn uses_redis(&self) -> bool {
        self.redis_enabled
    }

    /// Whether a command round-trips to Redis right now (live connectivity).
    pub async fn redis_live(&self) -> bool {
        match &self.backend {
            ScratchBackend::Redis(r) => r.live().await,
            ScratchBackend::Memory(_) => false,
        }
    }

    pub async fn clear(&self, scope: &ScratchScope) -> Result<()> {
        let key = scope.redis_key();
        match &self.backend {
            ScratchBackend::Redis(r) => {
                r.fallback.write().await.remove(&key);
                if let Ok(mut conn) = r.conn().await {
                    if let Err(e) = conn.del::<_, ()>(&key).await {
                        r.drop_conn().await;
                        debug!("redis DEL scratch failed (cleared buffer anyway): {e}");
                    }
                }
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
        match &self.backend {
            ScratchBackend::Redis(r) => {
                let json = serde_json::to_string(&payload)?;
                match r.conn().await {
                    Ok(mut conn) => match conn.set::<_, _, ()>(&key, &json).await {
                        Ok(()) => {
                            r.fallback.write().await.remove(&key);
                        }
                        Err(e) => {
                            r.drop_conn().await;
                            debug!("redis SET scratch failed, buffering in-memory: {e}");
                            r.fallback.write().await.insert(key, payload);
                        }
                    },
                    Err(_) => {
                        r.fallback.write().await.insert(key, payload);
                    }
                }
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
            ScratchBackend::Redis(r) => {
                match r.conn().await {
                    Ok(mut conn) => {
                        match conn.get::<_, Option<String>>(&key).await {
                            Ok(Some(s)) => Ok(Some(
                                serde_json::from_str(&s).context("parse scratch JSON")?,
                            )),
                            // Not in Redis — may be a write buffered during an outage.
                            Ok(None) => Ok(r.fallback.read().await.get(&key).cloned()),
                            Err(e) => {
                                r.drop_conn().await;
                                debug!("redis GET scratch failed, reading in-memory buffer: {e}");
                                Ok(r.fallback.read().await.get(&key).cloned())
                            }
                        }
                    }
                    Err(_) => Ok(r.fallback.read().await.get(&key).cloned()),
                }
            }
            ScratchBackend::Memory(map) => Ok(map.read().await.get(&key).cloned()),
        }
    }

    /// Format scratch for injection as a meta system block.
    pub async fn format_for_inject(&self, scope: &ScratchScope) -> Result<Option<String>> {
        let Some(payload) = self.read(scope).await? else {
            return Ok(None);
        };
        Ok(format_recall_block(
            &payload.snippets,
            self.scratch_max_tokens,
            self.chars_per_token,
        ))
    }

    pub async fn enqueue_distill(&self, job: DistillJob) -> Result<()> {
        let json = serde_json::to_string(&job)?;
        if let ScratchBackend::Redis(r) = &self.backend {
            match r.conn().await {
                Ok(mut conn) => match conn.lpush::<_, _, usize>(&self.distill_queue, &json).await {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        r.drop_conn().await;
                        warn!("redis LPUSH distill failed, using file queue: {e}");
                    }
                },
                Err(_) => {
                    debug!("redis unavailable for distill enqueue, using file queue");
                }
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
        if let ScratchBackend::Redis(r) = &self.backend {
            if let Ok(mut conn) = r.conn().await {
                let result: redis::RedisResult<Option<(String, String)>> = redis::cmd("BRPOP")
                    .arg(&self.distill_queue)
                    .arg(timeout_secs)
                    .query_async(&mut conn)
                    .await;
                match result {
                    // BRPOP already blocked up to timeout_secs.
                    Ok(Some((_, json))) => return Ok(Some(serde_json::from_str(&json)?)),
                    Ok(None) => return Ok(None),
                    Err(e) => {
                        r.drop_conn().await;
                        debug!("redis BRPOP distill failed, using file queue: {e}");
                    }
                }
            }
        }
        // File fallback: emulate BRPOP's blocking so callers polling on this
        // method don't spin in a tight loop while Redis is down/disabled.
        if let Some(job) = self.pop_distill_file().await? {
            return Ok(Some(job));
        }
        let nap = timeout_secs.clamp(0.0, 5.0);
        if nap > 0.0 {
            tokio::time::sleep(Duration::from_secs_f64(nap)).await;
        }
        Ok(None)
    }

    async fn pop_distill_file(&self) -> Result<Option<DistillJob>> {
        let mut entries = tokio::fs::read_dir(&self.distill_fallback_dir).await.ok();
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

#[cfg(test)]
mod tests {
    use super::*;

    fn snip(content: &str) -> RecallSnippet {
        RecallSnippet {
            content: content.to_string(),
            score: 1.0,
            fact_id: None,
            evidence_text: None,
        }
    }

    #[test]
    fn empty_scratch_is_none() {
        assert!(
            format_recall_block(&[], 2000, 3.5).is_none(),
            "empty scratch must not inject a [RECALL] block"
        );
    }

    #[test]
    fn overflow_omits_snippets_that_do_not_fit() {
        let snippets = vec![snip("alpha"), snip(&"overflow-payload-".repeat(80))];
        let out = format_recall_block(&snippets, 30, 3.5)
            .expect("first snippet must fit a 30-token budget");
        assert!(out.contains("alpha"));
        assert!(
            !out.contains("overflow-payload-"),
            "must stop at token bound"
        );
        assert!(
            estimate_text_tokens(&out, 3.5) <= 30 + 8,
            "joined block stays near the bound (header + per-line overhead)"
        );
    }

    #[test]
    fn overflow_too_tight_is_empty() {
        let snippets = vec![snip(&"huge".repeat(100))];
        assert!(
            format_recall_block(&snippets, 8, 3.5).is_none(),
            "nothing fits → no header-only inject"
        );
    }

    #[tokio::test]
    async fn write_empty_clears_scratch() {
        let svc = ScratchService::memory(2000);
        let scope = ScratchScope::Main {
            session_id: "empty-bounds".into(),
        };
        svc.write(&scope, vec![snip("keep")]).await.unwrap();
        assert!(svc.read(&scope).await.unwrap().is_some());
        svc.write(&scope, vec![]).await.unwrap();
        assert!(
            svc.read(&scope).await.unwrap().is_none(),
            "empty write is a clear"
        );
        assert!(svc.format_for_inject(&scope).await.unwrap().is_none());
    }
}

/// Format snippets as a `[RECALL]` inject block.
/// Empty scratch, or overflow that fits nothing, is `None`.
pub fn format_recall_block(
    snippets: &[RecallSnippet],
    max_tokens: usize,
    chars_per_token: f64,
) -> Option<String> {
    if snippets.is_empty() {
        return None;
    }

    let mut lines = vec!["[RECALL]".to_string()];
    let mut used = estimate_text_tokens("[RECALL]\n", chars_per_token);

    for snip in snippets {
        let line = match (&snip.fact_id, &snip.evidence_text) {
            (Some(id), Some(ev)) if !ev.trim().is_empty() => format!(
                "- [{:.2}] ({}) {}\n  source_span: {}",
                snip.score,
                id,
                snip.content,
                ev.trim()
            ),
            (Some(id), _) => format!("- [{:.2}] ({}) {}", snip.score, id, snip.content),
            (None, Some(ev)) if !ev.trim().is_empty() => format!(
                "- [{:.2}] {}\n  source_span: {}",
                snip.score,
                snip.content,
                ev.trim()
            ),
            _ => format!("- [{:.2}] {}", snip.score, snip.content),
        };
        let cost = estimate_text_tokens(&line, chars_per_token);
        if used + cost > max_tokens {
            break;
        }
        used += cost;
        lines.push(line);
    }

    if lines.len() <= 1 {
        return None;
    }
    Some(lines.join("\n"))
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
