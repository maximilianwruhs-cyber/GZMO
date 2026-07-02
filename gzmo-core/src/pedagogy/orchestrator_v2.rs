//! Simplified 2-agent Pedagogy Orchestrator
//!
//! Reduces latency from ~12s (4 sequential calls) to ~3s (2 calls).
//! Combines Diagnoser + Planner + Affective into a single Evaluator agent.
//!
//! DEPRECATED: Original 4-agent orchestrator had:
//! - Diagnoser (cognitive state, ~1.5s)
//! - Planner (curriculum, ~1.5s)
//! - Affective (emotional, ~1.5s)
//! - Tutor (Socratic, ~3s)
//! = ~7.5s minimum, typically 10-12s with overhead
//!
//! NEW: 2-agent orchestrator:
//! - Evaluator (combined diagnosis + planning + affective, ~2s)
//! - Tutor (Socratic, ~3s)
//! = ~5s maximum, typically 2-3s

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::config::PedagogyConfig;
use crate::gateway::{LlmGateway, LlmResponse};
use crate::obolus::TokenUsage;
use crate::pedagogy::edf::{EdfRecord, EdfStore, StealthMetrics, ZpdPhase};
use crate::pedagogy::graph::PrerequisiteGraph;
use crate::pedagogy::learner::LearnerProfile;
use crate::pedagogy::trio::TrioMode;
use crate::types::{Message, Role};

const EVALUATOR_TEMPERATURE: f32 = 0.40;  // Balanced: creative enough for diagnosis, consistent enough for planning
const MAX_LEAKAGE_RETRIES: u8 = 2;

const EVALUATOR_SYSTEM: &str = r#"You are the Evaluator Agent inside GZMO's Pedagogy stack.

Your job is to analyze the student's input ONCE and extract all necessary information for tutoring.

Output EXACTLY these lines (no markdown, no extra text):

COGNITIVE_STATE: [novice|developing|proficient|stuck]
MISCONCEPTION: [none or 1-sentence description]
CONFIDENCE: [0.0-1.0] your confidence in the above assessment
TARGET_NODE: [concept id or freeform topic - what should we teach next?]
PREREQ_GAPS: [comma-separated gaps or "none"]
ZPD_PHASE: [i_do|we_do|you_do] scaffolding level needed
COGNITIVE_LOAD: [low|medium|high]
FRUSTRATION: [none|mild|elevated]
HINT_LEVEL: [0-5] number of hints to provide
PEDAGOGICAL_NOTE: [1 sentence on pacing/adjustment needed, or "none"]

Be concise. This output feeds directly into the Socratic Tutor."#;

const TUTOR_SYSTEM: &str = r#"You are the Socratic Tutor — the ONLY student-facing voice.

Context provided (internal only, never mention these labels):
- Student's cognitive state and misconceptions
- Target concept and prerequisite gaps
- ZPD scaffolding level (i_do = heavy guidance, we_do = collaborative, you_do = minimal)
- Cognitive load and frustration indicators
- Recommended hint level

Rules:
1. ZERO solution leakage: never give the final answer before the student earns it
2. Match scaffolding to ZPD_PHASE:
   - i_do: Provide structured guidance, walk through examples
   - we_do: Ask collaborative questions, work together
   - you_do: Minimal hints, let them struggle productively
3. Adjust pacing based on COGNITIVE_LOAD and FRUSTRATION
4. Use graduated hints (respect HINT_LEVEL)
5. Protect epistemic agency — guide thinking, do not replace it
6. Be concise. No meta-talk about "internal agents" or "evaluation"
7. You cannot run shell commands or inspect live systems

Output ONLY the message the student should read — no labels, no XML."#;

const TUTOR_LEAKAGE_RETRY: &str = r#"Your previous draft leaked the solution (full command, complete code, or direct answer).
Rewrite using ONLY Socratic questions and graduated hints. Do not include runnable shell one-liners or fenced code blocks with the answer."#;

