# 06 — Live evidence reachability

## Scope

Read-only probe from this workstation (Windows 11 Enterprise, session cwd outside the living LAN) of what **live** GZMO living-host evidence can be observed versus what must remain Doc-dated or Unreachable. No services started, no gate writers run, no CT101 mutation. Active bet `felt-use-mass-growth` is horizon context only.

Probe axes:

1. SSH / `ct101` host reachability and local living vault / gate artifact presence
2. Gate script offline vs host-dependent modes (source-only inspection)
3. Committed dated living-count snapshots
4. Binary deploy lag / `#166` / `harvest-organs` signals without host access

## Contract inventory

### Reachability matrix

| Evidence class | Path / probe | Status | Label | Notes |
|----------------|--------------|--------|-------|-------|
| OpenSSH client | `C:\WINDOWS\System32\OpenSSH\ssh.exe` | present | **Observed** | Client exists |
| `~/.ssh` / `Host ct101` config | `$HOME/.ssh`, `$USERPROFILE/.ssh` | absent | **Observed** | No SSH dir; `ssh -G ct101` uses bare hostname `ct101`, default user, port 22 |
| DNS `ct101` | `ssh ct101`, `nslookup ct101` | fail | **Unreachable** | “Host unbekannt” / NXDOMAIN; Quad9 DNS timeouts |
| CT101 LXC IP | ICMP `192.168.31.202` | fail | **Unreachable** | 100% loss (1 probe) |
| VM200 embed host | ICMP `192.168.31.110` | fail | **Unreachable** | 100% loss (1 probe) |
| `/opt/gzmo` living root | local FS | absent | **Unreachable** | No path on this OS image |
| `~/.gzmo-living/data/vault.db` | local FS | absent | **Unreachable** | No Keep vault on this workstation |
| `target/release/gzmo` / `~/.local/bin/gzmo` | local FS | absent | **Unreachable** | No living/lab binary in clone or home |
| `data-next/keep-quality/` | clone | absent | **Observed** | Dir not present; only committed placeholders under `data-next/` |
| `data-next/brain-feed/` | clone | absent | **Observed** | Absent |
| `data-next/felt-use-depth/` | clone | absent | **Observed** | Absent |
| `data-next/{memory,sessions,distill-queue}/` | clone | empty `.gitkeep` only | **Observed** | No runtime JSON |
| Gate scripts (source) | `scripts/{felt-use-depth,brain-feed-check,keep-quality-gate,living-readiness-gate}.sh` | readable | **Observed** | Contracts inspectable offline |
| Live gate artifacts (`latest.json`) | would be written under gitignored `data-next/*` | not run | **Unreachable** | Running gates would create artifacts; assignment forbids writers / live gate execution |
| CT101 honeypot / daemon / ripen live counts | SSH + vault SQL | — | **Unreachable** | Requires host |
| Binary on `/opt/gzmo/current` vs main `#166` | remote `PATH` / binary | — | **Unreachable** | Deploy lag not measurable here |
| Living-count tables in docs/research | see inventory below | committed | **Doc-dated** | Historical snapshots only |

### SSH / host config (Observed)

- **Observed:** `ssh.exe` is installed. There is **no** `~/.ssh` directory and **no** committed or home `Host ct101` stanza with `ProxyJump`/`Hostname` (docs describe `ssh ct101` with ProxyJump via `pve` in `docs/CT101_DEPLOY.md` L49–52 and `config/openclaw-workspace/TOOLS.ecosystem.md` L8–12).
- **Observed:** `ssh -G ct101` expands to `hostname ct101`, `port 22`, current Windows user `ad001\z005a5ff`, pubkey preferred — i.e. default client behavior, not a living-host alias.
- **Unreachable:** Interactive or BatchMode SSH to living host. First attempt: `Could not resolve hostname ct101`.

### Local / ignored artifacts (Observed)

