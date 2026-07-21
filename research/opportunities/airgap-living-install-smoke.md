---
id: airgap-living-install-smoke
title: Airgap living install smoke (stranger / one-box path)
status: soaked
score: 20
uniqueness: 4
brain_profit: 3
credit_cost: 5
attention_cost: 3
usp_fit: 5
stack_ids: []
created: 2026-07-20
updated: 2026-07-21
---

# Airgap living install smoke

## Why rare

USP is full living on one airgapped box — not MCP-on-webserver. Install path exists (`install-living-airgap.sh`) but day-to-day proof is still CT101-shaped.

## Brain profit

Stranger/operator can demable “is this box living-capable?” without claiming GREEN when sidecars/LLM are missing (honest degrade).

## Done when

Smoke/check wraps `install-living-airgap` / living-appliance compose + airgap honesty checklist; FAIL/HOLD never says living GREEN for lite; ADR-0003 refuse if another overnight writer exists.

**Soaked 2026-07-21** — `scripts/airgap-living-install-smoke.sh`; `living_green_claimed=false` always.

## Operator

```bash
bash scripts/airgap-living-install-smoke.sh
# AIRGAP_SMOKE_REQUIRE_LIVE=1  # optional hard sidecar requirement
```
