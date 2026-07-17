//! Help overlay — documents active key bindings.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::tui::theme::{COPPER, GOLD, MUTED, PARCHMENT, TEXT};

const BINDINGS: &[(&str, &str)] = &[
    ("Ctrl+P", "Open command palette"),
    ("Esc", "Close palette / help overlay"),
    ("?", "Toggle this help overlay"),
    ("Enter", "Submit input (helm) or palette command"),
    ("Up/Down", "Navigate palette selection"),
    ("PgUp/PgDn", "Scroll transcript"),
    ("Mouse wheel", "Scroll transcript"),
    ("Ctrl+C", "Quit"),
];

pub fn render_help(f: &mut Frame<'_>, area: Rect) {
    let width = 58u16.min(area.width.saturating_sub(2));
    let height = 18u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let help_area = Rect::new(x, y, width, height);

    f.render_widget(Clear, help_area);

    let mut lines = vec![
        Line::from(Span::styled(
            " KEYBOARD ",
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    for (key, desc) in BINDINGS {
        lines.push(Line::from(vec![
            Span::styled(format!("  {key:<14}"), Style::default().fg(COPPER)),
            Span::styled(*desc, Style::default().fg(TEXT)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Focus: helm when palette/help closed; palette is modal when open.",
        Style::default().fg(MUTED),
    )));
    lines.push(Line::from(Span::styled(
        "  LLM ONLINE/OFFLINE ≠ chaos pulse ONLINE/FALLEN.",
        Style::default().fg(PARCHMENT),
    )));

    let p = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title(" HELP (Esc to close) ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(GOLD)),
        );

    f.render_widget(p, help_area);
}
