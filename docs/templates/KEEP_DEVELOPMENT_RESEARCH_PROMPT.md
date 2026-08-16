# Keep development — research prompt (agent kickoff)

Copy everything below the line into a **new** agent chat. Do not babysit with “continue.”  
This is how GZMO keeps developing after parking operator glass (Observatory / OKForge).

---

## Mission

**Product:** a living Keep — honeypot + verify + promote on one airgapped box.  
**Not the product:** Observatory glass, OKForge wiki export, HSP, pantheon, AOS CE, `eml-core`, stitcher-as-OS, public forge SKU.

**Active ship (exactly one):** [`felt-use-mass-growth`](../../research/opportunities/felt-use-mass-growth.md) — genuine recall/utility mass from **real CT101 sessions**. Schema/`utility_score`/Q-select are already in-tree. Remaining done-when is **living mass**, not more telescope plumbing.

**Research job (this chat):** harvest local-memory *algorithms* from the field, map each steal onto an **existing organ**, propose at most **one** candidate graft that could follow felt-use mass — then implement only if it is the active bet or a legally small loop on that bet. Do not start a second overnight writer. Do not flip `felt-use-mass-growth` off to chase theater GREEN.

**Done when (falsifiable):**

1. A written harvest table: steal / non-steal / organ / in-tree gap, citing [research/lineage-watch/](../../research/lineage-watch/README.md).
2. CT101 census attempted: `bash scripts/felt-use-depth.sh` (SSH fail → `INCONCLUSIVE`/`RED`, never synthetic 0=GREEN).
3. Either (a) one PR that moves felt-use/utility on a real path, or (b) a candidate bet file with score ≥18, `brain_profit≥3`, `usp_fit≥4`, status `candidate` — not `active`.
4. Explicit park list for this sitting (glass, forge, gym, SKU).

---

## Read first (in this order)

1. [`MACHINE.md`](../../MACHINE.md) — two sentences.
2. [`docs/TELESCOPE_LINEAGE.md`](../TELESCOPE_LINEAGE.md) — two rooms, one animal.
3. [`docs/ADR-0004-airgap-living-usp.md`](../ADR-0004-airgap-living-usp.md) · [`docs/ADR-0007-one-product-living.md`](../ADR-0007-one-product-living.md)
4. [`docs/BRAIN_FEED.md`](../BRAIN_FEED.md) — nutrients vs theater.
5. [`docs/KEEP_QUALITY.md`](../KEEP_QUALITY.md) · [`docs/OPPORTUNITY_DISCOVERY.md`](../OPPORTUNITY_DISCOVERY.md)
6. [`research/opportunities/felt-use-mass-growth.md`](../../research/opportunities/felt-use-mass-growth.md)
7. [`research/lineage-watch/README.md`](../../research/lineage-watch/README.md) + latest `sota-*.md`

Workspace: telescope `/home/mw/gzmo_full`. Living: CT101 `/opt/gzmo` (`current`, vault `/opt/gzmo/data/vault.db`). This host does **not** run `gzmo serve`.

---

## What is already decided (do not re-litigate)

| Decision | Meaning |
|----------|---------|
| One writer | ADR-0003 / mutex. Telescope never overnight. |
| No lite SKU | Clients attach to the living writer. `~/.gzmo` is incomplete. |
| Glass ≠ Keep | Observatory / OKForge wiki push are operator export. Soft-fail. Not living GREEN. Not a public SKU. PR 169 is costume plumbing if merged. |
| Theater parked | HSP, pantheon, AOS CE, wiki-mind demo — demable after **3 honest soak nights** (≥18h spacing), not a research excuse. |
| Harvest ≠ SKU | Steal retrieve/forget/supersede/consolidate/verify/time. Reject SaaS, multi-tenant HTTP, cloud-required core, overnight LoRA. |
| One active bet | `felt-use-mass-growth` until soaked or killed. Lineage-watch must not spawn a second `status: active`. |
| `eml-core` | R&D calculator. No `gzmo-core` callers. Not an organ. |
| herdr / okforge mirrors | Private R&D. Do not publicize. |

---

## Organs you may evolve (loops, not new repos)

Work **one loop** per PR. Map every steal here:

| Organ | Job | Honest evolve-toward |
|-------|-----|----------------------|
| Distill | Session that happened → candidate facts | Denser facts; origin tags that survive into honeypot |
| Gate | Qualify or refuse | Typed refuse/supersede; rollback on failed promote |
| Spark | Stale × importance × cosine, then **verify** | Verified links; no mood |
| Recall | RRF + **utility then recall**; felt-use touch | Q moves from later real work, not search gym |
| Dream / ripen / immune | Compact, dual-gate ripen, value forget | Region rewrite that *supersedes*; forget as signal |
| Calibrate | Fuse sibling toml; human pin | Suggestions only |

**Graft queue (after CT101 census — still not new organs):**

1. Outcome-linked utility — bump Q when a later takeaway cites/bonds a recalled fact (MemRL). No gym reward.
2. Dream as region rewrite — replacement set supersedes a working region (Auto-Dreamer). No second-model RL gym.
3. Named night labels — TRIAGE → CONSOLIDATE → AUDIT on distill/gate/soak. Docs/scheduler names only. No Observatory.
4. Verify rollback + typed supersession — quarantine/rollback + conflict tag (SuperLocalMemory / SleepGate). No multi-tenant RBAC product.

---

## Research questions (answer with evidence, not vibe)

**A. Utility (active bet)**

- Does living MCP/search on CT101 actually order by `utility_score` in `/opt/gzmo/current` (not only telescope `main`)?
- What are `recall_ge1`, `recall_ge3`, `utility_positive` on the living vault tonight? SSH fail = INCONCLUSIVE.
- Where does Q still increment from glance/search rather than later takeaway cite/bond? Close that loop or document the gap as the next *candidate*.

**B. Night metabolism**

- Is dream compaction a **superseding rewrite** with provenance, or append-only noise?
- Does the gate emit a typed supersede/refuse the operator can audit?
- Immune forget: plan-only vs apply on living — which is true, and is apply still lab?

**C. Eval honesty**

- Keep-quality soak: `honest_nights` vs 3? Spacing HOLD?
- Do not replace soak with LoCoMo/LongMemEval as living GREEN. Borrow-eval only.

**D. Airgap / attach**

- Daemon owns `vault.db`. CLI/MCP attach via socket. No ECONNREFUSED fallback that opens the living vault (ADR-0006).
- Core path must not require OpenRouter.

---

## Method

```text
Sense (lineage-watch + living census)
  → Rank (opportunity rubric; USP filter)
    → at most one candidate bet
      → if it is the active bet: implement one loop
        → cargo test / clippy / targeted scripts
          → commit → push → PR → CI green
            → stop with PR URL or blocker
```

- Prefer side-effect of real work (takeaway on close). No memory-gym Cursor chats.
- Fail closed: no stubs, no fabricated joules, no hybrid recall without embeddings, no synthetic soak.
- `TMPDIR=/home/mw/.cache/tmp` on this workstation (tmpfs fills).
- Deploy to CT101 only via mutex-gated path (`scripts/living-host-mutex.sh`, `docs/CT101_DEPLOY.md`).

```bash
# telescope
cd /home/mw/gzmo_full
export TMPDIR=/home/mw/.cache/tmp
bash scripts/opportunity-sense.sh
bash scripts/felt-use-depth.sh          # living census or INCONCLUSIVE
bash scripts/brain-feed-check.sh        # nutrient vs theater
# LIVING_GATE_SKIP_TAKEAWAY=1 bash scripts/keep-quality-gate.sh   # if CT101 reachable

cargo test
cargo clippy --all-targets -- -D warnings
```

---

## Forbidden this sitting

- Installing or greening OKForge / `/observatory` / wiki-push timers
- Starting `gzmo serve` on the telescope
- Publicizing `herdr` / `okforge`
- New organ crates, second bet `active`, Unpark theater waves as “next strengthen”
- Overnight weight updates, energy-as-USP, stitcher OS, `eml-core` productization
- Ecosystem tour of 40 memory startups

---

## Output (leave these artifacts)

| Artifact | Role |
|----------|------|
| PR **or** `research/opportunities/<id>.md` (`candidate`) | The one next move |
| Short harvest delta under `research/lineage-watch/` only if a paper moved a field | Cite + steal + non-steal |
| Chat closer | PR URL, or `INCONCLUSIVE` + missing SSH/census, or blocker (secrets / dual-writer / doctrine) |

Do not write a second manifesto. Do not revive Observatory as the research object.

---

## Brutal test

If the morning vault on CT101 would be unchanged by your PR, you built costume. Stop and pick a different loop.
