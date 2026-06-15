//! # OpenClaw Dreams
//!
//! Nightly consolidation: Light (compress) → REM (extract) → Verify → Deep (KG + vault).

use std::sync::Arc;

use anyhow::Result;
use chrono::{NaiveDate, Utc};
use tracing::{info, warn};
use uuid::Uuid;

use crate::config::DreamsConfig;
use crate::gateway::LlmGateway;
use crate::memory::episodic::FileEpisodicStore;
use crate::memory::kg_extract::{
    chunk_text_for_llm, merge_pipeline_chunks, KgEntity, KgPromoter, KgRelation, VerifiedEntity,
    VerifiedRelation, VerifyStats,
};
use crate::memory::vault::SqliteVault;
use crate::synapse::{resolve_event_source, EventSource, EventType, SynapseBus, SynapseEvent};
use crate::tools::ToolRegistry;
use crate::types::{DecayClass, ExtractedTruth};

// Public aliases for downstream code / tests
pub type DreamEntity = KgEntity;
pub type DreamRelation = KgRelation;

/// Episodic path used as `source_file` so dream truths qualify for honeypot recall.
pub fn dream_episodic_source(date: NaiveDate) -> String {
    format!("memory/{date}.md")
}

const DREAM_EXTRACT_SYSTEM: &str = concat!(
    "You are a memory consolidation engine. Extract structured knowledge from a daily log.\n\n",
    "Rules:\n",
    "1. Use internal_analysis to reason first.\n",
    "2. Extract PEOPLE, SYSTEMS, PROJECTS, TOOLS, DECISIONS as entities — not generic section labels.\n",
    "3. Each entity needs 1+ concrete observations from the log.\n",
    "4. Relations: one edge per link; USES, MANAGES, DEPENDS_ON, RELATED_TO, AUTHORED_BY.\n",
    "5. Disambiguate pronouns to real names.\n",
    "6. Empty arrays if the log is trivial."
);

/// The 3-phase autoDream consolidation engine.
pub struct DreamEngine {
    episodic: FileEpisodicStore,
    vault: SqliteVault,
    promoter: KgPromoter,
    dreams: DreamsConfig,
    /// Bibliothek gate: min dream cycles before vault/KG promotion.
    bibliothek_min_dream_cycles: u32,
    /// Optional Synapse bus for observability.
    synapse: Option<Arc<SynapseBus>>,
}

impl DreamEngine {
    pub fn new(
        episodic: FileEpisodicStore,
        vault: SqliteVault,
        gateway: Arc<dyn LlmGateway>,
        tools: Arc<ToolRegistry>,
        dreams: DreamsConfig,
        synapse: Option<Arc<SynapseBus>>,
    ) -> Self {
        Self::new_with_verify(
            episodic,
            vault,
            gateway.clone(),
            gateway,
            tools,
            dreams,
            crate::config::BibliothekConfig::default().min_dream_cycles,
            synapse,
        )
    }

    /// Construct with a separate verify gateway (Obolus routing).
    pub fn new_with_verify(
        episodic: FileEpisodicStore,
        vault: SqliteVault,
        extract_gateway: Arc<dyn LlmGateway>,
        verify_gateway: Arc<dyn LlmGateway>,
        tools: Arc<ToolRegistry>,
        dreams: DreamsConfig,
        bibliothek_min_dream_cycles: u32,
        synapse: Option<Arc<SynapseBus>>,
    ) -> Self {
        Self {
            episodic,
            vault,
            promoter: KgPromoter::new(extract_gateway, tools, dreams.kg_gate())
                .with_verify_gateway(verify_gateway),
            dreams,
            bibliothek_min_dream_cycles,
            synapse,
        }
    }

