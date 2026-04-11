//! # OpenClaw Dreams
//!
//! The autoDream 3-phase nightly consolidation engine.
//! Mirrors biological sleep to shift ephemeral episodic data
//! into abstract, permanent semantic knowledge.
//!
//! Phase 1 (Light): Deterministic compression — strips noise, zero LLM cost.
//! Phase 2 (REM):   LLM-powered structured extraction via grammar-constrained JSON.
//! Phase 3 (Deep):  Write extracted entities/relations to the MCP Knowledge Graph
//!                  AND the local SqliteVault (belt + suspenders).

use std::sync::Arc;

use anyhow::Result;
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::gateway::LlmGateway;
use crate::tools::{ToolRegistry, ToolResult};
use crate::types::{DecayClass, ExtractedTruth, Message, Role};
use crate::memory::episodic::FileEpisodicStore;
use crate::memory::vault::SqliteVault;

// ---------------------------------------------------------------------------
// Extraction schema types (what the LLM returns)
// ---------------------------------------------------------------------------

/// An entity extracted by the LLM during the REM phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamEntity {
    pub name: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub observations: Vec<String>,
}

/// A relationship extracted by the LLM during the REM phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamRelation {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub relation_type: String,
}

/// The full structured extraction response from the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DreamExtraction {
    /// Free-text scratchpad — lets the 7B model reason before committing values.
    /// This field is discarded after extraction; it exists solely to preserve
    /// the model's chain-of-thought capability under grammar constraints.
    #[allow(dead_code)]
    internal_analysis: String,
    entities: Vec<DreamEntity>,
    relations: Vec<DreamRelation>,
}

// ---------------------------------------------------------------------------
// JSON Schema for grammar-constrained extraction
// ---------------------------------------------------------------------------

/// Build the JSON Schema that LM Studio compiles into a GBNF grammar.
/// Uses the scratchpad technique: `internal_analysis` comes first so the
/// model can reason in free text before the grammar locks down the structs.
fn extraction_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "internal_analysis": {
                "type": "string",
                "description": "Your step-by-step analysis of the daily log. Identify key people, systems, decisions, and relationships before extracting them."
            },
            "entities": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "type": { "type": "string" },
                        "observations": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
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

// ---------------------------------------------------------------------------
// Dream Engine
// ---------------------------------------------------------------------------

/// The 3-phase autoDream consolidation engine.
pub struct DreamEngine {
    episodic: FileEpisodicStore,
    vault: SqliteVault,
    gateway: Arc<dyn LlmGateway>,
    tools: Arc<ToolRegistry>,
}

impl DreamEngine {
    pub fn new(
        episodic: FileEpisodicStore,
        vault: SqliteVault,
        gateway: Arc<dyn LlmGateway>,
        tools: Arc<ToolRegistry>,
    ) -> Self {
        Self {
            episodic,
            vault,
            gateway,
            tools,
        }
    }

