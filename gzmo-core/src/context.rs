//! # Context Window Management
//!
//! Prevents context overflow by intelligently pruning conversation history
//! before each LLM call. The full history is preserved in memory (and on disk
//! via session persistence) — only the *view* sent to the model is trimmed.
//!
//! ## Strategy
//!
//! 1. System prompt is always retained (index 0).
//! 2. Messages are kept from most-recent backward until the token budget is reached.
//! 3. Tool chain integrity: if a `Tool` result message is kept, the preceding
//!    `Assistant` tool-call message that triggered it is also kept.
//! 4. Token estimation uses a rough heuristic (chars / 3.5) which is conservative
//!    enough for most tokenizers (GPT, Llama, Qwen all average ~3.5-4.0 chars/token).

use crate::types::Message;
use crate::types::Role;
use std::collections::HashSet;

/// Configuration for context window management.
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Maximum token budget for the hot context window (after response reserve).
    pub max_tokens: usize,

    /// Characters-per-token estimate. Conservative default of 3.5.
    /// Lower values = more aggressive pruning (safer).
    pub chars_per_token: f64,

    /// Archive when estimated tokens exceed this (default: 90% of max_tokens).
    pub archive_trigger_tokens: usize,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self::with_hot_budget(6144)
    }
}

impl ContextConfig {
    /// Hot budget with 90% archive trigger.
    pub fn with_hot_budget(hot_tokens: usize) -> Self {
        Self {
            max_tokens: hot_tokens,
            chars_per_token: 3.5,
            archive_trigger_tokens: (hot_tokens as f64 * 0.90) as usize,
        }
    }

    /// Create a context config for a specific model context length.
    /// Reserves 10% for response; hot budget uses remaining space.
    pub fn for_context_length(context_length: usize) -> Self {
        let hot = (context_length as f64 * 0.90) as usize;
        Self::with_hot_budget(hot)
    }

    /// From `[context_memory]` settings.
    pub fn from_memory_config(cfg: &crate::config::ContextMemoryConfig) -> Self {
        Self::with_hot_budget(cfg.hot_budget_tokens())
    }
}

/// Result of pruning with optional archival slice.
#[derive(Debug, Clone)]
pub struct PruneResult {
    pub windowed: Vec<Message>,
    pub archived: Vec<Message>,
    pub estimated_before: usize,
    pub estimated_after: usize,
}

/// Estimate token count for a plain string (role overhead included).
pub fn estimate_text_tokens(content: &str, chars_per_token: f64) -> usize {
    let content_tokens = (content.len() as f64 / chars_per_token).ceil() as usize;
    content_tokens + 4
}

/// Estimate the token count of a message.
fn estimate_tokens(content: &str, chars_per_token: f64) -> usize {
    // Add overhead for role tag, formatting, etc. (~4 tokens per message)
    let content_tokens = (content.len() as f64 / chars_per_token).ceil() as usize;
    content_tokens + 4
}

/// Estimate total token count for a slice of messages.
pub fn estimate_total_tokens(messages: &[Message], chars_per_token: f64) -> usize {
    messages
        .iter()
        .map(|m| estimate_tokens(&m.content, chars_per_token))
        .sum()
}

/// Prune with archival: messages dropped from the hot window go to `archived`
/// when total tokens exceed `archive_trigger_tokens` (90%) or `max_tokens`.
pub fn prune_with_archive(messages: &[Message], config: &ContextConfig) -> PruneResult {
    if messages.is_empty() {
        return PruneResult {
            windowed: Vec::new(),
            archived: Vec::new(),
            estimated_before: 0,
            estimated_after: 0,
        };
    }

    let estimated_before = estimate_total_tokens(messages, config.chars_per_token);

    let target_budget = if estimated_before > config.max_tokens {
        config.max_tokens
    } else if estimated_before > config.archive_trigger_tokens {
        config.archive_trigger_tokens
    } else {
        return PruneResult {
            windowed: messages.to_vec(),
            archived: Vec::new(),
            estimated_before,
            estimated_after: estimated_before,
        };
    };

    let mut trim_cfg = config.clone();
    trim_cfg.max_tokens = target_budget;
    let windowed = prune_to_budget_inner(messages, &trim_cfg);
    let archived = messages_not_in_window(messages, &windowed);
    let estimated_after = estimate_total_tokens(&windowed, config.chars_per_token);

    if !archived.is_empty() {
        tracing::info!(
            archived_messages = archived.len(),
            estimated_before,
            estimated_after,
            target_budget,
            "Context archived"
        );
    }

    PruneResult {
        windowed,
        archived,
        estimated_before,
        estimated_after,
    }
}