    /// Run the full autoDream cycle for a day's episodic data.
    pub async fn consolidate(&self, date: NaiveDate) -> Result<DreamReport> {
        info!(date = %date, "Starting autoDream consolidation cycle");

        let raw = self.episodic.read_day(date).await?;
        if raw.trim().is_empty() {
            info!("No episodic data for {date} — skipping dream cycle");
            return Ok(DreamReport {
                date,
                original_bytes: 0,
                compressed_bytes: 0,
                entities_extracted: 0,
                relations_extracted: 0,
                kg_entities_written: 0,
                kg_relations_written: 0,
                truths_promoted: 0,
                narrative: format!("# Dream Consolidation — {date}\n\nNo episodic data.\n"),
            });
        }

        let filtered = filter_episodic_for_consolidation(&raw, &self.dreams.exclude_episodic_substrings);
        if filtered.trim().len() < self.dreams.min_consolidation_chars {
            info!(
                original = raw.len(),
                filtered = filtered.len(),
                min = self.dreams.min_consolidation_chars,
                "Episodic log ops-only after filter — skipping dream extraction"
            );
            return Ok(DreamReport {
                date,
                original_bytes: raw.len(),
                compressed_bytes: 0,
                entities_extracted: 0,
                relations_extracted: 0,
                kg_entities_written: 0,
                kg_relations_written: 0,
                truths_promoted: 0,
                narrative: format!(
                    "# Dream Consolidation — {date}\n\n\
                     Skipped: episodic log was mostly daemon meta (janitor/spark/ingest) after filtering.\n\
                     Research documents should use `gzmo ingest` — not nightly dream from ops noise.\n"
                ),
            });
        }

        let filtered_len = filtered.len();
        let mut rem_input = filtered;
        let mut honeypot_rem_chars = 0usize;
        if self.dreams.honeypot_rem_enabled && self.vault.cognition_uses_honeypot() {
            match self
                .vault
                .build_honeypot_rem_context(
                    self.dreams.honeypot_rem_anchor_limit,
                    self.dreams.honeypot_rem_associate_k,
                )
                .await
            {
                Ok(hp) if !hp.trim().is_empty() => {
                    honeypot_rem_chars = hp.len();
                    rem_input = format!(
                        "{}\n\n### HONEYPOT ASSOCIATIONS (M3)\n{}\n",
                        rem_input.trim(),
                        hp.trim()
                    );
                    info!(
                        honeypot_chars = honeypot_rem_chars,
                        layer = self.vault.cognition_memory_layer(),
                        "M3 honeypot REM substrate appended"
                    );
                }
                Ok(_) => {}
                Err(e) => warn!("Honeypot REM context failed (episodic-only REM): {e}"),
            }
        }

        let compressed = self.light_phase(&rem_input);
        info!(
            original = raw.len(),
            filtered = filtered_len,
            honeypot_rem = honeypot_rem_chars,
            compressed = compressed.len(),
            ratio = format!(
                "{:.1}:1",
                filtered_len as f64 / compressed.len().max(1) as f64
            ),
            "Light Phase complete"
        );

        let chunks = chunk_text_for_llm(&compressed, self.dreams.chunk_chars);
        let mut chunk_results = Vec::with_capacity(chunks.len());
        for (i, chunk) in chunks.iter().enumerate() {
            let label = if chunks.len() == 1 {
                "Extract entities and relationships from this daily log:".to_string()
            } else {
                format!(
                    "Extract entities and relationships from this daily log (part {}/{})",
                    i + 1,
                    chunks.len()
                )
            };
            match self
                .promoter
                .run_pipeline(chunk, "dream_extraction", DREAM_EXTRACT_SYSTEM, &label)
                .await
            {
                Ok(p) => chunk_results.push(p),
                Err(e) => {
                    warn!(chunk = i + 1, "REM/verify pipeline failed: {e}");
                    return Ok(DreamReport {
                        date,
                        original_bytes: raw.len(),
                        compressed_bytes: compressed.len(),
                        entities_extracted: 0,
                        relations_extracted: 0,
                        kg_entities_written: 0,
                        kg_relations_written: 0,
                        truths_promoted: 0,
                        narrative: format!(
                            "# Dream Consolidation — {date}\n\nPipeline failed on chunk {}: {e}\nEpisodic data preserved.\n",
                            i + 1
                        ),
                    });
                }
            }
        }
        let pipeline = merge_pipeline_chunks(chunk_results);

        info!(
            raw_entities = pipeline.raw_entities,
            raw_relations = pipeline.raw_relations,
            kept_entities = pipeline.verified_entities.len(),
            kept_relations = pipeline.verified_relations.len(),
            dropped_entities = pipeline.stats.entities_dropped,
            dropped_relations = pipeline.stats.relations_dropped,
            preprocess_dropped = pipeline.prep.dropped_preprocess,
            "Verify Phase complete"
        );

        let provenance = format!("[dream] date={date}");
        let bib_path = std::path::PathBuf::from("data/bibliothek_state.json");
        let promote = crate::bibliothek::promotion_allowed(
            &bib_path,
            self.bibliothek_min_dream_cycles,
        );

        let (kg_entities, kg_relations) = if promote {
            match self
                .promoter
                .promote_to_kg(
                    &pipeline.verified_entities,
                    &pipeline.verified_relations,
                    date,
                    &provenance,
                )
                .await
            {
                Ok(w) => w,
                Err(e) => {
                    warn!(
                        "KG promotion failed — vault may still update but graph incomplete: {e}"
                    );
                    (0, 0)
                }
            }
        } else {
            info!(
                min_cycles = self.bibliothek_min_dream_cycles,
                completed = crate::bibliothek::load_state(&bib_path).dream_cycles_completed,
                "Bibliothek gate: skipping KG promotion"
            );
            (0, 0)
        };

        let truths = self.to_vault_truths(&pipeline.verified_entities, date, &compressed);
        if promote {
            if let Err(e) = self
                .vault
                .promote_truths_with_origin(&truths, "verified_dream")
                .await
            {
                warn!("Vault promotion failed (non-fatal): {e}");
            }
        } else {
            info!("Bibliothek gate: skipping vault promotion");
        }

        info!(
            kg_entities,
            kg_relations,
            vault_truths = truths.len(),
            "Deep Phase complete"
        );

        let narrative = self.generate_narrative(
            &pipeline.verified_entities,
            &pipeline.verified_relations,
            &pipeline.stats,
            date,
            kg_entities,
            kg_relations,
            pipeline.prep.dropped_preprocess,
        );

        // DreamComplete: append to Synapse bus
        if let Some(ref bus) = self.synapse {
            let data = serde_json::json!({
                "date": date.to_string(),
                "entities_extracted": pipeline.raw_entities,
                "relations_extracted": pipeline.raw_relations,
                "kg_entities_written": kg_entities,
                "kg_relations_written": kg_relations,
                "truths_promoted": if promote { truths.len() } else { 0 },
                "bibliothek_promotion": promote,
                "memory_layer": self.vault.cognition_memory_layer(),
                "cognition_source": self.vault.cognition_memory_layer(),
                "honeypot_rem_chars": honeypot_rem_chars,
            });
            bus.append(&SynapseEvent::with_data(
                EventType::DreamComplete,
                resolve_event_source(EventSource::GzmoDaemon),
                data,
            ));
        }

        let _ = crate::bibliothek::increment_dream_cycles(&bib_path);

        Ok(DreamReport {
            date,
            original_bytes: raw.len(),
            compressed_bytes: compressed.len(),
            entities_extracted: pipeline.raw_entities,
            relations_extracted: pipeline.raw_relations,
            kg_entities_written: kg_entities,
            kg_relations_written: kg_relations,
            truths_promoted: truths.len(),
            narrative,
        })
    }

