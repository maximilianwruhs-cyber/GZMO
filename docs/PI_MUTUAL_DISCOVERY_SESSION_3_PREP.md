# Session 3 Prep — Mutual Discovery (verified baseline)

**Prerequisite:** Read [`PI_MUTUAL_DISCOVERY_VERIFIED_FINDINGS.md`](./PI_MUTUAL_DISCOVERY_VERIFIED_FINDINGS.md) — L1–L5 are **audited ground truth**. Session 3 extends the graph (L6+), does not re-derive L1–L5.

---

## 1. Preflight

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
git checkout feat/context-compress-headroom
systemctl --user is-active gzmo-daemon && ./target/release/gzmo mentor ping

# Optional: distill session 2 before starting session 3
~/Projects/_foundation-audit/survey_GZMO/scripts/pi/distill_latest_pi_session.sh
```

**New Pi session required.**

---

## 2. What we learned from sessions 1–2

| Works | Fails |
|-------|-------|
| Short **Pi:/GZMO:/LINK:** beats | Re-printing locked LINKs |
| One search → one mentor | Meta-planning ("The user wants…") |
| Human says `crucial` / `halt` | Pi confirmation essays after `crucial` |
| Discovery of real architecture | Expecting visible dialogue every turn without labels |

**Session 3 mode:** **Human-led beats 1–3**, mentor optional beat 4 only.

---

## 3. Verified baseline (reference L1–L5 only)

Do not reprint. See verified findings doc for code citations.

| ID | One line |
|----|----------|
| L1 | Mythos (wiki) → SOUL.md hot-reload identity |
| L2 | SOUL persona + chaos temperature + MentorTeach feedback |
| L3 | Wiki honeypot research ≠ runtime vault→honeypot pipeline |
| L4 | PedagogyOrchestrator parallel path vs AgentLoop (ops/tools) |
| L5 | Tool gap closed by Pi JSONL → session_distill → vault → honeypot; EDF is parallel |

---

## 4. Session 3 goals — L6–L8

| Target | Territory | Seed question |
|--------|-----------|---------------|
| **L6** | Synapse vs memory | Is `events.jsonl` telemetry the same as vault recall? |
| **L7** | Synapse + chaos | Does `MentorTeach` / `chaos.rho_telemetry` appear on bus? |
| **L8** | Routing economics | `TaskKind::PedagogyInternal` vs `Chat` — who uses Prime? |

Pick **two** of three. Stop after **L7** or **L8** → distill.

---

## 5. Session 3 opener (paste into Pi)

```
MUTUAL DISCOVERY v3 — session 3

VERIFIED BASELINE L1–L5 (reference only — NEVER reprint, NEVER re-derive):
See ~/Projects/_foundation-audit/survey_GZMO/docs/PI_MUTUAL_DISCOVERY_VERIFIED_FINDINGS.md

GOAL: discover L6–L8 only.

## Mode: human-led beats 1–3
- Beats 1–3: NO gzmo_mentor_teach. I may paste evidence; you write LINK lines only.
- Beat 4: ONE mentor call allowed if I say "mentor ok".
- Max 4 beats total then halt.

## Output format (strict)
**Pi:** ≤3 sentences + one path
**GZMO:** (skip beats 1–3) OR one quoted line beat 4 only
**LINK:** L6/L7/L8: ...

## Forbidden
- Meta-planning ("the user", "I should confirm")
- Repeating any sentence twice
- Reprinting L1–L5
- Essays longer than 8 lines

If I say crucial → reply ONLY: "Ln locked."
If I say halt → list L6–L8 and ask distill.

Beat 1 territory: L6 synapse vs vault.
Search: read gzmo-core/src/synapse.rs event types OR tail data/Synapse/events.jsonl
Begin.
```

---

## 6. Human evidence cheatsheet (paste if Pi loops)

**L6 seed evidence (you can paste):**
```
EVIDENCE: data/Synapse/events.jsonl — append-only telemetry (mentor_teach, session_end, chaos.rho_telemetry).
Vault/honeypot — durable qualified facts via session_distill + qualifies_for_honeypot.
Pi: write LINK L6 only.
```

**L7 seed evidence:**
```
EVIDENCE: mentor_ipc emit_mentor_chaos_feedback; synapse MentorTeach / chaos events on bus.
Pi: write LINK L7 only.
```

**L8 seed evidence:**
```
EVIDENCE: pedagogy_bridge uses router.gateway(TaskKind::PedagogyInternal) for Diagnoser/Planner; tutor uses Chat gateway.
Pi: write LINK L8 only.
```

---

## 7. Close + distill scorecard

```bash
# In Pi after halt: gzmo_distill_pi
~/Projects/_foundation-audit/survey_GZMO/scripts/pi/distill_latest_pi_session.sh
./target/release/gzmo memory search "synapse vault" 2>/dev/null | head -15
```

| LINK | In vault after distill? |
|------|-------------------------|
| L6 | ☐ |
| L7 | ☐ |
| L8 | ☐ |
| Any L1–L5 from prior sessions | ☐ |

**Pass:** New L6+ captured OR prior links finally appear in vault.  
**Fail:** Loop noise only.

---

## 8. Files

| File | Role |
|------|------|
| `PI_MUTUAL_DISCOVERY_VERIFIED_FINDINGS.md` | Audited L1–L5 |
| `PI_MUTUAL_DISCOVERY_SESSION_3_PREP.md` | This doc |
| `~/.pi/agent/NEXT_SESSION.md` | Short opener |
| `~/.pi/agent/LOCKED_LINKS.md` | Registry |