pub struct OrchestratorInputV2<'a> {
    pub user_message: &'a str,
    pub learner_profile: &'a LearnerProfile,
    pub trio_mode: TrioMode,
    pub learn_prep_notes: Option<&'a str>,
    pub conversation_tail: &'a str,
    pub chaos_context: Option<&'a str>,
    pub discovery_context: Option<&'a str>,
    pub max_hint_level: u8,
    pub teachback_due: bool,
    pub chaos_snapshot_rx: Option<&'a tokio::sync::watch::Receiver<gzmo_chaos::pulse::ChaosSnapshot>>,
}

/// Per-agent LLM call metrics for text-based MAS research baselines.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AgentCallMetrics {
    pub agent: String,
    pub latency_ms: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

/// Aggregated orchestration metrics (Evaluator + Tutor handoff costs).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct OrchestratorMetrics {
    pub calls: Vec<AgentCallMetrics>,
    pub total_latency_ms: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_tokens: u64,
}

impl OrchestratorMetrics {
    fn push_call(&mut self, call: AgentCallMetrics) {
        self.total_latency_ms += call.latency_ms;
        self.total_input_tokens += call.input_tokens;
        self.total_output_tokens += call.output_tokens;
        self.total_tokens += call.total_tokens;
        self.calls.push(call);
    }
}

pub struct OrchestratorOutputV2 {
    pub response: String,
    pub edf_record: EdfRecord,
    pub internal_trace: String,
    pub latency_ms: u64,
    pub metrics: OrchestratorMetrics,
}

pub struct SimplifiedOrchestrator {
    config: PedagogyConfig,
    graph: Option<PrerequisiteGraph>,
}

impl SimplifiedOrchestrator {
    pub fn new(config: PedagogyConfig, graph: Option<PrerequisiteGraph>) -> Self {
        Self { config, graph }
    }

