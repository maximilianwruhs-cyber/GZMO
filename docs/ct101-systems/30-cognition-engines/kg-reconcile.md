# Subsystem — KG Reconcile

**Source:** `gzmo-core/src/kg_reconcile.rs`  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Daily gated Neo4j ontology reconciliation via MCP memory tools: canonicalize entity types and relation labels toward the shared PEOPLE/SYSTEMS/PROJECTS/TOOLS/DECISIONS ontology, with dry-run support.

---

## 2. How it works

### Canonical entity types

```12:70:gzmo-core/src/kg_reconcile.rs
const CANONICAL_ENTITY_TYPES: &[&str] = &[
    "PEOPLE", "SYSTEMS", "PROJECTS", "TOOLS", "DECISIONS",
];

pub fn canonicalize_entity_type(raw: &str) -> String {
    let upper = raw.trim().to_uppercase();
    if CANONICAL_ENTITY_TYPES.contains(&upper.as_str()) {
        return upper;
    }
    match upper.as_str() {
        "PERSON" | "PEOPLE" | "HUMAN" | "USER" => "PEOPLE".to_string(),
        "SYSTEM" | "SERVICE" | "INFRA" => "SYSTEMS".to_string(),
        // ...
    }
}
```

### Run pipeline

```72:98:gzmo-core/src/kg_reconcile.rs
pub async fn run_kg_reconcile(
    tools: &ToolRegistry,
    cfg: &KgReconcileConfig,
) -> Result<ReconcileReport> {
    if !tools.has_tool("mcp__memory__read_graph") {
        anyhow::bail!("mcp__memory__read_graph not available");
    }
    let result = tools.dispatch(&ToolCall {
        function_name: "mcp__memory__read_graph".to_string(),
        arguments: serde_json::json!({}),
    }).await;
    let graph: KnowledgeGraph = serde_json::from_str(&result.output)?;
```

### Relation recanonicalization

```135:196:gzmo-core/src/kg_reconcile.rs
    for rel in &graph.relations {
        let canon = canonicalize_relation_type(&rel.relation_type);
        if canon == rel.relation_type { continue; }
        to_delete.push(rel.clone());
        to_create.push(json!({ "source", "target", "relationType": canon }));
    }
    // dry_run → count only
    // else delete_relations batch + create_relations chunks of 20
```

### Daemon cron

```616:645:gzmo-cli/src/daemon_cmd.rs
    let kg_handle = tokio::spawn(async move {
        if !kg_cfg.enabled { continue; }
        if !cron_due_today(&now, kg_cfg.cron_hour, kg_cfg.cron_minute, last_run_date) { continue; }
        match gzmo_core::kg_reconcile::run_kg_reconcile(kg_tools.as_ref(), &kg_cfg).await {
            Ok(report) => info!(entities = report.entities_scanned, relations_fixed = report.relations_recanonicalized, ...),
        }
    });
```

---

## 3. Interfaces

| Interface | Config |
|-----------|--------|
| Enable | `[kg_reconcile] enabled` |
| Cron | `cron_hour`, `cron_minute` |
| Dry run | `[kg_reconcile] dry_run = true` |
| MCP tools | `read_graph`, `add_observations`, `delete_relations`, `create_relations` |
| Neo4j | sidecar Bolt + MCP memory server env |

**Live (2026-07-14):** 63,572 nodes, 64,224 relations in Neo4j.

---

## 4. THINKING nodes

> **THINKING — kg_reconcile.rs:delete+create relations**
> - *Reviewed:* Non-canonical relation types deleted and recreated with canonical label.
> - *Insight:* Neo4j relation type is immutable — must delete/recreate.
> - *Risk / limitation:* Batch failure mid-way leaves partial graph state.
> - *Enhancement:* Transactional batch with rollback log. [GZMO-next]

> **THINKING — kg_reconcile.rs:dry_run**
> - *Reviewed:* dry_run counts fixes without MCP writes.
> - *Insight:* Safe operator preview before enabling on production graph.
> - *Risk / limitation:* Entity note path still confusing vs relation path in dry_run.
> - *Enhancement:* Unified reconcile report file under `data/kg-reconcile/`. [CT101-safe]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| Shared ontology | Same types used by KgPromoter in dream/ingest/distill |
| GZMO-next | Could run reconcile pre-deploy as migration step |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Persist reconcile report JSONL | [CT101-safe] |
| 2 | Transactional relation rewrite | [GZMO-next] |
| 3 | Entity merge (duplicate names) | [GZMO-next] |
| 4 | Alert when relations_fixed > threshold | [CT101-safe] |
