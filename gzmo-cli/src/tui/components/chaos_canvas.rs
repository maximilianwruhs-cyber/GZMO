//! Lorenz attractor instrument — braille trail with auto-fit bounds.

use color_eyre::Result;
use gzmo_chaos::chaos::Phase;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Painter, Shape},
        Block, Borders, Paragraph,
    },
    Frame,
};
use std::collections::VecDeque;

use crate::tui::action::Action;
use crate::tui::component::Component;
use crate::tui::theme::{self, CYAN, GOLD, MUTED, PARCHMENT, STEEL};

/// Samples kept for orbit length (~10× the original 150).
const MAX_HISTORY: usize = 1500;
/// Cap paint ops so a filled butterfly cannot freeze the terminal.
const MAX_DRAW_POINTS: usize = 320;
/// Extra inset so the dense trail never sits on the widget chrome.
const BOUNDS_PAD_FRAC: f64 = 0.22;

pub struct ChaosCanvasComponent {
    pub history: VecDeque<(f64, f64, f64)>,
    pub max_history: usize,
    pub active_tension: f64,
    pub phase: Phase,
    pub speaking: bool,
    pub last_xyz: (f64, f64, f64),
    pub tick: u64,
    pub live: bool,
}

impl ChaosCanvasComponent {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(MAX_HISTORY + 100),
            max_history: MAX_HISTORY,
            active_tension: 0.0,
            phase: Phase::Idle,
            speaking: false,
            last_xyz: (0.0, 0.0, 0.0),
            tick: 0,
            live: false,
        }
    }

    /// Axis-aligned bounds that fit the trail (with padding). Falls back to classic Lorenz box.
    pub fn compute_bounds(history: &VecDeque<(f64, f64, f64)>) -> ([f64; 2], [f64; 2]) {
        if history.len() < 2 {
            return ([-25.0, 25.0], [-25.0, 25.0]);
        }
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for &(x, y, _) in history {
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
        if !min_x.is_finite() {
            return ([-25.0, 25.0], [-25.0, 25.0]);
        }
        let mut pad_x = ((max_x - min_x) * BOUNDS_PAD_FRAC).max(2.0);
        let mut pad_y = ((max_y - min_y) * BOUNDS_PAD_FRAC).max(2.0);
        // Keep a usable viewport even if the orbit collapses
        if (max_x - min_x) < 1.0 {
            pad_x = 8.0;
        }
        if (max_y - min_y) < 1.0 {
            pad_y = 8.0;
        }
        (
            [min_x - pad_x, max_x + pad_x],
            [min_y - pad_y, max_y + pad_y],
        )
    }

    /// Keep full orbit length in memory, but paint a strided subset (always includes head).
    fn draw_samples(history: &VecDeque<(f64, f64, f64)>) -> Vec<(f64, f64, f64)> {
        let len = history.len();
        if len == 0 {
            return Vec::new();
        }
        let step = len.div_ceil(MAX_DRAW_POINTS).max(1);
        let mut out = Vec::with_capacity(MAX_DRAW_POINTS + 1);
        for (i, &p) in history.iter().enumerate() {
            if i % step == 0 || i + 1 == len {
                if p.0.is_finite() && p.1.is_finite() {
                    out.push(p);
                }
            }
        }
        out
    }
}