    /// Run 2-agent orchestration: Evaluator → Tutor
    pub async fn run(
        &self,
        tutor_gateway: &dyn LlmGateway,
        internal_gateway: &dyn LlmGateway,
        input: OrchestratorInputV2<'_>,
    ) -> Result<OrchestratorOutputV2> {
        let start_time = std::time::Instant::now();
        
        // Build context blocks
        let learner_ctx = input.learner_profile.prompt_block(1500);
        let graph_ctx = self.graph.as_ref()
            .map(|g| g.planner_context(input.user_message))
            .unwrap_or_default();
        let prep = input.learn_prep_notes
            .map(|n| format!("\nPrep:\n{n}"))
            .unwrap_or_default();
        
        let trio_line = match input.trio_mode {
            TrioMode::StudentGenAi => "Mode: Student–GenAI teaching.",
            TrioMode::EducatorGenAi => "Mode: Educator–GenAI meta-design.",
            TrioMode::ThirdEye => "Mode: Third Eye reflection.",
        };
        
        // Step 1: EVALUATOR (single call combining diagnosis + planning + affective)
        let eval_prompt = format!(
            "{trio_line}\n{learner_ctx}\n{graph_ctx}{prep}\n\
             Conversation:\n{tail}\n\nStudent message:\n{msg}",
            tail = input.conversation_tail,
            msg = input.user_message
        );
        
        let (evaluation, eval_metrics) = self.evaluator_call(internal_gateway, &eval_prompt).await?;
        let eval_latency_ms = eval_metrics.latency_ms;
        
        // Parse evaluation output
        let zpd = parse_zpd(&evaluation);
        let hint_level = parse_hint_level(&evaluation, input.max_hint_level);
        let stealth = parse_stealth(&evaluation);
        
        // Step 2: TUTOR (single call with all context)
        let mut tutor_context = format!(
            "Evaluator assessment:\n{evaluation}\n\n\
             ZPD phase: {zpd:?}\n\
             Hint level: {hint_level}\n\n\
             Student message:\n{}",
            input.user_message
        );
        
        // Add optional contexts
        if let Some(ctx) = input.chaos_context.filter(|s| !s.trim().is_empty()) {
            tutor_context.push_str(&format!("\n\nSystem state (adapt silently):\n{ctx}"));
        }
        if let Some(ctx) = input.discovery_context.filter(|s| !s.trim().is_empty()) {
            tutor_context.push_str(&format!("\n\nDiscovery context (steer toward):\n{ctx}"));
        }
        if input.teachback_due {
            tutor_context.push_str("\n\nCHECKPOINT: Ask student to explain what they learned so far.");
        }
        
        // Apply chaos overrides if available
        if let Some(rx) = input.chaos_snapshot_rx {
            let snap = rx.borrow();
            tutor_gateway.set_chaos_overrides(snap.llm_temperature, snap.llm_max_tokens);
        }
        
        let (response, leakage_detected, leakage_retries, tutor_metrics) = self
            .tutor_with_leakage_guard(tutor_gateway, &tutor_context, zpd)
            .await?;
        
        let tutor_latency_ms = tutor_metrics.latency_ms;
        
        // Clear chaos overrides
        tutor_gateway.clear_chaos_overrides();
        
        // Build output
        let total_latency_ms = start_time.elapsed().as_millis() as u64;
        let internal_trace = format!(
            "<orchestration_v2>\n<evaluator>{}</evaluator>\n\
             <latency eval='{}ms' tutor='{}ms' total='{}ms'/></orchestration_v2>",
            escape_xml(&evaluation),
            eval_latency_ms,
            tutor_latency_ms,
            total_latency_ms
        );
        
        let preview: String = response.chars().take(120).collect();
        let edf_record = EdfRecord {
            timestamp: chrono::Utc::now(),
            user_input: input.user_message.to_string(),
            evidence: extract_line(&evaluation, "MISCONCEPTION:"),
            decision: format!("zpd={zpd:?} hint={hint_level}"),
            zpd_phase: zpd,
            hint_level,
            stealth,
            tutor_response_preview: preview,
            leakage_detected,
            leakage_retries,
            compute_used: false,
        };
        
        let _ = EdfStore::new(&self.config).append(&edf_record).await;

        let mut metrics = OrchestratorMetrics::default();
        metrics.push_call(eval_metrics);
        metrics.push_call(tutor_metrics);

        tracing::info!(
            target: "gzmo::pedagogy::orchestrator_v2",
            eval_latency_ms,
            tutor_latency_ms,
            total_latency_ms,
            eval_tokens = metrics.calls.first().map(|c| c.total_tokens).unwrap_or(0),
            tutor_tokens = metrics.calls.get(1).map(|c| c.total_tokens).unwrap_or(0),
            total_tokens = metrics.total_tokens,
            "Pedagogy orchestrator metrics"
        );
        
        Ok(OrchestratorOutputV2 {
            response: crate::text_util::strip_mentor_channel_noise(&response),
            edf_record,
            internal_trace,
            latency_ms: total_latency_ms,
            metrics,
        })
    }

    /// Flipped classroom async prep — gather teaching materials before sync session.
    pub async fn run_learn_prep(
        &self,
        internal_gateway: &dyn LlmGateway,
        topic: &str,
    ) -> Result<String> {
        const PREP_SYSTEM: &str = "You are GZMO preparing a flipped-classroom session. \
            Summarize key concepts, common misconceptions, and 3 Socratic opening questions. \
            Be concise. Use bullet points.";
        self.internal_agent_call(
            internal_gateway,
            PREP_SYSTEM,
            &format!("Topic: {topic}"),
            0.30,
        )
        .await
    }

    fn metrics_from_gateway(agent: &str, started: std::time::Instant, gateway: &dyn LlmGateway) -> AgentCallMetrics {
        let usage = gateway.take_last_usage().unwrap_or(TokenUsage {
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
        });
        let latency_ms = gateway
            .take_last_latency_ms()
            .unwrap_or_else(|| started.elapsed().as_millis() as u64);
        AgentCallMetrics {
            agent: agent.to_string(),
            latency_ms,
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            total_tokens: usage.total_tokens,
        }
    }

