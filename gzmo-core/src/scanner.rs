//! # Environment Scanner
//!
//! Probes the local machine for running LLM inference endpoints.
//! Used by `gzmo init` to auto-detect available backends.
//!
//! Zero external calls — only scans localhost on known ports.

use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;
use tracing::debug;

/// A discovered LLM endpoint.
#[derive(Debug, Clone)]
pub struct DiscoveredEndpoint {
    /// Human-readable name (e.g. "LM Studio", "Ollama", "vLLM")
    pub name: String,
    /// Full base URL (e.g. "http://localhost:1234/v1")
    pub url: String,
    /// Latency of the probe in milliseconds
    pub latency_ms: u64,
    /// Models available at this endpoint
    pub models: Vec<String>,
}

/// Known local LLM endpoint patterns to probe.
const KNOWN_ENDPOINTS: &[(&str, &str, &str)] = &[
    ("LM Studio",   "http://localhost:1234/v1",    "/models"),
    ("Ollama",      "http://localhost:11434/v1",    "/models"),
    ("vLLM",        "http://localhost:8000/v1",     "/models"),
    ("LocalAI",     "http://localhost:8080/v1",     "/models"),
    ("text-gen-ui", "http://localhost:5000/v1",     "/models"),
    ("Jan",         "http://localhost:1337/v1",     "/models"),
    ("LiteLLM",    "http://localhost:4000/v1",     "/models"),
];

/// OpenAI-compatible models list response.
#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
struct ModelEntry {
    id: String,
}

/// Scan all known local endpoints for running LLM servers.
///
/// Returns a list of discovered endpoints with their available models.
/// Non-responding endpoints are silently skipped.
pub async fn scan_endpoints() -> Vec<DiscoveredEndpoint> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(800))
        .build()
        .unwrap_or_default();

    let mut results = Vec::new();

    for (name, base_url, models_path) in KNOWN_ENDPOINTS {
        let url = format!("{}{}", base_url, models_path);
        debug!(endpoint = %name, url = %url, "Probing");

        let start = std::time::Instant::now();
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let latency = start.elapsed().as_millis() as u64;
                let models = match resp.json::<ModelsResponse>().await {
                    Ok(m) => m.data.into_iter().map(|e| e.id).collect(),
                    Err(_) => vec![],
                };

                results.push(DiscoveredEndpoint {
                    name: name.to_string(),
                    url: base_url.to_string(),
                    latency_ms: latency,
                    models,
                });
            }
            _ => {
                debug!(endpoint = %name, "Not responding");
            }
        }
    }

    results
}

/// Probe a single custom endpoint URL.
pub async fn probe_endpoint(base_url: &str) -> Result<DiscoveredEndpoint> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(2000))
        .build()?;

    let url = format!("{}/models", base_url);
    let start = std::time::Instant::now();
    let resp = client.get(&url).send().await?;
    let latency = start.elapsed().as_millis() as u64;

    if !resp.status().is_success() {
        anyhow::bail!("Endpoint returned {}", resp.status());
    }

    let models = match resp.json::<ModelsResponse>().await {
        Ok(m) => m.data.into_iter().map(|e| e.id).collect(),
        Err(_) => vec![],
    };

    Ok(DiscoveredEndpoint {
        name: "Custom".to_string(),
        url: base_url.to_string(),
        latency_ms: latency,
        models,
    })
}
