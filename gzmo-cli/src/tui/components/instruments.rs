//! Right-rail instruments: phase meter + cargo (vault / ρ / thoughts).

use color_eyre::Result;
use gzmo_chaos::chaos::Phase;
use gzmo_chaos::pulse::ChaosSnapshot;
use gzmo_core::memory::vault::SqliteVault;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};
use std::sync::Arc;

use crate::tui::action::Action;
use crate::tui::component::Component;
use crate::tui::theme::{self, COPPER, CYAN, GOLD, MUTED, PARCHMENT, RITUAL, STEEL, TEXT};

pub struct InstrumentsComponent {
    snap: Option<ChaosSnapshot>,
    vault: Option<Arc<SqliteVault>>,
    vault_count: u64,
    tick: u64,
    speaking: bool,
}

impl InstrumentsComponent {
    pub fn new(vault: Option<Arc<SqliteVault>>) -> Self {
        let vault_count = vault.as_ref().and_then(|v| v.count().ok()).unwrap_or(0) as u64;
        Self {
            snap: None,
            vault,
            vault_count,
            tick: 0,
            speaking: false,
        }
    }

    fn refresh_vault(&mut self) {
        if let Some(v) = &self.vault {
            if let Ok(n) = v.count() {
                self.vault_count = n as u64;
            }
        }
    }
}

impl Component for InstrumentsComponent {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::ChaosSnapshot(s) => {
                self.snap = Some(s);
            }
            Action::AgentTokenStream(_) => {
                self.speaking = true;
            }
            Action::AgentResponse(_) => {
                self.speaking = false;
            }
            Action::Tick => {
                self.tick = self.tick.wrapping_add(1);
                if self.tick.is_multiple_of(90) {
                    self.refresh_vault();
                }
                // speaking afterglow decays visually via agent actions; soft clear
                if self.speaking && self.tick.is_multiple_of(45) {
                    // keep until AgentResponse — no-op
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let tension = self.snap.as_ref().map(|s| s.tension).unwrap_or(0.0);
        let phase = self.snap.as_ref().map(|s| s.phase).unwrap_or(Phase::Idle);
        let drop = matches!(phase, Phase::Drop);
        let border = theme::chrome_border(tension, self.speaking, drop);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // phase
                Constraint::Min(4),    // cargo
            ])
            .split(area);

        // ── Phase meter ──────────────────────────────────────────
        let phase_ratio = (tension / 100.0).clamp(0.0, 1.0) as f64;
        let phase_label = match phase {
            Phase::Idle => "IDLE",
            Phase::Build => "BUILD",
            Phase::Drop => "DROP",
        };
        let gauge_color = theme::tension_color(tension);
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" PHASE ")
                    .title_style(Style::default().fg(COPPER).add_modifier(Modifier::BOLD))
                    .border_style(Style::default().fg(border)),
            )
            .gauge_style(Style::default().fg(gauge_color).bg(theme::DIM))
            .ratio(phase_ratio)
            .label(format!("{phase_label}  τ{tension:.0}%"));
        f.render_widget(gauge, chunks[0]);

        // ── Cargo / homeostasis ──────────────────────────────────
        let (rho, d_rho, thoughts_i, thoughts_c, valence, deaths, alive, energy) =
            if let Some(s) = &self.snap {
                (
                    s.rho_effective,
                    s.rho_mod_delta,
                    s.thoughts_incubating,
                    s.thoughts_crystallized,
                    s.llm_valence,
                    s.deaths,
                    s.alive,
                    s.energy,
                )
            } else {
                (28.0, 0.0, 0, 0, 0.0, 0, true, 100.0)
            };

        let alive_span = if alive {
            Span::styled("PULSE OK", Style::default().fg(CYAN))
        } else {
            Span::styled(
                "FALLEN",
                Style::default().fg(RITUAL).add_modifier(Modifier::BOLD),
            )
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("chaos   ", Style::default().fg(MUTED)),
                alive_span,
                Span::styled(format!("  ×{deaths}"), Style::default().fg(STEEL)),
            ]),
            Line::from(vec![
                Span::styled("vault   ", Style::default().fg(MUTED)),
                Span::styled(
                    format!("{} records", self.vault_count),
                    Style::default().fg(PARCHMENT),
                ),
            ]),
            Line::from(vec![
                Span::styled("thoughts ", Style::default().fg(MUTED)),
                Span::styled(
                    format!("{thoughts_i} incub · {thoughts_c} crystal"),
                    Style::default().fg(TEXT),
                ),
            ]),
            Line::from(vec![
                Span::styled("ρ       ", Style::default().fg(MUTED)),
                Span::styled(
                    format!("{rho:.2}"),
                    Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  Δ{d_rho:+.3}"),
                    Style::default().fg(if d_rho >= 0.0 { CYAN } else { COPPER }),
                ),
            ]),
            Line::from(vec![
                Span::styled("valence ", Style::default().fg(MUTED)),
                Span::styled(
                    format!("{valence:+.2}"),
                    Style::default().fg(theme::tension_color(((1.0 - valence) * 50.0) as f64)),
                ),
                Span::styled(format!("  ε{energy:.0}%"), Style::default().fg(STEEL)),
            ]),
        ];

        let cargo = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" CARGO ")
                .title_style(Style::default().fg(COPPER).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(border)),
        );
        f.render_widget(cargo, chunks[1]);
        Ok(())
    }
}
