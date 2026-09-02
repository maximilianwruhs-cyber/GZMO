# Continuous Evolution Operations and Cutover Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Operate the connected repository loop and air-gapped appliance loop continuously with one candidate, explicit budgets, unified status, signed offline handoff, exercised stop/rollback, and a clean retirement of duplicate legacy evolve paths after parity.

**Architecture:** Connected development uses hardened system services with distinct `gzmo-evolver-coordinator` and `gzmo-evolver-worker` identities; only the coordinator can use the GitHub App, while only the worker can modify independent candidate clones. The Living owner performs daily observation and bounded internal evolution through `EvolutionController`, without GitHub or network credentials. Both emit the same candidate/evaluation/audit contracts. Portable signed bundles bridge approved artifacts across the airgap. Legacy script/idle paths run in shadow until parity, then are removed in one cutover.

**Tech Stack:** systemd system services plus appliance owner scheduler, Rust status/control, evolution-contracts, PostgreSQL audit, GitHub App, OMP with a qualified local code model in a private network namespace, signed tar.zst bundles, existing GZMO health/opportunity/keep-quality gates.

**Spec:** `docs/superpowers/specs/2026-08-31-self-developing-living-database-design.md`

## Global Constraints

- Connected runner and Living owner use different OS identities, state roots, credentials, and schedules.
- GitHub credentials never cross onto portable media or the air-gapped appliance.
- Operator signing private keys and recovery keys are never stored on either active runner.
- One active opportunity and one nonterminal candidate per repository/appliance.
- Defaults: connected mission refresh Monday 07:00 UTC; candidate run Monday 08:00 UTC; PR shepherd every 15 minutes only while a candidate PR exists; Living observe daily 06:30 UTC after metabolism; hard-candidate generation at most weekly.
- Signed envelopes may reduce cadence/budgets but unsigned config cannot increase them.
- Candidate work stops first under resource pressure. Recall and one-writer authority are preserved.
- No auto-merge, no direct main push, no silent production promotion, no cloud fallback.
- Legacy and replacement paths never both apply mutations. Shadow mode compares artifacts only.
- Remove legacy paths only after four consecutive weekly Stage 1 checkpoints and one complete offline Stage 2 soak/rollback drill.

## File Structure

| Path | Responsibility |
|---|---|
| `systemd/system/gzmo-repo-evolver.service` | Trusted coordinator oneshot |
| `systemd/system/gzmo-repo-evolver.timer` | Weekly candidate cadence |
| `systemd/system/gzmo-repo-evolver-shepherd.service` | Poll/repair one active candidate PR |
| `systemd/system/gzmo-repo-evolver-shepherd.timer` | Bounded 15-minute polling |
| `systemd/system/gzmo-evolver-worker@.service` | Fixed uncredentialed OMP worker sandbox |
| `systemd/system/gzmo-evolver-model.service` | Read-only local code-model endpoint in the worker network namespace |
| `deploy/repo-evolver/60-gzmo-evolver-worker.rules` | Narrow authorization to start/stop only worker instances |
| `scripts/install-repo-evolver.sh` | Safe connected-host installation/disable/status |
| `gzmo-core/src/evolution/scheduler.rs` | Living observe/candidate due calculation |
| `gzmo-cli/src/daemon_cmd.rs` | Start one evolution scheduler under owner claim |
| `gzmo-core/src/evolution/status.rs` | Unified operational state |
| `gzmo-core/src/ecosystem_status.rs` | Read-only evolution summary |
| `gzmo-core/src/mcp/serve.rs` | Read-only evolution status tool |
| `gzmo-evolver/src/bundle.rs` | Export connected candidate/evaluation bundle |
| `gzmo-core/src/evolution/bundle.rs` | Verify/import offline bundle without promotion |
| `scripts/evolution-contract-check.sh` | Cross-stage schema/audit/status gate |
| `tests/evolution-contract-test.sh` | Hermetic contract fixture test |
| `scripts/evolution-chaos-drill.sh` | Fixture-only stop/rollback/projection drill |
| `docs/CONTINUOUS_EVOLUTION.md` | Operator runbook and authority model |

---

### Task 1: Install the Connected Runner Without GitHub Actions Privilege

