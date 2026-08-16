# Lineage from the telescope

**What this is:** a development guide you can write and follow on a workstation that does **not** hold the living mutex.  
**What this is not:** a second overnight brain, a 46-repo product plan, or permission to start `gzmo serve` here.  
**Living host:** CT101 (`/opt/gzmo`). **This host:** telescope — code, tests, PRs, thought.  
**Public SKU:** [GZMO](https://github.com/maximilianwruhs-cyber/GZMO) only.  
**Workshop:** private `little-tools-lab` + organs. Beat-gate, then graft **one loop**.

Written 2026-08-15 after the public zoo was taken private and archived. The storefront is gone. The animal is not.

**Why this file is the anchor (not a mood):** it is the operational constitution for a durable tool. Two rooms, one animal — the telescope is a *model* of the Keep (Ashby), never a second writer. Thought notes are constraints: Feynman forbids fake progress; Bateson forbids furniture rows; Beer (POSIWID) judges the morning vault; James makes the gate the organ of forgetting; Pasteur forbids unverified spark. Promote-by-loop is surgery, not a textbook rewrite. Fail-closed: no SSH → `INCONCLUSIVE` / RED, never a synthetic green. §8 is the lock on costumes.

---

> The first principle is that you must not fool yourself — and you are the easiest person to fool.  
> — Richard Feynman

**Thought.** We fooled ourselves with faces. Forty-eight public repos, five “live chains,” an EML sigmoid that was still `f64`. Honesty is not pessimism. It is the only way a lineage starts.

---

## 0. The sentence that is allowed to be proud

From [MACHINE.md](../MACHINE.md):

**Honeypot + verify + promote = GZMO.**  
GZMO is a distillation pipeline — not a chatbot with a memory attachment.

That is the uniqueness. Everything in this guide is how to evolve *that* without growing a costume.

> Information is a difference that makes a difference.  
> — Gregory Bateson

**Thought.** A vault row that never changes a later session is not information. It is furniture. Felt use is Bateson made operational: `recall_count` and `utility_score` are how a difference proves it mattered.

---

## 1. Two rooms, one animal

```text
TELESCOPE (this box)                    LAB (CT101, elsewhere)
─────────────────────                   ─────────────────────
edit gzmo-core / pieces                 sole overnight writer
cargo test, clippy                      vault / honeypot / dream / spark
open PR                                 takeaway, soak, keep-quality
imagine the next graft                  apply the graft
never gzmo serve                        living-host-mutex claim=ct101
```

> Every good regulator of a system must be a model of that system.  
> — W. Ross Ashby

**Thought.** The workshop is allowed to be a *model* of the Keep (fixtures, copied snapshots, beat-gates). It is not allowed to *be* the Keep. Ashby’s law is why we have two stacks. A lab that writes the living vault is not a regulator. It is a second animal.

### Telescope contract (non-negotiable)

| Allowed | Forbidden |
|---------|-----------|
| Product code in `gzmo-core` / `gzmo-cli` | `gzmo serve`, overnight scheduler |
| Unit tests, clippy, PRs | `--now` distill into a second vault |
| Private workshop clones, fixture smokes | Claiming living on this host |
| Design notes, this guide, mission cards | Publicizing `herdr` / `okforge` |
| Honest docs | Invented joules, canned stitcher as live gen, hybrid recall without embeddings |

If `ct101` does not resolve from here, you **cannot attest** keep-quality, felt-use census, or Brain Feed GREEN. You can still queue the patch.

---

## 2. What “really special” actually means

Not: an autonomic OS, energy-aware routing as USP, MIDI heartbeat, 46 mature checkmarks.

Yes: a **closed-set metabolism** on hardware you own.

```text
real work
  → session close --takeaway          (side-effect, not a gym)
    → session-distill                 (facts from a life that happened)
      → honeypot-gate                 (qualify or refuse)
        → vault / honeypot
          → felt use (glance / cited / bonded)
            → utility_score re-rank
              → spark-link (stale × importance × cosine, then verify)
                → rrf-recall
                  → dream / ripen / immune forget
                    → next real work
```

A LangChain app can store chat. It cannot, without copying this wiring, make **overnight refusal** (gate), **value-aware recall** (utility), **verified serendipity** (spark), and **promote-by-loop** (lab may only graft a winner) into one airgapped cron.

> The purpose of a system is what it does.  
> — Stafford Beer

**Thought.** POSIWID. If the public face was 48 repos, the purpose was publishing. If the living host writes one vault and the lab only grafts winners, the purpose is metabolism. Judge the stack by the vault in the morning, not by the catalog at night.

---

## 3. The workshop as dojo, not storefront

Private repos that still earn a place in the lineage:

| Lane | Repos | Role in the animal |
|------|--------|--------------------|
| Meta | `little-tools-lab` | Catalog, schemas, `cognition-smoke.sh`, beat-gates, CI |
| Organs | `honeypot-gate` `spark-link` `session-distill` `rrf-recall` | Distill → qualify → spark → recall |
| Calibration | `temp-bench` `config-fuse` `verify-suite` | Suggestion → `*-fused.toml` → **human pin** |
| Ops | `gzmo_skills` `vllm-blackwell-backend` | Units / GPU profiles — on the machine that has the silicon |

Parked even in private (do not grow unless a later bet names a call site): `code-stitcher`, `gzmo-os`, `Obolus`, `Obolus-Arena`, `evidence-locate` (pull when a smoke needs it).

`eml-core` lives inside GZMO as a paper crate. `vs_f64` lost (14× / 29× / 1200×). It is not an organ.

> I have made this longer than usual because I have not had time to make it shorter.  
> — Blaise Pascal

**Thought.** The 561 MB lab meta-repo is Pascal inverted: we had time to make it *larger*. The dojo you want is four organs and a smoke script. Clone `little-tools-lab` plus the four organs when you will edit a loop this month. Otherwise the Keep in `gzmo-core` is enough.

### Path resolution (do not invent a third tree)

```text
Telescope clone     /home/mw/gzmo_full
Lab GZMO clone      /home/gzmo/github-clone/GZMO
Lab LTL             /home/gzmo/github-clone/little-tools-lab
Living data         /opt/gzmo  (CT101 only)
Lite bootstrap      ~/.gzmo    (no overnight)
```

`gzmo-core` `assembly.rs` `lab_root()`:

1. `LITTLE_TOOLS_LAB_ROOT`
2. `$GZMO_CLONE_ROOT/little-tools-lab`
3. fallback `/home/gzmo/github-clone/little-tools-lab`

On this telescope, if you clone the workshop, **export the env**. Do not pretend the lab path exists here.

```bash
export GZMO_CLONE_ROOT=/home/mw/workshop   # or wherever you put the private clones
export LITTLE_TOOLS_LAB_ROOT=$GZMO_CLONE_ROOT/little-tools-lab
```

`GZMO_INSTANCE=next` is the only switch that allows lab backends. Unset or CT101 legacy → inline `gzmo-core`. That guard is the two-stack honesty. Do not bypass it to “just test living.”

---

## 4. The four rings (where a telescope may touch)

From [CONTINUOUS_UPGRADE.md](CONTINUOUS_UPGRADE.md):

| Ring | Name | Telescope | Lab (CT101) |
|------|------|-----------|-------------|
| 1 | Living nutrient | Design takeaway/felt-use **code**. Do not run `--now`. | Real sessions, close ritual, soak |
| 2 | Suggestions | Improve fuse/bench **code**. Never clobber live toml. | Human pins `*-fused.toml` |
| 3 | Lab → production | Write the piece, fixture smoke, PR | Beat-gate vs snapshot, promote **one loop** |
| 4 | Portfolio craft | Re-score uniqueness. Deepen S/A kernels. Do not invent organs. | Monthly, after soaks |

**Active ship (one at a time):** [`felt-use-mass-growth`](../research/opportunities/felt-use-mass-growth.md) (score 23).

Schema v8/v9 `honeypot.utility_score`, `reinforce_by`, and `ORDER BY utility_score` are already in `gzmo-core`. Remaining done-when is **mass from real sessions** on the living vault. That measurement is lab-only (`scripts/felt-use-depth.sh` SSHes `ct101`).

**Telescope half of that bet:** tests that utility order is real; MCP search touches felt-use; no path that green-passes empty recall; docs that say census is RED when SSH fails — not “0 facts, all good.”

**Field watch (does not start a second bet):** [research/lineage-watch/](../research/lineage-watch/README.md) — local agentic memory is the field; harvest every *rule*, reject their *SKU*. Graft queue is four organ-mapped steals after CT101 census. Latest: [sota-2026-08-15.md](../research/lineage-watch/sota-2026-08-15.md).

---

## 5. Loop book — how to evolve each organ

Work a **loop**, not a repo. One PR, one beat-gate, one promote.

### 5.1 Distill — `session-distill`

**Job.** Turn a session that already happened into candidate facts. No gym sessions whose only job is to feed the vault ([BRAIN_FEED.md](BRAIN_FEED.md)).

**In-tree.** Session close / takeaway enqueue. Scheduler shells lab recipe when `GZMO_INSTANCE=next`.

**Workshop.** `session-distill` piece + `session-to-dream.sh`.

**Evolve toward.** Fewer, denser facts. Origin tags that survive into honeypot (`session_distill` vs `honeypot` vs `librarian_extract`). If origin is all `SessionDistill` and honeypot-origin is zero, the identity sentence is still a wish — that was the 2026-07-16 gap in the uniqueness thesis. The special stack has **honeypot-origin rows**.

**Thought.** Distill is grief work. You throw away the chat to keep the scar. If you keep the chat, you built a log. If you keep the scar and it gets recalled next week, you built a Keep.

> We do not remember days, we remember moments.  
> — Cesare Pavese

### 5.2 Gate — `honeypot-gate`

**Job.** Qualify or refuse. Lifecycle classifier. Goldens must match GZMO (`scripts/lifecycle-goldens-check.sh`, `gzmo-core` `lifecycle.rs`).

**Evolve toward.** Refusal as a first-class event (why rejected, decay class). Immune: low `utility_score` + zero recall is allowed to die. A Keep that cannot forget is a hoard.

**Thought.** The gate is the only moral organ. Spark and dream are appetite. The gate is shame — useful shame.

> The art of being wise is the art of knowing what to overlook.  
> — William James

### 5.3 Spark — `spark-link`

**Job.** Not random recall. Triangle: `stale_sweetness` × importance × recent cosine, then hypothesize + **verify**. In-tree cousin: `gzmo-core` `spark.rs` / `spark_lineage.rs`.

**Evolve toward.** Verified links that promote back (serendipity apply, dry-run default). Lineage you can read in the morning. If spark cannot name the two facts and the verify step, it is a mood.

**Thought.** Serendipity without verify is a slot machine. The unique move is *stale* — the fact that waited — meeting *recent* under a score you can recompute.

> Chance favors the prepared mind.  
> — Louis Pasteur

The prepared mind, here, is a honeypot with utility and a triangle that prefers the mid-aged scar.

### 5.4 Recall — `rrf-recall`

**Job.** Fusion. In-tree search must `ORDER BY utility_score DESC, … recall_count` and **touch** felt-use (`FeltUseKind::Glance` / `Cited` / `Bonded`).

**Evolve toward.** Utility that moves because a later session used the fact, not because a bench looped search. Dual-gate ripen: `recall≥3` among felt facts. Thin depth → HOLD, not fake Ready.

**Thought.** RRF is plumbing. Utility is politics. Who gets to be remembered? The organism answers: whoever changed a later hour.

### 5.5 Calibrate — `temp-bench` → `config-fuse`

**Job.** Sweep, fuse, write **sibling** `*-fused.toml`. Never clobber live `[assembly]` / `[memory]`. Operator pins.

Lorenz→params is a signature **only** as a suggestion that a human promotes ([UNIQUENESS_THESIS.md](UNIQUENESS_THESIS.md) claim 4). Chaos is not an overnight PulseLoop (ADR-0002).

**Thought.** Calibration without a human is a coup. The fused file is a letter. The living toml is the law.

> In the beginner’s mind there are many possibilities, but in the expert’s there are few.  
> — Shunryu Suzuki

We want the expert’s few: one writer, one pin, one loop.

---

## 6. The graft protocol (promote-by-loop)

This is the culture that makes the workshop a lineage.

```text
1. Name the loop          distill | gate | spark | recall | fuse
2. Change one piece       private repo or gzmo-core inline
3. Fixture smoke          cognition-smoke.sh --fixture
                          beat-gate.sh --loop <name> --fixture
4. Snapshot smoke         copy of living vault, never the live file
5. PR from telescope      GZMO and/or the piece
6. On CT101               beat-gate vs incumbent + operator ack
7. Promote that loop      not the whole host
8. Soak                   keep-quality-soak.sh — honest nights, ≥18h apart
```

Whole-host cutover still needs `CUTOVER_APPROVED=1`. Lab backends stay dead unless `GZMO_INSTANCE=next`.

**Thought.** A graft is surgery. You do not replace the patient with the textbook.

### Mission card (copy into the next agent)

```text
Bet id:     felt-use-mass-growth
Title:      Utility-ordered honeypot that gains mass from real sessions
Why rare:   Memory policy evolves without weight updates; warehouse RAG cannot
Brain:      Facts that were useful rise; junk sinks; ripen stays honest
Done when:  Living census (CT101) shows rising recall≥3 / utility mass;
            Brain Feed GREEN; no memory-gym
Telescope:  tests + fail-closed gates when SSH missing; no gzmo serve
Lab:        felt-use-depth.sh, soak, takeaway from real work
```

Full template: [templates/MISSION_CARD.md](templates/MISSION_CARD.md).

---

## 7. What you can write this week without CT101

These are real, falsifiable, and legal on a telescope:

1. **Felt-use honesty in code** — every MCP/search hit calls `felt_use::touch`; no swallowed `store_text` on the product path; tests that empty recall does not look like GREEN.
2. **Gate when SSH is gone** — `felt-use-depth.sh` / `keep-quality-gate.sh` print `INCONCLUSIVE` / RED with `ct101 unresolved`, never a zeroed success.
3. **Goldens sync** — `lifecycle-goldens-check.sh` still agrees between `honeypot-gate` and `gzmo-core` after a loop change.
4. **One organ PR** — e.g. spark lineage field that `spark_lineage.rs` already tries to recover; or distill origin that can become honeypot-origin.
5. **This guide** — revise when a graft lands. Do not add a new organ to the table without a bet that passes the opportunity rubric (`score ≥ 18`, `brain_profit ≥ 3`, `usp_fit ≥ 4`).

Verify locally:

```bash
cargo test -p gzmo-core --lib
cargo clippy -p gzmo-core --all-targets -- -D warnings
# workshop, only if cloned:
# bash $LITTLE_TOOLS_LAB_ROOT/scripts/cognition-smoke.sh --fixture
```

On the lab, later:

```bash
bash scripts/felt-use-depth.sh
bash scripts/brain-feed-check.sh
LIVING_GATE_SKIP_TAKEAWAY=1 bash scripts/keep-quality-gate.sh
bash scripts/keep-quality-soak.sh --summary
```

---

## 8. Costumes we do not put back on

| Costume | Why it stays off |
|---------|------------------|
| Stitcher Phases 2–5 as “autonomic pipeline” | Library tests. No board worker. Canned recipe ≠ live generation. |
| `eml-core` as honeypot math | Worse float. Paper primitive only. |
| Energy routing as USP | RAPL or refuse. TDP×time is a lie. |
| HSP / pantheon / Observatory | Glass. Does not feed the vault. |
| 48 public faces | Purpose becomes publishing (Beer). |
| Second overnight writer | Kills the animal. |
| Memory-gym chats | Inflates felt use; ruins ripen honesty. |

> Perfection is achieved, not when there is nothing more to add, but when there is nothing left to take away.  
> — Antoine de Saint-Exupéry

**Thought.** We already took away the zoo. The work now is not to add a universe. It is to make four organs and one night undeniable.

---

## 9. Inspirational field notes (keep these in the room)

**On building an animal, not a platform**

> A system is a set of variables sufficiently isolated to stay in a defined relation.  
> — W. Ross Ashby (paraphrase of the essential variables)

Our essential variables: one writer, honeypot quality, felt utility, airgap-honest core path. If a feature does not move those, it is not lineage.

**On overnight**

> The night is a time of rigor.  
> — Gaston Bachelard

Dream and spark are allowed to be strange. They are not allowed to be unmeasured. Morning status (`DREAMS.md`, soak log, felt-use census) is the rigor.

**On craft**

> All craft is local.  
> — Richard Sennett, *The Craftsman* (spirit of)

The local is CT101’s vault and this telescope’s diff. Global is the costume we archived.

**On memory**

> He who cannot draw on three thousand years is living from hand to mouth.  
> — Goethe

A Keep is how a single operator draws on *his* years. Not three thousand. Enough that next Tuesday is not hand to mouth.

**On the telescope**

> We are the local embodiment of a Cosmos grown to self-awareness.  
> — Carl Sagan

Do not inflate that. You are a man with a lab in another room and a clone on this desk. The self-awareness that matters is: *which host writes tonight?*

---

## 10. Picture of the stack in twelve honest months

If the lineage holds:

- Public GitHub still shows **one** repo.
- CT101 (or a later mutex holder) has honeypot-origin facts, rising `recall≥3` share, utility-ordered search, spark lineage you can read.
- The workshop still has four organs and a smoke. Maybe one new piece, born from a scored bet, not a catalog hunger.
- Calibration still ends in a human pin.
- No second writer. No energy fiction. No stitcher OS.
- A stranger can install **lite** MCP in five minutes and never touch living.
- You can point at a morning soak and say: the animal ate, refused, sparked, and forgot — on a box I own.

That is special. It is also small. Small is the point.

---

## 11. How to use this file

Operating cadence for every agent or human on this telescope:

1. **§§0–2** — identity (`honeypot + verify + promote = GZMO`); two rooms, one animal.
2. **§§5–6** — pick **one** loop and graft it; no whole-pipeline rewrites.
3. **§7** — ship falsifiable telescope wins when CT101 is out of reach.
4. **§8** — refuse costumes and scope creep on sight.
5. Update the active ship line when opportunity-discovery changes the bet.
6. Do not add a workshop repo to §3 without saying which loop it serves.
7. Refresh [lineage-watch](../research/lineage-watch/README.md) monthly (or when a paper moves a field). A watch item is not a ship bet.

Related: [MACHINE.md](../MACHINE.md) · [ADR-0004](ADR-0004-airgap-living-usp.md) · [ADR-0005](ADR-0005-flywheel-over-frozen-topology.md) · [BRAIN_FEED.md](BRAIN_FEED.md) · [OPPORTUNITY_DISCOVERY.md](OPPORTUNITY_DISCOVERY.md) · [CONTINUOUS_UPGRADE.md](CONTINUOUS_UPGRADE.md) · [UNIQUENESS_THESIS.md](UNIQUENESS_THESIS.md) · [SPINE_FOCUS.md](SPINE_FOCUS.md) · [lineage-watch](../research/lineage-watch/README.md)
