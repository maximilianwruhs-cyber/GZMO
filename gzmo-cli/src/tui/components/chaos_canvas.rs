use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{
        canvas::Canvas,
        Block, Borders,
    },
    Frame,
};
use color_eyre::Result;
use std::collections::VecDeque;

use crate::tui::action::Action;
use crate::tui::component::Component;

pub struct ChaosCanvasComponent {
    /// Historical Lorenz attractor points for "tail" trail effect.
    pub history: VecDeque<(f64, f64, f64)>,
    pub max_history: usize,
    pub active_tension: f64,
}

impl ChaosCanvasComponent {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(200),
            max_history: 150,
            active_tension: 0.0,
        }
    }
}

impl Component for ChaosCanvasComponent {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::ChaosSnapshot(snap) = action {
            self.history
                .push_back((snap.x as f64, snap.y as f64, snap.z as f64));
            self.active_tension = snap.tension as f64;

            if self.history.len() > self.max_history {
                self.history.pop_front();
            }
        }
        Ok(None)
    }

    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" ∰ LORENZ ATTRACTOR ")
            .title_style(Style::default().fg(Color::Rgb(0, 200, 210)))
            .border_style(Style::default().fg(Color::Rgb(60, 60, 70)));

        // Tension-reactive color: Cyan at normal ↔ Ritual Red above 80%
        let base_rgb = if self.active_tension > 80.0 {
            (137.0, 0.0, 0.0)
        } else {
            (0.0, 245.0, 255.0)
        };

        let canvas = Canvas::default()
            .block(block)
            .x_bounds([-25.0, 25.0])
            .y_bounds([-25.0, 25.0])
            .marker(ratatui::symbols::Marker::Braille)
            .paint(|ctx| {
                let len = self.history.len();
                if len == 0 {
                    return;
                }
                for (i, &(x, y, z)) in self.history.iter().enumerate() {
                    // Age-based fade: older points dim out
                    let age_factor = i as f64 / len as f64;
                    // Z-depth brightness: higher Z = brighter (max ~50)
                    let z_factor = z.min(50.0).max(0.0) / 50.0;
                    let brightness = age_factor * (0.4 + (0.6 * z_factor));

                    let color = Color::Rgb(
                        (base_rgb.0 * brightness) as u8,
                        (base_rgb.1 * brightness) as u8,
                        (base_rgb.2 * brightness) as u8,
                    );

                    ctx.draw(&Point { x, y, color });
                }
            });

        f.render_widget(canvas, area);
        Ok(())
    }
}

/// Custom point struct implementing `ratatui::widgets::canvas::Shape`
/// for per-point dynamic coloring.
struct Point {
    x: f64,
    y: f64,
    color: Color,
}

impl ratatui::widgets::canvas::Shape for Point {
    fn draw(&self, painter: &mut ratatui::widgets::canvas::Painter) {
        if let Some((x, y)) = painter.get_point(self.x, self.y) {
            painter.paint(x, y, self.color);
        }
    }
}