**Files:**
- Create: `systemd/system/gzmo-repo-evolver.service`
- Create: `systemd/system/gzmo-repo-evolver.timer`
- Create: `systemd/system/gzmo-repo-evolver-shepherd.service`
- Create: `systemd/system/gzmo-repo-evolver-shepherd.timer`
- Create: `systemd/system/gzmo-evolver-worker@.service`
- Create: `systemd/system/gzmo-evolver-model.service`
- Create: `deploy/repo-evolver/60-gzmo-evolver-worker.rules`
- Create: `scripts/install-repo-evolver.sh`
- Test: `tests/repo-evolver-systemd-test.sh`

**Interfaces:**
- Produces: disabled-by-default services with explicit install/enable/disable/status actions.
- Consumes: `gzmo-evolver` binary, config, GitHub App secret path, dedicated OMP profile.

- [ ] **Step 1: Write unit-file assertions**

Test requires coordinator `User=gzmo-evolver-coordinator`, worker `User=gzmo-evolver-worker`, model `User=gzmo-evolver-model`, `Type=oneshot` for runner units, absolute trusted config/state paths, `UMask=0077`, `NoNewPrivileges=true`, `PrivateTmp=true`, `ProtectSystem=strict`, `ProtectHome=true`, worker/model shared private network namespace with no host route, explicit per-role writable paths, no provider/PAT/GitHub token environment, no worker read access to `/etc/gzmo/github-app.pem` or coordinator state, and no self-hosted Actions runner.

- [ ] **Step 2: Run to verify failure**

Run: `bash tests/repo-evolver-systemd-test.sh`

Expected: FAIL: units missing.

- [ ] **Step 3: Create the trusted runner service**

Create the coordinator unit with these effective properties:

```ini
[Service]
Type=oneshot
User=gzmo-evolver-coordinator
Group=gzmo-evolver-coordinator
EnvironmentFile=/etc/gzmo/repo-evolver.env
ExecStart=/usr/local/bin/gzmo-evolver --config /etc/gzmo/repo-evolver.toml run
UMask=0077
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadOnlyPaths=/etc/gzmo
ReadWritePaths=/var/lib/gzmo-evolver/coordinator /var/lib/gzmo-evolver/workspaces /run/gzmo-evolver
```

Create the local model and worker units:

```ini
# gzmo-evolver-model.service
[Service]
User=gzmo-evolver-model
PrivateNetwork=yes
ProtectSystem=strict
ProtectHome=true
NoNewPrivileges=true
ExecStart=/usr/local/bin/llama-server --offline --host 127.0.0.1 --port 8011 --model /var/lib/gzmo-evolver-model/code.gguf

# gzmo-evolver-worker@.service
[Unit]
Requires=gzmo-evolver-model.service
After=gzmo-evolver-model.service
JoinsNamespaceOf=gzmo-evolver-model.service
[Service]
Type=oneshot
User=gzmo-evolver-worker
PrivateNetwork=yes
ProtectSystem=strict
ProtectHome=true
PrivateDevices=true
NoNewPrivileges=true
RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6
ReadOnlyPaths=/run/gzmo-evolver/%i/request.json
ReadWritePaths=/var/lib/gzmo-evolver/workspaces/%i /var/lib/gzmo-evolver/worker/%i
ExecStart=/usr/local/bin/gzmo-evolver worker --request /run/gzmo-evolver/%i/request.json
```

The coordinator writes request directories mode 0750 and request files mode 0440 for the worker group. It alone reads `/etc/gzmo/github-app.pem` (root/coordinator group, mode 0640). The model and workers share a private loopback-only namespace with no host/default route; the OMP profile targets `127.0.0.1:8011`. The worker cannot traverse coordinator state or `/etc/gzmo`.

Install this exact narrow Polkit rule from `deploy/repo-evolver/60-gzmo-evolver-worker.rules`:

```javascript
polkit.addRule(function(action, subject) {
  const unit = action.lookup("unit") || "";
  const verb = action.lookup("verb") || "";
  if (action.id === "org.freedesktop.systemd1.manage-units" &&
      subject.user === "gzmo-evolver-coordinator" &&
      /^gzmo-evolver-worker@[a-z0-9-]+\\.service$/.test(unit) &&
      (verb === "start" || verb === "stop")) {
    return polkit.Result.YES;
  }
});
```

No rule grants model, timer, coordinator, or arbitrary-unit control.

- [ ] **Step 4: Create cadence timers**

Candidate timer: `OnCalendar=Mon *-*-* 08:00:00 UTC`, `Persistent=true`, `RandomizedDelaySec=10m`. Shepherd timer: `OnUnitActiveSec=15m`; service exits immediately when no active PR. Timers use `RefuseManualStart` only if it would block recovery; manual one-shot must remain available.

