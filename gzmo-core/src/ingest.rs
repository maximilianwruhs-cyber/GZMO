//! Gated document ingest — extract, verify, dedupe, promote (replaces headless watcher).

use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use tracing::{info, warn};

use crate::config::{IngestConfig, WikiConfig};
use crate::gateway::LlmGateway;
use crate::ingest_prep::{
    build_provenance, classify_document, extract_system_prompt, infer_agent_name, prepare_body,
    split_frontmatter, DocClass, Frontmatter,
};
use crate::memory::episodic::FileEpisodicStore;
use crate::memory::kg_extract::{
    chunk_text_for_llm, KgPromoter, PipelineResult, VerifiedEntity, VerifiedRelation,
};
use crate::memory::kg_promotion::normalize_entity_key;
use crate::memory::vault::SqliteVault;
use crate::synapse::{resolve_event_source, EventSource, EventType, SynapseBus, SynapseEvent};
use crate::tools::ToolRegistry;
use crate::types::{DecayClass, EpisodicEntry, EpisodicSource, ExtractedTruth};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct IngestEngine {
    promoter: KgPromoter,
    vault: SqliteVault,
    episodic: FileEpisodicStore,
    config: IngestConfig,
    synapse: Option<Arc<SynapseBus>>,
    /// When set and `emit_on_ingest`, a `wiki/sources` page is written on promotion.
    wiki: Option<WikiConfig>,
}

struct PreparedDocument {
    body: String,
    frontmatter: Frontmatter,
    doc_class: DocClass,
    file_name: String,
}

impl IngestEngine {
    pub fn new(
        vault: SqliteVault,
        episodic: FileEpisodicStore,
        gateway: Arc<dyn LlmGateway>,
        tools: Arc<ToolRegistry>,
        config: IngestConfig,
        synapse: Option<Arc<SynapseBus>>,
    ) -> Self {
        Self::new_with_verify(
            vault,
            episodic,
            gateway.clone(),
            gateway,
            tools,
            config,
            synapse,
        )
    }

    pub fn new_with_verify(
        vault: SqliteVault,
        episodic: FileEpisodicStore,
        extract_gateway: Arc<dyn LlmGateway>,
        verify_gateway: Arc<dyn LlmGateway>,
        tools: Arc<ToolRegistry>,
        config: IngestConfig,
        synapse: Option<Arc<SynapseBus>>,
    ) -> Self {
        Self {
            promoter: KgPromoter::new(extract_gateway, tools, config.kg_gate())
                .with_verify_gateway(verify_gateway),
            vault,
            episodic,
            config,
            synapse,
            wiki: None,
        }
    }

    /// Enable wiki page emission on successful promotion (`[wiki].emit_on_ingest`).
    pub fn with_wiki(mut self, wiki: WikiConfig) -> Self {
        self.wiki = Some(wiki);
        self
    }

    /// True if `path` lives under the configured wiki directory — such pages are
    /// agent-owned synthesis and must never be re-ingested as raw sources.
    fn is_wiki_source(&self, path: &Path) -> bool {
        let wiki_dir = self
            .wiki
            .as_ref()
            .map(|w| w.directory.clone())
            .unwrap_or_else(|| "wiki".to_string());
        let wiki_dir = wiki_dir.trim_end_matches('/');
        if wiki_dir.is_empty() {
            return false;
        }
        path.components()
            .any(|c| c.as_os_str() == std::ffi::OsStr::new(wiki_dir))
    }

    pub async fn ingest_file(&self, path: &Path) -> Result<IngestReport> {
        if !self.config.enabled {
            anyhow::bail!("IngestEngine disabled in [ingest] config");
        }
        if self.is_wiki_source(path) {
            info!(file = %path.display(), "Skipping ingest — path is under the wiki/ layer (agent-owned synthesis)");
            return Ok(IngestReport::skipped_wiki(path));
        }
        let prepared = self.load_document(path).await?;
        let content_hash = ingest_content_hash(&prepared.body);
        if self.vault.ingest_dedup_seen(&content_hash)? {
            info!(
                file = %prepared.file_name,
                hash = %content_hash,
                "Skipping ingest — identical content already processed"
            );
            return Ok(IngestReport::skipped_duplicate(path, &prepared.file_name));
        }
        let (pipeline, chunk_count) = self.run_pipeline(&prepared).await?;
        self.finish_ingest(path, &prepared, pipeline, chunk_count, false, &content_hash)
            .await
    }

