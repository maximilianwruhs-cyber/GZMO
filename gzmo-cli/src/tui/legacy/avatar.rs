//! Persistent avatar for the TUI sidebar — friendly talking pixel cogwheel.
//!
//! Half-block truecolor sprite stays on stage for the whole session.
//! Mood from chaos valence; mouth/spin react to listening, speaking, skills.

use color_eyre::Result;
use gzmo_chaos::chaos::Phase;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::action::Action;
use crate::tui::legacy::avatar_art::{
    bulb_row, theatrical_line, MoodBand, Performance, GOLD, RUBY,
};
use crate::tui::legacy::avatar_pixel::{pose_from_state, render_cog_lines};
use crate::tui::component::Component;

const SPEAKING_DECAY: u32 = 30;
const LISTENING_HOLD: u32 = u32::MAX;
const WORKING_DECAY: u32 = 90;
const ALERT_DECAY: u32 = 75;
const REBIRTH_DECAY: u32 = 90;

const SPEAK_STEP: u32 = 4;
const IDLE_STEP: u32 = 8;
const BLINK_EVERY: u32 = 140;
const BLINK_LEN: u32 = 8;

pub struct AvatarComponent {
    valence: f32,
    tension: f64,
    energy: f64,
    lorenz_x: f64,
    phase: Phase,
    alive: bool,
    was_alive: bool,

    performance: Performance,
    speaking_decay: u32,
    listening_decay: u32,
    working_decay: u32,
    alert_decay: u32,
    rebirth_decay: u32,

    anim_frame: u8,
    tick_accum: u32,
    blink_ticks: u32,
}

impl AvatarComponent {
    pub fn new() -> Self {
        Self {
            valence: 0.0,
            tension: 0.0,
            energy: 100.0,
            lorenz_x: 0.0,
            phase: Phase::Idle,
            alive: true,
            was_alive: true,
            performance: Performance::Idle,
            speaking_decay: 0,
            listening_decay: 0,
            working_decay: 0,
            alert_decay: 0,
            rebirth_decay: 0,
            anim_frame: 0,
            tick_accum: 0,
            blink_ticks: 0,
        }
    }

    fn mood(&self) -> MoodBand {
        MoodBand::from_valence(self.valence)
    }

    fn recompute_performance(&mut self) {
        if !self.alive {
            self.performance = Performance::Dead;
            return;
        }
        if self.rebirth_decay > 0 {
            self.performance = Performance::Rebirth;
            return;
        }
        if self.alert_decay > 0 {
            self.performance = Performance::Alert;
            return;
        }
        if self.speaking_decay > 0 {
            self.performance = Performance::Speaking;
            return;
        }
        if self.working_decay > 0 {
            self.performance = Performance::Working;
            return;
        }
        if self.listening_decay > 0 {
            self.performance = Performance::Listening;
            return;
        }
        self.performance = Performance::Idle;
    }

    fn anim_rate(&self) -> u32 {
        match self.performance {
            Performance::Speaking
            | Performance::Working
            | Performance::Alert
            | Performance::Rebirth => SPEAK_STEP,
            Performance::Dead => IDLE_STEP * 3,
            Performance::Listening => IDLE_STEP,
            Performance::Idle => {
                if matches!(self.phase, Phase::Drop) || self.tension > 70.0 {
                    SPEAK_STEP
                } else if self.energy < 25.0 {
                    IDLE_STEP * 2
                } else {
                    IDLE_STEP
                }
            }
        }
    }

    fn blinking(&self) -> bool {
        matches!(self.performance, Performance::Idle | Performance::Listening)
            && self.blink_ticks > 0
            && self.blink_ticks <= BLINK_LEN
    }

    fn build_lines(&self, inner_w: u16, inner_h: u16) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let mood = self.mood();
        let blinking = self.blinking();
        let pose = pose_from_state(
            mood,
            self.performance,
            self.tension,
            self.energy,
            self.lorenz_x,
            self.anim_frame,
            blinking,
        );

        if inner_h < 6 || inner_w < 16 {
            // Emergency tiny glyph
            lines.push(Line::from(Span::styled(
                "⚙",
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            )));
            return lines;
        }

        let hot = self.tension > 70.0
            || matches!(
                self.performance,
                Performance::Speaking | Performance::Alert | Performance::Rebirth
            );

        if inner_h >= 18 {
            let bulb_w = (inner_w as usize).saturating_sub(2);
            lines.push(Line::from(Span::styled(
                bulb_row(bulb_w, self.anim_frame, hot),
                Style::default().fg(if hot { RUBY } else { GOLD }),
            )));
        }

        let mut cog = render_cog_lines(&pose);
        // If pane is narrower than 32, trim; if shorter, take top rows.
        let max_w = inner_w as usize;
        if max_w < 32 {
            for line in &mut cog {
                let mut spans = std::mem::take(&mut line.spans);
                spans.truncate(max_w);
                line.spans = spans;
            }
        }
        let caption_reserve = 1usize;
        let bulb_reserve = if inner_h >= 18 { 1 } else { 0 };
        let avail = (inner_h as usize).saturating_sub(caption_reserve + bulb_reserve);
        if cog.len() > avail {
            cog.truncate(avail);
        }
        lines.extend(cog);

        lines.push(Line::from(Span::styled(
            theatrical_line(self.performance, self.phase, mood, self.tension).to_string(),
            Style::default().fg(GOLD).add_modifier(Modifier::ITALIC),
        )));

