//! `gzmo chaos skill <command> [args]` — one-shot ritual/lab skill runner.
//!
//! This command never starts a PulseLoop. It reads the latest lab snapshot and
//! appends emitted feedback for a chat/TUI ritual bridge to drain later.

use std::sync::Arc;

use anyhow::{bail, Result};
use gzmo_chaos::feedback_ipc;
use gzmo_chaos::pulse::ChaosSnapshot;
use tokio::sync::mpsc;

use gzmo_core::config::GzmoConfig;
use gzmo_core::gateway::{LlmGateway, TurboQuantGateway, VllmConfig};
use gzmo_core::skills::{dispatch, register_pantheon, SkillRegistry};

const USAGE: &str = "\
Usage: gzmo chaos skill <command> [args...] [--json]

Run a ritual/lab pantheon skill with the latest saved chaos snapshot.
Feedback is queued for a chat or TUI ritual bridge; this command never starts
the living daemon or a PulseLoop.
";

/// Run a one-shot pantheon skill without living-daemon dependencies.
pub async fn run(config: &GzmoConfig, args: &[String]) -> Result<()> {
    if args.is_empty() || matches!(args[0].as_str(), "help" | "--help" | "-h") {
        print!("{USAGE}");
        return Ok(());
    }

    let cmd = &args[0];
    let (skill_args, json) = split_json_flag(&args[1..]);

    let mut registry = SkillRegistry::new();
    register_pantheon(&mut registry, config);

    let data_dir = dispatch::data_dir(config);
    let snap = dispatch::load_live_chaos_snapshot(data_dir, &ChaosSnapshot::default());
    let gateway: Arc<dyn LlmGateway> = Arc::new(TurboQuantGateway::new(VllmConfig::from(
        config.engine.active_engine(),
    )));
    gateway.set_chaos_overrides(snap.llm_temperature, snap.llm_max_tokens);

    // A local sender preserves normal skill behavior. Feedback is persisted
    // below from the returned SkillOutput, not delivered to a live daemon.
    let (feedback_tx, _feedback_rx) = mpsc::channel(64);
    let output = dispatch::run_registry_skill_with_gateway(
        &registry,
        config,
        cmd,
        &skill_args,
        &snap,
        &feedback_tx,
        Some(gateway),
    )
    .await?;

    let inbox = feedback_ipc::default_inbox_path(data_dir);
    for event in &output.feedback {
        feedback_ipc::append_event(&inbox, event)?;
    }

    if json {
        let payload = output.evidence.unwrap_or_else(|| {
            serde_json::json!({
                "skill": cmd,
                "display": output.display,
                "feedback_count": output.feedback.len(),
            })
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else if !output.display.is_empty() {
        print!("{}", output.display);
    } else if !output.feedback.is_empty() {
        println!("(skill ok — chaos feedback queued)");
    } else {
        bail!("skill produced no output");
    }

    Ok(())
}

fn split_json_flag(args: &[String]) -> (String, bool) {
    let mut json = false;
    let args = args
        .iter()
        .filter(|arg| {
            if arg.as_str() == "--json" {
                json = true;
                false
            } else {
                true
            }
        })
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(" ");
    (args, json)
}

#[cfg(test)]
mod tests {
    use super::split_json_flag;

    #[test]
    fn json_flag_is_removed_from_skill_args() {
        let args = vec!["d20".into(), "--json".into(), "advantage".into()];
        assert_eq!(split_json_flag(&args), ("d20 advantage".into(), true));
    }
}
