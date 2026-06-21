//! Hypervisor-side external verification of workstream acceptance[] commands and action probe scripts.

use std::path::{Path, PathBuf};

/// Run execute workstream acceptance[] commands.
pub fn run_execute_acceptance(
    plan_dir: &Path,
    workstream_id: &str,
    project_root: &Path,
    skills_root: &Path,
    log_id: Option<&str>,
) -> Vec<String> {
    let workstream = match crate::discovery_execute::load_workstream(plan_dir, workstream_id) {
        Ok(ws) => ws,
        Err(e) => return vec![format!("Failed to load workstream {workstream_id}: {e}")],
    };

    let mut failed = Vec::new();
    if let Some(arr) = workstream.get("acceptance").and_then(|a| a.as_array()) {
        for cmd_val in arr {
            if let Some(cmd_str) = cmd_val.as_str() {
                let mut cmd = std::process::Command::new("bash");
                cmd.arg("-c").arg(cmd_str);
                cmd.current_dir(project_root);
                cmd.env("GZMO_ROOT", project_root);
                cmd.env("GZMO_SKILLS_ROOT", skills_root);

                match cmd.output() {
                    Ok(output) => {
                        let status = output.status;
                        if !status.success() {
                            let mut msg = format!("acceptance command `{cmd_str}` exited non-zero: {status}");
                            if let Some(lid) = log_id {
                                let stdout_str = String::from_utf8_lossy(&output.stdout);
                                let stderr_str = String::from_utf8_lossy(&output.stderr);
                                let log_dir = skills_root.join("data/gate-outputs");
                                let log_path = log_dir.join(format!("{lid}.log"));
                                if std::fs::create_dir_all(&log_dir).is_ok() {
                                    let log_content = format!(
                                        "--- Command: {cmd_str} ---\nExit Status: {status}\n\nStdout:\n{stdout_str}\n\nStderr:\n{stderr_str}\n"
                                    );
                                    let _ = std::fs::write(&log_path, log_content);
                                    msg.push_str(&format!(" [Logs truncated; full log saved to file: file://{}]", log_path.display()));
                                }
                            }
                            failed.push(msg);
                        }
                    }
                    Err(e) => {
                        failed.push(format!("acceptance command `{cmd_str}` failed to start: {e}"));
                    }
                }
            }
        }
    }
    failed
}