- **Observed:** `.gitignore` L42–52 ignores `data-next/*` while keeping only `.gitkeep` under `data-next/`, `memory/`, `sessions/`, `distill-queue/`.
- **Observed:** Clone `data-next/` contains only those empty keepers. **No** `keep-quality/`, `brain-feed/`, or `felt-use-depth/` directories or `latest.json` / `latest.md` artifacts.
- **Observed:** No local living vault at `~/.gzmo-living` and no `/opt/gzmo/data/vault.db`.
- **[INFERENCE]:** Prior research on other Keeps (`research/felt-use-shipped-vs-opportunity.md`) describes gitignored depth JSON on a Linux operator host; that state is **not** present on this Windows session host.

### Gate script modes without live host (Observed source contracts)

| Script | Offline / skip knobs | Without local vault + without SSH | Meaningful living GREEN? |
|--------|----------------------|-----------------------------------|---------------------------|
| `scripts/felt-use-depth.sh` | Prefers `$HOME/.gzmo-living/data/vault.db` if file exists (`L20–25`); else default vault `/opt/gzmo/data/vault.db` via SSH (`run_host` L57–62). No fixture mode. | Census `ok: false` → verdict **RED**, advice `felt_use_depth_unreachable` (`L167–170`); writes `data-next/felt-use-depth/latest.*` | **No** — needs local vault file **or** SSH |
| `scripts/brain-feed-check.sh` | Local subchecks (herdr, tinyfolder, serendipity, docs) + always runs depth script. Thin depth re-queries honeypot via `ssh … ct101` (`L94–110`). | Depth FAIL if census not ok; thin path SSH FAIL → `felt-use` FAIL | **No** full Brain Feed GREEN without host or prior local depth JSON + other local nutrients |
| `scripts/keep-quality-gate.sh` | `KEEP_QUALITY_SKIP_LIVING_READY=1` HOLDs readiness only (`L49–50`); `LIVING_GATE_SKIP_TAKEAWAY=1` documented. Still SSHes spark/immune/ripen/lymph/health (`L104+`, `L154+`, `L183+`, `L202+`, `L232`). | Living-readiness FAIL (smoke/SSH); felt-use falls back to SSH SQL (`L84–99`) then FAIL; organs FAIL/HOLD | **No** — skip flags do not create offline GREEN living USP |
| `scripts/living-readiness-gate.sh` | `LIVING_GATE_SKIP_TAKEAWAY=1` only. Runs `ct101-living-smoke.sh` + multi-probe `ssh_ct` health (`header L1–79 region`). | Dual-writer row can PASS if local `gzmo-serve` inactive; smoke/health **FAIL** without CT101 | **No** LIVING GREEN offline |
| Faithfulness fixtures | `scripts/fixtures/faithfulness-*.json` | Fixture data for separate faithfulness checks — **not** a substitute living census | N/A for living counts |

Hard-coded defaults (host-shaped even when local short-circuit exists):

- `CT101_SSH_HOST` default `ct101` — `felt-use-depth.sh` L19, `brain-feed-check.sh` L10, `keep-quality-gate.sh` L15, `living-readiness-gate.sh` L12.
- `KEEP_QUALITY_VAULT_DB` / data defaults under `/opt/gzmo/data` — keep-quality L17–18; brain-feed L11; felt-use-depth fallback L24.
- Binary default `/opt/gzmo/current/target/release/gzmo` unless `~/.local/bin/gzmo` executable (`felt-use-depth.sh` L26–29).

**Observed (source):** Thin depth is intentionally **HOLD not RED** when census succeeds (`felt-use-depth.sh` L166–187; `brain-feed-check.sh` L94). Unreachable census is RED.

**[INFERENCE]:** On this workstation, invoking the gates without a vault would only prove RED/FAIL artifact emission under gitignored `data-next/`; it would not yield live living counts. Gates were **not** executed (read-only assignment).

