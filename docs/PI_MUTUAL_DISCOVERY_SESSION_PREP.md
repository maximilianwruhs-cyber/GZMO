# Next Session Prep — Pi ↔ GZMO Mutual Discovery (v2)

**After session 1 (2026-06-12):** Good discoveries (SOUL, chaos, honeypot homonym). **Loops** on locked LINKs and mentor threads. This doc prepares session 2 with stricter rules and visible dialogue.

---

## 1. Before you open Pi (terminal)

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
git checkout feat/context-compress-headroom

# Kill stale daemon if restart fails
cat /tmp/gzmo_daemon.pid 2>/dev/null | xargs kill 2>/dev/null; sleep 2
systemctl --user restart gzmo-daemon
sleep 6
systemctl --user is-active gzmo-daemon    # active
./target/release/gzmo mentor ping         # pong
./scripts/pi/smoke.sh                     # optional
```

**Log terminal** (second window):

```bash
tail -f ~/Projects/_foundation-audit/survey_GZMO/data/Synapse/events.jsonl | rg 'mentor_|topic_shift|session_end'
```

---

## 2. Session 1 carryover (do not rediscover — build on these)

Mark these **LOCKED** at session start. Pi must **not repeat** them verbatim; only reference by number (L1, L2, L3).

| ID | LINK (summary) |
|----|----------------|
| **L1** | Mythos (wiki) —narrative-wraps→ SOUL.md (identity.rs hot-reload) |
| **L2** | SOUL —persona→ system prompt; chaos PulseLoop —modulates→ llm_temperature; MentorTeach —feeds-back→ chaos |
| **L3** | Wiki "cascading honeypot" (research) —homonym-not-same→ runtime honeypot (vault → qualify → SQLite → Qdrant); chaos = inquiry layer, honeypot = memory gate |

**Session 2 goal:** L4–L7 **new** links. Suggested territories (pick 2–3):

- Pedagogy orchestrator vs ops agent loop (who can write vault?)
- Synapse bus vs distill pipeline (telemetry vs memory)
- Prime routing vs PedagogyInternal (`TaskKind`)
- Wiki synthesis layer vs honeypot recall (emit-only vs qualified facts)

---

## 3. What changed in v2 (loop lessons)

| Session 1 problem | Session 2 rule |
|-------------------|----------------|
| Locked LINKs reprinted 6× | LINKs are **write-once**; refer as L1/L2/L3 only |
| Dialogue invisible | Every beat: **Pi:** / **GZMO:** labels |
| Mentor called without new evidence | **No mentor** until Pi shows a new path/snippet in the same beat |
| Meta-planning ("The user wants me to…") | **HALT trigger** — user says `halt`; Pi outputs nothing except LOCKED list |
| User says `crucial` on a LINK | Pi replies only: `L5 locked.` — no confirmation essay |
| Beat "complete" without Pi answer | Beat incomplete until **Pi:** answer + **LINK:** (or "no link yet") |
| Abstract chaos/SOUL loops | **Pivot** after 2 mentor calls on same thread |

---

## 4. Paste into Pi (session 2 opener)

Copy everything between the lines into **Pi chat** (not bash):

```
# MUTUAL DISCOVERY v2 — Pi & GZMO (session 2)

LOCKED from last time (reference only as L1/L2/L3 — NEVER reprint full text):
L1: Mythos → SOUL.md (identity hot-reload)
L2: SOUL persona + chaos temperature + MentorTeach feedback
L3: wiki honeypot research ≠ runtime honeypot; chaos=inquiry, honeypot=memory gate

Goal: discover L4–L7. Build relationships FROM locked links to new evidence.

## Hard rules
1. Label turns: **Pi:** / **GZMO:** / **LINK:**
2. LOCKED links: mention as L1/L2/L3 only — do not copy-paste them again.
3. Max ONE gzmo_mentor_teach per beat, only AFTER a new search/read snippet in that beat.
4. Never repeat any sentence twice. If you catch repetition, stop and say "LOOP DETECTED".
5. No meta-planning ("the user wants me to"). No bash gzmo mentor.
6. I may say: crucial | pivot | loop | halt | distill

## Beat shape
**Pi:** [one search/read result + path]
**GZMO:** [quote gzmo_mentor_teach response, trimmed]
**Pi:** [answer in own words]
**LINK:** L4: <A> —<rel>→ <B> | EVIDENCE: <path> | WHY: <phrase>
[wait for my crucial/pivot]

Start Beat 1 on territory: pedagogy orchestrator vs ops agent loop.
Search gzmo_wiki_search or read pedagogy_bridge.rs / PI_OPERATOR_GUIDE — ONE action first.
```

---

## 5. Your cheat sheet (human)

| Say | When |
|-----|------|
| **crucial** | Lock new LINK as L4, L5, … |
| **pivot** | New territory, no more mentor on current thread |
| **loop** | Pi repeated text — next message must have zero mentor calls |
| **halt** | End beats; list L1–Ln; ask distill |
| **distill** | Run gzmo_distill_pi |

**Max intervention:** 1 word per beat is enough.

---

## 6. Fallback mode (if loops return immediately)

**Human-led beats** — mentor disabled for first 4 beats:

```
Mentor tools DISABLED for beats 1–4.
I paste evidence; you write LINK lines only.
Beat 1 evidence: [paste path + snippet yourself]
Pi: propose LINK L4 only. No gzmo_mentor_teach.
```

Re-enable mentor at Beat 5 for one Socratic check on the graph.

---

## 7. Close session

When you say **halt** or **distill**:

```bash
# In Pi: gzmo_distill_pi
# Or terminal:
~/Projects/_foundation-audit/survey_GZMO/scripts/pi/distill_latest_pi_session.sh

# Verify
tail -15 ~/Projects/_foundation-audit/survey_GZMO/data/Synapse/events.jsonl | rg 'session_end|mentor_teach|distill'
```

### Distill scorecard

| Locked LINK | Appeared in vault / memory search after distill? |
|-------------|--------------------------------------------------|
| L1 | ☐ |
| L2 | ☐ |
| L3 | ☐ |
| L4 | ☐ |
| … | ☐ |

**Pass:** ≥2 relationships survive with correct evidence.  
**Fail:** generic summary only, or loop noise dominates.

---

## 8. Files to keep open

| File | Role |
|------|------|
| `docs/PI_MUTUAL_DISCOVERY_SESSION_PREP.md` | This doc |
| `~/.pi/agent/NEXT_SESSION.md` | Short opener |
| `~/.pi/agent/LOCKED_LINKS.md` | Living LINK registry |
| `docs/PI_GZMO_SOCRATIC_KNOWLEDGE_DIALOGUE.md` | Full experiment design |

---

## 9. Optional: distill session 1 first

Before session 2, capture whether session 1 loop transcript distilled usefully:

```bash
~/Projects/_foundation-audit/survey_GZMO/scripts/pi/distill_latest_pi_session.sh
./target/release/gzmo memory status
```

Note results in `LOCKED_LINKS.md` § distill notes.
