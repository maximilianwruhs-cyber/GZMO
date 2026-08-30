//! Distill `data/sessions/*.json` into `SessionDistill` vault facts and rich episodic.

use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::config::SessionDistillConfig;
use crate::gateway::{LlmGateway, LlmResponse};
use crate::memory::episodic::FileEpisodicStore;
use crate::memory::kg_extract::{chunk_text_for_llm, merge_pipeline_chunks, KgPromoter};
use crate::memory::scratch::DistillSource;
use crate::memory::vault::SqliteVault;
use crate::session::{Session, SessionManager};
use crate::synapse::{resolve_event_source, EventSource, EventType, SynapseBus, SynapseEvent};
use crate::tools::ToolRegistry;
use crate::types::{DecayClass, EpisodicEntry, EpisodicSource, ExtractedTruth, Message, Role};

const SESSION_DISTILL_SYSTEM: &str = concat!(
    "You are a session memory distiller. Extract durable facts from a chat transcript.\n\n",
    "Rules:\n",
    "1. Use internal_analysis to reason first.\n",
    "2. Extract PEOPLE, SYSTEMS, PROJECTS, TOOLS, DECISIONS — not generic labels.\n",
    "3. Each entity needs 1+ observations grounded in the transcript.\n",
    "4. Relations: USES, MANAGES, DEPENDS_ON, RELATED_TO, AUTHORED_BY.\n",
    "5. Ignore boilerplate greetings unless they encode a real decision.\n",
    "6. Empty arrays if the transcript is trivial."
);

/// Synthetic source path so distilled session facts qualify for honeypot recall.
/// Avoids the `chat_history`/`chat_session` exclusion patterns (these are *distilled,
/// verified* facts, not raw chat) so spark `SessionDistill` anchors are reachable.
pub fn session_distill_source(session_id: &str) -> String {
    let safe: String = session_id
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("sessions/{safe}.md")
}

/// Stable key for deduplicating archive-worker vs nightly-cron distill of the same transcript.
pub fn distill_transcript_dedup_key(session_id: &str, transcript: &str) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    session_id.hash(&mut h);
    transcript.trim().hash(&mut h);
    format!("{:016x}", h.finish())
}

pub struct SessionDistillEngine {
    promoter: KgPromoter,
    summary_gateway: Option<Arc<dyn LlmGateway>>,
    vault: SqliteVault,
    episodic: FileEpisodicStore,
    sessions: SessionManager,
    config: SessionDistillConfig,
    /// Optional Synapse bus for observability.
    synapse: Option<Arc<SynapseBus>>,
}

impl SessionDistillEngine {
    pub fn new(
        vault: SqliteVault,
        episodic: FileEpisodicStore,
        sessions_dir: impl AsRef<Path>,
        extract_gateway: Arc<dyn LlmGateway>,
        verify_gateway: Arc<dyn LlmGateway>,
        summary_gateway: Option<Arc<dyn LlmGateway>>,
        tools: Arc<ToolRegistry>,
        config: SessionDistillConfig,
        synapse: Option<Arc<SynapseBus>>,
    ) -> Self {
        Self {
            promoter: KgPromoter::new(extract_gateway, tools, config.kg_gate())
                .with_verify_gateway(verify_gateway),
            summary_gateway,
            vault,
            episodic,
            sessions: SessionManager::new(sessions_dir),
            config,
            synapse,
        }
    }

    /// Distill every session JSON in the sessions directory.
    pub async fn distill_all(&self) -> Result<Vec<SessionDistillReport>> {
        if !self.config.enabled {
            anyhow::bail!("SessionDistill disabled in [session_distill] config");
        }
        self.sessions.ensure_dir().await?;
        let ids: Vec<String> = self
            .sessions
            .list()
            .await?
            .into_iter()
            .map(|m| m.id)
            .collect();
        let mut reports = Vec::with_capacity(ids.len());
        for id in ids {
            reports.push(self.distill_one(&id).await?);
        }
        Ok(reports)
    }

