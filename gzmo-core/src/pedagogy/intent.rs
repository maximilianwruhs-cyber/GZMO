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

/// Prefixes that indicate an ops intent. These must be specific enough to avoid
/// false positives on creative requests like "write a poem" or "run a bath".
/// Each prefix requires a shell/file/code artifact as the next token, not just
/// any word.
const OPS_COMMAND_PREFIXES: &[&str] = &[
    "execute ",
    "fix the ",     // require determiner — "fix the bug" not "fix things"
    "fix this ",
    "fix that ",
    "deploy ",
    "install ",
    "delete ",
    "create file",
    "write file",
    "write script",
    "write the script",
    "write a script",
    "write a function",
    "write a program",
    "write a command",
    "write the command",
    "rm ",
    "git ",
    "docker ",
    "systemctl ",
    "apt ",
    "pip ",
    "npm ",
    "cargo ",
];

/// Check if input looks like a shell command or code request (ops) vs creative
/// writing (poem, story, song — should go to teach/generative skills).
fn looks_like_ops_command(lower: &str) -> bool {
    // Shell command prefixes are unambiguous
    if OPS_COMMAND_PREFIXES.iter().any(|p| lower.starts_with(p)) {
        return true;
    }
    // "write" followed by a file/script/function/program — ops
    // "write" followed by a poem/story/song/letter — not ops
    if lower.starts_with("write ") {
        let after_write = &lower[6..];
        // Skip leading articles to find the actual subject
        let after_article = if after_write.starts_with("a ") {
            &after_write[2..]
        } else if after_write.starts_with("an ") {
            &after_write[3..]
        } else if after_write.starts_with("the ") {
            &after_write[4..]
        } else {
            after_write
        };
        let creative_words = ["poem", "story", "song", "letter", "haiku", "novel", "essay", "journal", "blog", "poetry"];
        let first_word = after_article.split_whitespace().next().unwrap_or("");
        if creative_words.iter().any(|cw| first_word.starts_with(cw)) {
            return false;  // creative writing — not ops
        }
        // "write <something>" that's not creative → likely code/file ops
        // But require minimum length to avoid matching "write me"
        if after_write.len() > 4 && !after_write.starts_with("me ") && !after_write.starts_with("us ") {
            return true;
        }
        return false;
    }
    // "fix" without determiner — check for code/bug context
    if lower.starts_with("fix ") {
        let after_fix = &lower[4..];
        // "fix the bug", "fix this error" — ops
        // "fix dinner", "fix a drink" — not ops
        let non_ops_words = ["dinner", "lunch", "breakfast", "a drink", "a sandwich", "a snack", "a meal", "the problem"];
        if non_ops_words.iter().any(|nw| after_fix.starts_with(nw)) {
            return false;
        }
        return true;
    }
    // "run" without shell context — check for code/command context
    if lower.starts_with("run ") {
        let after_run = &lower[4..];
        // "run a bath", "run for it", "run away" — not ops
        let non_ops_words = ["a bath", "for it", "away", "a marathon", "a mile", "some errands", "a test"];
        if non_ops_words.iter().any(|nw| after_run.starts_with(nw)) {
            // "run a test" is ambiguous — could be ops. Let ops phrases handle it.
            return false;
        }
        return true;
    }
    false
}

/// Strong learn signals that override ops_mode and route back to Teach.
/// These are explicit teaching/learning requests that the user clearly wants
/// answered pedagogically even while in ops mode.
const ESCAPE_HATCH_PHRASES: &[&str] = &[
    "teach me",
    "help me understand",
    "explain how",
    "why does",
    "how do i",
    "walk me through",
    "show me how",
    "i want to learn",
    "let's learn",
    "erkläre mir",
    "beibringen",
    "wie funktioniert",
];

