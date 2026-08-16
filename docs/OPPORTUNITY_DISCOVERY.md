# Opportunity discovery cycle

**Status:** Active (2026-07-20) — first product automation for *what to build next*  
**USP lock:** [ADR-0004-airgap-living-usp.md](./ADR-0004-airgap-living-usp.md)  
**Nutrient lock:** [BRAIN_FEED.md](./BRAIN_FEED.md)  
**Atlas:** [STACK_OPPORTUNITY_MAP.md](./STACK_OPPORTUNITY_MAP.md)  
**Gate:** `bash scripts/opportunity-discovery-check.sh` → `data-next/opportunity-discovery/`

## Why this exists

Ship babysitting keeps PRs green. Brain Feed nourishes the vault.  
**This cycle decides which rare upgrades deserve scarce attention** — without Socratic tourism or “keep going” babysitting.

```text
Sense  →  Rank  →  Bet  →  Ship mission  →  Soak
```

## Hard rules

1. **USP filter** — bets must strengthen airgap living and/or Brain Feed nutrients. Theater (HSP, pantheon, Observatory glass) is out unless it *feeds* the vault.
2. **Local intel parked** — do not bet on “run a smarter local 256k model on 32GB” until that world exists. Document as *horizon later*, not active ship.
3. **Credit honesty** — prefer upgrades that work as side-effects of real work or overnight metabolism. Do not require Cursor credit burn to stay healthy.
4. **One writer** — never propose a second overnight brain ([ADR-0003](./ADR-0003-one-instance-metabolism.md)).
5. **One active bet** — at most one `status: active` ship mission at a time (kids / attention budget).
6. **Not living KPI** — this cycle is operator research automation. It does not redefine keep-quality GREEN math.

## Ranking rubric (0–5 each)

| Axis | Question |
|------|----------|
| **uniqueness** | Hard to copy in the known universe (Mem0/RAG toys / cloud notebooks)? |
| **brain_profit** | Does it put verified mass or felt recall into the living vault? |
| **credit_cost** | Inverse: 5 = almost free of Cursor/cloud play; 0 = needs constant superior-model sessions |
| **attention_cost** | Inverse: 5 = runs without human babysitting; 0 = needs you every step |
| **usp_fit** | Strengthens airgap living / Brain Feed / keep-quality? |

**Score** = `uniqueness + brain_profit + credit_cost + attention_cost + usp_fit` (max 25).  
**Ship bar:** score ≥ 18 **and** `brain_profit ≥ 3` **and** `usp_fit ≥ 4`.

## Cadence (attention-honest)

| When | What |
|------|------|
| After keep-quality / brain-feed gate | `bash scripts/opportunity-sense.sh` |
| When picking next build | `bash scripts/opportunity-rank.sh` → review top rows |
| Lock a ship | `bash scripts/opportunity-bet.sh --from <id>` → bet file + mission card |
| After ship merges | Update bet `status: soaked` / `killed`; re-sense |

Weekly is enough. Daily Sense is optional if gates already ran.

### Sense v2 — find *new* upgrades (not empty-log theater)

After nutrient bets soak, Rank goes empty unless Sense mines **depth scars**. `opportunity-sense.sh` now also emits:

| Signal | Why |
|--------|-----|
| Felt Use depth census (`recall≥1` / `recall≥3` on living vault) | Ripen honesty — nonzero recall alone is a shallow win |
| Serendipity apply staleness | Cadence without apply ≠ vault mass |
| Soak sample spacing | Same-hour GREEN ≠ N nights |
| STACK nutrient gaps (`m1`,`s1`,`o1`,`m4`,`r5`,`o5`) | Map near/singular ids not covered by bet `stack_ids` |

If Sense says `bet_log_starved` or lists depth scars, **write new candidate bets** — do not re-ship soaked plumbing.

## Artifacts

| Path | Role |
|------|------|
| [`research/opportunities/`](../research/opportunities/) | Bet log (git-tracked) |
| [`research/lineage-watch/`](../research/lineage-watch/README.md) | Field SOTA — informs candidates; does **not** flip the active bet |
| [`docs/templates/MISSION_CARD.md`](./templates/MISSION_CARD.md) | Paste into agent kickoff |
| `data-next/opportunity-discovery/` | Sense / rank / check JSON (gitignored lab) |

## Operator commands

```bash
bash scripts/opportunity-sense.sh
bash scripts/opportunity-rank.sh
bash scripts/opportunity-bet.sh --from serendipity-apply-cadence
bash scripts/opportunity-discovery-check.sh
```

## Relationship to Socratic / discovery theater

Mutual-discovery / auto-Socratic is **pedagogy theater** — demoted for living KPI ([MUTUAL_DISCOVERY_THEATER.md](./MUTUAL_DISCOVERY_THEATER.md)).  
This cycle may *borrow* a Socratic grill for one bet’s stress-test; it must not spawn mentor cycles as health.

## What is automated (and what is not)

| Layer | Automated? | How |
|-------|------------|-----|
| Sense / Rank / cadence scripts | Yes | `opportunity-sense.sh`, `rank`, `serendipity-cadence.sh` (cron or local) |
| Bet log + mission card emit | Yes | `opportunity-next-mission.sh` → `data-next/opportunity-discovery/next-mission.md` |
| Weekly digest cadence | Yes (if enabled) | Cursor Automation **GZMO weekly opportunity + serendipity cadence** (cron) — digest only; **never** auto-applies serendipity |
| PR CI babysit | Yes (if enabled) | Cursor Automation **GZMO PR ship babysitter** — runs **after** a PR exists |
| Starting a new Cloud/IDE agent on the active bet | **No** by default | Needs one kickoff (“go” / paste mission card) **or** a separate Automation that pastes `next-mission.md` into a ship agent |
| Human “continue” in chat | Reduced | User rule **Finish through without babysitting** |

So: scripts + weekly digest + PR babysitter are automated **after** you enable them; **shipping a bet still needs one kickoff** unless you wire a cron ship agent to the mission card. That is intentional — kids / attention budget, not a missing feature.

## Success

- Bet log has ≥1 `active` or recently `soaked` bet aligned to USP  
- `opportunity-discovery-check.sh` GREEN  
- Next agent ship starts from a mission card, not from “what should we do?” chat  