- [ ] **Step 5: Implement installer verbs**

```text
install-repo-evolver.sh --install-only
install-repo-evolver.sh --enable
install-repo-evolver.sh --disable
install-repo-evolver.sh --status
```

`--install-only` creates the three locked system users, directories/ownership, private model namespace/service, runner units, and narrow worker authorization; it never fabricates secrets or enables timers. `--enable` verifies binary/config/policy/App key permissions, separate worker access, local code-role qualification, zero worker egress, and `gzmo-evolver status`, then requires `REPO_EVOLVER_ENABLE=1`. It never modifies GitHub settings.

- [ ] **Step 6: Run unit/installer tests**

Run: `bash tests/repo-evolver-systemd-test.sh`

Expected: PASS using a temporary root, fake systemctl/user database, and permission-mode assertions.

- [ ] **Step 7: Commit**

```bash
git add systemd/system/gzmo-repo-evolver* systemd/system/gzmo-evolver-worker@.service systemd/system/gzmo-evolver-model.service deploy/repo-evolver scripts/install-repo-evolver.sh tests/repo-evolver-systemd-test.sh
git commit -m "feat: schedule review-only repository evolution"
```

---

### Task 2: Schedule Living Evolution Under the Existing Owner

**Files:**
- Create: `gzmo-core/src/evolution/scheduler.rs`
- Modify: `gzmo-core/src/evolution/mod.rs`
- Modify: `gzmo-cli/src/daemon_cmd.rs`
- Test: `gzmo-core/src/evolution/scheduler.rs`

**Interfaces:**
- Produces: `EvolutionSchedule::{next_action,record_attempt}` and one daemon task handle.
- Consumes: `EvolutionConfig`, owner claim lifetime, controller, active envelope/circuit breaker.

- [ ] **Step 1: Write catch-up and suppression tests**

```rust
#[test]
fn daily_observe_catches_up_but_candidate_never_bursts() {
    let schedule = schedule_at(6, 30, Weekday::Mon, 7, 0);
    assert_eq!(schedule.next_action(mon_at(6, 45), none_run()), Some(Action::Observe));
    assert_eq!(schedule.next_action(mon_at(8, 00), observed_today()), Some(Action::GenerateCandidate));
    assert_eq!(schedule.next_action(mon_at(8, 01), candidate_attempted_today()), None);
    assert_eq!(schedule.next_action(mon_at(8, 02), active_candidate()), None);
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-core evolution::scheduler`

Expected: FAIL: scheduler missing.

- [ ] **Step 3: Implement schedule semantics**

Observe daily at/after configured time with once-per-date catch-up. Generate hard candidate no more than weekly and only after a fresh observation, all hard floors, no active candidate, cooldown, qualified role, and envelope permission. Memory repair/Tunable opportunities may run more frequently only within signed envelope frequency caps.

- [ ] **Step 4: Start under owner lifetime**

After `claim_owner` and all strict startup checks, daemon starts one `EvolutionSchedule` task with an `Arc<EvolutionController>`. Add it to the existing join/abort handling. Do not spawn a second binary/writer or shell script.

- [ ] **Step 5: Suppress under degradation**

No candidate generation when PostgreSQL, audit, owner, rollback, `extract_verify`, disk reserve, or required meter fails. Code candidate role absence disables only hard candidate generation. Candidate work yields to recall/metabolism under resource pressure.

- [ ] **Step 6: Run scheduler tests**

Run: `cargo test -p gzmo-core evolution::scheduler`

Expected: PASS including restart catch-up, future stamps, DST immunity via UTC, active candidate, stop file, and degraded roles.

- [ ] **Step 7: Commit**

```bash
git add gzmo-core/src/evolution gzmo-cli/src/daemon_cmd.rs
git commit -m "feat: schedule evolution under the living owner"
```

---

### Task 3: Expose One Honest Evolution Status

**Files:**
- Modify: `gzmo-core/src/evolution/status.rs`
- Modify: `gzmo-core/src/ecosystem_status.rs`
- Modify: `gzmo-core/src/mcp/serve.rs`
- Modify: `gzmo-cli/src/status_cmd.rs`
- Test: modules above

**Interfaces:**
- Produces: identical `gzmo.evolution.status/v1` through `gzmo evolve status`, `gzmo status`, and `gzmo_evolution_status`.
- Consumes: connected or appliance state adapter; no mutation.