    /// Run the full autoDream cycle for today's episodic data.
    pub async fn consolidate(&self, date: NaiveDate) -> Result<DreamReport> {
        info!(date = %date, "Starting autoDream consolidation cycle");

        // Phase 1: Light — deterministic compression
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

        let compressed = self.light_phase(&raw);
        info!(
            original = raw.len(),
            compressed = compressed.len(),
            ratio = format!("{:.1}:1", raw.len() as f64 / compressed.len().max(1) as f64),
            "Light Phase complete"
        );

        // Phase 2: REM — LLM-powered structured extraction
        let extraction = match self.rem_phase(&compressed).await {
            Ok(ext) => {
                info!(
                    entities = ext.entities.len(),
                    relations = ext.relations.len(),
                    "REM Phase complete"
                );
                ext
            }
            Err(e) => {
                warn!("REM Phase failed (JSON parse error): {e}");
                warn!("Skipping consolidation — episodic file preserved intact");
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
                        "# Dream Consolidation — {date}\n\nREM Phase failed: {e}\nEpisodic data preserved.\n"
                    ),
                });
            }
        };

        // Phase 3: Deep — Write to Knowledge Graph + SqliteVault
        let (kg_entities, kg_relations) = self
            .consolidate_to_kg(&extraction.entities, &extraction.relations)
            .await;

        // Also promote to vault as backup
        let truths = self.to_vault_truths(&extraction.entities, date);
        if let Err(e) = self.vault.promote_truths(&truths) {
            warn!("Vault promotion failed (non-fatal): {e}");
        }

        info!(
            kg_entities,
            kg_relations,
            vault_truths = truths.len(),
            "Deep Phase complete"
        );

        // Generate DREAMS.md narrative
        let narrative = self.generate_narrative(&extraction, date, kg_entities, kg_relations);

        Ok(DreamReport {
            date,
            original_bytes: raw.len(),
            compressed_bytes: compressed.len(),
            entities_extracted: extraction.entities.len(),
            relations_extracted: extraction.relations.len(),
            kg_entities_written: kg_entities,
            kg_relations_written: kg_relations,
            truths_promoted: truths.len(),
            narrative,
        })
    }

    // -----------------------------------------------------------------------
    // Phase 1: Light Phase — Deterministic Compression
    // -----------------------------------------------------------------------

    /// Zero LLM cost. Strips verbose tool outputs, HTML, base64 data,
    /// and repetitive log lines to achieve 10:1 to 20:1 compression.
    fn light_phase(&self, raw: &str) -> String {
        let mut output = String::with_capacity(raw.len() / 10);

        for line in raw.lines() {
            let trimmed = line.trim();

            // Skip empty lines and horizontal rules
            if trimmed.is_empty() || trimmed == "---" {
                continue;
            }

            // Skip base64 encoded data
            if trimmed.len() > 200
                && trimmed
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '=')
            {
                output.push_str("[BASE64_DATA_STRIPPED]\n");
                continue;
            }

            // Skip HTML tags (verbose tool outputs)
            if trimmed.starts_with('<') && trimmed.ends_with('>') && trimmed.len() > 100 {
                output.push_str("[HTML_STRIPPED]\n");
                continue;
            }

            // Truncate extremely long lines (API responses)
            if trimmed.len() > 500 {
                output.push_str(&trimmed[..500]);
                output.push_str("... [TRUNCATED]\n");
                continue;
            }

            output.push_str(trimmed);
            output.push('\n');
        }

        output
    }

    // -----------------------------------------------------------------------
    // Phase 2: REM Phase — LLM-Powered Structured Extraction
    // -----------------------------------------------------------------------

    /// Sends the compressed episodic log to the LLM with a grammar-constrained
    /// JSON schema. The model extracts entities, observations, and relationships.
    /// Uses the scratchpad technique: an `internal_analysis` free-text field
    /// precedes the structured data so the 7B model can reason first.
    async fn rem_phase(&self, compressed: &str) -> Result<DreamExtraction> {
        let messages = vec![
            Message {
                role: Role::System,
                content: concat!(
                    "You are a memory consolidation engine. Your task is to extract ",
                    "structured knowledge from a daily activity log.\n\n",
                    "Rules:\n",
                    "1. Use the internal_analysis field to reason step-by-step before extracting.\n",
                    "2. Extract PEOPLE, SYSTEMS, PROJECTS, TOOLS, DECISIONS as entities.\n",
                    "3. Each entity must have concrete observations (facts, not opinions).\n",
                    "4. Extract relationships between entities (e.g. uses, manages, depends_on).\n",
                    "5. Disambiguate pronouns: 'he' -> use the actual name.\n",
                    "6. If the log is trivial or contains no meaningful entities, return empty arrays."
                ).to_string(),
                is_meta: false, tool_calls: None, tool_call_id: None,
            },
            Message {
                role: Role::User,
                content: format!(
                    "Extract entities and relationships from this daily log:\n\n---\n{}\n---",
                    compressed
                ),
                is_meta: false, tool_calls: None, tool_call_id: None,
            },
        ];

        let raw_json = self
            .gateway
            .complete_structured(&messages, "dream_extraction", extraction_schema())
            .await?;

        let extraction: DreamExtraction = serde_json::from_str(&raw_json)
            .map_err(|e| anyhow::anyhow!("Failed to parse LLM extraction: {e}\nRaw: {raw_json}"))?;

        Ok(extraction)
    }

    // -----------------------------------------------------------------------
    // Phase 3: Deep Phase — Knowledge Graph + Vault Consolidation
    // -----------------------------------------------------------------------

    /// Write extracted entities and relations to the MCP Knowledge Graph
    /// via programmatic tool dispatch. Returns (entities_written, relations_written).
    async fn consolidate_to_kg(
        &self,
        entities: &[DreamEntity],
        relations: &[DreamRelation],
    ) -> (usize, usize) {
        let mut entities_written = 0usize;
        let mut relations_written = 0usize;

        // Write entities via mcp__memory__create_entities
        if !entities.is_empty() {
            let entities_payload: Vec<serde_json::Value> = entities
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "name": e.name,
                        "entityType": e.entity_type,
                        "observations": e.observations,
                    })
                })
                .collect();

            let call = crate::gateway::ToolCall {
                id: format!("dream_kg_{}", Uuid::new_v4()),
                function_name: "mcp__memory__create_entities".to_string(),
                arguments: serde_json::json!({ "entities": entities_payload }),
            };

            match self.tools.dispatch(&call).await {
                ToolResult { success: true, .. } => {
                    entities_written = entities.len();
                    info!(count = entities_written, "KG entities written");
                }
                ToolResult { output, .. } => {
                    warn!("KG entity write failed: {output}");
                }
            }
        }

        // Write relations via mcp__memory__create_relations
        if !relations.is_empty() {
            let relations_payload: Vec<serde_json::Value> = relations
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "from": r.from,
                        "to": r.to,
                        "relationType": r.relation_type,
                    })
                })
                .collect();

            let call = crate::gateway::ToolCall {
                id: format!("dream_kg_{}", Uuid::new_v4()),
                function_name: "mcp__memory__create_relations".to_string(),
                arguments: serde_json::json!({ "relations": relations_payload }),
            };

            match self.tools.dispatch(&call).await {
                ToolResult { success: true, .. } => {
                    relations_written = relations.len();
                    info!(count = relations_written, "KG relations written");
                }
                ToolResult { output, .. } => {
                    warn!("KG relation write failed: {output}");
                }
            }
        }

        (entities_written, relations_written)
    }

    /// Convert extracted entities into ExtractedTruth for the SqliteVault backup.
    fn to_vault_truths(&self, entities: &[DreamEntity], date: NaiveDate) -> Vec<ExtractedTruth> {
        entities
            .iter()
            .flat_map(|entity| {
                entity.observations.iter().map(move |obs| ExtractedTruth {
                    id: Uuid::new_v4(),
                    content: format!("[{}:{}] {}", entity.entity_type, entity.name, obs),
                    confidence: 0.7,
                    mmr_score: 0.0,
                    source_date: date,
                    decay_class: DecayClass::CuratedVault,
                })
            })
            .collect()
    }

    /// Generate the DREAMS.md narrative for human auditability.
    fn generate_narrative(
        &self,
        extraction: &DreamExtraction,
        date: NaiveDate,
        kg_entities: usize,
        kg_relations: usize,
    ) -> String {
        let mut narrative = format!("# Dream Consolidation — {}\n\n", date);
        narrative.push_str(&format!(
            "Processed at {}\n\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        if extraction.entities.is_empty() {
            narrative.push_str("No significant patterns detected today.\n");
            return narrative;
        }

        narrative.push_str("## Extracted Entities\n\n");
        for entity in &extraction.entities {
            narrative.push_str(&format!(
                "- **{}** ({})\n",
                entity.name, entity.entity_type
            ));
            for obs in &entity.observations {
                narrative.push_str(&format!("  - {}\n", obs));
            }
        }

        if !extraction.relations.is_empty() {
            narrative.push_str("\n## Relationships\n\n");
            for rel in &extraction.relations {
                narrative.push_str(&format!(
                    "- {} → ({}) → {}\n",
                    rel.from, rel.relation_type, rel.to
                ));
            }
        }

        narrative.push_str(&format!(
            "\n## Consolidation Stats\n\n- KG entities written: {}\n- KG relations written: {}\n",
            kg_entities, kg_relations
        ));

        narrative
    }
}

// ---------------------------------------------------------------------------
// Dream Report
// ---------------------------------------------------------------------------

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