    /// Distill a single session by id.
    pub async fn distill_one(&self, session_id: &str) -> Result<SessionDistillReport> {
        if !self.config.enabled {
            anyhow::bail!("SessionDistill disabled in [session_distill] config");
        }
        let session = self.sessions.load(session_id).await?;
        let transcript = build_transcript(&session, self.config.max_transcript_chars);
        self.distill_transcript(session_id, &transcript, DistillSource::MainArchive)
            .await
    }

    /// Distill an archived transcript chunk (fire-and-forget worker path).
    pub async fn distill_transcript(
        &self,
        session_id: &str,
        transcript: &str,
        source: DistillSource,
    ) -> Result<SessionDistillReport> {
        if !self.config.enabled {
            return Ok(SessionDistillReport::skipped(
                session_id,
                "SessionDistill disabled",
            ));
        }
        if transcript.trim().len() < 80 {
            return Ok(SessionDistillReport::skipped(
                session_id,
                "Transcript too short after filtering meta/tool noise",
            ));
        }

        let dedup_key = distill_transcript_dedup_key(session_id, transcript);
        if self.vault.distill_dedup_seen(&dedup_key)? {
            return Ok(SessionDistillReport::skipped(
                session_id,
                "Duplicate transcript (dedup)",
            ));
        }

        let date = Utc::now().date_naive();
        info!(
            session_id,
            bytes = transcript.len(),
            ?source,
            "Starting transcript distill"
        );

        let chunks = chunk_text_for_llm(transcript, self.config.chunk_chars);
        let mut chunk_results = Vec::with_capacity(chunks.len());
        for (i, chunk) in chunks.iter().enumerate() {
            let label = format!(
                "Extract entities and relationships from this chat session ({session_id}, part {}/{})",
                i + 1,
                chunks.len()
            );
            match self
                .promoter
                .run_pipeline(chunk, "session_distill", SESSION_DISTILL_SYSTEM, &label)
                .await
            {
                Ok(p) => chunk_results.push(p),
                Err(e) => {
                    warn!(session_id, "Session distill pipeline failed: {e}");
                    return Ok(SessionDistillReport::failed(
                        session_id,
                        &format!("Pipeline failed: {e}"),
                    ));
                }
            }
        }

        let pipeline = merge_pipeline_chunks(chunk_results);
        if pipeline.verified_entities.is_empty() && pipeline.verified_relations.is_empty() {
            return Ok(SessionDistillReport::skipped(
                session_id,
                "No verified entities or relations",
            ));
        }

        let session_source = session_distill_source(session_id);
        let provenance = format!("[session_distill] id={session_id}");
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
                warn!(session_id, "Session distill KG promotion failed: {e}");
                (0, 0)
            }
        };

        let truths: Vec<ExtractedTruth> = pipeline
            .verified_entities
            .iter()
            .flat_map(|ve| {
                let obs_count = ve.entity.observations.len();
                let session_source = session_source.clone();
                ve.entity
                    .observations
                    .iter()
                    .map(move |obs| ExtractedTruth {
                        id: Uuid::new_v4(),
                        content: format!("[{}:{}] {}", ve.entity.entity_type, ve.entity.name, obs),
                        confidence: ve.confidence as f32,
                        mmr_score: 0.0,
                        source_date: date,
                        decay_class: DecayClass::SessionDistill,
                        source_file: Some(session_source.clone()),
                        evidence: crate::memory::evidence_localize::localize_observation_evidence(
                            transcript,
                            obs,
                            &ve.evidence,
                            obs_count,
                        ),
                    })
            })
            .collect();

        if let Err(e) = self
            .vault
            .promote_truths_with_origin(&truths, "session_distill")
            .await
        {
            warn!("Session distill vault promotion failed: {e}");
        } else if let Err(e) = self.vault.record_distill_dedup(
            &dedup_key,
            session_id,
            &format!("{source:?}"),
            truths.len(),
        ) {
            warn!(session_id, error = %e, "Failed to record distill dedup key");
        }

        let mut summary = format_summary(session_id, &pipeline.verified_entities, truths.len());
        if self.config.librarian_summary && matches!(source, DistillSource::MainArchive) {
            if let Some(narrative) = self
                .librarian_episodic_summary(session_id, transcript)
                .await
            {
                summary = format!("{narrative}\n\n{summary}");
            }
        }
        if matches!(source, DistillSource::MainArchive) {
            self.log_episodic(session_id, &summary).await?;
        }

        // DistillComplete: append to Synapse bus
        if let Some(ref bus) = self.synapse {
            let data = serde_json::json!({
                "session_id": session_id,
                "entities_promoted": pipeline.verified_entities.len(),
                "relations_promoted": pipeline.verified_relations.len(),
                "kg_entities_written": kg_entities,
                "kg_relations_written": kg_relations,
                "vault_truths": truths.len(),
            });
            bus.append(&SynapseEvent::with_data(
                EventType::DistillComplete,
                resolve_event_source(EventSource::GzmoCli),
                data,
            ));
        }

        Ok(SessionDistillReport {
            session_id: session_id.to_string(),
            entities_promoted: pipeline.verified_entities.len(),
            relations_promoted: pipeline.verified_relations.len(),
            kg_entities_written: kg_entities,
            kg_relations_written: kg_relations,
            vault_truths: truths.len(),
            summary,
            skipped: false,
        })
    }

    async fn librarian_episodic_summary(
        &self,
        session_id: &str,
        transcript: &str,
    ) -> Option<String> {
        let gw = self.summary_gateway.as_ref()?;
        let excerpt: String = transcript.chars().take(6_000).collect();
        let messages = vec![
            Message {
                role: Role::System,
                content: "You write concise session memory for a personal knowledge base. \
                    Output 2-4 factual sentences about decisions, people, systems, and projects. \
                    No greetings or meta commentary."
                    .to_string(),
                is_meta: false,
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: Role::User,
                content: format!("Summarize chat session `{session_id}`:\n\n{excerpt}"),
                is_meta: false,
                tool_calls: None,
                tool_call_id: None,
            },
        ];
        match gw.complete(&messages, &[]).await {
            Ok(LlmResponse::Text(text)) if text.trim().len() > 40 => Some(text.trim().to_string()),
            Ok(LlmResponse::Text(_)) | Ok(LlmResponse::ToolCalls(_)) => None,
            Err(e) => {
                warn!(session_id, "Librarian episodic summary failed: {e}");
                None
            }
        }
    }

    async fn log_episodic(&self, session_id: &str, summary: &str) -> Result<()> {
        let entry = EpisodicEntry {
            timestamp: Utc::now(),
            source: EpisodicSource::SessionDistill {
                session_id: session_id.to_string(),
            },
            content: summary.to_string(),
            is_silent: true,
        };
        self.episodic.append(&entry).await
    }
}

