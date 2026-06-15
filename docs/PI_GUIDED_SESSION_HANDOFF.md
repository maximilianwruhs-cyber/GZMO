# Guided Pi Session — Operator Handoff

**Date:** 2026-06-12  
**Purpose:** One scripted Pi session to prove **mentor**, **topic-shift distill (V1)**, and **session_end distill (V2)** end-to-end.  
**Read this before starting Pi.** Keep a second terminal open for log checks.

---

## 0. Preconditions (run once)

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
git checkout feat/context-compress-headroom   # PR #23 not on main yet
systemctl --user is-active gzmo-daemon        # expect: active
./target/release/gzmo mentor ping             # expect: pong
./scripts/pi/smoke.sh                         # expect: OK
```

| Check | Expected |
|-------|----------|
| Branch | `feat/context-compress-headroom` (Pi platform not on `main` until PR #23 merges) |
| `topic_shift_enabled` | `true` in `gzmo.toml` `[session_distill]` |
| Pi extensions | `~/.pi/agent/settings.json` lists `gzmo-integration/index.ts` + `synapse-notifier.ts` |
| Embed API | VM200 `:8081` reachable (`./scripts/pi/test_topic_shift_distill.sh` passes) |

**Note (2026-06-12):** `cargo build` on branch HEAD may fail (`SkillContext` fields). The **existing** `target/release/gzmo` (Jun 12 build) is fine for this session. Do not rebuild until compile errors are fixed.

**Daemon footgun:** If `systemctl restart` fails with "already running", kill stale PID then restart:
```bash
cat /tmp/gzmo_daemon.pid | xargs kill 2>/dev/null; sleep 2
systemctl --user restart gzmo-daemon
```

---

## 1. Start Pi correctly

1. **Quit any old Pi session** completely.
2. **Start a new Pi session** (extensions load `gzmo.toml` at startup).
3. Optional: paste this into Pi's first message so it knows the playbook:

```
Read ~/Projects/_foundation-audit/survey_GZMO/docs/PI_GUIDED_SESSION_HANDOFF.md.
We are running a guided verification session. Use gzmo_mentor_* tools (not bash gzmo mentor).
Follow the phases below with me.
```

---

## 2. Phase A — Mentor smoke (5 min)

**Goal:** Pi → GZMO Socratic mentor over socket.

| Step | You say / do | Pi should use | Pass if |
|------|----------------|---------------|---------|
| A1 | "Run gzmo_mentor_ping" | `gzmo_mentor_ping` tool | Returns pong |
| A2 | "Run gzmo_mentor_status" | `gzmo_mentor_status` | `mentor=true`, learner shown |
| A3 | "Teach me what a symbolic link is in Linux" | `gzmo_mentor_teach` | Socratic answer (question back, not lecture) |

### Wrong vs right (do not let Pi loop)

| Wrong | Why |
|-------|-----|
| `bash gzmo mentor teach` | May open interactive **chat** REPL |
| `gzmo mentor_teach` as shell command | Invalid subcommand |
| MCP `gzmo_memory_search` for mentor | Mentor is **not** on memory MCP |
| Repeated bash retries | Use **one** `gzmo_mentor_teach` or `gzmo_mentor_reflect` |

---

## 3. Phase B — Topic A (topic-shift baseline) (10–15 min)

**Goal:** Build embedding baseline on one domain. Need **4+ user turns**, each **substantial** (100+ chars on baseline turn).

**Suggested topic A:** Kubernetes pod scheduling

Copy-paste these one at a time (edit if you prefer):

1. > I want to understand how Kubernetes schedules pods onto nodes. What factors does the scheduler consider beyond just CPU and memory requests?

2. > Explain node affinity and anti-affinity rules. When would I use requiredDuringSchedulingIgnoredDuringExecution versus preferredDuringSchedulingIgnoredDuringExecution?

3. > How do taints and tolerations interact with pod scheduling? Give a concrete example for a GPU workload pool.

4. > What happens when no node satisfies pod requirements? Walk me through pending state, events, and common operator mistakes.

5. > How would I debug a pod stuck in Pending due to resource quotas in a multi-tenant cluster?

**Do not switch topics yet.** Wait for Pi to finish each reply.

**Terminal B** (optional live watch):
```bash
tail -f ~/Projects/_foundation-audit/survey_GZMO/data/Synapse/events.jsonl | rg 'quest_complete|turn'
```

---

## 4. Phase C — Topic B (topic-shift trigger) (10–15 min)

**Goal:** Sharp unrelated switch → `topic_shift_distill` event + partial distill.

**Rate limits:** ≥3 turns and ≥10 minutes since last topic-shift distill.

**Suggested topic B:** Sourdough baking (unrelated to K8s)

Copy-paste:

1. > Switching topics completely: I am learning sourdough. What is the difference between hydration percentage and baker's percentage when building a levain?

2. > How does fermentation temperature affect flavor versus rise in a 75% hydration country loaf?

3. > When should I use autolyse, and how long is typical for high-protein bread flour?

4. > Explain oven spring and why steam matters in the first ten minutes of baking.

**After turn 4**, check synapse (wait ~30s):

```bash
tail -30 ~/Projects/_foundation-audit/survey_GZMO/data/Synapse/events.jsonl | rg topic_shift_distill
```

**Pass:** Line with `"event_type":"topic_shift_distill"` and fields `distance`, `startTurn`, `maxTurns`.

**If missing:** Messages may be too short, or rate limit not met — continue 2 more topic-B turns, wait 10 min, retry.

```bash
journalctl --user -u gzmo-daemon -n 40 --no-pager | rg -i 'distill|topic'
```

---

## 5. Phase D — Optional learn mode (5 min)

**Goal:** Multi-turn mentor session without bash loops.

| Step | You say | Tool |
|------|---------|------|
| D1 | "Start learn mode on Unix file permissions" | `gzmo_mentor_learn_start` with topic |
| D2 | Ask 2–3 follow-up questions | `gzmo_mentor_teach` (conversation history builds) |
| D3 | "End learn mode" | `gzmo_mentor_learn_end` |

---

## 6. Phase E — Manual distill (optional, 2 min)

Before quitting, optional one-shot:

> Distill this Pi session into GZMO memory.

Pi should call `gzmo_distill_pi` (no path = latest session).

Or in terminal:
```bash
~/Projects/_foundation-audit/survey_GZMO/scripts/pi/distill_latest_pi_session.sh
```

---

## 7. Phase F — End session (V2 / M5) (5 min)

1. **Quit Pi** (`/exit` or close session).
2. Within **90 seconds**, run:

```bash
# session_end on bus
tail -10 ~/Projects/_foundation-audit/survey_GZMO/data/Synapse/events.jsonl | rg session_end

