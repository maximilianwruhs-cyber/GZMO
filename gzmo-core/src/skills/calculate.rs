//! # Calculate Skill — `/calculate`
//!
//! Deterministic math via GNU `bc -l`. The numeric result is exact for a given
//! expression; chaos coupling affects display flavor and organism feedback only.
//!
//! CCL-2: tick footer, inv #N, chaos-indexed commentary, `ChaosEvent::Custom`.

use anyhow::Result;
use async_trait::async_trait;

use gzmo_chaos::chaos::Phase;
use gzmo_chaos::feedback::ChaosEvent;
use gzmo_chaos::feedback_ipc::event_to_json_value;
use gzmo_chaos::pulse::ChaosSnapshot;

use super::attractor_common::next_call_serial;
use super::{Skill, SkillContext, SkillOutput, SkillType};

/// One step in a Schritt-fuer-Schritt decomposition (v2 display).
#[derive(Debug, Clone, serde::Serialize)]
pub struct CalcStep {
    pub label_de: String,
    pub expr: String,
    pub partial: String,
}

const WHITE: &str = "\x1b[97m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const RED: &str = "\x1b[31m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

const FLAVOR: [&str; 8] = [
    "The attractor computed this without consulting the oracle.",
    "Feigenbaum would approve the precision.",
    "Zero butterfly effect on the numeric manifold.",
    "A clean eigenvalue emerges from the expression.",
    "The logistic map converges — for this input.",
    "Fixed-point arithmetic in a chaotic phase space.",
    "Lyapunov exponent: negligible. Result: exact.",
    "The Lorenz variable held still long enough to finish.",
];

pub struct CalculateSkill;

#[async_trait]
impl Skill for CalculateSkill {
    fn name(&self) -> &str {
        "calculate"
    }
    fn description(&self) -> &str {
        "Evaluate a math expression via bc (chaos-indexed frame, deterministic result)"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Mechanical
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let expr = parse_expression(ctx.args);
        if expr.is_empty() {
            return Ok(SkillOutput::new(
                format!(
                    "  {RED}✗ Usage: /calculate <expression>{RESET}\n\
                       Examples: /calculate 2^10\n\
                                 /calculate \"sqrt(144)\"\n\
                                 /calculate \"3.14 * 42^2\""
                ),
                vec![],
                false,
            ));
        }

        let normalized = normalize_expr(&expr);
        let result = match eval_via_bc(&normalized) {
            Some(r) => trim_result(&r),
            None => {
                return Ok(SkillOutput::new(
                    format!("  {RED}✗ Invalid expression: {expr}{RESET}"),
                    vec![],
                    false,
                ));
            }
        };

        let inv = next_call_serial(&ctx.skills_dir.join(".calculate_inv"))
            .unwrap_or(ctx.chaos.tick);
        let flavor = pick_flavor(ctx.chaos);
        let label = magnitude_label(&result);
        let steps = decompose_steps(&normalized);
        let interpretation = magnitude_interpretation_de(label);

        let feedback_event = calculate_mechanical_feedback(&result, ctx.chaos);
        let _ = ctx.feedback_tx.send(feedback_event.clone()).await;
        let feedback = vec![feedback_event.clone()];

        let display = format_display_v2(
            &expr,
            &result,
            flavor,
            label,
            inv,
            ctx.chaos,
            &steps,
            &interpretation,
        );

        let evidence = build_calculate_evidence(
            &expr,
            &normalized,
            &result,
            flavor,
            label,
            inv,
            ctx.chaos,
            &feedback,
            &steps,
            &interpretation,
        );

        Ok(SkillOutput {
            display,
            feedback,
            inject_to_conversation: true,
            evidence: Some(evidence),
        })
    }
}

/// Strip `--json` and surrounding quotes from skill args.
pub fn parse_expression(args: &str) -> String {
    let cleaned: Vec<&str> = args
        .split_whitespace()
        .filter(|t| *t != "--json")
        .collect();
    let joined = cleaned.join(" ");
    let trimmed = joined.trim();
    if trimmed.len() >= 2 {
        let bytes = trimmed.as_bytes();
        if (bytes[0] == b'"' && bytes[trimmed.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[trimmed.len() - 1] == b'\'')
        {
            return trimmed[1..trimmed.len() - 1].to_string();
        }
    }
    trimmed.to_string()
}

/// Normalize common operator aliases for bc.
pub fn normalize_expr(expr: &str) -> String {
    expr.replace("**", "^")
        .replace('×', "*")
        .replace('÷', "/")
        .replace('−', "-")
}

/// Evaluate via GNU bc with math library. Returns None on failure.
pub fn eval_via_bc(expr: &str) -> Option<String> {
    use std::io::Write;

    let mut child = std::process::Command::new("bc")
        .arg("-l")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .ok()?;
    if let Some(ref mut stdin) = child.stdin {
        let _ = write!(stdin, "{expr}\n");
    }
    let output = child.wait_with_output().ok()?;

    if !output.status.success() {
        return None;
    }
    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

/// Trim trailing zeros from bc decimal output.
pub fn trim_result(raw: &str) -> String {
    if raw.contains('.') {
        raw.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        raw.to_string()
    }
}

fn chaos_index(snap: &ChaosSnapshot, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    ((snap.chaos_val * 10000.0 + snap.x.abs() * 100.0 + snap.y.abs()) as usize) % len
}

pub fn pick_flavor(snap: &ChaosSnapshot) -> &'static str {
    FLAVOR[chaos_index(snap, FLAVOR.len())]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagnitudeLabel {
    Integer,
    Fractional,
    Huge,
    Tiny,
}

impl MagnitudeLabel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Integer => "INTEGER",
            Self::Fractional => "FRACTIONAL",
            Self::Huge => "HUGE",
            Self::Tiny => "TINY",
        }
    }
}

