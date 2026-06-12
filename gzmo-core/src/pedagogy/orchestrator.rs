//! Agentic Teacher orchestrator — sequential 4-agent Think–Plan–Act loop.

use anyhow::{bail, Result};

use crate::config::PedagogyConfig;
use crate::gateway::{LlmGateway, LlmResponse};
use crate::pedagogy::edf::{EdfRecord, EdfStore, StealthMetrics, ZpdPhase};
use crate::pedagogy::graph::PrerequisiteGraph;
use crate::pedagogy::learner::LearnerProfile;
use crate::pedagogy::trio::TrioMode;
use crate::types::{Message, Role};
use crate::tools::{python_sandbox::PythonSandboxTool, ToolHandler};

const INTERNAL_AGENT_TEMPERATURE: f32 = 0.35;
const MAX_LEAKAGE_RETRIES: u8 = 2;

const DIAGNOSER_SYSTEM: &str = r#"You are the Diagnoser/Evaluator Agent inside GZMO's Agentic Teacher stack.
Analyze the student's raw input to infer cognitive state. Detect hidden misconceptions.
Output ONLY these lines (no markdown):
STATE: [novice|developing|proficient|stuck]
MISCONCEPTION: [none or brief description]
EVIDENCE: [2 sentences of diagnostic reasoning]
STEALTH_PSU: [0.0-1.0 prompt structure uptake]
STEALTH_SDR: [0.0-1.0 scaffolded depth/revision]
STEALTH_LVD: [0.0-1.0 logic validation]"#;

const PLANNER_SYSTEM: &str = r#"You are the Curriculum Planner Agent inside GZMO's Agentic Teacher stack.
Map the student's understanding to prerequisite concepts. Identify gaps.
Output ONLY these lines:
TARGET_NODE: [concept id or freeform topic]
GAPS: [comma-separated prerequisite gaps or none]
ZPD: [i_do|we_do|you_do]
MICRO_PIVOT: [none or brief modality change suggestion]
COMPUTE: [none or python expression to execute, e.g. 2**20 or math.sqrt(28)]"#;

const AFFECTIVE_SYSTEM: &str = r#"You are the Affective/Moderation Agent inside GZMO's Agentic Teacher stack.
Monitor frustration and cognitive overload. Maintain emotional boundaries (not a therapist).
Output ONLY these lines:
LOAD: [low|medium|high]
FRUSTRATION: [none|mild|elevated]
BOUNDARY: [ok|remind_ai_nature]
ADJUSTMENT: [none or brief pacing/scaffolding note]"#;

const TUTOR_SYSTEM: &str = r#"You are the Socratic Tutor — the ONLY student-facing voice of GZMO.
You are the Friendly Linux Mentor: witty, technically precise, loyal, candid.
Rules:
- ZERO solution leakage: never give the final answer or complete command before the student earns it.
- Use graduated hints, probing questions, and ZPD-appropriate scaffolding.
- Protect epistemic agency — guide thinking, do not replace it.
- Mirror the student's language (German/English).
- Be concise. No meta-talk about internal agents.
- When sandbox intermediate outputs are provided, use them to ask better Socratic questions, but NEVER paste the final numeric answer or code solution verbatim; guide the student to derive it.
Output ONLY the message the student should read — no labels, no XML."#;

const TUTOR_LEAKAGE_RETRY: &str = r#"Your previous draft leaked the solution (full command, complete code, or direct answer).
Rewrite using ONLY Socratic questions and graduated hints. Do not include runnable shell one-liners or fenced code blocks with the answer."#;

pub struct OrchestratorInput<'a> {
    pub user_message: &'a str,
    pub learner_profile: &'a LearnerProfile,
    pub trio_mode: TrioMode,
    pub learn_prep_notes: Option<&'a str>,
    pub conversation_tail: &'a str,
    pub max_hint_level: u8,
    /// When true, Tutor asks the student for a teachback before continuing.
    pub teachback_due: bool,
}

pub struct OrchestratorOutput {
    pub response: String,
    pub edf_record: EdfRecord,
    pub internal_trace: String,
}

pub struct PedagogyOrchestrator {
    config: PedagogyConfig,
    graph: Option<PrerequisiteGraph>,
}

impl PedagogyOrchestrator {
    pub fn new(config: PedagogyConfig, graph: Option<PrerequisiteGraph>) -> Self {
        Self { config, graph }
    }

