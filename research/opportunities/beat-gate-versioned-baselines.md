---
id: beat-gate-versioned-baselines
title: Beat-gate versioned baselines + honest gate_passed
status: soaked
score: 21
uniqueness: 4
brain_profit: 3
credit_cost: 5
attention_cost: 4
usp_fit: 5
stack_ids: [o2]
created: 2026-07-21
updated: 2026-07-21
---

# Beat-gate versioned baselines

## Why rare

Kit PASS today can mean “recipe exited 0” with `gate_passed: null` on cognition/knowledge. Promotion science needs versioned incumbent baselines and a boolean gate for every loop — not soft nulls.

## Brain profit

Honest S2 → safe promote-by-loop (ADR-0005). Wrong green kit is how theater sneaks into living.

## Done when

1. Committed baseline JSON per core loop under `little-tools-lab/fixtures/beat-baselines/`
2. `beat-gate-meta` loads baseline file, emits `baseline_id` + `metrics.gate_passed` for every loop
3. `beat-gate-kit.sh` fails soft report if cognition/knowledge `gate_passed` is null
4. Full kit still PASS fixture 5/5 with versioned baselines

## Operator

```bash
bash scripts/beat-gate-kit.sh --loops config,cognition,knowledge,discovery,ops
python3 -c "import json;d=json.load(open('data-next/beat-gate/latest.json'));print([(r['loop'],r.get('gate_passed'),r.get('baseline_id')) for r in d['loops']])"
```

## Sources

- [stack-future-opportunities-2026-07-21.md](../stack-future-opportunities-2026-07-21.md) O4
- [CONTINUOUS_UPGRADE.md](../../docs/CONTINUOUS_UPGRADE.md) W3

**Soaked 2026-07-21** — versioned baselines + kit honesty (gate_passed + baseline_id) green 5/5.
