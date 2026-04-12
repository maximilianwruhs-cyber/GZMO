use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use std::collections::VecDeque;

use crate::tui::action::Action;
use crate::tui::component::Component;

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
}

impl TranscriptComponent {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::with_capacity(100),
            scroll_offset: 0,
            active_stream: String::new(),
        }
    }
}

impl Component for TranscriptComponent {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::SubmitInput(text) => {
                self.messages.push_back(ChatMessage {
                    message_type: MessageType::User,
                    content: text,
                });
                self.scroll_offset = 0;
            }
            Action::AgentTokenStream(token) => {
                self.active_stream.push_str(&token);
                self.scroll_offset = 0;
            }
            Action::AgentResponse(full_text) => {
                self.active_stream.clear();
                self.messages.push_back(ChatMessage {
                    message_type: MessageType::Agent,
                    content: full_text,
                });
                self.scroll_offset = 0;
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
                    if m.is_meta { continue; }
                    let m_type = match m.role {
                        gzmo_core::types::Role::User => MessageType::User,
                        gzmo_core::types::Role::System | gzmo_core::types::Role::Assistant | gzmo_core::types::Role::Tool => MessageType::Agent,
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
        if let Some(Event::Key(key)) = event {
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
        Ok(None)
    }

    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let mut lines = Vec::new();

        for msg in &self.messages {
            let (prefix, style) = match msg.message_type {
                MessageType::User => (
                    " ★ YOU › ",
                    Style::default()
                        .fg(Color::Rgb(201, 209, 217))
                        .add_modifier(Modifier::BOLD),
                ),
                MessageType::Agent => (
                    " ⚙ GZMO › ",
                    Style::default().fg(Color::Rgb(180, 130, 255)),
                ),
                MessageType::System => (
                    " ⚡ SYS › ",
                    Style::default().fg(Color::Rgb(0, 245, 255)),
                ),
                MessageType::Lore => (
                    " 📡 LORE › ",
                    Style::default().fg(Color::Rgb(100, 130, 140)),
                ),
            };

            // Split long lines for readability
            let body_style = match msg.message_type {
                MessageType::User => Style::default().fg(Color::Rgb(220, 225, 230)),
                MessageType::Agent => Style::default().fg(Color::Rgb(210, 200, 240)),
                MessageType::System => Style::default().fg(Color::Rgb(180, 230, 240)),
                MessageType::Lore => Style::default().fg(Color::Rgb(130, 150, 155)),
            };
            lines.push(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(&msg.content, body_style),
            ]));
        }

        // Render active stream if exists
        if !self.active_stream.is_empty() {
            lines.push(Line::from(vec![
                Span::styled(
                    " ⚙ GZMO › ",
                    Style::default().fg(Color::Rgb(123, 44, 255)),
                ),
                Span::raw(&self.active_stream),
                Span::styled(
                    "█",
                    Style::default()
                        .fg(Color::Rgb(123, 44, 255))
                        .add_modifier(Modifier::RAPID_BLINK),
                ),
            ]));
        }

        let text_height = lines
            .iter()
            .map(|l| (l.width() as u16 / area.width.max(1)) + 1)
            .sum::<u16>();
        let max_scroll = if text_height > area.height.saturating_sub(2) {
            text_height - area.height.saturating_sub(2)
        } else {
            0
        };

        let actual_scroll = max_scroll.saturating_sub(self.scroll_offset);

        let p = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((actual_scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(60, 60, 70))),
            );

        f.render_widget(p, area);
        Ok(())
    }
}
