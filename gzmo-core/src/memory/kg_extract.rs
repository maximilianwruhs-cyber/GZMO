//! Shared extract → verify → promote pipeline for DreamEngine and IngestEngine.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::Result;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::gateway::{LlmGateway, ToolCall};
use crate::memory::kg_promotion::{
    canonicalize_relation_type, is_valid_entity_name, is_valid_relation_endpoints,
    normalize_entity_key, provenance_note, KG_BATCH_SIZE, MIN_EVIDENCE_CHARS,
};
use crate::tools::{ToolRegistry, ToolResult};
use crate::types::{Message, Role};

// ---------------------------------------------------------------------------
// Schema types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgEntity {
    pub name: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub observations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgRelation {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub relation_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgExtraction {
    #[allow(dead_code)]
    pub internal_analysis: String,
    pub entities: Vec<KgEntity>,
    pub relations: Vec<KgRelation>,
}

#[derive(Debug, Clone, Deserialize)]
struct Verdict {
    index: i64,
    supported: bool,
    confidence: f64,
    #[serde(default)]
    evidence: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerificationResult {
    entity_verdicts: Vec<Verdict>,
    relation_verdicts: Vec<Verdict>,
}

#[derive(Debug, Clone)]
pub struct VerifiedEntity {
    pub entity: KgEntity,
    pub confidence: f64,
    pub evidence: String,
}

#[derive(Debug, Clone)]
pub struct VerifiedRelation {
    pub relation: KgRelation,
    pub confidence: f64,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct VerifyStats {
    pub entities_dropped: usize,
    pub relations_dropped: usize,
}

// ---------------------------------------------------------------------------
// JSON schemas
// ---------------------------------------------------------------------------

pub fn extraction_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "internal_analysis": { "type": "string" },
            "entities": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "type": { "type": "string" },
                        "observations": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["name", "type", "observations"],
                    "additionalProperties": false
                }
            },
            "relations": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "from": { "type": "string" },
                        "to": { "type": "string" },
                        "type": { "type": "string" }
                    },
                    "required": ["from", "to", "type"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["internal_analysis", "entities", "relations"],
        "additionalProperties": false
    })
}

fn verification_schema() -> serde_json::Value {
    let verdict = serde_json::json!({
        "type": "object",
        "properties": {
            "index": { "type": "integer" },
            "supported": { "type": "boolean" },
            "confidence": { "type": "number" },
            "evidence": { "type": "string" }
        },
        "required": ["index", "supported", "confidence", "evidence"],
        "additionalProperties": false
    });
    serde_json::json!({
        "type": "object",
        "properties": {
            "entity_verdicts": { "type": "array", "items": verdict },
            "relation_verdicts": { "type": "array", "items": verdict }
        },
        "required": ["entity_verdicts", "relation_verdicts"],
        "additionalProperties": false
    })
}

// ---------------------------------------------------------------------------
// Normalization / dedupe
// ---------------------------------------------------------------------------

const NOISE_ENTITY_TYPES: &[&str] = &["CATEGORY", "SECTION", "DOMAIN", "FIELD", "TOPIC", "AREA"];

const GENERIC_ENTITY_NAMES: &[&str] = &[
    "algorithms",
    "data structures",
    "databases",
    "complexity theory",
    "computability",
    "distributed systems",
    "programming",
    "software engineering",
    "systems",
    "theory",
    "culture",
    "philosophy",
];

/// Drop section-header noise and generic category labels before verify/KG write.
pub fn filter_noise_entities(entities: Vec<KgEntity>) -> Vec<KgEntity> {
    entities
        .into_iter()
        .filter(|e| {
            let name_lc = e.name.trim().to_lowercase();
            if !is_valid_entity_name(&e.name) {
                return false;
            }
            let ty = e.entity_type.to_uppercase();
            if NOISE_ENTITY_TYPES.iter().any(|t| ty.contains(t)) {
                warn!(entity = %e.name, entity_type = %e.entity_type, "Dropped noise entity (type)");
                return false;
            }
            if GENERIC_ENTITY_NAMES.contains(&name_lc.as_str()) {
                warn!(entity = %e.name, "Dropped noise entity (generic label)");
                return false;
            }
            true
        })
        .collect()
}

