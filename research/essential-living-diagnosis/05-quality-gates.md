# Quality / Brain Feed / Felt-use gate contracts

## Scope

Primary-source inventory of product **measurement** scripts that bind (or merely report toward) living Keep quality and the active bet `felt-use-mass-growth`. Sources are first-party scripts under `scripts/`, doctrine under `docs/`, and the opportunity note. No live gates were executed; no scripts or configs were modified. ADR-0010 is horizon context only (demo-enqueue removal comment in Brain Feed). Active-bet boundary: diagnose contracts only — do not replace or kill `felt-use-mass-growth`.

## Contract inventory

### 1. `keep-quality-gate.sh` — USP living quality bar

**Observed** (`scripts/keep-quality-gate.sh`, `docs/KEEP_QUALITY.md`, ADR-0004/ADR-0007).

| Item | Contract |
|------|----------|
| Role | Continuous **living** quality bar on the airgap writer host — not “binary installs” |
| Default remote | `CT101_SSH_HOST=ct101`, vault `/opt/gzmo/data/vault.db`, data `/opt/gzmo/data` |
| Binary | `CT101_GZMO_BIN` default `/opt/gzmo/current/target/release/gzmo` |
| Artifacts | `data-next/keep-quality/latest.json`, `latest.md`, `gate.log` |
| Schema | `gzmo.keep.quality/v1` |
| Verdict | `GREEN` iff `fail == 0`; **HOLD does not fail** the gate (`ok = fail_n == 0`) |
| Env knobs | `LIVING_GATE_SKIP_TAKEAWAY` (compose living-readiness), `KEEP_QUALITY_SKIP_LIVING_READY=1` (organs only → living-readiness **HOLD**), `KEEP_QUALITY_MIN_NONZERO_RECALL` (default 1), `KEEP_QUALITY_MIN_SPARK_UNIQUE` (default 2), `KEEP_QUALITY_SPARK_LAST_N` (default 8) |

**Pillars (row name → prove vs report):**

| Row | Proves (can FAIL gate) | Merely reports / soft |
|-----|------------------------|------------------------|
| `living-readiness` | Composes `living-readiness-gate.sh`; **PASS** only if `data-next/living-readiness/latest.json` `verdict==GREEN`; else **FAIL** | Skip env → **HOLD** only |
| `felt-use` | Latest honeypot `recall_count>0` count ≥ `MIN_NONZERO_RECALL` (default 1), via `felt-use-depth` census or direct `sqlite3` fallback | Counts/detail only after floor met |
| `felt-use-depth` | — | Soft: `depth_ok` → **PASS**, else **HOLD** (thin ≠ RED). Missing artifact → **HOLD** |
| `spark-refractory` | Last-N anchors unique ≥ `MIN_SPARK_UNIQUE` → **PASS**; window present but unique short → **FAIL** (“monoculture risk”) | Missing/empty/unparsed refractory → **HOLD** |
| `immune` | — | Plan present + candidate count: candidates=0 **PASS**, candidates>0 or missing **HOLD**. **Never FAIL** |
| `ripen` | `gzmo ripen status` on living host must emit parseable `Nonzero recall_count`; command fail → **FAIL** | `Starved` / unclear → **HOLD** (honest starved_recall); nonzero + not starved → **PASS** |
| `night-lymph` | — | `/opt/gzmo/data/night-lymph/latest.json` present → **PASS**; missing → **HOLD** |
| `mcp-attach` | `living-mcp-attach-check.sh` artifact `ok`; mislabel path → **FAIL** | `ok` but `found_living==0` → **HOLD** |
| `airgap-honesty` | `gzmo health` log: `prime_llm`+`embeddings` **PASS**; cloud OK without prime → **FAIL**; no prime → **FAIL** | prime OK, embeddings not → **HOLD** |

Doctrine table in `docs/KEEP_QUALITY.md` matches these pillars (ops, felt recall floor via living-readiness compose, Felt Use, soft depth, spark, immune, ripen honesty, lymph, attach, airgap).

### 2. `keep-quality-soak.sh` — honest-night rules

