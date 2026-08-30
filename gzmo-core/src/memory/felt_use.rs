//! Felt Use — graded synaptic weight for honeypot facts.
//!
//! Search/MCP paths must touch living memory so `recall_count` can drive ripen.
//! `utility_score` is MemRL Q: it moves on cite/bond/outcome, not on glance.

use anyhow::Result;
use uuid::Uuid;

use crate::memory::vault::SqliteVault;

/// How strongly a fact was used in the organism.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeltUseKind {
    /// Ranked in search but not written to session scratch.
    Glance,
    /// Written into session `[RECALL]` scratch (operator-facing).
    Cited,
    /// Survived spark verify/promote or dream evidence bonding.
    Bonded,
    /// A later takeaway/distill observation cited a previously recalled fact.
    Outcome,
}

impl FeltUseKind {
    /// Ripen / recall-count delta (Glance still counts as felt, not as Q).
    pub fn recall_weight(self) -> i64 {
        match self {
            FeltUseKind::Glance => 1,
            FeltUseKind::Cited | FeltUseKind::Outcome => 3,
            FeltUseKind::Bonded => 5,
        }
    }

    /// MemRL Q delta. Glance is 0 so search traffic cannot mint utility.
    pub fn utility_weight(self) -> i64 {
        match self {
            FeltUseKind::Glance => 0,
            FeltUseKind::Cited => 3,
            FeltUseKind::Bonded => 5,
            FeltUseKind::Outcome => 8,
        }
    }

    pub fn weight(self) -> i64 {
        self.recall_weight()
    }
}

/// Increment honeypot recall (+ optional utility) by graded kind.
pub fn touch(vault: &SqliteVault, fact_id: Uuid, kind: FeltUseKind) -> Result<()> {
    vault.reinforce_felt(fact_id, kind.recall_weight(), kind.utility_weight())
}

/// Touch every vault/honeypot hit; skip Pi-knowledge rows (`fact_id` None).
pub fn touch_hits<'a, I>(vault: &SqliteVault, ids: I, kind: FeltUseKind)
where
    I: IntoIterator<Item = Option<&'a Uuid>>,
{
    for id in ids.into_iter().flatten() {
        if let Err(e) = touch(vault, *id, kind) {
            tracing::debug!(fact_id = %id, error = %e, "felt_use touch skipped");
        }
    }
}

/// Total MemRL Q delta. Glance is 0, so search traffic cannot mint utility.
pub fn utility_delta(events: &[(Uuid, FeltUseKind)]) -> i64 {
    events.iter().map(|(_, k)| k.utility_weight()).sum()
}

/// Events whose kind would change Q (non-Glance).
pub fn q_changing(events: &[(Uuid, FeltUseKind)]) -> Vec<(Uuid, FeltUseKind)> {
    events
        .iter()
        .copied()
        .filter(|(_, k)| k.utility_weight() != 0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weights_are_strictly_ordered() {
        assert!(FeltUseKind::Glance.recall_weight() < FeltUseKind::Cited.recall_weight());
        assert!(FeltUseKind::Cited.recall_weight() < FeltUseKind::Bonded.recall_weight());
        assert_eq!(FeltUseKind::Glance.utility_weight(), 0);
        assert!(FeltUseKind::Cited.utility_weight() < FeltUseKind::Bonded.utility_weight());
        assert!(FeltUseKind::Bonded.utility_weight() < FeltUseKind::Outcome.utility_weight());
    }

    #[test]
    fn utility_delta_empty_list_is_zero() {
        let events: &[(Uuid, FeltUseKind)] = &[];
        assert_eq!(utility_delta(events), 0);
        assert!(q_changing(events).is_empty());
    }

    #[test]
    fn utility_delta_all_glance_is_zero() {
        let id = Uuid::nil();
        let events = [(id, FeltUseKind::Glance), (id, FeltUseKind::Glance)];
        assert_eq!(utility_delta(&events), 0);
        assert!(q_changing(&events).is_empty());
    }

    #[test]
    fn utility_delta_mixed_outcome_and_glance() {
        let id = Uuid::nil();
        let events = [
            (id, FeltUseKind::Glance),
            (id, FeltUseKind::Outcome),
            (id, FeltUseKind::Glance),
        ];
        assert_eq!(utility_delta(&events), FeltUseKind::Outcome.utility_weight());
        assert_eq!(q_changing(&events), vec![(id, FeltUseKind::Outcome)]);
    }
}