    pub async fn run(
        &self,
        tutor_gateway: &dyn LlmGateway,
        internal_gateway: &dyn LlmGateway,
        input: OrchestratorInput<'_>,
    ) -> Result<OrchestratorOutput> {
        let learner_ctx = input.learner_profile.prompt_block(1500);
        let graph_ctx = self
            .graph
            .as_ref()
            .map(|g| g.planner_context(input.user_message))
            .unwrap_or_default();

        let trio_line = match input.trio_mode {
            TrioMode::StudentGenAi => "Mode: Student–GenAI teaching.",
            TrioMode::EducatorGenAi => "Mode: Educator–GenAI meta-design.",
            TrioMode::ThirdEye => "Mode: Third Eye reflection.",
        };

        let prep = input
            .learn_prep_notes
            .map(|n| format!("\nPrep materials:\n{n}"))
            .unwrap_or_default();

        let user_block = format!(
            "{trio_line}\n{learner_ctx}\n{graph_ctx}{prep}\n\
             Recent context:\n{}\n\nStudent message:\n{}",
            input.conversation_tail, input.user_message
        );

        let diag = self
            .internal_agent_call(internal_gateway, DIAGNOSER_SYSTEM, &user_block)
            .await?;
        let plan = self
            .internal_agent_call(
                internal_gateway,
                PLANNER_SYSTEM,
                &format!("Diagnoser output:\n{diag}\n\nStudent message:\n{}", input.user_message),
            )
            .await?;
        let affect = self
            .internal_agent_call(
                internal_gateway,
                AFFECTIVE_SYSTEM,
                &format!(
                    "Diagnoser:\n{diag}\nPlanner:\n{plan}\n\nStudent message:\n{}",
                    input.user_message
                ),
            )
            .await?;

        let zpd = parse_zpd(&plan);
        let hint_level = parse_hint_level(&plan, input.max_hint_level);

        // Parse and execute COMPUTE if orchestrator offloading is enabled
        let compute_expr = extract_line(&plan, "COMPUTE:");
        let mut sandbox_output = None;
        let mut compute_used = false;

        if !compute_expr.is_empty() && compute_expr.to_lowercase() != "none" {
            if self.config.sandbox.orchestrator_offload {
                let code = if compute_expr.contains("print") || compute_expr.contains('\n') || compute_expr.contains("import") {
                    compute_expr.clone()
                } else {
                    format!(
                        "import math, statistics, re, json, itertools\n\
                         from fractions import Fraction\n\
                         from decimal import Decimal\n\
                         print({})",
                        compute_expr
                    )
                };

                let sandbox = PythonSandboxTool::new(&self.config);
                match sandbox.execute(serde_json::json!({ "code": code })).await {
                    Ok(out) => {
                        sandbox_output = Some(out);
                        compute_used = true;
                    }
                    Err(e) => {
                        sandbox_output = Some(format!("Sandbox execution failed: {e}"));
                        compute_used = true;
                    }
                }
            }
        }

        let mut tutor_user = format!(
            "Diagnoser:\n{diag}\n\nPlanner:\n{plan}\n\nAffective:\n{affect}\n\n\
             Max hint level: {hint_level}/{}\n\nStudent message:\n{}",
            input.max_hint_level, input.user_message
        );
        if let Some(ref out) = sandbox_output {
            tutor_user = format!(
                "Tutor Context - Sandbox Intermediate:\n{}\n\n{}",
                out, tutor_user
            );
        }
        if input.teachback_due {
            tutor_user.push_str(
                "\n\nTEACHBACK CHECKPOINT: Before continuing, ask the student to explain \
                 what they have learned so far in their own words. One focused prompt only.",
            );
        }

        let (response, leakage_detected, leakage_retries) = self
            .tutor_with_leakage_guard(tutor_gateway, &tutor_user, zpd, sandbox_output.as_deref())
            .await?;

        let stealth = parse_stealth(&diag);
        let internal_trace = format!(
            "<agentic_orchestration>\n<diagnoser>{diag}</diagnoser>\n\
             <planner>{plan}</planner>\n<affective>{affect}</affective>\n\
             <sandbox_output>{}</sandbox_output>\n</agentic_orchestration>",
            sandbox_output.as_deref().unwrap_or("none")
        );

        let preview: String = response.chars().take(120).collect();
        let edf_record = EdfRecord {
            timestamp: chrono::Utc::now(),
            user_input: input.user_message.to_string(),
            evidence: extract_line(&diag, "EVIDENCE:"),
            decision: format!(
                "zpd={} hint={hint_level} {}",
                zpd.as_str(),
                extract_line(&plan, "MICRO_PIVOT:")
            ),
            zpd_phase: zpd,
            hint_level,
            stealth,
            tutor_response_preview: preview,
            leakage_detected,
            leakage_retries,
            compute_used,
        };

        let store = EdfStore::new(&self.config);
        let _ = store.append(&edf_record).await;

        Ok(OrchestratorOutput {
            response,
            edf_record,
            internal_trace,
        })
    }