/// Merge entities that normalize to the same key; keep the longest display name.
pub fn dedupe_entities(entities: Vec<KgEntity>) -> (Vec<KgEntity>, HashMap<String, String>) {
    let mut by_key: HashMap<String, KgEntity> = HashMap::new();
    let mut alias_map: HashMap<String, String> = HashMap::new();

    for e in entities {
        let key = normalize_entity_key(&e.name);
        match by_key.get_mut(&key) {
            Some(existing) => {
                alias_map.insert(e.name.clone(), existing.name.clone());
                if e.name.len() > existing.name.len() {
                    let old_canon = existing.name.clone();
                    existing.name = e.name.clone();
                    alias_map.insert(old_canon.clone(), existing.name.clone());
                    for canon in alias_map.values_mut() {
                        if *canon == old_canon {
                            *canon = existing.name.clone();
                        }
                    }
                }
                for obs in e.observations {
                    if !existing.observations.contains(&obs) {
                        existing.observations.push(obs);
                    }
                }
            }
            None => {
                alias_map.insert(e.name.clone(), e.name.clone());
                by_key.insert(key, e);
            }
        }
    }

    let mut entities: Vec<KgEntity> = by_key.into_values().collect();
    for e in &mut entities {
        e.observations.retain(|o| !o.trim().is_empty());
        e.name = e.name.trim().to_string();
        e.entity_type = e.entity_type.trim().to_string();
    }
    entities.retain(|e| {
        let ok = is_valid_entity_name(&e.name) && !e.observations.is_empty();
        if !ok {
            warn!(entity = %e.name, "Dropped entity (invalid name or no observations)");
        }
        ok
    });
    let (entities, alias_map) = merge_subset_aliases(entities, alias_map);
    (entities, alias_map)
}

/// Merge entities when one's name tokens are a subset of another (e.g. "Rivest" ⊂ "Ronald L. Rivest").
fn merge_subset_aliases(
    entities: Vec<KgEntity>,
    mut alias_map: HashMap<String, String>,
) -> (Vec<KgEntity>, HashMap<String, String>) {
    fn tokens(name: &str) -> HashSet<String> {
        normalize_entity_key(name)
            .split_whitespace()
            .map(str::to_string)
            .collect()
    }

    let mut merged: Vec<KgEntity> = Vec::new();
    let mut used = vec![false; entities.len()];

    for i in 0..entities.len() {
        if used[i] {
            continue;
        }
        let mut canon = entities[i].clone();
        let set_i = tokens(&canon.name);
        used[i] = true;

        for j in (i + 1)..entities.len() {
            if used[j] {
                continue;
            }
            let set_j = tokens(&entities[j].name);
            if set_j.is_subset(&set_i) || set_i.is_subset(&set_j) {
                used[j] = true;
                alias_map.insert(entities[j].name.clone(), canon.name.clone());
                if entities[j].name.len() > canon.name.len() {
                    let old = canon.name.clone();
                    canon.name = entities[j].name.clone();
                    alias_map.insert(old.clone(), canon.name.clone());
                    for v in alias_map.values_mut() {
                        if *v == old {
                            *v = canon.name.clone();
                        }
                    }
                }
                for obs in &entities[j].observations {
                    if !canon.observations.contains(obs) {
                        canon.observations.push(obs.clone());
                    }
                }
            }
        }
        merged.push(canon);
    }

    for e in &merged {
        alias_map.insert(e.name.clone(), e.name.clone());
    }
    (merged, alias_map)
}

/// Stats from deterministic pre-verify cleanup.
#[derive(Debug, Clone, Copy, Default)]
pub struct PrepareStats {
    pub raw_entities: usize,
    pub raw_relations: usize,
    pub dropped_preprocess: usize,
}

/// Filter noise, dedupe, canonicalize predicates, drop invalid endpoints.
pub fn prepare_candidates(
    entities: Vec<KgEntity>,
    relations: Vec<KgRelation>,
) -> (Vec<KgEntity>, Vec<KgRelation>, PrepareStats) {
    let raw_entities = entities.len();
    let raw_relations = relations.len();
    let filtered = filter_noise_entities(entities);
    let (entities, alias_map) = dedupe_entities(filtered);
    let relations_in: Vec<KgRelation> = relations
        .into_iter()
        .filter_map(|mut r| {
            r.from = r.from.trim().to_string();
            r.to = r.to.trim().to_string();
            r.relation_type = canonicalize_relation_type(&r.relation_type);
            if r.relation_type.is_empty() {
                warn!(from = %r.from, to = %r.to, "Dropped relation (forbidden or empty type)");
                return None;
            }
            Some(r)
        })
        .collect();
    let relations = dedupe_relations(relations_in, &alias_map);
    let relations: Vec<KgRelation> = relations
        .into_iter()
        .filter(|r| {
            let ok = is_valid_relation_endpoints(&r.from, &r.to, &r.relation_type);
            if !ok {
                warn!(from = %r.from, to = %r.to, ty = %r.relation_type, "Dropped invalid relation");
            }
            ok
        })
        .collect();
    let dropped_preprocess = raw_entities + raw_relations - entities.len() - relations.len();
    (
        entities,
        relations,
        PrepareStats {
            raw_entities,
            raw_relations,
            dropped_preprocess,
        },
    )
}

