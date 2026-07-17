//! Full-screen ecosystem health LED board for `gzmo observatory`.

use std::io::stdout;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use gzmo_core::config::GzmoConfig;
use gzmo_core::observatory_board::{collect_health_led_board, HealthLed, HealthLedBoard, LedState};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Terminal;

use crate::tui::theme::{self, COPPER, CYAN, GOLD, MUTED, PARCHMENT, RITUAL, STEEL, TEXT};

fn led_color(state: LedState) -> ratatui::style::Color {
    match state {
        LedState::Up => CYAN,
        LedState::Degraded => GOLD,
        LedState::Down => RITUAL,
        LedState::Unknown => STEEL,
    }
}

fn led_glyph(state: LedState) -> &'static str {
    match state {
        LedState::Up => "●",
        LedState::Degraded => "◐",
        LedState::Down => "●",
        LedState::Unknown => "○",
    }
}

fn led_line(led: &HealthLed) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{} ", led_glyph(led.state)),
            Style::default().fg(led_color(led.state)),
        ),
        Span::styled(
            format!("{:<22}", led.label),
            Style::default().fg(PARCHMENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<8}", led.state.label()),
            Style::default().fg(led_color(led.state)),
        ),
        Span::styled(truncate(&led.detail, 48), Style::default().fg(MUTED)),
    ])
}

fn truncate(s: &str, max: usize) -> String {
    let t: String = s.chars().take(max).collect();
    if s.chars().count() > max {
        format!("{t}…")
    } else {
        t
    }
}

fn render_board(f: &mut ratatui::Frame<'_>, board: &HealthLedBoard, status: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(f.size());

    let (up, deg, down, unk) = board.counts();
    let rail = Line::from(vec![
        Span::styled(
            " GZMO OBSERVATORY ",
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ),
        Span::styled("HEALTH", Style::default().fg(COPPER)),
        Span::styled("  │  ", Style::default().fg(MUTED)),
        Span::styled(&board.instance, Style::default().fg(CYAN)),
        Span::styled("  ", Style::default()),
        Span::styled(&board.engine_mode, Style::default().fg(STEEL)),
        Span::styled("  ", Style::default()),
        Span::styled(&board.engine_model, Style::default().fg(MUTED)),
        Span::styled("  │  ", Style::default().fg(MUTED)),
        Span::styled(format!("↑{up} "), Style::default().fg(CYAN)),
        Span::styled(format!("~{deg} "), Style::default().fg(GOLD)),
        Span::styled(format!("↓{down} "), Style::default().fg(RITUAL)),
        Span::styled(format!("?{unk}"), Style::default().fg(STEEL)),
    ]);
    f.render_widget(
        Paragraph::new(rail).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme::DIM)),
        ),
        chunks[0],
    );

    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(chunks[1]);

    let unit_lines: Vec<Line> = board.units.iter().map(led_line).collect();
    f.render_widget(
        Paragraph::new(unit_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" UNITS ")
                .title_style(Style::default().fg(COPPER).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(theme::DIM)),
        ),
        mid[0],
    );

    let probe_lines: Vec<Line> = board.probes.iter().map(led_line).collect();
    f.render_widget(
        Paragraph::new(probe_lines)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" PROBES ")
                    .title_style(Style::default().fg(COPPER).add_modifier(Modifier::BOLD))
                    .border_style(Style::default().fg(theme::DIM)),
            ),
        mid[1],
    );

    let failed: Vec<&HealthLed> = board
        .all_leds()
        .filter(|l| matches!(l.state, LedState::Down | LedState::Degraded))
        .collect();
    let detail = if failed.is_empty() {
        format!("{status}  ·  all clear  ·  r refresh · q quit")
    } else {
        let first = failed[0];
        format!(
            "{status}  ·  {} {}: {}  ·  r refresh · q quit",
            first.label,
            first.state.label(),
            truncate(&first.detail, 60)
        )
    };
    f.render_widget(
        Paragraph::new(Span::styled(detail, Style::default().fg(TEXT))).block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme::DIM)),
        ),
        chunks[2],
    );
}

/// Run the Observatory health LED TUI until quit.
pub async fn run(config: &GzmoConfig) -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut board = collect_health_led_board(config).await;
    let mut status = "probed".to_string();
    let mut refresh = tokio::time::interval(Duration::from_secs(2));
    refresh.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    let result = loop {
        terminal.draw(|f| render_board(f, &board, &status))?;

        tokio::select! {
            _ = refresh.tick() => {
                board = collect_health_led_board(config).await;
                status = "auto".into();
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                if event::poll(Duration::from_millis(0))? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                                KeyCode::Char('c')
                                    if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
                                {
                                    break Ok(());
                                }
                                KeyCode::Char('r') => {
                                    board = collect_health_led_board(config).await;
                                    status = "manual".into();
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    };

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    result
}
