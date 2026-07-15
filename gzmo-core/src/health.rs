//! Startup and CLI health probes for LLM, embeddings, Neo4j, and optional Sovereign.

use std::net::ToSocketAddrs;
use std::time::Duration;

use anyhow::{bail, Result};
use reqwest::Client;
use tracing::{info, warn};

use std::path::Path;

use crate::config::{
    EmbeddingsConfig, EngineMode, EngineProfileConfig, GzmoConfig, LibrarianConfig, QdrantConfig,
    RedisConfig, RerankConfig,
};
use crate::memory::rerank::Reranker;
use crate::synapse::{resolve_event_source, EventSource, EventType, SynapseBus, SynapseEvent};
use crate::tools::ToolRegistry;
use crate::gateway::ToolCall;

/// Outcome of a subsystem probe.
#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub name: &'static str,
    pub ok: bool,
    pub detail: String,
}

impl ProbeResult {
    fn pass(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            ok: true,
            detail: detail.into(),
        }
    }

    fn fail(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            ok: false,
            detail: detail.into(),
        }
    }
}

/// GET `{url}/models` — OpenAI-compatible liveness.
pub async fn probe_llm_models(profile: &EngineProfileConfig) -> ProbeResult {
    let base = profile.url.trim_end_matches('/');
    let url = format!("{base}/models");
    let client = match Client::builder().timeout(Duration::from_secs(8)).build() {
        Ok(c) => c,
        Err(e) => return ProbeResult::fail("llm", format!("HTTP client: {e}")),
    };
    let mut req = client.get(&url);
    if !profile.api_key.is_empty() {
        req = req.bearer_auth(&profile.api_key);
    }
    match req.send().await {
        Ok(r) if r.status().is_success() => {
            ProbeResult::pass("llm", format!("{} → {}", profile.model, url))
        }
        Ok(r) => ProbeResult::fail(
            "llm",
            format!("{} returned HTTP {}", url, r.status()),
        ),
        Err(e) => ProbeResult::fail("llm", format!("{} unreachable: {e}", url)),
    }
}

/// POST a tiny embedding when `[embeddings].enabled`.
pub async fn probe_embeddings(cfg: &EmbeddingsConfig, redis_cfg: &RedisConfig) -> ProbeResult {
    if !cfg.enabled {
        return ProbeResult::pass("embeddings", "disabled in config");
    }
    match crate::memory::embeddings::Embedder::from_config(cfg, redis_cfg) {
        Ok(e) => match e.embed("health probe").await {
            Ok(v) if !v.is_empty() => {
                ProbeResult::pass("embeddings", format!("{} dims @ {}", v.len(), cfg.url))
            }
            Ok(_) => ProbeResult::fail("embeddings", "empty vector returned"),
            Err(e) => ProbeResult::fail("embeddings", e.to_string()),
        },
        Err(e) => ProbeResult::fail("embeddings", e.to_string()),
    }
}

/// TCP reachability for `bolt://host:port` (Neo4j sidecar).
pub fn probe_neo4j_bolt(bolt_url: &str) -> ProbeResult {
    let host_port = bolt_url
        .trim()
        .strip_prefix("bolt://")
        .or_else(|| bolt_url.strip_prefix("bolt+s://"))
        .unwrap_or(bolt_url);
    let addr = match host_port.to_socket_addrs() {
        Ok(mut a) => a.next(),
        Err(e) => return ProbeResult::fail("neo4j", format!("resolve {bolt_url}: {e}")),
    };
    let Some(addr) = addr else {
        return ProbeResult::fail("neo4j", format!("no address for {bolt_url}"));
    };
    match std::net::TcpStream::connect_timeout(&addr, Duration::from_secs(3)) {
        Ok(_) => ProbeResult::pass("neo4j", format!("bolt reachable {addr}")),
        Err(e) => ProbeResult::fail("neo4j", format!("{addr}: {e}")),
    }
}

/// PING Redis when `[redis].enabled` — surfaces the scratch backend status
/// (unreachable, auth required, etc.) instead of it silently degrading to the
/// in-memory buffer. Authentication errors come back in the PING reply.
pub async fn probe_redis(cfg: &RedisConfig) -> ProbeResult {
    if !cfg.enabled {
        return ProbeResult::pass("redis", "disabled in config");
    }
    let client = match redis::Client::open(cfg.url.as_str()) {
        Ok(c) => c,
        Err(e) => return ProbeResult::fail("redis", format!("bad url {}: {e}", cfg.url)),
    };
    let connect = client.get_connection_manager();
    let mut conn = match tokio::time::timeout(Duration::from_secs(3), connect).await {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => return ProbeResult::fail("redis", format!("{} unreachable: {e}", cfg.url)),
        Err(_) => return ProbeResult::fail("redis", format!("{} connect timed out", cfg.url)),
    };
    let pong: redis::RedisResult<String> = redis::cmd("PING").query_async(&mut conn).await;
    match pong {
        Ok(_) => ProbeResult::pass("redis", format!("PONG @ {}", cfg.url)),
        Err(e) => ProbeResult::fail("redis", format!("{} PING failed: {e}", cfg.url)),
    }
}