fn messages_not_in_window(original: &[Message], windowed: &[Message]) -> Vec<Message> {
    if original.len() <= 1 {
        return Vec::new();
    }
    let keep_from = original
        .len()
        .saturating_sub(windowed.len().saturating_sub(1));
    if keep_from <= 1 {
        return Vec::new();
    }
    original[1..keep_from].to_vec()
}

/// Prune messages to fit within the token budget.
///
/// Returns a new `Vec<Message>` containing:
/// 1. The system prompt (always first)
/// 2. As many recent messages as fit within the budget
/// 3. Tool chain integrity preserved (tool results keep their parent tool-call message)
///
/// The input `messages` is NOT mutated.
pub fn prune_to_budget(messages: &[Message], config: &ContextConfig) -> Vec<Message> {
    prune_with_archive(messages, config).windowed
}

fn prune_to_budget_inner(messages: &[Message], config: &ContextConfig) -> Vec<Message> {
    if messages.is_empty() {
        return Vec::new();
    }

    let total = estimate_total_tokens(messages, config.chars_per_token);

    // If we're under budget, return everything as-is
    if total <= config.max_tokens {
        return messages.to_vec();
    }

    // Always keep the system prompt
    let system_msg = &messages[0];
    let system_tokens = estimate_tokens(&system_msg.content, config.chars_per_token);

    // If the system prompt alone exceeds the budget, we're in trouble
    // but we still return it (the model needs SOME instructions)
    if system_tokens >= config.max_tokens {
        tracing::warn!(
            system_tokens,
            budget = config.max_tokens,
            "System prompt alone exceeds token budget"
        );
        return vec![system_msg.clone()];
    }

    let remaining_budget = config.max_tokens - system_tokens;

    // Walk backwards from the most recent message, accumulating tokens
    let conversation = &messages[1..]; // everything after system prompt
    let mut keep_indices: Vec<usize> = Vec::new();
    let mut used_tokens = 0usize;

    for (i, msg) in conversation.iter().enumerate().rev() {
        let msg_tokens = estimate_tokens(&msg.content, config.chars_per_token);

        if used_tokens + msg_tokens > remaining_budget {
            break;
        }

        keep_indices.push(i);
        used_tokens += msg_tokens;
    }

    // Reverse to maintain chronological order
    keep_indices.reverse();

    // Tool chain integrity: ensure that if we have a Tool message,
    // we also have the preceding Assistant message that requested it.
    // Use HashSet for O(1) lookups instead of O(N²) Vec::contains.
    let keep_set: HashSet<usize> = keep_indices.iter().copied().collect();
    let mut final_indices: Vec<usize> = Vec::new();

    for &idx in &keep_indices {
        let msg = &conversation[idx];

        if msg.role == Role::Tool && idx > 0 {
            // Check if the previous message (the tool-call request) is already included
            let prev_idx = idx - 1;
            if !keep_set.contains(&prev_idx) {
                // The parent tool-call message was pruned — we need to drop this
                // orphaned tool result too, as it makes no sense without context
                tracing::debug!(
                    index = idx,
                    "Dropping orphaned tool result (parent tool-call was pruned)"
                );
                continue;
            }
        }

        final_indices.push(idx);
    }

    // Build the pruned message list
    let mut pruned = Vec::with_capacity(final_indices.len() + 1);
    pruned.push(system_msg.clone());

    for idx in final_indices {
        pruned.push(conversation[idx].clone());
    }

    let pruned_total = estimate_total_tokens(&pruned, config.chars_per_token);
    let original_count = messages.len();
    let pruned_count = pruned.len();

    if pruned_count < original_count {
        tracing::info!(
            original_messages = original_count,
            pruned_messages = pruned_count,
            dropped = original_count - pruned_count,
            estimated_tokens = pruned_total,
            budget = config.max_tokens,
            "Context window pruned"
        );
    }

    pruned
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_msg(role: Role, content: &str) -> Message {
        Message {
            role,
            content: content.to_string(),
            is_meta: false,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    #[test]
    fn test_under_budget_returns_all() {
        let messages = vec![
            make_msg(Role::System, "You are GZMO."),
            make_msg(Role::User, "Hello"),
            make_msg(Role::Assistant, "Hi there!"),
        ];

        let config = ContextConfig::with_hot_budget(10000);

        let pruned = prune_to_budget(&messages, &config);
        assert_eq!(pruned.len(), 3);
    }

    #[test]
    fn test_over_budget_keeps_recent() {
        let mut messages = vec![make_msg(Role::System, "System prompt.")];

        // Add 50 user/assistant pairs with substantial content
        for i in 0..50 {
            messages.push(make_msg(
                Role::User,
                &format!(
                    "User message number {} with some extra content to use tokens",
                    i
                ),
            ));
            messages.push(make_msg(
                Role::Assistant,
                &format!(
                    "Assistant response number {} with some extra content to use tokens",
                    i
                ),
            ));
        }

        let config = ContextConfig::with_hot_budget(200);

        let pruned = prune_to_budget(&messages, &config);

        // Should keep system prompt + some recent messages
        assert!(
            pruned.len() < messages.len(),
            "Should have pruned some messages"
        );
        assert!(pruned.len() >= 2, "Should keep at least system + 1 message");

        // First message should always be system
        assert_eq!(pruned[0].role, Role::System);

        // Last message should be the most recent from the original
        let last_pruned = &pruned[pruned.len() - 1];
        let last_original = &messages[messages.len() - 1];
        assert_eq!(last_pruned.content, last_original.content);
    }

    #[test]
    fn test_tool_chain_integrity() {
        let messages = vec![
            make_msg(Role::System, "System."),
            make_msg(Role::User, "Old user message"),
            make_msg(Role::Assistant, "[Tool calls: shell_exec]"),
            make_msg(Role::Tool, "[Tool Result: shell_exec] output here"),
            make_msg(Role::User, "Recent user message"),
            make_msg(Role::Assistant, "Recent response"),
        ];

        let config = ContextConfig::with_hot_budget(100);

        let pruned = prune_to_budget(&messages, &config);

        // If a Tool message is kept, its parent Assistant should also be kept
        for (i, msg) in pruned.iter().enumerate() {
            if msg.role == Role::Tool && i > 0 {
                assert_eq!(
                    pruned[i - 1].role,
                    Role::Assistant,
                    "Tool message should be preceded by its parent Assistant message"
                );
            }
        }
    }

    #[test]
    fn test_prune_with_archive_under_threshold() {
        let messages = vec![
            make_msg(Role::System, "System."),
            make_msg(Role::User, "Short."),
            make_msg(Role::Assistant, "Ok."),
        ];
        let config = ContextConfig::with_hot_budget(10_000);
        let result = prune_with_archive(&messages, &config);
        assert!(result.archived.is_empty());
        assert_eq!(result.windowed.len(), messages.len());
    }

    #[test]
    fn test_prune_with_archive_over_threshold() {
        let mut messages = vec![make_msg(Role::System, "System prompt.")];
        for i in 0..40 {
            messages.push(make_msg(
                Role::User,
                &format!("User {i} with enough text to consume token budget quickly"),
            ));
            messages.push(make_msg(
                Role::Assistant,
                &format!("Assistant {i} with enough text to consume token budget quickly"),
            ));
        }
        let config = ContextConfig::with_hot_budget(200);
        let result = prune_with_archive(&messages, &config);
        assert!(
            !result.archived.is_empty(),
            "expected archival when over 90% trigger"
        );
        assert!(result.windowed.len() < messages.len());
        assert!(result.estimated_after <= config.archive_trigger_tokens + 50);
    }

    #[test]
    fn test_token_estimation() {
        // "Hello" = 5 chars → 5/3.5 ≈ 2 + 4 overhead = 6 tokens
        let tokens = estimate_tokens("Hello", 3.5);
        assert_eq!(tokens, 6);

        // Empty string → 0 + 4 overhead = 4 tokens
        let tokens = estimate_tokens("", 3.5);
        assert_eq!(tokens, 4);
    }
}
