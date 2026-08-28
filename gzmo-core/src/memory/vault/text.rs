//! Vault text normalization.

/// Normalize vault fact text for dedup (lowercase, collapsed whitespace).
pub fn normalize_truth_content(content: &str) -> String {
    content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod normalize_tests {
    use super::*;

    #[test]
    fn normalize_collapses_whitespace_and_lowercases() {
        assert_eq!(
            normalize_truth_content("[SYSTEM:GZMO]  Hello   World"),
            "[system:gzmo] hello world"
        );
    }
}
