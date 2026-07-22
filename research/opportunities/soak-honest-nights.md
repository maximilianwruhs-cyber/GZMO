---
id: soak-honest-nights
title: Honest soak nights (≥18h spacing)
status: candidate
score: 19
uniqueness: 2
brain_profit: 3
credit_cost: 5
attention_cost: 4
usp_fit: 5
stack_ids: [r5]
created: 2026-07-21
updated: 2026-07-22
---

# Honest soak nights

## Why rare

Same-hour GREEN streaks are dishonest. Theater Unpark requires `soak_ready_unpark_ok` with ≥3 GREEN samples spaced ≥18h. Craft may continue while soak HOLDs — soak gates theater, not craft (ADR-0005).

## Brain profit

Protects USP bar so Unpark cannot claim strengthen while soak is theatrical.

## Done when

1. `bash scripts/keep-quality-soak.sh --summary` reports `soak_ready_unpark_ok` / honest_nights≥3
2. No same-hour triple-GREEN gaming
3. Timer already soaked (`soak-night-honest-timer`) — this bet is operator nights only

## Progress 2026-07-22

- Scar still live: `soak_spacing_hold` — trailing_GREEN=3 but honest_nights=1 (rejected_close=2)
- Keep-quality gate GREEN today (counts as one honest sample only after ≥18h from prior)
- **Blocked on wall-clock** — cannot finish this bet in one session

## Operator

```bash
# Once per calendar night for 3 nights:
bash scripts/keep-quality-soak.sh --summary
```

## Sources

- [stack-future-opportunities-2026-07-21.md](../stack-future-opportunities-2026-07-21.md) O2
- [soak-night-honest-timer.md](soak-night-honest-timer.md)