pub fn magnitude_label(result: &str) -> MagnitudeLabel {
    let Ok(v) = result.parse::<f64>() else {
        return MagnitudeLabel::Fractional;
    };
    let abs = v.abs();
    if abs == 0.0 {
        return MagnitudeLabel::Integer;
    }
    if abs >= 1e12 {
        MagnitudeLabel::Huge
    } else if abs > 0.0 && abs < 1e-6 {
        MagnitudeLabel::Tiny
    } else if abs.fract() == 0.0 {
        MagnitudeLabel::Integer
    } else {
        MagnitudeLabel::Fractional
    }
}

pub fn calculate_mechanical_feedback(result: &str, snap: &ChaosSnapshot) -> ChaosEvent {
    let magnitude = result
        .parse::<f64>()
        .map(|v| v.abs().log10().max(0.0))
        .unwrap_or(1.0);
    let chaos_boost = snap.chaos_val as f64 * 0.5;
    ChaosEvent::Custom {
        tension_delta: (magnitude * 0.3 + chaos_boost).min(5.0),
        energy_delta: -0.25,
        thought_seed: None,
    }
}

fn phase_str(phase: Phase) -> &'static str {
    match phase {
        Phase::Idle => "Idle",
        Phase::Build => "Build",
        Phase::Drop => "Drop",
    }
}

pub fn magnitude_interpretation_de(label: MagnitudeLabel) -> &'static str {
    match label {
        MagnitudeLabel::Integer => "Ganzzahl — exaktes Ergebnis ohne Nachkommastellen.",
        MagnitudeLabel::Fractional => "Bruchzahl — Dezimalanteil vorhanden.",
        MagnitudeLabel::Huge => "Sehr gross — Betrag ab 10^12.",
        MagnitudeLabel::Tiny => "Sehr klein — Betrag unter 10^-6.",
    }
}

fn needs_decomposition(expr: &str) -> bool {
    let ops = expr
        .chars()
        .filter(|c| matches!(c, '+' | '-' | '*' | '/' | '^'))
        .count();
    ops > 0 || expr.contains('(')
}

