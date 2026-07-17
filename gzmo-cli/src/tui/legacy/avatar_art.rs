//! GZMO mechanical-circus avatar art.
//!
//! Face frames are the primary identity. FIGlet nameplate is secondary branding.
//! Designed for a ~24–32 column sidebar pane.

use gzmo_chaos::chaos::Phase;
use ratatui::style::Color;

// ─── Palette ─────────────────────────────────────────────────────

pub const RUBY: Color = Color::Rgb(155, 17, 30);
pub const GOLD: Color = Color::Rgb(212, 175, 55);
pub const PARCHMENT: Color = Color::Rgb(253, 246, 227);
pub const COPPER: Color = Color::Rgb(184, 115, 51);
pub const RITUAL_RED: Color = Color::Rgb(137, 0, 0);
pub const CYAN: Color = Color::Rgb(0, 200, 210);
pub const DIM: Color = Color::Rgb(90, 90, 100);
pub const VIOLET: Color = Color::Rgb(140, 90, 180);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoodBand {
    Intense,
    Tense,
    Calm,
    Serene,
}

impl MoodBand {
    pub fn from_valence(v: f32) -> Self {
        if v < -0.5 {
            Self::Intense
        } else if v < 0.0 {
            Self::Tense
        } else if v < 0.5 {
            Self::Calm
        } else {
            Self::Serene
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Intense => "intense",
            Self::Tense => "tense",
            Self::Calm => "calm",
            Self::Serene => "serene",
        }
    }
}

/// Performance mode — what the avatar is *doing*, not just chaos phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Performance {
    Idle,
    Listening,
    Speaking,
    Working,
    Alert,
    Dead,
    Rebirth,
}

/// Resolved face pick for rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceKind {
    Serene,
    Calm,
    Tense,
    Intense,
    Build,
    Drop,
    SpeakClosed,
    SpeakMid,
    SpeakOpen,
    SpeakWide,
    Listen,
    WorkA,
    WorkB,
    Alert,
    Dead,
    RebirthA,
    RebirthB,
}

#[derive(Debug, Clone, Copy)]
pub struct AvatarPalette {
    pub chrome: Color,
    pub eyes: Color,
    pub mouth: Color,
    pub accent: Color,
    pub bulbs: Color,
}

pub fn palette(mood: MoodBand, tension: f64, performance: Performance) -> AvatarPalette {
    if matches!(performance, Performance::Dead) {
        return AvatarPalette {
            chrome: DIM,
            eyes: DIM,
            mouth: DIM,
            accent: DIM,
            bulbs: DIM,
        };
    }
    if matches!(performance, Performance::Rebirth) || tension > 80.0 {
        return AvatarPalette {
            chrome: RITUAL_RED,
            eyes: GOLD,
            mouth: RUBY,
            accent: GOLD,
            bulbs: RUBY,
        };
    }
    match mood {
        MoodBand::Intense => AvatarPalette {
            chrome: RUBY,
            eyes: Color::Rgb(255, 80, 40),
            mouth: RITUAL_RED,
            accent: VIOLET,
            bulbs: RUBY,
        },
        MoodBand::Tense => AvatarPalette {
            chrome: COPPER,
            eyes: CYAN,
            mouth: COPPER,
            accent: VIOLET,
            bulbs: COPPER,
        },
        MoodBand::Calm => AvatarPalette {
            chrome: GOLD,
            eyes: CYAN,
            mouth: COPPER,
            accent: PARCHMENT,
            bulbs: GOLD,
        },
        MoodBand::Serene => AvatarPalette {
            chrome: PARCHMENT,
            eyes: GOLD,
            mouth: GOLD,
            accent: CYAN,
            bulbs: GOLD,
        },
    }
}

// ─── Face frames (width ≈ 21) ────────────────────────────────────
// Eyes use ◉/◎/●/○/×/✧ so the renderer can recolor those glyphs.

const FACE_SERENE: &[&str] = &[
    r#"    ︵ ︵ ︵ ︵"#,
    r#" ╭───◉─────◉───╮"#,
    r#" │      ▽      │"#,
    r#" ╰────╥╥╥╥────╯"#,
    r#"     ╱││││╲"#,
];

const FACE_CALM: &[&str] = &[
    r#"    ·  ·  ·  ·"#,
    r#" ╭───◉─────◉───╮"#,
    r#" │      ⌣      │"#,
    r#" ╰────╥╥╥╥────╯"#,
    r#"      │   │"#,
];