    async fn internal_agent_call(
        &self,
        gateway: &dyn LlmGateway,
        system: &str,
        user: &str,
        temperature: f32,
    ) -> Result<String> {
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
        gateway.set_chaos_overrides(temperature, self.config.internal_max_tokens);
        let started = std::time::Instant::now();
        let result = match gateway.complete(&messages, &[]).await {
            Ok(LlmResponse::Text(t)) => Ok(t.trim().to_string()),
            Ok(LlmResponse::ToolCalls(_)) => bail!("internal agent returned unexpected tool calls"),
            Err(e) => {
                gateway.clear_chaos_overrides();
                return Err(e.into());
            }
        };
        gateway.clear_chaos_overrides();
        let _ = Self::metrics_from_gateway("internal", started, gateway);
        result
    }

    async fn evaluator_call(
        &self,
        gateway: &dyn LlmGateway,
        prompt: &str,
    ) -> Result<(String, AgentCallMetrics)> {
        let messages = vec![
            Message {
                role: Role::System,
                content: EVALUATOR_SYSTEM.to_string(),
                is_meta: true,
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: Role::User,
                content: prompt.to_string(),
                is_meta: false,
                tool_calls: None,
                tool_call_id: None,
            },
        ];
        
        gateway.set_chaos_overrides(EVALUATOR_TEMPERATURE, self.config.internal_max_tokens);
        let started = std::time::Instant::now();
        
        let text: String = match gateway.complete(&messages, &[]).await {
            Ok(LlmResponse::Text(t)) => t.trim().to_string(),
            Ok(LlmResponse::ToolCalls(_)) => {
                gateway.clear_chaos_overrides();
                bail!("evaluator returned unexpected tool calls");
            }
            Err(e) => {
                gateway.clear_chaos_overrides();
                return Err(e.into());
            }
        };
        
        gateway.clear_chaos_overrides();
        let metrics = Self::metrics_from_gateway("evaluator", started, gateway);
        Ok((text, metrics))
    }

    async fn tutor_with_leakage_guard(
        &self,
        gateway: &dyn LlmGateway,
        context: &str,
        zpd: ZpdPhase,
    ) -> Result<(String, bool, u8, AgentCallMetrics)> {
        let mut ctx = context.to_string();
        let mut retries: u8 = 0;
        let enforce = self.config.solution_leakage_penalty > 0.0;
        
        let started = std::time::Instant::now();
        loop {
            let messages = vec![
                Message {
                    role: Role::System,
                    content: TUTOR_SYSTEM.to_string(),
                    is_meta: true,
                    tool_calls: None,
                    tool_call_id: None,
                },
                Message {
                    role: Role::User,
                    content: ctx.clone(),
                    is_meta: false,
                    tool_calls: None,
                    tool_call_id: None,
                },
            ];
            
            let response = match gateway.complete(&messages, &[]).await {
                Ok(LlmResponse::Text(t)) => t.trim().to_string(),
                Ok(LlmResponse::ToolCalls(_)) => bail!("tutor returned unexpected tool calls"),
                Err(e) => {
                    gateway.clear_chaos_overrides();
                    return Err(e.into());
                }
            };
            
            let leaky = enforce && detect_leakage(&response, zpd);
            
            if !leaky || retries >= MAX_LEAKAGE_RETRIES {
                gateway.clear_chaos_overrides();
                let metrics = Self::metrics_from_gateway("tutor", started, gateway);
                return Ok((response, leaky, retries, metrics));
            }
            
            retries += 1;
            ctx = format!("{context}\n\n{TUTOR_LEAKAGE_RETRY}");
        }
    }
}

// Simplified parsers for Evaluator output
fn parse_zpd(eval: &str) -> ZpdPhase {
    let line = extract_line(eval, "ZPD_PHASE:").to_lowercase();
    if line.contains("you_do") {
        ZpdPhase::YouDo
    } else if line.contains("i_do") {
        ZpdPhase::IDo
    } else {
        ZpdPhase::WeDo
    }
}