fn find_innermost_parens(s: &str) -> Option<(usize, usize)> {
    let mut depth = 0i32;
    let mut start = None;
    for (i, c) in s.char_indices() {
        match c {
            '(' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            ')' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(st) = start {
                        return Some((st, i));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn find_next_op(s: &str, ops: &[char]) -> Option<(usize, char)> {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ if depth == 0 && ops.contains(&c) => {
                if c == '-' && i == 0 {
                    continue;
                }
                if c == '-' {
                    let prev = s[..i].trim_end().chars().last();
                    if prev.is_none()
                        || prev == Some('(')
                        || prev == Some('+')
                        || prev == Some('-')
                        || prev == Some('*')
                        || prev == Some('/')
                    {
                        continue;
                    }
                }
                return Some((i, c));
            }
            _ => {}
        }
    }
    None
}

fn token_span_left(s: &str, op_idx: usize) -> usize {
    let left = s[..op_idx].trim_end();
    if let Some(i) = left.rfind(|c: char| matches!(c, '+' | '-' | '*' | '/')) {
        if left.as_bytes().get(i) == Some(&b'-') && i > 0 {
            let prev = left.as_bytes().get(i - 1).copied();
            if prev == Some(b'(') || prev == Some(b'+') || prev == Some(b'-') || prev == Some(b'*') || prev == Some(b'/') {
                return i;
            }
        }
        i + 1
    } else {
        0
    }
}

fn token_span_right(s: &str, op_idx: usize) -> usize {
    let rest = s[op_idx + 1..].trim_start();
    let base = op_idx + 1 + (s[op_idx + 1..].len() - rest.len());
    if let Some(rel) = rest.find(|c: char| matches!(c, '+' | '-' | '*' | '/')) {
        base + rel
    } else {
        s.len()
    }
}

fn eval_binary_op(s: &str, op_idx: usize, op: char) -> Option<(String, String, String)> {
    let left_start = token_span_left(s, op_idx);
    let right_end = token_span_right(s, op_idx);
    let left = s[left_start..op_idx].trim().to_string();
    let right = s[op_idx + 1..right_end].trim().to_string();
    if left.is_empty() || right.is_empty() {
        return None;
    }
    let expr = format!("{left} {op} {right}");
    let partial = trim_result(&eval_via_bc(&expr)?);
    Some((left, right, partial))
}

/// Decompose compound expressions into bc-evaluated steps (no LLM).
pub fn decompose_steps(normalized: &str) -> Vec<CalcStep> {
    let trimmed = normalized.trim();
    if trimmed.is_empty() || !needs_decomposition(trimmed) {
        return Vec::new();
    }

    let mut steps = Vec::new();
    let mut step_n = 1u32;
    let mut current = trimmed.to_string();

    while let Some((start, end)) = find_innermost_parens(&current) {
        let inner = &current[start + 1..end];
        if let Some(partial) = eval_via_bc(inner).map(|r| trim_result(&r)) {
            steps.push(CalcStep {
                label_de: format!("Schritt {step_n} (Klammer)"),
                expr: inner.to_string(),
                partial: partial.clone(),
            });
            step_n += 1;
            current.replace_range(start..=end, &partial);
        } else {
            break;
        }
    }

    while let Some((op_idx, op)) = find_next_op(&current, &['*', '/']) {
        if let Some((left, right, partial)) = eval_binary_op(&current, op_idx, op) {
            steps.push(CalcStep {
                label_de: format!("Schritt {step_n}"),
                expr: format!("{left} {op} {right}"),
                partial: partial.clone(),
            });
            step_n += 1;
            let left_start = token_span_left(&current, op_idx);
            let right_end = token_span_right(&current, op_idx);
            current.replace_range(left_start..right_end, &partial);
        } else {
            break;
        }
    }

    while let Some((op_idx, op)) = find_next_op(&current, &['+', '-']) {
        if let Some((left, right, partial)) = eval_binary_op(&current, op_idx, op) {
            steps.push(CalcStep {
                label_de: format!("Schritt {step_n}"),
                expr: format!("{left} {op} {right}"),
                partial: partial.clone(),
            });
            step_n += 1;
            let left_start = token_span_left(&current, op_idx);
            let right_end = token_span_right(&current, op_idx);
            current.replace_range(left_start..right_end, &partial);
        } else {
            break;
        }
    }

    steps
}

fn format_display_v2(
    expr: &str,
    result: &str,
    flavor: &str,
    label: MagnitudeLabel,
    inv: u64,
    snap: &ChaosSnapshot,
    steps: &[CalcStep],
    interpretation: &str,
) -> String {
    let steps_block = if steps.is_empty() {
        String::new()
    } else {
        let mut block = format!("\n{DIM}  Schritt-fuer-Schritt:{RESET}\n");
        for st in steps {
            block.push_str(&format!(
                "    {DIM}{}{RESET}  {BOLD}{}{RESET} {DIM}->{RESET} {WHITE}{}{RESET}\n",
                st.label_de, st.expr, st.partial
            ));
        }
        block.push_str(&format!(
            "    {DIM}Endergebnis{RESET}  {BOLD}{WHITE}{result}{RESET}\n"
        ));
        block
    };

    format!(
        "\n{DIM}┌─────────────────────────────────────────────────┐{RESET}\n\
         {BOLD}{GREEN}  CALCULATE{RESET}  {DIM}inv #{inv}{RESET}\n\
         {DIM}├─────────────────────────────────────────────────┤{RESET}\n\n\
           {DIM}expr: {expr}{RESET}\n\
           {BOLD}{WHITE}  =  {result}{RESET}  {CYAN}[{label}]{RESET}\n\
           {DIM}  {interpretation}{RESET}\n\
         {steps_block}\
           {DIM}  {flavor}{RESET}\n\n\
         {DIM}  tick:{tick} phase:{phase} chaos:{chaos:.2}{RESET}\n\
         {DIM}└─────────────────────────────────────────────────┘{RESET}\n",
        label = label.as_str(),
        tick = snap.tick,
        phase = phase_str(snap.phase),
        chaos = snap.chaos_val,
    )
}

#[allow(dead_code)]
fn format_display(
    expr: &str,
    result: &str,
    flavor: &str,
    label: MagnitudeLabel,
    inv: u64,
    snap: &ChaosSnapshot,
) -> String {
    format!(
        "\n{DIM}┌─────────────────────────────────────────────────┐{RESET}\n\
         {BOLD}{GREEN}  🧮 CALCULATE{RESET}  {DIM}inv #{inv}{RESET}\n\
         {DIM}├─────────────────────────────────────────────────┤{RESET}\n\n\
           {DIM}expr: {expr}{RESET}\n\
           {BOLD}{WHITE}  =  {result}{RESET}  {CYAN}[{label}]{RESET}\n\n\
           {DIM}  {flavor}{RESET}\n\n\
         {DIM}  ⚙ tick:{tick} phase:{phase} chaos:{chaos:.2}{RESET}\n\
         {DIM}└─────────────────────────────────────────────────┘{RESET}\n",
        label = label.as_str(),
        tick = snap.tick,
        phase = phase_str(snap.phase),
        chaos = snap.chaos_val,
    )
}

pub fn build_calculate_evidence(
    expr: &str,
    normalized: &str,
    result: &str,
    flavor: &str,
    label: MagnitudeLabel,
    inv: u64,
    snap: &ChaosSnapshot,
    feedback: &[ChaosEvent],
    steps: &[CalcStep],
    interpretation: &str,
) -> serde_json::Value {
    serde_json::json!({
        "skill": "calculate",
        "version": 2,
        "inv": inv,
        "expr": expr,
        "normalized": normalized,
        "result": result,
        "label": label.as_str(),
        "interpretation": interpretation,
        "steps": steps,
        "flavor": flavor,
        "chaos": {
            "tick": snap.tick,
            "phase": format!("{}", snap.phase),
            "chaos_val": snap.chaos_val,
            "energy": snap.energy,
            "tension": snap.tension,
            "rho_effective": snap.rho_effective,
        },
        "feedback": feedback.iter().map(event_to_json_value).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gzmo_chaos::chaos::Phase;

    fn test_snap(tick: u64, x: f64, y: f64) -> ChaosSnapshot {
        ChaosSnapshot {
            tick,
            x,
            y,
            z: 0.0,
            chaos_val: 0.5,
            energy: 100.0,
            tension: 0.0,
            phase: Phase::Idle,
            ..Default::default()
        }
    }

    #[test]
    fn parse_expression_strips_json_flag() {
        assert_eq!(parse_expression("--json 2+2"), "2+2");
        assert_eq!(parse_expression("sqrt(144) --json"), "sqrt(144)");
    }

    #[test]
    fn parse_expression_unquotes() {
        assert_eq!(parse_expression("\"2^10\""), "2^10");
    }

    #[test]
    fn normalize_expr_aliases() {
        assert_eq!(normalize_expr("2**10"), "2^10");
        assert_eq!(normalize_expr("6×7"), "6*7");
    }

    #[test]
    fn trim_result_drops_trailing_zeros() {
        assert_eq!(trim_result("12.00000000000000000000"), "12");
        assert_eq!(trim_result("3.140"), "3.14");
    }

    #[test]
    fn eval_via_bc_power_and_sqrt() {
        assert_eq!(eval_via_bc("2^10").map(|r| trim_result(&r)), Some("1024".into()));
        assert_eq!(eval_via_bc("sqrt(144)").map(|r| trim_result(&r)), Some("12".into()));
    }

    #[test]
    fn magnitude_labels() {
        assert_eq!(magnitude_label("42"), MagnitudeLabel::Integer);
        assert_eq!(magnitude_label("3.14"), MagnitudeLabel::Fractional);
        assert_eq!(magnitude_label("9999999999999"), MagnitudeLabel::Huge);
        assert_eq!(magnitude_label("0.0000001"), MagnitudeLabel::Tiny);
    }

    #[test]
    fn pick_flavor_is_deterministic_for_snap() {
        let snap = test_snap(1, 1.23, 4.56);
        assert_eq!(pick_flavor(&snap), pick_flavor(&snap));
    }

    #[test]
    fn calculate_feedback_is_custom_event() {
        let snap = test_snap(0, 0.0, 0.0);
        let ev = calculate_mechanical_feedback("1000", &snap);
        assert!(matches!(ev, ChaosEvent::Custom { .. }));
        assert!(ev.tension_delta() > 0.0);
    }

    #[test]
    fn build_calculate_evidence_shape() {
        let snap = test_snap(3, 0.0, 0.0);
        let fb = vec![calculate_mechanical_feedback("42", &snap)];
        let ev = build_calculate_evidence(
            "6*7",
            "6*7",
            "42",
            FLAVOR[0],
            MagnitudeLabel::Integer,
            5,
            &snap,
            &fb,
            &[],
            magnitude_interpretation_de(MagnitudeLabel::Integer),
        );
        assert_eq!(ev["skill"], "calculate");
        assert_eq!(ev["version"], 2);
        assert_eq!(ev["result"], "42");
        assert_eq!(ev["inv"], 5);
        assert_eq!(ev["feedback"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn decompose_steps_multiply_before_add() {
        let steps = decompose_steps("2+3*4");
        assert!(!steps.is_empty());
        let final_result = eval_via_bc("2+3*4").map(|r| trim_result(&r));
        assert_eq!(final_result, Some("14".into()));
    }

    #[test]
    fn decompose_steps_sqrt() {
        let steps = decompose_steps("sqrt(144)");
        assert!(!steps.is_empty());
        assert_eq!(eval_via_bc("sqrt(144)").map(|r| trim_result(&r)), Some("12".into()));
    }
}