**Observed** (`scripts/keep-quality-soak.sh`, `docs/KEEP_QUALITY.md` §Unpark gate).

| Rule | Contract |
|------|----------|
| Sample | Runs `keep-quality-gate.sh`, appends full gate payload + `soak_rc` to `data-next/keep-quality/soak-log.jsonl` |
| Exit | Same as gate (`exit $rc`) — sample only “counts” as GREEN night if gate GREEN |
| Default soak loop | `LIVING_GATE_SKIP_TAKEAWAY` defaults to `1` |
| `--summary` floors | `KEEP_QUALITY_SOAK_NIGHTS` default **3**; `KEEP_QUALITY_SOAK_MIN_HOURS` default **18** |
| Honest trail | Newest→oldest: count GREEN only when ≥ min hours before previously counted sample; non-GREEN breaks trail; too-close GREENS increment `spacing_rejects`, do **not** inflate `honest_nights` |
| Advice | `soak_ready_unpark_ok` if `honest_nights >= need`; else if raw trailing GREEN ≥ need but honest short → `soak_spacing_hold …`; else `need_N_trailing_honest_GREEN_have_H` |
| Doctrine use | Unpark Wave 1 must not expand as brand work until honest soak history; same-hour streaks → HOLD not ready |

**Proves:** spacing-honest multi-sample keep-quality GREEN history for Unpark readiness advice.  
**Does not prove:** multi-night biology by itself if samples are same-day (explicit note in `research/keep-quality-soak-2026-07-20.md` Doc-dated 2026-07-20).

### 3. `brain-feed-check.sh` — P0 nutrient rows + dual-writer

**Observed** (`scripts/brain-feed-check.sh`, `docs/BRAIN_FEED.md`).

| Item | Contract |
|------|----------|
| Role | Operator/telescope check that Unpark **nutrient** satellites demable toward living vault; **does not replace** keep-quality / soak (`BRAIN_FEED.md` hard rule 4) |
| Artifacts | `data-next/brain-feed/latest.json`, `latest.md`, `gate.log` |
| Schema | `gzmo.brain_feed.check/v1` |
| Verdict | `GREEN` iff `fail == 0`; HOLD allowed |
| Side effect | Runs `distill-queue-drain.sh --apply --older-than 7` (archive aged queue; non-destructive to vault) |

**Rows:**

| Row | Status contract | Proves vs reports |
|-----|-----------------|-------------------|
| `dual-writer` | Workstation `systemctl --user is-active gzmo-serve.service` == `active` → **FAIL**; else **PASS** | **Proves** one-writer hygiene for feed path (hard rule 1 / ADR-0003) |
| `herdr-takeaway` | `herdr-metabolism/latest.json` `ok` → **PASS**; else **HOLD** | **Reports** optional plugin surface |
| `takeaway-recall` | `ct101-takeaway-recall/latest.json` `living_proof` → **PASS**; else **HOLD** | **Reports** living same-sitting HIT artifact if present |
| `takeaway-side-effect` | `takeaway-side-effect/latest.json` `ok` → **PASS**; else **FAIL** | **Proves** remind surfaces exist (PR template / hook path) — not that takeaways ran |
| `tinyfolder-living` | `tinyfolder/living-enqueue.json` `ok` → **PASS**; else **FAIL** | **Proves** living-enqueue artifact contract; demo enqueue removed 2026-08-23 (ADR-0010 Phase 1 comment) |
| `felt-use` | Census `ok` + (`depth_ok` **or** nonzero ≥ min) → **PASS**; else **FAIL** | **Proves** nonzero felt floor (P0); thin depth still needs nonzero |
| `felt-use-depth` | `depth_ok` → **PASS**; thin → **HOLD**; census fail → **FAIL** | Soft depth floor; thin ≠ Brain Feed RED |
| `serendipity-cadence` | `serendipity/cadence-latest.json` `ok` → **PASS**; else **FAIL** | **Proves** cadence script produced ok artifact |
| `serendipity-promote` | promote-latest dry-run/apply ok; 0 candidates dry-run → **HOLD**; fail → **FAIL** | Mixed: pipeline runnable vs candidate mass |
| `serendipity-apply-proof` | apply-proof or non-stale cadence applies → **PASS**; else **HOLD** | **Reports** human apply closed loop; no auto-apply |
| `dream-compact` | `dream-compact-lab.sh` executable → **PASS**; missing → **HOLD** | Presence only; doctrine: soft / off GREEN math intent — script still **PASS**es when file exists (does not force FAIL) |
| `doctrine` | `docs/BRAIN_FEED.md` present → **PASS**; else **FAIL** | Doc file exists |

