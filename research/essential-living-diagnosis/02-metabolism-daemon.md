# Metabolism and daemon overnight contracts

## Scope

Overnight/living **writer** contracts for GZMO Essential Living Diagnosis: `METABOLISM_JOBS` / telemetry / `metabolism_needs_work`, daemon cron ticks (dream, spark, distill, promote, embed, Qdrant), orchestrator headless waves vs daemon CheapCheck, control-plane owner gating, and code↔`OVERNIGHT_METABOLISM.md` / template drift (especially spark clocks). Brain Feed satellites (tinyFolder, felt-use census, brain-feed-check) are named only to separate them from daemon law. Non-goals: side-store internals, MCP tool inventory, live host operations.

## Contract inventory

### Metabolism ledger (`metabolism.rs`)

| Symbol | Contract (Observed) |
|--------|---------------------|
| `METABOLISM_JOBS` | `["distill", "promote", "embed", "dream", "spark"]` — GREEN board rows and newest-job filter (`gzmo-core/src/metabolism.rs:148`). |
| `LEARNING_LOOP_JOBS` | `["dream", "spark"]` — night identity ring via `upsert_learning_loop_night` (`:38-39`, `:138-144`). |
| `WATCHDOG_JOBS` | `["distill", "dream"]` only — soft-fail stale signal; not full `METABOLISM_JOBS` (`:150-151`, `:271-315`). |
| `DEFAULT_METABOLISM_STALE_SECS` | `26 * 3600`; override `GZMO_METABOLISM_STALE_SECS` (`:153-163`). |
| `write_job_run` | Writes `{vault_parent}/scheduler-runs/{job}-{stamp}.json`, copies `latest.json` + `latest-{job}.json`, stamps `runner` + `night_id` (`:104-146`). |
| `collect_metabolism_board` / `format_overnight_metabolism` | Per-job OK/FAIL/missing table; GREEN if ≥3 OK jobs and honeypot non-empty; watchdog stale demotes non-RED → YELLOW without flipping core GREEN math (`:317-392`, `:457-512`). |
| `metabolism_needs_work` | `true` iff latest honeypot rows > 0 **or** semantic_vault rows missing embeddings; missing DB → fail-closed `false` (`:538-546`, `:514-535`). Call sites: `promote_cmd` / `embed_cmd` skip path still records OK run (`gzmo-cli/src/promote_cmd.rs:13-17`, `embed_cmd.rs:11-15`). |
| Price shift | Soft advice for `distill`/`dream` when `GZMO_PRICE_SHIFT=1`; does not rewrite cron (`metabolism.rs:548-571`; used in `serve_cmd.rs:281-312`). |

### Product clock SoT vs compiled defaults

Doctrine clock SoT: `docs/OVERNIGHT_METABOLISM.md` + product template `gzmo.toml.example` + living overview; compiled defaults apply only when TOML omits keys (`OVERNIGHT_METABOLISM.md:5-6`).

| Tick (UTC) | Doctrine / template | Config source | Compiled default if key omitted |
|------------|---------------------|---------------|----------------------------------|
| 01:00 Dream | CONSOLIDATE | `[dreams] cron_hour/minute` (`gzmo.toml.example:79-81`) | hour `1`, minute `0` (`config.rs:2940-2945`) |
| 01:45 Qdrant sync | AUDIT (index) | `[qdrant] sync_cron_*` (`gzmo.toml.example:167-169`) | hour `1`, minute `45`; **`sync_enabled` default `false`** (`config.rs:1819-1835`) |
| 02:15 SessionDistill | TRIAGE | `[session_distill] cron_*` + `daemon_scheduled` (`gzmo.toml.example:103-105`) | hour `2`, minute `15`, `daemon_scheduled=true` (`config.rs:1368-1378`) |
| 02:30 promote | TRIAGE | `[metabolism] promote_cron_*` | hour `2`, minute `30` (`config.rs:1435-1440`) |
| 02:45 embed | TRIAGE (index write) | `[metabolism] embed_cron_*` | hour `2`, minute `45` (`config.rs:1441-1446`) |
| 03:30 + 22:30 Spark | CONSOLIDATE | `[spark] cron_hours = [3, 22]`, `cron_minute = 30` (`gzmo.toml.example:122-124`; `config/gzmo-next.toml:81-83`) | **`[9, 14, 21]` at minute `17`** (`config.rs:2970-2974`) |
| Continuous ingest watcher | TRIAGE | `[ingest]` + orchestration watchers | — |

