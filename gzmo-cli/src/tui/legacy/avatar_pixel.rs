//! Half-block pixel avatar — a friendly talking cogwheel.
//!
//! Each terminal cell is two vertical pixels via `▀` (fg = top, bg = bottom).
//! Truecolor required for the look; falls back to readable blocks otherwise.

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use super::avatar_art::{MoodBand, Performance};

const W: usize = 32;
const H: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rgb(u8, u8, u8);

impl Rgb {
    const fn new(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b)
    }

    fn to_color(self) -> Color {
        Color::Rgb(self.0, self.1, self.2)
    }

    fn dim(self, factor: f32) -> Self {
        Self(
            ((self.0 as f32) * factor) as u8,
            ((self.1 as f32) * factor) as u8,
            ((self.2 as f32) * factor) as u8,
        )
    }
}

#[derive(Clone, Copy)]
struct Palette {
    metal: Rgb,
    metal_dark: Rgb,
    metal_light: Rgb,
    hub: Rgb,
    eye_white: Rgb,
    pupil: Rgb,
    mouth: Rgb,
    cheek: Rgb,
    tooth: Rgb,
    spark: Rgb,
    bg: Option<Rgb>, // None = transparent
}

fn palette_for(mood: MoodBand, tension: f64, performance: Performance) -> Palette {
    if matches!(performance, Performance::Dead) {
        return Palette {
            metal: Rgb::new(70, 70, 75),
            metal_dark: Rgb::new(40, 40, 45),
            metal_light: Rgb::new(110, 110, 115),
            hub: Rgb::new(55, 55, 60),
            eye_white: Rgb::new(140, 140, 145),
            pupil: Rgb::new(30, 30, 35),
            mouth: Rgb::new(50, 50, 55),
            cheek: Rgb::new(70, 70, 75),
            tooth: Rgb::new(90, 90, 95),
            spark: Rgb::new(60, 60, 65),
            bg: None,
        };
    }

    let (metal, accent, mouth) = if tension > 80.0
        || matches!(performance, Performance::Alert | Performance::Rebirth)
    {
        (
            Rgb::new(180, 40, 45),
            Rgb::new(255, 200, 60),
            Rgb::new(120, 20, 30),
        )
    } else {
        match mood {
            MoodBand::Intense => (
                Rgb::new(170, 45, 50),
                Rgb::new(255, 120, 40),
                Rgb::new(100, 25, 35),
            ),
            MoodBand::Tense => (
                Rgb::new(190, 120, 55),
                Rgb::new(0, 200, 210),
                Rgb::new(140, 70, 40),
            ),
            MoodBand::Calm => (
                Rgb::new(200, 165, 70),
                Rgb::new(80, 180, 190),
                Rgb::new(150, 90, 50),
            ),
            MoodBand::Serene => (
                Rgb::new(220, 195, 110),
                Rgb::new(100, 200, 160),
                Rgb::new(170, 110, 70),
            ),
        }
    };

    Palette {
        metal,
        metal_dark: metal.dim(0.55),
        metal_light: Rgb::new(
            metal.0.saturating_add(40).min(255),
            metal.1.saturating_add(35).min(255),
            metal.2.saturating_add(25).min(255),
        ),
        hub: metal.dim(0.75),
        eye_white: Rgb::new(250, 245, 230),
        pupil: Rgb::new(30, 25, 40),
        mouth,
        cheek: Rgb::new(220, 120, 110),
        tooth: accent,
        spark: accent,
        bg: None,
    }
}

#[derive(Clone)]
struct Buf {
    w: usize,
    h: usize,
    px: Vec<Option<Rgb>>,
}

