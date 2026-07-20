# Ripen / knowledge_core honesty (2026-07-20)

## Verdict

**Overnight “0 rows” was gate starvation, not an empty core.**

| Fact | Evidence |
|------|----------|
| `knowledge_core.db` already large | **32 161** rows before today’s export (schema = fact export, Jul 8 bulk) |
| Nightly job printed 0 | `export-knowledge-core.py --min-confidence 0.90 --min-recall 3` while **all** `recall_count=0` |
| Dream narrative “ripen exported 0” | LLM paraphrasing that gate miss — not “core missing” |
| After Felt Use | dual gate **≥17**; origin-filtered export **15** rows (`INSERT OR REPLACE`) |

## Cause chain

```text
dead recall_count  →  dual gate empty  →  export 0  →  dream text says “ripen idle”
     ↑
Felt Use (2026-07-20) unblocks; search panel → nonzero recall → export emits
```

## Fixes shipped

1. `scripts/export-knowledge-core.py` — gate diagnostics + `data/ripen/latest.json` advice (`starved_recall` / `gate_miss` / `exported_N`).
2. `gzmo ripen status` — living census of dual / dual+origin / core size / last advice.
3. Immune refine — drop “Legacy auto_dream … disabled” false positives; keep clean-slate DreamEngine lore.
4. Example job prompt uses `/opt/gzmo/current/…` and asks the agent to surface `ripen/latest.json` advice.

## Operator actions (CT101)

```bash
GZMO_CONFIG=/opt/gzmo/gzmo.toml gzmo ripen status
python3 /opt/gzmo/current/scripts/export-knowledge-core.py \
  --db /opt/gzmo/data/vault.db --output /opt/gzmo/data/knowledge_core.db
# Optional: align living toml honeypot_ripen prompt with gzmo.toml.example
```

## Not claiming

- Charter concept-card ripen (`ripen-knowledge-core.py` / Rust `ripen_honeypot`) is a **different** schema path — living appliance uses the **fact export** DB today.
- We did not lower the dual gate; Felt Use made the existing gate honest.
