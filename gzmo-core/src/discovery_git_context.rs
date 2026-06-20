//! Optional git context enrichment for discovery agent briefs (Jules jules-action pattern).

use std::path::Path;
use std::process::Command;

/// When `DISCOVERY_INCLUDE_GIT_CONTEXT=1`, append recent commit diff and log to brief sections.
pub fn git_context_enabled() -> bool {
    std::env::var("DISCOVERY_INCLUDE_GIT_CONTEXT")
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

/// Collect git show + git log --stat from `repo_root` (best-effort; empty on failure).
pub fn collect_git_context(repo_root: &Path) -> String {
    if !git_context_enabled() {
        return String::new();
    }
    let mut sections = Vec::new();

    if let Ok(out) = Command::new("git")
        .args(["show", "--stat", "--format=medium"])
        .current_dir(repo_root)
        .output()
    {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            if !text.trim().is_empty() {
                sections.push(format!(
                    "Latest commit (git show):\n```\n{}\n```",
                    text.trim()
                ));
            }
        }
    }

    if let Ok(out) = Command::new("git")
        .args(["log", "-20", "--stat", "--oneline"])
        .current_dir(repo_root)
        .output()
    {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            if !text.trim().is_empty() {
                sections.push(format!(
                    "Recent commits (git log -20 --stat):\n```\n{}\n```",
                    text.trim()
                ));
            }
        }
    }

    if sections.is_empty() {
        return String::new();
    }
    format!("\n\nGit context:\n{}", sections.join("\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_by_default() {
        std::env::remove_var("DISCOVERY_INCLUDE_GIT_CONTEXT");
        assert!(!git_context_enabled());
        assert!(collect_git_context(Path::new(".")).is_empty());
    }
}