        let max = inner_h as usize;
        if lines.len() > max {
            lines.truncate(max);
        }
        lines
    }

    fn title(&self) -> &'static str {
        match self.performance {
            Performance::Speaking => " ⚙ GZMO · SPEAKING ",
            Performance::Listening => " ⚙ GZMO · LISTENING ",
            Performance::Working => " ⚙ GZMO · WORKING ",
            Performance::Alert => " ⚙ GZMO · ALERT ",
            Performance::Dead => " ⚙ GZMO · FALLEN ",
            Performance::Rebirth => " ⚙ GZMO · REBIRTH ",
            Performance::Idle => match self.phase {
                Phase::Drop => " ⚙ GZMO · DROP ",
                Phase::Build => " ⚙ GZMO · BUILD ",
                Phase::Idle => " ⚙ GZMO ",
            },
        }
    }
}

impl Default for AvatarComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for AvatarComponent {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::ChaosSnapshot(snap) => {
                if !self.was_alive && snap.alive {
                    self.rebirth_decay = REBIRTH_DECAY;
                }
                self.was_alive = snap.alive;
                self.alive = snap.alive;
                self.valence = snap.llm_valence;
                self.tension = snap.tension;
                self.energy = snap.energy;
                self.lorenz_x = snap.x;
                self.phase = snap.phase;
                self.recompute_performance();
            }
            Action::SubmitInput(_) => {
                if self.alive {
                    self.listening_decay = LISTENING_HOLD;
                    self.speaking_decay = 0;
                    self.recompute_performance();
                }
            }
            Action::AgentTokenStream(_) => {
                self.speaking_decay = SPEAKING_DECAY;
                self.listening_decay = 0;
                self.recompute_performance();
            }
            Action::AgentResponse(_) => {
                self.speaking_decay = SPEAKING_DECAY / 2;
                self.listening_decay = 0;
                self.recompute_performance();
            }
            Action::TriggerSkill(_, _) => {
                self.working_decay = WORKING_DECAY;
                self.recompute_performance();
            }
            Action::TriggerNotification(_) | Action::LoreEvent(_, _, _) => {
                self.alert_decay = ALERT_DECAY;
                self.recompute_performance();
            }
            Action::Tick => {
                self.tick_accum = self.tick_accum.wrapping_add(1);
                if self.tick_accum.is_multiple_of(self.anim_rate()) {
                    self.anim_frame = self.anim_frame.wrapping_add(1);
                }

                self.blink_ticks = self.blink_ticks.wrapping_add(1);
                if self.blink_ticks >= BLINK_EVERY + BLINK_LEN {
                    self.blink_ticks = 0;
                }

                let mut dirty = false;
                for slot in [
                    &mut self.speaking_decay,
                    &mut self.working_decay,
                    &mut self.alert_decay,
                    &mut self.rebirth_decay,
                ] {
                    if *slot > 0 {
                        *slot -= 1;
                        dirty = true;
                    }
                }
                if dirty || !self.alive {
                    self.recompute_performance();
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let border = if self.tension > 80.0 || matches!(self.performance, Performance::Alert) {
            RUBY
        } else if matches!(self.performance, Performance::Speaking) {
            GOLD
        } else {
            Color::Rgb(60, 60, 70)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title())
            .title_style(Style::default().fg(GOLD).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(border));

        let inner = block.inner(area);
        let content = self.build_lines(inner.width, inner.height);
        let p = Paragraph::new(content)
            .alignment(Alignment::Center)
            .block(block);

        f.render_widget(p, area);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::action::Action;
    use gzmo_chaos::pulse::ChaosSnapshot;

    fn snap(alive: bool, valence: f32, phase: Phase, tension: f64) -> ChaosSnapshot {
        ChaosSnapshot {
            alive,
            llm_valence: valence,
            phase,
            tension,
            energy: 80.0,
            x: 0.0,
            ..Default::default()
        }
    }

    #[test]
    fn submit_then_stream_transitions() {
        let mut av = AvatarComponent::new();
        av.update(Action::ChaosSnapshot(snap(true, 0.2, Phase::Idle, 10.0)))
            .unwrap();
        av.update(Action::SubmitInput("hello".into())).unwrap();
        assert_eq!(av.performance, Performance::Listening);
        for _ in 0..200 {
            av.update(Action::Tick).unwrap();
        }
        assert_eq!(av.performance, Performance::Listening);
        av.update(Action::AgentTokenStream("Hi".into())).unwrap();
        assert_eq!(av.performance, Performance::Speaking);
    }

    #[test]
    fn build_lines_includes_halfblocks() {
        let av = AvatarComponent::new();
        let lines = av.build_lines(36, 20);
        let text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();
        assert!(text.contains('▀') || text.contains('▄') || text.contains('⚙'));
    }

    #[test]
    fn death_and_rebirth() {
        let mut av = AvatarComponent::new();
        av.update(Action::ChaosSnapshot(snap(true, 0.0, Phase::Idle, 10.0)))
            .unwrap();
        av.update(Action::ChaosSnapshot(snap(false, 0.0, Phase::Drop, 90.0)))
            .unwrap();
        assert_eq!(av.performance, Performance::Dead);
        av.update(Action::ChaosSnapshot(snap(true, 0.0, Phase::Idle, 20.0)))
            .unwrap();
        assert_eq!(av.performance, Performance::Rebirth);
    }
}
