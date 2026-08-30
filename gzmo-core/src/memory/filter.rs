//! NO_REPLY fail-closed filter.
//!
//! Intercepts LLM output chunks and routes silent turns (internal monologue)
//! to the episodic log instead of the user interface.

use crate::types::{EpisodicEntry, EpisodicSource};
use chrono::Utc;

use crate::memory::episodic::FileEpisodicStore;

/// Result of filtering a single LLM output chunk.
pub enum FilterResult {
    /// Forward this content to the user interface.
    Forward(String),
    /// This was a silent turn — already routed to episodic log.
    Suppressed,
}

/// Fail-closed: empty/whitespace is never user-forwardable.
fn is_forwardable_chunk(chunk: &str) -> bool {
    !chunk.trim().is_empty()
}

/// Token scan. Empty input is a no-match (do not invent a silent turn).
fn has_no_reply_token(chunk: &str) -> bool {
    chunk.contains("<NO_REPLY>") || chunk.contains("[NO_REPLY]")
}

/// The fail-closed NO_REPLY filter. Ensures internal cognitive processes
/// never accidentally stream to external user replies.
pub struct NoReplyFilter {
    store: FileEpisodicStore,
}

impl NoReplyFilter {
    pub fn new(store: FileEpisodicStore) -> Self {
        Self { store }
    }

    /// Process a single LLM output chunk.
    /// If it contains <NO_REPLY>, dump to episodic log and suppress.
    /// Otherwise, forward to user.
    pub async fn process(&self, chunk: &str) -> FilterResult {
        if !is_forwardable_chunk(chunk) {
            return FilterResult::Suppressed;
        }
        if has_no_reply_token(chunk) {
            // Strip the NO_REPLY tokens before logging
            let cleaned = chunk
                .replace("<NO_REPLY>", "")
                .replace("</NO_REPLY>", "")
                .replace("[NO_REPLY]", "")
                .trim()
                .to_string();

            if !cleaned.is_empty() {
                let entry = EpisodicEntry {
                    timestamp: Utc::now(),
                    source: EpisodicSource::InternalMonologue,
                    content: cleaned,
                    is_silent: true,
                };

                // Fire-and-forget: we never block the main loop for logging
                if let Err(e) = self.store.append(&entry).await {
                    tracing::warn!("Failed to log silent turn: {e}");
                }
            }

            FilterResult::Suppressed
        } else {
            FilterResult::Forward(chunk.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_not_forwardable() {
        assert!(
            !is_forwardable_chunk(""),
            "empty input must fail closed — never Forward"
        );
        assert!(!is_forwardable_chunk("   \n\t  "));
    }

    #[test]
    fn no_match_plain_text_is_forwardable() {
        assert!(
            is_forwardable_chunk("hello from the model"),
            "real content with no NO_REPLY token must still forward"
        );
        assert!(!has_no_reply_token("hello from the model"));
        assert!(!has_no_reply_token(""));
        assert!(!has_no_reply_token("   "));
    }

    #[test]
    fn no_reply_tokens_match() {
        assert!(has_no_reply_token("<NO_REPLY> think silently"));
        assert!(has_no_reply_token("[NO_REPLY]"));
        assert!(!has_no_reply_token("NO_REPLY without brackets"));
    }
}