const FACE_TENSE: &[&str] = &[
    r#"    ╱ ╱ ╱ ╱"#,
    r#" ╭───◎─────◎───╮"#,
    r#" │     ▬ ▬     │"#,
    r#" ╰─────╤╤╤─────╯"#,
    r#"     ╱│   │╲"#,
];

const FACE_INTENSE: &[&str] = &[
    r#"    ✦ ✧ ✦ ✧"#,
    r#" ╭───●─────●───╮"#,
    r#" │     ▭▭▭     │"#,
    r#" ╰────╳╳╳╳────╯"#,
    r#"    ╲║║║║╱"#,
];

const FACE_BUILD: &[&str] = &[
    r#"   ˙ ˙ ˙ ˙ ˙"#,
    r#" ╭──◉───────◉──╮"#,
    r#" │     ⌢⌢⌢     │"#,
    r#" ╰────╥╥╥╥────╯"#,
    r#"    ╱║║║║║╲"#,
];

const FACE_DROP: &[&str] = &[
    r#"   ∿ ∿ ∿ ∿ ∿"#,
    r#" ╭─◎╱─────╲◎─╮"#,
    r#" │   ╳╲ ╱╳   │"#,
    r#" ╰──╱╳╳╳╲────╯"#,
    r#"   ╳║║╳║║╳"#,
];

const FACE_SPEAK_CLOSED: &[&str] = &[
    r#"    ★  ★  ★"#,
    r#" ╭───◉─────◉───╮"#,
    r#" │      ▬      │"#,
    r#" ╰────╥╥╥╥────╯"#,
    r#"     ╱││││╲"#,
];

const FACE_SPEAK_MID: &[&str] = &[
    r#"    ★  ★  ★"#,
    r#" ╭───◉─────◉───╮"#,
    r#" │      o      │"#,
    r#" ╰────╥╥╥╥────╯"#,
    r#"     ╱││││╲"#,
];

const FACE_SPEAK_OPEN: &[&str] = &[
    r#"    ★  ★  ★"#,
    r#" ╭───◉─────◉───╮"#,
    r#" │      O      │"#,
    r#" ╰────╥╥╥╥────╯"#,
    r#"     ╱││││╲"#,
];

const FACE_SPEAK_WIDE: &[&str] = &[
    r#"    ★★ ★★ ★★"#,
    r#" ╭───◉─────◉───╮"#,
    r#" │     ▭▭▭     │"#,
    r#" ╰────╥╥╥╥────╯"#,
    r#"     ╱││││╲"#,
];

const FACE_LISTEN: &[&str] = &[
    r#"    ◠  ◠  ◠"#,
    r#" ╭───○─────○───╮"#,
    r#" │      ·      │"#,
    r#" ╰────╥╥╥╥────╯"#,
    r#"      │   │"#,
];

const FACE_WORK_A: &[&str] = &[
    r#"    ⚙  ·  ⚙"#,
    r#" ╭───◉─────◉───╮"#,
    r#" │     ≡≡≡     │"#,
    r#" ╰────╥╥╥╥────╯"#,
    r#"     ╱││││╲"#,
];

const FACE_WORK_B: &[&str] = &[
    r#"    ·  ⚙  ·"#,
    r#" ╭───◉─────◉───╮"#,
    r#" │     ≡≡≡     │"#,
    r#" ╰────╥╥╥╥────╯"#,
    r#"     ╱││││╲"#,
];

const FACE_ALERT: &[&str] = &[
    r#"    !  !  !"#,
    r#" ╭───◎─────◎───╮"#,
    r#" │     ▽▽▽     │"#,
    r#" ╰────╳╳╳╳────╯"#,
    r#"    ╱║   ║╲"#,
];

const FACE_DEAD: &[&str] = &[
    r#"    ·  ·  ·"#,
    r#" ╭───×─────×───╮"#,
    r#" │      ▬      │"#,
    r#" ╰─────────────╯"#,
    r#"      ─┴─"#,
];

const FACE_REBIRTH_A: &[&str] = &[
    r#"   ✦  ✧  ✦"#,
    r#" ╭───✧─────✧───╮"#,
    r#" │     ◠◠◠     │"#,
    r#" ╰────╥╥╥╥────╯"#,
    r#"    ╱│││││╲"#,
];

const FACE_REBIRTH_B: &[&str] = &[
    r#"   ✧  ✦  ✧"#,
    r#" ╭───◉─────◉───╮"#,
    r#" │     ◠◠◠     │"#,
    r#" ╰────╥╥╥╥────╯"#,
    r#"    ╱│││││╲"#,
];