/// Optional Sovereign FrankenMoE on :8010 (GGUF may not exist yet).
pub async fn probe_sovereign(profile: &EngineProfileConfig) -> ProbeResult {
    probe_llm_models(profile).await
}

/// GET Qdrant collection info when `[qdrant].enabled`.
pub async fn probe_qdrant(cfg: &QdrantConfig) -> ProbeResult {
    if !cfg.enabled {
        return ProbeResult::pass("qdrant", "disabled in config");
    }
    let base = cfg.url.trim_end_matches('/');
    let url = format!("{base}/collections/{}", cfg.collection);
    let client = match Client::builder().timeout(Duration::from_secs(8)).build() {
        Ok(c) => c,
        Err(e) => return ProbeResult::fail("qdrant", format!("HTTP client: {e}")),
    };
    match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => {
            let detail = match r.json::<serde_json::Value>().await {
                Ok(v) => {
                    let pts = v["result"]["points_count"].as_u64().unwrap_or(0);
                    let status = v["result"]["status"].as_str().unwrap_or("?");
                    format!(
                        "{} → {} points ({status})",
                        cfg.collection, pts
                    )
                }
                Err(_) => format!("{} OK", cfg.collection),
            };
            ProbeResult::pass("qdrant", detail)
        }
        Ok(r) if r.status().as_u16() == 404 => ProbeResult::fail(
            "qdrant",
            format!(
                "collection '{}' missing at {} — run: bash scripts/qdrant-vault-sync.sh",
                cfg.collection, url
            ),
        ),
        Ok(r) => ProbeResult::fail(
            "qdrant",
            format!("{} returned HTTP {}", url, r.status()),
        ),
        Err(e) => ProbeResult::fail("qdrant", format!("{url} unreachable: {e}")),
    }
}

/// VM200 reranker (:8082) when `[rerank].enabled`.
pub async fn probe_rerank(cfg: &RerankConfig) -> ProbeResult {
    if !cfg.enabled {
        return ProbeResult::pass("rerank", "disabled in config");
    }
    match Reranker::from_config(cfg) {
        Ok(r) => match r
            .rerank(
                "health probe",
                &[
                    "gzmo production stack".to_string(),
                    "unrelated weather forecast".to_string(),
                ],
                Some(1),
            )
            .await
        {
            Ok(hits) if !hits.is_empty() => ProbeResult::pass(
                "rerank",
                format!("{} @ {} (top {:.3})", cfg.model, cfg.url, hits[0].1),
            ),
            Ok(_) => ProbeResult::fail("rerank", "empty rerank results"),
            Err(e) => ProbeResult::fail("rerank", e.to_string()),
        },
        Err(e) => ProbeResult::fail("rerank", e.to_string()),
    }
}

/// VM200 librarian (:8083) when `[librarian].enabled`.
pub async fn probe_librarian(cfg: &LibrarianConfig) -> ProbeResult {
    if !cfg.enabled {
        return ProbeResult::pass("librarian", "disabled in config");
    }
    let profile = EngineProfileConfig {
        provider: "local".into(),
        url: cfg.url.clone(),
        model: cfg.model.clone(),
        api_key: cfg.api_key.clone(),
        temperature: 0.0,
        top_p: 1.0,
        max_tokens: 256,
        reasoning_effort: None,
    };
    let mut r = probe_llm_models(&profile).await;
    r.name = "librarian";
    r
}

/// MCP `read_graph` smoke test when memory server is registered.
pub async fn probe_mcp_memory(tools: &ToolRegistry) -> ProbeResult {
    if !tools.has_tool("mcp__memory__read_graph") {
        return ProbeResult::fail("mcp_memory", "mcp__memory__read_graph not registered");
    }
    let call = ToolCall {
        id: "health_read_graph".into(),
        function_name: "mcp__memory__read_graph".to_string(),
        arguments: serde_json::json!({ "sample_limit": 1 }),
    };
    match tools.dispatch(&call).await {
        crate::tools::ToolResult { success: true, output, .. } => {
            let preview: String = output.chars().take(120).collect();
            ProbeResult::pass("mcp_memory", format!("read_graph OK ({preview}…)"))
        }
        crate::tools::ToolResult { output, .. } => {
            ProbeResult::fail("mcp_memory", output)
        }
    }
}

