# Sokratischer Dialog × Forum Romanum — Drei Modi

Operativer Guide für Modus **A** (Sokratik), **B** (Forum Romanum) und **C** (Hybrid) in GZMO.

Siehe auch: [`FORUM_ROMANUM_SCHEMA.md`](./FORUM_ROMANUM_SCHEMA.md), [`SYNAPSE_EVENT_OWNERSHIP.md`](./SYNAPSE_EVENT_OWNERSHIP.md).

## Übersicht

| Modus | Akteure | Bus-Events | Runtime |
|-------|---------|------------|---------|
| **A** Sokratik | Pi ↔ `gzmo_mentor_teach` | `mentor_teach` | Mentor-Socket, pi-mentor-discovery |
| **B-Lite** Forum | Prometheus ↔ Epimetheus (ein Modell) | — | `pi -p` + Research-Prompt |
| **B-Full** Forum | Prometheus ↔ Epimetheus (pi-crew) | `agent.message`, `proposal.*` | `handleTeamTool` + forum-romanum-bridge |
| **C0** Hybrid | A dann B (Prompt) | — | `socratic-forum-hybrid.md` |
| **C-Full** Hybrid | A → Handoff JSON → B | `mentor_teach` + `agent.message` | `run-hybrid-mode-c-full.sh` |

## Phase 0 — Voraussetzungen

```bash
chmod +x ~/gzmo_skills/scripts/verify-three-modes-prereqs.sh
~/gzmo_skills/scripts/verify-three-modes-prereqs.sh
```

Erwartet: `gzmo mentor ping`, Bridge installiert, Bus vorhanden, pi-crew mock-Integration, `.crew/`-Scaffold.

## Modus A — Sokratischer Dyade

**Definition:** Pi scoutet; GZMO antwortet elenchisch über den Mentor-Socket.

```bash
# Smoke
gzmo mentor ping
gzmo mentor teach "Was ist ein sokratischer Dialog?"

# Vollständiger Discovery-Zyklus (produktiv)
cd ~/gzmo_skills && ./scripts/start-pi-mentor-discovery-session.sh

# AUTO Low-Tension
gzmo chaos skill ops AUTO   # toggle
# Daemon triggert auto-socratic-discovery-cycle.sh bei τ < threshold

# Verifikation
~/gzmo_skills/scripts/verify-mode-a.sh
```

**Research (nur Dialog):**

```bash
ROUNDS=6
pi -p "$(sed "s/{{ROUNDS}}/$ROUNDS/" \
  "$HOME/gzmo_skills/prompts/research/socratic-dialogue-only.md")"
```

Artefakte: `gzmo_skills/prompts/research/socratic-dialogue-only.md`

## Modus B — Forum Romanum

### B-Lite (thematisches Gespräch, kein Bus)

```bash
ROUNDS=8
pi -p "$(sed "s/{{ROUNDS}}/$ROUNDS/" \
  "$HOME/gzmo_skills/prompts/research/socratic-forum-interaction.md")" \
  | tee /tmp/b-lite.txt

~/gzmo_skills/scripts/archive-research-transcript.sh /tmp/b-lite.txt
```

### B-Full (pi-crew + Bus)

Scaffold: `survey_GZMO/.crew/{agents,teams,workflows}/`

Runner: `gzmo_skills/scripts/run-forum-romanum-mode-b-full-runner.ts` (via `tsx`, nicht `pi -p /team-run`)

```bash
# Mock smoke (kein LLM)
PI_TEAMS_MOCK_CHILD_PI=success \
  ~/gzmo_skills/scripts/run-forum-romanum-mode-b-full.sh

# Live (LLM + Bridge in worker Pi)
FORUM_GOAL="Sokratik vs Forum Romanum" \
  ~/gzmo_skills/scripts/run-forum-romanum-mode-b-full.sh

~/gzmo_skills/scripts/verify-forum-bus-thread.sh
```

**Interaktiv (Debugging):** frisches `pi` → `/team-run forum-romanum-research "<goal>"` — **nicht** `pi -p /team-run`.

**Anti-Pattern:** `pi -p /team-run` → stale `ExtensionRunner` ctx.

## Modus C — Hybrid

### C0 (Prompt-only)

```bash
ROUNDS=8
pi -p "$(sed "s/{{ROUNDS}}/$ROUNDS/" \
  "$HOME/gzmo_skills/prompts/research/socratic-forum-hybrid.md")"
```

### C-Full (Handoff + Forum)

```bash
~/gzmo_skills/scripts/run-hybrid-mode-c-full.sh
```

Handoff-Schema: `gzmo_skills/schemas/socratic-forum-handoff.schema.json`

Phase 1 allein:

```bash
~/gzmo_skills/scripts/run-hybrid-phase1-socratic.sh
~/gzmo_skills/scripts/validate-handoff-json.py \
  ~/gzmo_skills/data/research/handoffs/*-socratic_brief.json
```

Phase 2 mit Handoff:

```bash
HANDOFF_PATH=~/gzmo_skills/data/research/handoffs/<id>-socratic_brief.json \
  ~/gzmo_skills/scripts/run-forum-romanum-mode-b-full.sh
```

## pi-crew Ressourcen (Projekt)

| Datei | Rolle |
|-------|-------|
| `.crew/agents/prometheus.md` | Proposer |
| `.crew/agents/epimetheus.md` | Critic / Synthesize |
| `.crew/teams/forum-romanum-research.team.md` | Team |
| `.crew/workflows/forum-romanum-debate.workflow.md` | 4 Debattenrunden + Synthese |

## Verifikation (Querschnitt)

```bash
# F5 oscillation + correlation_id chain
~/gzmo_skills/scripts/discovery-probes/probe-pedagogy-oscillation.sh

# Layer-1 learning predicates (extern, Synapse-firewall-konform)
~/gzmo_skills/scripts/verify-learning-after-oscillation.sh --oscillation-id <UUID>

# Strict CLI wait (bus completeness)
gzmo pedagogy oscillate start --wait --strict --json

~/gzmo_skills/scripts/test-three-modes-integration.sh
cd ~/Projects/_foundation-audit/survey_GZMO
./scripts/pi/test_forum_romanum_schema.sh
```

**Defaults:** Hybrid C-Full nutzt `FORUM_MODE=b-lite` bis B-Full stabil; Handoff v2 setzt `GZMO_HANDOFF_PATH` für Oscillation-Ingest.

## Erfolgskriterien

### Modus A
- [ ] `gzmo mentor ping` → pong
- [ ] Discovery strict: ≥1 `gzmo_mentor_teach` pro Zyklus
- [ ] Bus: `mentor_teach` mit `session_id`

### Modus B
- [ ] B-Lite: `DIALOGUE_COMPLETE`, keine Spark/Dream-Verwechslung
- [ ] B-Full: pi-crew run completed (mock oder live)
- [ ] B-Full live: `agent.message` threaded auf Bus

### Modus C
- [ ] Handoff JSON validiert
- [ ] Forum-Goal enthält `open_questions`
- [ ] Transcript unter `data/research/transcripts/`

## Bezug zum Ökosystem

- **pi-mentor-discovery** = Modus A produktiv
- **Kurator fix-from-discovery** = Remediation (Epimetheus), **nicht** Modus C
- **forum-romanum-bridge** = Bus-Mapping für pi-crew hooks
- **Research prompts** = konzeptuelle Arbeit ohne Ops-Scope
