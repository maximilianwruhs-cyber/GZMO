# felt-use-mass-growth vs what `origin/main` already shipped

Primary sources: `research/opportunities/felt-use-mass-growth.md` (active), `felt-use-ripen-floor.md` (soaked), `gzmo-core/src/memory/recall_rrf.rs`, `scripts/felt-use-depth.sh`. Tip: `origin/main` `8fc07a2` (2026-08-30).

## Opportunity done-when

1. Living schema has `honeypot.utility_score` and search orders by it.
2. Weekly `felt-use-depth.sh` + utility census show rising dual-gate / utility mass from real sessions.
3. Brain Feed stays GREEN; no memory-gym.

## Already on main

- Schema: this Keep's `honeypot.utility_score` column exists (PRAGMA). 78 of 3005 latest rows have `utility_score > 0`.
- Search: `recall_rrf` → rerank → `apply_utility_select` / `apply_utility_boost` (Q-select inside the pool). Glance does not mint Q.
- Census script: `felt-use-depth.sh` uses the local living vault when `~/.gzmo-living/data/vault.db` exists (no SSH). Telescope line that mass is "CT101-only until #166" is **stale**.
- Ripen-floor census artifact (soaked opportunity) is a measurement tool, not doctrine ingest.

## Still required (not shipped as living outcome)

- **Doctrine row**: opportunity never asked for a Felt Use / MemRL fact in the vault. Mechanism without the row is why search collides.
- **Done-when 2**: rising dual-gate / utility mass from real sessions — not yet a trend on this Keep; 78 utility-positive is thin vs 3005 latest.
- **Done-when 3**: Brain Feed GREEN belongs to a later map (fog on [Felt Use findable](https://github.com/maximilianwruhs-cyber/GZMO/issues/221)), not this spec's prove-first gate.

Product tip is `origin/main`. `feat/living-research-intel` is not the tip.