/// Probe cloud LLM when `active_mode=cloud`.
pub async fn probe_cloud_llm(config: &GzmoConfig) -> ProbeResult {
    if config.engine.active_mode != EngineMode::Cloud {
        return ProbeResult::pass("cloud_llm", "active_mode != cloud (skipped)");
    }
    let cloud = config.engine.active_engine_for_mode(EngineMode::Cloud);
    let mut r = probe_llm_models(&cloud).await;
    r.name = "cloud_llm";
    r
}

/// Prime / local fallback engine reachability.
pub async fn probe_prime_llm(config: &GzmoConfig) -> ProbeResult {
    let local = config.engine.active_engine_for_mode(EngineMode::Local);
    let mut r = probe_llm_models(&local).await;
    r.name = "prime_llm";
    if config.engine.active_mode == EngineMode::Cloud && !r.ok {
        let cloud = config.engine.active_engine_for_mode(EngineMode::Cloud);
        let cloud_r = probe_llm_models(&cloud).await;
        if cloud_r.ok {
            return ProbeResult::pass(
                "prime_llm",
                format!("offline (cloud-primary OK) — {}", r.detail),
            );
        }
    }
    r
}

/// Fail when cloud-primary and Prime fallback are both unreachable.
pub async fn probe_cognition_blackout(config: &GzmoConfig) -> ProbeResult {
    if config.engine.active_mode != EngineMode::Cloud {
        return ProbeResult::pass("cognition_blackout", "not cloud-primary (skipped)");
    }
    let cloud = probe_cloud_llm(config).await;
    let prime = probe_prime_llm(config).await;
    if !cloud.ok && !prime.ok {
        ProbeResult::fail(
            "cognition_blackout",
            format!(
                "cloud AND prime down — cloud: {}; prime: {}",
                cloud.detail, prime.detail
            ),
        )
    } else if !cloud.ok {
        ProbeResult::pass(
            "cognition_blackout",
            format!("cloud down, prime fallback OK — {}", prime.detail),
        )
    } else {
        ProbeResult::pass("cognition_blackout", cloud.detail)
    }
}

/// Redis distill queue depth + oldest fallback file age.
pub async fn probe_distill_queue(redis_cfg: &RedisConfig, fallback_dir: &Path) -> ProbeResult {
    let mut redis_depth: u64 = 0;
    let mut oldest_hint = String::new();

    if redis_cfg.enabled {
        let client = match redis::Client::open(redis_cfg.url.as_str()) {
            Ok(c) => c,
            Err(e) => return ProbeResult::fail("distill_queue", format!("bad redis url: {e}")),
        };
        let connect = client.get_connection_manager();
        let mut conn = match tokio::time::timeout(Duration::from_secs(3), connect).await {
            Ok(Ok(c)) => c,
            Ok(Err(e)) => {
                return ProbeResult::fail("distill_queue", format!("redis unreachable: {e}"))
            }
            Err(_) => return ProbeResult::fail("distill_queue", "redis connect timed out"),
        };
        let len: redis::RedisResult<u64> = redis::cmd("LLEN")
            .arg(&redis_cfg.distill_queue)
            .query_async(&mut conn)
            .await;
        match len {
            Ok(n) => redis_depth = n,
            Err(e) => return ProbeResult::fail("distill_queue", format!("LLEN failed: {e}")),
        };
        if redis_depth > 0 {
            let tail: redis::RedisResult<Option<String>> = redis::cmd("LINDEX")
                .arg(&redis_cfg.distill_queue)
                .arg(-1i64)
                .query_async(&mut conn)
                .await;
            if let Ok(Some(json)) = tail {
                oldest_hint = format!(
                    "redis_tail_session={}",
                    serde_json::from_str::<serde_json::Value>(&json)
                        .ok()
                        .and_then(|v| v.get("session_id").and_then(|s| s.as_str()).map(str::to_string))
                        .unwrap_or_else(|| "?".into())
                );
            }
        }
    }

    let mut fallback_count = 0u64;
    let mut oldest_age_secs: Option<u64> = None;
    if fallback_dir.is_dir() {
        if let Ok(rd) = std::fs::read_dir(fallback_dir) {
            let now = std::time::SystemTime::now();
            for entry in rd.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                fallback_count += 1;
                if let Ok(meta) = entry.metadata() {
                    if let Ok(modified) = meta.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            oldest_age_secs = Some(oldest_age_secs.map_or(age.as_secs(), |a| a.max(age.as_secs())));
                        }
                    }
                }
            }
        }
    }

    let detail = format!(
        "redis_depth={} queue={} {}{}",
        redis_depth,
        redis_cfg.distill_queue,
        oldest_hint,
        format!(
            " fallback_files={}{}",
            fallback_count,
            oldest_age_secs
                .map(|s| format!(" oldest_fallback_age_sec={s}"))
                .unwrap_or_default()
        )
    );

    if redis_depth > 50 || fallback_count > 10 {
        ProbeResult::fail("distill_queue", format!("backlog high — {detail}"))
    } else {
        ProbeResult::pass("distill_queue", detail)
    }
}

