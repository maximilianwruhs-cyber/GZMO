// Visual Skill — Chaos-driven procedural art rendered in the terminal.
//
// Generates images via the Python chaos_art.py generator using current
// engine state, then renders them in-terminal via `chafa`.
//
// Modes: lorenz, energy, mood, sigil
// Usage: /visual [mode]

use std::process::Command;

use anyhow::Result;
use async_trait::async_trait;

use crate::skills::{Skill, SkillContext, SkillOutput, SkillType};
use gzmo_chaos::feedback::ChaosEvent;

pub struct VisualSkill;

const MODES: &[&str] = &["lorenz", "energy", "mood", "sigil"];

#[async_trait]
impl Skill for VisualSkill {
    fn name(&self) -> &str { "visual" }
    fn description(&self) -> &str { "Chaos-driven procedural art rendered in the terminal" }
    fn skill_type(&self) -> SkillType { SkillType::Mechanical }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let snap = ctx.chaos;

        let mode = if ctx.args.trim().is_empty() {
            // Auto-select based on tension/chaos
            if snap.tension > 70.0 { "energy" }
            else if snap.tension < 30.0 { "mood" }
            else if snap.chaos_val > 0.7 { "sigil" }
            else { "lorenz" }
        } else {
            let requested = ctx.args.trim();
            let lower = requested.to_lowercase();
            if MODES.contains(&lower.as_str()) {
                match lower.as_str() {
                    "lorenz" => "lorenz",
                    "energy" => "energy",
                    "mood" => "mood",
                    "sigil" => "sigil",
                    _ => "lorenz",
                }
            } else {
                return Ok(SkillOutput {
                    display: format!(
                        "\n┌─────────────────────────────────────────────────┐\n  \
                         ⚠ Unknown visual mode: {}\n  \
                         Available: {}\n\
                         └─────────────────────────────────────────────────┘",
                        requested,
                        MODES.join(", "),
                    ),
                    feedback: vec![],
                    inject_to_conversation: false,
                });
            }
        };

        // Write current chaos state for the Python generator
        let state_path = "/tmp/gzmo_visual_state.json";
        let output_path = format!("/tmp/gzmo_visual_{}.png", snap.tick);

        if let Ok(json) = serde_json::to_string_pretty(snap) {
            let _ = std::fs::write(state_path, &json);
        }

        // Run Python generator
        let gen_result = Command::new("python3")
            .arg("skills/visuals/chaos_art.py")
            .arg(mode)
            .arg(&output_path)
            .arg("--state")
            .arg(state_path)
            .output();

        match gen_result {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.contains("OK:") {
                    return Ok(SkillOutput {
                        display: format!("  ⚠ Generator failed: {}", stdout),
                        feedback: vec![],
                        inject_to_conversation: false,
                    });
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Ok(SkillOutput {
                    display: format!("  ⚠ Generator error: {}", stderr),
                    feedback: vec![],
                    inject_to_conversation: false,
                });
            }
            Err(e) => {
                return Ok(SkillOutput {
                    display: format!("  ⚠ Failed to run generator: {}", e),
                    feedback: vec![],
                    inject_to_conversation: false,
                });
            }
        }

        // Render via chafa
        let chafa_result = Command::new("chafa")
            .arg("--size=50x25")
            .arg("--format=symbols")
            .arg(&output_path)
            .output();

        let rendered = match chafa_result {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Ok(SkillOutput {
                    display: format!("  ⚠ chafa error: {}", stderr),
                    feedback: vec![],
                    inject_to_conversation: false,
                });
            }
            Err(_) => {
                return Ok(SkillOutput {
                    display: "  ⚠ chafa not found. Install: apt install chafa".to_string(),
                    feedback: vec![],
                    inject_to_conversation: false,
                });
            }
        };

        // Clean up temp file
        let _ = std::fs::remove_file(&output_path);

        // Build output
        let phase_str = format!("{}", snap.phase);
        let valence_desc = if snap.llm_valence < -0.5 { "dark" }
            else if snap.llm_valence < 0.0 { "tense" }
            else if snap.llm_valence < 0.5 { "calm" }
            else { "serene" };

        let display = format!(
            "\n\
            ┌─────────────────────────────────────────────────────────────┐\n  \
            🎨 CHAOS VISUAL  [{mode}]  tick {tick}\n\
            ├─────────────────────────────────────────────────────────────┤\n\
            \n{art}\n\
            ├─────────────────────────────────────────────────────────────┤\n  \
            Mood: {valence_desc} | Phase: {phase} | τ:{tension:.0}% ε:{energy:.0}%\n\
            └─────────────────────────────────────────────────────────────┘",
            mode = mode,
            tick = snap.tick,
            art = rendered.lines()
                .map(|l| format!("  {l}"))
                .collect::<Vec<_>>()
                .join("\n"),
            valence_desc = valence_desc,
            phase = phase_str,
            tension = snap.tension,
            energy = snap.energy,
        );

        // Emit a subtle custom feedback event — visuals drain a bit of energy
        let feedback = vec![
            ChaosEvent::Custom {
                tension_delta: -1.0, // Visual art is calming
                energy_delta: -0.5,  // Slight energy cost
                thought_seed: None,
            }
        ];

        Ok(SkillOutput {
            display,
            feedback,
            inject_to_conversation: false,
        })
    }
}
