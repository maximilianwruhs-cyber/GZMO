use crate::types::EvidenceSpan;

/// Localizes a verifier quote within the original document body.
/// Expands the matched span to include a sentence window of ±1 sentence.
pub fn localize_evidence(body: &str, verifier_quote: &str) -> EvidenceSpan {
    let verifier_quote = verifier_quote.trim();
    if verifier_quote.is_empty() {
        return EvidenceSpan {
            evidence_text: String::new(),
            quote_verifier: String::new(),
            char_start: None,
            char_end: None,
        };
    }

    let (norm_body, body_map) = normalize_with_map(body);
    let norm_quote = normalize_only(verifier_quote);

    let char_range = if let Some(byte_start) = norm_body.find(&norm_quote) {
        let _byte_end = byte_start + norm_quote.len();
        let char_start = norm_body[..byte_start].chars().count();
        let char_end = char_start + norm_quote.chars().count();
        Some((char_start, char_end))
    } else {
        // Try fuzzy match using Longest Common Substring (LCS) >= 12 chars
        let body_chars: Vec<char> = norm_body.chars().collect();
        let quote_chars: Vec<char> = norm_quote.chars().collect();
        longest_common_substring_chars(&body_chars, &quote_chars)
    };

    if let Some((start_char, end_char)) = char_range {
        if start_char < body_map.len() && end_char <= body_map.len() && start_char < end_char {
            let match_start = body_map[start_char];
            let match_end = if end_char < body_map.len() {
                body_map[end_char]
            } else {
                body.len()
            };

            // Segment body into sentences to construct sentence window
            let sentences = segment_sentences(body);
            let mut min_idx = None;
            let mut max_idx = None;

            for (idx, &(s_start, s_end)) in sentences.iter().enumerate() {
                if s_start < match_end && s_end > match_start {
                    if min_idx.is_none() {
                        min_idx = Some(idx);
                    }
                    max_idx = Some(idx);
                }
            }

            if let (Some(min_s), Some(max_s)) = (min_idx, max_idx) {
                let low_idx = min_s.saturating_sub(1);
                let high_idx = (max_s + 1).min(sentences.len() - 1);
                let window_start = sentences[low_idx].0;
                let window_end = sentences[high_idx].1;

                let evidence_text = body[window_start..window_end].trim().to_string();
                return EvidenceSpan {
                    evidence_text,
                    quote_verifier: verifier_quote.to_string(),
                    char_start: Some(window_start),
                    char_end: Some(window_end),
                };
            }
        }
    }

    // Fallback: store verifier quote verbatim
    EvidenceSpan {
        evidence_text: verifier_quote.to_string(),
        quote_verifier: verifier_quote.to_string(),
        char_start: None,
        char_end: None,
    }
}

/// Helper to normalize a string for matching (lowercase and collapse whitespace).
fn normalize_only(s: &str) -> String {
    let mut norm = String::new();
    let mut in_space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !in_space {
                norm.push(' ');
                in_space = true;
            }
        } else {
            for lc_c in c.to_lowercase() {
                norm.push(lc_c);
            }
            in_space = false;
        }
    }
    norm.trim().to_string()
}

/// Normalizes a string and returns the mapping of each normalized char index back to its original byte offset.
fn normalize_with_map(s: &str) -> (String, Vec<usize>) {
    let mut norm = String::new();
    let mut map = Vec::new();
    let mut in_space = false;

    for (orig_idx, c) in s.char_indices() {
        if c.is_whitespace() {
            if !in_space {
                norm.push(' ');
                map.push(orig_idx);
                in_space = true;
            }
        } else {
            for lc_c in c.to_lowercase() {
                norm.push(lc_c);
                map.push(orig_idx);
            }
            in_space = false;
        }
    }
    (norm, map)
}

