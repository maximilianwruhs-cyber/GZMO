# Thema 006 — Loop Engineering Synthesis

This document presents the operator-facing synthesis of Loop Engineering concepts, aligning thema_006 research, industry best practices, and GZMO's platform mechanics.

---

## 1. Executive Synthesis

The GZMO auto-discovery, implementation, and verification loops are vulnerable to systemic failures when the boundaries between probabilistic model generation and deterministic execution gates are blurred. GZMO's current architecture and the thema_006 research files describe the same core failure modes:

| Research Diagnosis (thema_006) | GZMO Symptom | Current Mitigation |
| :--- | :--- | :--- |
| **Event Swallowing** — LLM ignores `spawn_fixing_agent` when gate stderr floods context | Fixer won't spawn when gates fail hard | External kurator hypervisor ([kurator_monitor.rs](file:///home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO/gzmo-core/src/kurator_monitor.rs)) decides spawn via Rust, not LLM |
| **Poisoned Blackboard** — fixer inherits broken workspace | Fixing agent crashes in contaminated CWD | `fixer_worktree_isolation = true` in [gzmo.toml](file:///home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO/gzmo.toml); ephemeral worktrees in [kurator_spawn.rs](file:///home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO/gzmo-core/src/kurator_spawn.rs) |
| **Schema Deadlocks** — fragile log parsers halt supervisor | Silent orchestrator stop on parse failure | Structured Rust parsing in [discovery_fixer.rs](file:///home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO/gzmo-core/src/discovery_fixer.rs); typed verify in [verify-gate.md](file:///home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO/docs/spec/verify-gate.md) |
| **Blind Spinning** — repeated useless mutations | Token burn, no progress | `StuckDetector` in [agent_loop.rs](file:///home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO/gzmo-core/src/agent_loop.rs) (checks duplicate signatures, ping-pongs) |
| **End-Only Verification** | Failures cascade across plan $\to$ execute | Partial: artifact verify + `discovery_acceptance_gate` after spawn; not between every phase transition |
| **KB Loop Open** — reports $\ne$ recall | Distill dedup ~96%, redundant findings | Tracked gaps G1–G12 in [DISCOVERY_KB_FEEDBACK_LOOP.md](file:///home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO/docs/DISCOVERY_KB_FEEDBACK_LOOP.md) |

---

## 2. Core Mechanics (The Four Pillars)

1. **State Space Perception & Observation Filters**
   Probabilistic models are vulnerable to token saturation when flooded with verbose stderr and logs. GZMO filters and compacts compiler outputs to keep the context clean.
2. **Agentic Decision & Action Trajectories**
   Constraint is the foundation of agent safety. GZMO utilizes Model Context Protocol (MCP) tool schemas and strict JSON output formatting.
3. **Deterministic Verification Gates**
   An agent cannot grade its own homework. Verification gates must execute programmatically in sandboxed sandboxes, returning boolean outcomes.
4. **Shared Blackboard State Architectures**
   Decoupling conversation history from logical system state. GZMO stores the execution plan, remediation tracking, and findings database in SQLite (`vault.db`) and relational files.

---

## 3. GZMO Module Mapping Table

| Research Primitive | GZMO Implementation Target | Status | Notes / Gaps |
| :--- | :--- | :--- | :--- |
| **Event Swallowing** | [kurator_monitor.rs](file:///home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO/gzmo-core/src/kurator_monitor.rs) (`process_discovery_report`) | **Implemented** | Spawn recommended by external Rust supervisor, avoiding LLM self-spawn failures. |
| **Poisoned Blackboard** | [kurator_spawn.rs](file:///home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO/gzmo-core/src/kurator_spawn.rs), [gzmo.toml](file:///home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO/gzmo.toml) | **Implemented** | Ephemeral Git worktrees prevent fixer workspace pollution. *Gap:* Execute phase is not isolated. |
| **Schema Deadlock** | [discovery_fixer.rs](file:///home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO/gzmo-core/src/discovery_fixer.rs), [verify-gate.md](file:///home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO/docs/spec/verify-gate.md) | **Implemented** | Typed structures enforce schema validation of agent artifacts. |
| **Blind Spinning** | [agent_loop.rs](file:///home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO/gzmo-core/src/agent_loop.rs) (`StuckDetector`) | **Implemented** | Checks for exact tool signature repetition and ping-pong patterns. *Gap:* No AST hash checks. |
| **Stage Gates** | [run-discovery-goal-pipeline.sh](file:///home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO/scripts/run-discovery-goal-pipeline.sh) | **Partial** | Gates run *after* spawn (artifact + acceptance), not at every phase transition. |
| **KB Recall & Closure** | [discovery-kb-recall-smoke.sh](file:///home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO/scripts/discovery-kb-recall-smoke.sh) | **Partial** | Recall verified via smoke tests; requires ingestion hardening to prevent G9/G12 drift. |

---

## 4. Context Compaction & Management

Long sessions degrade attention ("Context Rot"). The factory.ai evaluation of 36,000 messages indicates the tradeoffs of different compaction strategies:

| Strategy | Mechanism | Multi-Session Retention | Verbatim Precision | System Cost / Impact |
| :--- | :--- | :--- | :--- | :--- |
| **Structured Summary** | LLM-based reduction | **37%** (63% lost) | Low (paraphrase risk) | High API costs, extends trajectories by 13-15% |
| **Verbatim Pruning (Morph)**| Delete boilerplate / logs | **High** | **98%** (Exact match) | Zero-hallucination, high throughput (>3k tok/s) |
| **Observation Masking** | Replace logs with hints | **Medium** | **High** | Cost neutral, +2.6% SWE-bench, 50% cost savings |
| **Opaque Compression** | Vector-encoded cache | N/A | N/A | 99.3% compression, complete vendor lock-in |

> [!TIP]
> GZMO utilizes a combination of **Verbatim Pruning** (cleaning old shell outputs) and **Observation Masking** (replacing flooded logs with filesystem pointers and retrieval hints).

---

## 5. Web & Academic References

- **cobusgreyling/loop-engineering**: Framework matrices detailing the orchestration of autonomous feedback loops.
- **Stage Gate Loops (Peerlist)**: Conceptualizing "micro-gates" that block transition across Analyze $\to$ Plan $\to$ Approve $\to$ Execute phases.
- **TrueFoundry Enterprise Runtime**: Budgeting execution cycles, implementing durable Human-in-the-Loop (HITL) gates, and preventing runaway costs.
- **Kitchen Loop (arXiv:2603.25697)**: Design for adversarial UAT and test-suite splits to prevent Maker-Checker collusion.

---

## Appendix: Phase 2 Prioritized Pack Backlog

### Pack F — Inter-Phase Stage Gates
- **Scope**: Inject deterministic gates between Analyze $\to$ Plan, Plan $\to$ Approve, Approve $\to$ Execute, and Execute $\to$ Distill transitions in [run-discovery-goal-pipeline.sh](file:///home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO/scripts/run-discovery-goal-pipeline.sh).
- **Effort**: Medium (1-2 days).
- **Impact**: **Critical**. Fails fast on early planning anomalies before executing mutations.

### Pack G — Gate Output Spill-to-File
- **Scope**: Write full stdout/stderr of failed compiler/test runs to `$GZMO_SKILLS_ROOT/data/gate-outputs/<task_id>.log` and inject a retrieval hint (e.g. `[Logs truncated; full log saved to file...]`) in the fixer brief.
- **Effort**: Low (0.5 days).
- **Impact**: **High**. Eliminates event swallowing and instruction dilution during fixer context preparation.

### Pack H — KB Epistemological Closure (G9, G12)
- **Scope**: Enforce `DISCOVERY_LOOP=1 scripts/ingest-quality/eval-quick.sh` before finalizing runs. Implement novelty-dedup fingerprint deferral.
- **Effort**: Medium (2 days).
- **Impact**: **High**. Prevents the daemon from spawning duplicate discovery runs for existing facts.

### Pack I — Execute Worktree Isolation
- **Scope**: Extend git-worktree sandbox isolation from fixer spawner to the `spawn_discovery_execute` runner in `kurator_spawn.rs`.
- **Effort**: Medium (1 day).
- **Impact**: **Medium**. Protects the execution workspace from contamination.

### Pack J — Research Ingest
- **Scope**: Operator path to copy GZMO loop engineering synthesis to the curated knowledge folder and run `gzmo ingest`.
- **Effort**: Low (0.5 days).
- **Impact**: **Medium**. Ensures future loops recall this synthesis.