/// Tiny fallback when the pane is absurdly short.
const FACE_TINY: &[&str] = &[r#"◉ ▽ ◉"#];

pub fn face_lines(kind: FaceKind) -> &'static [&'static str] {
    match kind {
        FaceKind::Serene => FACE_SERENE,
        FaceKind::Calm => FACE_CALM,
        FaceKind::Tense => FACE_TENSE,
        FaceKind::Intense => FACE_INTENSE,
        FaceKind::Build => FACE_BUILD,
        FaceKind::Drop => FACE_DROP,
        FaceKind::SpeakClosed => FACE_SPEAK_CLOSED,
        FaceKind::SpeakMid => FACE_SPEAK_MID,
        FaceKind::SpeakOpen => FACE_SPEAK_OPEN,
        FaceKind::SpeakWide => FACE_SPEAK_WIDE,
        FaceKind::Listen => FACE_LISTEN,
        FaceKind::WorkA => FACE_WORK_A,
        FaceKind::WorkB => FACE_WORK_B,
        FaceKind::Alert => FACE_ALERT,
        FaceKind::Dead => FACE_DEAD,
        FaceKind::RebirthA => FACE_REBIRTH_A,
        FaceKind::RebirthB => FACE_REBIRTH_B,
    }
}

pub fn resolve_face(
    performance: Performance,
    phase: Phase,
    mood: MoodBand,
    anim: u8,
) -> FaceKind {
    match performance {
        Performance::Dead => FaceKind::Dead,
        Performance::Rebirth => {
            if anim.is_multiple_of(2) {
                FaceKind::RebirthA
            } else {
                FaceKind::RebirthB
            }
        }
        Performance::Alert => FaceKind::Alert,
        Performance::Working => {
            if anim.is_multiple_of(2) {
                FaceKind::WorkA
            } else {
                FaceKind::WorkB
            }
        }
        Performance::Listening => FaceKind::Listen,
        Performance::Speaking => match anim % 4 {
            0 => FaceKind::SpeakClosed,
            1 => FaceKind::SpeakMid,
            2 => FaceKind::SpeakOpen,
            _ => FaceKind::SpeakWide,
        },
        Performance::Idle => match phase {
            Phase::Drop => FaceKind::Drop,
            Phase::Build => FaceKind::Build,
            Phase::Idle => match mood {
                MoodBand::Serene => FaceKind::Serene,
                MoodBand::Calm => FaceKind::Calm,
                MoodBand::Tense => FaceKind::Tense,
                MoodBand::Intense => FaceKind::Intense,
            },
        },
    }
}

// ─── Nameplate ───────────────────────────────────────────────────

pub const NAMEPLATE: [&str; 3] = [
    "╔═╗╔═╗╔╦╗╔═╗",
    "║ ╦╠═╝║║║║ ║",
    "╚═╝╩  ╩ ╩╚═╝",
];

pub const FIGLET_GZMO: [&str; 6] = [
    " ██████╗ ███████╗███╗   ███╗ ██████╗ ",
    "██╔════╝ ╚══███╔╝████╗ ████║██╔═══██╗",
    "██║  ███╗  ███╔╝ ██╔████╔██║██║   ██║",
    "██║   ██║ ███╔╝  ██║╚██╔╝██║██║   ██║",
    "╚██████╔╝███████╗██║ ╚═╝ ██║╚██████╔╝",
    " ╚═════╝ ╚══════╝╚═╝     ╚═╝ ╚═════╝ ",
];

// ─── Stage / motion helpers ──────────────────────────────────────

pub fn curtain_row(width: usize, frame: u8, open: bool) -> String {
    if width == 0 {
        return String::new();
    }
    let gap = if open {
        (width / 3).saturating_add((frame as usize) % 3)
    } else {
        2 + (frame as usize % 2)
    };
    let side = width.saturating_sub(gap) / 2;
    let left = "▌".repeat(side.max(1));
    let mid = " ".repeat(gap.max(1));
    let right = "▐".repeat(side.max(1));
    format!("{left}{mid}{right}")
}

