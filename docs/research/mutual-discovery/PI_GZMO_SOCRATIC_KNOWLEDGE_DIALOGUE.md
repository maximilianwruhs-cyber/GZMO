> **Recovered 2026-07-19** from `origin/feat/context-compress-headroom`.
> **Research archive — not living CT101 doctrine.** See [MUTUAL_DISCOVERY_THEATER.md](../../MUTUAL_DISCOVERY_THEATER.md).
> Inventory: [LOST_KNOWLEDGE_INVENTORY.md](../../LOST_KNOWLEDGE_INVENTORY.md).

# Mutual Discovery Dialogue — Pi ↔ GZMO

**Experiment:** Can two agents **discover the knowledge base together** — find what matters, name **relationships** between ideas — and produce something worth distilling?

| Best case | Worst case |
|-----------|------------|
| They surface load-bearing topics you forgot were there, and map **crucial links** (A enables B, B contradicts C, C is legacy) | They circle the same question without new evidence; mentor calls repeat; no LINK lines |

**Paste §2 into Pi** (chat, not bash). You are the third voice: **pivot**, **yes that's crucial**, or **stop looping**.

---

## 1. Preflight

```bash
cd /opt/gzmo/current (ritual/lab clone; see CT101_PATH_AUTHORITY.md)
systemctl --user is-active gzmo-daemon && ./target/release/gzmo mentor ping
```

New Pi session. Tools: `knowledge_search`, `gzmo_memory_search`, `gzmo_wiki_search`, `read`, `gzmo_mentor_teach`, `gzmo_mentor_reflect`, `gzmo_distill_pi`.

---

## 2. Master prompt — paste into Pi

```
# MUTUAL DISCOVERY SESSION — Pi & GZMO walk the knowledge base together
# We are not teacher and student. We are two minds trying to see the same map.

You are Pi. GZMO speaks only through gzmo_mentor_teach (and gzmo_mentor_reflect if stuck).
I am present. I may say: pivot | crucial | loop | distill | stop.

## Shared goal
Discover together:
- What topics are **actually important** (load-bearing, not decorative docs)
- **Relationships** between topics (depends-on, contradicts, supersedes, enables, rhymes-with)
- What the knowledge base *implies* but never says out loud

No pre-planned syllabus. Follow curiosity and friction.

## How you discover together (one "beat")
1. **Pi** searches or reads ONE new thing (knowledge_search | gzmo_memory_search | gzmo_wiki_search | read).
   Bring back a short quote or path — something concrete from the stacks.
2. **Pi** tells GZMO what you found and what confuses or excites you (2–5 sentences).
3. **Pi** calls gzmo_mentor_teach with that material. Ask GZMO not for a lecture but:
   "What relationship might exist here that we haven't named yet? What would you probe next?"
4. **GZMO** responds (Socratic — questions and connections, not answer keys).
5. **Pi** answers GZMO in your own words — agree, disagree, or say "that suggests a link to …"
6. **Together** try to write one line if you found something real:
   LINK: <A> —<relationship>→ <B> | EVIDENCE: <path or snippet> | WHY IT MATTERS: <one phrase>
7. **I** react: crucial (keep) | pivot (new area) | loop (you're circling) | meh (skip)

Repeat beats. Let important topics **emerge**. Do not force a list upfront.

## Relationship types (use these words in LINK lines)
- enables / blocks
- contradicts / supersedes
- mirrors (same idea, two stores)
- legacy-vs-active
- operator-invariant (if wrong, prod breaks)
- pedagogy-vs-ops (teaching path vs execution path)
- surprising-neighbor (ideas that rhyme but live in different folders)

## Anti-loop rules (worst case prevention)
- **New evidence every beat.** No second gzmo_mentor_teach on the same snippet without a new search/read.
- **Max 3 mentor calls on the same topic thread.** Then pivot or I will say "pivot".
- If you feel stuck or GZMO repeats: ONE gzmo_mentor_reflect call with your current LINK drafts, then change territory.
- If two beats produce no new LINK and no new path: Pi must search a **different** tool (e.g. switched memory_search → wiki_search).
- Never bash `gzmo mentor`. Never retry mentor in a loop hoping for a different answer — chaos temperature changes, but **evidence** must change too.
- Speak your LINK lines aloud to me. Silent "thinking" doesn't count for distill.

## Opening (no plan — true discovery)
Start blind:
1. Pi: "I'm going to ask the knowledge base what it thinks is central." Run a broad gzmo_memory_search or knowledge_search on "architecture" OR "invariants" OR "how GZMO works" — your choice.
2. Share the most surprising hit with GZMO.
3. First dialogue beat begins from **that** surprise, not from a checklist.

## Close (after ~8–15 beats or when I say stop)
1. Pi reads back all LINK lines marked **crucial** by me.
2. Pi names: top 3 topics that turned out important, top 3 relationships, 1 gap ("the KB is silent about …").
3. Ask me: distill? If yes → gzmo_distill_pi.

Begin beat 1 now. Discover together.
```

