//! Intent classification — mentor vs ops vs learn prep.

/// Classified user intent for routing chat turns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionIntent {
    /// Default — run pedagogy orchestrator (Socratic mentor).
    Teach,
    /// Explicit execution — skip orchestrator, use tool agent loop.
    Ops,
    /// Flipped classroom topic prep (`/learn <topic>`).
    LearnPrep,
    /// Continue Socratic session after learn prep completed.
    LearnSync,
}

const OPS_PHRASES: &[&str] = &[
    "just run it",
    "execute now",
    "run this",
    "do it for me",
    "fix it now",
    "implement it",
    "write the script",
    "führe aus",
    "mach das",
    "einfach ausführen",
];

const LEARN_PHRASES: &[&str] = &[
    "teach me",
    "help me understand",
    "explain how",
    "why does",
    "what is",
    "how do i",
    "wie funktioniert",
    "erkläre mir",
    "beibringen",
];

const OPS_COMMAND_PREFIXES: &[&str] = &[
    "run ",
    "execute ",
    "fix ",
    "deploy ",
    "install ",
    "delete ",
    "create file",
    "write ",
];

pub fn classify_intent(
    input: &str,
    ops_mode: bool,
    learn_prep_active: bool,
    learn_prep_ready: bool,
) -> InteractionIntent {
    if ops_mode {
        return InteractionIntent::Ops;
    }

    let lower = input.to_lowercase();
    let trimmed = input.trim();

    if trimmed.starts_with("/learn ") || trimmed == "/learn" {
        return InteractionIntent::LearnPrep;
    }
    if trimmed == "/ops" || trimmed.starts_with("/ops ") {
        return InteractionIntent::Ops;
    }

    if learn_prep_ready {
        return InteractionIntent::LearnSync;
    }
    if learn_prep_active {
        return InteractionIntent::LearnPrep;
    }

    if OPS_PHRASES.iter().any(|p| lower.contains(p)) {
        return InteractionIntent::Ops;
    }

    if OPS_COMMAND_PREFIXES.iter().any(|p| lower.starts_with(p)) {
        return InteractionIntent::Ops;
    }

    if LEARN_PHRASES.iter().any(|p| lower.contains(p)) {
        return InteractionIntent::Teach;
    }

    // Mentor-first default: questions and exploratory input → teach.
    if trimmed.ends_with('?') || trimmed.contains("how ") || trimmed.contains("why ") {
        return InteractionIntent::Teach;
    }

    InteractionIntent::Teach
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ops_mode_overrides() {
        assert_eq!(
            classify_intent("what is systemd?", true, false, false),
            InteractionIntent::Ops
        );
    }

    #[test]
    fn just_run_it_is_ops() {
        assert_eq!(
            classify_intent("just run it please", false, false, false),
            InteractionIntent::Ops
        );
    }

    #[test]
    fn question_defaults_teach() {
        assert_eq!(
            classify_intent("what is a symlink?", false, false, false),
            InteractionIntent::Teach
        );
    }
}
