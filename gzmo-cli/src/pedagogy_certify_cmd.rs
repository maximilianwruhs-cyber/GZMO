//! `gzmo pedagogy certify` — Layer 3 operator certification on Synapse bus.

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use gzmo_core::config::GzmoConfig;
use gzmo_core::synapse::{EventSource, EventType, SynapseBus, SynapseEvent};
use serde_json::json;
use uuid::Uuid;

fn synapse_path(config: &GzmoConfig) -> PathBuf {
    config
        .memory
        .vault_db
        .parent()
        .unwrap_or(std::path::Path::new("data"))
        .join("Synapse/events.jsonl")
}

pub async fn run(config: &GzmoConfig, args: &[String]) -> Result<()> {
    let mut osc_id: Option<String> = None;
    let mut learning_verified = true;
    let mut certified_by = "operator".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--oscillation-id" => {
                i += 1;
                osc_id = Some(args.get(i).context("--oscillation-id requires value")?.clone());
            }
            "--learning-verified" => {
                i += 1;
                let v = args.get(i).context("--learning-verified requires value")?;
                learning_verified = matches!(v.as_str(), "true" | "1" | "yes");
            }
            "--certified-by" => {
                i += 1;
                certified_by = args.get(i).context("--certified-by requires value")?.clone();
            }
            other => bail!("unknown arg: {other}"),
        }
        i += 1;
    }

    let osc_id = osc_id.context("--oscillation-id is required")?;
    let uuid = Uuid::parse_str(&osc_id).context("invalid oscillation_id UUID")?;

    let bus = SynapseBus::with_path(synapse_path(config));
    gzmo_core::synapse::set_event_source(EventSource::GzmoCli);

    bus.append(&SynapseEvent::with_envelope(
        EventType::PedagogyLearningCertified,
        EventSource::GzmoCli,
        Some(uuid),
        None,
        Some(json!({
            "oscillation_id": osc_id,
            "learning_verified": learning_verified,
            "certified_by": certified_by,
            "layer": 3,
        })),
    ));

    println!("pedagogy.learning_certified written for oscillation_id={osc_id}");
    Ok(())
}
