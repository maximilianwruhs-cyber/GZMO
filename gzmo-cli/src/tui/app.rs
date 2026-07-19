use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures::{FutureExt, StreamExt};
use ratatui::{backend::CrosstermBackend, Frame};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io::stdout;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

use super::accessibility::AccessibilityFlags;
use super::action::Action;
use super::component::Component;
use super::components::palette::PaletteComponent;
use super::help;

pub struct AppComponents {
    pub input: Box<dyn Component>,
    pub transcript: Box<dyn Component>,
    pub status: Box<dyn Component>,
    pub instruments: Box<dyn Component>,
    pub canvas: Box<dyn Component>,
    pub agent: Box<dyn Component>,
    pub palette: PaletteComponent,
}

impl AppComponents {
    pub fn iter_mut(&mut self) -> Vec<&mut dyn Component> {
        vec![
            self.input.as_mut(),
            self.transcript.as_mut(),
            self.status.as_mut(),
            self.instruments.as_mut(),
            self.canvas.as_mut(),
            self.agent.as_mut(),
        ]
    }
}

pub struct App {
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub components: AppComponents,
    pub should_quit: bool,
    pub is_palette_open: bool,
    pub is_help_open: bool,
    pub terminal_size: (u16, u16),
    pub a11y: AccessibilityFlags,
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
                is_help_open: false,
                terminal_size: (80, 24),
                a11y: AccessibilityFlags::from_env(),
            },
            action_tx,
            action_rx,
        )
    }

    /// Draw the ops console layout onto any ratatui backend (testable).
    pub fn draw_ops_console(&mut self, f: &mut Frame<'_>) {
        let area = f.size();
        let narrow = area.width < 60;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(area);

        let _ = self.components.status.render(f, chunks[0]);

        if narrow {
            let stack = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(chunks[1]);
            let _ = self.components.transcript.render(f, stack[0]);
            let right = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
                .split(stack[1]);
            let _ = self.components.canvas.render(f, right[0]);
            let _ = self.components.instruments.render(f, right[1]);
        } else {
            let middle = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(72), Constraint::Percentage(28)])
                .split(chunks[1]);
            let _ = self.components.transcript.render(f, middle[0]);
            let right = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
                .split(middle[1]);
            let _ = self.components.canvas.render(f, right[0]);
            let _ = self.components.instruments.render(f, right[1]);
        }

        let _ = self.components.input.render(f, chunks[2]);

        if self.is_palette_open {
            let _ = self.components.palette.render(f, area);
        }
        if self.is_help_open {
            help::render_help(f, area);
        }
    }

    /// App-level action dispatch (shared by the run loop and tests).
    pub fn handle_action(
        &mut self,
        action: Action,
        action_tx: &mpsc::UnboundedSender<Action>,
    ) -> Result<()> {
        match action {
            Action::Quit => self.should_quit = true,
            Action::ToggleCommandPalette => {
                let open = !self.is_palette_open;
                self.sync_palette(open, action_tx)?;
            }
            Action::SetCommandPalette(open) => {
                self.sync_palette(open, action_tx)?;
            }
            Action::ToggleHelp => {
                let open = !self.is_help_open;
                self.sync_help(open, action_tx)?;
            }
            Action::SetHelp(open) => {
                self.sync_help(open, action_tx)?;
            }
            Action::Resize(w, h) => {
                self.terminal_size = (w, h);
            }
            Action::SubmitInput(ref text) => {
                if self.is_palette_open {
                    self.sync_palette(false, action_tx)?;
                }
                if self.is_help_open {
                    self.sync_help(false, action_tx)?;
                }
                for component in self.components.iter_mut() {
                    if let Some(sub_action) = component.update(Action::SubmitInput(text.clone()))? {
                        let _ = action_tx.send(sub_action);
                    }
                }
            }
            _ => {
                for component in self.components.iter_mut() {
                    if let Some(sub_action) = component.update(action.clone())? {
                        let _ = action_tx.send(sub_action);
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn run(
        &mut self,
        action_tx: mpsc::UnboundedSender<Action>,
        mut action_rx: mpsc::UnboundedReceiver<Action>,
    ) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            stdout(),
            crossterm::terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        )?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        terminal.clear()?;
        if let Some(size) = terminal.size().ok() {
            self.terminal_size = (size.width, size.height);
        }

        for component in self.components.iter_mut() {
            component.init(action_tx.clone())?;
        }
        self.components.palette.init(action_tx.clone())?;

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
                        if let Event::Resize(w, h) = event {
                            let _ = action_tx.send(Action::Resize(w, h));
                        }
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
                                if key.code == KeyCode::Char('?') {
                                    let _ = action_tx.send(Action::ToggleHelp);
                                }
                                if key.code == KeyCode::Esc {
                                    if self.is_palette_open {
                                        let _ = action_tx.send(Action::SetCommandPalette(false));
                                    } else if self.is_help_open {
                                        let _ = action_tx.send(Action::SetHelp(false));
                                    }
                                }
                            }
                        }

                        if self.is_palette_open {
                            if let Some(action) =
                                self.components.palette.handle_events(Some(event.clone()))?
                            {
                                let _ = action_tx.send(action);
                            }
                        } else if !self.is_help_open {
                            let base_comps: Vec<&mut dyn Component> = vec![
                                self.components.input.as_mut(),
                                self.components.transcript.as_mut(),
                                self.components.status.as_mut(),
                                self.components.instruments.as_mut(),
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
                    if matches!(action, Action::Render) {
                        terminal.draw(|f| self.draw_ops_console(f))?;
                    } else {
                        self.handle_action(action, &action_tx)?;
                    }
                }
            }
        }

        crossterm::execute!(
            stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        )?;
        crossterm::terminal::disable_raw_mode()?;

        Ok(())
    }

    fn sync_palette(
        &mut self,
        open: bool,
        action_tx: &mpsc::UnboundedSender<Action>,
    ) -> Result<()> {
        self.is_palette_open = open;
        if open {
            self.is_help_open = false;
        }
        self.components
            .palette
            .update(Action::SetCommandPalette(open))?;
        for component in self.components.iter_mut() {
            if let Some(sub_action) = component.update(Action::SetCommandPalette(open))? {
                let _ = action_tx.send(sub_action);
            }
        }
        Ok(())
    }

    fn sync_help(&mut self, open: bool, action_tx: &mpsc::UnboundedSender<Action>) -> Result<()> {
        self.is_help_open = open;
        if open {
            self.is_palette_open = false;
            self.components
                .palette
                .update(Action::SetCommandPalette(false))?;
        }
        let _ = action_tx;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::components::{
        agent::AgentComponent, chaos_canvas::ChaosCanvasComponent, input::InputComponent,
        instruments::InstrumentsComponent, palette::PaletteComponent,
        status_bar::StatusBarComponent, transcript::TranscriptComponent,
    };
    use gzmo_chaos::pulse::{ChaosConfig, PulseLoop};
    use gzmo_core::agent_session::AgentSession;
    use gzmo_core::config::GzmoConfig;
    use gzmo_core::session::SessionManager;
    use gzmo_core::tools::ToolRegistry;
    use ratatui::backend::TestBackend;
    use std::sync::Arc;

    async fn test_app() -> (
        App,
        mpsc::UnboundedSender<Action>,
        mpsc::UnboundedReceiver<Action>,
    ) {
        let config = GzmoConfig::default();
        let (snap_tx, snap_rx) = tokio::sync::watch::channel(Default::default());
        let _ = snap_tx;
        let active = config.engine.active_engine();
        let gateway: Arc<tokio::sync::RwLock<Arc<dyn gzmo_core::gateway::LlmGateway>>> = Arc::new(
            tokio::sync::RwLock::new(Arc::new(gzmo_core::gateway::TurboQuantGateway::new(
                gzmo_core::gateway::VllmConfig::from(active),
            ))),
        );
        let config_arc = Arc::new(tokio::sync::RwLock::new(config.clone()));
        let soul = Arc::new(tokio::sync::RwLock::new(gzmo_core::types::SoulContext {
            raw_markdown: "test".into(),
            persona_name: "GZMO".into(),
            core_directives: vec![],
            ethical_guardrails: vec![],
            loaded_at: chrono::Utc::now(),
        }));
        let episodic = Arc::new(gzmo_core::memory::episodic::FileEpisodicStore::new(
            "memory",
        ));
        let session_mgr = Arc::new(SessionManager::new("memory/sessions"));
        let tools = Arc::new(ToolRegistry::new());
        let chaos_skills = Arc::new(gzmo_core::skills::SkillRegistry::new());
        let (feedback_tx, _) = tokio::sync::mpsc::channel(1);
        let agent_session = Arc::new(tokio::sync::Mutex::new(
            AgentSession::new_main(
                &config.redis,
                &config.context_memory,
                SessionManager::new_session_id(),
            )
            .await,
        ));
        let pedagogy = Arc::new(tokio::sync::Mutex::new(
            crate::pedagogy_bridge::PedagogyRuntime::boot(&config)
                .await
                .expect("pedagogy boot"),
        ));
        let router = Arc::new(gzmo_core::gateway::GatewayRouter::new(&config));

        let comps = AppComponents {
            input: Box::new(InputComponent::new()),
            transcript: Box::new(TranscriptComponent::new()),
            status: Box::new(StatusBarComponent::new("LOCAL", "test-model")),
            instruments: Box::new(InstrumentsComponent::new(None)),
            canvas: Box::new(ChaosCanvasComponent::new()),
            agent: Box::new(AgentComponent::new(
                gateway,
                tools,
                "system".into(),
                4,
                8192,
                soul,
                None,
                episodic,
                session_mgr,
                agent_session,
                snap_rx,
                chaos_skills,
                feedback_tx,
                config_arc,
                std::path::PathBuf::from("gzmo.toml"),
                None,
                false,
                Arc::new(gzmo_core::workflow_skills::WorkflowSkillIndex::empty()),
                Arc::new(std::sync::Mutex::new(
                    gzmo_core::workflow_skills::WorkflowSessionState::default(),
                )),
                pedagogy,
                router,
            )),
            palette: PaletteComponent::new(),
        };
        let (mut app, tx, rx) = App::new(comps);
        for component in app.components.iter_mut() {
            component.init(tx.clone()).unwrap();
        }
        app.components.palette.init(tx.clone()).unwrap();
        (app, tx, rx)
    }

    #[tokio::test]
    async fn palette_toggle_keeps_app_and_component_in_sync() {
        let (mut app, tx, _rx) = test_app().await;
        assert!(!app.is_palette_open);
        assert!(!app.components.palette.is_active);

        app.handle_action(Action::ToggleCommandPalette, &tx)
            .unwrap();
        assert!(app.is_palette_open);
        assert!(app.components.palette.is_active);

        app.handle_action(Action::SetCommandPalette(true), &tx)
            .unwrap();
        assert!(app.is_palette_open);
        assert!(app.components.palette.is_active);

        app.handle_action(Action::SetCommandPalette(false), &tx)
            .unwrap();
        assert!(!app.is_palette_open);
        assert!(!app.components.palette.is_active);
    }

    fn render_buffer(app: &mut App, w: u16, h: u16) -> String {
        let backend = TestBackend::new(w, h);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| app.draw_ops_console(f)).unwrap();
        format!("{}", terminal.backend())
    }

    #[tokio::test]
    async fn palette_enter_closes_before_submit() {
        let (mut app, tx, _rx) = test_app().await;
        app.handle_action(Action::SetCommandPalette(true), &tx)
            .unwrap();
        app.handle_action(Action::SubmitInput("/stats".into()), &tx)
            .unwrap();
        assert!(!app.is_palette_open);
        assert!(!app.components.palette.is_active);
    }

    #[tokio::test]
    async fn golden_palette_open() {
        let (mut app, tx, _rx) = test_app().await;
        app.handle_action(Action::SetCommandPalette(true), &tx)
            .unwrap();
        let buf = render_buffer(&mut app, 100, 30);
        assert!(buf.contains("COMMAND PALETTE"));
    }

    #[tokio::test]
    async fn golden_warming_attractor() {
        let (mut app, _tx, _rx) = test_app().await;
        let buf = render_buffer(&mut app, 100, 30);
        assert!(buf.contains("warming attractor") || buf.contains("LORENZ"));
    }

    #[tokio::test]
    async fn golden_streaming_chrome() {
        let (mut app, tx, _rx) = test_app().await;
        app.handle_action(Action::AgentTokenStream("hello".into()), &tx)
            .unwrap();
        let buf = render_buffer(&mut app, 100, 30);
        assert!(buf.contains("STREAM") || buf.contains("cogitating"));
    }

    #[tokio::test]
    async fn golden_narrow_layout() {
        let (mut app, _tx, _rx) = test_app().await;
        let buf = render_buffer(&mut app, 48, 24);
        assert!(buf.contains("TRANSCRIPT"));
        assert!(buf.contains("you >"));
    }

    #[tokio::test]
    async fn bridge_immediate_flush_reaches_canvas() {
        let handle = PulseLoop::start(ChaosConfig::default());
        let gateway: Arc<tokio::sync::RwLock<Arc<dyn gzmo_core::gateway::LlmGateway>>> = Arc::new(
            tokio::sync::RwLock::new(Arc::new(gzmo_core::gateway::TurboQuantGateway::new(
                gzmo_core::gateway::VllmConfig::from(GzmoConfig::default().engine.active_engine()),
            ))),
        );
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();
        let (feedback_tx, _feedback_rx) = tokio::sync::mpsc::channel(4);
        let state_dir =
            std::env::temp_dir().join(format!("gzmo-bridge-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&state_dir);

        let _bridge = crate::chaos_bootstrap::spawn_snapshot_bridge(
            handle.snapshot_rx.clone(),
            gateway,
            feedback_tx,
            state_dir.clone(),
            None,
            Some(action_tx),
            None,
            gzmo_core::synapse::EventSource::GzmoCli,
            "test".into(),
            true,
            crate::chaos_bootstrap::SnapshotBridgeOpts::TUI,
        );

        let got = tokio::time::timeout(Duration::from_secs(2), action_rx.recv())
            .await
            .expect("timeout")
            .expect("channel closed");
        assert!(matches!(got, Action::ChaosSnapshot(_)));

        let _ = std::fs::remove_dir_all(state_dir);
    }

    #[tokio::test]
    async fn golden_stale_pulse_label() {
        let (mut app, tx, _rx) = test_app().await;
        app.handle_action(
            Action::ChaosSnapshot(gzmo_chaos::pulse::ChaosSnapshot::default()),
            &tx,
        )
        .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(
            (crate::tui::theme::PULSE_STALE_SECS * 1000 + 100) as u64,
        ))
        .await;
        let buf = render_buffer(&mut app, 100, 30);
        assert!(buf.contains("PULSE STALE"));
    }
}
