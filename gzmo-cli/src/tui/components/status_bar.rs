//! Top ops rail — mode, model, chaos vitals, host load.

use color_eyre::Result;
use gzmo_chaos::pulse::ChaosSnapshot;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

use crate::tui::action::Action;
use crate::tui::component::Component;
use crate::tui::theme::{self, COPPER, CYAN, GOLD, MUTED, PARCHMENT, RITUAL, STEEL, TEXT, PULSE_STALE_SECS};

pub struct StatusBarComponent {
    pub last_snapshot: Option<ChaosSnapshot>,
    pub last_snapshot_at: Option<Instant>,
    pub mode: String,
    pub model: String,
    pub cache_phase: String,
    pub cache_energy: String,
    pub cache_tension: String,
    pub cache_rho: String,
    pub cache_cpu: String,
    pub cache_mem: String,
    pub llm_status: String,
    pub llm_latency: String,
    pub speaking: bool,
    pub alert: Option<String>,
    alert_ticks: u32,
}

impl StatusBarComponent {
    pub fn new(mode: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            last_snapshot: None,
            last_snapshot_at: None,
            mode: mode.into(),
            model: model.into(),
            cache_phase: "IDLE".into(),
            cache_energy: "—".into(),
            cache_tension: "—".into(),
            cache_rho: "28.0".into(),
            cache_cpu: "—".into(),
            cache_mem: "—".into(),
            llm_status: "—".into(),
            llm_latency: String::new(),
            speaking: false,
            alert: None,
            alert_ticks: 0,
        }
    }
}

impl Component for StatusBarComponent {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::ChaosSnapshot(snap) => {
                self.cache_phase = format!("{:?}", snap.phase).to_uppercase();
                self.cache_energy = format!("{:.0}", snap.energy);
                self.cache_tension = format!("{:.0}", snap.tension);
                self.cache_rho = format!("{:.2}", snap.rho_effective);
                self.last_snapshot = Some(snap);
                self.last_snapshot_at = Some(Instant::now());
            }
            Action::EngineHealth(status, latency) => {
                self.llm_status = status;
                self.llm_latency = latency;
            }
            Action::Telemetry(cpu, mem) => {
                self.cache_cpu = format!("{:.0}", cpu);
                self.cache_mem = format!("{:.0}", mem);
            }
            Action::AgentTokenStream(_) => {
                self.speaking = true;
            }
            Action::AgentResponse(_) => {
                self.speaking = false;
            }
            Action::TriggerNotification(msg) => {
                let short: String = msg.chars().take(48).collect();
                self.alert = Some(short);
                self.alert_ticks = 180; // ~3s
            }
            Action::Tick => {
                if self.alert_ticks > 0 {
                    self.alert_ticks -= 1;
                    if self.alert_ticks == 0 {
                        self.alert = None;
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let t_val = self
            .last_snapshot
            .as_ref()
            .map(|s| s.tension)
            .unwrap_or(0.0);
        let drop = self
            .last_snapshot
            .as_ref()
            .map(|s| matches!(s.phase, gzmo_chaos::chaos::Phase::Drop))
            .unwrap_or(false);
        let border = theme::chrome_border(t_val, self.speaking, drop);
        let t_style = Style::default().fg(theme::tension_color(t_val));

        // Build left→right by priority so narrow rails keep phase/vitals intact
        // (previously IDLE truncated to "ID" at 80 cols).
        let stale = self
            .last_snapshot_at
            .map(|t| t.elapsed() > Duration::from_secs(PULSE_STALE_SECS))
            .unwrap_or(true);

        let mut spans = vec![
            Span::styled(
                " GZMO ",
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            ),
            Span::styled("SOVEREIGN", Style::default().fg(COPPER)),
            Span::styled(" │ ", Style::default().fg(MUTED)),
            Span::styled(&self.mode, Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(MUTED)),
            Span::styled("τ", Style::default().fg(MUTED)),
            Span::styled(&self.cache_tension, t_style),
            Span::styled(" ε", Style::default().fg(MUTED)),
            Span::styled(&self.cache_energy, Style::default().fg(TEXT)),
            Span::styled(" ρ", Style::default().fg(MUTED)),
            Span::styled(
                &self.cache_rho,
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(MUTED)),
            Span::styled(&self.cache_phase, Style::default().fg(PARCHMENT)),
        ];

        if stale {
            spans.push(Span::styled(" │ ", Style::default().fg(MUTED)));
            spans.push(Span::styled(
                "PULSE STALE",
                Style::default()
                    .fg(RITUAL)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        let llm_label = if self.llm_status == "ONLINE" {
            format!(
                "LLM ONLINE{}",
                if self.llm_latency.is_empty() {
                    String::new()
                } else {
                    format!(" {}", self.llm_latency)
                }
            )
        } else if self.llm_status == "OFFLINE" {
            "LLM OFFLINE".to_string()
        } else {
            String::new()
        };
        if !llm_label.is_empty() {
            spans.push(Span::styled(" │ ", Style::default().fg(MUTED)));
            spans.push(Span::styled(
                llm_label,
                Style::default().fg(if self.llm_status == "ONLINE" {
                    CYAN
                } else {
                    RITUAL
                }),
            ));
        }

        let mut used = line_width(&spans);
        let budget = area.width.saturating_sub(1) as usize;

        let model = short_model(&self.model, 28);
        let model_chunk = format!(" │ {model}");
        if used + model_chunk.chars().count() <= budget {
            spans.push(Span::styled(" │ ", Style::default().fg(MUTED)));
            spans.push(Span::styled(model, Style::default().fg(STEEL)));
            used = line_width(&spans);
        }

        let host = format!(" │ cpu {} mem {}", self.cache_cpu, self.cache_mem);
        if used + host.chars().count() <= budget {
            spans.push(Span::styled(
                host,
                Style::default().fg(STEEL),
            ));
            used = line_width(&spans);
        }

        if self.speaking {
            let stream = " * STREAM";
            if used + stream.chars().count() <= budget {
                spans.push(Span::styled(
                    stream,
                    Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
                ));
                used = line_width(&spans);
            }
        }

        if let Some(alert) = &self.alert {
            let a = format!(" ! {alert}");
            if used + a.chars().count() <= budget {
                spans.push(Span::styled(
                    a,
                    Style::default().fg(RITUAL).add_modifier(Modifier::BOLD),
                ));
            }
        }

        let p = Paragraph::new(Line::from(spans))
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(border)),
            );

        f.render_widget(p, area);
        Ok(())
    }
}

fn line_width(spans: &[Span<'_>]) -> usize {
    spans.iter().map(|s| s.content.chars().count()).sum()
}

fn short_model(model: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let base = model
        .rsplit(['/', '\\'])
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(model);
    if base.chars().count() <= max {
        return base.to_string();
    }
    let mut out: String = base.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::short_model;

    #[test]
    fn short_model_uses_basename() {
        assert_eq!(
            short_model("/home/gzmo/models/ornith-35b-Q4_K_M.gguf", 40),
            "ornith-35b-Q4_K_M.gguf"
        );
    }

    #[test]
    fn short_model_truncates_long_basename() {
        let s = short_model("very-long-model-name-that-will-not-fit.gguf", 16);
        assert_eq!(s.chars().count(), 16);
        assert!(s.ends_with('…'));
    }
}
