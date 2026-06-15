//! Live cloud-first routing probes (ignored by default; require network + the
//! production gzmo.toml/.env and a reachable Prime :8000).
//!
//! Run:
//!   cargo test -p gzmo-core --test live_cloud_probe -- --ignored --nocapture
//!
//! These exercise the real background routing path:
//!   GzmoConfig::load -> GatewayRouter::new -> gateway(TaskKind::SparkHypothesis)
//!   -> FallbackGateway(cloud -> legacy)
//! without touching the vault or DREAMS.md (no SparkEngine, no memory writes).

use std::path::PathBuf;

use gzmo_core::config::{GzmoConfig, TaskKind};
use gzmo_core::gateway::{GatewayRouter, LlmResponse};
use gzmo_core::types::{Message, Role};

/// Path to the production config at the repo root (parent of this crate).
fn prod_config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("gzmo.toml")
}

fn ping_messages() -> Vec<Message> {
    vec![
        Message {
            role: Role::System,
            content: "You are a routing probe. Answer in one word.".to_string(),
            is_meta: false,
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::User,
            content: "Reply with exactly: PONG".to_string(),
            is_meta: false,
            tool_calls: None,
            tool_call_id: None,
        },
    ]
}

fn text_of(r: LlmResponse) -> String {
    match r {
        LlmResponse::Text(t) => t,
        LlmResponse::ToolCalls(c) => format!("<tool_calls: {} call(s)>", c.len()),
    }
}

/// Proves: (1) config is cloud-first, (2) the OpenRouter cloud leaf answers live,
/// (3) the wired background task (SparkHypothesis) returns through the chain.
#[tokio::test]
#[ignore = "live: requires network, OpenRouter key in .env, Prime :8000"]
async fn live_background_uses_cloud_first() {
    let cfg = GzmoConfig::load(&prod_config_path()).expect("load gzmo.toml + .env");

    assert!(
        cfg.routing.cloud_first_background,
        "cloud_first_background must be true for cloud-first background routing"
    );
    let cloud = cfg.engine.cloud.as_ref().expect("[engine.cloud] configured");
    assert!(
        !cloud.api_key.is_empty(),
        "cloud api_key must be injected from GZMO_OPENROUTER_KEY (.env)"
    );
    eprintln!(
        "[probe] cloud model={} url={} key=sk-...{}",
        cloud.model,
        cloud.url,
        &cloud.api_key[cloud.api_key.len().saturating_sub(4)..]
    );

    let router = GatewayRouter::new(&cfg);

    // (2) Direct cloud leaf — unambiguous live OpenRouter hit (no fallback here
    // because no Gemini key is configured).
    let cloud_gw = router
        .gateway_by_name("cloud")
        .expect("cloud leaf must exist when cloud-first is enabled");
    let cloud_resp = cloud_gw
        .complete(&ping_messages(), &[])
        .await
        .expect("OpenRouter cloud completion must succeed live");
    eprintln!("[probe] DIRECT cloud reply: {}", text_of(cloud_resp));

    // (3) The actual background routing path used by the daemon/spark.
    let spark_gw = router.gateway(TaskKind::SparkHypothesis);
    let spark_resp = spark_gw
        .complete(&ping_messages(), &[])
        .await
        .expect("SparkHypothesis (cloud-first chain) must succeed live");
    eprintln!(
        "[probe] BACKGROUND SparkHypothesis reply: {}",
        text_of(spark_resp)
    );
}

/// Proves automatic failover: with a broken OpenRouter key, the background task
/// fails over to the legacy local profile (Prime :8000) and still succeeds.
#[tokio::test]
#[ignore = "live: requires Prime :8000 reachable"]
async fn live_background_falls_back_to_prime_on_bad_cloud_key() {
    let mut cfg = GzmoConfig::load(&prod_config_path()).expect("load gzmo.toml + .env");
    assert!(cfg.routing.cloud_first_background);

    // Break the cloud key so OpenRouter returns 401 -> FallbackGateway must skip
    // to the legacy profile (Prime). Gemini inner-fallback stays disabled (no key).
    if let Some(cloud) = cfg.engine.cloud.as_mut() {
        cloud.api_key = "sk-or-v1-deliberately-invalid-probe-key".to_string();
        cloud.fallback_api_key = None;
    }

    let router = GatewayRouter::new(&cfg);
    let spark_gw = router.gateway(TaskKind::SparkHypothesis);
    let resp = spark_gw
        .complete(&ping_messages(), &[])
        .await
        .expect("must fail over to Prime and succeed despite broken cloud key");
    eprintln!("[probe] FALLBACK reply (served by Prime): {}", text_of(resp));
}