fn parse_hint_level(eval: &str, max: u8) -> u8 {
    extract_line(eval, "HINT_LEVEL:")
        .parse::<u8>()
        .unwrap_or(3)
        .min(max)
        .max(1)
}

fn parse_stealth(eval: &str) -> StealthMetrics {
    StealthMetrics {
        psu: parse_metric(eval, "CONFIDENCE:"),
        sdr: parse_metric(eval, "CONFIDENCE:"), // Reuse confidence as proxy
        lvd: parse_metric(eval, "CONFIDENCE:"),
    }
}

fn parse_metric(eval: &str, prefix: &str) -> f64 {
    extract_line(eval, prefix)
        .parse::<f64>()
        .unwrap_or(0.5)
        .clamp(0.0, 1.0)
}

fn extract_line(block: &str, prefix: &str) -> String {
    block
        .lines()
        .find(|l| l.starts_with(prefix))
        .map(|l| l.strip_prefix(prefix).unwrap_or(l).trim().to_string())
        .unwrap_or_default()
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn detect_leakage(response: &str, zpd: ZpdPhase) -> bool {
    let lower = response.to_lowercase();
    
    // Check for code blocks in YouDo phase
    if matches!(zpd, ZpdPhase::YouDo) && response.contains("```") {
        return true;
    }
    
    // Check for direct solutions
    let leak_indicators = [
        "the answer is",
        "the solution is",
        "here's the code:",
        "just run",
        "simply type",
    ];
    
    leak_indicators.iter().any(|&indicator| lower.contains(indicator))
}

impl From<OrchestratorOutputV2> for super::orchestrator::OrchestratorOutput {
    fn from(v2: OrchestratorOutputV2) -> Self {
        Self {
            response: v2.response,
            edf_record: v2.edf_record,
            internal_trace: v2.internal_trace,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parse_zpd_test() {
        let eval = "ZPD_PHASE: you_do\nCOGNITIVE_STATE: novice";
        assert_eq!(parse_zpd(eval), ZpdPhase::YouDo);
        
        let eval = "ZPD_PHASE: i_do\nCOGNITIVE_STATE: stuck";
        assert_eq!(parse_zpd(eval), ZpdPhase::IDo);
        
        let eval = "ZPD_PHASE: we_do\nCOGNITIVE_STATE: developing";
        assert_eq!(parse_zpd(eval), ZpdPhase::WeDo);
    }
    
    #[test]
    fn parse_hint_level_test() {
        let eval = "HINT_LEVEL: 3\nZPD_PHASE: we_do";
        assert_eq!(parse_hint_level(eval, 5), 3);
        
        let eval = "HINT_LEVEL: 8\nZPD_PHASE: we_do";
        assert_eq!(parse_hint_level(eval, 5), 5); // Clamped to max
    }
    
    #[test]
    fn detect_leakage_test() {
        assert!(detect_leakage("The answer is 42.", ZpdPhase::YouDo));
        assert!(detect_leakage("```python\nprint(42)\n```", ZpdPhase::YouDo));
        assert!(!detect_leakage("What do you think?", ZpdPhase::WeDo));
    }

    #[test]
    fn orchestrator_metrics_push() {
        let mut m = OrchestratorMetrics::default();
        m.push_call(AgentCallMetrics {
            agent: "evaluator".into(),
            latency_ms: 100,
            input_tokens: 50,
            output_tokens: 20,
            total_tokens: 70,
        });
        m.push_call(AgentCallMetrics {
            agent: "tutor".into(),
            latency_ms: 200,
            input_tokens: 80,
            output_tokens: 40,
            total_tokens: 120,
        });
        assert_eq!(m.total_latency_ms, 300);
        assert_eq!(m.total_tokens, 190);
        assert_eq!(m.calls.len(), 2);
    }
}