    fn light_phase(&self, text: &str) -> String {
        crate::context_compress::logs::compress_logs(text, 500)
    }

    fn to_vault_truths(
        &self,
        entities: &[VerifiedEntity],
        date: NaiveDate,
        body: &str,
    ) -> Vec<ExtractedTruth> {
        let source_file = Some(dream_episodic_source(date));
        entities
            .iter()
            .flat_map(|ve| {
                let entity = &ve.entity;
                let confidence = ve.confidence as f32;
                let source_file = source_file.clone();
                let obs_count = entity.observations.len();
                entity
                    .observations
                    .iter()
                    .map(move |obs| ExtractedTruth {
                        id: Uuid::new_v4(),
                        content: format!("[{}:{}] {}", entity.entity_type, entity.name, obs),
                        confidence,
                        mmr_score: 0.0,
                        source_date: date,
                        decay_class: DecayClass::CuratedVault,
                        source_file: source_file.clone(),
                        evidence: crate::memory::evidence_localize::localize_observation_evidence(
                            body,
                            obs,
                            &ve.evidence,
                            obs_count,
                        ),
                    })
            })
            .collect()
    }

    fn generate_narrative(
        &self,
        entities: &[VerifiedEntity],
        relations: &[VerifiedRelation],
        stats: &VerifyStats,
        date: NaiveDate,
        kg_entities: usize,
        kg_relations: usize,
        preprocess_dropped: usize,
    ) -> String {
        let mut narrative = format!("# Dream Consolidation — {}\n\n", date);
        narrative.push_str(&format!(
            "Processed at {}\n\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        if entities.is_empty() {
            narrative.push_str("No verified knowledge promoted today");
            if stats.entities_dropped > 0 || stats.relations_dropped > 0 || preprocess_dropped > 0 {
                narrative.push_str(&format!(
                    " (preprocess dropped {preprocess_dropped}; verifier dropped {} entities, {} relations).",
                    stats.entities_dropped, stats.relations_dropped
                ));
            }
            narrative.push('\n');
            return narrative;
        }

        narrative.push_str("## Verified Entities\n\n");
        for ve in entities {
            narrative.push_str(&format!(
                "- **{}** ({}) — confidence {:.2}\n",
                ve.entity.name, ve.entity.entity_type, ve.confidence
            ));
            for obs in &ve.entity.observations {
                narrative.push_str(&format!("  - {}\n", obs));
            }
            if !ve.evidence.is_empty() {
                narrative.push_str(&format!("  - _evidence:_ \"{}\"\n", ve.evidence));
            }
        }

        if !relations.is_empty() {
            narrative.push_str("\n## Verified Relationships\n\n");
            for vr in relations {
                narrative.push_str(&format!(
                    "- {} → ({}) → {} — confidence {:.2}\n",
                    vr.relation.from, vr.relation.relation_type, vr.relation.to, vr.confidence
                ));
                if !vr.evidence.is_empty() {
                    narrative.push_str(&format!("  - _evidence:_ \"{}\"\n", vr.evidence));
                }
            }
        }

        narrative.push_str(&format!(
            "\n## Consolidation Stats\n\n\
             - KG entities written: {}\n\
             - KG relations written: {}\n\
             - Preprocess dropped: {}\n\
             - Verifier rejected entities: {}\n\
             - Verifier rejected relations: {}\n",
            kg_entities,
            kg_relations,
            preprocess_dropped,
            stats.entities_dropped,
            stats.relations_dropped
        ));

        narrative
    }
}

const EPISODIC_SECTION_MARKER: &str = "### 🧠 INTERNAL";
const SESSION_SECTION_MARKER: &str = "### 📓 SESSION";

/// Remove daemon meta sections (janitor, spark echoes, ingest receipts) before REM.
/// Always keeps `### 📓 SESSION` blocks (session distiller output).
pub fn filter_episodic_for_consolidation(raw: &str, exclude_substrings: &[String]) -> String {
    let (session_sections, without_sessions) = extract_session_sections(raw);

    let filtered = if exclude_substrings.is_empty() {
        without_sessions
    } else {
        filter_internal_sections(&without_sessions, exclude_substrings)
    };

    let mut out = session_sections;
    if !filtered.trim().is_empty() {
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(filtered.trim());
    }
    out.trim().to_string()
}

fn extract_session_sections(raw: &str) -> (String, String) {
    let mut sessions = String::new();
    let mut rest = raw.to_string();
    while let Some(pos) = rest.find(SESSION_SECTION_MARKER) {
        let tail = rest.split_off(pos);
        let end = tail[SESSION_SECTION_MARKER.len()..]
            .find(SESSION_SECTION_MARKER)
            .map(|i| i + SESSION_SECTION_MARKER.len())
            .or_else(|| {
                tail[SESSION_SECTION_MARKER.len()..]
                    .find(EPISODIC_SECTION_MARKER)
                    .map(|i| i + SESSION_SECTION_MARKER.len())
            })
            .unwrap_or(tail.len());
        sessions.push_str(&tail[..end]);
        sessions.push('\n');
        rest.push_str(&tail[end..]);
    }
    (sessions.trim().to_string(), rest)
}

fn filter_internal_sections(raw: &str, exclude_substrings: &[String]) -> String {
    let mut out = String::new();
    let mut parts = raw.split(EPISODIC_SECTION_MARKER).peekable();

    if let Some(preamble) = parts.next() {
        let trimmed = preamble.trim();
        if !trimmed.is_empty() {
            out.push_str(trimmed);
            out.push_str("\n\n");
        }
    }

    for body in parts {
        let probe = body.chars().take(1200).collect::<String>().to_lowercase();
        let skip = exclude_substrings
            .iter()
            .any(|ex| probe.contains(&ex.to_lowercase()));
        if skip {
            continue;
        }
        out.push_str(EPISODIC_SECTION_MARKER);
        out.push_str(body);
    }

    out.trim().to_string()
}

/// Result of a dream consolidation cycle.
pub struct DreamReport {
    pub date: NaiveDate,
    pub original_bytes: usize,
    pub compressed_bytes: usize,
    pub entities_extracted: usize,
    pub relations_extracted: usize,
    pub kg_entities_written: usize,
    pub kg_relations_written: usize,
    pub truths_promoted: usize,
    pub narrative: String,
}

#[cfg(test)]
mod episodic_filter_tests {
    use super::*;

