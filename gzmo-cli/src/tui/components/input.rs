use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::tui::accessibility::AccessibilityFlags;
use crate::tui::action::Action;
use crate::tui::component::Component;
use crate::tui::theme::{self, GOLD, MUTED, TEXT};

pub struct InputComponent {
    pub input: Input,
    pub action_tx: Option<UnboundedSender<Action>>,
    speaking: bool,
    speak_pulse: u8,
    tension: f64,
    drop: bool,
}

impl InputComponent {
    pub fn new() -> Self {
        Self {
            input: Input::default(),
            action_tx: None,
            speaking: false,
            speak_pulse: 0,
            tension: 0.0,
            drop: false,
        }
    }
}

impl Default for InputComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for InputComponent {
    fn init(&mut self, action_tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(action_tx);
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        if let Some(Event::Key(key)) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter => {
                        let submission = self.input.value().to_string();
                        // Empty Enter used to fire a full LLM turn (STREAM with no user text).
                        if submission.trim().is_empty() {
                            return Ok(None);
                        }
                        self.input.reset();
                        return Ok(Some(Action::SubmitInput(submission)));
                    }
                    _ => {
                        self.input.handle_event(&Event::Key(key));
                    }
                }
            }
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::AgentTokenStream(_) => {
                self.speaking = true;
            }
            Action::AgentResponse(_) => {
                self.speaking = false;
            }
            Action::ChaosSnapshot(snap) => {
                self.tension = snap.tension;
                self.drop = matches!(snap.phase, gzmo_chaos::chaos::Phase::Drop);
            }
            Action::Tick => {
                if self.speaking {
                    self.speak_pulse = self.speak_pulse.wrapping_add(1);
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);

        let (text, style) = if self.input.value().is_empty() {
            (
                "command the engine…  Ctrl+P palette · ? help · Ctrl+C quit".to_string(),
                Style::default().fg(MUTED),
            )
        } else {
            (
                self.input.value().to_string(),
                Style::default().fg(TEXT),
            )
        };

        let a11y = AccessibilityFlags::from_env();
        let mut border = theme::chrome_border_with_flags(self.tension, self.speaking, self.drop, &a11y);
        if self.speaking && self.speak_pulse.is_multiple_of(2) && !a11y.reduced_motion {
            border = GOLD;
        }

        let title = if self.speaking {
            " * you >  · cogitating "
        } else {
            " * you > "
        };

        let p = Paragraph::new(text)
            .style(style)
            .scroll((0, scroll as u16))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .title_style(Style::default().fg(GOLD).add_modifier(Modifier::BOLD))
                    .border_style(Style::default().fg(border)),
            );

        f.render_widget(p, area);

        let cursor_x = area.x + 1 + (self.input.visual_cursor().max(scroll) - scroll) as u16;
        f.set_cursor(cursor_x, area.y + 1);

        Ok(())
    }
}
