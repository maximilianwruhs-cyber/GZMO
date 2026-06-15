use color_eyre::Result;
use crossterm::event::Event;
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

use super::action::Action;

/// Universal component trait — Elm Architecture (TEA) pattern.
pub trait Component {
    /// Initialize the component, providing it with a channel to dispatch actions.
    #[allow(unused_variables)]
    fn init(&mut self, action_tx: UnboundedSender<Action>) -> Result<()> {
        Ok(())
    }

    /// Read raw input events (keyboard, mouse) and optionally emit an Action.
    #[allow(unused_variables)]
    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        Ok(None)
    }

    /// React to internal dispatched Actions state changes.
    #[allow(unused_variables)]
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }

    /// Render the component onto the frame.
    #[allow(unused_variables)]
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        Ok(())
    }
}