### Committed living-count snapshots (Doc-dated)

| Snapshot | Date | Counts (as written) | Source path |
|----------|------|---------------------|-------------|
| Memory data plane SYSTEM table | **2026-07-14** | semantic_vault **60,031**; honeypot latest **37,807**; Qdrant honeypot **24,322**; Neo4j **63,572** nodes | `docs/ct101-systems/50-memory-data-plane/SYSTEM.md` L11–16 |
| CT101 vault archaeology | **2026-07-20** | semantic_vault **61,081**; honeypot latest **38,730**; honeypot all **48,217**; evidence **48,014**; quarantine **1,012** | `research/ct101-vault-archaeology-2026-07-20.md` L13–17 |
| Felt Use ripen floor baseline | **2026-07-20** | latest≈**38743** / ge1≈**107** / ge3≈**60** | `research/opportunities/felt-use-ripen-floor.md` L32; echoed in `felt-use-depth.sh` `baseline_note` L199 |
| Living external attach measure | **2026-07-22** | CT101 vault_facts≈**61750**, honeypot≈**39835**; lab `~/.gzmo` ≈**783** | `research/living-external-attach-plug-and-play-2026-07-22.md` L19–24 |
| Keep census (other host, not this workstation) | **2026-08-30** | honeypot latest **3005**; recall≥1 **104**; recall≥3 **79**; utility_positive **78**; vault_facts **4711**; HOLD vs floor ge3≥100 | `research/felt-use-shipped-vs-opportunity.md` L45–52 |
| Brain Feed / keep-quality soak notes | **2026-07-20** | BF check GREEN 8/8; soak trailing GREEN 3/3 — refer to then-current gitignored artifacts | `research/brain-feed-2026-07-20.md`; `research/keep-quality-soak-2026-07-20.md` |

Root `SYSTEM.md` is **not** present in this clone (**Observed** absence). Living counts live under `docs/ct101-systems/**/SYSTEM.md` and dated research notes.

### Binary deploy lag / `#166` / harvest-organs (Doc-dated; live Unreachable)

| Claim | Where | Label |
|-------|-------|-------|
| Living mass done-when 2–3 “remains CT101-only until `#166`/`harvest-organs` binaries are on `/opt/gzmo/current`” | `research/opportunities/felt-use-mass-growth.md` Telescope **2026-08-16** L59–61 | **Doc-dated** |
| Same line called **stale leftover**; `#166`/`#167`/`#193` on `origin/main`; mechanism already on a Keep without `/opt/gzmo` | `research/felt-use-shipped-vs-opportunity.md` L15, L29, L91–98 | **Doc-dated** (research correction; not live CT101 proof) |
| `harvest-organs` / `#166` on `/opt/gzmo/current` not a binary in tree / not this Keep’s mass path | `research/felt-use-ingest-path.md` L28, L130 | **Doc-dated** |
| Production binary path `/opt/gzmo/current/target/release/gzmo`; build **on CT101** (workstation glibc newer → `GLIBC_2.39 not found` if scp’d) | `docs/CT101_DEPLOY.md` L11–12, L55–68 | **Doc-dated** contract |
| Script-only sync without daemon restart via `scripts/ct101-brain-feed-sync.sh` | `docs/CT101_DEPLOY.md` L84–94 | **Doc-dated**; script present in tree (**Observed** path) but cannot run against host |
| Actual CT101 binary age, `readlink /opt/gzmo/current`, presence of utility/Q-select in remote binary, harvest-organs on remote PATH | would need SSH | **Unreachable** |

**[INFERENCE]:** Deploy-lag relative to main cannot be asserted from this workstation; only the **documented** risk (rsync + on-host `cargo build --release`, glibc skew, telescope wording vs later “stale” correction) is available.

## Gaps and drift