    /// Flipped classroom async prep — gather teaching materials before sync session.
    pub async fn run_learn_prep(
        &self,
        internal_gateway: &dyn LlmGateway,
        topic: &str,
    ) -> Result<String> {
        let system = "You are GZMO preparing a flipped-classroom session. \
            Summarize key concepts, common misconceptions, and 3 Socratic opening questions. \
            Be concise. Use bullet points.";
        self.internal_agent_call(internal_gateway, system, &format!("Topic: {topic}"))
            .await
    }

    async fn tutor_with_leakage_guard(
        &self,
        gateway: &dyn LlmGateway,
        tutor_user: &str,
        zpd: ZpdPhase,
        sandbox_output: Option<&str>,
    ) -> Result<(String, bool, u8)> {
        let mut user_block = tutor_user.to_string();
        let mut retries: u8 = 0;
        let enforce = self.config.solution_leakage_penalty > 0.0;

        loop {
            let response = self
                .agent_call(gateway, TUTOR_SYSTEM, &user_block, None)
                .await?;
            let leaky = enforce && detect_solution_leakage(&response, zpd, sandbox_output);
            if !leaky || retries >= MAX_LEAKAGE_RETRIES {
                return Ok((response, leaky, retries));
            }
            retries += 1;
            user_block = format!("{tutor_user}\n\n{TUTOR_LEAKAGE_RETRY}");
        }
    }

    async fn internal_agent_call(
        &self,
        gateway: &dyn LlmGateway,
        system: &str,
        user: &str,
    ) -> Result<String> {
        self.agent_call(
            gateway,
            system,
            user,
            Some(self.config.internal_max_tokens),
        )
        .await
    }

    async fn agent_call(
        &self,
        gateway: &dyn LlmGateway,
        system: &str,
        user: &str,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        if let Some(cap) = max_tokens {
            gateway.set_chaos_overrides(INTERNAL_AGENT_TEMPERATURE, cap);
        }

        let result = async {
            let messages = vec![
                Message {
                    role: Role::System,
                    content: system.to_string(),
                    is_meta: true,
                    tool_calls: None,
                    tool_call_id: None,
                },
                Message {
                    role: Role::User,
                    content: user.to_string(),
                    is_meta: false,
                    tool_calls: None,
                    tool_call_id: None,
                },
            ];
            match gateway.complete(&messages, &[]).await? {
                LlmResponse::Text(t) => Ok(t.trim().to_string()),
                LlmResponse::ToolCalls(_) => {
                    bail!("pedagogy agent returned unexpected tool calls")
                }
            }
        }
        .await;

        if max_tokens.is_some() {
            gateway.clear_chaos_overrides();
        }

        result
    }
}

/// Heuristic scorer for Tutor outputs that give away the answer too early.
pub fn detect_solution_leakage(
    response: &str,
    zpd: ZpdPhase,
    sandbox_output: Option<&str>,
) -> bool {
    let lower = response.to_lowercase();

    if response.contains("```") {
        let fence_body = lower
            .split("```")
            .nth(1)
            .unwrap_or("")
            .to_string();
        if contains_shell_solution(&fence_body) {
            return true;
        }
    }

    for line in response.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            continue;
        }
        if contains_shell_solution(trimmed) {
            return true;
        }
        if matches!(zpd, ZpdPhase::YouDo)
            && trimmed.len() > 12
            && !trimmed.ends_with('?')
            && (trimmed.starts_with("Run ") || trimmed.starts_with("Use "))
        {
            return true;
        }
    }

    // Flag responses that repeat sandbox final values verbatim when not in YouDo (autonomous/fading) phase.
    if !matches!(zpd, ZpdPhase::YouDo) {
        if let Some(out) = sandbox_output {
            let leak_values = extract_sandbox_values(out);
            for val in leak_values {
                if is_word_boundary_match(response, &val) {
                    return true;
                }
            }
        }
    }

    false
}