impl Buf {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            px: vec![None; w * h],
        }
    }

    fn set(&mut self, x: i32, y: i32, c: Rgb) {
        if x < 0 || y < 0 {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        if x < self.w && y < self.h {
            self.px[y * self.w + x] = Some(c);
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<Rgb> {
        if x < self.w && y < self.h {
            self.px[y * self.w + x]
        } else {
            None
        }
    }

    fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, c: Rgb) {
        let r2 = r * r;
        for y in (cy - r)..=(cy + r) {
            for x in (cx - r)..=(cx + r) {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= r2 {
                    self.set(x, y, c);
                }
            }
        }
    }

    fn fill_ring(&mut self, cx: i32, cy: i32, r_outer: i32, r_inner: i32, c: Rgb) {
        let ro2 = r_outer * r_outer;
        let ri2 = r_inner * r_inner;
        for y in (cy - r_outer)..=(cy + r_outer) {
            for x in (cx - r_outer)..=(cx + r_outer) {
                let dx = x - cx;
                let dy = y - cy;
                let d2 = dx * dx + dy * dy;
                if d2 <= ro2 && d2 >= ri2 {
                    self.set(x, y, c);
                }
            }
        }
    }

    fn fill_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: Rgb) {
        for y in y0.min(y1)..=y0.max(y1) {
            for x in x0.min(x1)..=x0.max(x1) {
                self.set(x, y, c);
            }
        }
    }

    fn fill_ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, c: Rgb) {
        if rx <= 0 || ry <= 0 {
            return;
        }
        for y in (cy - ry)..=(cy + ry) {
            for x in (cx - rx)..=(cx + rx) {
                let nx = (x - cx) as f32 / rx as f32;
                let ny = (y - cy) as f32 / ry as f32;
                if nx * nx + ny * ny <= 1.0 {
                    self.set(x, y, c);
                }
            }
        }
    }
}

/// Expression knobs for one cog frame.
#[derive(Clone, Copy)]
pub struct CogPose {
    pub mood: MoodBand,
    pub performance: Performance,
    pub tension: f64,
    pub energy: f64,
    /// -1 left … +1 right
    pub gaze_x: i8,
    pub blink: bool,
    /// 0 closed … 3 wide open
    pub mouth: u8,
    /// Rotation offset for teeth (animation)
    pub spin: u8,
    pub sparkle: bool,
}

impl Default for CogPose {
    fn default() -> Self {
        Self {
            mood: MoodBand::Calm,
            performance: Performance::Idle,
            tension: 0.0,
            energy: 80.0,
            gaze_x: 0,
            blink: false,
            mouth: 0,
            spin: 0,
            sparkle: false,
        }
    }
}

