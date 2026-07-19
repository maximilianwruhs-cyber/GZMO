//! # Poker Skill — `/poker`
//!
//! Deals a chaos-driven 5-card poker hand from a full 52-card deck.
//! Hand evaluation is pure Rust — no shell, no LLM.

use anyhow::Result;
use async_trait::async_trait;

use gzmo_chaos::feedback::ChaosEvent;

use super::{Skill, SkillContext, SkillOutput, SkillType};

const WHITE: &str = "\x1b[97m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

const SUITS: [&str; 4] = ["♠", "♥", "♦", "♣"];
const SUIT_COLORS: [&str; 4] = [WHITE, RED, RED, WHITE];
const RANKS: [&str; 13] = [
    "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A",
];

pub struct PokerSkill;

#[derive(Clone)]
struct Card {
    rank_idx: usize,
    suit_idx: usize,
}

impl Card {
    fn value(&self) -> u8 {
        (self.rank_idx + 2) as u8
    }
    fn display(&self) -> String {
        format!(
            "{}{}{}{}",
            SUIT_COLORS[self.suit_idx], RANKS[self.rank_idx], SUITS[self.suit_idx], RESET
        )
    }
}

fn deal_hand(snap: &gzmo_chaos::pulse::ChaosSnapshot) -> Vec<Card> {
    let mut deck: Vec<Card> = (0..52)
        .map(|i| Card {
            rank_idx: i % 13,
            suit_idx: i / 13,
        })
        .collect();

    // Fisher-Yates shuffle using chaos values
    let mut entropy = snap.chaos_val * 10000.0 + snap.x.abs() * 100.0 + snap.y.abs();
    for i in (1..52).rev() {
        entropy = (entropy * 1.618033988 + snap.z.abs()).fract() * 10000.0;
        let j = (entropy as usize) % (i + 1);
        deck.swap(i, j);
    }

    deck[..5].to_vec()
}

fn evaluate_hand(hand: &[Card]) -> (&str, &str) {
    let mut values: Vec<u8> = hand.iter().map(|c| c.value()).collect();
    values.sort();

    let is_flush = hand.iter().all(|c| c.suit_idx == hand[0].suit_idx);
    let is_straight = values.windows(2).all(|w| w[1] - w[0] == 1) || values == vec![2, 3, 4, 5, 14]; // Ace-low

    // Count occurrences
    let mut counts = [0u8; 15];
    for &v in &values {
        counts[v as usize] += 1;
    }
    let mut sorted_counts: Vec<u8> = counts.iter().filter(|&&c| c > 0).copied().collect();
    sorted_counts.sort_unstable_by(|a, b| b.cmp(a));

    match (is_flush, is_straight, sorted_counts.as_slice()) {
        (true, true, _) if values[0] == 10 => ("ROYAL FLUSH", "👑"),
        (true, true, _) => ("STRAIGHT FLUSH", "🌟"),
        (_, _, [4, ..]) => ("FOUR OF A KIND", "💎"),
        (_, _, [3, 2, ..]) => ("FULL HOUSE", "🏠"),
        (true, _, _) => ("FLUSH", "♦️"),
        (_, true, _) => ("STRAIGHT", "📏"),
        (_, _, [3, ..]) => ("THREE OF A KIND", "🎯"),
        (_, _, [2, 2, ..]) => ("TWO PAIR", "✌️"),
        (_, _, [2, ..]) => ("ONE PAIR", "👫"),
        _ => ("HIGH CARD", "🃏"),
    }
}

#[async_trait]
impl Skill for PokerSkill {
    fn name(&self) -> &str {
        "poker"
    }
    fn description(&self) -> &str {
        "Deal a chaos-driven 5-card poker hand"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Mechanical
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let hand = deal_hand(ctx.chaos);
        let (name, icon) = evaluate_hand(&hand);
        let cards_display: Vec<String> = hand.iter().map(|c| c.display()).collect();

        let display = format!(
            "\n{DIM}┌─────────────────────────────────────────────────┐{RESET}\n\
             {BOLD}{CYAN}  🃏 POKER HAND{RESET}\n\
             {DIM}├─────────────────────────────────────────────────┤{RESET}\n\n\
                  {}  {}  {}  {}  {}\n\n\
             {BOLD}{YELLOW}  {icon} {name}{RESET}\n\n\
             {DIM}└─────────────────────────────────────────────────┘{RESET}\n",
            cards_display[0],
            cards_display[1],
            cards_display[2],
            cards_display[3],
            cards_display[4],
        );

        // Strong hands generate feedback
        let feedback = match name {
            "ROYAL FLUSH" | "STRAIGHT FLUSH" | "FOUR OF A KIND" => {
                let ev = ChaosEvent::DiceRoll { value: 20, max: 20 }; // Treat like nat 20
                let _ = ctx.feedback_tx.send(ev.clone()).await;
                vec![ev]
            }
            "FULL HOUSE" | "FLUSH" | "STRAIGHT" => {
                let ev = ChaosEvent::DiceRoll { value: 15, max: 20 };
                let _ = ctx.feedback_tx.send(ev.clone()).await;
                vec![ev]
            }
            _ => vec![],
        };

        Ok(SkillOutput {
            display,
            feedback,
            inject_to_conversation: true,
            evidence: None,
        })
    }
}