    pub async fn ingest_file_dry_run(&self, path: &Path) -> Result<IngestReport> {
        if self.is_wiki_source(path) {
            return Ok(IngestReport::skipped_wiki(path));
        }
        let prepared = self.load_document(path).await?;
        let content_hash = ingest_content_hash(&prepared.body);
        let (pipeline, chunk_count) = self.run_pipeline(&prepared).await?;
        self.finish_ingest(path, &prepared, pipeline, chunk_count, true, &content_hash)
            .await
    }

    async fn load_document(&self, path: &Path) -> Result<PreparedDocument> {
        let raw = tokio::fs::read_to_string(path).await?;
        let file_name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "document".to_string());
        if has_synthetic_frontmatter(&raw) {
            anyhow::bail!(
                "Refusing to ingest agent-synthesized wiki page (gzmo_synthetic: true): {file_name}"
            );
        }
        let (frontmatter, body) = split_frontmatter(&raw);
        let doc_class = classify_document(&file_name, &frontmatter, &body);
        let body = prepare_body(doc_class, &body);
        let body = truncate_source(&body, self.config.max_source_chars);
        Ok(PreparedDocument {
            body,
            frontmatter,
            doc_class,
            file_name,
        })
    }

    async fn run_pipeline(&self, prepared: &PreparedDocument) -> Result<(PipelineResult, usize)> {
        let chunks = chunk_text_for_llm(&prepared.body, self.config.chunk_chars);
        let extract_system = extract_system_prompt(prepared.doc_class);
        let skip_relation_verify =
            prepared.doc_class == DocClass::AgentSpec || prepared.doc_class == DocClass::Reference;

        info!(
            file = %prepared.file_name,
            doc_class = ?prepared.doc_class,
            bytes = prepared.body.len(),
            chunks = chunks.len(),
            "Starting ingest pipeline (verify-on-merged)"
        );

        let mut pipeline = self
            .promoter
            .run_merged_pipeline(
                &prepared.body,
                &chunks,
                "ingest_extraction",
                extract_system,
                &prepared.file_name,
                skip_relation_verify,
            )
            .await?;

        ensure_primary_agent_entity(
            &mut pipeline.verified_entities,
            prepared.doc_class,
            &prepared.file_name,
            &prepared.frontmatter,
            self.config.min_confidence,
        );
        relink_relations_after_entities(
            &mut pipeline,
            prepared.doc_class,
            self.config.min_confidence,
        );
        append_inferred_relations(
            &mut pipeline.verified_entities,
            &mut pipeline.verified_relations,
            prepared.doc_class,
            &prepared.frontmatter,
            &prepared.file_name,
            self.config.min_confidence,
        );

        Ok((pipeline, chunks.len()))
    }

    async fn finish_ingest(
        &self,
        path: &Path,
        prepared: &PreparedDocument,
        pipeline: PipelineResult,
        chunk_count: usize,
        dry_run: bool,
        content_hash: &str,
    ) -> Result<IngestReport> {
        let file_name = &prepared.file_name;
        let date = Utc::now().date_naive();

        info!(
            chunks = chunk_count,
            raw_entities = pipeline.raw_entities,
            raw_relations = pipeline.raw_relations,
            kept_entities = pipeline.verified_entities.len(),
            kept_relations = pipeline.verified_relations.len(),
            dropped_entities = pipeline.stats.entities_dropped,
            dropped_relations = pipeline.stats.relations_dropped,
            preprocess_dropped = pipeline.prep.dropped_preprocess,
            dry_run,
            "Ingest pipeline complete"
        );

        if pipeline.verified_entities.is_empty() && pipeline.verified_relations.is_empty() {
            return Ok(IngestReport::failed(
                path,
                "No verified entities or relations to promote",
            ));
        }

        let provenance = build_provenance(file_name, &prepared.frontmatter);
        let (kg_entities, kg_relations, vault_truths) = if dry_run {
            (0, 0, 0)
        } else {
            let (kg_entities, kg_relations) = match self
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
                    warn!("Ingest KG promotion failed: {e}");
                    return Ok(IngestReport::failed(
                        path,
                        &format!("KG promotion failed: {e}"),
                    ));
                }
            };

            let truths = collect_truths(
                &pipeline.verified_entities,
                &pipeline.verified_relations,
                date,
                file_name,
                &prepared.body,
            );
            if let Err(e) = self.vault.promote_truths(&truths).await {
                warn!("Vault promotion failed (non-fatal): {e}");
            }

            (kg_entities, kg_relations, truths.len())
        };

        let summary = format_summary(
            file_name,
            chunk_count,
            &pipeline.verified_entities,
            &pipeline.verified_relations,
            &pipeline.stats,
            pipeline.prep.dropped_preprocess,
            dry_run,
        );

        if !dry_run {
            if let Err(e) = self
                .vault
                .record_ingest_dedup(content_hash, &path.display().to_string())
            {
                warn!(error = %e, "Failed to record ingest dedup key");
            }

            self.log_episodic(file_name, &summary).await?;

            if let Some(ref bus) = self.synapse {
                let data = serde_json::json!({
                    "file": file_name,
                    "entities_extracted": pipeline.raw_entities,
                    "relations_extracted": pipeline.raw_relations,
                    "entities_promoted": pipeline.verified_entities.len(),
                    "relations_promoted": pipeline.verified_relations.len(),
                    "kg_entities_written": kg_entities,
                    "kg_relations_written": kg_relations,
                });
                bus.append(&SynapseEvent::with_data(
                    EventType::IngestComplete,
                    resolve_event_source(EventSource::GzmoDaemon),
                    data,
                ));
                let data_dir = bus.path.parent().unwrap_or(Path::new("data"));
                let names: Vec<String> = pipeline
                    .verified_entities
                    .iter()
                    .map(|e| e.entity.name.clone())
                    .collect();
                let _ = crate::pi_recent_discoveries::record_ingest(
                    data_dir,
                    file_name,
                    &names,
                    pipeline.verified_relations.len(),
                );
            }

            // WikiEngine emit hook — derive a wiki/sources page from the
            // already-verified facts. Non-fatal; pages are emit-only (never
            // re-ingested — see is_wiki_source / has_synthetic_frontmatter).
            if let Some(ref wiki_cfg) = self.wiki {
                if wiki_cfg.enabled && wiki_cfg.emit_on_ingest {
                    let engine = crate::wiki::WikiEngine::new(wiki_cfg.clone());
                    if let Err(e) = engine
                        .emit_source_page(
                            file_name,
                            &pipeline.verified_entities,
                            &pipeline.verified_relations,
                            date,
                        )
                        .await
                    {
                        warn!(error = %e, "Wiki page emission failed (non-fatal)");
                    }
                }
            }
        }

        Ok(report_from_pipeline(
            path,
            pipeline,
            kg_entities,
            kg_relations,
            vault_truths,
            summary,
        ))
    }

    async fn log_episodic(&self, file_name: &str, summary: &str) -> Result<()> {
        let entry = EpisodicEntry {
            timestamp: Utc::now(),
            source: EpisodicSource::InternalMonologue,
            content: format!("[ingest:{file_name}] {summary}"),
            is_silent: true,
        };
        self.episodic.append(&entry).await
    }
}

