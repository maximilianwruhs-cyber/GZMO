# Infrastructure Remediation — Completion Status

**Updated:** 2026-06-14  
**Repo:** `survey_GZMO`

## Workstreams (all implemented)

| ID | Status | Deliverables |
|----|--------|--------------|
| **A** Container network | Done | `container-lan-forward.sh`, health perspective |
| **B** Feedback loop | Done | audit log, `feedback-audit`, `chaos.feedback_drained` |
| **C** Low-tension dialogue | Done | threshold=18, idle trigger, KG openings, **C4 Neo4j persist** |
| **D** Display visibility | Done | 6 synapse event types + **D3 recent discoveries panel** |
| **E** Prerequisite graphs | Done | skill_prereqs, readiness gate, **`export-prerequisite-graph.py`** |
| **F** Honeypot rejection | Done | reject log, **`honeypot review` queue**, **F2 histogram script** |

## New commands / artifacts

```bash
# C4 — low-tension dialogues persist as SOCRATIC_DIALOGUE in Neo4j (daemon MCP)
# D3 — data/pi-recent-discoveries.json (ingest/distill hooks)
# Pi extension shows panel on session_start (restart Pi agent)

gzmo honeypot rejects --tail 50
gzmo honeypot review list
gzmo honeypot review promote <vault_id>

./scripts/ingest-quality/honeypot-confidence-histogram.py
./scripts/export-prerequisite-graph.py   # requires: pip install neo4j
./scripts/remediation-verify.sh
```

## F2 confidence tuning

Run the histogram script. It writes `data/honeypot-confidence-report.json` with a
data-driven recommendation. **Do not lower `HONEYPOT_MIN_CONFIDENCE` unless the
report recommends it** (>=100 rows in 0.80-0.84 band with >=30% already in honeypot).

## E2 Neo4j export workflow

```bash
uv pip install neo4j --python python3   # or: pip install neo4j
uv run python3 ./scripts/export-prerequisite-graph.py
gzmo pedagogy graph validate data/pedagogy/graphs/pending/
# human review, then mv pending/*.yaml -> data/pedagogy/graphs/
```

## Verify

```bash
cargo build --release -p gzmo-cli
./scripts/restart-daemon.sh
./scripts/remediation-verify.sh
```

## Operator checklist

1. Host: `./scripts/container-lan-forward.sh start`
2. Pi: restart agent (display + recent discoveries extensions)
3. Optional: `pip install neo4j` then run graph export
