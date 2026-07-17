//! Headless checks for TUI chaos → canvas pipeline.
#![cfg(test)]

use std::time::Duration;

use gzmo_chaos::pulse::{ChaosConfig, PulseLoop};

use crate::tui::action::Action;
use crate::tui::component::Component;
use crate::tui::components::chaos_canvas::ChaosCanvasComponent;
use crate::tui::components::instruments::InstrumentsComponent;

#[tokio::test]
async fn pulse_loop_feeds_canvas_history() {
    let handle = PulseLoop::start(ChaosConfig::default());
    let mut rx = handle.snapshot_rx.clone();
    let mut canvas = ChaosCanvasComponent::new();

    // Wait for the pulse to advance past the default tick
    let mut saw_motion = false;
    for _ in 0..40 {
        let _ = tokio::time::timeout(Duration::from_millis(500), rx.changed()).await;
        let snap = rx.borrow_and_update().clone();
        canvas.update(Action::ChaosSnapshot(snap.clone())).unwrap();
        if snap.tick > 5 && canvas.history.len() > 5 {
            // Coordinates should have left the near-zero default seed
            let (x, y, z) = canvas.last_xyz;
            if x.abs() + y.abs() + z.abs() > 1.0 {
                saw_motion = true;
                break;
            }
        }
    }

    assert!(saw_motion, "pulse never moved Lorenz state into canvas");
    assert!(canvas.live);
    assert!(canvas.history.len() >= 2);

    let (xb, yb) = ChaosCanvasComponent::compute_bounds(&canvas.history);
    assert!(xb[1] > xb[0]);
    assert!(yb[1] > yb[0]);
}

#[tokio::test]
async fn instruments_receive_rho_from_pulse() {
    let handle = PulseLoop::start(ChaosConfig::default());
    let mut rx = handle.snapshot_rx.clone();
    let mut instruments = InstrumentsComponent::new(None);

    let mut ok = false;
    for _ in 0..30 {
        let _ = tokio::time::timeout(Duration::from_millis(400), rx.changed()).await;
        let snap = rx.borrow_and_update().clone();
        instruments
            .update(Action::ChaosSnapshot(snap.clone()))
            .unwrap();
        if snap.tick > 0 && snap.rho_effective > 0.0 {
            ok = true;
            break;
        }
    }
    assert!(ok, "instruments never saw a live rho sample");
}