impl Default for ChaosCanvasComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for ChaosCanvasComponent {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::ChaosSnapshot(snap) => {
                // Skip duplicate rebroadcasts (TUI 200 ms heartbeat) so the long
                // trail is real orbit length, not repeated samples.
                let sx = snap.x as f64;
                let sy = snap.y as f64;
                let sz = snap.z as f64;
                let finite = sx.is_finite() && sy.is_finite() && sz.is_finite();
                let moved = self.history.back().map(|&(x, y, z)| {
                    (x - sx).abs() + (y - sy).abs() + (z - sz).abs() > 1e-9
                });
                let new_tick = snap.tick != self.tick;
                if finite && (self.history.is_empty() || new_tick || moved.unwrap_or(true)) {
                    self.history.push_back((sx, sy, sz));
                    if self.history.len() > self.max_history {
                        self.history.pop_front();
                    }
                }
                self.active_tension = snap.tension as f64;
                self.phase = snap.phase;
                if finite {
                    self.last_xyz = (sx, sy, sz);
                }
                self.tick = snap.tick;
                // Live pulse advances tick; quarantine stays at 0
                self.live = snap.tick > 0 || self.history.len() > 3;
            }
            Action::AgentTokenStream(_) => self.speaking = true,
            Action::AgentResponse(_) => self.speaking = false,
            _ => {}
        }
        Ok(None)
    }

    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let drop = matches!(self.phase, Phase::Drop);
        let border = theme::chrome_border(self.active_tension, self.speaking, drop);
        let title_color = if drop {
            theme::RITUAL
        } else if self.speaking {
            GOLD
        } else {
            CYAN
        };

        let (x, y, z) = self.last_xyz;
        let title = lorenz_title(area.width, x, y, z);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(Style::default().fg(title_color).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(border));

        // Not enough trail yet — show status instead of an empty black box
        if self.history.len() < 2 {
            let msg = if self.live {
                "warming attractor…"
            } else {
                "awaiting pulse…"
            };
            let inner = block.inner(area);
            f.render_widget(block, area);
            let p = Paragraph::new(vec![
                Line::from(Span::styled(msg, Style::default().fg(MUTED))),
                Line::from(Span::styled(
                    format!("tick {}  τ{:.0}%", self.tick, self.active_tension),
                    Style::default().fg(STEEL),
                )),
            ]);
            f.render_widget(p, inner);
            return Ok(());
        }

        let base_rgb = if self.active_tension > 80.0 {
            (137.0, 0.0, 0.0)
        } else if self.active_tension > 50.0 {
            (184.0, 115.0, 51.0)
        } else {
            (0.0, 245.0, 255.0)
        };

        let (x_bounds, y_bounds) = Self::compute_bounds(&self.history);
        let samples = Self::draw_samples(&self.history);
        let head = samples.last().map(|&(hx, hy, _)| (hx, hy));

        let canvas = Canvas::default()
            .block(block)
            .x_bounds(x_bounds)
            .y_bounds(y_bounds)
            .marker(ratatui::symbols::Marker::Braille)
            .paint(|ctx| {
                // One Shape pass — avoids 1500× Painter construction and label OOB.
                ctx.draw(&Trail {
                    points: &samples,
                    base_rgb,
                    head,
                });
            });

        f.render_widget(canvas, area);
        Ok(())
    }
}

/// Fit coords into the panel title so the border corner never eats `z`.
fn lorenz_title(width: u16, x: f64, y: f64, z: f64) -> String {
    let inner = width.saturating_sub(2) as usize; // border columns
    let (x, y, z) = (
        if x.is_finite() { x } else { 0.0 },
        if y.is_finite() { y } else { 0.0 },
        if z.is_finite() { z } else { 0.0 },
    );
    let candidates = [
        format!(" LORENZ ({x:.1},{y:.1},{z:.1}) "),
        format!(" LOR ({x:.1},{y:.1},{z:.1}) "),
        format!(" ({x:.0},{y:.0},{z:.0}) "),
        " LORENZ ".to_string(),
    ];
    for c in candidates {
        if c.chars().count() <= inner.max(8) {
            return c;
        }
    }
    " L ".to_string()
}

/// Full trail painted in a single `Shape::draw` (strided + quantized colors).
struct Trail<'a> {
    points: &'a [(f64, f64, f64)],
    base_rgb: (f64, f64, f64),
    head: Option<(f64, f64)>,
}