Doctrine P0 table (`BRAIN_FEED.md`): herdr+takeaway, tinyFolder, Memory MCP Felt Use, Felt Use depth floor, Serendipity promote-back, Dream compaction (soft). P1 intel promote is **not** a hard Brain Feed gate row (separate `brain-intel-promote.sh`).

### 4. `felt-use-depth.sh` — depth + utility census

**Observed** (`scripts/felt-use-depth.sh`).

| Item | Contract |
|------|----------|
| Role | Side-effect measurement only — **does not** run memory-gym searches |
| Artifacts | `data-next/felt-use-depth/latest.{json,md}` |
| Schema | `gzmo.brain_feed.felt_use_depth/v1` |
| Vault resolution | Prefer local file `~/.gzmo-living/data/vault.db` if present and `KEEP_QUALITY_VAULT_DB` unset; else SSH/`KEEP_QUALITY_VAULT_DB` default `/opt/gzmo/data/vault.db` |
| Binary | Prefer `~/.local/bin/gzmo` if executable and `CT101_GZMO_BIN` unset; else `/opt/gzmo/current/target/release/gzmo` |

**Census SQL fields (latest honeypot):**

| Field | Meaning |
|-------|---------|
| `latest` | `COUNT(*) WHERE is_latest=1` |
| `recall_ge1` | `recall_count >= 1` |
| `recall_ge3` | `recall_count >= 3` |
| `share_ge1` | ge1 / latest |
| `share_ge3` | **ge3 / ge1** (felt denominator; honest nutrient signal) |
| `share_ge3_of_latest` | ge3 / latest — **trend only**, not the floor |
| `utility_positive` | `utility_score > 0` count |
| `utility_avg` / `utility_max` | avg/max utility on latest |

**Floors / verdict:**

- Defaults: `FELT_USE_MIN_GE3=100`, `FELT_USE_MIN_SHARE_GE3=0.40`
- `depth_ok` ⇔ `recall_ge3 >= min_ge3` **and** `share_ge3 >= min_share`
- Census unreachable → `verdict=RED`, `ok=false` (exit 1)
- Depth thin → `verdict=HOLD`, **`ok=true`** (exit 0) — honest thin ≠ gate RED
- Depth ok → `verdict=GREEN`, `ok=true`
- Ripen dual/dual_origin/nonzero/starved parsed into payload **report**; **not** part of `depth_ok` math

Utility fields are **reported in advice/census** when depth ok; they are **not** independent FAIL floors inside this script.

### 5. `product-readiness-gate.sh` — non-product (ADR-0007)

**Observed** (`scripts/product-readiness-gate.sh`, `docs/PRODUCT_PRODUCTION_READINESS.md`, `docs/ADR-0007-one-product-living.md` L40).

| Item | Contract |
|------|----------|
| Role | Historical `~/.gzmo` / `gzmo-memory` **client-attach smoke** |
| Explicit non-product | ADR-0007: quality bar is `keep-quality-gate.sh` on living host; this gate is **not** a second product GREEN |
| Binary default | Local `GZMO_BIN` / cargo target release — **not** `/opt/gzmo/current` |
| Schema | `gzmo.product.readiness/v1` |
| Verdict | GREEN iff no FAIL; HOLD ok for engine / optional CT101 / release lag |
| Payload note | `"Product GREEN = stranger laptop MCP path. Living CT101 is separate owner lane."` |

