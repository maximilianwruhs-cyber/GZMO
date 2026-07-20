//! Felt Use — graded synaptic weight for honeypot facts.
//!
//! Search/MCP paths must touch living memory so `recall_count` can drive ripen,
//! profile ranking, and forget-lint. Weights distinguish glance vs cited vs bonded.

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
}

impl FeltUseKind {
    pub fn weight(self) -> i64 {
        match self {
            FeltUseKind::Glance => 1,
            FeltUseKind::Cited => 3,
            FeltUseKind::Bonded => 5,
        }
    }
}

/// Increment honeypot `recall_count` (+ vault confirmation) by graded weight.
pub fn touch(vault: &SqliteVault, fact_id: Uuid, kind: FeltUseKind) -> Result<()> {
    vault.reinforce_by(fact_id, kind.weight())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weights_are_strictly_ordered() {
        assert!(FeltUseKind::Glance.weight() < FeltUseKind::Cited.weight());
        assert!(FeltUseKind::Cited.weight() < FeltUseKind::Bonded.weight());
    }
}