fn resolve_name(name: &str, alias_map: &HashMap<String, String>) -> String {
    alias_map
        .get(name)
        .cloned()
        .filter(|c| !c.is_empty())
        .unwrap_or_else(|| name.to_string())
}

/// Rewrite relation endpoints to canonical names and drop duplicate edges.
pub fn dedupe_relations(
    relations: Vec<KgRelation>,
    alias_map: &HashMap<String, String>,
) -> Vec<KgRelation> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for mut r in relations {
        r.from = resolve_name(&r.from, alias_map);
        r.to = resolve_name(&r.to, alias_map);
        if r.from == r.to {
            continue;
        }
        let rel = canonicalize_relation_type(&r.relation_type);
        if rel.is_empty() {
            continue;
        }
        let key = (
            normalize_entity_key(&r.from),
            normalize_entity_key(&r.to),
            rel.clone(),
        );
        if seen.insert(key) {
            r.relation_type = rel;
            out.push(r);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Promoter
// ---------------------------------------------------------------------------

/// Shared quality gates for dream, ingest, and future L2 pipelines.
#[derive(Debug, Clone)]
pub struct KgGateConfig {
    pub verify: bool,
    pub min_confidence: f64,
    pub verify_temperature: f32,
    pub require_evidence: bool,
    pub strict_kg: bool,
}

impl Default for KgGateConfig {
    fn default() -> Self {
        Self {
            verify: true,
            min_confidence: 0.85,
            verify_temperature: 0.1,
            require_evidence: true,
            strict_kg: true,
        }
    }
}

pub struct KgPromoter {
    pub gateway: Arc<dyn LlmGateway>,
    verify_gateway: Option<Arc<dyn LlmGateway>>,
    pub tools: Arc<ToolRegistry>,
    pub gate: KgGateConfig,
}

impl KgPromoter {
    pub fn new(gateway: Arc<dyn LlmGateway>, tools: Arc<ToolRegistry>, gate: KgGateConfig) -> Self {
        Self {
            gateway,
            verify_gateway: None,
            tools,
            gate,
        }
    }

    /// Fact-checker pass (defaults to `gateway` when unset).
    pub fn with_verify_gateway(mut self, verify_gateway: Arc<dyn LlmGateway>) -> Self {
        self.verify_gateway = Some(verify_gateway);
        self
    }

    fn verify_gateway(&self) -> &Arc<dyn LlmGateway> {
        self.verify_gateway.as_ref().unwrap_or(&self.gateway)
    }

    fn verdict_passes(&self, v: &Verdict, is_relation: bool) -> bool {
        if !v.supported || v.confidence < self.gate.min_confidence {
            return false;
        }
        if self.gate.require_evidence {
            let ev = v.evidence.trim();
            if ev.len() < MIN_EVIDENCE_CHARS {
                warn!(
                    supported = v.supported,
                    confidence = v.confidence,
                    is_relation,
                    evidence_len = ev.len(),
                    "Dropped: supported claim lacks quotable evidence"
                );
                return false;
            }
        }
        true
    }

    pub async fn run_pipeline(
        &self,
        source: &str,
        schema_name: &str,
        extract_system: &str,
        user_label: &str,
    ) -> Result<PipelineResult> {
        let extraction = self
            .extract(source, schema_name, extract_system, user_label)
            .await?;
        let raw_entities = extraction.entities.len();
        let raw_relations = extraction.relations.len();
        let (entities, relations, prep) =
            prepare_candidates(extraction.entities, extraction.relations);

        let (verified_entities, verified_relations, stats) = if self.gate.verify {
            let result = self.verify(source, &entities, &relations).await?;
            self.apply_verdicts(&entities, &relations, &result)
        } else {
            self.pass_through_verification(&entities, &relations)
        };
        Ok(PipelineResult {
            verified_entities,
            verified_relations,
            stats,
            prep,
            raw_entities,
            raw_relations,
            candidates_entities: entities.len(),
            candidates_relations: relations.len(),
            candidate_relations: relations,
        })
    }

    /// Extract per chunk, merge candidates, verify once against full source.
    pub async fn run_merged_pipeline(
        &self,
        full_source: &str,
        chunks: &[String],
        schema_name: &str,
        extract_system: &str,
        file_name: &str,
        skip_relation_verify: bool,
    ) -> Result<PipelineResult> {
        let mut extractions = Vec::with_capacity(chunks.len());
        let mut raw_entities = 0usize;
        let mut raw_relations = 0usize;

        for (i, chunk) in chunks.iter().enumerate() {
            let label = if chunks.len() == 1 {
                format!("Extract entities and relationships from this document ({file_name}):")
            } else {
                format!(
                    "Extract entities and relationships from this document ({file_name}, part {}/{})",
                    i + 1,
                    chunks.len()
                )
            };
            let extraction = self
                .extract(chunk, schema_name, extract_system, &label)
                .await?;
            raw_entities += extraction.entities.len();
            raw_relations += extraction.relations.len();
            extractions.push(extraction);
        }

        let merged = merge_extractions_pre_verify(extractions);
        let (entities, relations, prep) = prepare_candidates(merged.entities, merged.relations);

        let (verified_entities, verified_relations, stats) = if self.gate.verify {
            if skip_relation_verify {
                let result = self.verify(full_source, &entities, &[]).await?;
                let (verified_entities, _, stats) = self.apply_verdicts(&entities, &[], &result);
                let kept_names: std::collections::HashSet<&str> = verified_entities
                    .iter()
                    .map(|e| e.entity.name.as_str())
                    .collect();
                let agent_conf = self.gate.min_confidence.max(0.8);
                let verified_relations: Vec<VerifiedRelation> = relations
                    .iter()
                    .filter(|r| Self::relation_endpoints_match(&r.from, &r.to, &kept_names))
                    .map(|r| VerifiedRelation {
                        relation: r.clone(),
                        confidence: agent_conf,
                        evidence: String::new(),
                    })
                    .collect();
                (verified_entities, verified_relations, stats)
            } else {
                let result = self.verify(full_source, &entities, &relations).await?;
                self.apply_verdicts(&entities, &relations, &result)
            }
        } else {
            self.pass_through_verification(&entities, &relations)
        };

        Ok(PipelineResult {
            verified_entities,
            verified_relations,
            stats,
            prep,
            raw_entities,
            raw_relations,
            candidates_entities: entities.len(),
            candidates_relations: relations.len(),
            candidate_relations: relations,
        })
    }

    pub async fn extract(
        &self,
        source: &str,
        schema_name: &str,
        system_prompt: &str,
        user_label: &str,
    ) -> Result<KgExtraction> {
        let messages = vec![
            Message {
                role: Role::System,
                content: system_prompt.to_string(),
                is_meta: false,
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: Role::User,
                content: format!("{user_label}\n\n---\n{source}\n---"),
                is_meta: false,
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let raw = self
            .gateway
            .complete_structured(&messages, schema_name, extraction_schema())
            .await?;

        serde_json::from_str(&raw)
            .map_err(|e| anyhow::anyhow!("Failed to parse extraction JSON: {e}\nRaw: {raw}"))
    }

    pub async fn verify(
        &self,
        source: &str,
        entities: &[KgEntity],
        relations: &[KgRelation],
    ) -> Result<VerificationResult> {
        let mut listing = String::from("ENTITIES:\n");
        for (i, e) in entities.iter().enumerate() {
            listing.push_str(&format!(
                "E{i}: {} (type: {}) — observations: {}\n",
                e.name,
                e.entity_type,
                e.observations.join(" | ")
            ));
        }
        listing.push_str("\nRELATIONS:\n");
        for (i, r) in relations.iter().enumerate() {
            listing.push_str(&format!(
                "R{i}: {} --[{}]--> {}\n",
                r.from, r.relation_type, r.to
            ));
        }

        let messages = vec![
            Message {
                role: Role::System,
                content: concat!(
                    "You are a strict fact-checker guarding long-term memory against hallucination.\n",
                    "Judge EACH numbered item against the SOURCE only.\n\n",
                    "Rules:\n",
                    "1. supported = true ONLY if the SOURCE explicitly states or unambiguously implies the claim.\n",
                    "2. Put the exact supporting quote in 'evidence' (at least 12 characters). Empty if unsupported.\n",
                    "3. confidence 0.0-1.0 — unsupported => near 0.\n",
                    "4. Do NOT use outside knowledge.\n",
                    "5. RELATION supported only if SOURCE asserts a directed link; 'unrelated' => supported=false.\n",
                    "6. One verdict per candidate index (E/R number)."
                )
                .to_string(),
                is_meta: false,
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: Role::User,
                content: format!(
                    "SOURCE:\n---\n{source}\n---\n\nCANDIDATES:\n{listing}"
                ),
                is_meta: false,
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let raw = self
            .verify_gateway()
            .complete_structured_with_temp(
                &messages,
                "kg_verification",
                verification_schema(),
                Some(self.gate.verify_temperature),
            )
            .await?;

        serde_json::from_str(&raw)
            .map_err(|e| anyhow::anyhow!("Failed to parse verification JSON: {e}\nRaw: {raw}"))
    }

    fn entity_keys_match(a: &str, b: &str) -> bool {
        let ak = crate::memory::kg_promotion::normalize_entity_key(a);
        let bk = crate::memory::kg_promotion::normalize_entity_key(b);
        if ak.is_empty() || bk.is_empty() {
            return false;
        }
        ak == bk || ak.contains(&bk) || bk.contains(&ak)
    }

    fn relation_endpoints_match(from: &str, to: &str, kept: &HashSet<&str>) -> bool {
        kept.iter().any(|n| Self::entity_keys_match(from, n))
            && kept.iter().any(|n| Self::entity_keys_match(to, n))
    }

    pub fn apply_verdicts(
        &self,
        entities: &[KgEntity],
        relations: &[KgRelation],
        result: &VerificationResult,
    ) -> (Vec<VerifiedEntity>, Vec<VerifiedRelation>, VerifyStats) {
        let e_verdicts: HashMap<usize, &Verdict> = result
            .entity_verdicts
            .iter()
            .filter(|v| v.index >= 0)
            .filter_map(|v| {
                let idx = v.index as usize;
                if idx < entities.len() {
                    Some((idx, v))
                } else {
                    warn!(
                        index = v.index,
                        "Ignoring out-of-range entity verdict index"
                    );
                    None
                }
            })
            .collect();
        let r_verdicts: HashMap<usize, &Verdict> = result
            .relation_verdicts
            .iter()
            .filter(|v| v.index >= 0)
            .filter_map(|v| {
                let idx = v.index as usize;
                if idx < relations.len() {
                    Some((idx, v))
                } else {
                    warn!(
                        index = v.index,
                        "Ignoring out-of-range relation verdict index"
                    );
                    None
                }
            })
            .collect();
        for v in result.entity_verdicts.iter().filter(|v| v.index < 0) {
            warn!(
                index = v.index,
                "Ignoring invalid entity verdict index from verifier"
            );
        }
        for v in result.relation_verdicts.iter().filter(|v| v.index < 0) {
            warn!(
                index = v.index,
                "Ignoring invalid relation verdict index from verifier"
            );
        }

        let mut stats = VerifyStats::default();
        let mut kept_entities = Vec::new();

        for (i, e) in entities.iter().enumerate() {
            match e_verdicts.get(&i) {
                Some(v) if self.verdict_passes(v, false) => {
                    kept_entities.push(VerifiedEntity {
                        entity: e.clone(),
                        confidence: v.confidence,
                        evidence: v.evidence.clone(),
                    });
                }
                Some(v) => {
                    stats.entities_dropped += 1;
                    warn!(
                        entity = %e.name,
                        supported = v.supported,
                        confidence = v.confidence,
                        "Dropped unverified entity"
                    );
                }
                None => {
                    stats.entities_dropped += 1;
                    warn!(entity = %e.name, "Dropped entity (no verdict)");
                }
            }
        }

        let kept_names: HashSet<&str> = kept_entities
            .iter()
            .map(|e| e.entity.name.as_str())
            .collect();

        let mut kept_relations = Vec::new();
        for (i, r) in relations.iter().enumerate() {
            let endpoints_ok = Self::relation_endpoints_match(&r.from, &r.to, &kept_names);
            match r_verdicts.get(&i) {
                Some(v) if self.verdict_passes(v, true) && endpoints_ok => {
                    kept_relations.push(VerifiedRelation {
                        relation: r.clone(),
                        confidence: v.confidence,
                        evidence: v.evidence.clone(),
                    });
                }
                Some(v) => {
                    stats.relations_dropped += 1;
                    warn!(
                        from = %r.from,
                        to = %r.to,
                        supported = v.supported,
                        confidence = v.confidence,
                        endpoints_ok,
                        "Dropped unverified relation"
                    );
                }
                None => {
                    stats.relations_dropped += 1;
                    warn!(from = %r.from, to = %r.to, "Dropped relation (no verdict)");
                }
            }
        }

        (kept_entities, kept_relations, stats)
    }

    pub fn pass_through_verification(
        &self,
        entities: &[KgEntity],
        relations: &[KgRelation],
    ) -> (Vec<VerifiedEntity>, Vec<VerifiedRelation>, VerifyStats) {
        let conf = self.gate.min_confidence;
        let entities = entities
            .iter()
            .map(|e| VerifiedEntity {
                entity: e.clone(),
                confidence: conf,
                evidence: String::new(),
            })
            .collect();
        let relations = relations
            .iter()
            .map(|r| VerifiedRelation {
                relation: r.clone(),
                confidence: conf,
                evidence: String::new(),
            })
            .collect();
        (entities, relations, VerifyStats::default())
    }

    pub async fn promote_to_kg(
        &self,
        entities: &[VerifiedEntity],
        relations: &[VerifiedRelation],
        date: NaiveDate,
        provenance_label: &str,
    ) -> Result<(usize, usize)> {
        let mut entities_written = 0usize;
        let mut relations_written = 0usize;

        if !entities.is_empty() {
            for chunk in entities.chunks(KG_BATCH_SIZE) {
                let payload: Vec<serde_json::Value> = chunk
                    .iter()
                    .map(|ve| {
                        let mut observations = ve.entity.observations.clone();
                        let note = if provenance_label.is_empty() {
                            provenance_note(date, ve.confidence, &ve.evidence)
                        } else {
                            format!(
                                "{} {}",
                                provenance_note(date, ve.confidence, &ve.evidence),
                                provenance_label
                            )
                        };
                        observations.push(note);
                        serde_json::json!({
                            "name": ve.entity.name,
                            "type": ve.entity.entity_type,
                            "observations": observations,
                        })
                    })
                    .collect();

                let call = ToolCall {
                    id: format!("kg_entities_{}", Uuid::new_v4()),
                    function_name: "mcp__memory__create_entities".to_string(),
                    arguments: serde_json::json!({ "entities": payload }),
                };

                match self.tools.dispatch(&call).await {
                    ToolResult { success: true, .. } => {
                        entities_written += chunk.len();
                    }
                    ToolResult { output, .. } => {
                        if self.gate.strict_kg {
                            anyhow::bail!("KG entity batch write failed: {output}");
                        }
                        warn!("KG entity batch write failed (non-strict): {output}");
                    }
                }
            }
            info!(count = entities_written, "KG entities written");
        }

        if !relations.is_empty() {
            if self.gate.strict_kg && entities_written < entities.len() {
                anyhow::bail!(
                    "KG relations skipped: only {entities_written}/{} entities written",
                    entities.len()
                );
            }
            for chunk in relations.chunks(KG_BATCH_SIZE) {
                let payload: Vec<serde_json::Value> = chunk
                    .iter()
                    .filter_map(|vr| {
                        let rel_type = canonicalize_relation_type(&vr.relation.relation_type);
                        if rel_type.is_empty()
                            || !is_valid_relation_endpoints(
                                &vr.relation.from,
                                &vr.relation.to,
                                &rel_type,
                            )
                        {
                            return None;
                        }
                        Some(serde_json::json!({
                            "source": vr.relation.from,
                            "target": vr.relation.to,
                            "relationType": rel_type,
                        }))
                    })
                    .collect();

                if payload.is_empty() {
                    continue;
                }

                let call = ToolCall {
                    id: format!("kg_relations_{}", Uuid::new_v4()),
                    function_name: "mcp__memory__create_relations".to_string(),
                    arguments: serde_json::json!({ "relations": payload }),
                };

                let batch_len = payload.len();
                match self.tools.dispatch(&call).await {
                    ToolResult { success: true, .. } => {
                        relations_written += batch_len;
                    }
                    ToolResult { output, .. } => {
                        if self.gate.strict_kg {
                            anyhow::bail!("KG relation batch write failed: {output}");
                        }
                        warn!("KG relation batch write failed (non-strict): {output}");
                    }
                }
            }
            info!(count = relations_written, "KG relations written");
        }

        if self.gate.strict_kg {
            if entities_written != entities.len() {
                anyhow::bail!(
                    "Incomplete KG entity promotion: {entities_written}/{}",
                    entities.len()
                );
            }
            if relations_written != relations.len() {
                anyhow::bail!(
                    "Incomplete KG relation promotion: {relations_written}/{}",
                    relations.len()
                );
            }
        }

        Ok((entities_written, relations_written))
    }
}

/// Split long text into LLM-sized chunks (paragraph-aware).
pub fn chunk_text_for_llm(raw: &str, max_chars: usize) -> Vec<String> {
    if raw.len() <= max_chars {
        return vec![raw.to_string()];
    }
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < raw.len() {
        let mut end = (start + max_chars).min(raw.len());
        if end < raw.len() {
            if let Some(rel) = raw[start..end].rfind("\n\n") {
                if rel > max_chars / 4 {
                    end = start + rel + 2;
                }
            }
        }
        chunks.push(raw[start..end].to_string());
        start = end;
    }
    chunks
}

fn merge_verified_entity(existing: &mut VerifiedEntity, incoming: &VerifiedEntity) {
    if incoming.confidence > existing.confidence {
        existing.confidence = incoming.confidence;
        existing.evidence = incoming.evidence.clone();
    }
    for obs in &incoming.entity.observations {
        if !existing.entity.observations.contains(obs) {
            existing.entity.observations.push(obs.clone());
        }
    }
}

/// Merge raw extractions from multiple chunks before prepare/verify.
pub fn merge_extractions_pre_verify(extractions: Vec<KgExtraction>) -> KgExtraction {
    let mut entities = Vec::new();
    let mut relations = Vec::new();
    let mut internal_analysis = String::new();
    for ext in extractions {
        if !ext.internal_analysis.is_empty() {
            if !internal_analysis.is_empty() {
                internal_analysis.push('\n');
            }
            internal_analysis.push_str(&ext.internal_analysis);
        }
        entities.extend(ext.entities);
        relations.extend(ext.relations);
    }
    KgExtraction {
        internal_analysis,
        entities,
        relations,
    }
}

/// Merge multiple chunk pipelines into one promotion-ready result.
pub fn merge_pipeline_chunks(chunks: Vec<PipelineResult>) -> PipelineResult {
    use std::collections::HashMap;

    let mut stats = VerifyStats::default();
    let mut preprocess_dropped = 0usize;
    let mut raw_entities = 0usize;
    let mut raw_relations = 0usize;
    let mut candidates_entities = 0usize;
    let mut candidates_relations = 0usize;

    let mut entity_by_key: HashMap<String, VerifiedEntity> = HashMap::new();
    let mut relation_by_key: HashMap<String, VerifiedRelation> = HashMap::new();

    for p in chunks {
        stats.entities_dropped += p.stats.entities_dropped;
        stats.relations_dropped += p.stats.relations_dropped;
        preprocess_dropped += p.prep.dropped_preprocess;
        raw_entities += p.raw_entities;
        raw_relations += p.raw_relations;
        candidates_entities += p.candidates_entities;
        candidates_relations += p.candidates_relations;

        for ve in p.verified_entities {
            let key = crate::memory::kg_promotion::normalize_entity_key(&ve.entity.name);
            entity_by_key
                .entry(key)
                .and_modify(|existing| merge_verified_entity(existing, &ve))
                .or_insert(ve);
        }
        for vr in p.verified_relations {
            let key = format!(
                "{}|{}|{}",
                crate::memory::kg_promotion::normalize_entity_key(&vr.relation.from),
                crate::memory::kg_promotion::normalize_entity_key(&vr.relation.to),
                vr.relation.relation_type
            );
            relation_by_key
                .entry(key)
                .and_modify(|existing| {
                    if vr.confidence > existing.confidence {
                        existing.confidence = vr.confidence;
                        existing.evidence = vr.evidence.clone();
                    }
                })
                .or_insert(vr);
        }
    }

    let entities: Vec<_> = entity_by_key.values().map(|v| v.entity.clone()).collect();
    let (deduped_entities, alias_map) = dedupe_entities(entities);

    let mut verified_entities: Vec<VerifiedEntity> = Vec::new();
    for e in deduped_entities {
        let key = crate::memory::kg_promotion::normalize_entity_key(&e.name);
        let confidence = entity_by_key.get(&key).map(|v| v.confidence).unwrap_or(0.0);
        let evidence = entity_by_key
            .get(&key)
            .map(|v| v.evidence.clone())
            .unwrap_or_default();
        verified_entities.push(VerifiedEntity {
            entity: e,
            confidence,
            evidence,
        });
    }

    let relations: Vec<_> = relation_by_key
        .values()
        .map(|v| v.relation.clone())
        .collect();
    let deduped_relations = dedupe_relations(relations, &alias_map);
    let verified_relations: Vec<VerifiedRelation> = deduped_relations
        .into_iter()
        .map(|r| {
            let key = format!(
                "{}|{}|{}",
                crate::memory::kg_promotion::normalize_entity_key(&r.from),
                crate::memory::kg_promotion::normalize_entity_key(&r.to),
                r.relation_type
            );
            let (confidence, evidence) = relation_by_key
                .get(&key)
                .map(|v| (v.confidence, v.evidence.clone()))
                .unwrap_or((0.0, String::new()));
            VerifiedRelation {
                relation: r,
                confidence,
                evidence,
            }
        })
        .collect();

    PipelineResult {
        verified_entities,
        verified_relations,
        stats,
        prep: PrepareStats {
            raw_entities,
            raw_relations,
            dropped_preprocess: preprocess_dropped,
        },
        raw_entities,
        raw_relations,
        candidates_entities,
        candidates_relations,
        candidate_relations: relation_by_key
            .values()
            .map(|v| v.relation.clone())
            .collect(),
    }
}

/// Output of extract → prepare → verify (ready for KG + vault).
pub struct PipelineResult {
    pub verified_entities: Vec<VerifiedEntity>,
    pub verified_relations: Vec<VerifiedRelation>,
    pub stats: VerifyStats,
    pub prep: PrepareStats,
    pub raw_entities: usize,
    pub raw_relations: usize,
    pub candidates_entities: usize,
    pub candidates_relations: usize,
    /// Merged relations before verify (AgentSpec relink after primary-agent inject).
    pub candidate_relations: Vec<KgRelation>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedupe_merges_author_aliases() {
        let entities = vec![
            KgEntity {
                name: "Rivest".into(),
                entity_type: "AUTHOR".into(),
                observations: vec!["co-author CLRS".into()],
            },
            KgEntity {
                name: "Ronald L. Rivest".into(),
                entity_type: "AUTHOR".into(),
                observations: vec!["CLRS".into()],
            },
        ];
        let (deduped, _) = dedupe_entities(entities);
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].name, "Ronald L. Rivest");
    }

    #[test]
    fn dedupe_relations_collapses_synonyms() {
        let mut alias = HashMap::new();
        alias.insert("Rivest".into(), "Ronald L. Rivest".into());
        let rels = vec![
            KgRelation {
                from: "Rivest".into(),
                to: "CLRS".into(),
                relation_type: "AUTHORED".into(),
            },
            KgRelation {
                from: "Ronald L. Rivest".into(),
                to: "CLRS".into(),
                relation_type: "AUTHOR".into(),
            },
        ];
        let out = dedupe_relations(rels, &alias);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn apply_verdicts_ignores_negative_and_out_of_range_indices() {
        use crate::gateway::{LlmGateway, LlmResponse, ToolDeclaration};
        use crate::tools::ToolRegistry;
        use crate::types::Message;
        use async_trait::async_trait;
        use std::sync::Arc;

        struct NoopGateway;

        #[async_trait]
        impl LlmGateway for NoopGateway {
            async fn complete(
                &self,
                _: &[Message],
                _: &[ToolDeclaration],
            ) -> anyhow::Result<LlmResponse> {
                anyhow::bail!("noop")
            }
            async fn complete_streaming(
                &self,
                _: &[Message],
                _: &[ToolDeclaration],
                _: Box<dyn Fn(String) + Send>,
            ) -> anyhow::Result<LlmResponse> {
                anyhow::bail!("noop")
            }
            async fn complete_structured(
                &self,
                _: &[Message],
                _: &str,
                _: serde_json::Value,
            ) -> anyhow::Result<String> {
                anyhow::bail!("noop")
            }
        }

        let promoter = KgPromoter::new(
            Arc::new(NoopGateway),
            Arc::new(ToolRegistry::new()),
            KgGateConfig {
                min_confidence: 0.5,
                require_evidence: false,
                ..Default::default()
            },
        );
        let entities = vec![KgEntity {
            name: "Firewall Agent".into(),
            entity_type: "AGENT".into(),
            observations: vec!["Monitors network policy".into()],
        }];
        let result = VerificationResult {
            entity_verdicts: vec![
                Verdict {
                    index: -1,
                    supported: true,
                    confidence: 0.99,
                    evidence: "ignored negative index".into(),
                },
                Verdict {
                    index: 99,
                    supported: true,
                    confidence: 0.99,
                    evidence: "ignored out of range".into(),
                },
                Verdict {
                    index: 0,
                    supported: true,
                    confidence: 0.95,
                    evidence: "valid supporting quote here".into(),
                },
            ],
            relation_verdicts: vec![],
        };
        let (kept, _, stats) = promoter.apply_verdicts(&entities, &[], &result);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].entity.name, "Firewall Agent");
        assert_eq!(stats.entities_dropped, 0);
    }
}
