//! Small string helpers (UTF-8 safe truncation).

/// Truncate to at most `max_chars` Unicode scalars; append `...` when shortened.
pub fn truncate_chars(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }
    let head: String = s.chars().take(max_chars).collect();
    format!("{head}...")
}