- [ ] **Step 1: Write status consistency tests**

Serialize one fixture status through CLI, ecosystem board, and MCP tool; parse JSON and assert the same candidate/state/audit/budget/authority values.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-core evolution_status`

Expected: FAIL: integration missing.

- [ ] **Step 3: Define required fields**

```text
mode: connected_repo | airgapped_appliance
state: lifecycle state
mission_id / candidate_id
baseline / candidate artifact digest
active authority tier and required next authority
budget max / used / remaining
hard-floor summary and evaluation digest
one-writer / airgap / audit verdict
projection watermarks when appliance
PR URL/checks when connected
last-known-good / rollback readiness
last attempt / next eligible run / stop reason
```

Never report GREEN when a required field is unknown; use `HOLD` or `FAIL` with reason.

- [ ] **Step 4: Keep MCP read-only**

MCP accepts no args beyond optional candidate ID and exposes no apply/sign/merge/rollback mutation. Operator actions remain local CLI/physical signing workflow.

- [ ] **Step 5: Run status tests**

Run: `cargo test -p gzmo-core evolution_status && cargo test -p gzmo-cli status_cmd`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add gzmo-core gzmo-cli
git commit -m "feat: expose one honest continuous evolution status"
```

---

### Task 4: Bridge Stages with Signed Offline Bundles

**Files:**
- Create: `gzmo-evolver/src/bundle.rs`
- Create: `gzmo-core/src/evolution/bundle.rs`
- Create: `crates/evolution-contracts/src/bundle.rs`
- Modify: `crates/evolution-contracts/src/lib.rs`
- Test: `crates/evolution-contracts/tests/bundle.rs`
- Test: `gzmo-core/tests/evolution_sandbox.rs`

**Interfaces:**
- Produces: `CandidateBundleManifest`, deterministic tar.zst export/import verification.
- Consumes: candidate manifest/artifact/evaluation/audit, operator-signed target metadata.

- [ ] **Step 1: Write tamper and credential-exclusion tests**

Bundle fixture contains:

```text
manifest.json
evaluation.json
audit-proof.json
attestation.json
artifact/<content-addressed files>
TUF/targets.json
TUF/snapshot.json
```

Assert deterministic ordering/timestamps, no symlink/device/absolute/parent path, no credential/private-key file, exact size/hash, complete snapshot, non-decreasing epoch, and signature requirement for import.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p evolution-contracts bundle`

Expected: FAIL: bundle types missing.

- [ ] **Step 3: Implement connected export**

Export only an evaluated/merged commit or an explicitly operator-selected candidate. The bundle contains no GitHub token, App key, OMP profile, raw logs, hidden holdouts, `.git`, or network config. Artifact provenance binds commit/tree/toolchain/evaluation digests.

- [ ] **Step 4: Implement appliance import**

Import from mounted courier into quarantine. Verify trust metadata, epoch/freshness, hashes/sizes, license, architecture/profile, policy/evaluation binding, and disk budget before materializing. Import creates/updates a Candidate in PromotionPending; it never promotes.

- [ ] **Step 5: Implement outbound evidence export**

Appliance may export sanitized status/evaluation/audit proofs to courier for company review. It never exports living facts, prompts, credentials, recovery keys, or private holdouts by default.

- [ ] **Step 6: Run bundle tests**

Run: `cargo test -p evolution-contracts bundle && cargo test -p gzmo-core evolution_bundle`

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/evolution-contracts gzmo-evolver gzmo-core
git commit -m "feat: bridge evolution stages with signed offline bundles"
```

---

### Task 5: Add Cross-Stage Contract and Chaos Gates

**Files:**
- Create: `scripts/evolution-contract-check.sh`
- Create: `tests/evolution-contract-test.sh`
- Create: `scripts/evolution-chaos-drill.sh`
- Test fixtures: `tests/fixtures/evolution/`

**Interfaces:**
- Produces: `gzmo.evolution.contract_check/v1` and `gzmo.evolution.chaos_drill/v1` artifacts.
- Consumes: schemas, status outputs, fixture runner/controller.

- [ ] **Step 1: Write failing contract fixture test**

Create one valid and malformed candidate/evaluation/audit/status fixture. Test both connected and appliance parsers accept valid bytes and reject schema mismatch, illegal transition, digest tamper, unknown authority, missing hard gate, or stale grant.

- [ ] **Step 2: Run to verify failure**

