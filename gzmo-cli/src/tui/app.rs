use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures::{FutureExt, StreamExt};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::stdout;
use tokio::sync::mpsc;
use tokio::time::interval;
use std::time::Duration;

use super::action::Action;
use super::component::Component;

pub struct AppComponents {
    pub input: Box<dyn Component>,
    pub transcript: Box<dyn Component>,
    pub status: Box<dyn Component>,
    pub canvas: Box<dyn Component>,
    pub agent: Box<dyn Component>,
    pub palette: Box<dyn Component>,
}

impl AppComponents {
    pub fn iter_mut(&mut self) -> Vec<&mut dyn Component> {
        vec![
            self.input.as_mut(),
            self.transcript.as_mut(),
            self.status.as_mut(),
            self.canvas.as_mut(),
            self.agent.as_mut(),
            self.palette.as_mut(),
        ]
    }
}

pub struct App {
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub components: AppComponents,
    pub should_quit: bool,
    pub is_palette_open: bool,
}

impl App {
    pub fn new(
        components: AppComponents,
    ) -> (
        Self,
        mpsc::UnboundedSender<Action>,
        mpsc::UnboundedReceiver<Action>,
    ) {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        (
            Self {
                tick_rate: 60.0,
                frame_rate: 60.0,
                components,
                should_quit: false,
                is_palette_open: false,
            },
            action_tx,
            action_rx,
        )
    }

    pub async fn run(
        &mut self,
        action_tx: mpsc::UnboundedSender<Action>,
        mut action_rx: mpsc::UnboundedReceiver<Action>,
    ) -> Result<()> {
        // Setup Terminal
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            stdout(),
            crossterm::terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        )?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        terminal.clear()?;

        // Init components
        for component in self.components.iter_mut() {
            component.init(action_tx.clone())?;
        }

        let mut event_stream = EventStream::new();
        let mut tick_interval = interval(Duration::from_secs_f64(1.0 / self.tick_rate));
        let mut render_interval = interval(Duration::from_secs_f64(1.0 / self.frame_rate));

        loop {
            if self.should_quit {
                break;
            }

            tokio::select! {
                _ = tick_interval.tick() => {
                    let _ = action_tx.send(Action::Tick);
                },
                _ = render_interval.tick() => {
                    let _ = action_tx.send(Action::Render);
                },
                maybe_event = event_stream.next().fuse() => {
                    if let Some(Ok(event)) = maybe_event {
                        // Global hotkeys
                        if let Event::Key(key) = event {
                            if key.kind == KeyEventKind::Press {
                                if key.code == KeyCode::Char('c')
                                    && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                                {
                                    let _ = action_tx.send(Action::Quit);
                                }
                                if key.code == KeyCode::Char('p')
                                    && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                                {
                                    let _ = action_tx.send(Action::ToggleCommandPalette);
                                }
                            }
                        }

                        // Route raw event
                        if self.is_palette_open {
                            if let Some(action) =
                                self.components.palette.handle_events(Some(event.clone()))?
                            {
                                let _ = action_tx.send(action);
                            }
                        } else {
                            // Normal mode: broadcast to all standard components
                            let base_comps: Vec<&mut dyn Component> = vec![
                                self.components.input.as_mut(),
                                self.components.transcript.as_mut(),
                                self.components.status.as_mut(),
                                self.components.canvas.as_mut(),
                                self.components.agent.as_mut(),
                            ];
                            for component in base_comps {
                                if let Some(action) =
                                    component.handle_events(Some(event.clone()))?
                                {
                                    let _ = action_tx.send(action);
                                }
                            }
                        }
                    }
                },
                Some(action) = action_rx.recv() => {
                    match action {
                        Action::Quit => self.should_quit = true,
                        Action::ToggleCommandPalette => {
                            self.is_palette_open = !self.is_palette_open;
                        }
                        Action::Render => {
                            terminal.draw(|f| {
                                use ratatui::layout::{Constraint, Direction, Layout};

                                let chunks = Layout::default()
                                    .direction(Direction::Vertical)
                                    .constraints([
                                        Constraint::Length(3), // status bar
                                        Constraint::Min(1),    // transcript + canvas
                                        Constraint::Length(3), // input box
                                    ])
                                    .split(f.size());

                                let _ = self.components.status.render(f, chunks[0]);

                                // Split middle horizontally: 70% transcript, 30% canvas
                                let middle_chunks = Layout::default()
                                    .direction(Direction::Horizontal)
                                    .constraints([
                                        Constraint::Percentage(70),
                                        Constraint::Percentage(30),
                                    ])
                                    .split(chunks[1]);

                                let _ = self.components.transcript.render(f, middle_chunks[0]);
                                let _ = self.components.canvas.render(f, middle_chunks[1]);
                                let _ = self.components.input.render(f, chunks[2]);

                                // Palette renders on top of everything (z-index highest)
                                if self.is_palette_open {
                                    let _ = self.components.palette.render(f, f.size());
                                }
                            })?;
                        }
                        _ => {
                            // Dispatch action to all components
                            for component in self.components.iter_mut() {
                                if let Some(sub_action) = component.update(action.clone())? {
                                    let _ = action_tx.send(sub_action);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Cleanup
        crossterm::execute!(
            stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        )?;
        crossterm::terminal::disable_raw_mode()?;

        Ok(())
    }
}
