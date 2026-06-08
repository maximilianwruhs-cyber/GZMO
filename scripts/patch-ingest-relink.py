#!/usr/bin/env python3
from pathlib import Path

ing = Path("gzmo-core/src/ingest.rs")
t = ing.read_text()
t = t.replace(
    "use crate::memory::kg_extract::{\n    chunk_text_for_llm, KgPromoter, PipelineResult, VerifiedEntity, VerifiedRelation,\n};",
    "use crate::memory::kg_extract::{\n    chunk_text_for_llm, KgPromoter, PipelineResult, VerifiedEntity, VerifiedRelation,\n};",
)
if "relink_relations_after_entities" not in t:
    t = t.replace(
        "        append_inferred_relations(\n            &mut pipeline.verified_entities,\n            &mut pipeline.verified_relations,\n            prepared.doc_class,\n            &prepared.frontmatter,\n            &prepared.file_name,\n            self.config.min_confidence,\n        );\n        ensure_primary_agent_entity(\n            &mut pipeline.verified_entities,\n            prepared.doc_class,\n            &prepared.file_name,\n            &prepared.frontmatter,\n            self.config.min_confidence,\n        );\n\n        Ok((pipeline, chunks.len()))",
        "        ensure_primary_agent_entity(\n            &mut pipeline.verified_entities,\n            prepared.doc_class,\n            &prepared.file_name,\n            &prepared.frontmatter,\n            self.config.min_confidence,\n        );\n        relink_relations_after_entities(\n            &mut pipeline,\n            prepared.doc_class,\n            self.config.min_confidence,\n        );\n        append_inferred_relations(\n            &mut pipeline.verified_entities,\n            &mut pipeline.verified_relations,\n            prepared.doc_class,\n            &prepared.frontmatter,\n            &prepared.file_name,\n            self.config.min_confidence,\n        );\n\n        Ok((pipeline, chunks.len()))",
    )
    relink_fn = '''
fn relink_relations_after_entities(
    pipeline: &mut PipelineResult,
    doc_class: DocClass,
    min_confidence: f64,
) {
    if doc_class != DocClass::AgentSpec && doc_class != DocClass::Reference {
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
        .filter(|r| kept_names.contains(&r.from) && kept_names.contains(&r.to))
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

'''
    t = t.replace("fn ensure_primary_agent_entity(", relink_fn + "fn ensure_primary_agent_entity(", 1)
ing.write_text(t)
print("ingest.rs patched")