Run: `bash tests/evolution-contract-test.sh`

Expected: FAIL: checker missing.

- [ ] **Step 3: Implement contract checker**

Run schema export drift check, parse latest connected/appliance status if present, verify audit chain, assert one active candidate maximum, check budget/floor fields, and emit PASS/HOLD/FAIL rows. Missing optional mode is HOLD; malformed existing mode is FAIL.

- [ ] **Step 4: Implement fixture-only chaos drill**

Faults:

- kill worker/evaluator/controller mid-transition;
- corrupt audit event;
- expire envelope/grant;
- create second candidate race;
- remove GitHub/model/PostgreSQL availability;
- exceed tokens/time/disk/thermal fixture values;
- lag/corrupt Qdrant/Neo4j/Redis projection;
- fail A/B candidate health and invoke fake rollback.

Never run destructive faults against living state without a separate explicit live flag and backup/target verification.

- [ ] **Step 5: Run gates**

```bash
bash tests/evolution-contract-test.sh
bash scripts/evolution-contract-check.sh
bash scripts/evolution-chaos-drill.sh --fixture
```

Expected: all exit 0; injected failures are detected and expected recovery receipts exist.

- [ ] **Step 6: Commit**

```bash
git add scripts/evolution-* tests/evolution-* tests/fixtures/evolution
git commit -m "test: verify continuous evolution contracts and recovery"
```

---

### Task 6: Shadow Existing Evolve Paths and Prove Parity

**Files:**
- Modify: `scripts/ecosystem-evolve-daily.sh`
- Modify: `scripts/ecosystem-evolve-weekly.sh`
- Modify: `scripts/opportunity-next-mission.sh`
- Create: `scripts/evolution-shadow-compare.sh`
- Create: `tests/evolution-shadow-test.sh`

**Interfaces:**
- Produces: parity report comparing legacy mission/opportunity outputs with new observer/runner outputs.
- Consumes: existing scripts and new status; no apply.

- [ ] **Step 1: Write fixture parity test**

Given the same bet log and health fixtures, assert legacy and new paths select the same active mission ID, ship-bar eligibility, and blocking hard-floor class. Differences in timestamps/display text are ignored; semantic differences fail.

- [ ] **Step 2: Run to verify failure**

Run: `bash tests/evolution-shadow-test.sh`

Expected: FAIL: comparator missing.

- [ ] **Step 3: Add shadow-only invocation**

Legacy daily/weekly scripts optionally call new observer/runner with `--shadow --stop-before-build`; they never activate a second candidate. Store both payloads and comparison under `data-next/evolution-shadow/`.

- [ ] **Step 4: Define parity criteria**

Require: same active bet, no extra mission, hard failure never downgraded, no credential/network on appliance, one candidate lock, and new output includes all legacy evidence pointers. New stricter HOLD/FAIL is acceptable only with explicit reason.

- [ ] **Step 5: Run fixture and manual shadow**

```bash
bash tests/evolution-shadow-test.sh
GZMO_EVOLUTION_SHADOW=1 bash scripts/ecosystem-evolve-weekly.sh
bash scripts/evolution-shadow-compare.sh
```

Expected: fixture PASS; manual report has no unexplained semantic divergence.

- [ ] **Step 6: Commit**

```bash
git add scripts/ecosystem-evolve-* scripts/opportunity-next-mission.sh scripts/evolution-shadow-compare.sh tests/evolution-shadow-test.sh
git commit -m "test: shadow constitutional evolution against legacy loops"
```

---

### Task 7: Retire Duplicate Legacy Execution After Soak

**Files:**
- Delete: `gzmo-cli/src/idle_evolve.rs`
- Modify: `gzmo-cli/src/main.rs`
- Modify: `gzmo-cli/src/daemon_cmd.rs:424-487` (remove script-spawn block; retain heartbeat)
- Delete after parity: `systemd/user/gzmo-ecosystem-evolve-daily.service`
- Delete after parity: `systemd/user/gzmo-ecosystem-evolve-daily.timer`
- Delete after parity: `systemd/user/gzmo-ecosystem-evolve-weekly.service`
- Delete after parity: `systemd/user/gzmo-ecosystem-evolve-weekly.timer`
- Modify: `scripts/install-ecosystem-evolve-timers.sh`
- Preserve: `scripts/opportunity-sense.sh`, `opportunity-rank.sh`, `opportunity-bet.sh`, `opportunity-next-mission.sh` until a separately reviewed Rust parity cutover
- Create: `docs/CONTINUOUS_EVOLUTION.md`