impl Shape for Trail<'_> {
    fn draw(&self, painter: &mut Painter<'_, '_>) {
        let len = self.points.len().max(1) as f64;
        for (i, &(px, py, pz)) in self.points.iter().enumerate() {
            let age_factor = i as f64 / len;
            let z_factor = pz.clamp(0.0, 50.0) / 50.0;
            let brightness = (0.35 + 0.65 * age_factor) * (0.45 + 0.55 * z_factor);
            // 8 levels — long trails used to emit unique RGB per sample and thrash diffs.
            let q = ((brightness * 8.0).floor() / 8.0).clamp(0.0, 1.0);
            let color = Color::Rgb(
                (self.base_rgb.0 * q) as u8,
                (self.base_rgb.1 * q) as u8,
                (self.base_rgb.2 * q) as u8,
            );
            if let Some((x, y)) = painter.get_point(px, py) {
                painter.paint(x, y, color);
            }
        }
        // Head marker via paint (not ctx.print) — print can OOB the buffer at edges.
        if let Some((hx, hy)) = self.head {
            if let Some((x, y)) = painter.get_point(hx, hy) {
                painter.paint(x, y, PARCHMENT);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gzmo_chaos::pulse::ChaosSnapshot;
    use ratatui::{backend::TestBackend, Terminal};
    use std::time::Instant;

    #[test]
    fn empty_history_uses_default_bounds() {
        let h = VecDeque::new();
        let (xb, yb) = ChaosCanvasComponent::compute_bounds(&h);
        assert_eq!(xb, [-25.0, 25.0]);
        assert_eq!(yb, [-25.0, 25.0]);
    }

    #[test]
    fn bounds_fit_trail() {
        let mut h = VecDeque::new();
        h.push_back((10.0, -5.0, 20.0));
        h.push_back((12.0, -3.0, 22.0));
        h.push_back((14.0, -1.0, 24.0));
        let (xb, yb) = ChaosCanvasComponent::compute_bounds(&h);
        assert!(xb[0] < 10.0 && xb[1] > 14.0);
        assert!(yb[0] < -5.0 && yb[1] > -1.0);
    }

    #[test]
    fn lorenz_title_fits_narrow_panel() {
        let t = super::lorenz_title(18, 14.0, 26.1, 38.2);
        assert!(t.chars().count() <= 16, "title={t:?}");
        assert!(!t.contains('┐'));
    }

    #[test]
    fn snapshots_accumulate_history() {
        let mut c = ChaosCanvasComponent::new();
        for i in 0..20 {
            let mut s = ChaosSnapshot::default();
            s.x = (i as f64) * 0.5;
            s.y = (i as f64) * 0.3;
            s.z = 25.0 + i as f64;
            s.tick = i as u64 + 1;
            c.update(Action::ChaosSnapshot(s)).unwrap();
        }
        assert!(c.history.len() >= 20);
        assert!(c.live);
        assert_eq!(c.tick, 20);
    }

    #[test]
    fn trail_holds_ten_x_samples_and_skips_duplicate_rebroadcasts() {
        let mut c = ChaosCanvasComponent::new();
        assert_eq!(c.max_history, MAX_HISTORY);
        for i in 0..1600u64 {
            let mut s = ChaosSnapshot::default();
            s.x = (i as f64) * 0.01;
            s.y = (i as f64) * 0.02;
            s.z = 27.0;
            s.tick = i + 1;
            c.update(Action::ChaosSnapshot(s.clone())).unwrap();
            // Heartbeat rebroadcast of the same snapshot must not grow the trail.
            c.update(Action::ChaosSnapshot(s)).unwrap();
        }
        assert_eq!(c.history.len(), MAX_HISTORY);
    }

    #[test]
    fn draw_samples_cap_and_keep_head() {
        let mut h = VecDeque::new();
        for i in 0..MAX_HISTORY {
            h.push_back((i as f64, i as f64 * 0.5, 27.0));
        }
        let samples = ChaosCanvasComponent::draw_samples(&h);
        assert!(samples.len() <= MAX_DRAW_POINTS + 1);
        assert_eq!(samples.last().copied(), h.back().copied());
    }

    #[test]
    fn full_trail_render_stays_fast_and_does_not_panic() {
        let mut c = ChaosCanvasComponent::new();
        let mut x = 0.1_f64;
        let mut y = 0.0_f64;
        let mut z = 0.0_f64;
        let dt = 0.005;
        for i in 0..MAX_HISTORY as u64 {
            let dx = 10.0 * (y - x);
            let dy = x * (28.0 - z) - y;
            let dz = x * y - (8.0 / 3.0) * z;
            x += dx * dt;
            y += dy * dt;
            z += dz * dt;
            c.history.push_back((x, y, z));
            c.last_xyz = (x, y, z);
            c.tick = i + 1;
            c.live = true;
        }

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        let t0 = Instant::now();
        for _ in 0..60 {
            terminal
                .draw(|f| {
                    let area = f.size();
                    let rect = Rect::new(
                        area.width / 2,
                        2,
                        area.width / 2,
                        area.height.saturating_sub(5),
                    );
                    c.render(f, rect).unwrap();
                })
                .unwrap();
        }
        let elapsed = t0.elapsed();
        assert!(
            elapsed.as_millis() < 250,
            "full-trail render too slow: {elapsed:?}"
        );
    }
}
