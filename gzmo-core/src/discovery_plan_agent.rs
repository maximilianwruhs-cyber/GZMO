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
        "7. Populate description in every workstream. spawn_command is OPTIONAL — omit it to use the default fixer path (gzmo kurator execute-workstream). If set, MUST start with `gzmo ` or `bash ` (examples: `bash $GZMO_SKILLS_ROOT/scripts/foo.sh`, `bash -c \"grep -q x $GZMO_ROOT/gzmo.toml\"`, `gzmo kurator execute-workstream --plan ABS_PLAN_DIR --workstream W1 --spawn`). Never bare script paths or unqualified shell one-liners.".to_string(),
        "8. Path resolution for acceptance commands: $GZMO_ROOT for gzmo-core/ paths, $GZMO_SKILLS_ROOT for gzmo_skills/ paths (never survey_GZMO/gzmo_skills chimera paths).".to_string(),
        "9. Do NOT spawn sub-agents. Scope: survey_GZMO/ and gzmo_skills/ only.".to_string(),
        "10. If remediation history is present below, apply proven patterns and avoid listed failure modes.".to_string(),
        "11. On retry feedback: expand plan.md — never shorten below the prior word count.".to_string(),
        "12. complexity=complex workstreams MUST include gzmo-core/ or gzmo.toml in target_paths.".to_string(),
        "13. Prefer sidecar complexity=moderate for probe/script work; reserve complex for true core edits.".to_string(),
        "14. Populate optional file_ownership map (path -> workstream id) for parallel dispatch safety.".to_string(),
        "15. Optional unaddressable[] for findings that cannot be automated (issue, reason, suggested_owner).".to_string(),
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

    let gzmo_root = std::env::var("GZMO_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));
    lines.push(crate::discovery_git_context::collect_git_context(&gzmo_root));

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

fn target_path_is_gitignored_data(path: &str) -> bool {
    let p = path.trim_start_matches("./");
    if p.starts_with("gzmo_skills/data/") || p.contains("/gzmo_skills/data/") {
        return true;
    }
    if p.starts_with("data/pi-mentor-discovery/") || p.contains("/data/pi-mentor-discovery/") {
        return true;
    }
    if let Ok(skills_root) = std::env::var("GZMO_SKILLS_ROOT") {
        let prefix = format!("{}/data/", skills_root.trim_end_matches('/'));
        if path.starts_with(&prefix) {
            return true;
        }
    }
    false
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

fn spawn_command_valid(cmd: &str) -> bool {
    let cmd = cmd.trim();
    cmd.is_empty() || cmd.starts_with("gzmo ") || cmd.starts_with("bash ")
}

fn normalize_sidecar_path(raw: &str) -> String {
    let mut p = raw.trim_start_matches("./").to_string();
    if let Some(rest) = p.strip_prefix("gzmo_skills/") {
        p = rest.to_string();
    }
    if let Some(rest) = p.strip_prefix("$GZMO_SKILLS_ROOT/") {
        p = rest.to_string();
    }
    if let Ok(skills_root) = std::env::var("GZMO_SKILLS_ROOT") {
        let prefix = format!("{}/", skills_root.trim_end_matches('/'));
        if let Some(rest) = p.strip_prefix(&prefix) {
            p = rest.to_string();
        }
    }
    p
}

fn sidecar_writer_allowlisted(path: &str) -> bool {
    path.contains("write-sidecar-remediation.sh")
        || path.contains("ensure-loop-proof-marker.sh")
        || path.contains("discovery-probes/")
}

fn resolve_bash_spawn_path(cmd: &str) -> Option<String> {
    let cmd = cmd.trim();
    if !cmd.starts_with("bash ") {
        return None;
    }
    let rest = cmd.trim_start_matches("bash ").trim();
    let token = rest.split_whitespace().next()?;
    if token == "-c" {
        return None;
    }
    let mut path = token.to_string();
    if let Some(suffix) = path.strip_prefix("$GZMO_SKILLS_ROOT/") {
        if let Ok(skills_root) = std::env::var("GZMO_SKILLS_ROOT") {
            path = format!("{}/{}", skills_root.trim_end_matches('/'), suffix);
        } else {
            path = suffix.to_string();
        }
    } else if !path.starts_with('/') {
        if let Ok(skills_root) = std::env::var("GZMO_SKILLS_ROOT") {
            path = format!("{}/{}", skills_root.trim_end_matches('/'), path);
        }
    }
    Some(path)
}

fn spawn_is_chicken_egg(workstream: &serde_json::Value, spawn_path: &str) -> bool {
    let norm_spawn = normalize_sidecar_path(spawn_path);
    if let Some(targets) = workstream.get("target_paths").and_then(|a| a.as_array()) {
        for tp in targets {
            if let Some(s) = tp.as_str() {
                if normalize_sidecar_path(s) == norm_spawn {
                    return true;
                }
            }
        }
    }
    false
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
                            if let Some(spawn_cmd) = ws.get("spawn_command").and_then(|v| v.as_str()) {
                                if !spawn_command_valid(spawn_cmd) {
                                    passed = false;
                                    notes.push(format!(
                                        "{wid} spawn_command must start with 'gzmo ' or 'bash ' (or omit spawn_command)"
                                    ));
                                } else if let Some(resolved) = resolve_bash_spawn_path(spawn_cmd) {
                                    if spawn_is_chicken_egg(ws, &resolved) {
                                        passed = false;
                                        notes.push(format!(
                                            "{wid} spawn_command chicken-egg: cannot bash a target_path — use write-sidecar-remediation.sh or omit spawn"
                                        ));
                                    } else if !std::path::Path::new(&resolved).is_file()
                                        && !sidecar_writer_allowlisted(&resolved)
                                    {
                                        passed = false;
                                        notes.push(format!(
                                            "{wid} spawn_command references missing script '{resolved}' — use write-sidecar-remediation.sh or omit spawn"
                                        ));
                                    }
                                }
                            }
                            if let Some(target_paths) =
                                ws.get("target_paths").and_then(|a| a.as_array())
                            {
                                for (i, tp) in target_paths.iter().enumerate() {
                                    if let Some(path) = tp.as_str() {
                                        if target_path_is_gitignored_data(path) {
                                            passed = false;
                                            notes.push(format!(
                                                "{wid} target_paths[{i}] '{path}' is under gitignored data/ — use gzmo_skills/scripts/discovery-remediations/<session_id>/ instead"
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    let ownership_conflicts = validate_workstream_ownership(&val);
                    if !ownership_conflicts.is_empty() {
                        passed = false;
                        notes.extend(ownership_conflicts);
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

/// Jules fleet-dispatch pattern: no two workstreams may claim the same target path.
pub fn workstream_target_paths(workstream: &serde_json::Value) -> Vec<String> {
    let mut paths = Vec::new();
    if let Some(arr) = workstream.get("target_paths").and_then(|a| a.as_array()) {
        for p in arr {
            if let Some(s) = p.as_str() {
                paths.push(s.to_string());
            }
        }
    }
    if let Some(spawn) = workstream.get("spawn_command").and_then(|v| v.as_str()) {
        paths.push(spawn.to_string());
    }
    paths
}

pub fn validate_workstream_ownership(plan_json: &serde_json::Value) -> Vec<String> {
    let mut claimed: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut conflicts = Vec::new();
    let Some(workstreams) = plan_json.get("workstreams").and_then(|a| a.as_array()) else {
        return conflicts;
    };
    for ws in workstreams {
        let wid = ws.get("id").and_then(|v| v.as_str()).unwrap_or("?");
        for path in workstream_target_paths(ws) {
            if let Some(other) = claimed.get(&path) {
                conflicts.push(format!(
                    "ownership conflict: \"{path}\" claimed by both \"{other}\" and \"{wid}\""
                ));
            } else {
                claimed.insert(path, wid.to_string());
            }
        }
    }
    conflicts
}

pub fn plan_approval_required() -> bool {
    std::env::var("DISCOVERY_PLAN_REQUIRE_APPROVAL")
        .map(|v| !matches!(v.as_str(), "0" | "false" | "no"))
        .unwrap_or(true)
}

pub fn is_plan_approved(plan_dir: &Path) -> bool {
    let path = plan_dir.join("plan.json");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return false;
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    val.get("approved_at")
        .and_then(|v| v.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false)
}

pub fn approve_plan(plan_dir: &Path) -> anyhow::Result<()> {
    let path = plan_dir.join("plan.json");
    let raw = std::fs::read_to_string(&path)?;
    let mut val: serde_json::Value = serde_json::from_str(&raw)?;
    let obj = val
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("plan.json root must be object"))?;
    obj.insert(
        "approved_at".to_string(),
        serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
    );
    std::fs::write(&path, serde_json::to_string_pretty(&val)?)?;
    Ok(())
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
    fn verify_rejects_overlapping_workstream_paths() {
        let output = temp_plan_output("ownership");
        std::fs::write(&output.plan_md, "word ".repeat(850)).unwrap();
        std::fs::write(
            &output.plan_json,
            r#"{"workstreams":[
              {"id":"WS1","finding_ids":["F1"],"target_paths":["gzmo-core/src/foo.rs"],"acceptance":["test -f /tmp/x","test -f /tmp/y"]},
              {"id":"WS2","finding_ids":["F2"],"target_paths":["gzmo-core/src/foo.rs"],"acceptance":["test -f /tmp/z","test -f /tmp/w"]}
            ],"deferred":[]}"#,
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
        assert!(v.notes.contains("ownership conflict"));
    }

    #[test]
    fn approve_plan_sets_timestamp() {
        let output = temp_plan_output("approve");
        std::fs::write(&output.plan_json, r#"{"workstreams":[]}"#).unwrap();
        assert!(!is_plan_approved(&output.plan_dir));
        approve_plan(&output.plan_dir).unwrap();
        assert!(is_plan_approved(&output.plan_dir));
        let _ = std::fs::remove_dir_all(&output.plan_dir);
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
    fn verify_rejects_gitignored_data_target() {
        let output = temp_plan_output("gitignored-data");
        std::fs::write(&output.plan_md, "word ".repeat(850)).unwrap();
        std::fs::write(
            &output.plan_json,
            r#"{"workstreams":[{"id":"WS1","finding_ids":["F1"],"complexity":"moderate","target_paths":["gzmo_skills/data/marker.md"],"acceptance":["test -f /tmp/x","test -f /tmp/y"]}],"deferred":[]}"#,
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
        assert!(v.notes.contains("gitignored data"));
    }

    #[test]
    fn verify_rejects_chicken_egg_sidecar_spawn() {
        let output = temp_plan_output("chicken-egg");
        std::fs::write(&output.plan_md, "word ".repeat(850)).unwrap();
        std::fs::write(
            &output.plan_json,
            r#"{"workstreams":[{"id":"WS1","finding_ids":["F1"],"complexity":"moderate","target_paths":["gzmo_skills/scripts/discovery-remediations/s1/network-audit.sh"],"spawn_command":"bash $GZMO_SKILLS_ROOT/scripts/discovery-remediations/s1/network-audit.sh","acceptance":["test -f /tmp/x","test -f /tmp/y"]}],"deferred":[]}"#,
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
        assert!(v.notes.contains("chicken-egg"));
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

    #[test]
    fn verify_rejects_invalid_spawn_command() {
        let output = temp_plan_output("spawn");
        std::fs::write(&output.plan_md, "word ".repeat(850)).unwrap();
        std::fs::write(
            &output.plan_json,
            r#"{"workstreams":[{"id":"WS1","finding_ids":["F1"],"complexity":"moderate","spawn_command":"grep -q foo /tmp/x","acceptance":["test -f /tmp/x","test -f /tmp/y"]}],"deferred":[]}"#,
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
        assert!(v.notes.contains("spawn_command"));
    }
}
