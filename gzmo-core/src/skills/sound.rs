//! # Sound Skill — `/sound`
//!
//! Heartbeat-reactive sound effects with ASCII art visuals
//! and sox audio synthesis. Categories are selected based on
//! chaos tension: high=aggressive, mid=ambient, low=ethereal.
//!
//! Emits `ChaosEvent::SoundFired` for feedback.

use anyhow::Result;
use async_trait::async_trait;

use gzmo_chaos::chaos::Phase;
use gzmo_chaos::feedback::{ChaosEvent, SoundCategory};

use super::{Skill, SkillContext, SkillOutput, SkillType};

const GOLD: &str = "\x1b[38;2;212;175;55m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const WHITE: &str = "\x1b[97m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";
const ORANGE: &str = "\x1b[38;2;255;140;0m";
const SKY: &str = "\x1b[38;2;135;206;235m";
const PINK: &str = "\x1b[38;2;255;105;180m";

pub struct SoundSkill;

#[derive(Clone, Copy)]
enum SoundType {
    Explosion,
    Thunder,
    Alarm,
    Roar,
    Bell,
    Guitar,
    Drum,
    Wave,
    Chime,
    Piano,
    Wind,
    Hum,
}

impl SoundType {
    fn name(&self) -> &str {
        match self {
            Self::Explosion => "explosion",
            Self::Thunder => "thunder",
            Self::Alarm => "alarm",
            Self::Roar => "roar",
            Self::Bell => "bell",
            Self::Guitar => "guitar",
            Self::Drum => "drum",
            Self::Wave => "wave",
            Self::Chime => "chime",
            Self::Piano => "piano",
            Self::Wind => "wind",
            Self::Hum => "hum",
        }
    }

    fn to_category(&self) -> SoundCategory {
        match self {
            Self::Explosion => SoundCategory::Explosion,
            Self::Thunder => SoundCategory::Thunder,
            Self::Alarm => SoundCategory::Alarm,
            Self::Roar => SoundCategory::Roar,
            Self::Bell => SoundCategory::Bell,
            Self::Guitar => SoundCategory::Guitar,
            Self::Drum => SoundCategory::Drum,
            Self::Wave => SoundCategory::Wave,
            Self::Chime => SoundCategory::Chime,
            Self::Piano => SoundCategory::Piano,
            Self::Wind => SoundCategory::Wind,
            Self::Hum => SoundCategory::Hum,
        }
    }

    fn sox_args(&self) -> Option<&str> {
        Some(match self {
            Self::Explosion => "synth 0.4 noise vol 0.5",
            Self::Thunder => "synth 0.6 brownnoise synth 0.1 sine 100 vol 0.4",
            Self::Alarm => "synth 0.15 sine 880 synth 0.15 sine 660 repeat 3 vol 0.3",
            Self::Roar => "synth 0.5 brownnoise tremolo 5 80 vol 0.4",
            Self::Bell => "synth 0.8 sine 1200 fade 0 0.8 0.5 vol 0.3",
            Self::Guitar => "synth 0.6 pluck 330 vol 0.4",
            Self::Drum => "synth 0.05 noise synth 0.3 sine 80 vol 0.4",
            Self::Wave => "synth 1.0 pinknoise tremolo 0.5 60 vol 0.3",
            Self::Chime => "synth 0.5 sine 2000 fade 0 0.5 0.3 vol 0.2",
            Self::Piano => "synth 0.5 pluck 440 vol 0.3",
            Self::Wind => "synth 0.8 pinknoise vol 0.2",
            Self::Hum => "synth 0.6 sine 220 tremolo 8 50 vol 0.2",
        })
    }
}

/// Pick a sound category based on tension level
fn pick_sound(snap: &gzmo_chaos::pulse::ChaosSnapshot) -> SoundType {
    let tension = snap.tension;
    let pool = if tension > 60.0 {
        &[
            SoundType::Explosion,
            SoundType::Thunder,
            SoundType::Alarm,
            SoundType::Roar,
        ]
    } else if tension > 30.0 {
        &[
            SoundType::Bell,
            SoundType::Guitar,
            SoundType::Drum,
            SoundType::Wave,
        ]
    } else {
        &[
            SoundType::Chime,
            SoundType::Piano,
            SoundType::Wind,
            SoundType::Hum,
        ]
    };
    let idx = ((snap.chaos_val * 1000.0 + snap.x.abs()) as usize) % pool.len();
    pool[idx]
}

/// Try to play audio via sox (non-blocking)
fn play_audio(sound: SoundType) {
    if let Some(args) = sound.sox_args() {
        let full_args: Vec<&str> = std::iter::once("-qn")
            .chain(args.split_whitespace())
            .collect();
        let _ = std::process::Command::new("play")
            .args(&full_args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn(); // Fire and forget
    }
}

fn render_visual(sound: SoundType) -> String {
    match sound {
        SoundType::Explosion => format!(
            "\n  {DIM}{RED}                    .  .{RESET}\n\
             {RED}               . .  :  . .{RESET}\n\
             {ORANGE}            .  :  .  .  :  .{RESET}\n\
             {YELLOW}          .  :  . {BOLD}{WHITE}💥{RESET}{YELLOW} .  :  .{RESET}\n\
             {ORANGE}         : .  . : . : .  . :{RESET}\n\
             {RED}          . .  : {BOLD}BOOM{RESET}{RED} :  . .{RESET}\n\
             {ORANGE}            .  :  .  .  :  .{RESET}\n\
             {RED}               . .  :  . .{RESET}\n\
             {DIM}{RED}                    .  .{RESET}\n\n\
             {DIM}  A thunderous detonation rips through the phase space{RESET}"
        ),
        SoundType::Thunder => format!(
            "\n  {DIM}{BLUE}  ░░░▒▒▒▓▓▓████▓▓▓▒▒▒░░░░░▒▒▓▓████▓▓▒▒░░{RESET}\n\
             {WHITE}{BOLD}                  ╲{RESET}\n\
             {YELLOW}{BOLD}                   ╲{RESET}\n\
             {YELLOW}{BOLD}                  ╱╲{RESET}\n\
             {WHITE}{BOLD}                 ╱{RESET}\n\
             {YELLOW}{BOLD}                ╱{RESET}\n\
             {YELLOW}{BOLD}               ╲╱{RESET}\n\
             {WHITE}{BOLD}                ╲   ⚡ KRAKOOM{RESET}\n\
             {YELLOW}{BOLD}                 ╲{RESET}\n\
             {DIM}{YELLOW}                  ┊{RESET}\n\n\
             {DIM}  Lightning splits the bifurcation diagram clean in half{RESET}"
        ),
        SoundType::Alarm => format!(
            "\n  {RED}{BOLD}  ▐██████████████████████████████████████▌{RESET}\n\
             {WHITE}{BOLD}  ▐█  ░░ {RED}▓▓ {WHITE}░░ {RED}▓▓ {WHITE}░░ ALERT ░░ {RED}▓▓ {WHITE}░░ {RED}▓▓ {WHITE}░░ █▌{RESET}\n\
             {RED}{BOLD}  ▐██████████████████████████████████████▌{RESET}\n\
             {WHITE}{BOLD}  ▐██████████████████████████████████████▌{RESET}\n\
             {RED}{BOLD}  ▐█  ▓▓ {WHITE}░░ {RED}▓▓ {WHITE}░░ {RED}▓▓ ALERT ▓▓ {WHITE}░░ {RED}▓▓ {WHITE}░░ {RED}▓▓ █▌{RESET}\n\
             {WHITE}{BOLD}  ▐██████████████████████████████████████▌{RESET}\n\n\
             {DIM}  The chaos engine redlines — all governors blown{RESET}"
        ),
        SoundType::Roar => format!(
            "\n  {RED}{BOLD}          ╭──────────────────╮{RESET}\n\
             {ORANGE}{BOLD}         ╱ ▲  ▲  ▲  ▲  ▲  ▲ ╲{RESET}\n\
             {RED}{BOLD}        ╱                      ╲{RESET}\n\
             {ORANGE}{BOLD}       │   ●              ●    │{RESET}\n\
             {RED}{BOLD}       │          ◆◆           │{RESET}\n\
             {ORANGE}{BOLD}       │     ╲____________╱    │{RESET}\n\
             {RED}{BOLD}        ╲  ▼  ▼  ▼  ▼  ▼  ▼  ╱{RESET}\n\
             {ORANGE}{BOLD}         ╰──────────────────╯{RESET}\n\
             {RED}{BOLD}          R  A  W  W  W  R{RESET}\n\n\
             {DIM}  Something ancient stirs in the attractor's core{RESET}"
        ),
        SoundType::Bell => format!(
            "\n  {GOLD}                   ╱╲{RESET}\n\
             {GOLD}                  ╱  ╲{RESET}\n\
             {GOLD}                 ╱    ╲{RESET}\n\
             {GOLD}{BOLD}                │ ░░░░ │{RESET}\n\
             {GOLD}{BOLD}                │ ▓▓▓▓ │{RESET}\n\
             {GOLD}{BOLD}               ╱ ██{WHITE}◉{GOLD}██ ╲{RESET}\n\
             {GOLD}{BOLD}              ╱________╲{RESET}\n\
             {YELLOW}          ─ ─ ─ {BOLD}D I N G{RESET}{YELLOW} ─ ─ ─{RESET}\n\
             {DIM}{YELLOW}            ∿  ∿  ∿  ∿  ∿  ∿{RESET}\n\n\
             {DIM}  A crystalline bell chime echoes from the attractor core{RESET}"
        ),
        SoundType::Guitar => format!(
            "\n  {ORANGE}{BOLD}              ┌─────┐{RESET}\n\
             {ORANGE}              │ ○ ○ ○│{RESET}\n\
             {ORANGE}              │     ╱│{RESET}\n\
             {ORANGE}              │    ╱ │{RESET}\n\
             {ORANGE}              │   ╱  │{RESET}\n\
             {ORANGE}{BOLD}              │  ╱   │{RESET}\n\
             {ORANGE}{BOLD}              │ ┃ ◎  │{RESET}\n\
             {ORANGE}{BOLD}              │ ┃    │{RESET}\n\
             {ORANGE}              └──┃──┘{RESET}\n\
             {YELLOW}        ∼∼∼∿∿∿{BOLD} TWANG {RESET}{YELLOW}∿∿∿∼∼∼{RESET}\n\n\
             {DIM}  A cosmic string vibrates at the Feigenbaum frequency{RESET}"
        ),
        SoundType::Drum => format!(
            "\n  {WHITE}{BOLD}       ╭─────────────────────────╮{RESET}\n\
             {WHITE}       │░░░░░░░░░░░░░░░░░░░░░░░░░│{RESET}\n\
             {WHITE}{BOLD}       │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│{RESET}\n\
             {WHITE}       ╰┬───────────────────────┬╯{RESET}\n\
             {WHITE}        │  {ORANGE}╱{WHITE}               {ORANGE}╲{WHITE}  │{RESET}\n\
             {WHITE}        │ {ORANGE}╱{WHITE}                 {ORANGE}╲{WHITE} │{RESET}\n\
             {WHITE}        ╰─────────────────────────╯{RESET}\n\n\
             {YELLOW}{BOLD}          BA{RESET}  {ORANGE}{BOLD}DUM{RESET}  {RED}{BOLD}TSS{RESET}  🥁\n\n\
             {DIM}  The chaos engine drops a rimshot — perfect timing{RESET}"
        ),
        SoundType::Wave => format!(
            "\n  {BLUE}{DIM}                                    ░░{RESET}\n\
             {BLUE}                                  ░░▒▒{RESET}\n\
             {SKY}                               ░░▒▒▓▓{RESET}\n\
             {CYAN}{BOLD}                    ╱╲       ░░▒▒▓▓██{RESET}\n\
             {SKY}{BOLD}               ╱╲╱  ╲╱╲  ░░▒▒▓▓████{RESET}\n\
             {BLUE}{BOLD}          ╱╲╱╲╱       ╲╱▒▒▓▓████████{RESET}\n\
             {CYAN}  ~~~~~~╱                ▓▓██████████████{RESET}\n\
             {DIM}{BLUE}  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓████████████████████{RESET}\n\n\
             {CYAN}{BOLD}               C R A S H H H{RESET}\n\n\
             {DIM}  A wave of entropy crashes against the system's shore{RESET}"
        ),
        SoundType::Chime => format!(
            "\n  {SKY}{DIM}            ═══════════════════{RESET}\n\
             {SKY}              │   │   │   │{RESET}\n\
             {WHITE}              ┃   ╽   ╿   ┃{RESET}\n\
             {SKY}              ┃   ╽   ╿   ┃{RESET}\n\
             {WHITE}              ○   ◇   ○   ◇{RESET}\n\n\
             {SKY}       ✧  ✧  ✧ {BOLD}CHIME{RESET}{SKY} ✧  ✧  ✧{RESET}\n\
             {DIM}{SKY}         ∿    ∿    ∿    ∿    ∿{RESET}\n\n\
             {DIM}  Windchimes stir in the entropy breeze{RESET}"
        ),
        SoundType::Piano => format!(
            "\n  {WHITE}{BOLD}  ┌─┬─┬┬─┬─┬─┬─┬┬─┬┬─┬─┬─┬─┬┬─┬─┐{RESET}\n\
             {WHITE}  │ │ │{DIM}█{RESET}{WHITE}│ │ │ │{DIM}█{RESET}{WHITE}│{DIM}█{RESET}{WHITE}│ │ │ │{DIM}█{RESET}{WHITE}│ │{RESET}\n\
             {WHITE}  │ │ │{DIM}█{RESET}{WHITE}│ │ │ │{DIM}█{RESET}{WHITE}│{DIM}█{RESET}{WHITE}│ │ │ │{DIM}█{RESET}{WHITE}│ │{RESET}\n\
             {WHITE}  │ └┬┘└┬┘ │ └┬┘└┬┘└┬┘ │ └┬┘└┬┘ │{RESET}\n\
             {WHITE}  │  │  │  │  │  │  │  │  │  │  │{RESET}\n\
             {WHITE}  │  │  │{YELLOW}{BOLD}▓▓{RESET}{WHITE}│  │  │  │  │  │  │{RESET}\n\
             {WHITE}  └──┴──┴──┴──┴──┴──┴──┴──┴──┴──┘{RESET}\n\n\
             {GOLD}{BOLD}              P L I N K{RESET}\n\
             {DIM}{GOLD}            ♩  ♪  ♫  ♬{RESET}\n\n\
             {DIM}  A single key pressed by the ghost of Edward Lorenz{RESET}"
        ),
        SoundType::Wind => format!(
            "\n  {SKY}{DIM}  ~ ~ ~ ~ ~ ~{RESET}\n\
             {WHITE}      ~ ~ ~ ~ ~ ~ ~ ~ ~{RESET}\n\
             {SKY}  ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~{RESET}\n\
             {CYAN}{BOLD}       ≈ ≈ ≈ WHOOSH ≈ ≈ ≈{RESET}\n\
             {SKY}  ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~{RESET}\n\
             {WHITE}      ~ ~ ~ ~ ~ ~ ~ ~ ~{RESET}\n\
             {SKY}{DIM}  ~ ~ ~ ~ ~ ~{RESET}\n\n\
             {DIM}  A cold front sweeps through the Lorenz orbital plane{RESET}"
        ),
        SoundType::Hum => format!(
            "\n  {MAGENTA}{DIM}  ─────────────────────────────────────{RESET}\n\
             {MAGENTA}     ╱╲   ╱╲   ╱╲   ╱╲   ╱╲   ╱╲{RESET}\n\
             {MAGENTA}{BOLD}    ╱  ╲ ╱  ╲ ╱  ╲ ╱  ╲ ╱  ╲ ╱  ╲{RESET}\n\
             {PINK}{BOLD}   ╱    ╳    ╳    ╳    ╳    ╳    ╲{RESET}\n\
             {MAGENTA}{BOLD}    ╲  ╱ ╲  ╱ ╲  ╱ ╲  ╱ ╲  ╱ ╲  ╱{RESET}\n\
             {MAGENTA}     ╲╱   ╲╱   ╲╱   ╲╱   ╲╱   ╲╱{RESET}\n\
             {MAGENTA}{DIM}  ─────────────────────────────────────{RESET}\n\n\
             {MAGENTA}{BOLD}          H  M  M  M  M  M{RESET}\n\n\
             {DIM}  A billion logistic map iterations per second — pure resonance{RESET}"
        ),
    }
}

#[async_trait]
impl Skill for SoundSkill {
    fn name(&self) -> &str {
        "sound"
    }
    fn description(&self) -> &str {
        "Chaos-reactive sound effect with visual + sox audio"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Mechanical
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let sound = pick_sound(ctx.chaos);
        let visual = render_visual(sound);
        let category = sound.to_category();

        // Play audio in background
        play_audio(sound);

        // Build display
        let phase_str = match ctx.chaos.phase {
            Phase::Idle => "Idle",
            Phase::Build => "Build",
            Phase::Drop => "Drop",
        };
        let display = format!(
            "\n{DIM}┌─────────────────────────────────────────────────┐{RESET}\n\
             {BOLD}{YELLOW}  🔊 SOUND EFFECT{RESET}  {DIM}[{}]{RESET}\n\
             {DIM}├─────────────────────────────────────────────────┤{RESET}\n\
             {visual}\n\n\
             {DIM}  ⚙ Tension:{:.0}% Energy:{:.0} Phase:{}{RESET}\n\
             {DIM}└─────────────────────────────────────────────────┘{RESET}\n",
            sound.name(),
            ctx.chaos.tension,
            ctx.chaos.energy,
            phase_str,
        );

        let feedback_event = ChaosEvent::SoundFired { category };
        let _ = ctx.feedback_tx.send(feedback_event.clone()).await;

        Ok(SkillOutput {
            display,
            feedback: vec![feedback_event],
            inject_to_conversation: true,
            evidence: None,
        })
    }
}
