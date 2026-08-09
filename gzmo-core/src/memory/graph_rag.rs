//! 2-Hop Graph-RAG Subgraph Expansion for Multi-Hop Relational Retrieval.
//!
//! Traverses knowledge graph relationships in SQLite (`semantic_vault` + `honeypot` entity chains)
//! to extract 2-hop concept chains (A -> R1 -> B -> R2 -> C) to feed RRF and reranker.

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubgraphHop {
    pub source_entity: String,
    pub relation: String,
    pub target_entity: String,
    pub fact_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubgraphChain {
    pub seed_entity: String,
    pub hops: Vec<SubgraphHop>,
}

/// Extract all `[TYPE:Name]` entity tags from a fact string.
pub fn extract_all_entities(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find('[') {
        let rest_inner = &rest[start + 1..];
        let Some(end) = rest_inner.find(']') else { break };
        let inner = &rest_inner[..end];
        if let Some((_tag, name)) = inner.split_once(':') {
            let s = name.trim();
            if s.len() >= 2 {
                out.push(s.to_string());
            }
        }
        rest = &rest_inner[end + 1..];
    }
    out
}

/// Perform 2-hop Graph-RAG traversal starting from seed entity tokens.
pub fn traverse_2hop_subgraph(
    conn: &Connection,
    seed_entities: &[String],
    max_chains: usize,
) -> Result<Vec<SubgraphChain>> {
    let mut chains = Vec::new();
    if seed_entities.is_empty() {
        return Ok(chains);
    }

    for seed in seed_entities {
        let pattern = format!("%{}%", seed.replace('%', ""));

        // Step 1: Hop 1 — find direct facts mentioning seed entity
        let mut stmt1 = conn.prepare(
            "SELECT content FROM honeypot
             WHERE is_latest = 1 AND content LIKE ?1
             LIMIT 10",
        )?;

        let hop1_rows: Vec<String> = stmt1
            .query_map(params![pattern], |r| r.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        for content1 in hop1_rows {
            let entities = extract_all_entities(&content1);
            for target1 in entities {
                if target1.to_lowercase() == seed.to_lowercase() {
                    continue;
                }

                // Step 2: Hop 2 — find facts mentioning target1
                let target1_pattern = format!("%{}%", target1.replace('%', ""));
                let mut stmt2 = conn.prepare(
                    "SELECT content FROM honeypot
                     WHERE is_latest = 1
                       AND content LIKE ?1
                       AND content NOT LIKE ?2
                     LIMIT 5",
                )?;

                let hop2_rows: Vec<String> = stmt2
                    .query_map(params![target1_pattern, pattern], |r| r.get(0))?
                    .filter_map(|r| r.ok())
                    .collect();

                let mut hops = vec![SubgraphHop {
                    source_entity: seed.clone(),
                    relation: "EXTENDS".to_string(),
                    target_entity: target1.clone(),
                    fact_content: content1.clone(),
                }];

                for content2 in hop2_rows {
                    for target2 in extract_all_entities(&content2) {
                        if target2.to_lowercase() != target1.to_lowercase()
                            && target2.to_lowercase() != seed.to_lowercase()
                        {
                            hops.push(SubgraphHop {
                                source_entity: target1.clone(),
                                relation: "LINKED_TO".to_string(),
                                target_entity: target2,
                                fact_content: content2.clone(),
                            });
                        }
                    }
                }

                chains.push(SubgraphChain {
                    seed_entity: seed.clone(),
                    hops,
                });

                if chains.len() >= max_chains {
                    return Ok(chains);
                }
            }
        }
    }

    Ok(chains)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_all_entities() {
        let text = "[CONCEPT:GZMO] uses [SERVICE:Prime] on [HOST:CT101]";
        let ents = extract_all_entities(text);
        assert_eq!(ents, vec!["GZMO", "Prime", "CT101"]);
    }
}