**Observed:** `gzmo.toml.example` has **no** `[metabolism]` section (grep empty). Promote/embed clocks therefore rely entirely on `MetabolismConfig` compiled defaults unless another host TOML pins them. Lab/next pin exists in `config/gzmo-next.toml:192-198`.

### Daemon writer path (`gzmo daemon` → `daemon_cmd.rs`)

Owner claim first: `control_plane::claim_owner(config)` (`daemon_cmd.rs:96`).

| Loop | Gate | Schedule helper | Job receipt | Notes |
|------|------|-----------------|-------------|-------|
| Heartbeat / CheapCheck | always | `HeartbeatEngine.interval` | none (writes `HEARTBEAT.md` markers) | `FileChangeCheck`, LLM/Prime `HealthPing`, `CognitionBlackoutCheck`, optional `EmbedHealthPing` (`:98-134`, `:424-486`). Silent OK path may idle-evolve `living-research-intel.sh` ≤1/6h. **Not a metabolism job.** |
| Dream | `dreams.enabled` | wall clock ≥ cron; once per **yesterday** date | `write_job_run(..., "dream", lab\|rust)` | Dedicated `DreamEngine` (or lab `session-to-dream.sh`). Comment: replaces headless `auto_dream` (`:489-611`). |
| Spark | `spark.enabled` | `spark_cron_slot_due` (cron) or dice | `write_job_run(..., "spark", ...)` | Multi-slot day; advances slot on fail to avoid 60s spin (`:613-787`). Lab: `cognition-smoke.sh --spark-run`. |
| Qdrant sync | `qdrant.sync_enabled` | `cron_due_today(sync_cron_*)` | **no** `METABOLISM_JOBS` receipt | Dedicated 01:45 loop (`:789-845`). |
| Session distill | `session_distill.enabled && daemon_scheduled` | `cron_due_today` | `write_job_run(..., "distill", ...)` | Spawns `gzmo distill` or lab handoff (`:847-1003`). Plus continuous BRPOP distill worker (`:239-242`). |
| Promote | `metabolism.enabled` | `metabolism.promote_cron_*` | via `promote_cmd` → `write_job_run` | Added so living daemon does not soft-miss triad (`:1334-1384`). |
| Embed (+ optional post-embed Qdrant) | `metabolism.enabled` | `metabolism.embed_cron_*` | via `embed_cmd` | May call `sync_vault_to_qdrant` again if qdrant enabled+sync (`:1385-1436`). |
| Watchdog | always | every 300s | `latest-watchdog.json` | Same ledger as serve (`:1319-1332`). |
| Synapse pull | `synapse_pull.enabled` | daily cron | no metabolism job | Episodic feed (`:1005-1079`). |
| KG reconcile | `kg_reconcile.enabled` | daily cron | no metabolism job | (`:1081-1136`). |
| Wiki sync / lint | `wiki.enabled` | sync daily; lint DOW+hour | no metabolism job | Doctrine: not Brain Feed AUDIT (`OVERNIGHT_METABOLISM.md:71-72`). |
| Config handoff 04:00 | **lab assembly only** | hardcoded 04:00 | none | Living skips (`daemon_cmd.rs:1270-1316`; doctrine table `OVERNIGHT_METABOLISM.md:67-69`). |

Catch-up: `daemon::cron_due_today` / `spark_cron_slot_due` fire on first tick **at or after** scheduled UTC, including after restart (`daemon.rs:23-59`). Dream loop uses a slightly different yesterday-keyed gate (`daemon_cmd.rs:500-508`) rather than `cron_due_today`.

### Thin serve writer path (`gzmo serve` → `serve_cmd.rs`)

- Requires `[metabolism].enabled` else bail (`serve_cmd.rs:157-159`).
- Claims same owner plane (`:161`).
- 60s loop: watchdog → dream → distill → promote → embed(+qdrant) → spark multi-slot → soft Sunday `dream-compact` 03:00 → soft OKForge wiki push (`:250-410`).
- GREEN core jobs match `METABOLISM_JOBS`; wiki/`dream-compact` are soft-fail satellites (module docs `:1-5`).
- Custom operator jobs: `[cron.jobs.*]` via `gzmo cron`; builtins reserved `["dream","distill","promote","embed","spark","wiki_push"]` (`gzmo-core/src/cron/mod.rs:19-20`).
- No CheapCheck / HeartbeatEngine on this path. No orchestrator headless waves.

### Orchestrator vs CheapCheck

