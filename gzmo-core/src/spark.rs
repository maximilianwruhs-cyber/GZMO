//! # Spark Engine
//!
//! Chaos-free serendipitous recall: revisit a **stale** vault fact, connect it to
//! **recent** context, verify the link, then promote only an `HYPOTHESIZED_LINK`
//! (L3) — never new L2 facts at confidence 1.0.

use std::sync::Arc;

use anyhow::Result;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use tracing::{info, warn};
use uuid::Uuid;

use crate::config::SparkConfig;
use crate::gateway::LlmGateway;
use crate::memory::episodic::FileEpisodicStore;
use crate::memory::felt_use::{self, FeltUseKind};
use crate::memory::kg_promotion::{entity_label_from_fact, HYPOTHESIZED_LINK};
use crate::memory::vault::{embedding_cosine_similarity, SqliteVault};
use crate::night_lymph::{self, LymphSpark};
use crate::spark_field;
use crate::synapse::{resolve_event_source, EventSource, EventType, SynapseBus, SynapseEvent};
use crate::tools::{ToolRegistry, ToolResult};
use crate::types::SemanticFact;
use crate::types::{EpisodicEntry, EpisodicSource, Message, Role};

// ---------------------------------------------------------------------------
// Selection / hypothesis / verification types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct SparkSelection {
    anchor: crate::types::SemanticFact,
    recent: Vec<crate::types::SemanticFact>,
    selection_score: f64,
    refractory_multiplier: f64,
    refractory_reason: &'static str,
    refractory_entries: usize,
    soft_pick_roll: f64,
    candidates_scored: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct SparkHypothesis {
    #[allow(dead_code)]
    internal_analysis: String,
    anchor_label: String,
    recent_label: String,
    connection: String,
    what_to_remember: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct SparkVerdict {
    supported: bool,
    confidence: f64,
    evidence_anchor: String,
    evidence_recent: String,
}

// ---------------------------------------------------------------------------
// Spark Engine
// ---------------------------------------------------------------------------

pub struct SparkEngine {
    vault: SqliteVault,
    episodic: FileEpisodicStore,
    gateway: Arc<dyn LlmGateway>,
    /// Separate gateway for the verification phase (Obolus routing).
    /// Falls back to `gateway` when not set.
    verify_gateway: Option<Arc<dyn LlmGateway>>,
    tools: Arc<ToolRegistry>,
    config: SparkConfig,
    /// Optional Synapse bus for observability.
    synapse: Option<Arc<SynapseBus>>,
}

impl SparkEngine {
    pub fn new(
        vault: SqliteVault,
        episodic: FileEpisodicStore,
        gateway: Arc<dyn LlmGateway>,
        tools: Arc<ToolRegistry>,
        config: SparkConfig,
        synapse: Option<Arc<SynapseBus>>,
    ) -> Self {
        Self {
            vault,
            episodic,
            gateway,
            verify_gateway: None,
            tools,
            config,
            synapse,
        }
    }

    /// Construct with a separate verify gateway (Obolus routing).
    pub fn new_with_verify(
        vault: SqliteVault,
        episodic: FileEpisodicStore,
        gateway: Arc<dyn LlmGateway>,
        verify_gateway: Arc<dyn LlmGateway>,
        tools: Arc<ToolRegistry>,
        config: SparkConfig,
        synapse: Option<Arc<SynapseBus>>,
    ) -> Self {
        Self {
            vault,
            episodic,
            gateway,
            verify_gateway: Some(verify_gateway),
            tools,
            config,
            synapse,
        }
    }

    /// Resolve the gateway to use for a given phase.
    fn resolve_gateway(&self, phase: &str) -> &Arc<dyn LlmGateway> {
        match phase {
            "verify" => self.verify_gateway.as_ref().unwrap_or(&self.gateway),
            _ => &self.gateway,
        }
    }

    fn emit_spark_complete(
        &self,
        date: NaiveDate,
        promoted: bool,
        kg_relations_written: usize,
        skip_reason: Option<&str>,
        anchor_id: Option<&str>,
    ) {
        let Some(ref bus) = self.synapse else {
            return;
        };
        let mut data = serde_json::json!({
            "date": date.to_string(),
            "promoted": promoted,
            "kg_relations_written": kg_relations_written,
            "memory_layer": self.vault.cognition_memory_layer(),
            "cognition_source": self.vault.cognition_memory_layer(),
        });
        if let Some(reason) = skip_reason {
            data["skip_reason"] = serde_json::Value::String(reason.to_string());
        }
        if let Some(id) = anchor_id {
            data["anchor_id"] = serde_json::Value::String(id.to_string());
        }
        bus.append(&SynapseEvent::with_data(
            EventType::SparkComplete,
            resolve_event_source(EventSource::GzmoDaemon),
            data,
        ));
    }

    /// Lymph + refractory for skipped cycles so overnight filtrate matches DREAMS.md growth.
    fn record_skipped_spark(
        &self,
        date: NaiveDate,
        reason: &str,
        anchor: Option<&crate::types::SemanticFact>,
    ) {
        if let Some(a) = anchor {
            spark_field::record_selection(self.vault.db_path(), a, self.config.refractory_slots);
        }
        let preview = anchor
            .map(|a| a.content.chars().take(120).collect::<String>())
            .unwrap_or_else(|| reason.chars().take(120).collect());
        if let Err(e) = night_lymph::record_spark(
            self.vault.db_path(),
            night_lymph::lymph_night_id(Utc::now()),
            LymphSpark {
                date: date.to_string(),
                promoted: false,
                kg_relations: 0,
                anchor_id: anchor.map(|a| a.id.to_string()),
                anchor_preview: Some(preview),
            },
        ) {
            warn!(error = %e, "night_lymph record_spark failed (skip path)");
        }
    }

    /// Run one spark cycle for `date` (used in logs and DREAMS.md headings).
    pub async fn run(&self, date: NaiveDate) -> Result<SparkReport> {
        if !self.config.enabled {
            info!("Spark disabled in config — skipping");
            let reason = "Spark disabled in [spark] config";
            self.emit_spark_complete(date, false, 0, Some(reason), None);
            return Ok(SparkReport::skipped(date, reason));
        }

        info!(date = %date, "Starting spark cycle");

        let selection = match self.select_phase().await? {
            Some(s) => s,
            None => {
                info!("Spark selection found no anchor/recent pair — skipped");
                let reason =
                    "No curated pair: empty pools, same-ingest recent slab, or [spark] gates.";
                self.emit_spark_complete(date, false, 0, Some(reason), None);
                self.record_skipped_spark(date, reason, None);
                return Ok(SparkReport::skipped(date, reason));
            }
        };

        let hypothesis = match self.hypothesis_phase(&selection).await {
            Ok(h) => h,
            Err(e) => {
                warn!("Spark hypothesis phase failed: {e}");
                let reason = format!("Hypothesis generation failed: {e}");
                self.emit_spark_complete(date, false, 0, Some(&reason), None);
                self.record_skipped_spark(date, &reason, Some(&selection.anchor));
                return Ok(SparkReport::skipped(date, &reason));
            }
        };

        let (promoted, verdict) = if self.config.verify {
            match self.verify_phase(&selection, &hypothesis).await {
                Ok(mut v) => {
                    let min_c = self.config.min_citation_chars;
                    repair_citations_from_facts(&selection, &mut v, min_c);
                    let citations_ok = citations_valid(&selection, &v, min_c);
                    let ok = spark_promote_ok(
                        v.supported,
                        v.confidence,
                        citations_ok,
                        self.config.min_confidence,
                    );
                    if !ok {
                        warn!(
                            supported = v.supported,
                            confidence = v.confidence,
                            citations_ok,
                            "Spark link failed verification or citations — abstaining"
                        );
                    }
                    if ok {
                        (true, Some(v))
                    } else {
                        (false, Some(v))
                    }
                }
                Err(e) => {
                    warn!("Spark verification failed — abstaining: {e}");
                    let reason = format!("Verification failed (nothing promoted): {e}");
                    self.emit_spark_complete(
                        date,
                        false,
                        0,
                        Some(&reason),
                        Some(&selection.anchor.id.to_string()),
                    );
                    self.record_skipped_spark(date, &reason, Some(&selection.anchor));
                    return Ok(SparkReport::skipped(date, &reason));
                }
            }
        } else {
            (true, None)
        };

        if !promoted {
            let reason = "verification unsupported or citations failed — abstained";
            self.emit_spark_complete(
                date,
                false,
                0,
                Some(reason),
                Some(&selection.anchor.id.to_string()),
            );
            self.record_skipped_spark(date, reason, Some(&selection.anchor));
            return Ok(SparkReport::skipped(date, reason));
        }

        let kg_written = self
            .promote_phase(date, &selection, &hypothesis, verdict.as_ref())
            .await;

        let _ = felt_use::touch(&self.vault, selection.anchor.id, FeltUseKind::Bonded);
        for r in &selection.recent {
            let _ = felt_use::touch(&self.vault, r.id, FeltUseKind::Bonded);
        }

        spark_field::record_selection(
            self.vault.db_path(),
            &selection.anchor,
            self.config.refractory_slots,
        );
        spark_field::write_last_spark_report(
            self.vault.db_path(),
            &date.to_string(),
            promoted,
            kg_written,
            Some(selection.anchor.id),
            Some(&selection.anchor.content),
            spark_field::SparkSelectionMetrics {
                selection_score: Some(selection.selection_score),
                refractory_multiplier: Some(selection.refractory_multiplier),
                refractory_reason: Some(selection.refractory_reason),
                refractory_entries: Some(selection.refractory_entries),
                soft_pick_roll: Some(selection.soft_pick_roll),
                soft_pick_top_k: Some(self.config.soft_pick_top_k),
                soft_pick_temperature: Some(self.config.soft_pick_temperature),
                candidates_scored: Some(selection.candidates_scored),
            },
        );
        if let Err(e) = night_lymph::record_spark(
            self.vault.db_path(),
            night_lymph::lymph_night_id(Utc::now()),
            LymphSpark {
                date: date.to_string(),
                promoted,
                kg_relations: kg_written,
                anchor_id: Some(selection.anchor.id.to_string()),
                anchor_preview: Some(selection.anchor.content.chars().take(120).collect()),
            },
        ) {
            warn!(error = %e, "night_lymph record_spark failed");
        }

        let section =
            self.format_spark_section(date, &selection, &hypothesis, promoted, verdict.as_ref());
        self.log_episodic(date, &section).await?;

        self.emit_spark_complete(
            date,
            true,
            kg_written,
            None,
            Some(&selection.anchor.id.to_string()),
        );

        Ok(SparkReport {
            date,
            promoted,
            kg_relations_written: kg_written,
            section,
        })
    }

    /// Phase 1 — scored curated anchor + recent pool (no LLM).
    async fn select_phase(&self) -> Result<Option<SparkSelection>> {
        let recent_fetch = self.config.recent_limit.saturating_mul(4).max(16);
        let recent_raw = self.vault.spark_recent_pool(
            &self.config.anchor_decay_classes,
            self.config.recent_max_age_hours,
            recent_fetch,
        )?;
        if recent_pool_is_ingest_slab(&recent_raw) {
            info!("Spark recent pool is one ingest slab — skipping");
            return Ok(None);
        }
        let recent = dedupe_recent_facts(recent_raw, self.config.recent_dedupe_similarity);
        let recent: Vec<_> = recent.into_iter().take(self.config.recent_limit).collect();

        if recent.is_empty() {
            return Ok(None);
        }

        let anchor_fetch = self.config.candidate_limit.saturating_mul(8).max(32);
        let anchors = self.vault.spark_anchor_pool(
            &self.config.anchor_decay_classes,
            self.config.anchor_min_age_hours,
            self.config.anchor_min_stale_days,
            self.config.anchor_max_stale_days,
            anchor_fetch,
        )?;

        let min_sim = self.config.min_anchor_recent_similarity;
        let field = spark_field::load_field(self.vault.db_path());
        let mut scored: Vec<(SemanticFact, f64, f64, spark_field::RefractoryExplain)> = Vec::new();

        for candidate in anchors {
            if !self.is_viable_anchor(&candidate) {
                continue;
            }
            if recent.iter().any(|r| r.id == candidate.id) {
                continue;
            }
            let max_sim = max_embedding_similarity(&candidate, &recent);
            let tag_bridge = recent
                .iter()
                .any(|r| shares_spark_concept_tag(&candidate.content, &r.content));
            if !anchor_passes_prefilter(&candidate, &recent, min_sim, tag_bridge, max_sim) {
                continue;
            }
            let base = score_spark_anchor(
                &candidate,
                &recent,
                self.config.anchor_min_stale_days,
                self.config.anchor_max_stale_days,
                min_sim,
                tag_bridge,
            );
            let expl = spark_field::explain_refractory(
                &candidate,
                &field,
                self.config.refractory_half_life_hours,
                self.config.refractory_strength,
            );
            let score = base * expl.multiplier;
            scored.push((candidate, score, max_sim, expl));
        }

        if scored.is_empty() {
            return Ok(None);
        }

        let candidates_scored = scored.len();
        let roll = spark_field::selection_roll(
            &Utc::now().date_naive().to_string(),
            self.config.dice_seed.unwrap_or(0x5a_51_4b),
        );
        let ranked: Vec<(usize, f64)> = scored
            .iter()
            .enumerate()
            .map(|(i, (_, s, _, _))| (i, *s))
            .collect();
        let Some((idx, score)) = spark_field::soft_pick(
            ranked,
            self.config.soft_pick_top_k,
            self.config.soft_pick_temperature,
            roll,
        ) else {
            return Ok(None);
        };
        let (anchor, _, max_sim, expl) = scored.swap_remove(idx);

        let recent = if self.vault.cognition_uses_honeypot() {
            let associated = self
                .vault
                .cognition_associate_similar(&anchor.content, self.config.recent_limit)
                .await?;
            match spark_honeypot_recent(
                associated,
                anchor.id,
                &anchor.content,
                self.config.recent_limit,
            ) {
                Some(neighbors) => neighbors,
                None => {
                    info!(
                        anchor_id = %anchor.id,
                        "Spark honeypot associate returned no neighbors — skipping (fail closed)"
                    );
                    return Ok(None);
                }
            }
        } else {
            recent
                .into_iter()
                .filter(|f| f.id != anchor.id)
                .take(self.config.recent_limit)
                .collect()
        };

        if recent.is_empty() {
            return Ok(None);
        }

        info!(
            anchor_id = %anchor.id,
            anchor_decay = %anchor.decay_class,
            anchor_score = score,
            max_recent_sim = max_sim,
            recent_count = recent.len(),
            refractory_entries = field.entries.len(),
            refractory_mul = expl.multiplier,
            refractory_reason = expl.reason.as_str(),
            soft_pick_roll = roll,
            "Spark smart selection complete"
        );

        Ok(Some(SparkSelection {
            anchor,
            recent,
            selection_score: score,
            refractory_multiplier: expl.multiplier,
            refractory_reason: expl.reason.as_str(),
            refractory_entries: field.entries.len(),
            soft_pick_roll: roll,
            candidates_scored,
        }))
    }

    /// Skip session stubs, ops noise, and non-curated decay classes.
    fn is_viable_anchor(&self, fact: &SemanticFact) -> bool {
        if matches!(
            fact.decay_class.as_str(),
            "Procedural" | "Episodic" | "ArchivedSession"
        ) {
            return false;
        }
        let c = fact.content.trim();
        if c.is_empty() {
            return false;
        }
        for sub in &self.config.exclude_anchor_substrings {
            if c.contains(sub) {
                if let Some(session_date) = parse_session_date(c) {
                    let age = Utc::now()
                        .date_naive()
                        .signed_duration_since(session_date)
                        .num_days();
                    if age > self.config.max_session_anchor_age_days as i64 {
                        return false;
                    }
                } else if sub == "[Session " {
                    return false;
                }
            }
        }
        true
    }

    async fn hypothesis_phase(&self, selection: &SparkSelection) -> Result<SparkHypothesis> {
        let source = format_selection_bundle(selection);
        let messages = vec![
            Message {
                role: Role::System,
                content: concat!(
                    "You are a serendipity engine for long-term memory. Your job is to propose ",
                    "ONE new connection between an OLD fact and RECENT context.\n\n",
                    "Rules:\n",
                    "1. Use internal_analysis to reason first.\n",
                    "2. anchor_label / recent_label: short names for the two sides (from the SOURCE).\n",
                    "3. connection: 3-5 sentences — a hypothesis, analogy, or consequence. Not a summary.\n",
                    "4. what_to_remember: 1-2 bullets of durable insight (neural-finesse style).\n",
                    "5. Do NOT invent facts absent from the SOURCE. If nothing connects, set connection ",
                    "to empty string and what_to_remember to []."
                )
                .to_string(),
                is_meta: false,
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: Role::User,
                content: format!("SOURCE:\n---\n{source}\n---"),
                is_meta: false,
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let raw = self
            .gateway
            .complete_structured_bounded(
                &messages,
                "spark_hypothesis",
                hypothesis_schema(),
                Some(self.config.hypothesis_temperature),
                Some(self.config.max_tokens_hypothesis),
            )
            .await?;

        let mut h: SparkHypothesis = crate::gateway::parse_json_lenient(&raw)
            .map_err(|e| anyhow::anyhow!("Failed to parse spark hypothesis: {e}\nRaw: {raw}"))?;

        if h.connection.trim().is_empty() {
            anyhow::bail!("Model returned no connection — abstaining");
        }

        if h.connection.len() > self.config.max_connection_chars {
            h.connection.truncate(self.config.max_connection_chars);
            h.connection.push_str("…");
        }

        Ok(h)
    }

    async fn verify_phase(
        &self,
        selection: &SparkSelection,
        hypothesis: &SparkHypothesis,
    ) -> Result<SparkVerdict> {
        let source = format_selection_bundle(selection);
        let messages = vec![
            Message {
                role: Role::System,
                content: concat!(
                    "You are a strict fact-checker for serendipitous memory links.\n",
                    "Judge whether the HYPOTHESIS is supported by the SOURCE only.\n\n",
                    "Rules:\n",
                    "1. supported = true ONLY if BOTH the anchor and recent spans justify the connection.\n",
                    "2. evidence_anchor / evidence_recent: exact quotes copied from [A0] and [R#] lines in SOURCE (empty if unsupported).\n",
                    "3. Each evidence field must be at least 12 characters and appear verbatim in SOURCE.\n",
                    "4. confidence 0.0-1.0 — unsupported must be near 0.\n",
                    "5. Do NOT use outside knowledge.\n",
                    "6. If the hypothesis claims no relationship or is not a real directed link, supported=false."
                )
                .to_string(),
                is_meta: false,
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: Role::User,
                content: format!(
                    "SOURCE:\n---\n{source}\n---\n\nHYPOTHESIS:\n{}\n",
                    hypothesis.connection
                ),
                is_meta: false,
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let gw = self.resolve_gateway("verify");
        let raw = gw
            .complete_structured_bounded(
                &messages,
                "spark_verification",
                verification_schema(),
                Some(self.config.verify_temperature),
                Some(self.config.max_tokens_verify),
            )
            .await?;

        crate::gateway::parse_json_lenient(&raw)
            .map_err(|e| anyhow::anyhow!("Failed to parse spark verification: {e}\nRaw: {raw}"))
    }

    async fn promote_phase(
        &self,
        date: NaiveDate,
        selection: &SparkSelection,
        hypothesis: &SparkHypothesis,
        verdict: Option<&SparkVerdict>,
    ) -> usize {
        let from = if hypothesis.anchor_label.trim().is_empty() {
            entity_label_from_fact(&selection.anchor.content)
        } else {
            hypothesis.anchor_label.clone()
        };
        let to = if hypothesis.recent_label.trim().is_empty() {
            "RecentContext".to_string()
        } else {
            hypothesis.recent_label.clone()
        };

        let evidence = verdict
            .map(|v| {
                format!(
                    "anchor: \"{}\" | recent: \"{}\"",
                    v.evidence_anchor, v.evidence_recent
                )
            })
            .unwrap_or_default();

        let call = crate::gateway::ToolCall {
            id: format!("spark_kg_{}", Uuid::new_v4()),
            function_name: "mcp__memory__create_relations".to_string(),
            arguments: serde_json::json!({
                "relations": [{
                    "source": from,
                    "target": to,
                    "relationType": HYPOTHESIZED_LINK,
                }]
            }),
        };

        match self.tools.dispatch(&call).await {
            ToolResult { success: true, .. } => {
                info!(from = %from, to = %to, "Spark HYPOTHESIZED_LINK written");
                let audit = format!(
                    "[HYPOTHESIS {date}] {from} --({HYPOTHESIZED_LINK})--> {to}: {} | {evidence}",
                    hypothesis.connection
                );
                if let Err(e) =
                    self.vault
                        .store_text(&audit, "Episodic", self.config.quarantine_confidence)
                {
                    warn!("Spark audit quarantine write failed: {e}");
                }
                1
            }
            ToolResult { output, .. } => {
                warn!("Spark KG write failed (DREAMS.md still updated): {output}");
                0
            }
        }
    }

    fn format_spark_section(
        &self,
        date: NaiveDate,
        selection: &SparkSelection,
        hypothesis: &SparkHypothesis,
        promoted: bool,
        verdict: Option<&SparkVerdict>,
    ) -> String {
        let mut s = format!(
            "\n## Spark — {date}\n\nProcessed at {}\n\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        s.push_str("### Anchor (stale)\n\n");
        s.push_str(&format!("- {}\n", selection.anchor.content));

        s.push_str("\n### Recent context\n\n");
        for r in &selection.recent {
            s.push_str(&format!("- {}\n", r.content));
        }

        s.push_str("\n### Crystallized connection (hypothesis)\n\n");
        s.push_str(&format!("{}\n", hypothesis.connection));

        if !hypothesis.what_to_remember.is_empty() {
            s.push_str("\n### What to remember\n\n");
            for line in &hypothesis.what_to_remember {
                s.push_str(&format!("- {line}\n"));
            }
        }

        if let Some(v) = verdict {
            s.push_str(&format!(
                "\n### Verification — confidence {:.2} — promoted: {}\n",
                v.confidence, promoted
            ));
            if !v.evidence_anchor.is_empty() {
                s.push_str(&format!("- _anchor evidence:_ \"{}\"\n", v.evidence_anchor));
            }
            if !v.evidence_recent.is_empty() {
                s.push_str(&format!("- _recent evidence:_ \"{}\"\n", v.evidence_recent));
            }
        } else if !promoted {
            s.push_str("\n### Verification — abstained (not promoted)\n");
        }

        s
    }

    async fn log_episodic(&self, date: NaiveDate, section: &str) -> Result<()> {
        let summary = section.lines().take(8).collect::<Vec<_>>().join(" ");
        let entry = EpisodicEntry {
            timestamp: Utc::now(),
            source: EpisodicSource::InternalMonologue,
            content: format!("[spark {date}] {summary}"),
            is_silent: true,
        };
        self.episodic.append(&entry).await
    }
}

fn format_selection_bundle(selection: &SparkSelection) -> String {
    let mut out = String::from("ANCHOR [A0] (stale fact):\n");
    out.push_str(&selection.anchor.content);
    out.push_str("\n\nRECENT facts:\n");
    for (i, r) in selection.recent.iter().enumerate() {
        out.push_str(&format!("[R{i}] {}\n", r.content));
    }
    out
}

/// Promote only a supported, cited, above-threshold link.
fn spark_promote_ok(
    supported: bool,
    confidence: f64,
    citations_ok: bool,
    min_confidence: f64,
) -> bool {
    supported && citations_ok && confidence >= min_confidence
}

/// Honeypot-layer neighbors for the picked anchor (M3). `None` = skip spark.
fn spark_honeypot_recent(
    associated: Vec<SemanticFact>,
    anchor_id: Uuid,
    anchor_content: &str,
    recent_limit: usize,
) -> Option<Vec<SemanticFact>> {
    if anchor_content.trim().is_empty() {
        return None;
    }
    let recent: Vec<_> = associated
        .into_iter()
        .filter(|f| f.id != anchor_id)
        .take(recent_limit)
        .collect();
    if recent.is_empty() {
        None
    } else {
        Some(recent)
    }
}

/// Bulk ingest writes many facts in one second. That is not recent context.
fn recent_pool_is_ingest_slab(recent: &[SemanticFact]) -> bool {
    let Some(first) = recent.first() else {
        return false;
    };
    recent.len() >= 2
        && recent
            .iter()
            .all(|f| (f.created_at - first.created_at).num_seconds().abs() < 60)
}

/// Require quotable spans from anchor and at least one recent fact (LDR / dream firewall).
fn citations_valid(selection: &SparkSelection, verdict: &SparkVerdict, min_chars: usize) -> bool {
    let a = verdict.evidence_anchor.trim();
    let r = verdict.evidence_recent.trim();
    if a.len() < min_chars || r.len() < min_chars {
        return false;
    }
    anchor_citation_valid(&selection.anchor.content, a, min_chars)
        && recent_citation_valid(&selection.recent, r, min_chars)
}

fn anchor_citation_valid(anchor_content: &str, quote: &str, min_chars: usize) -> bool {
    quote.len() >= min_chars && source_contains_quote(anchor_content, quote)
}

/// When the verifier paraphrases, snap evidence to verbatim spans from vault facts.
fn repair_citations_from_facts(
    selection: &SparkSelection,
    verdict: &mut SparkVerdict,
    min_chars: usize,
) {
    if !anchor_citation_valid(
        &selection.anchor.content,
        verdict.evidence_anchor.trim(),
        min_chars,
    ) {
        if let Some(q) = first_quotable_span(&selection.anchor.content, min_chars) {
            verdict.evidence_anchor = q;
        }
    }
    if !recent_citation_valid(&selection.recent, verdict.evidence_recent.trim(), min_chars) {
        for fact in &selection.recent {
            if let Some(q) = first_quotable_span(&fact.content, min_chars) {
                verdict.evidence_recent = q;
                break;
            }
        }
    }
}

/// First sentence or prefix long enough for the citation firewall.
fn first_quotable_span(content: &str, min_chars: usize) -> Option<String> {
    let trimmed = content.trim();
    if trimmed.len() < min_chars {
        return None;
    }
    if let Some((_, rest)) = trimmed.split_once("] ") {
        let after_tag = rest.trim();
        if after_tag.len() >= min_chars {
            let end = after_tag
                .find(". ")
                .map(|i| i + 1)
                .unwrap_or(after_tag.len().min(240));
            return Some(after_tag[..end].trim().to_string());
        }
    }
    let end = trimmed
        .find(". ")
        .map(|i| i + 1)
        .unwrap_or(trimmed.len().min(240));
    let span = trimmed[..end].trim();
    if span.len() >= min_chars {
        Some(span.to_string())
    } else {
        Some(trimmed[..trimmed.len().min(240)].to_string())
    }
}

fn recent_citation_valid(recent: &[SemanticFact], quote: &str, min_chars: usize) -> bool {
    if quote.len() < min_chars {
        return false;
    }
    recent
        .iter()
        .any(|f| source_contains_quote(&f.content, quote))
}

fn source_contains_quote(source: &str, quote: &str) -> bool {
    if quote.is_empty() {
        return false;
    }
    if source.contains(quote) {
        return true;
    }
    let norm_src: String = source.split_whitespace().collect::<Vec<_>>().join(" ");
    let norm_q: String = quote.split_whitespace().collect::<Vec<_>>().join(" ");
    !norm_q.is_empty() && norm_src.contains(&norm_q)
}

fn days_since_created(fact: &SemanticFact) -> f64 {
    let now = Utc::now();
    (now - fact.created_at).num_seconds() as f64 / 86400.0
}

/// Triangular peak in the middle of the stale window (Generative-Agents-style sweet spot).
fn stale_sweetness(days: f64, min_days: f64, max_days: f64) -> f64 {
    if days < min_days || days > max_days || min_days >= max_days {
        return 0.0;
    }
    let mid = (min_days + max_days) / 2.0;
    if days <= mid {
        (days - min_days) / (mid - min_days)
    } else {
        (max_days - days) / (max_days - mid)
    }
}

fn anchor_importance(fact: &SemanticFact) -> f64 {
    let confirm_bonus = (fact.confirmation_count as f64).max(1.0).ln();
    let tags = extract_spark_concept_tags(&fact.content);
    let lineage_bonus = (tags.len() as f64) * 0.1;
    fact.confidence * (1.0 + confirm_bonus * 0.1 + lineage_bonus)
}

fn embeddings_usable(a: &[f32], b: &[f32]) -> bool {
    !a.is_empty() && !b.is_empty() && a.len() == b.len()
}

fn max_embedding_similarity(anchor: &SemanticFact, recent: &[SemanticFact]) -> f64 {
    recent
        .iter()
        .filter_map(|r| {
            if embeddings_usable(&anchor.embedding, &r.embedding) {
                Some(embedding_cosine_similarity(&anchor.embedding, &r.embedding))
            } else {
                None
            }
        })
        .fold(0.0_f64, f64::max)
}

/// Bracket tags from ingest (`[PEOPLE:Socrates]`, `[CONCEPT:…]`, etc.).
fn extract_spark_concept_tags(content: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find('[') {
        let after = &rest[start + 1..];
        if let Some(end) = after.find(']') {
            let inner = after[..end].trim();
            if let Some((kind, val)) = inner.split_once(':') {
                let kind = kind.trim().to_lowercase();
                let val = val.trim().to_lowercase();
                if !kind.is_empty() && !val.is_empty() {
                    tags.push(format!("{kind}:{val}"));
                }
            }
            rest = &after[end + 1..];
        } else {
            break;
        }
    }
    tags
}

fn lexical_overlap_score(a: &str, b: &str) -> f64 {
    let words_a: std::collections::HashSet<_> = a
        .split_whitespace()
        .filter(|w| w.len() > 4)
        .map(|w| w.to_lowercase())
        .collect();
    if words_a.is_empty() {
        return 0.0;
    }
    let shared = b
        .split_whitespace()
        .filter(|w| w.len() > 4)
        .filter(|w| words_a.contains(&w.to_lowercase()))
        .count();
    shared as f64 / words_a.len() as f64
}

fn anchor_passes_prefilter(
    anchor: &SemanticFact,
    recent: &[SemanticFact],
    min_sim: f64,
    tag_bridge: bool,
    max_sim: f64,
) -> bool {
    if tag_bridge || max_sim >= min_sim {
        return true;
    }
    recent
        .iter()
        .any(|r| lexical_overlap_score(&anchor.content, &r.content) >= 0.12)
}

fn shares_spark_concept_tag(a: &str, b: &str) -> bool {
    let ta = extract_spark_concept_tags(a);
    if ta.is_empty() {
        return false;
    }
    let tb = extract_spark_concept_tags(b);
    ta.iter().any(|t| tb.contains(t))
}

fn score_spark_anchor(
    anchor: &SemanticFact,
    recent: &[SemanticFact],
    min_stale_days: u32,
    max_stale_days: u32,
    min_sim: f64,
    tag_bridge: bool,
) -> f64 {
    let days = days_since_created(anchor);
    let stale = stale_sweetness(days, min_stale_days as f64, max_stale_days as f64);
    let imp = anchor_importance(anchor);
    let max_sim = max_embedding_similarity(anchor, recent);
    let sim_factor = if tag_bridge && max_sim < min_sim {
        min_sim
    } else {
        max_sim.max(min_sim)
    };
    imp * stale * sim_factor
}

fn dedupe_recent_facts(facts: Vec<SemanticFact>, max_similarity: f64) -> Vec<SemanticFact> {
    let mut kept: Vec<SemanticFact> = Vec::new();
    for fact in facts {
        let duplicate = kept
            .iter()
            .any(|k| embedding_cosine_similarity(&fact.embedding, &k.embedding) >= max_similarity);
        if !duplicate {
            kept.push(fact);
        }
    }
    kept
}

fn parse_session_date(content: &str) -> Option<NaiveDate> {
    let marker = "[Session ";
    let start = content.find(marker)? + marker.len();
    let date_str = content.get(start..start + 10)?;
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
}

#[cfg(test)]
mod selection_tests {
    use super::*;

    fn fact(content: &str, days_stale: i64) -> SemanticFact {
        let now = Utc::now();
        let accessed = now - chrono::Duration::days(days_stale);
        SemanticFact {
            id: Uuid::new_v4(),
            content: content.to_string(),
            embedding: vec![1.0, 0.0],
            half_life_days: 60.0,
            confidence: 0.95,
            confirmation_count: 2,
            decay_class: "CuratedVault".to_string(),
            created_at: accessed,
            last_accessed_at: accessed,
        }
    }

    #[test]
    fn stale_sweetness_peaks_mid_window() {
        let low = stale_sweetness(3.0, 3.0, 60.0);
        let mid = stale_sweetness(31.0, 3.0, 60.0);
        let high = stale_sweetness(60.0, 3.0, 60.0);
        assert!(mid > low);
        assert!(mid > high);
    }

    #[test]
    fn concept_tags_bridge_distinct_strings() {
        let a = "[CONCEPT:dialectics] Hegel thesis.";
        let b = "[CONCEPT:dialectics] Socratic method.";
        assert!(shares_spark_concept_tag(a, b));
        assert!(!shares_spark_concept_tag("no tags", b));
    }

    #[test]
    fn people_tags_bridge_ingest_facts() {
        let a = "[PEOPLE:Socrates] Developed the Socratic Elenchus.";
        let b = "[PEOPLE:Socrates] Attacked the alleged expertise of Athenians.";
        assert!(shares_spark_concept_tag(a, b));
    }

    #[test]
    fn anchor_importance_lineage_bonus() {
        let base = fact("Plain text without tags.", 0);
        let rich = fact("[CONCEPT:A] [CONCEPT:B] Connected text.", 0);
        let empty = fact("", 0);

        let score_base = anchor_importance(&base);
        let score_rich = anchor_importance(&rich);
        let score_empty = anchor_importance(&empty);

        assert!(score_rich > score_base);
        assert_eq!(score_base, score_empty);
    }

    #[test]
    fn dedupe_drops_near_duplicate_embeddings() {
        let a = fact("a", 1);
        let mut b = fact("b", 1);
        b.embedding = vec![1.0, 0.0];
        let out = dedupe_recent_facts(vec![a, b], 0.99);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn same_minute_recent_pool_is_ingest_slab() {
        let t = Utc::now();
        let mut a = fact("chatgpt traditional", 0);
        let mut b = fact("chatgpt fails personalization", 0);
        a.created_at = t;
        b.created_at = t + chrono::Duration::seconds(2);
        assert!(recent_pool_is_ingest_slab(&[a.clone(), b.clone()]));
        b.created_at = t + chrono::Duration::hours(2);
        assert!(!recent_pool_is_ingest_slab(&[a.clone(), b]));
        assert!(!recent_pool_is_ingest_slab(&[a]));
        assert!(!recent_pool_is_ingest_slab(&[]));
    }

    #[test]
    fn empty_honeypot_associate_yields_no_pair() {
        let anchor = fact("stale curated fact about dialectics", 30);
        assert!(spark_honeypot_recent(vec![], anchor.id, &anchor.content, 8).is_none());

        let mut only_self = fact("same-id neighbor", 1);
        only_self.id = anchor.id;
        assert!(spark_honeypot_recent(vec![only_self], anchor.id, &anchor.content, 8).is_none());

        assert!(spark_honeypot_recent(vec![fact("x", 1)], anchor.id, "   ", 8).is_none());

        let neighbor = fact("honeypot neighbor distillate", 1);
        let nid = neighbor.id;
        let out =
            spark_honeypot_recent(vec![anchor.clone(), neighbor], anchor.id, &anchor.content, 8)
                .expect("neighbors minus anchor is a pair");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, nid);
    }
}

#[cfg(test)]
mod citation_tests {
    use super::*;

    #[test]
    fn parse_session_anchor_date() {
        let d = parse_session_date("[Session 2026-04-08 16:09] foo").unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 4, 8).unwrap());
    }

    #[test]
    fn citations_require_substrings() {
        let selection = SparkSelection {
            anchor: SemanticFact {
                id: Uuid::new_v4(),
                content: "Socrates taught Plato.".into(),
                embedding: vec![],
                half_life_days: 60.0,
                confidence: 1.0,
                confirmation_count: 1,
                decay_class: "CuratedVault".into(),
                created_at: Utc::now(),
                last_accessed_at: Utc::now(),
            },
            recent: vec![SemanticFact {
                id: Uuid::new_v4(),
                content: "Hegel dialectics.".into(),
                embedding: vec![],
                half_life_days: 60.0,
                confidence: 1.0,
                confirmation_count: 1,
                decay_class: "CuratedVault".into(),
                created_at: Utc::now(),
                last_accessed_at: Utc::now(),
            }],
            selection_score: 1.0,
            refractory_multiplier: 1.0,
            refractory_reason: "none",
            refractory_entries: 0,
            soft_pick_roll: 0.0,
            candidates_scored: 1,
        };
        let ok = SparkVerdict {
            supported: true,
            confidence: 0.9,
            evidence_anchor: "Socrates taught Plato.".into(),
            evidence_recent: "Hegel dialectics.".into(),
        };
        assert!(citations_valid(&selection, &ok, 12));
        let bad = SparkVerdict {
            evidence_anchor: "fabricated".into(),
            evidence_recent: "Hegel dialectics.".into(),
            ..ok
        };
        assert!(!citations_valid(&selection, &bad, 12));
    }

    #[test]
    fn unsupported_verdict_does_not_promote_even_with_citations() {
        assert!(!spark_promote_ok(false, 0.6, true, 0.85));
        assert!(!spark_promote_ok(false, 0.95, true, 0.85));
        assert!(spark_promote_ok(true, 0.9, true, 0.85));
        assert!(!spark_promote_ok(true, 0.9, false, 0.85));
        assert!(!spark_promote_ok(true, 0.5, true, 0.85));
    }

    #[test]
    fn citations_can_be_valid_when_unsupported() {
        let selection = SparkSelection {
            anchor: SemanticFact {
                id: Uuid::new_v4(),
                content: "[SYSTEM:Awareness Agent] Kommunikation präziser Hardware-Statusmeldungen an den Strategy-Analyst"
                    .into(),
                embedding: vec![],
                half_life_days: 60.0,
                confidence: 1.0,
                confirmation_count: 1,
                decay_class: "CuratedVault".into(),
                created_at: Utc::now(),
                last_accessed_at: Utc::now(),
            },
            recent: vec![SemanticFact {
                id: Uuid::new_v4(),
                content: "[TOOL:FastAPI] Runs under Uvicorn and provides native WebSocket implementation."
                    .into(),
                embedding: vec![],
                half_life_days: 60.0,
                confidence: 1.0,
                confirmation_count: 1,
                decay_class: "CuratedVault".into(),
                created_at: Utc::now(),
                last_accessed_at: Utc::now(),
            }],
            selection_score: 1.0,
            refractory_multiplier: 1.0,
            refractory_reason: "none",
            refractory_entries: 0,
            soft_pick_roll: 0.0,
            candidates_scored: 1,
        };
        let v = SparkVerdict {
            supported: false,
            confidence: 0.0,
            evidence_anchor:
                "Kommunikation präziser Hardware-Statusmeldungen an den Strategy-Analyst".into(),
            evidence_recent: "Runs under Uvicorn and provides native WebSocket implementation."
                .into(),
        };
        assert!(citations_valid(&selection, &v, 12));
    }
}

fn hypothesis_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "internal_analysis": { "type": "string", "maxLength": 600 },
            "anchor_label": { "type": "string" },
            "recent_label": { "type": "string" },
            "connection": { "type": "string" },
            "what_to_remember": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        "required": ["internal_analysis", "anchor_label", "recent_label", "connection", "what_to_remember"],
        "additionalProperties": false
    })
}

fn verification_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "supported": { "type": "boolean" },
            "confidence": { "type": "number" },
            "evidence_anchor": { "type": "string" },
            "evidence_recent": { "type": "string" }
        },
        "required": ["supported", "confidence", "evidence_anchor", "evidence_recent"],
        "additionalProperties": false
    })
}

// ---------------------------------------------------------------------------
// Report + DREAMS.md append helper
// ---------------------------------------------------------------------------

pub struct SparkReport {
    pub date: NaiveDate,
    pub promoted: bool,
    pub kg_relations_written: usize,
    pub section: String,
}

impl SparkReport {
    fn skipped(date: NaiveDate, reason: &str) -> Self {
        Self {
            date,
            promoted: false,
            kg_relations_written: 0,
            section: format!("\n## Spark — {date}\n\nSkipped: {reason}\n"),
        }
    }
}

/// Append a spark section to DREAMS.md (preserves existing dream narrative and prior spark blocks).
pub async fn append_spark_to_dreams(path: &std::path::Path, section: &str) -> Result<()> {
    let existing = if path.exists() {
        tokio::fs::read_to_string(path).await.unwrap_or_default()
    } else {
        String::from("# Dream Consolidation\n\n")
    };
    let mut out = existing;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(section);
    tokio::fs::write(path, out).await?;
    Ok(())
}