/// Run acceptance[] commands for a specific finding in the code implementer flow.
pub fn run_code_implement_acceptance(
    plan_json_path: &Path,
    finding_id: &str,
    project_root: &Path,
    skills_root: &Path,
    log_id: Option<&str>,
) -> Vec<String> {
    let mut failed = Vec::new();
    if !plan_json_path.is_file() {
        return failed;
    }
    let raw = match std::fs::read_to_string(plan_json_path) {
        Ok(s) => s,
        Err(e) => return vec![format!("Failed to read plan.json: {e}")],
    };
    let plan_val: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => return vec![format!("Failed to parse plan.json: {e}")],
    };

    if let Some(workstreams) = plan_val.get("workstreams").and_then(|a| a.as_array()) {
        for ws in workstreams {
            let has_finding = ws.get("finding_ids")
                .and_then(|a| a.as_array())
                .map(|arr| arr.iter().any(|id_val| id_val.as_str() == Some(finding_id)))
                .unwrap_or(false);
            if has_finding {
                if let Some(acceptance_cmds) = ws.get("acceptance").and_then(|a| a.as_array()) {
                    for cmd_val in acceptance_cmds {
                        if let Some(cmd_str) = cmd_val.as_str() {
                            let mut cmd = std::process::Command::new("bash");
                            cmd.arg("-c").arg(cmd_str);
                            cmd.current_dir(project_root);
                            cmd.env("GZMO_ROOT", project_root);
                            cmd.env("GZMO_SKILLS_ROOT", skills_root);

                            match cmd.output() {
                                Ok(output) => {
                                    let status = output.status;
                                    if !status.success() {
                                        let mut msg = format!("workstream acceptance `{cmd_str}` exited non-zero: {status}");
                                        if let Some(lid) = log_id {
                                            let stdout_str = String::from_utf8_lossy(&output.stdout);
                                            let stderr_str = String::from_utf8_lossy(&output.stderr);
                                            let log_dir = skills_root.join("data/gate-outputs");
                                            let log_path = log_dir.join(format!("{lid}.log"));
                                            if std::fs::create_dir_all(&log_dir).is_ok() {
                                                let log_content = format!(
                                                    "--- Command: {cmd_str} ---\nExit Status: {status}\n\nStdout:\n{stdout_str}\n\nStderr:\n{stderr_str}\n"
                                                );
                                                let _ = std::fs::write(&log_path, log_content);
                                                msg.push_str(&format!(" [Logs truncated; full log saved to file: file://{}]", log_path.display()));
                                            }
                                        }
                                        failed.push(msg);
                                    }
                                }
                                Err(e) => {
                                    failed.push(format!("workstream acceptance `{cmd_str}` failed to start: {e}"));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    failed
}

/// Run mapped probe script for a fixer finding if its kind is ACTION.
pub fn run_fixer_probe(
    finding: &crate::discovery_fixer::ActionableFinding,
    project_root: &Path,
    skills_root: &Path,
    log_id: Option<&str>,
) -> Vec<String> {
    let mut failed = Vec::new();
    let action_text = format!("{} {}", finding.title, finding.excerpt);
    let script_name = probe_script_for_action(&action_text);
    let probe_path = skills_root.join("scripts/discovery-probes").join(&script_name);

    if !probe_path.is_file() {
        return vec![format!("Probe script {script_name} not found at {}", probe_path.display())];
    }

    let mut cmd = std::process::Command::new(&probe_path);
    if script_name == "probe-generic-discovery-action.sh" {
        cmd.arg(&finding.excerpt);
    }
    cmd.current_dir(skills_root);
    cmd.env("GZMO_ROOT", project_root);
    cmd.env("GZMO_SKILLS_ROOT", skills_root);

    match cmd.output() {
        Ok(output) => {
            let status = output.status;
            if !status.success() {
                let mut msg = format!("Probe script {script_name} exited non-zero: {status}");
                if let Some(lid) = log_id {
                    let stdout_str = String::from_utf8_lossy(&output.stdout);
                    let stderr_str = String::from_utf8_lossy(&output.stderr);
                    let log_dir = skills_root.join("data/gate-outputs");
                    let log_path = log_dir.join(format!("{lid}.log"));
                    if std::fs::create_dir_all(&log_dir).is_ok() {
                        let log_content = format!(
                            "--- Probe Script: {script_name} ---\nExit Status: {status}\n\nStdout:\n{stdout_str}\n\nStderr:\n{stderr_str}\n"
                        );
                        let _ = std::fs::write(&log_path, log_content);
                        msg.push_str(&format!(" [Logs truncated; full log saved to file: file://{}]", log_path.display()));
                    }
                }
                failed.push(msg);
            }
        }
        Err(e) => {
            failed.push(format!("Probe script {script_name} failed to start: {e}"));
        }
    }
    failed
}

/// Match finding text to standard probe scripts in GZMO_SKILLS_ROOT.
pub fn probe_script_for_action(action_text: &str) -> String {
    let lower = action_text.to_lowercase();
    if lower.contains("probe b09") || (lower.contains("chaosevent") && lower.contains("emission")) {
        "probe-chaos-event-trace.sh".to_string()
    } else if lower.contains("probe b10") || (lower.contains("e_total") && lower.contains("skill")) {
        "probe-obolus-budget.sh".to_string()
    } else if lower.contains("probe b11") || lower.contains("librarian_summary") {
        "probe-librarian-fallback.sh".to_string()
    } else if lower.contains("probe b12") || (lower.contains("qdrant") && lower.contains("partition")) || (lower.contains("vault sync") && lower.contains("retry")) {
        "probe-qdrant-partition.sh".to_string()
    } else if lower.contains("probe b13") || lower.contains("max_session_anchor_age") {
        "probe-session-anchor-retention.sh".to_string()
    } else if lower.contains("probe a02") {
        "probe-a02-spark-distill.sh".to_string()
    } else if lower.contains("obolus") || lower.contains("denied/promoted") || lower.contains("budget-starv") {
        "probe-obolus-budget.sh".to_string()
    } else if lower.contains("episodic backlog") || (lower.contains("undistilled") && lower.contains("session")) {
        "probe-episodic-backlog.sh".to_string()
    } else if lower.contains("curation deficit") || (lower.contains("honeypot") && lower.contains("neo4j")) {
        "probe-curation-deficit.sh".to_string()
    } else if lower.contains("dice") && lower.contains("17") {
        "probe-dice-coverage.sh".to_string()
    } else if lower.contains("distill queue") || lower.contains("cold chain") {
        "probe-distill-queue.sh".to_string()
    } else if lower.contains("probe a03") {
        "probe-a03-neo4j-orphans.sh".to_string()
    } else if lower.contains("probe a04") {
        "probe-a04-rho-correlation.sh".to_string()
    } else if lower.contains("probe a05") {
        "probe-a05-wiki-entities.sh".to_string()
    } else if lower.contains("vault_truths") && lower.contains("monitor") {
        "monitor-vault-truths-drift.sh".to_string()
    } else if lower.contains("friction") || lower.contains("crystallize") {
        "probe-friction-model.sh".to_string()
    } else if lower.contains("seed") && lower.contains("pulseloop") || lower.contains("seed stability") {
        "probe-pulseloop-seed-stability.sh".to_string()
    } else if lower.contains("rust skill registry") || lower.contains("registry audit") {
        "probe-rust-registry-audit.sh".to_string()
    } else if lower.contains("pulseloop") || lower.contains("auto-lore") || lower.contains("auto_lore") {
        "probe-pulseloop-trigger.sh".to_string()
    } else if lower.contains("memory_index") || lower.contains("vault.db") {
        "probe-memory-index.sh".to_string()
    } else if lower.contains("chaossnapshot") || lower.contains("snapshot_rx") {
        "probe-chaos-snapshot.sh".to_string()
    } else if lower.contains("rho_breath") {
        "probe-rho-phase-zero.sh".to_string()
    } else if lower.contains("wiki entity") {
        "probe-a05-wiki-entities.sh".to_string()
    } else if lower.contains("spark_complete") && lower.contains("distill_complete") {
        "probe-a02-spark-distill.sh".to_string()
    } else if lower.contains("orphan") && lower.contains("neo4j") {
        "probe-a03-neo4j-orphans.sh".to_string()
    } else if lower.contains("pillar b") || lower.contains("next session") {
        "probe-pillar-handoff.sh".to_string()
    } else if lower.contains("after-boot") || lower.contains("after_boot") {
        "probe-after-boot-verify.sh".to_string()
    } else if lower.contains("systemd") || lower.contains("list-units") {
        "probe-systemd-units.sh".to_string()
    } else if lower.contains("neo4j") {
        "probe-neo4j-query.sh".to_string()
    } else if lower.contains("qdrant") {
        "probe-qdrant.sh".to_string()
    } else if lower.contains("mcp") && lower.contains("latenc") {
        "probe-mcp-memory-latency.sh".to_string()
    } else if lower.contains("lxc101") || lower.contains("autostart") {
        "probe-lxc101-autostart.sh".to_string()
    } else if lower.contains("vm200") || lower.contains("warmup") || lower.contains("embeddings") {
        "probe-vm200-warmup.sh".to_string()
    } else {
        "probe-generic-discovery-action.sh".to_string()
    }
}