1. **Live living-host evidence is entirely unreachable** from this session host: no DNS/LAN to CT101/VM200, no SSH alias/keys, no local living vault or `gzmo` binary.
2. **Gate scripts are not offline living fixtures.** Skip env vars narrow rows; they do not fabricate census. Without vault file, `felt-use-depth.sh` is contractually RED (`felt_use_depth_unreachable`).
3. **No residual gate artifacts** in the clone’s `data-next/` (gitignored runtime empty). Diagnosis cannot reuse a prior `latest.json` from this tree.
4. **Living population numbers in-repo are Doc-dated** (primarily 2026-07-14 … 2026-07-22 CT101; 2026-08-30 other-Keep HOLD). None are live as of this probe date.
5. **`#166` / harvest-organs / `/opt/gzmo/current` binary lag** is a **doc/telescope** seam: opportunity file still states CT101-only mass until those binaries land; later research marks that sentence stale for mechanism-on-main. **Neither** live binary inventory **nor** lag delta is Observable here.
6. Docs still teach `ssh ct101` + `/opt/gzmo/...` as the operator path (`MACHINE.md` L50–51, `CT101_DEPLOY.md`, OpenClaw TOOLS) while this workstation has neither name resolution nor keys — operator-path drift vs actual reachability.

## Evidence status

| Label | Meaning in this brief |
|-------|------------------------|
| **Observed** | Directly inspected on this workstation or read as first-party source/artifact in the clone |
| **Unreachable** | Live host/network/binary/vault fact required; probe failed or resource absent; not guessed |
| **Doc-dated** | Numbers or deploy claims taken from committed docs/research with explicit dates; not re-measured live |
| **[INFERENCE]** | Reasoned only where marked; not used as live proof |

Probe constraints honored: no gate execution (would mkdir/write under `data-next/`), no SSH writers, no service starts. DNS/SSH/ping failures accepted after single attempts.

## Sources

- Workstation probes (2026-08-31 session): `ssh -G ct101`; `ssh -o BatchMode=yes -o ConnectTimeout=3 ct101`; `nslookup ct101`; ICMP `192.168.31.202` / `192.168.31.110`; FS checks for `~/.ssh`, `~/.gzmo-living`, `/opt/gzmo`, `target/release/gzmo`, `data-next/**`
- `scripts/felt-use-depth.sh` (local vault short-circuit L20–30, L57–62; RED unreachable L167–170; baseline_note L199)
- `scripts/brain-feed-check.sh` (depth + thin SSH fallback L84–115)
- `scripts/keep-quality-gate.sh` (skip living-ready L49–50; SSH organs; felt-use SSH fallback L84–100)
- `scripts/living-readiness-gate.sh` (CT101 smoke + health SSH; takeaway skip only)
- `.gitignore` L42–52 (`data-next/*`)
- `docs/ct101-systems/50-memory-data-plane/SYSTEM.md` L11–16 (2026-07-14 live count table)
- `docs/CT101_DEPLOY.md` L1–120 (paths, binary deploy, script sync)
- `docs/CT101_PATH_AUTHORITY.md` (release tree / binary authority)
- `research/ct101-vault-archaeology-2026-07-20.md` L13–17
- `research/opportunities/felt-use-ripen-floor.md` L32–34
- `research/opportunities/felt-use-mass-growth.md` L59–61 (telescope `#166`/harvest-organs)
- `research/felt-use-shipped-vs-opportunity.md` L13–61, L91–98 (stale telescope; 2026-08-30 Keep census)
- `research/felt-use-ingest-path.md` L28, L130
- `research/living-external-attach-plug-and-play-2026-07-22.md` L19–24
- `research/brain-feed-2026-07-20.md`; `research/keep-quality-soak-2026-07-20.md`
- `MACHINE.md` L50–51; `config/openclaw-workspace/TOOLS.ecosystem.md` L8–12
- `scripts/ct101-brain-feed-sync.sh` (path presence); `scripts/fixtures/faithfulness-*.json` (fixture scope)