fn draw_cog(pose: &CogPose) -> Buf {
    let pal = palette_for(pose.mood, pose.tension, pose.performance);
    let mut buf = Buf::new(W, H);
    let cx = (W / 2) as i32;
    let cy = (H / 2) as i32;

    // Spinning teeth
    let teeth = 10;
    let outer = 14;
    let inner = 11;
    let tooth_len = 3;
    let spin = (pose.spin as f32) * (std::f32::consts::TAU / 40.0);
    for i in 0..teeth {
        let a = spin + (i as f32) * (std::f32::consts::TAU / teeth as f32);
        let (sa, ca) = (a.sin(), a.cos());
        // tooth as small radial block
        for t in 0..=tooth_len {
            let r = outer + t;
            let x = cx + (ca * r as f32).round() as i32;
            let y = cy + (sa * r as f32).round() as i32;
            // widen tooth sideways
            let px = (-sa).round() as i32;
            let py = ca.round() as i32;
            buf.set(x, y, pal.tooth);
            buf.set(x + px, y + py, pal.metal_light);
            buf.set(x - px, y - py, pal.metal_dark);
        }
    }

    // Main disc + rim
    buf.fill_circle(cx, cy, outer, pal.metal);
    buf.fill_ring(cx, cy, outer, outer - 1, pal.metal_light);
    buf.fill_ring(cx, cy, inner, inner - 2, pal.metal_dark);

    // Hub
    buf.fill_circle(cx, cy, 7, pal.hub);
    buf.fill_circle(cx, cy, 6, pal.metal.dim(0.85));

    // Cheeks (friendly)
    if !matches!(pose.performance, Performance::Dead) && pose.mood != MoodBand::Intense {
        buf.fill_circle(cx - 5, cy + 2, 1, pal.cheek);
        buf.fill_circle(cx + 5, cy + 2, 1, pal.cheek);
    }

    // Eyes
    let eye_y = cy - 1;
    let eye_dx = 3;
    let pupil_shift = pose.gaze_x.clamp(-1, 1) as i32;
    for side in [-1i32, 1] {
        let ex = cx + side * eye_dx;
        if pose.blink || matches!(pose.performance, Performance::Dead) {
            // closed lids
            buf.fill_rect(ex - 2, eye_y, ex + 2, eye_y, pal.metal_dark);
            if matches!(pose.performance, Performance::Dead) {
                // X eyes
                buf.set(ex - 1, eye_y - 1, pal.pupil);
                buf.set(ex + 1, eye_y - 1, pal.pupil);
                buf.set(ex - 1, eye_y + 1, pal.pupil);
                buf.set(ex + 1, eye_y + 1, pal.pupil);
                buf.set(ex, eye_y, pal.pupil);
            }
        } else {
            buf.fill_ellipse(ex, eye_y, 2, 2, pal.eye_white);
            let px = ex + pupil_shift;
            let py = if matches!(pose.performance, Performance::Listening) {
                eye_y + 1
            } else {
                eye_y
            };
            buf.fill_circle(px, py, 1, pal.pupil);
            // highlight
            buf.set(px - 1, py - 1, Rgb::new(255, 255, 255));
        }
    }

    // Brows for tense / build / alert
    if matches!(
        pose.performance,
        Performance::Alert | Performance::Working
    ) || pose.mood == MoodBand::Tense
        || pose.mood == MoodBand::Intense
    {
        buf.fill_rect(cx - 5, eye_y - 3, cx - 2, eye_y - 3, pal.metal_dark);
        buf.fill_rect(cx + 2, eye_y - 3, cx + 5, eye_y - 3, pal.metal_dark);
    }

    // Mouth
    let mouth_y = cy + 3;
    match pose.performance {
        Performance::Dead => {
            buf.fill_rect(cx - 2, mouth_y, cx + 2, mouth_y, pal.mouth);
        }
        _ => {
            let open = match pose.mouth {
                0 => 0,
                1 => 1,
                2 => 2,
                _ => 3,
            };
            if open == 0 {
                // smile curve
                buf.set(cx - 2, mouth_y, pal.mouth);
                buf.set(cx - 1, mouth_y + 1, pal.mouth);
                buf.set(cx, mouth_y + 1, pal.mouth);
                buf.set(cx + 1, mouth_y + 1, pal.mouth);
                buf.set(cx + 2, mouth_y, pal.mouth);
            } else {
                buf.fill_ellipse(cx, mouth_y + 1, 2 + open / 2, open, pal.mouth);
                // tongue / depth
                if open >= 2 {
                    buf.fill_ellipse(cx, mouth_y + 1, 1, open - 1, pal.mouth.dim(0.6));
                }
            }
        }
    }

    // Listening ear-marks / speak sparks
    if pose.sparkle || matches!(pose.performance, Performance::Speaking) {
        let spark_pts = [
            (cx - 13, cy - 4),
            (cx + 13, cy - 3),
            (cx - 12, cy + 5),
            (cx + 12, cy + 4),
            (cx, cy - 15),
        ];
        for (i, (x, y)) in spark_pts.iter().enumerate() {
            if (pose.spin as usize + i) % 2 == 0 {
                buf.set(*x, *y, pal.spark);
                buf.set(*x + 1, *y, pal.spark.dim(0.7));
            }
        }
    }

    // Working: small gear tick marks orbiting
    if matches!(pose.performance, Performance::Working) {
        let a = spin * 2.0;
        let x = cx + (a.cos() * 13.0).round() as i32;
        let y = cy + (a.sin() * 13.0).round() as i32;
        buf.fill_circle(x, y, 1, pal.spark);
    }

    // Energy dimming — darken whole buffer slightly when exhausted
    if pose.energy < 25.0 && !matches!(pose.performance, Performance::Dead) {
        for p in buf.px.iter_mut() {
            if let Some(c) = p {
                *c = c.dim(0.55);
            }
        }
    }

    let _ = pal.bg; // reserved
    buf
}