**Rows (summary):** `gzmo-binary`, `verify-product-mcp`, `mcp-attach` (product), `product-config-hygiene` (FAIL if LAN/CT101 in `~/.gzmo/gzmo.toml`), `product-engine` (HOLD if down), `refresh-engine`, `product-hello` (HOLD without engine), `prefer-prime-tests`, `ct101-living-owner` (HOLD unless `PRODUCT_GATE_REQUIRE_CT101=1`), `release-freshness` (soft HOLD if stale tag).

### 6. Active bet `felt-use-mass-growth` vs ambient USP health

**Observed** (`research/opportunities/felt-use-mass-growth.md` done-when; gates above). Diagnosis only — bet remains active.

| Bet done-when | Gate row / artifact that **binds** it | What gate actually enforces |
|---------------|----------------------------------------|-----------------------------|
| 1. `honeypot.utility_score` + search orders by it | **No keep-quality/Brain Feed FAIL row** on utility column or Q-select | Mechanism lives in binary/schema; `felt-use-depth` **reports** `utility_positive`/`avg`/`max` only |
| 2. Weekly depth + utility census show **rising** dual-gate / utility mass from real sessions | `felt-use-depth.sh` snapshot + keep-quality/Brain Feed `felt-use` / `felt-use-depth` rows | **Proves** reachable census + soft depth floors; **does not** prove week-over-week rise or session origin (no rising gate in-tree) |
| 3. Brain Feed stays GREEN; no memory-gym | `brain-feed-check.sh` verdict + depth script operator rules | **Proves** FAIL-free nutrient surface set; gym ban is **doctrine/operator**, not a machine-checked anti-gym detector |

| Ambient USP / Unpark health (not bet done-when text) | Binding surface |
|------------------------------------------------------|-----------------|
| Living ops + faithfulness + appliance | keep-quality `living-readiness` |
| Nonzero felt floor (starvation) | keep-quality + Brain Feed `felt-use` |
| Spark monoculture | keep-quality `spark-refractory` |
| Ripen honesty (no empty-core lie) | keep-quality `ripen` + depth ripen report |
| Immune / lymph presence | keep-quality HOLD/PASS report rows |
| Living MCP label | keep-quality `mcp-attach` |
| Airgap core path | keep-quality `airgap-honesty` |
| Honest multi-night Unpark unlock | `keep-quality-soak.sh --summary` → `soak_ready_unpark_ok` |
| One overnight writer while feeding | Brain Feed `dual-writer` |
| Nutrient satellites (herdr remind, tinyfolder, serendipity) | Brain Feed P0 rows |
| Client laptop attach | `product-readiness-gate.sh` only |

### 7. Script ↔ binary coupling (deploy lag)

**Observed:**

| Coupling | Evidence |
|----------|----------|
| keep-quality ripen/health | SSH runs `$GZMO_BIN ripen status` and `$GZMO_BIN health` with default `/opt/gzmo/current/target/release/gzmo` (`keep-quality-gate.sh` L16, L183, L232) |
| felt-use-depth ripen + vault | Same default binary when remote; local vault short-circuits SSH but still invokes local/bin path for ripen (`felt-use-depth.sh` L26–30, L116–120) |
| Script-only sync | `ct101-brain-feed-sync.sh` rsyncs listed scripts/docs to `/opt/gzmo/current`, restores `+x`, **never** rebuilds binary or restarts `gzmo-daemon`; asserts daemon `ActiveEnterTimestamp` unchanged |
| Doctrine | `docs/CT101_DEPLOY.md` §“Sync docs/scripts only”; `docs/BRAIN_FEED.md` §“After merge → living host (script-only)” |
| Risk shape | Workstation/repo scripts can go GREEN while living **binary** at `/opt/gzmo/current/target/release/gzmo` lags (utility Q-select, Felt Use organs, ripen wording). Census SQL on vault can still PASS floors that the **running** search path does not implement until rebuild. Opportunity telescope line still names living mass until `#166`/harvest-organs on `/opt/gzmo/current` (`felt-use-mass-growth.md` L61) — binary deploy is a separate axis from script GREEN |

## Gaps and drift

