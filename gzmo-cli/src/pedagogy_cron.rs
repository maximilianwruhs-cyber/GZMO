//! Optional cron trigger for pedagogy oscillation when `schedule_mode = cron`.

use gzmo_chaos::feedback::ChaosEvent;
use gzmo_chaos::feedback_ipc;
use gzmo_chaos::pedagogy_oscillator::PedagogyOscillateAction;
use gzmo_core::config::{GzmoConfig, TensionOscillationScheduleMode};

pub fn maybe_queue_cron_start(config: &GzmoConfig, hour_utc: u32) -> anyhow::Result<bool> {
    let osc = &config.pedagogy.tension_oscillation;
    if !osc.enabled || osc.schedule_mode != TensionOscillationScheduleMode::Cron {
        return Ok(false);
    }
    if osc.cron_hours.is_empty() {
        return Ok(false);
    }
    if !osc.cron_hours.contains(&hour_utc) {
        return Ok(false);
    }
    let data_dir = config
        .memory
        .vault_db
        .parent()
        .unwrap_or(std::path::Path::new("data"));
    let inbox = feedback_ipc::default_inbox_path(data_dir);
    let event = ChaosEvent::PedagogyOscillate {
        action: PedagogyOscillateAction::Start,
    };
    feedback_ipc::append_event(&inbox, &event)?;
    tracing::info!(hour_utc, "Queued pedagogy oscillation start (cron schedule)");
    Ok(true)
}
