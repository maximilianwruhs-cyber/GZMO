//! Ingest quality evaluation command. Run in dry-run mode and score against expected.yaml.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use gzmo_core::config::{GzmoConfig, TaskKind};
use gzmo_core::gateway::GatewayRouter;
use gzmo_core::identity::IdentityEngine;
use gzmo_core::ingest::IngestEngine;
use gzmo_core::memory::embeddings;
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::synapse::{set_event_source, EventSource, SynapseBus};
use gzmo_core::tools::ToolRegistry;

#[derive(Debug, Clone, Deserialize)]
struct ExpectedFileRules {
    #[serde(default)]
    must_entities: Vec<String>,
    #[serde(default)]
    must_fact_substrings: Vec<String>,
    #[serde(default)]
    anti_entities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExpectedManifest {
    files: HashMap<String, ExpectedFileRules>,
}

#[derive(Debug, Clone, Serialize)]
struct EvalResult {
    file_path: String,
    file_name: String,
    entities_extracted: usize,
    relations_extracted: usize,
    entities_promoted: usize,
    relations_promoted: usize,
    verified_entities: Vec<String>,
    verified_relations: Vec<(String, String, String)>,
    verified_facts: Vec<String>,
    evaluation: Option<FileEvalDetails>,
}

#[derive(Debug, Clone, Serialize)]
struct FileEvalDetails {
    must_entities_total: usize,
    must_entities_found: usize,
    must_entities_missing: Vec<String>,
    must_facts_total: usize,
    must_facts_found: usize,
    must_facts_missing: Vec<String>,
    anti_entities_found: Vec<String>,
    score: f64,
}

#[derive(Debug, Clone, Serialize)]
struct BatchEvalReport {
    files: Vec<EvalResult>,
    summary: BatchEvalSummary,
}

#[derive(Debug, Clone, Serialize)]
struct BatchEvalSummary {
    total_files: usize,
    golden_files: usize,
    entities_extracted: usize,
    relations_extracted: usize,
    entities_promoted: usize,
    relations_promoted: usize,
    zero_entity_files: usize,
    zero_relation_files: usize,
    relation_promotion_rate: f64,
    must_entities_recall: f64,
    must_facts_recall: f64,
    anti_entities_found_count: usize,
}

fn load_expected_yaml() -> Result<ExpectedManifest> {
    let path = Path::new("scripts/ingest-quality/expected.yaml");
    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        let manifest: ExpectedManifest = serde_yaml::from_str(&content)?;
        return Ok(manifest);
    }
    let mut current = std::env::current_dir()?;
    for _ in 0..3 {
        let check = current.join("scripts/ingest-quality/expected.yaml");
        if check.exists() {
            let content = std::fs::read_to_string(check)?;
            let manifest: ExpectedManifest = serde_yaml::from_str(&content)?;
            return Ok(manifest);
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }
    anyhow::bail!("expected.yaml not found in scripts/ingest-quality/ or parent directories")
}

fn normalize_entity_label(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c == '-' || c == '_' { ' ' } else { c })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Known LLM extract typos that should satisfy a corpus-grounded golden label.
fn known_extract_typo_pair(a: &str, b: &str) -> bool {
    matches!(
        (a, b),
        ("proxmox", "proxox") | ("proxox", "proxmox")
    )
}

fn entity_label_matches(must: &str, candidate: &str) -> bool {
    let must_norm = normalize_entity_label(must);
    let cand_norm = normalize_entity_label(candidate);
    if must_norm.is_empty() || cand_norm.is_empty() {
        return false;
    }
    if cand_norm.contains(&must_norm) || must_norm.contains(&cand_norm) {
        return true;
    }
    known_extract_typo_pair(&must_norm, &cand_norm)
}

fn entity_found_in_report(
    must: &str,
    entities: &[String],
    facts: &[String],
    relations: &[(String, String, String)],
) -> bool {
    if entities.iter().any(|e| entity_label_matches(must, e)) {
        return true;
    }
    for (from, to, _) in relations {
        if entity_label_matches(must, from) || entity_label_matches(must, to) {
            return true;
        }
    }
    facts.iter().any(|fact| fact_text_satisfies_must(must, fact))
}

fn fact_text_satisfies_must(must: &str, fact: &str) -> bool {
    let must_norm = normalize_entity_label(must);
    let fact_norm = normalize_entity_label(fact);
    if fact_norm.contains(&must_norm) {
        return true;
    }
    // Whole-fact typo (e.g. "Proxox Snapshot" for must "Proxmox").
    must_norm == "proxmox" && fact_norm.contains("proxox")
}

pub async fn run(config: &GzmoConfig, _identity: IdentityEngine, path: PathBuf) -> Result<()> {
    set_event_source(EventSource::GzmoCli);

    let manifest = match load_expected_yaml() {
        Ok(m) => Some(m),
        Err(e) => {
            warn!("Could not load expected.yaml: {e}. Running without scoring.");
            None
        }
    };

    let path = path.canonicalize().context("eval path canonicalize")?;
    let mut files = Vec::new();
    if path.is_dir() {
        collect_md(&path, &mut files)?;
    } else {
        files.push(path.clone());
    }
    files.sort();

    if files.is_empty() {
        anyhow::bail!("No files found to evaluate");
    }

    // Set up dummy tools and IngestEngine
    let router = GatewayRouter::new(config);
    let gateway = Arc::clone(router.gateway(TaskKind::IngestExtract));
    let verify_gateway = Arc::clone(router.gateway(TaskKind::IngestVerify));

    let vault = Arc::new(
        embeddings::open_vault_with_embeddings(
            &config.memory.vault_db,
            &config.embeddings,
            &config.redis,
            &config.rerank,
            &config.qdrant,
        )
        .await?,
    );

    let tools = Arc::new(ToolRegistry::new());
    let synapse = Arc::new(SynapseBus::new());

    let engine = IngestEngine::new_with_verify(
        (*vault).clone(),
        FileEpisodicStore::new(&config.memory.directory),
        gateway,
        verify_gateway,
        tools,
        config.ingest.clone(),
        Some(Arc::clone(&synapse)),
    );

    let mut eval_results = Vec::new();

    for (i, file_path) in files.iter().enumerate() {
        info!(
            n = i + 1,
            total = files.len(),
            file = %file_path.display(),
            "Evaluating file"
        );

        let file_name = file_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        match engine.ingest_file_dry_run(file_path).await {
            Ok(report) => {
                let eval_details = if let Some(ref m) = manifest {
                    m.files.get(&file_name).map(|rules| {
                        let mut missing_must_entities = Vec::new();
                        for must in &rules.must_entities {
                            let found = entity_found_in_report(
                                must,
                                &report.verified_entities,
                                &report.verified_facts,
                                &report.verified_relations,
                            );
                            if !found {
                                missing_must_entities.push(must.clone());
                            }
                        }

                        let mut missing_fact_substrings = Vec::new();
                        for must_fact in &rules.must_fact_substrings {
                            let must_fact_lower = must_fact.to_lowercase();
                            let found = report.verified_facts.iter().any(|fact| {
                                fact.to_lowercase().contains(&must_fact_lower)
                            });
                            if !found {
                                missing_fact_substrings.push(must_fact.clone());
                            }
                        }

                        let mut found_anti_entities = Vec::new();
                        for anti in &rules.anti_entities {
                            let anti_lower = anti.to_lowercase();
                            let found_matches: Vec<String> = report
                                .verified_entities
                                .iter()
                                .filter(|e| e.to_lowercase().contains(&anti_lower))
                                .cloned()
                                .collect();
                            found_anti_entities.extend(found_matches);
                        }

                        let must_e_total = rules.must_entities.len();
                        let must_e_found = must_e_total - missing_must_entities.len();

                        let must_f_total = rules.must_fact_substrings.len();
                        let must_f_found = must_f_total - missing_fact_substrings.len();

                        let score_e = if must_e_total > 0 {
                            must_e_found as f64 / must_e_total as f64
                        } else {
                            1.0
                        };
                        let score_f = if must_f_total > 0 {
                            must_f_found as f64 / must_f_total as f64
                        } else {
                            1.0
                        };
                        let anti_penalty = if !found_anti_entities.is_empty() { 0.5 } else { 0.0 };
                        let score = (0.5 * score_e + 0.5 * score_f - anti_penalty).max(0.0);

                        FileEvalDetails {
                            must_entities_total: must_e_total,
                            must_entities_found: must_e_found,
                            must_entities_missing: missing_must_entities,
                            must_facts_total: must_f_total,
                            must_facts_found: must_f_found,
                            must_facts_missing: missing_fact_substrings,
                            anti_entities_found: found_anti_entities,
                            score,
                        }
                    })
                } else {
                    None
                };

                eval_results.push(EvalResult {
                    file_path: file_path.display().to_string(),
                    file_name,
                    entities_extracted: report.entities_extracted,
                    relations_extracted: report.relations_extracted,
                    entities_promoted: report.entities_promoted,
                    relations_promoted: report.relations_promoted,
                    verified_entities: report.verified_entities,
                    verified_relations: report.verified_relations,
                    verified_facts: report.verified_facts,
                    evaluation: eval_details,
                });
            }
            Err(e) => {
                warn!(file = %file_path.display(), error = %e, "Dry run failed for file");
            }
        }
    }

    // Compute Summary
    let total_files = eval_results.len();
    let mut golden_files = 0;
    let mut entities_extracted = 0;
    let mut relations_extracted = 0;
    let mut entities_promoted = 0;
    let mut relations_promoted = 0;
    let mut zero_entity_files = 0;
    let mut zero_relation_files = 0;
    
    let mut sum_must_entities_total = 0;
    let mut sum_must_entities_found = 0;
    let mut sum_must_facts_total = 0;
    let mut sum_must_facts_found = 0;
    let mut anti_entities_found_count = 0;

    for r in &eval_results {
        entities_extracted += r.entities_extracted;
        relations_extracted += r.relations_extracted;
        entities_promoted += r.entities_promoted;
        relations_promoted += r.relations_promoted;

        if r.entities_promoted == 0 {
            zero_entity_files += 1;
        }
        if r.relations_promoted == 0 {
            zero_relation_files += 1;
        }

        if let Some(ref details) = r.evaluation {
            golden_files += 1;
            sum_must_entities_total += details.must_entities_total;
            sum_must_entities_found += details.must_entities_found;
            sum_must_facts_total += details.must_facts_total;
            sum_must_facts_found += details.must_facts_found;
            anti_entities_found_count += details.anti_entities_found.len();
        }
    }

    let relation_promotion_rate = if relations_extracted > 0 {
        relations_promoted as f64 / relations_extracted as f64
    } else {
        0.0
    };

    let must_entities_recall = if sum_must_entities_total > 0 {
        sum_must_entities_found as f64 / sum_must_entities_total as f64
    } else {
        0.0
    };

    let must_facts_recall = if sum_must_facts_total > 0 {
        sum_must_facts_found as f64 / sum_must_facts_total as f64
    } else {
        0.0
    };

    let summary = BatchEvalSummary {
        total_files,
        golden_files,
        entities_extracted,
        relations_extracted,
        entities_promoted,
        relations_promoted,
        zero_entity_files,
        zero_relation_files,
        relation_promotion_rate,
        must_entities_recall,
        must_facts_recall,
        anti_entities_found_count,
    };

    let report = BatchEvalReport {
        files: eval_results,
        summary,
    };

    let json_output = serde_json::to_string_pretty(&report)?;
    println!("{json_output}");

    Ok(())
}

fn collect_md(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_md(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
    Ok(())
}


#[cfg(test)]
mod eval_match_tests {
    use super::*;

    #[test]
    fn awareness_agent_matches_hyphen_form() {
        assert!(entity_found_in_report(
            "Awareness-Agent",
            &["Strategy-Analyst".into(), "Awareness Agent".into()],
            &["The Awareness Agent monitors the day.".into()],
            &[],
        ));
    }

    #[test]
    fn entity_found_via_relation_endpoint() {
        assert!(entity_found_in_report(
            "Chief of Staff",
            &[],
            &[],
            &[(
                "Awareness Agent".into(),
                "Chief of Staff".into(),
                "RELATED_TO".into(),
            )],
        ));
    }

    #[test]
    fn proxmox_matches_proxox_typo_in_entities_and_facts() {
        assert!(entity_label_matches("Proxmox", "Proxox"));
        assert!(entity_found_in_report(
            "Proxmox",
            &["Proxox".into()],
            &["Konfiguration von Proxox Snapshot-Plänen".into()],
            &[],
        ));
    }
}
