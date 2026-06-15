//! KG-aware low tension dialogue opening generator.
//!
//! Evaluates the prerequisite graph against recently touched concepts and the learner profile
//! to select 1-2 low-recall concepts or unmastered prerequisites.

use anyhow::Result;
use std::path::PathBuf;

use gzmo_chaos::pulse::ChaosSnapshot;
use crate::config::GzmoConfig;
use crate::pedagogy::graph::{PrerequisiteGraph, PrerequisiteNode};
use crate::pedagogy::learner::LearnerProfile;
use crate::memory::vault::SqliteVault;
use crate::tools::ToolRegistry;

use super::low_tension_persist::prior_opening_hints;

#[derive(Debug, Clone)]
pub struct LowTensionOpening {
    pub prompt: String,
    pub concept_ids: Vec<String>,
    pub concept_titles: Vec<String>,
}

impl LowTensionOpening {
    fn from_template(template: &str, snap: &ChaosSnapshot) -> Self {
        Self {
            prompt: format_opening_fallback(template, snap),
            concept_ids: vec![],
            concept_titles: vec![],
        }
    }
}

fn is_node_mastered(node: &PrerequisiteNode, mastered_vectors: &[String]) -> bool {
    let id_lower = node.id.to_lowercase();
    let title_lower = node.title.to_lowercase();
    for mv in mastered_vectors {
        let mv_lower = mv.to_lowercase();
        if mv_lower.contains(&id_lower)
            || id_lower.contains(&mv_lower)
            || mv_lower.contains(&title_lower)
            || title_lower.contains(&mv_lower)
        {
            return true;
        }
    }
    false
}

pub async fn build_opening(
    config: &GzmoConfig,
    learner_profile: &LearnerProfile,
    snap: &ChaosSnapshot,
    tools: Option<&ToolRegistry>,
) -> Result<LowTensionOpening> {
    // 1. Load prerequisite graph
    let graphs_dir = PathBuf::from(&config.pedagogy.prerequisite_graphs_dir);
    let graph_opt = if graphs_dir.is_dir() {
        PrerequisiteGraph::load_dir(&graphs_dir).ok()
    } else {
        None
    };

    let Some(graph) = graph_opt else {
        return Ok(LowTensionOpening::from_template(
            &config.pedagogy.low_tension_dialogue.opening_template,
            snap,
        ));
    };

    if graph.nodes.is_empty() {
        return Ok(LowTensionOpening::from_template(
            &config.pedagogy.low_tension_dialogue.opening_template,
            snap,
        ));
    }

    // 2. Query recent facts from the Sqlite vault
    let recent_facts = match SqliteVault::open(&config.memory.vault_db) {
        Ok(vault) => vault.recent_semantic_facts(30).unwrap_or_default(),
        Err(_) => vec![],
    };

    // 3. Scan graph nodes for recently touched ones (mentioned in recent facts or episodic memory)
    let mut recently_touched_nodes = vec![];
    for node in &graph.nodes {
        let id_lower = node.id.to_lowercase();
        let title_lower = node.title.to_lowercase();

        let mut touched = false;
        for fact in &recent_facts {
            let content_lower = fact.content.to_lowercase();
            if content_lower.contains(&id_lower) || content_lower.contains(&title_lower) {
                touched = true;
                break;
            }
        }

        if !touched {
            let start_idx = learner_profile.episodic.entries.len().saturating_sub(5);
            for entry in &learner_profile.episodic.entries[start_idx..] {
                let sum_lower = entry.summary.to_lowercase();
                let struggle_lower = entry.struggle.as_ref().map(|s| s.to_lowercase()).unwrap_or_default();
                if sum_lower.contains(&id_lower)
                    || sum_lower.contains(&title_lower)
                    || struggle_lower.contains(&id_lower)
                    || struggle_lower.contains(&title_lower)
                {
                    touched = true;
                    break;
                }
            }
        }

        if touched {
            recently_touched_nodes.push(node);
        }
    }

    // 4. Identify low-recall concepts or unmastered prerequisites
    let mut unmastered_prereqs = vec![];
    let mut low_recall_concepts = vec![];

    for node in &recently_touched_nodes {
        let node_mastered = is_node_mastered(node, &learner_profile.semantic.mastery_vectors);
        if !node_mastered {
            low_recall_concepts.push((*node).clone());
        }

        // Check prerequisites
        for prereq_id in &node.prerequisites {
            if let Some(prereq_node) = graph.nodes.iter().find(|n| &n.id == prereq_id) {
                if !is_node_mastered(prereq_node, &learner_profile.semantic.mastery_vectors) {
                    if !unmastered_prereqs.iter().any(|n: &PrerequisiteNode| &n.id == prereq_id) {
                        unmastered_prereqs.push(prereq_node.clone());
                    }
                }
            }
        }
    }

    // 5. Select target nodes
    let mut targets = vec![];
    if !unmastered_prereqs.is_empty() {
        // Prioritize unmastered prerequisites of recently touched concepts
        targets.extend(unmastered_prereqs.into_iter().take(2));
    } else if !low_recall_concepts.is_empty() {
        // Fallback to the unmastered concepts themselves
        targets.extend(low_recall_concepts.into_iter().take(2));
    } else {
        // Fallback to any unmastered concept in the entire graph
        let mut any_unmastered = vec![];
        for node in &graph.nodes {
            if !is_node_mastered(node, &learner_profile.semantic.mastery_vectors) {
                any_unmastered.push(node.clone());
            }
        }
        any_unmastered.sort_by_key(|n| n.prerequisites.len());
        targets.extend(any_unmastered.into_iter().take(2));
    }

    if targets.is_empty() {
        return Ok(LowTensionOpening::from_template(
            &config.pedagogy.low_tension_dialogue.opening_template,
            snap,
        ));
    }

    let concept_ids: Vec<String> = targets.iter().map(|t| t.id.clone()).collect();
    let concept_titles: Vec<String> = targets.iter().map(|t| t.title.clone()).collect();

    let prior_hints = if let Some(tools) = tools {
        prior_opening_hints(tools, &concept_ids).await
    } else {
        vec![]
    };

    let concepts_desc = targets
        .iter()
        .map(|t| format!("'{}' ({}: {})", t.title, t.id, t.description))
        .collect::<Vec<_>>()
        .join(" and ");

    let avoid_repeat = if prior_hints.is_empty() {
        String::new()
    } else {
        format!(
            " Do not repeat these prior question stems: {}.",
            prior_hints.join(" | ")
        )
    };

    let custom_prompt = format!(
        "[AUTONOMOUS — low tension] System tension is very low (τ={:.1}%, tick {}, phase {}). \
         The learner has recently touched or needs reinforcement on: {}. \
         Begin a Socratic dialogue with the learner: ask one inviting, context-rich question to probe their understanding of this/these concepts. \
         Connect this to stillness or the calmness of the chaos field. Do not lecture; do not give the answer.{}",
        snap.tension, snap.tick, snap.phase, concepts_desc, avoid_repeat
    );

    Ok(LowTensionOpening {
        prompt: custom_prompt,
        concept_ids,
        concept_titles,
    })
}

fn format_opening_fallback(template: &str, snap: &ChaosSnapshot) -> String {
    template
        .replace("{tension}", &format!("{:.1}", snap.tension))
        .replace("{tick}", &snap.tick.to_string())
        .replace("{phase}", &format!("{}", snap.phase))
}
