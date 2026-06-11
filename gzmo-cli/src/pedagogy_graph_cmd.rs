//! `gzmo pedagogy graph validate <file|dir>` — prerequisite graph checks.

use std::path::PathBuf;

use anyhow::{bail, Result};

use gzmo_core::pedagogy::PrerequisiteGraph;

pub async fn run(args: &[String]) -> Result<()> {
    if args.first().map(|s| s.as_str()) != Some("graph") {
        bail!("Usage: gzmo pedagogy graph validate <file.yaml|directory>");
    }
    if args.get(1).map(|s| s.as_str()) != Some("validate") {
        bail!("Usage: gzmo pedagogy graph validate <file.yaml|directory>");
    }
    let Some(target) = args.get(2) else {
        bail!("Usage: gzmo pedagogy graph validate <file.yaml|directory>");
    };
    let path = PathBuf::from(target);
    let graph = if path.is_dir() {
        PrerequisiteGraph::load_dir(&path)?
    } else {
        PrerequisiteGraph::load(&path)?
    };
    graph.validate()?;
    println!(
        "OK: {} ({} nodes)",
        graph.domain,
        graph.nodes.len()
    );
    Ok(())
}
