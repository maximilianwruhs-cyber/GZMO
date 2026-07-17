//! Full-screen overnight metabolism board for `gzmo metabolism`.

use std::io::stdout;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use gzmo_core::config::GzmoConfig;
use gzmo_core::metabolism::{collect_metabolism_board, JobRowStatus, MetabolismBoard};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Terminal;

use crate::tui::theme::{self, COPPER, CYAN, GOLD, MUTED, PARCHMENT, RITUAL, STEEL, TEXT};

fn status_color(s: JobRowStatus) -> ratatui::style::Color {
    match s {
        JobRowStatus::Ok => CYAN,
        JobRowStatus::Fail => RITUAL,
        JobRowStatus::Missing => STEEL,
    }
}

fn verdict_color(v: &str) -> ratatui::style::Color {
    if v.starts_with("GREEN") {
        CYAN
    } else if v.starts_with("YELLOW") {
        GOLD
    } else {
        RITUAL
    }
}

fn render_board(f: &mut ratatui::Frame<'_>, board: &MetabolismBoard, status: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(f.size());

    let newest = board
        .newest
        .as_ref()
        .map(|j| format!("{} {}", if j.ok { "OK" } else { "FAIL" }, j.job))
        .unwrap_or_else(|| "none".into());

    let rail = Line::from(vec![
        Span::styled(
            " GZMO METABOLISM ",
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ),
        Span::styled("OVERNIGHT", Style::default().fg(COPPER)),
        Span::styled("  │  ", Style::default().fg(MUTED)),
        Span::styled("latest ", Style::default().fg(MUTED)),
        Span::styled(newest, Style::default().fg(PARCHMENT)),
        Span::styled("  │  ", Style::default().fg(MUTED)),
        Span::styled(
            &board.verdict,
            Style::default()
                .fg(verdict_color(&board.verdict))
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(
        Paragraph::new(rail).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme::DIM)),
        ),
        chunks[0],
    );

    let mut rows = vec![Line::from(vec![
        Span::styled(
            format!("{:<10}", "JOB"),
            Style::default().fg(MUTED).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<28}", "LAST RUN"),
            Style::default().fg(MUTED).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<8}", "RESULT"),
            Style::default().fg(MUTED).add_modifier(Modifier::BOLD),
        ),
        Span::styled("DETAIL", Style::default().fg(MUTED).add_modifier(Modifier::BOLD)),
    ])];

    for job in &board.jobs {
        let finished = job.finished.as_deref().unwrap_or("—");
        let detail = job
            .error
            .as_deref()
            .or(job.runner.as_deref())
            .unwrap_or("");
        rows.push(Line::from(vec![
            Span::styled(
                format!("{:<10}", job.job),
                Style::default().fg(PARCHMENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{finished:<28}"), Style::default().fg(TEXT)),
            Span::styled(
                format!("{:<8}", job.status.label()),
                Style::default().fg(status_color(job.status)),
            ),
            Span::styled(detail.to_string(), Style::default().fg(MUTED)),
        ]));
    }

    f.render_widget(
        Paragraph::new(rows).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" JOBS  ·  {} ", board.runs_dir.display()))
                .title_style(Style::default().fg(COPPER).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(theme::DIM)),
        ),
        chunks[1],
    );

    let wiki_color = match board.wiki.healthy {
        Some(true) => CYAN,
        Some(false) => RITUAL,
        None => STEEL,
    };
    let cargo = vec![
        Line::from(vec![
            Span::styled("honeypot  ", Style::default().fg(MUTED)),
            Span::styled(
                board
                    .honeypot_rows
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "?".into()),
                Style::default().fg(PARCHMENT),
            ),
            Span::styled("   missing embeddings  ", Style::default().fg(MUTED)),
            Span::styled(
                board
                    .missing_embeddings
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "?".into()),
                Style::default().fg(PARCHMENT),
            ),
        ]),
        Line::from(vec![
            Span::styled("wiki push  ", Style::default().fg(MUTED)),
            Span::styled(
                board.wiki.detail.clone(),
                Style::default().fg(wiki_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            "Observatory web: http://127.0.0.1:3000/observatory",
            Style::default().fg(STEEL),
        )),
    ];
    f.render_widget(
        Paragraph::new(cargo).wrap(Wrap { trim: false }).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" CARGO / WIKI ")
                .title_style(Style::default().fg(COPPER).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(theme::DIM)),
        ),
        chunks[2],
    );

    f.render_widget(
        Paragraph::new(Span::styled(
            format!("{status}  ·  r refresh · q quit"),
            Style::default().fg(TEXT),
        ))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme::DIM)),
        ),
        chunks[3],
    );
}

/// Run the metabolism overnight board TUI until quit.
pub async fn run(config: &GzmoConfig) -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut board = collect_metabolism_board(config);
    let mut status = "loaded".to_string();
    let mut refresh = tokio::time::interval(Duration::from_secs(3));
    refresh.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    let result = loop {
        terminal.draw(|f| render_board(f, &board, &status))?;

        tokio::select! {
            _ = refresh.tick() => {
                board = collect_metabolism_board(config);
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
                                    board = collect_metabolism_board(config);
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
