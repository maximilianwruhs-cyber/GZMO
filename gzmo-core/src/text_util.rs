//! Small string helpers (UTF-8 safe truncation, display cleanup).

/// Truncate to at most `max_chars` Unicode scalars; append `...` when shortened.
pub fn truncate_chars(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }
    let head: String = s.chars().take(max_chars).collect();
    format!("{head}...")
}

const THINKING_CHANNEL_MARKERS: &[&str] = &["<|channel>thought", "<channel>thought"];

/// Strip Gemma/Qwen thinking-channel wrappers from mentor tutor output.
pub fn strip_mentor_channel_noise(text: &str) -> String {
    let mut out = String::new();
    let mut in_redacted = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<think>") {
            in_redacted = true;
            continue;
        }
        if in_redacted {
            if trimmed.starts_with("</think>") {
                in_redacted = false;
            }
            continue;
        }
        if THINKING_CHANNEL_MARKERS
            .iter()
            .any(|marker| trimmed.eq_ignore_ascii_case(marker))
        {
            continue;
        }
        let stripped = trimmed
            .strip_prefix("<|channel|>")
            .or_else(|| trimmed.strip_prefix("<channel|>"))
            .unwrap_or(trimmed)
            .trim();
        if stripped.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(stripped);
    }
    out
}

/// Strip ANSI SGR escape sequences from skill output.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
            continue;
        }
        out.push(ch);
    }
    out
}

/// Skill output for Pi/Telegram: strip ANSI but keep ASCII/box layout (cards, dice frames).
pub fn pi_skill_display(raw: &str) -> String {
    strip_ansi(raw)
}

/// Skill box art → readable plain text for mentor injection (drops box-drawing chars).
pub fn plain_skill_display(raw: &str) -> String {
    strip_ansi(raw)
        .chars()
        .filter(|c| {
            !matches!(
                c,
                '┌' | '┐' | '└' | '┘' | '├' | '┤' | '─' | '│' | '╔' | '╗' | '╚' | '╝' | '║' | '═'
                    | '╠' | '╣'
            )
        })
        .collect::<String>()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_chars_respects_unicode() {
        assert_eq!(truncate_chars("hello", 10), "hello");
        assert_eq!(truncate_chars("hello world", 5), "hello...");
    }

    #[test]
    fn strip_mentor_channel_noise_removes_thought_wrapper() {
        let raw = "<|channel>thought\n<channel|>Hello learner.";
        assert_eq!(strip_mentor_channel_noise(raw), "Hello learner.");
    }

    #[test]
    fn strip_ansi_removes_color_codes() {
        let raw = "\x1b[32mok\x1b[0m";
        assert_eq!(strip_ansi(raw), "ok");
    }
}