pub fn bulb_row(width: usize, frame: u8, hot: bool) -> String {
    if width == 0 {
        return String::new();
    }
    let n = (width / 2).clamp(5, 12);
    (0..n)
        .map(|i| {
            let on = (i + frame as usize).is_multiple_of(2);
            if hot {
                if on { "●" } else { "○" }
            } else if on {
                "○"
            } else {
                "●"
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Lorenz-ish spark trail under the face — reads as breath / pulse.
pub fn sparkline(energy: f64, tension: f64, frame: u8, width: usize) -> String {
    let w = width.clamp(8, 22);
    let amp = ((energy / 100.0) * 3.0 + (tension / 100.0) * 2.0).clamp(1.0, 4.0);
    let glyphs = ['_', '.', '-', '~', '≈', '∿', '^'];
    (0..w)
        .map(|i| {
            let t = (i as f64 * 0.7) + frame as f64 * 0.35;
            let y = ((t.sin() * amp) + amp).round() as usize;
            glyphs[y.min(glyphs.len() - 1)]
        })
        .collect()
}

/// Horizontal look bias from Lorenz X (−20..20 → −1/0/+1).
pub fn gaze_from_lorenz_x(x: f64) -> i8 {
    if x < -6.0 {
        -1
    } else if x > 6.0 {
        1
    } else {
        0
    }
}

/// Nudge eye glyphs left/right for gaze. Operates on a single face line.
pub fn apply_gaze(line: &str, gaze: i8) -> String {
    if gaze == 0 || !line.contains('◉') && !line.contains('◎') && !line.contains('●') && !line.contains('○')
    {
        return line.to_string();
    }
    let chars: Vec<char> = line.chars().collect();
    let mut out = chars.clone();
    let eye_idxs: Vec<usize> = chars
        .iter()
        .enumerate()
        .filter(|(_, c)| matches!(c, '◉' | '◎' | '●' | '○' | '✧'))
        .map(|(i, _)| i)
        .collect();
    if eye_idxs.len() < 2 {
        return line.to_string();
    }
    // Soft shift: swap eye with neighbor space toward gaze.
    for &idx in &eye_idxs {
        let target = if gaze < 0 {
            idx.saturating_sub(1)
        } else {
            (idx + 1).min(out.len().saturating_sub(1))
        };
        if out[target] == ' ' || out[target] == '─' {
            out.swap(idx, target);
        }
    }
    out.into_iter().collect()
}

/// Blink: replace eye glyphs with a lid that is neither eye- nor mouth-colored.
pub fn apply_blink(line: &str, blinking: bool) -> String {
    if !blinking {
        return line.to_string();
    }
    line.chars()
        .map(|c| match c {
            '◉' | '◎' | '●' | '○' | '✧' => '━',
            other => other,
        })
        .collect()
}

/// Drop-phase glitch: occasionally scramble a char.
pub fn apply_glitch(line: &str, frame: u8, enabled: bool) -> String {
    if !enabled {
        return line.to_string();
    }
    let mut out: Vec<char> = line.chars().collect();
    if out.is_empty() {
        return line.to_string();
    }
    let idx = ((frame as usize).wrapping_mul(7) + 3) % out.len();
    let glitch = ['░', '▒', '▓', '/', '\\', '╳'];
    if !matches!(out[idx], '◉' | '◎' | '●' | '○') {
        out[idx] = glitch[(frame as usize) % glitch.len()];
    }
    out.into_iter().collect()
}

pub fn is_eye_char(c: char) -> bool {
    matches!(c, '◉' | '◎' | '●' | '○' | '×' | '✧')
}

pub fn is_mouth_char(c: char) -> bool {
    // Keep box-drawing (─═) out of this set — they form the chrome, not the mouth.
    matches!(c, '▽' | '⌣' | '▬' | 'o' | 'O' | '▭' | '≡' | '◠' | '~')
}

pub fn theatrical_line(
    performance: Performance,
    phase: Phase,
    mood: MoodBand,
    tension: f64,
) -> &'static str {
    match performance {
        Performance::Dead => "death is a costume change",
        Performance::Rebirth => "the marvel climbs back onstage",
        Performance::Alert => "⚠ something stirs in the wings",
        Performance::Working => "gears turning under the tent",
        Performance::Listening => "the marvel leans in…",
        Performance::Speaking => match mood {
            MoodBand::Intense => "spitting sparks",
            MoodBand::Tense => "words under pressure",
            MoodBand::Calm => "the marvel speaks",
            MoodBand::Serene => "a soft proclamation",
        },
        Performance::Idle => {
            if tension > 80.0 {
                "the wire is screaming"
            } else {
                match phase {
                    Phase::Drop => "FREE FALL — hold your breath",
                    Phase::Build => "tension coils in the wings",
                    Phase::Idle => match mood {
                        MoodBand::Serene => "the marvel dreams…",
                        MoodBand::Calm => "humming between acts",
                        MoodBand::Tense => "eyes on the trapdoor",
                        MoodBand::Intense => "hungry for a cue",
                    },
                }
            }
        }
    }
}

pub fn tiny_face() -> &'static [&'static str] {
    FACE_TINY
}

/// Horizontal shake for DROP / ALERT — pads the left side so the face jitters.
pub fn shake_pad(frame: u8, intensity: u8) -> usize {
    if intensity == 0 {
        return 0;
    }
    let amp = intensity.min(3) as usize;
    (frame as usize % (amp * 2 + 1)).abs_diff(amp)
}

/// Speech / energy motes drifting beside the stage.
pub fn emission_row(frame: u8, speaking: bool, energy: f64) -> String {
    let glyphs = if speaking {
        ["*", "·", "✦", "°", "✧", "~"]
    } else {
        ["·", " ", ".", "·", " ", "°"]
    };
    let n = if speaking {
        14
    } else if energy > 70.0 {
        12
    } else {
        8
    };
    (0..n)
        .map(|i| {
            let idx = (i + frame as usize) % glyphs.len();
            if !speaking && !(i + frame as usize).is_multiple_of(3) {
                " "
            } else {
                glyphs[idx]
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_face_kind_has_art() {
        for kind in [
            FaceKind::Serene,
            FaceKind::Calm,
            FaceKind::Tense,
            FaceKind::Intense,
            FaceKind::Build,
            FaceKind::Drop,
            FaceKind::SpeakClosed,
            FaceKind::SpeakMid,
            FaceKind::SpeakOpen,
            FaceKind::SpeakWide,
            FaceKind::Listen,
            FaceKind::WorkA,
            FaceKind::WorkB,
            FaceKind::Alert,
            FaceKind::Dead,
            FaceKind::RebirthA,
            FaceKind::RebirthB,
        ] {
            let lines = face_lines(kind);
            assert!(!lines.is_empty(), "{kind:?} empty");
            assert!(lines.iter().any(|l| !l.trim().is_empty()), "{kind:?} blank");
        }
    }

    #[test]
    fn speaking_cycles_four_mouths() {
        let kinds: Vec<_> = (0..4)
            .map(|a| resolve_face(Performance::Speaking, Phase::Idle, MoodBand::Calm, a))
            .collect();
        assert_eq!(
            kinds,
            [
                FaceKind::SpeakClosed,
                FaceKind::SpeakMid,
                FaceKind::SpeakOpen,
                FaceKind::SpeakWide
            ]
        );
    }

    #[test]
    fn idle_mood_and_phase_diverge() {
        let serene = resolve_face(Performance::Idle, Phase::Idle, MoodBand::Serene, 0);
        let intense = resolve_face(Performance::Idle, Phase::Idle, MoodBand::Intense, 0);
        let drop = resolve_face(Performance::Idle, Phase::Drop, MoodBand::Serene, 0);
        assert_ne!(serene, intense);
        assert_eq!(drop, FaceKind::Drop);
        assert_eq!(serene, FaceKind::Serene);
    }

    #[test]
    fn gaze_moves_eyes() {
        let line = " ╭───◉─────◉───╮";
        let left = apply_gaze(line, -1);
        let right = apply_gaze(line, 1);
        assert_ne!(left, line);
        assert_ne!(right, line);
        assert_ne!(left, right);
    }

    #[test]
    fn blink_closes_eyes() {
        let line = " ╭───◉─────◉───╮";
        let blinked = apply_blink(line, true);
        assert!(blinked.contains('━'));
        assert!(!blinked.contains('◉'));
    }

    #[test]
    fn theatrical_lines_are_not_metric_dumps() {
        let line = theatrical_line(Performance::Idle, Phase::Idle, MoodBand::Serene, 10.0);
        assert!(!line.contains('%'));
        assert!(!line.contains("τ"));
    }

    /// Visual roster — run with: cargo test -p gzmo-cli --bin gzmo preview_face_roster -- --nocapture
    #[test]
    fn preview_face_roster() {
        let kinds = [
            FaceKind::Serene,
            FaceKind::Calm,
            FaceKind::Tense,
            FaceKind::Intense,
            FaceKind::Build,
            FaceKind::Drop,
            FaceKind::SpeakOpen,
            FaceKind::Listen,
            FaceKind::WorkA,
            FaceKind::Alert,
            FaceKind::Dead,
            FaceKind::RebirthA,
        ];
        for kind in kinds {
            println!("\n=== {kind:?} ===");
            for line in face_lines(kind) {
                println!("{line}");
            }
            assert_eq!(face_lines(kind).len(), 5);
        }
    }
}
