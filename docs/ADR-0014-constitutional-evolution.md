# ADR-0014 — Constitutional evolution and promotion

**Decision status:** Accepted (2026-08-31)
**Implementation status:** Not started
**Supersedes:** [ADR-0005](./ADR-0005-flywheel-over-frozen-topology.md); partial supersession of [ADR-0010](./ADR-0010-clean-sheet-onebox.md) (evolution phases move to implementation plan)
**Spec:** [2026-08-31-self-developing-living-database-design.md](./superpowers/specs/2026-08-31-self-developing-living-database-design.md) §§10–12, 15

## Context

Continuous improvement is necessary, but prior flywheel language mixed process velocity with production authority. Quality gates reported important properties without a complete evolution authority model. Candidates must be able to learn and propose without ever self-issuing production power.

## Decision

Constitutional self-development follows:

```text
Observe → Hypothesize → Build → Evaluate → Archive
                               │
                  bounded tunable or signed artifact
                               ▼
                       Promote → Soak
                               ▼
                        Keep | Roll back
```

### Authority tiers

| Tier | Scope | Production authority |
|---|---|---|
| **M — Memory** | Verified facts, evidence, supersession, consolidation, derived indexes, outcome learning | Autonomous within fixed memory floors |
| **T — Tunables** | Signed allowlisted numeric/enum parameters | Autonomous only inside the envelope after shadow evaluation and all hard floors |
| **C — Candidates** | Code, schemas, models, runtimes, security/evaluator changes | Generate/build/evaluate only; no production authority |
| **P — Promotion** | Bind approved artifact to inactive production target | Operator signature over artifact, evaluation, policy, target, and expiry |
| **A — Authority** | Roots, floors, evaluators, envelopes, allowed capabilities | Operator-only, out of agent write set |

### Trusted evolution controls

1. **Immutable evaluator.** Candidate code cannot alter evaluator binaries, fixtures, or scoring policy. Metrics/fixtures/evaluators are signed immutable inputs.
2. **Signed promotion.** Code/schema/model/security/capability changes require detached operator signature. Promotion binds only to an inactive target after hard-floor conjunction passes.
3. **Audit.** Candidates, evaluations, promotions, rejections, envelope changes, slot changes, and rollbacks are append-only and integrity-linked under AuditRoot.
4. **Soak.** One-node canarying uses deterministic replay, shadow evaluation, and time-bounded live soak against last-known-good on private hold-outs and captured real workloads before Keep.
5. **Rollback.** Every production update preserves verified last-known-good and an exercised rollback path. Failure restores the prior signed slot automatically; break-glass recovery remains signed and audited.

Production, builder, evaluator, audit sink, and promoter use separate OS identities or equivalent capability domains. The proposing process cannot score authoritatively, sign, widen its envelope, mark success, remove last-known-good, or rewrite audit.

## Invariants

- No self-issued authority (ADR-0011): candidates never modify trust roots, floors, envelopes, evaluator, promoter, or audit root.
- Hard floors are a conjunction evaluated before any quality comparison.
- Continuous improvement from ADR-0005 is retained only inside capability envelopes and these tiers.
- Evolving the evaluator or constitution is an Authority-tier proposal requiring separate operator review.

## Consequences

- ADR-0005 process/topology authority is superseded; retained intent is bounded continuous improvement under envelopes.
- Beat-gate / promote-by-loop historical practice is non-authoritative relative to M/T/C/P/A and signed promotion.
- Implementation plans may schedule flywheel work but cannot grant candidates production write, self-scoring, or envelope widening.
- Living status must expose pending candidate authority class, last-known-good readiness, and soak/rollback state.

## Rejected alternatives

- Unbounded autonomous code/schema/model self-promotion.
- Letting candidates modify evaluators, fixtures, or floors in the same proposal they are scored under.
- Performance-only promotion without hard-floor conjunction.
- Freezing all topology/process so continuous improvement cannot occur even inside signed envelopes.
- Multi-node canary requirements for the one-box product.

## Verification

- Candidate attempts at network access, production write, evaluator modification, key access, envelope widening, audit deletion, and false success marking all fail structurally (North Star §15.5).
- Recovery drills: power loss during update, bad tunable restore, schema-candidate rollback, lost TPM recovery-key path (§15.6).
- Fitness comparison on private hold-outs with all hard floors plus explicit deltas (§15.7); at least one complete metabolism/evolution soak under signed release policy (§15.8).
- `scripts/adr-check.sh` validates Accepted status, supersession links, and required headings.