fn extract_sandbox_values(sandbox_output: &str) -> Vec<String> {
    let mut values = Vec::new();
    for line in sandbox_output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Exit code:")
            || trimmed.starts_with("--- stdout ---")
            || trimmed.starts_with("--- stderr ---")
            || trimmed.starts_with("Sandbox execution failed")
        {
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }
        // Check if it's numeric
        let is_numeric = trimmed.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-');
        if is_numeric {
            values.push(trimmed.to_string());
        } else if trimmed.len() >= 3 {
            let lower = trimmed.to_lowercase();
            if lower != "true" && lower != "false" && lower != "none" {
                values.push(trimmed.to_string());
            }
        }
    }
    values
}

fn is_word_boundary_match(text: &str, value: &str) -> bool {
    let text_lower = text.to_lowercase();
    let val_lower = value.to_lowercase();
    let val_len = val_lower.len();
    if val_len == 0 {
        return false;
    }

    let mut start = 0;
    while let Some(pos) = text_lower[start..].find(&val_lower) {
        let actual_pos = start + pos;
        let char_before = if actual_pos > 0 {
            text_lower.as_bytes().get(actual_pos - 1).map(|&b| b as char)
        } else {
            None
        };
        let char_after = text_lower.as_bytes().get(actual_pos + val_len).map(|&b| b as char);

        let is_boundary_before = match char_before {
            Some(c) => !c.is_alphanumeric() && c != '_',
            None => true,
        };
        let is_boundary_after = match char_after {
            Some(c) => !c.is_alphanumeric() && c != '_',
            None => true,
        };

        if is_boundary_before && is_boundary_after {
            return true;
        }
        start = actual_pos + 1;
    }
    false
}

fn contains_shell_solution(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.starts_with("sudo ")
        || lower.starts_with("chmod ")
        || lower.starts_with("systemctl ")
        || lower.starts_with("curl ")
        || lower.starts_with("wget ")
        || lower.contains("rm -rf")
}

fn extract_line(block: &str, prefix: &str) -> String {
    block
        .lines()
        .find(|l| l.starts_with(prefix))
        .map(|l| l.strip_prefix(prefix).unwrap_or(l).trim().to_string())
        .unwrap_or_default()
}

fn parse_zpd(plan: &str) -> ZpdPhase {
    let line = extract_line(plan, "ZPD:").to_lowercase();
    if line.contains("you_do") {
        ZpdPhase::YouDo
    } else if line.contains("i_do") {
        ZpdPhase::IDo
    } else {
        ZpdPhase::WeDo
    }
}

fn parse_hint_level(plan: &str, max: u8) -> u8 {
    let zpd = parse_zpd(plan);
    let base = match zpd {
        ZpdPhase::IDo => 1,
        ZpdPhase::WeDo => 3,
        ZpdPhase::YouDo => max.saturating_sub(1).max(1),
    };
    base.min(max)
}

fn parse_stealth(diag: &str) -> StealthMetrics {
    StealthMetrics {
        psu: parse_metric(diag, "STEALTH_PSU:"),
        sdr: parse_metric(diag, "STEALTH_SDR:"),
        lvd: parse_metric(diag, "STEALTH_LVD:"),
    }
}

fn parse_metric(block: &str, prefix: &str) -> f64 {
    extract_line(block, prefix)
        .parse::<f64>()
        .unwrap_or(0.5)
        .clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_fenced_chmod_as_leakage() {
        let text = "Try thinking about permissions.\n```bash\nchmod 755 script.sh\n```";
        assert!(detect_solution_leakage(text, ZpdPhase::WeDo, None));
    }

    #[test]
    fn socratic_question_without_command_is_not_leakage() {
        let text = "What do you think the three digits in chmod represent?";
        assert!(!detect_solution_leakage(text, ZpdPhase::WeDo, None));
    }

    #[test]
    fn direct_sudo_line_is_leakage() {
        let text = "sudo systemctl restart nginx";
        assert!(detect_solution_leakage(text, ZpdPhase::YouDo, None));
    }

    #[test]
    fn detects_sandbox_value_leakage() {
        let text = "The result is 1048576.";
        let sandbox_out = "Exit code: 0\n1048576";
        assert!(detect_solution_leakage(text, ZpdPhase::WeDo, Some(sandbox_out)));

        // Not a leak in YouDo phase
        assert!(!detect_solution_leakage(text, ZpdPhase::YouDo, Some(sandbox_out)));

        // Substring boundary check: 10485764 should not match 1048576
        let text_no_leak = "Let's check 10485764.";
        assert!(!detect_solution_leakage(text_no_leak, ZpdPhase::WeDo, Some(sandbox_out)));
    }
}
