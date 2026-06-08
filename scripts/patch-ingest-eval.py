#!/usr/bin/env python3
from pathlib import Path

ev = Path("gzmo-cli/src/ingest_eval_cmd.rs")
t = ev.read_text()
old = """fn entity_found_in_report(must: &str, entities: &[String], facts: &[String]) -> bool {
    if entities
        .iter()
        .any(|e| entity_label_matches(must, e))
    {
        return true;
    }
    let must_norm = normalize_entity_label(must);
    facts.iter().any(|fact| {
        let fact_norm = normalize_entity_label(fact);
        fact_norm.contains(&must_norm)
    })
}
"""
new = """fn entity_found_in_report(
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
    let must_norm = normalize_entity_label(must);
    facts.iter().any(|fact| {
        let fact_norm = normalize_entity_label(fact);
        fact_norm.contains(&must_norm)
    })
}
"""
if old not in t:
    raise SystemExit("entity_found block not found")
t = t.replace(old, new, 1)
t = t.replace(
    """                            let found = entity_found_in_report(
                                must,
                                &report.verified_entities,
                                &report.verified_facts,
                            );""",
    """                            let found = entity_found_in_report(
                                must,
                                &report.verified_entities,
                                &report.verified_facts,
                                &report.verified_relations,
                            );""",
    1,
)
if "#[cfg(test)]" not in t:
    t += r'''

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
}
'''
ev.write_text(t)
print("ingest_eval_cmd.rs patched")
