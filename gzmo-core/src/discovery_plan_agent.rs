//! Discovery plan agent — rich implementation dossier before fixer execute phase.
//!
//! Spawns a sub-agent with a path-only brief; output lands in Forum-2 `plans/`.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::discovery_fixer::ActionableFinding;

#[derive(Debug, Clone, Serialize)]
pub struct PlanFindingSeed {
    pub finding_id: String,
    pub kind: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct PlanOutputPaths {
    pub plan_dir: PathBuf,
    pub plan_md: PathBuf,
    pub plan_json: PathBuf,
    pub plan_provenance: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PlanVerification {
    pub passed: bool,
    pub notes: String,
}

pub fn log_spawn_brief(session_id: &str, brief: &str) {
    let dir = discovery_implementation_data_root().join("logs/spawn-briefs");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let safe = session_id.replace(':', "_");
    let snippet = crate::text_util::truncate_chars(brief, 2000);
    let path = dir.join(format!("{safe}.txt"));
    let _ = std::fs::write(path, snippet);
}

pub fn assert_discovery_parent_session(prefix: &str, parent_session: &str, agent_label: &str) {
    if !parent_session.starts_with(prefix) {
        tracing::warn!(
            parent_session = %parent_session,
            expected_prefix = %prefix,
            agent = %agent_label,
            "Discovery sub-agent parent_session prefix mismatch — context isolation risk"
        );
    }
    if parent_session.contains("arc_session")
        || parent_session.contains("discovery-implement:")
        || parent_session.contains("pi-mentor-discovery-arc")
    {
        tracing::warn!(
            parent_session = %parent_session,
            agent = %agent_label,
            "Discovery sub-agent parent_session may reuse Pi arc context"
        );
    }
}

pub fn discovery_implementation_data_root() -> PathBuf {
    std::env::var("DISCOVERY_IMPLEMENTATION_DATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let skills = std::env::var("GZMO_SKILLS_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("gzmo_skills")
                });
            skills.join("data/discovery-implementation")
        })
}

pub fn plan_id_from_report(report_path: &Path, discovery_session_id: &str) -> String {
    report_path
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(discovery_session_id)
        .to_string()
}

pub fn discovery_plan_session_id(plan_id: &str) -> String {
    format!("discovery-plan:{plan_id}")
}

pub fn is_discovery_plan_recommendation(
    rec: &crate::kurator_monitor::PendingRecommendation,
) -> bool {
    rec.kind.as_deref() == Some("discovery_plan")
        || rec.reason.starts_with("discovery_plan:")
        || rec.session_id.starts_with("discovery-plan:")
}

pub fn resolve_plan_output_paths(plan_id: &str) -> PlanOutputPaths {
    let plan_dir = discovery_implementation_data_root().join("plans").join(plan_id);
    PlanOutputPaths {
        plan_md: plan_dir.join("plan.md"),
        plan_json: plan_dir.join("plan.json"),
        plan_provenance: plan_dir.join("plan-provenance.json"),
        plan_dir,
    }
}

pub fn resolve_probe_results_dir() -> PathBuf {
    crate::discovery_code_implementer::resolve_probe_results_dir()
}

pub fn discovery_plan_reason(finding_count: usize) -> String {
    format!("discovery_plan: {finding_count} actionable finding(s) — write plan dossier")
}

pub fn findings_seed_json(findings: &[ActionableFinding]) -> String {
    let seeds: Vec<PlanFindingSeed> = findings
        .iter()
        .map(|f| PlanFindingSeed {
            finding_id: f.finding_id.clone(),
            kind: f.kind.as_str().to_string(),
            title: f.title.clone(),
        })
        .collect();
    serde_json::to_string(&seeds).unwrap_or_else(|_| "[]".into())
}

