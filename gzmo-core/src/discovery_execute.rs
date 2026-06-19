//! Discovery execute agent — one workstream per spawn from approved plan.json.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::discovery_fixer;

pub fn discovery_execute_session_id(plan_id: &str, workstream_id: &str) -> String {
    format!("discovery-fix:{plan_id}:{workstream_id}")
}

pub fn is_discovery_execute_recommendation(
    rec: &crate::kurator_monitor::PendingRecommendation,
) -> bool {
    rec.kind.as_deref() == Some("discovery_execute")
        || rec.reason.starts_with("discovery_execute:")
        || rec.session_id.starts_with("discovery-fix:")
}

pub fn resolve_plan_json_path(plan_path: &Path) -> PathBuf {
    if plan_path.is_dir() {
        plan_path.join("plan.json")
    } else {
        plan_path.to_path_buf()
    }
}

pub fn load_workstream(plan_path: &Path, workstream_id: &str) -> anyhow::Result<Value> {
    let json_path = resolve_plan_json_path(plan_path);
    let raw = std::fs::read_to_string(&json_path)?;
    let doc: Value = serde_json::from_str(&raw)?;
    doc.get("workstreams")
        .and_then(|a| a.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|ws| ws.get("id").and_then(|v| v.as_str()) == Some(workstream_id))
                .cloned()
        })
        .ok_or_else(|| anyhow::anyhow!("workstream {workstream_id} not found in plan"))
}

pub fn discovery_execute_reason(workstream_id: &str) -> String {
    format!("discovery_execute: workstream {workstream_id}")
}

pub fn build_execute_brief(
    plan_dir: &Path,
    workstream_id: &str,
    workstream: &Value,
    git_baseline_tag: &str,
    max_chars: usize,
) -> String {
    let plan_md = plan_dir.join("plan.md");
    let probe_dir = crate::discovery_code_implementer::resolve_probe_results_dir();
    let ws_json = serde_json::to_string_pretty(workstream).unwrap_or_else(|_| "{}".into());
    let probe_script = workstream
        .get("probe_script")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let mut lines = vec![
        format!("Discovery fixer — execute workstream `{workstream_id}` only."),
        format!("Plan dir (file_read): {}", plan_dir.display()),
        format!("Plan markdown (file_read): {}", plan_md.display()),
        format!("Probe results dir (file_read): {}", probe_dir.display()),
        format!("Git baseline tag: {git_baseline_tag}"),
        String::new(),
        "Workstream JSON:".to_string(),
        ws_json,
        String::new(),
    ];
    if !probe_script.is_empty() {
        lines.push(format!("Probe script hint: {probe_script}"));
    }
    lines.push("Task:".to_string());
    lines.push("1. file_read plan.md section for this workstream and relevant probe JSON.".to_string());
    lines.push("2. Implement ONLY this workstream — file_write under survey_GZMO/ or gzmo_skills/.".to_string());
    lines.push("3. Run each acceptance[] command via shell_exec before finishing.".to_string());
    lines.push("4. Summary lists only paths you actually wrote.".to_string());
    lines.push(String::new());
    lines.push("Scope: survey_GZMO/ and gzmo_skills/ only.".to_string());

    crate::text_util::truncate_chars(&lines.join("\n"), max_chars)
}

pub fn verify_execute_outcome(
    summary: &str,
    hit_max_iterations: bool,
    roots: &[PathBuf],
    written_paths: &[String],
) -> discovery_fixer::DiscoveryFixVerification {
    crate::discovery_code_implementer::verify_code_implement_outcome(
        summary,
        hit_max_iterations,
        roots,
        written_paths,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_workstream_from_plan() {
        let dir = std::env::temp_dir().join(format!("plan-exec-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let plan = dir.join("plan.json");
        let mut f = std::fs::File::create(&plan).unwrap();
        write!(
            f,
            r#"{{"workstreams":[{{"id":"W1","title":"t","finding_ids":["F1"],"acceptance":["bash -n x"],"complexity":"moderate"}}]}}"#
        )
        .unwrap();
        let ws = load_workstream(&dir, "W1").unwrap();
        assert_eq!(ws["id"], "W1");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
