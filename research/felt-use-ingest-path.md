# Felt Use: markdown → latest honeypot row (this Keep)

**Ticket:** [#222](https://github.com/maximilianwruhs-cyber/GZMO/issues/222) (map [#221](https://github.com/maximilianwruhs-cyber/GZMO/issues/221))  
**Tip:** `origin/main` @ `8fc07a2` (not `feat/living-research-intel`)  
**Prove-first vault:** `/home/gzmo/.gzmo-living/data/vault.db`  
**Status:** Path inventory only. No ingest, no vault writes, no gym searches.

---

## Answer gist

On this Keep a markdown file becomes a `honeypot.is_latest=1` row only when the **`gzmo` binary** runs **live `ingest`** against **`GZMO_CONFIG=~/.gzmo-living/gzmo.toml`**. That call is `IngestEngine::ingest_file` → extract/verify → `promote_truths` (origin `ingest`) → `insert_honeypot_lifecycle` if the truth passes the honeypot gate. The living daemon is the sole overnight writer, but **this Keep’s toml has no `[orchestration].watchers` and no `[ingest]` block**, so there is **no auto-ingest of `research/opportunities/*.md`**. `memory_record`, `gzmo ingest --dry-run` / `ingest-eval`, and harvest-organs are not this path. Memory-gym search is forbidden.

---

## 1. This Keep’s living wiring

| Claim | Evidence |
|-------|----------|
| Living vault path is `~/.gzmo-living/data/vault.db` | [`~/.gzmo-living/gzmo.toml`](/home/gzmo/.gzmo-living/gzmo.toml) L47: `vault_db = "/home/gzmo/.gzmo-living/data/vault.db"` |
| Operator binary is `gzmo` at `~/.local/bin/gzmo` → repo `target/release/gzmo` | `install-living-airgap.sh` L43–54, L120–121; this host: symlink `~/.local/bin/gzmo` → `/home/gzmo/Projects/GZMO/target/release/gzmo` |
| Daemon is the sole vault writer | `~/.config/systemd/user/gzmo-daemon.service` L8–10: `GZMO_CONFIG=/home/gzmo/.gzmo-living/gzmo.toml`, `ExecStart=/home/gzmo/.local/bin/gzmo daemon`; unit was `active` when probed |
| Living toml has **no** `[ingest]` and **no** `[orchestration]` | File ends after Neo4j MCP (L93–102). Ingest therefore uses crate defaults; watchers stay empty |
| Ingest defaults: enabled, verify on, min_confidence 0.85, evidence+strict KG, **batch off** | `config.rs` L2946–2953 (`default_ingest_enabled` / `default_dream_verify` / `default_dream_min_confidence` / `default_kg_require_evidence` / `default_kg_strict`); `IngestConfig::default` L1087–1088 `batch_enabled: false` |
| Empty watcher map starts nothing | `watcher.rs` L22–24: `if watcher_configs.is_empty() { return Ok(()); }` |
| Dual-writer refuse on install | `install-living-airgap.sh` L36–41: die if `gzmo-serve` is active |

`harvest-organs` / `#166` on `/opt/gzmo/current` is named in the opportunity note as **CT101-only** and is **not** a binary in this tree (`felt-use-mass-growth.md` L61). It is not a shipped path on this Keep.

---

## 2. Shipped write path (markdown → latest honeypot)

Operator command that actually targets this vault:

```bash
GZMO_CONFIG=/home/gzmo/.gzmo-living/gzmo.toml \
  /home/gzmo/.local/bin/gzmo ingest research/opportunities/felt-use-mass-growth.md
```

### 2.1 Binary entry

| Step | Evidence |
|------|----------|
| CLI: `gzmo ingest <path>` (`--dry-run` optional) | `gzmo-cli/src/main.rs` L225–238, L474–481 |
| Live run calls `IngestEngine::ingest_file` | `gzmo-cli/src/ingest_cmd.rs` L100–104 |
| Vault opened from config (`embeddings::open_vault_with_embeddings`) | `ingest_cmd.rs` L58–67 |
| Post-live Qdrant sync is non-fatal | `ingest_cmd.rs` L114–120 |
| Bulk sibling: `gzmo ingest-dir` (`.md` only, same engine) | `ingest_dir_cmd.rs` L38–43 |

`gzmo ingest-dir` is the same write path at directory scale. Not required for one file.

### 2.2 `IngestEngine` gates (before any vault write)

| Gate | Fail closed? | Evidence |
|------|----------------|----------|
| `[ingest].enabled` | Yes — bail | `ingest.rs` L108–110 |
| Path under wiki dir (default `wiki/`) | Skip, no write | `ingest.rs` L91–104, L111–114 |
| `gzmo_synthetic: true` frontmatter | Bail | `ingest.rs` L147–150, L710–722 |
| Identical body already ingested (`ingest_dedup`) | Skip | `ingest.rs` L116–124 |
| No verified entities **and** no verified relations | Fail report, no promote | `ingest.rs` L239–244 |

`felt-use-mass-growth.md` is under `research/opportunities/`, not `wiki/`. Its YAML is opportunity metadata (`id`, `title`, `status`…), not `gzmo_synthetic`. `ingest_prep::Frontmatter` only deserializes `migration_id` / `source` / `notebook` / `original_path` / `wave` (`ingest_prep.rs` L6–12); extra keys are ignored. Filename heuristics land it on **`DocClass::Narrative`** (`ingest_prep.rs` L55–106).

Optional **pre-flight** (not required by the engine): `scripts/pre-ingest-gate.sh` — size / extension / binary checks; `--dry-run` only runs `gzmo ingest --dry-run` (L8–9, L65).

### 2.3 Extract → verify

| Step | Evidence |
|------|----------|
| Chunk + `KgPromoter::run_merged_pipeline` (`ingest_extraction`) | `ingest.rs` L164–188 |
| Verify on unless `[ingest].verify` is false (default **true**) | `kg_extract.rs` L519–524, L573–593; `config.rs` L1034–1035, L2931–2932 |
| Verdict must be supported, `confidence ≥ min_confidence` (default **0.85**), and if `require_evidence` a quote ≥ **12** chars | `kg_extract.rs` L484–501; `kg_promotion.rs` L8–9 `MIN_EVIDENCE_CHARS` |
| `strict_kg` default **true** | `config.rs` L1051–1052, L2952–2953 |

### 2.4 Promote: KG then vault then honeypot

| Step | Evidence |
|------|----------|
| Neo4j KG write via `promote_to_kg` — **fatal** on error | `ingest.rs` L250–268 |
| Truths built from **entity observations** and **relations**; `source_file` is the **basename** only | `ingest.rs` L363–438, L142–146 |
| `vault.promote_truths` origin **`ingest`** — vault error is **non-fatal** | `ingest.rs` L277–279; `vault.rs` L1983–1986 |
| Confidence **&lt; 0.85** → `quarantine_vault`, no honeypot | `vault.rs` L2041–2067 |
| Else insert/corroborate `semantic_vault`, then honeypot if qualified | `vault.rs` L1775–1938 |
| New / extend / contradict: `insert_honeypot_lifecycle` with `is_latest = 1` | `honeypot.rs` L50–68 |
| Duplicate entity: `upsert_honeypot_row` | `vault.rs` L1761–1769 |

`source_file` on the row is `felt-use-mass-growth.md`, not the `research/opportunities/…` path. Honeypot path-blockers that match substrings of a **full** research-library path (`notebooklm`, `/takeout/`, …) do **not** fire on this basename (`honeypot.rs` L16–34).

### 2.5 Honeypot qualify gate (the last door)

`qualifies_for_honeypot` (`honeypot.rs` L12–39):

1. `confidence ≥ 0.85` (`HONEYPOT_MIN_CONFIDENCE`, L10)
2. `source_file` present and non-empty
3. `source_file` must **not** contain: `chat_history`, `chat_session`, `quelltext`, `sources`, `notebooklm`, `drive_clean`, `/takeout/`, `takeout_curated`
4. Content must **not** start with `[relation:` (ingest relation truths are `[RELATION:…]` — **never honey**)
5. Not boilerplate (`sources do not contain`, `migration_id`, takeout/corpus slogans)

Plus `!is_unverified_derived` (`lifecycle.rs` L186–204). Origin `ingest` is **not** in the derived set (`dream` / `spark` / `session_distill`), so ingest truths skip that extra bar.

A latest row is `INSERT … is_latest, recall_count` = `1, 0` (`honeypot.rs` L63–68).

---

## 3. Sibling paths that are **not** “markdown file → honey”

| Path | What it writes | Why it is not this ticket |
|------|----------------|---------------------------|
| `gzmo ingest --dry-run` / `gzmo ingest-eval` | Nothing (dry `finish_ingest`) | `ingest.rs` L130–138, L247–248 |
| Daemon file watcher | Same `ingest_file` | **Not configured** on this Keep (empty `watchers`) |
| Nightly `ingest-dir` via `[ingest].batch_enabled` | Same engine | Default **false**; living toml does not enable it (`config.rs` L1020–1022, L1088) |
| `memory_record` MCP / `store_text` | `semantic_vault` or quarantine only — **no honeypot**, **no `source_file`** | `tools/memory.rs` L23–65; `vault.rs` L2408–2457 |
| `gzmo memory promote` | Vault rows **already** ≥0.85 and missing from honeypot | `promote_cmd.rs` L11–20; `vault.rs` L2471–2530. Catch-up, not markdown ingest. `living-promote-embed-oneshot.sh` is **CT101 SSH** (`HOST=ct101`, `/opt/gzmo`) |
| `gzmo distill` / session close | `promote_truths_with_origin(..., "session_distill")` | `session_distill.rs` L239–241. Transcript, not a repo `.md` |
| `scripts/living-research-intel.sh` | Drafts under `~/.gzmo-living/data/research-intel/` | Header: never touches `vault.db` |

`memory_record` facts can later enter honeypot only if someone runs **`gzmo memory promote`**: missing `source_file` is replaced with `"semantic_vault"` (`vault.rs` L2505–2507), which then passes the empty-source check. That is a **second** hop, not ingest of `felt-use-mass-growth.md`.

---

## 4. What is gym / forbidden

From the opportunity note and living scripts (not from a gym run):

| Forbidden | Evidence |
|-----------|----------|
| Memory-gym chats / searches whose job is to mint recall/utility | `felt-use-mass-growth.md` L29, L35 (“no memory-gym”); `felt-use-depth.sh` L3 (“Does NOT run memory-gym searches”); L184, L202 |
| Using `organism-memory-bench-spike.sh` as the ingest/growth path | Opportunity L47 lists it as operator **bench**; the script SSH-`sqlite3` LIKE-searches CT101 honeypot (`organism-memory-bench-spike.sh` L40–51). Borrow-eval, not doctrine ingest |
| `harvest-organs` / `#166` on `/opt/gzmo/current` as this Keep’s mass path | Opportunity L61: living mass remains **CT101-only** until those binaries exist there |
| Re-ingesting wiki / `gzmo_synthetic` pages | `ingest.rs` L91–114, L147–150 |
| Headless watcher fallback (ungated prompt) | `watcher.rs` L241–248 — only if `ingest_engine` is `None`; not the live daemon path when `[ingest].enabled` |

Allowed measurement on this Keep (not ingest): `bash scripts/felt-use-depth.sh` reads `~/.gzmo-living/data/vault.db` when present (`felt-use-depth.sh` L20–25). Census only.

---

## 5. Worked example: `research/opportunities/felt-use-mass-growth.md`

If an operator ran live `gzmo ingest` on that file with this living config:

1. Class **Narrative**; extract/verify against the body (schema/Q-select prose, MemRL cite, done-whens).
2. Each **verified observation** becomes `[TYPE:Name] …` with `source_file=felt-use-mass-growth.md`, `origin=ingest`.
3. Those with confidence ≥ 0.85 and a basename that is not a blocked library token become **`honeypot` `is_latest=1`** in the same `promote_truths` transaction. Relations do not.
4. **Nothing in the living appliance auto-does this.** The note sitting on disk is not a honeypot row. Map #221’s charting fact (zero Felt Use strings in honeypot) is consistent with “file exists, ingest never run.”

Utility / Felt Use **Q** (`apply_utility_boost`, `felt-use-depth.sh` utility census) is **after** a row exists. The opportunity’s done-when 1 (schema + MCP order) is mechanism; done-when 2–3 (mass from real sessions, Brain Feed GREEN, no gym) are not created by dropping a markdown file.

---

## 6. Binaries and scripts named

| Name | Role on this Keep |
|------|-------------------|
| `/home/gzmo/.local/bin/gzmo` (`gzmo-cli`) | **The** write binary: `ingest`, `ingest-dir`, `memory promote`, `distill`, `daemon` |
| `gzmo daemon` (systemd `gzmo-daemon.service`) | Overnight owner; IngestEngine is built (`daemon_cmd.rs` L301–312, L392–396) but **no watchers fire** |
| `scripts/pre-ingest-gate.sh` | Optional static gate |
| `scripts/install-living-airgap.sh` | How this living home + MCP fragment were meant to be stood up |
| `scripts/felt-use-depth.sh` | Measure only |
| `scripts/living-promote-embed-oneshot.sh` | CT101 promote+embed, not this laptop’s markdown ingest |
| `scripts/organism-memory-bench-spike.sh` | Gym-adjacent bench — not ingest |

---

## Not claimed

- Whether a past operator already ingested this file (would need a vault **read**; this note does not query `vault.db`).
- Which ingest path **doctrine** should use for Felt Use (that is #226, blocked on this ticket).
- Product changes. None.
