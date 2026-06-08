use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

use crate::tui::action::Action;
use crate::tui::component::Component;

pub struct PaletteComponent {
    pub is_active: bool,
    pub items: Vec<String>,
    pub state: ListState,
}

impl PaletteComponent {
    pub fn new() -> Self {
        let items = vec![
            "/sys - System Diagnostics".to_string(),
            "/vault - Knowledge Vault".to_string(),
            "/chaos - Entropic Status".to_string(),
            "/stabilize - Stabilize Attractor".to_string(),
            "/stats - Session Statistics".to_string(),
            "/clear - Reset Context".to_string(),
            "/mode - Engine Switch".to_string(),
            "/system - Show System Prompt".to_string(),
            "/quit - Shutdown GZMO".to_string(),
        ];
        Self {
            is_active: false,
            items,
            state: ListState::default(),
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

impl Component for PaletteComponent {
    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        if !self.is_active {
            return Ok(None);
        }

        if let Some(Event::Key(key)) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => {
                        self.is_active = false;
                        return Ok(Some(Action::ToggleCommandPalette));
                    }
                    KeyCode::Down => self.next(),
                    KeyCode::Up => self.previous(),
                    KeyCode::Enter => {
                        if let Some(idx) = self.state.selected() {
                            let cmd = self.items[idx]
                                .split_whitespace()
                                .next()
                                .unwrap_or("")
                                .to_string();
                            self.is_active = false;
                            return Ok(Some(Action::SubmitInput(cmd)));
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(None) // Absorb event — palette is modal
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::ToggleCommandPalette = action {
            self.is_active = !self.is_active;
            if self.is_active && self.state.selected().is_none() {
                self.state.select(Some(0));
            }
        }
        Ok(None)
    }

    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        if !self.is_active {
            return Ok(());
        }

        // Center the palette overlay
        let width = 60u16;
        let height = 15u16;
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let palette_area = Rect::new(x, y, width, height.min(area.height));

        // Clear underlying content for proper z-ordering
        f.render_widget(Clear, palette_area);

        let items_iter = self
            .items
            .iter()
            .map(|i| ListItem::new(i.as_str()).style(Style::default().fg(Color::Rgb(201, 209, 217))));

        let list = List::new(items_iter)
            .block(
                Block::default()
                    .title(" COMMAND PALETTE (ESC to close) ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(123, 44, 255))),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(123, 44, 255))
                    .fg(Color::Rgb(0, 0, 0)),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, palette_area, &mut self.state);

        Ok(())
    }
}
