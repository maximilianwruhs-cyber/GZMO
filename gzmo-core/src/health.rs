//! Startup and CLI health probes for LLM, embeddings, Neo4j, and optional Sovereign.

use std::net::ToSocketAddrs;
use std::time::Duration;

use anyhow::{bail, Result};
use reqwest::Client;
use tracing::{info, warn};

use crate::config::{
    EmbeddingsConfig, EngineProfileConfig, GzmoConfig, LibrarianConfig, QdrantConfig, RedisConfig,
    RerankConfig,
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
pub async fn probe_sovereign(profile: &EngineProfileConfig, active_mode: crate::config::EngineMode) -> ProbeResult {
    let mut r = probe_llm_models(profile).await;
    r.name = "sovereign";
    if !r.ok && active_mode != crate::config::EngineMode::Sovereign {
        r.detail = "PARKED (sovereign is deprioritized and port is down)".to_string();
    }
    r
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
        Ok(r) => ProbeResult::fail(
            "qdrant",
            format!("{} returned HTTP {}", url, r.status()),
        ),
        Err(e) => ProbeResult::fail("qdrant", format!("{url} unreachable: {e}")),
    }
}

/// VM200 retrieval router rerank preset when `[rerank].enabled`.
pub async fn probe_rerank(cfg: &RerankConfig) -> ProbeResult {
    if !cfg.enabled {
        return ProbeResult::pass("rerank", "disabled in config");
    }
    const MIN_RERANK_SCORE: f64 = 1e-6;
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
            Ok(hits) if hits.is_empty() => ProbeResult::fail("rerank", "empty rerank results"),
            Ok(hits) if hits[0].1.abs() < MIN_RERANK_SCORE => ProbeResult::fail(
                "rerank",
                format!(
                    "near-zero top score {:.3e} — broken GGUF or wrong model preset",
                    hits[0].1
                ),
            ),
            Ok(hits) => ProbeResult::pass(
                "rerank",
                format!("{} @ {} (top {:.3})", cfg.model, cfg.url, hits[0].1),
            ),
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
        arguments: serde_json::json!({}),
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

/// Run all configured probes; logs warnings for failures (non-fatal unless `strict`).
/// Optionally appends events to the Synapse bus for observability.
pub async fn run_startup_probes(
    config: &GzmoConfig,
    tools: Option<&ToolRegistry>,
    strict: bool,
    synapse: Option<&SynapseBus>,
) -> Result<Vec<ProbeResult>> {
    // HealthTick: probes started
    if let Some(bus) = synapse {
        bus.append(&SynapseEvent::new(
            EventType::HealthTick,
            resolve_event_source(EventSource::GzmoDaemon),
        ));
    }

    let mut results = Vec::new();

    let prime = config.engine.active_engine_for_mode(crate::config::EngineMode::Local);
    results.push(probe_llm_models(&prime).await);

    results.push(probe_embeddings(&config.embeddings, &config.redis).await);
    results.push(probe_qdrant(&config.qdrant).await);
    results.push(probe_rerank(&config.rerank).await);
    results.push(probe_librarian(&config.librarian).await);
    results.push(probe_redis(&config.redis).await);

    if let Some(srv) = config.active_mcp_servers().find(|s| s.name == "memory") {
        if let Some(url) = srv.env.get("NEO4J_URL") {
            results.push(probe_neo4j_bolt(url));
        }
        if let Some(tools) = tools {
            results.push(probe_mcp_memory(tools).await);
        }
    }

    if let Some(ref sovereign) = config.engine.sovereign {
        let mut r = probe_sovereign(sovereign, config.engine.active_mode).await;
        if !r.ok && config.engine.active_mode == crate::config::EngineMode::Sovereign {
            r.detail = format!(
                "{} (expected until sovereign-moe GGUF is built)",
                r.detail
            );
        }
        results.push(r);
    }

    for r in &results {
        if r.ok {
            info!(probe = r.name, "{}", r.detail);
        } else if strict && r.name != "sovereign" {
            warn!(probe = r.name, "STRICT health failure: {}", r.detail);
        } else {
            warn!(probe = r.name, "{}", r.detail);
        }
    }

    // Check for failures and emit HealthFail if strict mode
    let failures: Vec<_> = results
        .iter()
        .filter(|r| !r.ok && r.name != "sovereign")
        .collect();

    if strict && !failures.is_empty() {
        // HealthFail: strict mode, failures detected
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

    // Non-strict failures: emit HealthFail for observability
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

    if let Some(bus) = synapse {
        append_routing_cognition_tick(config, bus);
    }

    Ok(results)
}

/// Emit HealthTick with routing/librarian config for daemon cognition alignment (F4).
pub fn append_routing_cognition_tick(config: &GzmoConfig, bus: &SynapseBus) {
    let routing_blindness =
        !config.librarian.enabled && !config.session_distill.use_librarian;
    bus.append(&SynapseEvent::with_data(
        EventType::HealthTick,
        resolve_event_source(EventSource::GzmoDaemon),
        serde_json::json!({
            "librarian_enabled": config.librarian.enabled,
            "use_librarian": config.session_distill.use_librarian,
            "librarian_summary": config.session_distill.librarian_summary,
            "routing_blindness": routing_blindness,
        }),
    ));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthPerspective {
    Host,
    Container,
}

impl std::fmt::Display for HealthPerspective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Host => write!(f, "host"),
            Self::Container => write!(f, "container"),
        }
    }
}

pub fn detect_perspective() -> HealthPerspective {
    if let Ok(val) = std::env::var("GZMO_HEALTH_PERSPECTIVE") {
        match val.to_lowercase().as_str() {
            "container" => return HealthPerspective::Container,
            "host" => return HealthPerspective::Host,
            _ => {}
        }
    }
    if std::path::Path::new("/.dockerenv").exists() {
        return HealthPerspective::Container;
    }
    if std::env::var("CONTAINER").is_ok() || std::path::Path::new("/run/.containerenv").exists() {
        return HealthPerspective::Container;
    }
    HealthPerspective::Host
}

/// Format probe list for `gzmo health` CLI.
pub fn format_report(results: &[ProbeResult]) -> String {
    let perspective = detect_perspective();
    let hostname = sysinfo::System::host_name().unwrap_or_else(|| "unknown".to_string());
    let mut out = format!("GZMO health report [perspective={perspective} hostname={hostname}]\n");
    for r in results {
        let mark = if r.ok {
            "OK"
        } else if r.name == "sovereign" && r.detail.contains("PARKED") {
            "PARKED"
        } else {
            "FAIL"
        };
        out.push_str(&format!("  [{mark}] {} — {}\n", r.name, r.detail));
    }
    out
}