pub fn classify_intent(
    input: &str,
    ops_mode: bool,
    learn_prep_active: bool,
    learn_prep_ready: bool,
) -> InteractionIntent {
    let lower = input.to_lowercase();
    let trimmed = input.trim();

    // Explicit slash commands always win
    if trimmed.starts_with("/learn ") || trimmed == "/learn" {
        return InteractionIntent::LearnPrep;
    }
    if trimmed == "/ops" || trimmed.starts_with("/ops ") {
        return InteractionIntent::Ops;
    }

    // Escape hatch: strong learn phrases override ops_mode
    if ops_mode {
        if ESCAPE_HATCH_PHRASES.iter().any(|p| lower.contains(p)) {
            return InteractionIntent::Teach;
        }
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

    if looks_like_ops_command(&lower) {
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

    #[test]
    fn write_a_poem_is_not_ops() {
        assert_eq!(
            classify_intent("write a poem about the sea", false, false, false),
            InteractionIntent::Teach
        );
    }

    #[test]
    fn write_a_story_is_not_ops() {
        assert_eq!(
            classify_intent("write a story about a robot", false, false, false),
            InteractionIntent::Teach
        );
    }

    #[test]
    fn write_a_script_is_ops() {
        assert_eq!(
            classify_intent("write a script to backup my home directory", false, false, false),
            InteractionIntent::Ops
        );
    }

    #[test]
    fn write_the_command_is_ops() {
        assert_eq!(
            classify_intent("write the command to restart nginx", false, false, false),
            InteractionIntent::Ops
        );
    }

    #[test]
    fn write_file_is_ops() {
        assert_eq!(
            classify_intent("write file /etc/nginx/nginx.conf", false, false, false),
            InteractionIntent::Ops
        );
    }

    #[test]
    fn run_a_bath_is_not_ops() {
        assert_eq!(
            classify_intent("run a bath for me", false, false, false),
            InteractionIntent::Teach
        );
    }

    #[test]
    fn run_nginx_is_ops() {
        assert_eq!(
            classify_intent("run nginx in a docker container", false, false, false),
            InteractionIntent::Ops
        );
    }

    #[test]
    fn fix_dinner_is_not_ops() {
        assert_eq!(
            classify_intent("fix dinner for tonight", false, false, false),
            InteractionIntent::Teach
        );
    }

    #[test]
    fn fix_the_bug_is_ops() {
        assert_eq!(
            classify_intent("fix the bug in the auth module", false, false, false),
            InteractionIntent::Ops
        );
    }

    #[test]
    fn git_command_is_ops() {
        assert_eq!(
            classify_intent("git push origin main", false, false, false),
            InteractionIntent::Ops
        );
    }

    #[test]
    fn docker_command_is_ops() {
        assert_eq!(
            classify_intent("docker ps -a", false, false, false),
            InteractionIntent::Ops
        );
    }

    #[test]
    fn write_me_is_not_ops() {
        assert_eq!(
            classify_intent("write me a letter", false, false, false),
            InteractionIntent::Teach
        );
    }

    // --- Escape hatch tests ---

    #[test]
    fn escape_hatch_teach_me_overrides_ops_mode() {
        assert_eq!(
            classify_intent("teach me how systemd works", true, false, false),
            InteractionIntent::Teach
        );
    }

    #[test]
    fn escape_hatch_explain_how_overrides_ops_mode() {
        assert_eq!(
            classify_intent("explain how iptables chains work", true, false, false),
            InteractionIntent::Teach
        );
    }

    #[test]
    fn escape_hatch_walk_me_through_overrides_ops_mode() {
        assert_eq!(
            classify_intent("walk me through setting up a firewall", true, false, false),
            InteractionIntent::Teach
        );
    }

    #[test]
    fn escape_hatch_german_erklare_overrides_ops_mode() {
        assert_eq!(
            classify_intent("erkläre mir wie systemd funktioniert", true, false, false),
            InteractionIntent::Teach
        );
    }

    #[test]
    fn ops_mode_still_routes_implicit_questions_to_ops() {
        // A bare question without escape-hatch phrase → stays in ops
        assert_eq!(
            classify_intent("what is systemd?", true, false, false),
            InteractionIntent::Ops
        );
    }

    #[test]
    fn ops_mode_still_routes_commands_to_ops() {
        assert_eq!(
            classify_intent("run nginx", true, false, false),
            InteractionIntent::Ops
        );
    }

    #[test]
    fn slash_learn_overrides_ops_mode() {
        assert_eq!(
            classify_intent("/learn networking", true, false, false),
            InteractionIntent::LearnPrep
        );
    }
}
