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
/// Prime / llama.cpp on `:8000` is listed first — product `gzmo init` prefers it when up.
const KNOWN_ENDPOINTS: &[(&str, &str, &str)] = &[
    ("Prime / llama.cpp", "http://127.0.0.1:8000/v1", "/models"),
    ("Prime / llama.cpp", "http://localhost:8000/v1", "/models"),
    ("LM Studio", "http://127.0.0.1:1234/v1", "/models"),
    ("LM Studio", "http://localhost:1234/v1", "/models"),
    ("Ollama", "http://localhost:11434/v1", "/models"),
    ("vLLM", "http://localhost:8000/v1", "/models"),
    ("LocalAI", "http://localhost:8080/v1", "/models"),
    ("text-gen-ui", "http://localhost:5000/v1", "/models"),
    ("Jan", "http://localhost:1337/v1", "/models"),
    ("LiteLLM", "http://localhost:4000/v1", "/models"),
];

/// Prefer product-friendly engines: Prime `:8000`, then lowest-latency remaining.
/// Dedupes by host:port so `127.0.0.1` and `localhost` do not both win.
pub fn prefer_product_engine(endpoints: &[DiscoveredEndpoint]) -> Option<&DiscoveredEndpoint> {
    if endpoints.is_empty() {
        return None;
    }
    let prime = endpoints
        .iter()
        .find(|e| e.url.contains(":8000/") || e.url.ends_with(":8000") || e.url.contains(":8000?"));
    if let Some(ep) = prime {
        return Some(ep);
    }
    endpoints.iter().min_by_key(|e| e.latency_ms)
}

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

    // Dedupe identical URLs (e.g. vLLM alias after Prime on :8000).
    let mut deduped: Vec<DiscoveredEndpoint> = Vec::new();
    for ep in results {
        if deduped.iter().any(|d| d.url == ep.url) {
            continue;
        }
        deduped.push(ep);
    }
    deduped
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ep(name: &str, url: &str, latency_ms: u64) -> DiscoveredEndpoint {
        DiscoveredEndpoint {
            name: name.to_string(),
            url: url.to_string(),
            latency_ms,
            models: vec!["m".into()],
        }
    }

    #[test]
    fn prefer_product_engine_picks_prime_8000_over_faster_lmstudio() {
        let eps = vec![
            ep("LM Studio", "http://127.0.0.1:1234/v1", 5),
            ep("Prime / llama.cpp", "http://127.0.0.1:8000/v1", 40),
        ];
        let picked = prefer_product_engine(&eps).expect("pick");
        assert!(picked.url.contains(":8000"));
    }

    #[test]
    fn prefer_product_engine_falls_back_to_lowest_latency() {
        let eps = vec![
            ep("Ollama", "http://localhost:11434/v1", 30),
            ep("LM Studio", "http://127.0.0.1:1234/v1", 8),
        ];
        let picked = prefer_product_engine(&eps).expect("pick");
        assert!(picked.url.contains(":1234"));
    }

    #[test]
    fn prefer_product_engine_empty() {
        assert!(prefer_product_engine(&[]).is_none());
    }
}