1. **Utility mass unbound by FAIL:** Active bet done-when 1–2 center on `utility_score` / rising utility; gates only **report** utility fields. GREEN keep-quality/Brain Feed is possible with flat or zero utility growth if nonzero recall and soft depth HOLD/PASS rules hold. **Observed** scripts; **[INFERENCE]** bet “done” cannot be read from gate GREEN alone.
2. **No rising-series contract:** Done-when 2 says “weekly … show rising”; scripts emit point-in-time JSON only — no comparison to prior week inside gate math. **Observed**.
3. **Depth HOLD vs Brain Feed GREEN:** Thin depth is HOLD on both keep-quality and Brain Feed while `ok`/nonzero can keep Brain Feed GREEN. Bet done-when 3 (“Brain Feed stays GREEN”) is weaker than “depth GREEN.” **Observed**.
4. **Immune/lymph never FAIL keep-quality:** Missing night-lymph or immune plan cannot RED the USP bar. **Observed**.
5. **product-readiness still named “PRODUCT GREEN” in script advice** (`product_ready — laptop Memory MCP production gate GREEN`) while ADR-0007/docs demote it — naming drift vs contract. **Observed**.
6. **CT101-shaped fallbacks remain:** Brain Feed thin-depth path re-queries via `ssh` + `KEEP_QUALITY_VAULT_DB` default `/opt/gzmo/...` even when `felt-use-depth` already censused a local vault (`brain-feed-check.sh` L94–110). Deploy/docs still CT101-primary. **Observed**.
7. **Script GREEN ≠ binary current:** Explicit script-only sync path encodes deploy lag as normal ops. **Observed**.
8. **Soak honesty vs calendar nights:** Spacing rule is wall-clock hours between samples, not overnight metabolism completion; Doc-dated soak note admitted same-day samples unlocked Unpark check without multi-night biology proof.

## Evidence status

| Claim class | Status |
|-------------|--------|
| Script row logic, schemas, floors, artifacts | **Observed** in `scripts/*.sh` |
| Doctrine roles (USP bar, Brain Feed vs soak, product non-bar) | **Observed** in `docs/KEEP_QUALITY.md`, `BRAIN_FEED.md`, `ADR-0007`, `PRODUCT_PRODUCTION_READINESS.md`, `CT101_DEPLOY.md` |
| Active bet done-when text | **Observed** in `research/opportunities/felt-use-mass-growth.md` |
| Live GREEN/RED of CT101 or this workstation gates | **Unreachable** this sitting (non-goal: no live gate execution; `data-next/` empty of gate artifacts) |
| Whether `/opt/gzmo/current` binary embeds current Felt Use | **Unreachable** here without host probe |

## Sources

- `scripts/keep-quality-gate.sh` — pillars, schema `gzmo.keep.quality/v1`, binary defaults, verdict
- `scripts/keep-quality-soak.sh` — soak-log append, honest_nights / min_hours / advice
- `scripts/brain-feed-check.sh` — dual-writer, P0 rows, schema `gzmo.brain_feed.check/v1`
- `scripts/felt-use-depth.sh` — census fields, floors, utility_*, schema `gzmo.brain_feed.felt_use_depth/v1`
- `scripts/product-readiness-gate.sh` — client smoke rows, schema `gzmo.product.readiness/v1`
- `scripts/ct101-brain-feed-sync.sh` — script-only rsync list, no daemon restart
- `docs/KEEP_QUALITY.md` — USP bar + Unpark soak doctrine
- `docs/BRAIN_FEED.md` — P0 lock table, hard rules, depth HOLD semantics
- `docs/ADR-0007-one-product-living.md` — product-readiness not product GREEN
- `docs/PRODUCT_PRODUCTION_READINESS.md` — attach smoke meaning
- `docs/CT101_DEPLOY.md` — quality commands + script-only sync
- `docs/PRODUCTION_READINESS.md` — living vs client gate table
- `research/opportunities/felt-use-mass-growth.md` — active bet done-when 1–3
- `research/keep-quality-soak-2026-07-20.md` — Doc-dated soak caveat (same-day samples)
