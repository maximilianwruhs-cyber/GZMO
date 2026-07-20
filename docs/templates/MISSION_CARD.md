# Mission card (agent kickoff)

Copy below into a new agent chat. Do not babysit with “continue.”

---

## Mission

**Bet id:** `<opportunity-id>`  
**Title:** `<one line>`  
**Why rare:** `<uniqueness one sentence>`  
**Brain profit:** `<how vault/honeypot improves>`  
**Done when:** `<falsifiable exit>`  

## Constraints

- USP: airgap living ([ADR-0004](../ADR-0004-airgap-living-usp.md)); Brain Feed nutrients preferred ([BRAIN_FEED.md](../BRAIN_FEED.md))
- One overnight writer ([ADR-0003](../ADR-0003-one-instance-metabolism.md))
- No local-intel quests; no Socratic tourism; no public webserver SKU
- Finish-through: implement → verify → commit → push → PR → CI green → stop with PR URL or blocker

## Verify

```bash
# at minimum:
bash scripts/opportunity-discovery-check.sh
# plus mission-specific gates, e.g.:
# bash scripts/brain-feed-check.sh
# bash scripts/keep-quality-gate.sh   # LIVING_GATE_SKIP_TAKEAWAY=1 if needed
```

## Bet file

`research/opportunities/<id>.md`

---
