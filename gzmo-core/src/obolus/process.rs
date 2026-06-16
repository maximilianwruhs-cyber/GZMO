//! Process taxonomy for Obolus ledger attribution.

use crate::config::TaskKind;

/// Map a routing `TaskKind` to a stable ledger process label.
pub fn process_from_task_kind(task: TaskKind) -> String {
    task.to_string()
}

/// Kurator sub-agent spawn kinds (see `spawn_gate::SpawnKind`).
pub fn kurator_process_label(spawn_kind: &str) -> String {
    format!("kurator_{spawn_kind}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_task_kinds_have_labels() {
        for &k in TaskKind::all() {
            let label = process_from_task_kind(k);
            assert!(!label.is_empty());
        }
    }

    #[test]
    fn kurator_labels() {
        assert_eq!(kurator_process_label("discovery_fix"), "kurator_discovery_fix");
        assert_eq!(
            kurator_process_label("session_triage"),
            "kurator_session_triage"
        );
    }
}