| Surface | Role | Overnight metabolism? |
|---------|------|------------------------|
| `orchestrator::start_orchestrator` | Headless simple prompts or multi-step **wave** pipelines (`depends_on` topo sort) for non-disabled `[orchestration.jobs.*]` (`orchestrator.rs:3-7`, `:106-172`, `:181-187`, `:407-448`). | **Not** the dream/spark/distill/promote/embed law. Daemon **strips** keys `spark` and `auto_dream` before start (`daemon_cmd.rs:403-407`). |
| Product template orch jobs | `sys_janitor` enabled (30m); `wiki_sync`/`wiki_lint`/`kg_reconcile`/`synapse_pull`/`auto_dream`/`spark` **disabled** to avoid duplicating typed loops (`gzmo.toml.example:404-511`). `honeypot_ripen` commented optional. | Janitor is LLM background ops, not `METABOLISM_JOBS`. |
| Legacy disabled spark orch cron | `0 17 9,14,21 * * *` (`gzmo.toml.example:509-511`) | Matches **compiled** spark defaults, not living template `[3,22]@30`. |
| CheapCheck | Deterministic pre-LLM triage trait + HEARTBEAT.md section (`daemon.rs:62-75`, `:140-178`, `:325-389`). | Health/anomaly plane only; does not schedule metabolism jobs. |

### Control-plane / who may run overnight

- **Owners:** `gzmo serve` and `gzmo daemon` — exclusive `{vault_db}.write.lock` flock + Unix socket API (`docs/ADR-0006-owner-control-plane.md:12-16`; `control_plane/mod.rs:1-5`, `claim_owner` `:48-62`; `lock.rs:24-42`).
- **Clients:** CLI memory / MCP attach prefer live socket; living vault under `/opt/gzmo` hard-fails without owner unless `--offline` / `GZMO_CONTROL_PLANE=0` (`attach.rs:26-68`).
- **Host mutex** (CT101 vs workstation vs appliance) remains placement doctrine; process ownership is the flock (`ADR-0003:12-20`, `ADR-0006:16`).
- Second serve/daemon on same vault fails closed at lock acquire (Observed in tests `control_plane/mod.rs:90-91`).

### Living daemon law vs Brain Feed satellites (not daemon law)

Doctrine separation (`OVERNIGHT_METABOLISM.md:43`, `:65-73`, `:7`):

| Item | Classification |
|------|----------------|
| Dream / distill / promote / embed / spark / Qdrant sync on owner host | Living daemon (or thin serve) writer ticks |
| Continuous ingest watcher | Living TRIAGE |
| tinyFolder overnight timer/scripts | Brain Feed **satellite** — not a `gzmo-daemon` cron slot |
| `felt-use-depth.sh` / felt-use census | Operator / telescope soak evidence |
| `brain-feed-check.sh` | Operator nutrient-path check — **not** a nightly daemon job |
| Lab `gzmo-handoff.sh` 04:00 | Lab assembly backend only |
| Wiki sync/lint / OKForge push | Soft plane; “Not Brain Feed; do not call Observatory emit AUDIT” |
| `gzmo-scheduler` lab recipes | Beat-gate / lab, not living authority (referenced in next runbooks; serve comment ADR-0003) |

## Gaps and drift

1. **Spark clock divergence (CT101 pin risk) — Observed + Doc-dated**  
   - Living template / doctrine: `cron_hours = [3, 22]`, `cron_minute = 30` (`gzmo.toml.example:122-124`; `OVERNIGHT_METABOLISM.md:55-57`).  
   - Compiled default if `[spark].cron_hours` omitted: `[9, 14, 21]` @ minute `17` (`config.rs:2970-2974`).  
   - Disabled legacy orch spark still encodes `9,14,21` @ 17 (`gzmo.toml.example:509-511`).  
   - Doctrine explicitly warns: missing TOML pin is **not** the living overview; pin template on CT101 (`OVERNIGHT_METABOLISM.md:57`).

2. **`[metabolism]` absent from product template — Observed**  
   Promote 02:30 / embed 02:45 are documented as MetabolismConfig defaults and appear in `gzmo-next.toml`, but `gzmo.toml.example` never declares `[metabolism]`. Behavior matches defaults today; pin surface is weaker than spark/dream sections.

3. **Qdrant `sync_enabled` default false vs template true — Observed**  
   Template living pin sets `sync_enabled = true` (`gzmo.toml.example:167`). Struct default `sync_enabled: false` (`config.rs:1833`). Untemplated config skips the 01:45 AUDIT index tick.

