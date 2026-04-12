use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use color_eyre::Result;
use gzmo_chaos::pulse::ChaosSnapshot;

use crate::tui::action::Action;
use crate::tui::component::Component;

pub struct StatusBarComponent {
    pub last_snapshot: Option<ChaosSnapshot>,
    pub cpu: Option<f32>,
    pub mem: Option<f32>,
    // Pre-cached strings to avoid 60fps format! allocations
    pub cache_phase: String,
    pub cache_energy: String,
    pub cache_tension: String,
    pub cache_lorenz: String,
    pub cache_cpu: String,
    pub cache_mem: String,
}

impl StatusBarComponent {
    pub fn new() -> Self {
        Self {
            last_snapshot: None,
            cpu: None,
            mem: None,
            cache_phase: "IDLE".to_string(),
            cache_energy: "0%".to_string(),
            cache_tension: "0%".to_string(),
            cache_lorenz: "0.0, 0.0, 0.0".to_string(),
            cache_cpu: "N/A".to_string(),
            cache_mem: "N/A".to_string(),
        }
    }
}

impl Component for StatusBarComponent {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::ChaosSnapshot(snap) => {
                self.cache_phase = format!("{:?}", snap.phase).to_uppercase();
                self.cache_energy = format!("{:.0}%", snap.energy);
                self.cache_tension = format!("{:.0}%", snap.tension);
                self.cache_lorenz = format!("{:.1}, {:.1}, {:.1}", snap.x, snap.y, snap.z);
                self.last_snapshot = Some(snap);
            }
            Action::Telemetry(cpu, mem) => {
                self.cache_cpu = format!("{:.0}%", cpu);
                self.cache_mem = format!("{:.0}%", mem);
                self.cpu = Some(cpu);
                self.mem = Some(mem);
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
        let tension_style = if t_val > 80.0 {
            Style::default().fg(Color::Rgb(137, 0, 0)) // Ritual / High Tension
        } else {
            Style::default().fg(Color::Rgb(0, 245, 255)) // Static / Base Flow
        };

        let content = Line::from(vec![
            Span::styled(
                " ⚙ GZMO SOVEREIGN AGENT │ ",
                Style::default().fg(Color::Rgb(180, 130, 255)),
            ),
            Span::raw(format!("☼ Phase: {} │ ", self.cache_phase)),
            Span::styled(
                format!("⚡ Energy: {} │ ", self.cache_energy),
                Style::default().fg(Color::Rgb(201, 209, 217)),
            ),
            Span::styled(
                format!("∿ Tension: {} │ ", self.cache_tension),
                tension_style,
            ),
            Span::raw(format!("∰ Lorenz: ({}) │ ", self.cache_lorenz)),
            Span::styled(
                format!("■ CPU: {} │ ", self.cache_cpu),
                Style::default().fg(Color::Rgb(100, 200, 255)),
            ),
            Span::styled(
                format!("■ MEM: {} ", self.cache_mem),
                Style::default().fg(Color::Rgb(200, 100, 255)),
            ),
        ]);

        let p = Paragraph::new(content).alignment(Alignment::Center).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Rgb(60, 60, 70))),
        );

        f.render_widget(p, area);
        Ok(())
    }
}
