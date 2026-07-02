//! Telemetry Dashboard — Real-time parameter effectiveness monitoring
//!
//! Displays live metrics from the chaos engine and self-improving loop,
//! showing the effectiveness of parameter modulation in escaping
//! repetitive patterns and improving output diversity.

use anyhow::Result;
use std::io::{self, Write};
use std::time::Duration;
use tokio::time::interval;

use gzmo_chaos::pulse::ChaosSnapshot;

/// Run the telemetry dashboard TUI
pub async fn run(snapshot_rx: tokio::sync::watch::Receiver<ChaosSnapshot>) -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║           GZMO Telemetry Dashboard                               ║");
    println!("║           Real-time Parameter Effectiveness                      ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Press Ctrl+C to exit");
    println!();

    let mut ticker = interval(Duration::from_millis(500));

    loop {
        ticker.tick().await;

        let snap = snapshot_rx.borrow().clone();
        print_dashboard(&snap);

        // Check for exit condition (simple key check not available in async,
        // so we just refresh continuously)
    }
}

/// Print a single dashboard frame
fn print_dashboard(snap: &ChaosSnapshot) {
    // Clear screen (ANSI escape codes)
    print!("\x1B[2J\x1B[H");

    // Header
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│  GZMO Telemetry Dashboard    Tick: {:<6}   Time: {:<12}   │",
        snap.tick, snap.timestamp);
    println!("└─────────────────────────────────────────────────────────────────┘");

    // Chaos Engine State
    println!("┌─ Chaos Engine ──────────────────────────────────────────────────┐");
    println!("│  Lorenz:  x={:>7.3}  y={:>7.3}  z={:>7.3}                     │",
        snap.x, snap.y, snap.z);
    println!("│  Energy:  {:>6.1}%  Phase: {:<12?}  Alive: {:<5}   │",
        snap.energy, snap.phase, snap.alive);
    println!("│  Tension: {:>6.1}   ρ_eff: {:>6.2}   ρ_delta: {:>+6.3}         │",
        snap.tension, snap.rho_effective, snap.rho_mod_delta);
    println!("└─────────────────────────────────────────────────────────────────┘");

    // LLM Parameters
    println!("┌─ LLM Parameters ──────────────────────────────────────────────┐");
    println!("│  Temperature: {:>5.2}   Max Tokens: {:<4}   Valence: {:>+5.2}     │",
        snap.llm_temperature, snap.llm_max_tokens, snap.llm_valence);
    println!("│  Chaos Value: {:>5.3}   Pedagogy Osc: {:<5}                     │",
        snap.chaos_val, if snap.pedagogy_oscillation_active { "ON" } else { "OFF" });
    println!("└─────────────────────────────────────────────────────────────────┘");

    // Self-Improving Loop Metrics
    println!("┌─ Self-Improving Loop ───────────────────────────────────────────┐");
    println!("│  Pattern State: {:<10}  Exploration: {:>5.1}%                  │",
        snap.pattern_state, snap.exploration_level * 100.0);
    println!("│  Diversity Score: {:>5.1}%  Avg Latency: {:>4}ms               │",
        snap.diversity_score * 100.0, snap.avg_latency_ms);
    println!("│  Generations: {:<4}  Stuck: {:<3}  Escaped: {:<3}  Rate: {:>5.1}% │",
        snap.generation_count, snap.stuck_count, snap.escape_count,
        if snap.stuck_count > 0 {
            (snap.escape_count as f64 / snap.stuck_count as f64) * 100.0
        } else { 0.0 });
    if let Some(effective) = snap.modulation_effective {
        println!("│  Last Modulation: {:<43} │",
            if effective { "✓ Effective" } else { "✗ Ineffective" });
    }
    println!("└─────────────────────────────────────────────────────────────────┘");

    // Parameter Mutation Queue
    println!("┌─ Parameter Mutation Queue ──────────────────────────────────────┐");
    println!("│  Pending: {:<3}  Applied: {:<5}  Gravity: {:>+6.3}              │",
        snap.thoughts_incubating, snap.thoughts_crystallized, snap.mutations.gravity_mod);
    println!("│  Friction: {:>+6.3}  Lorenz ρ: {:>+6.3}  Tension: {:>+6.3}       │",
        snap.mutations.friction_mod, snap.mutations.lorenz_rho_mod,
        snap.mutations.tension_bias);
    println!("└─────────────────────────────────────────────────────────────────┘");

    // Effectiveness Indicators
    println!("┌─ Effectiveness Summary ───────────────────────────────────────────┐");
    print_effectiveness_line("Temperature", snap.llm_temperature as f64, 0.3, 1.2, "diversity");
    print_effectiveness_line("Exploration", snap.exploration_level, 0.0, 1.0, "escape rate");
    print_effectiveness_line("Diversity", snap.diversity_score, 0.0, 1.0, "quality");
    println!("└─────────────────────────────────────────────────────────────────┘");

    io::stdout().flush().unwrap();
}