fn collect_truths(
    entities: &[VerifiedEntity],
    relations: &[VerifiedRelation],
    date: chrono::NaiveDate,
    source_file: &str,
    body: &str,
) -> Vec<ExtractedTruth> {
    let mut truths = truths_from_pipeline(entities, date, source_file, body);
    truths.extend(truths_from_relations(relations, date, source_file, body));
    truths
}

fn truths_from_pipeline(
    entities: &[VerifiedEntity],
    date: chrono::NaiveDate,
    source_file: &str,
    body: &str,
) -> Vec<ExtractedTruth> {
    entities
        .iter()
        .flat_map(|ve| {
            let obs_count = ve.entity.observations.len();
            ve.entity.observations.iter().map(move |obs| ExtractedTruth {
                id: Uuid::new_v4(),
                content: format!(
                    "[{}:{}] {}",
                    ve.entity.entity_type, ve.entity.name, obs
                ),
                confidence: ve.confidence as f32,
                mmr_score: 0.0,
                source_date: date,
                decay_class: DecayClass::CuratedVault,
                source_file: Some(source_file.to_string()),
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

fn truths_from_relations(
    relations: &[VerifiedRelation],
    date: chrono::NaiveDate,
    source_file: &str,
    body: &str,
) -> Vec<ExtractedTruth> {
    relations
        .iter()
        .map(|vr| {
            let evidence_span = if !vr.evidence.is_empty() {
                Some(crate::memory::evidence_localize::localize_evidence(body, &vr.evidence))
            } else {
                None
            };
            ExtractedTruth {
                id: Uuid::new_v4(),
                content: format!(
                    "[RELATION:{}] {} → {}",
                    vr.relation.relation_type, vr.relation.from, vr.relation.to
                ),
                confidence: vr.confidence as f32,
                mmr_score: 0.0,
                source_date: date,
                decay_class: DecayClass::CuratedVault,
                source_file: Some(source_file.to_string()),
                evidence: evidence_span,
            }
        })
        .collect()
}


fn entity_name_in_set(name: &str, kept: &std::collections::HashSet<String>) -> bool {
    let key = normalize_entity_key(name);
    if key.is_empty() {
        return false;
    }
    kept.iter().any(|k| {
        let kk = normalize_entity_key(k);
        kk == key || kk.contains(&key) || key.contains(&kk)
    })
}

fn relink_relations_after_entities(
    pipeline: &mut PipelineResult,
    doc_class: DocClass,
    min_confidence: f64,
) {
    if doc_class != DocClass::AgentSpec {
        return;
    }
    let kept_names: std::collections::HashSet<String> = pipeline
        .verified_entities
        .iter()
        .map(|ve| ve.entity.name.clone())
        .collect();
    let agent_conf = min_confidence.max(0.8);
    let kept: Vec<VerifiedRelation> = pipeline
        .candidate_relations
        .iter()
        .filter(|r| entity_name_in_set(&r.from, &kept_names) && entity_name_in_set(&r.to, &kept_names))
        .map(|r| VerifiedRelation {
            relation: r.clone(),
            confidence: agent_conf,
            evidence: String::new(),
        })
        .collect();
    if !kept.is_empty() {
        pipeline.verified_relations = kept;
    }
}

fn ensure_primary_agent_entity(
    entities: &mut Vec<VerifiedEntity>,
    doc_class: DocClass,
    file_name: &str,
    frontmatter: &Frontmatter,
    min_confidence: f64,
) {
    if doc_class != DocClass::AgentSpec {
        return;
    }
    let Some(agent_name) = infer_agent_name(file_name, frontmatter) else {
        return;
    };
    let agent_key = agent_name.to_lowercase().replace(['-', '_'], "");
    let already = entities.iter().any(|ve| {
        ve.entity
            .name
            .to_lowercase()
            .replace(['-', '_'], "")
            .contains(&agent_key)
            || agent_key.contains(&ve.entity.name.to_lowercase().replace(['-', '_'], ""))
    });
    if already {
        return;
    }
    entities.insert(
        0,
        VerifiedEntity {
            entity: crate::memory::kg_extract::KgEntity {
                name: agent_name.clone(),
                entity_type: "AGENT".into(),
                observations: vec![format!(
                    "Primary agent defined in {file_name} (inferred from path/filename)"
                )],
            },
            confidence: min_confidence.max(0.85),
            evidence: String::new(),
        },
    );
}

fn append_inferred_relations(
    entities: &[VerifiedEntity],
    relations: &mut Vec<VerifiedRelation>,
    doc_class: DocClass,
    frontmatter: &Frontmatter,
    file_name: &str,
    min_confidence: f64,
) {
    if doc_class != DocClass::AgentSpec {
        return;
    }
    let Some(notebook) = frontmatter.notebook.as_ref() else {
        return;
    };
    let agent = entities.first().map(|e| e.entity.name.clone());
    let Some(agent) = agent else {
        return;
    };
    let already = relations.iter().any(|r| {
        r.relation.from == agent
            && r.relation.to == *notebook
            && r.relation.relation_type == "PART_OF"
    });
    if !already {
        relations.push(VerifiedRelation {
            relation: crate::memory::kg_extract::KgRelation {
                from: agent,
                to: notebook.clone(),
                relation_type: "PART_OF".into(),
            },
            confidence: min_confidence.max(0.8),
            evidence: format!("inferred from notebook frontmatter on {file_name}"),
        });
    }
}

fn truncate_source(raw: &str, max_chars: usize) -> String {
    if raw.len() <= max_chars {
        return raw.to_string();
    }
    warn!(
        original = raw.len(),
        kept = max_chars,
        "Truncating ingest source before chunking"
    );
    raw[..max_chars].to_string()
}

fn format_summary(
    file_name: &str,
    chunk_count: usize,
    entities: &[VerifiedEntity],
    relations: &[VerifiedRelation],
    stats: &crate::memory::kg_extract::VerifyStats,
    preprocess_dropped: usize,
    dry_run: bool,
) -> String {
    let prefix = if dry_run { "dry-run " } else { "" };
    let chunks_note = if chunk_count > 1 {
        format!(" via {chunk_count} chunks")
    } else {
        String::new()
    };
    format!(
        "{prefix}ingested `{file_name}`{chunks_note}: promoted {} entities, {} relations (verify dropped {}E {}R; preprocess dropped {preprocess_dropped})",
        entities.len(),
        relations.len(),
        stats.entities_dropped,
        stats.relations_dropped
    )
}

fn report_from_pipeline(
    path: &Path,
    pipeline: PipelineResult,
    kg_entities: usize,
    kg_relations: usize,
    vault_truths: usize,
    summary: String,
) -> IngestReport {
    IngestReport {
        file_path: path.display().to_string(),
        entities_extracted: pipeline.raw_entities,
        relations_extracted: pipeline.raw_relations,
        entities_promoted: pipeline.verified_entities.len(),
        relations_promoted: pipeline.verified_relations.len(),
        kg_entities_written: kg_entities,
        kg_relations_written: kg_relations,
        vault_truths,
        summary,
        verified_entities: pipeline
            .verified_entities
            .iter()
            .map(|ve| ve.entity.name.clone())
            .collect(),
        verified_relations: pipeline
            .verified_relations
            .iter()
            .map(|vr| {
                (
                    vr.relation.from.clone(),
                    vr.relation.to.clone(),
                    vr.relation.relation_type.clone(),
                )
            })
            .collect(),
        verified_facts: pipeline
            .verified_entities
            .iter()
            .flat_map(|ve| ve.entity.observations.clone())
            .collect(),
    }
}

pub struct IngestReport {
    pub file_path: String,
    pub entities_extracted: usize,
    pub relations_extracted: usize,
    pub entities_promoted: usize,
    pub relations_promoted: usize,
    pub kg_entities_written: usize,
    pub kg_relations_written: usize,
    pub vault_truths: usize,
    pub summary: String,
    pub verified_entities: Vec<String>,
    pub verified_relations: Vec<(String, String, String)>,
    pub verified_facts: Vec<String>,
}

impl IngestReport {
    fn failed(path: &Path, reason: &str) -> Self {
        Self {
            file_path: path.display().to_string(),
            entities_extracted: 0,
            relations_extracted: 0,
            entities_promoted: 0,
            relations_promoted: 0,
            kg_entities_written: 0,
            kg_relations_written: 0,
            vault_truths: 0,
            summary: reason.to_string(),
            verified_entities: Vec::new(),
            verified_relations: Vec::new(),
            verified_facts: Vec::new(),
        }
    }

    fn skipped_wiki(path: &Path) -> Self {
        Self {
            file_path: path.display().to_string(),
            entities_extracted: 0,
            relations_extracted: 0,
            entities_promoted: 0,
            relations_promoted: 0,
            kg_entities_written: 0,
            kg_relations_written: 0,
            vault_truths: 0,
            summary: "Skipped — path is under the wiki/ layer (agent-owned synthesis, emit-only)"
                .to_string(),
            verified_entities: Vec::new(),
            verified_relations: Vec::new(),
            verified_facts: Vec::new(),
        }
    }

    fn skipped_duplicate(path: &Path, file_name: &str) -> Self {
        Self {
            file_path: path.display().to_string(),
            entities_extracted: 0,
            relations_extracted: 0,
            entities_promoted: 0,
            relations_promoted: 0,
            kg_entities_written: 0,
            kg_relations_written: 0,
            vault_truths: 0,
            summary: format!(
                "Skipped duplicate ingest for {file_name} — identical prepared content already processed"
            ),
            verified_entities: Vec::new(),
            verified_relations: Vec::new(),
            verified_facts: Vec::new(),
        }
    }
}

/// True if the raw document opens with a `---` frontmatter block declaring
/// `gzmo_synthetic: true`. Such pages are emitted by `WikiEngine` and must not
/// be re-ingested (would create circular, derived facts).
fn has_synthetic_frontmatter(raw: &str) -> bool {
    let trimmed = raw.trim_start_matches('\u{feff}');
    let Some(rest) = trimmed.strip_prefix("---\n") else {
        return false;
    };
    let Some(end) = rest.find("\n---") else {
        return false;
    };
    rest[..end].lines().any(|l| {
        let l = l.trim();
        l == "gzmo_synthetic: true" || l == "gzmo_synthetic:true"
    })
}

/// SHA-256 of the prepared ingest body (post frontmatter strip, class prep, truncation).
pub fn ingest_content_hash(body: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(body.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest_prep::split_frontmatter;

    #[test]
    fn chunk_source_single_small_doc() {
        let s = "hello world";
        let c = chunk_text_for_llm(s, 28_000);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0], s);
    }

    #[test]
    fn frontmatter_not_in_body() {
        let raw = "---\nmigration_id: x\n---\n\n# Hello\n";
        let (_, body) = split_frontmatter(raw);
        assert!(!body.contains("migration_id"));
        assert!(body.contains("# Hello"));
    }

    #[test]
    fn ingest_content_hash_stable() {
        let h1 = ingest_content_hash("same body");
        let h2 = ingest_content_hash("same body");
        let h3 = ingest_content_hash("other body");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert_eq!(h1.len(), 64);
    }
}