pub fn build_plan_agent_brief(
    report_path: &Path,
    discovery_session_id: &str,
    plan_id: &str,
    findings: &[ActionableFinding],
    output: &PlanOutputPaths,
    eval_feedback: Option<&str>,
    max_chars: usize,
) -> String {
    let probe_dir = resolve_probe_results_dir();
    let manifest = crate::discovery_code_implementer::resolve_implement_manifest_path(discovery_session_id);
    let seed = findings_seed_json(findings);
    let prompt_template = plan_prompt_template_path();

    let mut lines = vec![
        format!("Discovery plan agent for session `{discovery_session_id}` — write a rich implementation dossier."),
        format!("Plan id: {plan_id}"),
        format!("Report path (file_read): {}", report_path.display()),
        format!("Probe results dir (file_read): {}", probe_dir.display()),
        format!("Probe manifest (file_read): {}", manifest.display()),
        format!("Plan prompt template (file_read): {}", prompt_template.display()),
        String::new(),
        "Findings seed JSON (expand via file_read on report + probes):".to_string(),
        seed,
        String::new(),
        "Required outputs (file_write):".to_string(),
        format!("- {}", output.plan_md.display()),
        format!("- {}", output.plan_json.display()),
        format!("- {}", output.plan_provenance.display()),
        String::new(),
        "Task:".to_string(),
        "1. file_read report, probe JSONs, and relevant gzmo-core/gzmo.toml paths.".to_string(),
        "2. Write substantive plan.md (≥800 words; target 900+) with sequencing and sidecar vs core rationale.".to_string(),
        "3. Write plan.json with workstreams (acceptance ≥2 each, each MUST be a valid bash one-liner passing 'bash -n -c \"<entry>\"') and deferred[] for skipped findings.".to_string(),
        "4. Every actionable finding_id from the seed MUST appear in workstreams[].finding_ids OR deferred[].finding_id — uncovered findings fail eval.".to_string(),
        format!(
            "5. Required finding coverage (map each to ≥1 workstream): {}",
            findings
                .iter()
                .map(|f| f.finding_id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        "6. Write plan-provenance.json listing files_read and grep_queries.".to_string(),
        "7. Populate description and spawn_command fields in every workstream.".to_string(),
        "8. Path resolution for acceptance commands: $GZMO_ROOT for gzmo-core/ paths, $GZMO_SKILLS_ROOT for gzmo_skills/ paths (never survey_GZMO/gzmo_skills chimera paths).".to_string(),
        "9. Do NOT spawn sub-agents. Scope: survey_GZMO/ and gzmo_skills/ only.".to_string(),
        "10. If remediation history is present below, apply proven patterns and avoid listed failure modes.".to_string(),
        "11. On retry feedback: expand plan.md — never shorten below the prior word count.".to_string(),
        "12. complexity=complex workstreams MUST include gzmo-core/ or gzmo.toml in target_paths.".to_string(),
        "13. Prefer sidecar complexity=moderate for probe/script work; reserve complex for true core edits.".to_string(),
    ];

    if let Some(feedback) = eval_feedback {
        if !feedback.is_empty() {
            lines.push(String::new());
            lines.push("Prior plan eval FAILED — address this feedback:".to_string());
            lines.push(feedback.to_string());
        }
    }

    if let Ok(remediation_history) = std::env::var("DISCOVERY_PLAN_REMEDIATION_HISTORY") {
        if !remediation_history.is_empty() {
            lines.push(String::new());
            lines.push("Remediation history from past pipeline runs:".to_string());
            lines.push(remediation_history);
        }
    }

    crate::text_util::truncate_chars(&lines.join("\n"), max_chars)
}

pub fn plan_md_word_count(path: &Path) -> usize {
    std::fs::read_to_string(path)
        .map(|s| s.split_whitespace().count())
        .unwrap_or(0)
}

fn acceptance_has_chimera_path(cmd: &str) -> bool {
    cmd.contains("survey_GZMO/gzmo_skills")
        || cmd.contains("gzmo_skills/survey_GZMO")
        || cmd.contains("GZMO_SKILLS_ROOT/survey_GZMO/")
}

fn complex_workstream_has_core_target(workstream: &serde_json::Value) -> bool {
    workstream
        .get("target_paths")
        .and_then(|a| a.as_array())
        .map(|paths| {
            paths.iter().any(|p| {
                p.as_str()
                    .map(|s| s.contains("gzmo-core/") || s.contains("gzmo.toml"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn covered_finding_ids(plan_json: &serde_json::Value) -> std::collections::HashSet<String> {
    let mut covered = std::collections::HashSet::new();
    if let Some(workstreams) = plan_json.get("workstreams").and_then(|a| a.as_array()) {
        for ws in workstreams {
            if let Some(ids) = ws.get("finding_ids").and_then(|a| a.as_array()) {
                for id in ids {
                    if let Some(s) = id.as_str() {
                        covered.insert(s.to_string());
                    }
                }
            }
        }
    }
    if let Some(deferred) = plan_json.get("deferred").and_then(|a| a.as_array()) {
        for entry in deferred {
            if let Some(fid) = entry.get("finding_id").and_then(|v| v.as_str()) {
                covered.insert(fid.to_string());
            }
        }
    }
    covered
}

fn plan_prompt_template_path() -> PathBuf {
    let skills = std::env::var("GZMO_SKILLS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("gzmo_skills")
        });
    skills.join("prompts/discovery-implementation/plan-agent.md")
}

pub fn verify_plan_agent_outcome(
    output: &PlanOutputPaths,
    written_paths: &[String],
    findings: &[ActionableFinding],
) -> PlanVerification {
    let mut notes = Vec::new();
    let mut passed = true;

    for label in [
        ("plan.md", &output.plan_md),
        ("plan.json", &output.plan_json),
        ("plan-provenance.json", &output.plan_provenance),
    ] {
        let exists_on_disk = label.1.is_file();
        let claimed = written_paths.iter().any(|p| p.contains(label.0));
        if !exists_on_disk && !claimed {
            passed = false;
            notes.push(format!("missing {}", label.0));
        }
    }

    let finding_count = findings.len();

    if output.plan_json.is_file() {
        if let Ok(raw) = std::fs::read_to_string(&output.plan_json) {
            match serde_json::from_str::<serde_json::Value>(&raw) {
                Err(_) => {
                    passed = false;
                    notes.push("plan.json invalid JSON".into());
                }
                Ok(val) => {
                    let covered = covered_finding_ids(&val);
                    for finding in findings {
                        if !covered.contains(&finding.finding_id) {
                            passed = false;
                            notes.push(format!(
                                "finding {} not in workstreams or deferred",
                                finding.finding_id
                            ));
                        }
                    }
                    if let Some(workstreams) = val.get("workstreams").and_then(|a| a.as_array()) {
                        for ws in workstreams {
                            let wid = ws.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                            if ws.get("complexity").and_then(|v| v.as_str()) == Some("complex")
                                && !complex_workstream_has_core_target(ws)
                            {
                                passed = false;
                                notes.push(format!(
                                    "{wid} complex workstream lacks gzmo-core/ or gzmo.toml target_paths"
                                ));
                            }
                            if let Some(acceptance) =
                                ws.get("acceptance").and_then(|a| a.as_array())
                            {
                                for (i, entry) in acceptance.iter().enumerate() {
                                    if let Some(cmd) = entry.as_str() {
                                        if acceptance_has_chimera_path(cmd) {
                                            passed = false;
                                            notes.push(format!(
                                                "{wid} acceptance[{i}] uses chimera path; use $GZMO_ROOT or $GZMO_SKILLS_ROOT"
                                            ));
                                        }
                                        let syntax_ok = std::process::Command::new("bash")
                                            .args(["-n", "-c", cmd])
                                            .output()
                                            .map(|o| o.status.success())
                                            .unwrap_or(false);
                                        if !syntax_ok {
                                            passed = false;
                                            notes.push(format!(
                                                "{wid} acceptance[{i}] bash syntax error: {cmd}"
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if output.plan_md.is_file() {
        let words = plan_md_word_count(&output.plan_md);
        if words < 800 {
            passed = false;
            notes.push(format!("plan.md word count {words} < 800"));
        }
    }

    if output.plan_provenance.is_file() {
        if let Ok(raw) = std::fs::read_to_string(&output.plan_provenance) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                let read_count = v
                    .get("files_read")
                    .and_then(|a| a.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let min_reads = finding_count.min(3);
                if read_count < min_reads {
                    passed = false;
                    notes.push(format!(
                        "provenance files_read {read_count} < min {min_reads}"
                    ));
                }
            } else {
                passed = false;
                notes.push("plan-provenance.json invalid JSON".into());
            }
        }
    }

    PlanVerification {
        passed,
        notes: if notes.is_empty() {
            "plan artifacts present".into()
        } else {
            notes.join("; ")
        },
    }
}

pub fn is_discovery_agent_brief(brief: &str) -> bool {
    brief.contains("Discovery plan agent")
}

pub fn plan_agent_write_phase_message() -> String {
    "WRITE PHASE — stop reading and exploring. Use file_write NOW to write all three required outputs \
(plan.md, plan.json, plan-provenance.json) to the exact absolute paths listed under \
\"Required outputs (file_write)\" in your task. Do not finish until all three files exist on disk. \
Never output tool-call XML or markdown code fences pretending to be files — call the file_write tool.".to_string()
}

pub fn plan_agent_require_file_write_prompt(brief: &str) -> String {
    let paths: Vec<&str> = brief
        .lines()
        .filter(|l| l.starts_with("- /") && (l.contains("plan.md") || l.contains("plan.json") || l.contains("plan-provenance")))
        .map(|l| l.trim_start_matches("- ").trim())
        .collect();
    if paths.is_empty() {
        return plan_agent_write_phase_message();
    }
    format!(
        "STOP — no file_write yet. You MUST file_write these paths now:\n{}\n\
         Do not reply with text only or pseudo tool-call markup.",
        paths.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery_fixer::FindingKind;

    #[test]
    fn brief_is_path_allowlist() {
        let output = resolve_plan_output_paths("sess-final");
        let brief = build_plan_agent_brief(
            Path::new("/tmp/report.md"),
            "sess-1",
            "sess-final",
            &[ActionableFinding {
                finding_id: "F1".into(),
                title: "Gap".into(),
                kind: FindingKind::Gap,
                excerpt: "x".repeat(600),
            }],
            &output,
            None,
            12_000,
        );
        assert!(brief.contains("file_read"));
        assert!(brief.contains("Findings seed JSON"));
        assert!(!brief.contains(&"x".repeat(600)));
    }

    #[test]
    fn plan_agent_brief_includes_remediation_history_env() {
        let output = resolve_plan_output_paths("sess-final");
        std::env::set_var(
            "DISCOVERY_PLAN_REMEDIATION_HISTORY",
            "Proven fixes:\n- [path] rewrite paths",
        );
        let brief = build_plan_agent_brief(
            Path::new("/tmp/report.md"),
            "sess-1",
            "sess-final",
            &[],
            &output,
            None,
            12_000,
        );
        std::env::remove_var("DISCOVERY_PLAN_REMEDIATION_HISTORY");
        assert!(brief.contains("Remediation history from past pipeline runs"));
        assert!(brief.contains("rewrite paths"));
    }

    fn temp_plan_output(prefix: &str) -> PlanOutputPaths {
        let plan_dir = std::env::temp_dir().join(format!("gzmo-plan-test-{prefix}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&plan_dir);
        std::fs::create_dir_all(&plan_dir).unwrap();
        PlanOutputPaths {
            plan_md: plan_dir.join("plan.md"),
            plan_json: plan_dir.join("plan.json"),
            plan_provenance: plan_dir.join("plan-provenance.json"),
            plan_dir,
        }
    }

    #[test]
    fn verify_rejects_uncovered_finding() {
        let output = temp_plan_output("uncovered");
        std::fs::write(&output.plan_md, "word ".repeat(850)).unwrap();
        std::fs::write(
            &output.plan_json,
            r#"{"workstreams":[{"id":"WS1","finding_ids":["F1"],"acceptance":["test -f /tmp/x"]}],"deferred":[]}"#,
        )
        .unwrap();
        std::fs::write(
            &output.plan_provenance,
            r#"{"files_read":["a","b","c"],"grep_queries":[]}"#,
        )
        .unwrap();
        let findings = vec![
            ActionableFinding {
                finding_id: "F1".into(),
                title: "One".into(),
                kind: FindingKind::Gap,
                excerpt: String::new(),
            },
            ActionableFinding {
                finding_id: "F2".into(),
                title: "Two".into(),
                kind: FindingKind::Gap,
                excerpt: String::new(),
            },
        ];
        let v = verify_plan_agent_outcome(&output, &[], &findings);
        assert!(!v.passed);
        assert!(v.notes.contains("F2"));
    }

    #[test]
    fn verify_accepts_deferred_finding() {
        let output = temp_plan_output("deferred");
        std::fs::write(&output.plan_md, "word ".repeat(850)).unwrap();
        std::fs::write(
            &output.plan_json,
            r#"{"workstreams":[{"id":"WS1","finding_ids":["F1"],"acceptance":["test -f /tmp/x","test -f /tmp/y"]}],"deferred":[{"finding_id":"F2","reason":"later"}]}"#,
        )
        .unwrap();
        std::fs::write(
            &output.plan_provenance,
            r#"{"files_read":["a","b","c"],"grep_queries":[]}"#,
        )
        .unwrap();
        let findings = vec![
            ActionableFinding {
                finding_id: "F1".into(),
                title: "One".into(),
                kind: FindingKind::Gap,
                excerpt: String::new(),
            },
            ActionableFinding {
                finding_id: "F2".into(),
                title: "Two".into(),
                kind: FindingKind::Gap,
                excerpt: String::new(),
            },
        ];
        let v = verify_plan_agent_outcome(&output, &[], &findings);
        assert!(v.passed, "{}", v.notes);
    }

    #[test]
    fn verify_rejects_chimera_acceptance_path() {
        let output = temp_plan_output("chimera");
        std::fs::write(&output.plan_md, "word ".repeat(850)).unwrap();
        std::fs::write(
            &output.plan_json,
            r#"{"workstreams":[{"id":"WS1","finding_ids":["F1"],"complexity":"moderate","acceptance":["test -f $GZMO_SKILLS_ROOT/survey_GZMO/scripts/x.sh","test -f /tmp/y"]}],"deferred":[]}"#,
        )
        .unwrap();
        std::fs::write(
            &output.plan_provenance,
            r#"{"files_read":["a","b","c"],"grep_queries":[]}"#,
        )
        .unwrap();
        let findings = vec![ActionableFinding {
            finding_id: "F1".into(),
            title: "One".into(),
            kind: FindingKind::Gap,
            excerpt: String::new(),
        }];
        let v = verify_plan_agent_outcome(&output, &[], &findings);
        assert!(!v.passed);
        assert!(v.notes.contains("chimera"));
    }

    #[test]
    fn verify_rejects_complex_without_core_target() {
        let output = temp_plan_output("complex");
        std::fs::write(&output.plan_md, "word ".repeat(850)).unwrap();
        std::fs::write(
            &output.plan_json,
            r#"{"workstreams":[{"id":"WS1","finding_ids":["F1"],"complexity":"complex","target_paths":["gzmo_skills/scripts/x.sh"],"acceptance":["test -f /tmp/x","test -f /tmp/y"]}],"deferred":[]}"#,
        )
        .unwrap();
        std::fs::write(
            &output.plan_provenance,
            r#"{"files_read":["a","b","c"],"grep_queries":[]}"#,
        )
        .unwrap();
        let findings = vec![ActionableFinding {
            finding_id: "F1".into(),
            title: "One".into(),
            kind: FindingKind::Gap,
            excerpt: String::new(),
        }];
        let v = verify_plan_agent_outcome(&output, &[], &findings);
        assert!(!v.passed);
        assert!(v.notes.contains("complex workstream"));
    }
}