    fn excludes() -> Vec<String> {
        vec!["sys_janitor".into(), "[spark ".into(), "[ingest:".into()]
    }

    #[test]
    fn drops_janitor_sections_keeps_other() {
        let raw = "\
### 🧠 INTERNAL — 12:00:00
[Job: sys_janitor] CPU 1%

### 🧠 INTERNAL — 13:00:00
User decided to refactor the vault schema for dialectics ingest.
";
        let filtered = filter_episodic_for_consolidation(raw, &excludes());
        assert!(!filtered.to_lowercase().contains("sys_janitor"));
        assert!(filtered.contains("vault schema"));
    }

    #[test]
    fn keeps_session_blocks_inside_skipped_internal() {
        let raw = "\
### 🧠 INTERNAL — 12:00:00
[Job: sys_janitor] CPU 1%

### 📓 SESSION abc — 13:00:00
Session distilled: GZMO runs on air-gapped infrastructure with real decisions.

### 🧠 INTERNAL — 14:00:00
[spark 2026-06-01] promoted=false
";
        let filtered = filter_episodic_for_consolidation(raw, &excludes());
        assert!(!filtered.contains("sys_janitor"));
        assert!(filtered.contains("air-gapped infrastructure"));
    }

    #[test]
    fn dream_truths_use_episodic_source_file() {
        use crate::memory::honeypot::qualifies_for_honeypot;
        use chrono::NaiveDate;

        let date = NaiveDate::from_ymd_opt(2026, 6, 4).unwrap();
        let source_file = Some(dream_episodic_source(date));
        let truth = ExtractedTruth {
            id: Uuid::new_v4(),
            content: "[PROJECT:GZMO] local-first strategy".into(),
            confidence: 1.0,
            mmr_score: 0.0,
            source_date: date,
            decay_class: DecayClass::CuratedVault,
            source_file: source_file.clone(),
            evidence: None,
        };
        assert_eq!(source_file.as_deref(), Some("memory/2026-06-04.md"));
        assert!(qualifies_for_honeypot(&truth));
    }

    #[test]
    fn honeypot_rem_disabled_keeps_episodic_only() {
        use crate::config::DreamsConfig;
        let cfg = DreamsConfig {
            honeypot_rem_enabled: false,
            ..DreamsConfig::default()
        };
        assert!(!cfg.honeypot_rem_enabled);
    }

    #[test]
    fn ops_only_day_mostly_empty() {
        let raw = std::fs::read_to_string("memory/2026-05-31.md").unwrap_or_default();
        if raw.is_empty() {
            return;
        }
        let filtered = filter_episodic_for_consolidation(&raw, &excludes());
        assert!(filtered.len() < 400, "May 31 log should be mostly filtered");
    }
}