fn build_transcript(session: &Session, max_chars: usize) -> String {
    let mut out = format!(
        "Chat session {} — created {}\n\n",
        session.id,
        session.created_at.format("%Y-%m-%d %H:%M UTC")
    );
    for msg in &session.messages {
        if msg.is_meta {
            continue;
        }
        if matches!(msg.role, Role::System | Role::Tool) {
            continue;
        }
        let body = msg.content.trim();
        if body.is_empty() {
            continue;
        }
        let role = match msg.role {
            Role::User => "USER",
            Role::Assistant => "ASSISTANT",
            Role::System => "SYSTEM",
            Role::Tool => "TOOL",
        };
        out.push_str(&format!("{role}: {body}\n\n"));
        if out.len() >= max_chars {
            out.truncate(max_chars);
            out.push_str("\n[TRUNCATED]\n");
            break;
        }
    }
    out
}

fn format_summary(
    session_id: &str,
    entities: &[crate::memory::kg_extract::VerifiedEntity],
    vault_truths: usize,
) -> String {
    let mut s = format!(
        "Session `{session_id}` distilled: {vault_truths} vault truths from {} verified entities.\n",
        entities.len()
    );
    for ve in entities.iter().take(8) {
        s.push_str(&format!(
            "- **{}** ({}) — {:.2}\n",
            ve.entity.name, ve.entity.entity_type, ve.confidence
        ));
        for obs in ve.entity.observations.iter().take(2) {
            s.push_str(&format!("  - {obs}\n"));
        }
    }
    s
}

