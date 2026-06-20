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
    // Do not match `discovery-fix:` session ids — discovery_fix uses the same prefix
    // (see discovery_fix_session_id). Execute spawns always carry kind + reason.
    rec.kind.as_deref() == Some("discovery_execute")
        || rec.reason.starts_with("discovery_execute:")
}

pub fn resolve_plan_json_path(plan_path: &Path) -> PathBuf {
    if plan_path.is_dir() {
        plan_path.join("plan.json")
    } else {
        plan_path.to_path_buf()
    }
}

pub fn load_workstream(plan_path: &Path, workstream_id: &str) -> anyhow::Result<Value> {
    ensure_plan_executable(plan_path)?;
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

/// Jules requirePlanApproval — block execute until operator approves plan.json.
pub fn ensure_plan_executable(plan_path: &Path) -> anyhow::Result<()> {
    if crate::discovery_plan_agent::plan_approval_required()
        && !crate::discovery_plan_agent::is_plan_approved(plan_path)
    {
        anyhow::bail!(
            "plan not approved: run `gzmo kurator approve-plan --plan {}` first",
            plan_path.display()
        );
    }
    Ok(())
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
    lines.push("2. Implement ONLY this workstream — file_write under $GZMO_ROOT/ or $GZMO_SKILLS_ROOT/.".to_string());
    lines.push("3. Run each acceptance[] command via shell_exec before finishing.".to_string());
    lines.push("4. Summary lists only paths you actually wrote.".to_string());
    lines.push(String::new());
    lines.push("Scope: GZMO project root and gzmo_skills only.".to_string());

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
    fn execute_recommendation_not_confused_with_discovery_fix_session() {
        use crate::kurator_monitor::PendingRecommendation;
        use chrono::Utc;
        let fix_rec = PendingRecommendation {
            event_id: "e1".into(),
            session_id: "discovery-fix:manual:cycle-1-report".into(),
            kind: Some("discovery_fix".into()),
            reason: "discovery_actionable: 1 FAIL, 1 GAP (2 actionable)".into(),
            suggested_agent_profile: "epimetheus".into(),
            created_at: Utc::now(),
            approved: false,
            spawn_task_id: None,
            report_path: None,
        };
        assert!(!is_discovery_execute_recommendation(&fix_rec));

        let exec_rec = PendingRecommendation {
            kind: Some("discovery_execute".into()),
            reason: discovery_execute_reason("W1"),
            session_id: discovery_execute_session_id("plan-a", "W1"),
            ..fix_rec
        };
        assert!(is_discovery_execute_recommendation(&exec_rec));
    }

    #[test]
    fn load_workstream_from_plan() {
        std::env::set_var("DISCOVERY_PLAN_REQUIRE_APPROVAL", "0");
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
        std::env::remove_var("DISCOVERY_PLAN_REQUIRE_APPROVAL");
    }
}