---

## 3. Shorter creative framing — "Two cartographers in fog"

```
Pi and GZMO are cartographers. The knowledge base is fog.

Pi has the compass (search, read). GZMO has the habit of asking "if that's true, what else must be true?"

Walk together. Every time Pi finds a landmark (a doc, a vault hit, a wiki page), GZMO asks what it connects to. You write LINK lines. I call out when you've found a mountain vs a molehill.

If you walk in circles, reflect once (gzmo_mentor_reflect) and I'll say pivot.

Start: search for something that feels like a center of gravity. Show GZMO. Ask what it connects to. Go.
```

---

## 4. What success looks like (for distill)

After session + `gzmo_distill_pi` (or session_end):

| Question | Good sign | Bad sign |
|----------|-----------|----------|
| Did they find **topics** without you assigning them? | Emergent themes in transcript | Only talked about what you named |
| Did they name **relationships**? | Several `LINK:` lines with evidence | Vague agreement, no A→B |
| Did distill capture links? | Vault facts mention dependencies/contradictions | Generic summary of "we discussed memory" |
| Loop? | ≤1 reflect call, pivots worked | Same mentor question 4+ times |

---

## 5. Your voice (minimal intervention)

| Say | When |
|-----|------|
| **crucial** | A LINK line is worth keeping |
| **pivot** | Force new territory |
| **loop** | They're circling — triggers reflect + move |
| **meh** | Skip without guilt |
| **distill** | End and engrave |
| **stop** | Session over |

Stay quiet when they're genuinely discovering. Interrupt on loops early.

---

## 6. Example beat (relationship, not quiz)

**Pi** (after `gzmo_wiki_search("honeypot")`): Wiki says honeypot is curated recall; operator guide says Qdrant `knowledge` collection is legacy.

**Pi → GZMO:** "Two retrieval layers — wiki synthesis vs Qdrant collections. What relationship are we missing between honeypot and vault?"

**GZMO:** "If honeypot mirrors vault, what breaks when someone queries the legacy collection anyway?"

**Pi:** "We'd double-stale — mirror drift plus a deprecated index. That's an operator invariant, not a doc typo."

**LINK:** `honeypot —mirrors→ vault SQLite` | `knowledge collection —legacy-supersedes→ honeypot` | EVIDENCE: PI_OPERATOR_GUIDE | WHY: wrong query path serves ghost data

**You:** `crucial`

---

## 7. Verify distill

```bash
/opt/gzmo/current (ritual/lab clone; see CT101_PATH_AUTHORITY.md)/scripts/pi/distill_latest_pi_session.sh
tail -10 /opt/gzmo/current (ritual/lab clone; see CT101_PATH_AUTHORITY.md)/data/Synapse/events.jsonl | rg session_end
```

Compare **crucial LINK lines** to new vault facts. That's the experiment.