pub struct SessionDistillReport {
    pub session_id: String,
    pub entities_promoted: usize,
    pub relations_promoted: usize,
    pub kg_entities_written: usize,
    pub kg_relations_written: usize,
    pub vault_truths: usize,
    pub summary: String,
    pub skipped: bool,
}

impl SessionDistillReport {
    fn skipped(id: &str, reason: &str) -> Self {
        Self {
            session_id: id.to_string(),
            entities_promoted: 0,
            relations_promoted: 0,
            kg_entities_written: 0,
            kg_relations_written: 0,
            vault_truths: 0,
            summary: format!("Session `{id}` skipped: {reason}"),
            skipped: true,
        }
    }

    fn failed(id: &str, reason: &str) -> Self {
        Self {
            session_id: id.to_string(),
            entities_promoted: 0,
            relations_promoted: 0,
            kg_entities_written: 0,
            kg_relations_written: 0,
            vault_truths: 0,
            summary: format!("Session `{id}` failed: {reason}"),
            skipped: true,
        }
    }
}

/// Background worker: BRPOP distill queue and run `distill_transcript` (fire-and-forget).
pub async fn run_distill_worker(
    scratch: std::sync::Arc<crate::memory::scratch::ScratchService>,
    engine: std::sync::Arc<SessionDistillEngine>,
) {
    info!("Distill worker started");
    loop {
        match scratch.pop_distill_job(5.0).await {
            Ok(Some(job)) => {
                info!(
                    session_id = %job.session_id,
                    bytes = job.transcript.len(),
                    "Distill worker processing job"
                );
                match engine
                    .distill_transcript(&job.session_id, &job.transcript, job.source)
                    .await
                {
                    Ok(report) if !report.skipped => {
                        info!(summary = %report.summary, "Distill worker complete");
                    }
                    Ok(report) => {
                        info!(summary = %report.summary, "Distill worker skipped");
                    }
                    Err(e) => {
                        warn!(error = %e, "Distill worker job failed");
                    }
                }
            }
            Ok(None) => {}
            Err(e) => {
                warn!(error = %e, "Distill worker pop failed");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Message, Role};
    use chrono::TimeZone;

    #[test]
    fn test_build_transcript_empty() {
        let session = Session {
            id: "test-empty".to_string(),
            name: Some("Test".to_string()),
            created_at: Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap(),
            last_active_at: Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap(),
            messages: vec![],
        };
        let t = build_transcript(&session, 1000);
        assert!(t.contains("Chat session test-empty — created 2023-01-01 12:00 UTC"));
        assert!(!t.contains("USER:"));
    }

    #[test]
    fn test_build_transcript_bounds() {
        let session = Session {
            id: "test-bounds".to_string(),
            name: Some("Test".to_string()),
            created_at: Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap(),
            last_active_at: Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap(),
            messages: vec![Message {
                role: Role::User,
                content: "Hello world this is a very long message that should be truncated"
                    .to_string(),
                is_meta: false,
                tool_calls: None,
                tool_call_id: None,
            }],
        };
        let t = build_transcript(&session, 80);
        assert!(t.contains("[TRUNCATED]"));
        assert!(t.len() <= 150);
    }
}