4. **Dual Qdrant paths — Observed**  
   Daemon: dedicated 01:45 sync loop **and** optional second sync after embed (~02:45). Serve: Qdrant only after embed job when enabled (`serve_cmd.rs:346-352`). Doctrine wall-clock table lists single 01:45 AUDIT slot (`OVERNIGHT_METABOLISM.md:20`); post-embed sync is code-path index write not separately labeled.

5. **Watchdog subset vs board jobs — Observed**  
   Board tracks five `METABOLISM_JOBS`; stale watchdog only ages `distill` + `dream`. Missing promote/embed/spark latest does not alone set `watchdog.stale`.

6. **`metabolism_needs_work` semantics — Observed**  
   Name/docs suggest “skip when nothing to ripen or promote”; implementation is honeypot-or-missing-embeddings presence, not a pending-promote queue depth. Empty vault → skip with OK receipt (can look GREEN-ish for promote/embed without work).

7. **Qdrant / wiki / synapse / kg not on GREEN job list — Observed**  
   Doctrine still names Qdrant as living AUDIT tick; receipts and GREEN math ignore it. Wiki is soft satellite on serve (`wiki` job name) but daemon wiki loops write no `scheduler-runs` metabolism rows.

8. **Serve vs daemon feature split — Observed**  
   Both claim owner and can write metabolism. Daemon adds CheapCheck, orchestrator janitor/waves, synapse/kg/wiki/handoff lab, ingest engine wiring. Serve is thin typed cron + custom `[cron]` + optional wiki OKForge. Doctrine prefers one living writer host; either process may be that writer depending on claim.

9. **Orchestrator `test_pulse` enabled in example — Observed**  
   `gzmo.toml.example:421-424` leaves `test_pulse` every 20s without `disabled = true` in the shown block — headless LLM noise risk if product template is copied verbatim (separate from metabolism law but same daemon process).

10. **Documented ticks present in code — Observed**  
    Dream, distill, promote, embed, spark, Qdrant sync, ingest watcher path all exist on daemon and/or serve. No doctrine overnight metabolism tick is wholly missing from code. Extra code jobs (wiki, kg, synapse, handoff lab, dream-compact, idle evolve) are either soft-fail, non-GREEN, or explicitly non-living.

## Evidence status

| Claim area | Status |
|------------|--------|
| `METABOLISM_JOBS`, watchdog, board, `metabolism_needs_work` | Observed in `metabolism.rs` + promote/embed cmds |
| Daemon tick inventory | Observed in `daemon_cmd.rs` + `daemon.rs` helpers |
| Serve loop | Observed in `serve_cmd.rs` |
| Spark / dream / distill / metabolism / qdrant defaults | Observed in `config.rs` + templates |
| Spark template vs compiled drift | Observed; Doc-dated in `OVERNIGHT_METABOLISM.md:57` |
| Orchestrator waves + job strip | Observed in `orchestrator.rs` + `daemon_cmd.rs:403-407` |
| Owner gating | Observed in `control_plane/*` + ADR-0003/0006 |
| Brain Feed satellites vs daemon law | Doc-dated `OVERNIGHT_METABOLISM.md`; scripts/timers exist in tree (not executed this pass) |
| Live CT101 schedule/TOML as deployed | Unreachable (no live host ops in scope) |
| Runtime proof of a night’s receipts | Unreachable |

## Sources

- `gzmo-core/src/metabolism.rs` — jobs, ledger, watchdog, board, `metabolism_needs_work`
- `gzmo-core/src/daemon.rs` — `cron_due_today`, `spark_cron_slot_due`, CheapCheck, HeartbeatEngine
- `gzmo-core/src/config.rs` — Dreams/Spark/SessionDistill/Metabolism/Qdrant defaults
- `gzmo-core/src/orchestrator.rs` — headless modes, wave resolution, `start_orchestrator`
- `gzmo-core/src/control_plane/{mod,lock,attach,client}.rs` — owner claim and client attach
- `gzmo-core/src/cron/mod.rs` — serve builtin job ids
- `gzmo-cli/src/daemon_cmd.rs` — full overnight loop wiring
- `gzmo-cli/src/serve_cmd.rs` — thin metabolism runner
- `gzmo-cli/src/{promote,embed}_cmd.rs` — needs_work skip + receipts
- `gzmo.toml.example` — product clock pins and orchestration legacy jobs
- `config/gzmo-next.toml` — next/lab metabolism pin including `[metabolism]`
- `docs/OVERNIGHT_METABOLISM.md` — named night labels, satellite boundary, spark pin warning
- `docs/ADR-0003-one-instance-metabolism.md`, `docs/ADR-0006-owner-control-plane.md`