**Interfaces:**
- Produces: one connected runner and one owner-integrated appliance controller; no duplicate scheduler/apply path.
- Consumes: four weekly Stage 1 checkpoints, offline soak, parity report.

- [ ] **Step 1: Verify retirement prerequisites**

Require signed/committed evidence of:

- four consecutive weekly Stage 1 terminal outcomes;
- no direct main push or auto-merge;
- one offline Memory candidate accepted;
- one Tunable accepted and one forced rollback;
- one hard Candidate stopped at PromotionPending;
- stop-file and audit-tamper drills;
- shadow semantic parity.

If any is missing, stop without deleting.

- [ ] **Step 2: Write a test that forbids duplicate execution**

Search compiled/module/service configuration and assert exactly one connected candidate timer and one appliance evolution scheduler. Ensure no daemon code invokes `living-research-intel.sh` directly.

- [ ] **Step 3: Remove idle script spawning**

Keep CheapCheck heartbeat. Delete `idle_evolve` module/import/tests and lines that write `last-idle-evolve` and spawn `living-research-intel.sh`. Research scripts may remain manually invocable but cannot schedule or mutate.

- [ ] **Step 4: Remove superseded ecosystem timers**

Update installer to install new system-level repo-evolver units and preserve independent ops/research/health timers only when they retain distinct responsibilities. Remove daily/weekly wrapper units and scripts only when no documented/manual consumer needs them; otherwise convert scripts to read-only report commands with no scheduling.

- [ ] **Step 5: Write the operator runbook**

`docs/CONTINUOUS_EVOLUTION.md` covers connected vs appliance identity, one-candidate lifecycle, cadence, status, GitHub App permissions, local OMP worker/model namespace, courier bundles, authority tiers, stop/disable, audit verification, PR review, signing, rollback, recovery, and exact non-goals.

- [ ] **Step 6: Run clean-cutover tests**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
bash tests/evolution-contract-test.sh
bash tests/evolution-shadow-test.sh
bash scripts/evolution-contract-check.sh
bash scripts/evolution-chaos-drill.sh --fixture
```

Expected: all pass; duplicate execution test reports one path per mode.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor: retire duplicate evolution schedulers"
```

---

### Task 8: Enable Continuous Operation Through Explicit Gates

**Files:**
- Runtime state only for enablement
- Add dated evidence: `research/evolution-soak/YYYY-MM-DD-hybrid-evolution-acceptance.md`

**Interfaces:**
- Produces: enabled connected cadence and enabled bounded appliance cadence with operator-owned acceptance record.
- Consumes: all preceding gates/runbooks.

- [ ] **Step 1: Enable connected runner manually**

```bash
REPO_EVOLVER_ENABLE=1 sudo bash scripts/install-repo-evolver.sh --enable
sudo systemctl start gzmo-repo-evolver.service
```

Expected: at most one draft/review PR; no main update; status/audit complete.

- [ ] **Step 2: Observe four weekly connected outcomes**

Each candidate must reach Rejected, Failed, or human-merged Accepted. No orphan Building/Evaluating state; no third repair; no credential finding.

- [ ] **Step 3: Enable appliance observation only**

Set `[evolution].enabled=true` with hard-candidate cadence disabled by signed envelope. Run daily observation for one week; compare resource and hard-floor status.

- [ ] **Step 4: Enable Memory/Tunable authority**

Import operator-signed envelope allowing exact keys/ranges and frequency. Run one Memory and one Tunable candidate; force a Tunable floor failure and verify automatic rollback.

- [ ] **Step 5: Enable hard-candidate generation**

Only after sandbox/authority drills. Generate one code candidate offline. Expected terminal state: PromotionPending; no apply without separately signed grant.

- [ ] **Step 6: Record acceptance**

Write the dated evidence note with hashes, candidate IDs, PRs, gate reports, authority decisions, stop/rollback outcomes, resource consumption, limitations, and explicit go/no-go for continued cadence.

- [ ] **Step 7: Final verification**

Run connected and appliance `status --json`; verify one active candidate maximum, audit chains valid, next cadence explicit, rollback ready, and no forbidden credentials/endpoints cross modes.

- [ ] **Step 8: Commit the hybrid acceptance evidence**

```bash
git add research/evolution-soak/
git commit -m "docs: accept bounded continuous evolution cadence"
```
