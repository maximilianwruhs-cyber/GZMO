//! Prerequisite graph for Curriculum Planner agent.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrerequisiteNode {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default)]
    pub bloom_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrerequisiteGraph {
    pub domain: String,
    pub nodes: Vec<PrerequisiteNode>,
}

impl PrerequisiteGraph {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("read prerequisite graph {:?}", path))?;
        if path.extension().is_some_and(|e| e == "json") {
            serde_json::from_str(&raw).context("parse prerequisite graph JSON")
        } else {
            serde_yaml::from_str(&raw).context("parse prerequisite graph YAML")
        }
    }

    /// Load and merge all `*.yaml` / `*.yml` graphs in a directory.
    pub fn load_dir(dir: &Path) -> Result<Self> {
        if !dir.is_dir() {
            bail!("prerequisite graphs directory not found: {:?}", dir);
        }
        let mut paths: Vec<_> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml")
            })
            .collect();
        paths.sort();
        if paths.is_empty() {
            bail!("no YAML graphs in {:?}", dir);
        }
        let mut merged_nodes = Vec::new();
        let mut domain = String::from("merged");
        for path in paths {
            let graph = Self::load(&path)?;
            if merged_nodes.is_empty() {
                domain = graph.domain.clone();
            }
            merged_nodes.extend(graph.nodes);
        }
        Ok(Self {
            domain,
            nodes: merged_nodes,
        })
    }

    /// Schema check: unique IDs, valid prerequisite refs, no cycles.
    pub fn validate(&self) -> Result<()> {
        let ids: HashSet<&str> = self.nodes.iter().map(|n| n.id.as_str()).collect();
        if ids.len() != self.nodes.len() {
            bail!("duplicate node ids in domain '{}'", self.domain);
        }
        for node in &self.nodes {
            if node.id.is_empty() || node.title.is_empty() {
                bail!("node missing id or title in domain '{}'", self.domain);
            }
            for prereq in &node.prerequisites {
                if !ids.contains(prereq.as_str()) {
                    bail!(
                        "node '{}' references unknown prerequisite '{}'",
                        node.id,
                        prereq
                    );
                }
            }
        }
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for node in &self.nodes {
            for prereq in &node.prerequisites {
                adj.entry(prereq.as_str())
                    .or_default()
                    .push(node.id.as_str());
            }
        }
        let mut visiting: HashSet<&str> = HashSet::new();
        let mut visited: HashSet<&str> = HashSet::new();
        for id in ids.iter() {
            if Self::dfs_cycle(*id, &adj, &mut visiting, &mut visited) {
                bail!("cycle detected involving node '{}'", id);
            }
        }
        Ok(())
    }

    fn dfs_cycle<'a>(
        node: &'a str,
        adj: &HashMap<&'a str, Vec<&'a str>>,
        visiting: &mut HashSet<&'a str>,
        visited: &mut HashSet<&'a str>,
    ) -> bool {
        if visiting.contains(node) {
            return true;
        }
        if visited.contains(node) {
            return false;
        }
        visiting.insert(node);
        if let Some(children) = adj.get(node) {
            for child in children {
                if Self::dfs_cycle(child, adj, visiting, visited) {
                    return true;
                }
            }
        }
        visiting.remove(node);
        visited.insert(node);
        false
    }

    pub fn find_node(&self, query: &str) -> Option<&PrerequisiteNode> {
        let q = query.to_lowercase();
        self.nodes.iter().find(|n| {
            n.id.to_lowercase() == q
                || n.title.to_lowercase().contains(&q)
                || n.description.to_lowercase().contains(&q)
        })
    }

    pub fn unmastered_prerequisites(&self, node_id: &str, mastered: &[String]) -> Vec<String> {
        let Some(node) = self.nodes.iter().find(|n| n.id == node_id) else {
            return vec![];
        };
        node.prerequisites
            .iter()
            .filter(|p| !mastered.iter().any(|m| m == *p))
            .cloned()
            .collect()
    }

    pub fn planner_context(&self, topic_hint: &str) -> String {
        let mut ctx = format!("Domain: {}\nNodes:\n", self.domain);
        for node in &self.nodes {
            if topic_hint.is_empty()
                || node
                    .title
                    .to_lowercase()
                    .contains(&topic_hint.to_lowercase())
                || node.id.to_lowercase().contains(&topic_hint.to_lowercase())
            {
                ctx.push_str(&format!(
                    "- {} ({}): {} [requires: {}]\n",
                    node.id,
                    node.title,
                    node.description,
                    if node.prerequisites.is_empty() {
                        "none".to_string()
                    } else {
                        node.prerequisites.join(", ")
                    }
                ));
            }
        }
        ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_unknown_prerequisite() {
        let graph = PrerequisiteGraph {
            domain: "test".into(),
            nodes: vec![PrerequisiteNode {
                id: "a".into(),
                title: "A".into(),
                description: "desc".into(),
                prerequisites: vec!["missing".into()],
                bloom_level: "remember".into(),
            }],
        };
        assert!(graph.validate().is_err());
    }

    #[test]
    fn validate_accepts_dag() {
        let graph = PrerequisiteGraph {
            domain: "test".into(),
            nodes: vec![
                PrerequisiteNode {
                    id: "a".into(),
                    title: "A".into(),
                    description: "desc".into(),
                    prerequisites: vec![],
                    bloom_level: "remember".into(),
                },
                PrerequisiteNode {
                    id: "b".into(),
                    title: "B".into(),
                    description: "desc".into(),
                    prerequisites: vec!["a".into()],
                    bloom_level: "understand".into(),
                },
            ],
        };
        assert!(graph.validate().is_ok());
    }
}