/// Print a visual effectiveness bar
fn print_effectiveness_line(label: &str, value: f64, min: f64, max: f64, metric: &str) {
    let normalized = ((value - min) / (max - min)).clamp(0.0, 1.0);
    let bar_width = 20usize;
    let filled = (normalized * bar_width as f64) as usize;
    let bar: String = std::iter::repeat('█')
        .take(filled)
        .chain(std::iter::repeat('░').take(bar_width - filled))
        .collect();

    println!("│  {:<12} [{:}] {:>5.2} → {:<12}              │",
        label, bar, value, metric);
}

/// Run one-shot telemetry report (non-interactive)
pub async fn report(snapshot_rx: tokio::sync::watch::Receiver<ChaosSnapshot>) -> Result<()> {
    let snap = snapshot_rx.borrow().clone();

    println!("# GZMO Telemetry Report");
    println!("Generated: {}", chrono::Local::now().to_rfc3339());
    println!();

    println!("## Chaos Engine State");
    println!("- Tick: {}", snap.tick);
    println!("- Lorenz: x={:.3}, y={:.3}, z={:.3}", snap.x, snap.y, snap.z);
    println!("- Energy: {:.1}%", snap.energy);
    println!("- Phase: {:?}", snap.phase);
    println!("- Alive: {}", snap.alive);
    println!("- Tension: {:.1}", snap.tension);
    println!("- ρ (effective): {:.2}", snap.rho_effective);
    println!();

    println!("## LLM Parameters");
    println!("- Temperature: {:.2}", snap.llm_temperature);
    println!("- Max Tokens: {}", snap.llm_max_tokens);
    println!("- Valence: {:+.2}", snap.llm_valence);
    println!();

    println!("## Self-Improving Loop");
    println!("- Pattern State: {}", snap.pattern_state);
    println!("- Exploration Level: {:.1}%", snap.exploration_level * 100.0);
    println!("- Diversity Score: {:.1}%", snap.diversity_score * 100.0);
    println!("- Average Latency: {}ms", snap.avg_latency_ms);
    println!("- Total Generations: {}", snap.generation_count);
    println!("- Stuck Detections: {}", snap.stuck_count);
    println!("- Successful Escapes: {}", snap.escape_count);
    if snap.stuck_count > 0 {
        println!("- Escape Rate: {:.1}%",
            (snap.escape_count as f64 / snap.stuck_count as f64) * 100.0);
    }
    println!();

    println!("## Parameter Mutations");
    println!("- Pending thoughts: {}", snap.thoughts_incubating);
    println!("- Crystallized: {}", snap.thoughts_crystallized);
    println!("- Gravity Mod: {:+.3}", snap.mutations.gravity_mod);
    println!("- Friction Mod: {:+.3}", snap.mutations.friction_mod);
    println!("- Lorenz ρ Mod: {:+.3}", snap.mutations.lorenz_rho_mod);
    println!("- Tension Bias: {:+.3}", snap.mutations.tension_bias);
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effectiveness_bar() {
        // Just verify it doesn't panic
        print_effectiveness_line("Test", 0.5, 0.0, 1.0, "test");
    }
}