use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind, MouseEventKind};
use gzmo_chaos::chaos::Phase;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::collections::VecDeque;

use crate::tui::action::Action;
use crate::tui::component::Component;
use crate::tui::theme::{self, COPPER, CYAN, GOLD, MUTED, PARCHMENT, STEEL, TEXT};

#[derive(Debug, Clone)]
pub enum MessageType {
    User,
    Agent,
    System,
    Lore,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub message_type: MessageType,
    pub content: String,
}

pub struct TranscriptComponent {
    pub messages: VecDeque<ChatMessage>,
    pub scroll_offset: u16,
    pub active_stream: String,
    tension: f64,
    phase: Phase,
    speaking: bool,
}

impl TranscriptComponent {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::with_capacity(100),
            scroll_offset: 0,
            active_stream: String::new(),
            tension: 0.0,
            phase: Phase::Idle,
            speaking: false,
        }
    }
}

impl Component for TranscriptComponent {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::SubmitInput(text) => {
                // Drop any leftover partial stream from a previous turn.
                self.active_stream.clear();
                self.speaking = false;
                self.messages.push_back(ChatMessage {
                    message_type: MessageType::User,
                    content: text,
                });
                self.scroll_offset = 0;
            }
            Action::AgentTokenStream(token) => {
                self.active_stream.push_str(&token);
                self.scroll_offset = 0;
                self.speaking = true;
            }
            Action::AgentResponse(full_text) => {
                // Prefer the completed stream when the final payload is empty
                // (some gateway paths stream tokens but return "" as the text body).
                let content = if full_text.trim().is_empty() && !self.active_stream.is_empty() {
                    std::mem::take(&mut self.active_stream)
                } else {
                    self.active_stream.clear();
                    full_text
                };
                self.speaking = false;
                if content.trim().is_empty() {
                    // Avoid pushing a blank "gzmo >" row that looks like a hang.
                    self.scroll_offset = 0;
                    return Ok(None);
                }
                self.messages.push_back(ChatMessage {
                    message_type: MessageType::Agent,
                    content,
                });
                self.scroll_offset = 0;
            }
            Action::ChaosSnapshot(snap) => {
                self.tension = snap.tension;
                self.phase = snap.phase;
            }
            Action::LoreEvent(category, author, text) => {
                let content = if author.is_empty() {
                    format!("[{}] {}", category, text)
                } else {
                    format!("[{}] \"{}\" — {}", category, text, author)
                };
                self.messages.push_back(ChatMessage {
                    message_type: MessageType::Lore,
                    content,
                });
                self.scroll_offset = 0;
            }
            Action::TriggerNotification(msg) => {
                self.messages.push_back(ChatMessage {
                    message_type: MessageType::System,
                    content: msg,
                });
                self.scroll_offset = 0;
            }
            Action::TranscriptClear => {
                self.messages.clear();
                self.scroll_offset = 0;
            }
            Action::TranscriptRestore(msgs) => {
                self.messages.clear();
                for m in msgs {
                    if m.is_meta {
                        continue;
                    }
                    let m_type = match m.role {
                        gzmo_core::types::Role::User => MessageType::User,
                        gzmo_core::types::Role::System
                        | gzmo_core::types::Role::Assistant
                        | gzmo_core::types::Role::Tool => MessageType::Agent,
                    };
                    self.messages.push_back(ChatMessage {
                        message_type: m_type,
                        content: m.content,
                    });
                }
                self.scroll_offset = 0;
            }
            _ => {}
        }

        // Keep buffer contained
        if self.messages.len() > 100 {
            self.messages.pop_front();
        }

        Ok(None)
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        if let Some(event) = event {
            match event {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::PageUp => {
                                self.scroll_offset = self.scroll_offset.saturating_add(5);
                            }
                            KeyCode::PageDown => {
                                self.scroll_offset = self.scroll_offset.saturating_sub(5);
                            }
                            _ => {}
                        }
                    }
                }
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        self.scroll_offset = self.scroll_offset.saturating_add(3);
                    }
                    MouseEventKind::ScrollDown => {
                        self.scroll_offset = self.scroll_offset.saturating_sub(3);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(None)
    }

    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let inner_width = area.width.saturating_sub(2).max(1);
        let inner_height = area.height.saturating_sub(2);
        let lines = self.build_lines();

        let text_height = wrapped_line_count(&lines, inner_width);
        let max_scroll = text_height.saturating_sub(inner_height);
        let actual_scroll = max_scroll.saturating_sub(self.scroll_offset);
        let drop = matches!(self.phase, Phase::Drop);
        let border = theme::chrome_border(self.tension, self.speaking, drop);

        let p = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((actual_scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" TRANSCRIPT ")
                    .title_style(Style::default().fg(COPPER).add_modifier(Modifier::BOLD))
                    .border_style(Style::default().fg(border)),
            );

        f.render_widget(p, area);
        Ok(())
    }
}