/// Collect all health probes for CLI and daemon startup.
pub async fn collect_health_probes(
    config: &GzmoConfig,
    tools: Option<&ToolRegistry>,
) -> Vec<ProbeResult> {
    let mut results = Vec::new();

    results.push(probe_cloud_llm(config).await);
    results.push(probe_prime_llm(config).await);
    results.push(probe_cognition_blackout(config).await);

    let active = config.engine.active_engine();
    let mut active_r = probe_llm_models(&active).await;
    active_r.name = "active_llm";
    results.push(active_r);

    results.push(probe_embeddings(&config.embeddings, &config.redis).await);
    results.push(probe_qdrant(&config.qdrant).await);
    results.push(probe_rerank(&config.rerank).await);
    results.push(probe_librarian(&config.librarian).await);
    results.push(probe_redis(&config.redis).await);
    results.push(
        probe_distill_queue(&config.redis, &config.redis.distill_fallback_dir)
            .await,
    );

    if let Some(srv) = config.active_mcp_servers().find(|s| s.name == "memory") {
        if let Some(url) = srv.env.get("NEO4J_URL") {
            results.push(probe_neo4j_bolt(url));
        }
        if let Some(tools) = tools {
            results.push(probe_mcp_memory(tools).await);
        }
    }

    if let Some(ref sovereign) = config.engine.sovereign {
        let mut r = probe_sovereign(sovereign).await;
        r.name = "sovereign";
        if !r.ok {
            r.detail = format!(
                "{} (expected until sovereign-moe GGUF is built)",
                r.detail
            );
        }
        results.push(r);
    }

    results
}

/// Run all configured probes; logs warnings for failures (non-fatal unless `strict`).
/// Optionally appends events to the Synapse bus for observability.
pub async fn run_startup_probes(
    config: &GzmoConfig,
    tools: Option<&ToolRegistry>,
    strict: bool,
    synapse: Option<&SynapseBus>,
) -> Result<Vec<ProbeResult>> {
    if let Some(bus) = synapse {
        bus.append(&SynapseEvent::new(
            EventType::HealthTick,
            resolve_event_source(EventSource::GzmoDaemon),
        ));
    }

    let results = collect_health_probes(config, tools).await;

    for r in &results {
        if r.ok {
            info!(probe = r.name, "{}", r.detail);
        } else if strict && r.name != "sovereign" {
            warn!(probe = r.name, "STRICT health failure: {}", r.detail);
        } else {
            warn!(probe = r.name, "{}", r.detail);
        }
    }

    let failures: Vec<_> = results
        .iter()
        .filter(|r| !r.ok && r.name != "sovereign")
        .collect();

    if strict && !failures.is_empty() {
        if let Some(bus) = synapse {
            let details: Vec<_> = failures
                .iter()
                .map(|r| format!("{}={}", r.name, r.detail))
                .collect();
            let data = serde_json::json!({
                "strict": true,
                "failures": details,
                "count": failures.len(),
            });
            bus.append(&SynapseEvent::with_data(
                EventType::HealthFail,
                resolve_event_source(EventSource::GzmoDaemon),
                data,
            ));
        }
        bail!(
            "Startup health check failed: {}",
            failures
                .iter()
                .map(|r| format!("{}={}", r.name, r.detail))
                .collect::<Vec<_>>()
                .join("; ")
        );
    }

    if !failures.is_empty() && synapse.is_some() {
        if let Some(bus) = synapse {
            let details: Vec<_> = failures
                .iter()
                .map(|r| format!("{}={}", r.name, r.detail))
                .collect();
            let data = serde_json::json!({
                "strict": false,
                "failures": details,
                "count": failures.len(),
            });
            bus.append(&SynapseEvent::with_data(
                EventType::HealthFail,
                resolve_event_source(EventSource::GzmoDaemon),
                data,
            ));
        }
    }

    Ok(results)
}

/// Format probe list for `gzmo health` CLI.
pub fn format_report(results: &[ProbeResult]) -> String {
    let mut out = String::from("GZMO health report\n");
    for r in results {
        let mark = if r.ok { "OK" } else { "FAIL" };
        out.push_str(&format!("  [{mark}] {} — {}\n", r.name, r.detail));
    }
    out
}
