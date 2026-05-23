use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{block::{Block, Position, Title}, Borders, Paragraph},
    Frame,
};
use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::tui::action::Action;
use crate::tui::component::Component;

pub struct InputComponent {
    pub input: Input,
    pub action_tx: Option<UnboundedSender<Action>>,
}

impl InputComponent {
    pub fn new() -> Self {
        Self {
            input: Input::default(),
            action_tx: None,
        }
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

    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);

        let p = Paragraph::new(self.input.value())
            .style(Style::default().fg(Color::Rgb(201, 209, 217)))
            .scroll((0, scroll as u16))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" ★ you › ")
                    .title_style(Style::default().fg(Color::Rgb(212, 175, 55)))
                    .title(
                        Title::from(Line::from(vec![
                            Span::styled(" [Enter] ", Style::default().fg(Color::Rgb(100, 100, 100))),
                            Span::raw("Send  "),
                            Span::styled(" [Ctrl+P] ", Style::default().fg(Color::Rgb(100, 100, 100))),
                            Span::raw("Palette "),
                        ]))
                        .alignment(Alignment::Right)
                        .position(Position::Bottom),
                    )
                    .border_style(Style::default().fg(Color::Rgb(60, 60, 70))),
            );

        f.render_widget(p, area);

        // Show cursor
        let cursor_x = area.x + 1 + (self.input.visual_cursor().max(scroll) - scroll) as u16;
        f.set_cursor(cursor_x, area.y + 1);

        Ok(())
    }
}