# dedup state (path appears after successful distill)
cat ~/Projects/_foundation-audit/survey_GZMO/data/synapse-pi-distill.state.json

# daemon picked up session_end (60s poll) or Pi notifier spawned distill
journalctl --user -u gzmo-daemon -n 50 --no-pager | rg -i distill
```

3. **Vault check** (optional):
```bash
cd ~/Projects/_foundation-audit/survey_GZMO
./target/release/gzmo memory status
# or search vault for pi- session id from events.jsonl
```

**Pass:**
- [ ] `session_end` with `targetSessionFile` pointing to your `.jsonl`
- [ ] Distill log line or updated state file
- [ ] Second quit of same file does **not** duplicate (dedup)

---

## 8. Scorecard (fill in after session)

| Phase | Pass? | Notes |
|-------|-------|-------|
| A — mentor ping/teach | ☐ | |
| B — topic A (4+ turns) | ☐ | |
| C — topic_shift_distill | ☐ | distance: _____ |
| D — learn mode | ☐ | optional |
| E — gzmo_distill_pi | ☐ | optional |
| F — session_end distill | ☐ | |

---

## 9. If something fails

| Symptom | Fix |
|---------|-----|
| `mentor ping` → connection refused | `systemctl --user restart gzmo-daemon`; check `data/gzmo_mentor.sock` |
| Pi opens chat banner | Wrong tool path — use `gzmo_mentor_*` tools |
| No `topic_shift_distill` | Longer messages; wait 10 min; new Pi session |
| No `session_end` distill | `distillOnSessionEnd: true` in settings; daemon up; check `synapse-pi-distill.state.json` |
| Daemon crash on German text | UTF-8 fix in `kg_extract.rs` (commit `f4d038d`) — rebuild when compile fixed |

---

## 10. After successful session

1. **Merge PR #23** on GitHub when ready.
2. **Rotate GitHub token** if it was ever pasted in chat ([`GITHUB_PUSH.md`](./GITHUB_PUSH.md)).
3. Log results in scorecard §8; update [`REMAINING_WORK_STEP_BY_STEP_GUIDE.md`](./REMAINING_WORK_STEP_BY_STEP_GUIDE.md) V1/V2 checkboxes.

---

## Quick reference — Pi tools

| Tool | When |
|------|------|
| `gzmo_mentor_ping` | Health check |
| `gzmo_mentor_status` | Before teaching |
| `gzmo_mentor_teach` | Socratic Q&A |
| `gzmo_mentor_learn_start` / `_end` | Multi-turn learn session |
| `gzmo_distill_pi` | Manual vault distill |
| `gzmo_memory_search` | Honeypot recall (not mentor) |

**Skill doc:** `~/.pi/agent/skills/gzmo-integration/SKILL.md`