/// Finds the longest common substring between body and quote characters, requiring length >= 12.
fn longest_common_substring_chars(body_chars: &[char], quote_chars: &[char]) -> Option<(usize, usize)> {
    let m = body_chars.len();
    let n = quote_chars.len();
    if m == 0 || n == 0 {
        return None;
    }
    let mut dp = vec![vec![0; n + 1]; 2];
    let mut max_len = 0;
    let mut end_body_idx = 0;

    for i in 1..=m {
        let curr = i % 2;
        let prev = (i - 1) % 2;
        for j in 1..=n {
            if body_chars[i - 1] == quote_chars[j - 1] {
                dp[curr][j] = dp[prev][j - 1] + 1;
                if dp[curr][j] > max_len {
                    max_len = dp[curr][j];
                    end_body_idx = i;
                }
            } else {
                dp[curr][j] = 0;
            }
        }
    }

    if max_len >= 12 {
        Some((end_body_idx - max_len, end_body_idx))
    } else {
        None
    }
}

/// Segments text into sentence spans (start_byte_idx, end_byte_idx).
fn segment_sentences(body: &str) -> Vec<(usize, usize)> {
    let mut sentences = Vec::new();
    let mut start = 0;
    let chars: Vec<(usize, char)> = body.char_indices().collect();
    let n = chars.len();

    let mut i = 0;
    while i < n {
        let (_, c) = chars[i];

        // Check for paragraph boundary \n\n
        if c == '\n' && i + 1 < n && chars[i + 1].1 == '\n' {
            let end_idx = chars[i + 1].0 + 1;
            if start < end_idx {
                sentences.push((start, end_idx));
            }
            start = end_idx;
            i += 2;
            continue;
        }

        // Check for sentence end followed by whitespace/newline
        if c == '.' || c == '!' || c == '?' {
            let mut is_end = false;
            if i + 1 == n {
                is_end = true;
            } else {
                let next_char = chars[i + 1].1;
                if next_char.is_whitespace() {
                    is_end = true;
                }
            }

            if is_end {
                // Consume any trailing whitespace/punctuation for this sentence
                while i + 1 < n && chars[i + 1].1.is_whitespace() {
                    i += 1;
                }
                let next_byte_idx = if i + 1 < n { chars[i + 1].0 } else { body.len() };
                sentences.push((start, next_byte_idx));
                start = next_byte_idx;
            }
        }

        i += 1;
    }

    if start < body.len() {
        sentences.push((start, body.len()));
    }

    sentences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_sentences() {
        let text = "Hallo Welt! Wie geht es? Gut.";
        let segs = segment_sentences(text);
        assert_eq!(segs.len(), 3);
        assert_eq!(&text[segs[0].0..segs[0].1], "Hallo Welt! ");
        assert_eq!(&text[segs[1].0..segs[1].1], "Wie geht es? ");
        assert_eq!(&text[segs[2].0..segs[2].1], "Gut.");
    }

    #[test]
    fn test_localize_exact() {
        let body = "Das ist ein Satz. Der Architectural Scout fokussiert auf die grundlegende Struktur des Rechenzentrums. Ein dritter Satz.";
        let quote = "Architectural Scout fokussiert auf die grundlegende Struktur";
        let span = localize_evidence(body, quote);
        assert!(span.char_start.is_some());
        assert_eq!(span.evidence_text, "Das ist ein Satz. Der Architectural Scout fokussiert auf die grundlegende Struktur des Rechenzentrums. Ein dritter Satz.");
    }

    #[test]
    fn test_localize_whitespace_variant() {
        let body = "Der  Scout  fokussiert  auf  die  Struktur.";
        let quote = "Scout fokussiert auf die";
        let span = localize_evidence(body, quote);
        assert!(span.char_start.is_some());
        assert_eq!(span.evidence_text, "Der  Scout  fokussiert  auf  die  Struktur.");
    }

    #[test]
    fn test_localize_fallback() {
        let body = "Ein Satz ohne Relevanz.";
        let quote = "Nicht vorhanden";
        let span = localize_evidence(body, quote);
        assert!(span.char_start.is_none());
        assert_eq!(span.evidence_text, "Nicht vorhanden");
    }
}
