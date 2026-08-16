//! # Context Window Management
//!
//! Prevents context overflow by intelligently pruning conversation history
//! before each LLM call. The full history is preserved in memory (and on disk
//! via session persistence) — only the *view* sent to the model is trimmed.
//!
//! ## Strategy
//!
//! 1. System prompt is always retained (index 0).
//! 2. Active workflow `SKILL.md` payloads (`[Workflow /…]`) are pinned
//!    system-adjacent (HumanLayer). Pantheon slash skills are not.
//! 3. Remaining budget fills from most-recent backward.
//! 4. Tool chain integrity: if a `Tool` result is kept, its parent
//!    `Assistant` tool-call message is also kept.
//! 5. Token estimation uses a rough heuristic (chars / 3.5) which is conservative
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

/// Walk back from a Tool result to its parent Assistant tool-call.
fn parent_assistant_index(conversation: &[Message], tool_idx: usize) -> Option<usize> {
    let mut j = tool_idx;
    while j > 0 {
        j -= 1;
        if conversation[j].role == Role::Assistant {
            return Some(j);
        }
    }
    None
}

/// True when this turn is an active workflow `SKILL.md` payload (`[Workflow /…]`).
///
/// Protects grill/tdd/diagnose/review/handoff contracts from prune compaction.
/// Pantheon chaos slash skills (`/dice`, `/story`, …) are not pinned.
pub fn is_pinned_workflow_skill(msg: &Message) -> bool {
    msg.content.starts_with("[Workflow /")
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
    let (windowed, archived) = prune_to_budget_with_archive(messages, &trim_cfg);
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

/// Prune messages to fit within the token budget.
///
/// Returns a new `Vec<Message>` containing:
/// 1. The system prompt (always first)
/// 2. Pinned workflow skill contracts (system-adjacent, never dropped unless total budget is smaller than system + skill)
/// 3. As many recent messages as fit within the remaining budget
/// 4. Tool chain integrity preserved (tool results keep their parent tool-call message)
///
/// The input `messages` is NOT mutated.
pub fn prune_to_budget(messages: &[Message], config: &ContextConfig) -> Vec<Message> {
    prune_with_archive(messages, config).windowed
}

fn prune_to_budget_with_archive(
    messages: &[Message],
    config: &ContextConfig,
) -> (Vec<Message>, Vec<Message>) {
    if messages.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let total = estimate_total_tokens(messages, config.chars_per_token);

    // If we're under budget, return everything as-is
    if total <= config.max_tokens {
        return (messages.to_vec(), Vec::new());
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
        let archived = messages[1..].to_vec();
        return (vec![system_msg.clone()], archived);
    }

    let conversation = &messages[1..]; // everything after system prompt
    let max_conv_budget = config.max_tokens - system_tokens;

    // 1. Identify pinned workflow skill messages (system-adjacent)
    let mut pinned_indices = Vec::new();
    for (i, msg) in conversation.iter().enumerate() {
        if is_pinned_workflow_skill(msg) {
            if msg.role == Role::Tool {
                if let Some(parent) = parent_assistant_index(conversation, i) {
                    if !pinned_indices.contains(&parent) {
                        pinned_indices.push(parent);
                    }
                }
            }
            if !pinned_indices.contains(&i) {
                pinned_indices.push(i);
            }
        }
    }

    let mut keep_set = HashSet::new();
    let mut used_tokens = 0usize;

    // Reserve tokens for pinned workflow skills first (most recent pinned preferred if tight)
    let mut pinned_rev = pinned_indices.clone();
    pinned_rev.sort_unstable();
    pinned_rev.reverse();

    for &idx in &pinned_rev {
        let msg_tokens = estimate_tokens(&conversation[idx].content, config.chars_per_token);
        if used_tokens + msg_tokens <= max_conv_budget {
            keep_set.insert(idx);
            used_tokens += msg_tokens;
        } else {
            tracing::warn!(
                index = idx,
                "Pinned workflow skill exceeds available context budget"
            );
        }
    }

    // 2. Walk backwards from the most recent non-pinned messages in conversation
    for (i, msg) in conversation.iter().enumerate().rev() {
        if keep_set.contains(&i) {
            continue;
        }
        let msg_tokens = estimate_tokens(&msg.content, config.chars_per_token);
        if used_tokens + msg_tokens > max_conv_budget {
            break;
        }
        keep_set.insert(i);
        used_tokens += msg_tokens;
    }

    // 3. Tool chain integrity: ensure that if we have a Tool message,
    // we also have the preceding Assistant message that requested it.
    let mut final_indices: Vec<usize> = Vec::new();
    let mut sorted_indices: Vec<usize> = keep_set.into_iter().collect();
    sorted_indices.sort_unstable();

    let check_set: HashSet<usize> = sorted_indices.iter().copied().collect();
    for &idx in &sorted_indices {
        let msg = &conversation[idx];
        if msg.role == Role::Tool {
            match parent_assistant_index(conversation, idx) {
                Some(parent) if check_set.contains(&parent) => {}
                None if is_pinned_workflow_skill(msg) => {}
                Some(_) | None => {
                    tracing::debug!(
                        index = idx,
                        "Dropping orphaned tool result (parent tool-call was pruned)"
                    );
                    continue;
                }
            }
        }
        final_indices.push(idx);
    }

    let final_set: HashSet<usize> = final_indices.iter().copied().collect();

    // Build windowed messages
    let mut windowed = Vec::with_capacity(final_indices.len() + 1);
    windowed.push(system_msg.clone());
    for &idx in &final_indices {
        windowed.push(conversation[idx].clone());
    }

    // Build archived messages: all conversation messages dropped from hot window
    let mut archived = Vec::new();
    for (i, msg) in conversation.iter().enumerate() {
        if !final_set.contains(&i) {
            archived.push(msg.clone());
        }
    }

    let pruned_total = estimate_total_tokens(&windowed, config.chars_per_token);
    let original_count = messages.len();
    let pruned_count = windowed.len();

    if pruned_count < original_count {
        tracing::info!(
            original_messages = original_count,
            pruned_messages = pruned_count,
            dropped = original_count - pruned_count,
            estimated_tokens = pruned_total,
            budget = config.max_tokens,
            pinned_skills = pinned_indices.len(),
            "Context window pruned (workflow skills protected)"
        );
    }

    (windowed, archived)
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

    #[test]
    fn test_pinned_workflow_skill_survives_pruning() {
        let mut messages = vec![
            make_msg(Role::System, "System prompt for GZMO."),
            make_msg(
                Role::Tool,
                "[Workflow /grill]\n# grill\n\nAsk clarifying questions before coding.",
            ),
        ];

        // Add 30 user/assistant turns that would easily exceed budget
        for i in 0..30 {
            messages.push(make_msg(
                Role::User,
                &format!("User message {i} with conversational filler text"),
            ));
            messages.push(make_msg(
                Role::Assistant,
                &format!("Assistant message {i} with conversational filler text"),
            ));
        }

        let config = ContextConfig::with_hot_budget(250);
        let result = prune_with_archive(&messages, &config);

        // System prompt must be kept (index 0)
        assert_eq!(result.windowed[0].role, Role::System);

        // Pinned workflow skill must survive in windowed messages
        let has_workflow_skill = result
            .windowed
            .iter()
            .any(|m| m.content.contains("[Workflow /grill]"));
        assert!(
            has_workflow_skill,
            "Workflow skill contract must survive pruning"
        );

        // Archived slice must contain dropped non-pinned turns
        assert!(
            !result.archived.is_empty(),
            "Non-pinned turns should be archived"
        );
        let archived_has_skill = result
            .archived
            .iter()
            .any(|m| m.content.contains("[Workflow /grill]"));
        assert!(
            !archived_has_skill,
            "Pinned workflow skill must not be archived"
        );
    }

    #[test]
    fn test_pinned_workflow_tool_call_chain_integrity() {
        let mut messages = vec![
            make_msg(Role::System, "System prompt."),
            make_msg(Role::Assistant, "[Tool call: activate_workflow_skill(tdd)]"),
            make_msg(
                Role::Tool,
                "[Workflow /tdd]\n# tdd\n\nWrite tests first, then write implementation.",
            ),
        ];

        for i in 0..25 {
            messages.push(make_msg(
                Role::User,
                &format!("Chat turn {i} consuming tokens"),
            ));
            messages.push(make_msg(
                Role::Assistant,
                &format!("Chat response {i} consuming tokens"),
            ));
        }

        let config = ContextConfig::with_hot_budget(220);
        let windowed = prune_to_budget(&messages, &config);

        // Both tool result and its preceding tool call must survive
        let tool_idx = windowed
            .iter()
            .position(|m| m.content.contains("[Workflow /tdd]"));
        assert!(tool_idx.is_some(), "Pinned Tool result must survive");
        let tool_idx = tool_idx.unwrap();
        assert!(tool_idx > 0);
        assert_eq!(
            windowed[tool_idx - 1].role,
            Role::Assistant,
            "Preceding Assistant tool-call must be preserved for pinned Tool message"
        );
    }

    #[test]
    fn test_pinned_workflow_survives_sibling_tool_result() {
        let dump = format!("SHELL DUMP: {}", "x".repeat(400));
        let mut messages = vec![
            make_msg(Role::System, "System prompt."),
            make_msg(
                Role::Assistant,
                "[Tool calls: shell_exec, activate_workflow_skill]",
            ),
            make_msg(Role::Tool, &dump),
            make_msg(
                Role::Tool,
                "[Workflow /review]\n# review\n\nCite evidence before claiming done.",
            ),
        ];
        for i in 0..20 {
            messages.push(make_msg(
                Role::User,
                &format!("Later user {i} filling tokens"),
            ));
            messages.push(make_msg(
                Role::Assistant,
                &format!("Later assistant {i} filling tokens"),
            ));
        }
        let config = ContextConfig::with_hot_budget(280);
        let windowed = prune_to_budget(&messages, &config);
        assert!(
            windowed
                .iter()
                .any(|m| m.content.contains("[Workflow /review]")),
            "Workflow skill after a sibling tool result must survive"
        );
        let skill_idx = windowed
            .iter()
            .position(|m| m.content.contains("[Workflow /review]"))
            .unwrap();
        assert!(windowed[..skill_idx]
            .iter()
            .any(|m| m.role == Role::Assistant));
    }

    #[test]
    fn test_unpinned_pantheon_slash_skill_prunes() {
        let mut messages = vec![
            make_msg(Role::System, "System prompt."),
            make_msg(Role::Tool, "/dice\nYou rolled a 6 on 1d6."),
        ];

        for i in 0..30 {
            messages.push(make_msg(
                Role::User,
                &format!("User {i} filling the token buffer"),
            ));
            messages.push(make_msg(
                Role::Assistant,
                &format!("Assistant {i} filling the token buffer"),
            ));
        }

        let config = ContextConfig::with_hot_budget(150);
        let windowed = prune_to_budget(&messages, &config);

        let has_dice = windowed.iter().any(|m| m.content.contains("/dice"));
        assert!(
            !has_dice,
            "Pantheon chaos slash skill must NOT be pinned against pruning"
        );
    }
}