impl TranscriptComponent {
    /// Build display lines: one logical row per content newline so Paragraph
    /// wrap + scroll math match what the terminal paints.
    fn build_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        for msg in &self.messages {
            // ASCII prefixes only — emoji (⚙/★/⚡) have ambiguous cell width and
            // produce corruption like "Conxtcleared" on narrow terminals.
            let (prefix, style) = match msg.message_type {
                MessageType::User => (
                    " * you > ",
                    Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
                ),
                MessageType::Agent => (
                    " * gzmo > ",
                    Style::default().fg(COPPER).add_modifier(Modifier::BOLD),
                ),
                MessageType::System => (" ! sys > ", Style::default().fg(CYAN)),
                MessageType::Lore => (" ~ lore > ", Style::default().fg(STEEL)),
            };

            let body_style = match msg.message_type {
                MessageType::User => Style::default().fg(TEXT),
                MessageType::Agent => Style::default().fg(PARCHMENT),
                MessageType::System => Style::default().fg(CYAN),
                MessageType::Lore => Style::default().fg(MUTED),
            };
            let body = strip_redundant_bullet(sanitize_tui_text(&msg.content));
            push_wrapped_message(&mut lines, prefix, style, &body, body_style);
        }

        if !self.active_stream.is_empty() {
            let stream = strip_redundant_bullet(sanitize_tui_text(&self.active_stream));
            let mut stream_lines = Vec::new();
            push_wrapped_message(
                &mut stream_lines,
                " * gzmo > ",
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
                &stream,
                Style::default().fg(PARCHMENT),
            );
            if let Some(last) = stream_lines.last_mut() {
                last.spans.push(Span::styled(
                    "#",
                    Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
                ));
            }
            lines.extend(stream_lines);
        }

        lines
    }
}

fn push_wrapped_message(
    lines: &mut Vec<Line<'static>>,
    prefix: &str,
    prefix_style: Style,
    body: &str,
    body_style: Style,
) {
    if body.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            prefix.to_string(),
            prefix_style,
        )]));
        return;
    }
    let mut first = true;
    for part in body.split('\n') {
        if first {
            lines.push(Line::from(vec![
                Span::styled(prefix.to_string(), prefix_style),
                Span::styled(part.to_string(), body_style),
            ]));
            first = false;
        } else {
            // Continuation indent matches prefix width (" * gzmo > " = 10 cells).
            lines.push(Line::from(vec![
                Span::styled("          ".to_string(), prefix_style),
                Span::styled(part.to_string(), body_style),
            ]));
        }
    }
}

fn wrapped_line_count(lines: &[Line<'_>], inner_width: u16) -> u16 {
    let w = inner_width.max(1) as usize;
    lines
        .iter()
        .map(|l| {
            let width = l.width().max(1);
            // ceil(width / inner_width)
            ((width + w - 1) / w) as u16
        })
        .sum()
}

/// Replace ambiguous-width symbols so ratatui column math matches the terminal.
fn sanitize_tui_text(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '⚙' | '★' | '✦' | '✧' => '*',
            '⚡' | '⚠' => '!',
            '∰' | '∮' => '~',
            '›' | '‹' => '>',
            '—' | '–' => '-',
            '●' => 'o',
            other => other,
        })
        .collect()
}

/// Agent slash replies still start with "⚙ …"; prefix already has `* gzmo >`.
fn strip_redundant_bullet(s: String) -> String {
    let trimmed = s.trim_start();
    if let Some(rest) = trimmed.strip_prefix("* ") {
        rest.to_string()
    } else if let Some(rest) = trimmed.strip_prefix("! ") {
        rest.to_string()
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::component::Component;

    #[test]
    fn sanitize_keeps_context_cleared_readable() {
        let s = sanitize_tui_text("⚙ Context cleared — new session.");
        assert_eq!(s, "* Context cleared - new session.");
        assert!(!s.contains("Conxt"));
        assert_eq!(strip_redundant_bullet(s), "Context cleared - new session.");
    }

    #[test]
    fn empty_agent_response_keeps_streamed_tokens() {
        let mut t = TranscriptComponent::new();
        t.update(Action::AgentTokenStream("hello ".into())).unwrap();
        t.update(Action::AgentTokenStream("world".into())).unwrap();
        t.update(Action::AgentResponse(String::new())).unwrap();
        assert_eq!(t.messages.len(), 1);
        assert_eq!(t.messages[0].content, "hello world");
        assert!(t.active_stream.is_empty());
    }

    #[test]
    fn blank_agent_response_without_stream_is_dropped() {
        let mut t = TranscriptComponent::new();
        t.update(Action::AgentResponse("   ".into())).unwrap();
        assert!(t.messages.is_empty());
    }

    #[test]
    fn multiline_status_splits_into_continuation_lines() {
        let mut t = TranscriptComponent::new();
        t.update(Action::AgentResponse("line1\nline2\nline3".into()))
            .unwrap();
        let lines = t.build_lines();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].spans[0].content.contains("gzmo"));
        assert_eq!(lines[1].spans[1].content, "line2");
        assert_eq!(lines[2].spans[1].content, "line3");
    }

    #[test]
    fn wrap_count_uses_inner_width_not_outer() {
        // A 40-cell line in a 20-cell inner width needs 2 rows, not 1.
        let line = Line::from("abcdefghijklmnopqrstuvwxyz0123456789abcd");
        assert_eq!(line.width(), 40);
        assert_eq!(wrapped_line_count(&[line], 20), 2);
        // Using outer width 22 (20+2 borders) would under-count — that's the bug.
        assert_eq!(
            wrapped_line_count(
                &[Line::from("abcdefghijklmnopqrstuvwxyz0123456789abcd")],
                22
            ),
            2
        );
    }
}