/// Encode framebuffer to half-block [`Line`]s for ratatui.
pub fn render_cog_lines(pose: &CogPose) -> Vec<Line<'static>> {
    let buf = draw_cog(pose);
    let mut lines = Vec::with_capacity(H / 2);
    for y in (0..H).step_by(2) {
        let mut spans = Vec::with_capacity(W);
        for x in 0..W {
            let top = buf.get(x, y);
            let bot = buf.get(x, y + 1);
            match (top, bot) {
                (None, None) => {
                    spans.push(Span::raw(" "));
                }
                (Some(t), Some(b)) => {
                    spans.push(Span::styled(
                        "▀",
                        Style::default().fg(t.to_color()).bg(b.to_color()),
                    ));
                }
                (Some(t), None) => {
                    spans.push(Span::styled("▀", Style::default().fg(t.to_color())));
                }
                (None, Some(b)) => {
                    // lower half only — use ▄ with fg
                    spans.push(Span::styled("▄", Style::default().fg(b.to_color())));
                }
            }
        }
        lines.push(Line::from(spans));
    }
    lines
}

/// Build a pose from avatar runtime state.
pub fn pose_from_state(
    mood: MoodBand,
    performance: Performance,
    tension: f64,
    energy: f64,
    lorenz_x: f64,
    anim_frame: u8,
    blinking: bool,
) -> CogPose {
    let gaze_x = if lorenz_x < -6.0 {
        -1
    } else if lorenz_x > 6.0 {
        1
    } else {
        0
    };

    let mouth = match performance {
        Performance::Speaking => anim_frame % 4,
        Performance::Alert => 2,
        Performance::Listening => 0,
        Performance::Working => 1,
        Performance::Rebirth => 1 + (anim_frame % 2),
        Performance::Dead => 0,
        Performance::Idle => 0,
    };

    let spin = if tension > 70.0
        || matches!(
            performance,
            Performance::Speaking | Performance::Working | Performance::Alert | Performance::Rebirth
        ) {
        anim_frame.wrapping_mul(2)
    } else if tension > 40.0 {
        anim_frame
    } else {
        anim_frame / 3
    };

    CogPose {
        mood,
        performance,
        tension,
        energy,
        gaze_x,
        blink: blinking || matches!(performance, Performance::Dead),
        mouth,
        spin,
        sparkle: matches!(
            performance,
            Performance::Speaking | Performance::Alert | Performance::Rebirth
        ) || energy > 85.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_nonempty_halfblocks() {
        let pose = CogPose::default();
        let lines = render_cog_lines(&pose);
        assert_eq!(lines.len(), H / 2);
        assert!(lines.iter().any(|l| !l.spans.is_empty()));
        // Should contain half-block glyphs
        let joined: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();
        assert!(joined.contains('▀') || joined.contains('▄'));
    }

    #[test]
    fn speaking_mouth_opens() {
        let mut closed = CogPose::default();
        closed.mouth = 0;
        let mut open = CogPose::default();
        open.mouth = 3;
        let a = draw_cog(&closed);
        let b = draw_cog(&open);
        assert_ne!(a.px, b.px);
    }

    #[test]
    fn blink_changes_pixels() {
        let mut open = CogPose::default();
        open.blink = false;
        let mut blink = CogPose::default();
        blink.blink = true;
        assert_ne!(draw_cog(&open).px, draw_cog(&blink).px);
    }

    #[test]
    fn dead_palette_differs() {
        let alive = CogPose::default();
        let mut dead = CogPose::default();
        dead.performance = Performance::Dead;
        assert_ne!(draw_cog(&alive).px, draw_cog(&dead).px);
    }
}
