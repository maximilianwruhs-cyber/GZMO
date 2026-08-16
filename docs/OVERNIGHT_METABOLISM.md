# Overnight Metabolism — named night labels

**Status:** Operator labels (2026-08-16) — names on existing `gzmo-daemon` ticks. Not a new crate. Not a rewrite of cron.  
**Literature vocabulary:** *Memory as Metabolism* (arXiv:2604.12034) · region rewrite cousin *Auto-Dreamer* (arXiv:2605.20616)  
**Invariants:** [ADR-0003](./ADR-0003-one-instance-metabolism.md) · [ADR-0004](./ADR-0004-airgap-living-usp.md) · [ADR-0007](./ADR-0007-one-product-living.md)  
**Clock SoT:** product template [`gzmo.toml.example`](../gzmo.toml.example) and living overview [INFRASTRUCTURE_OVERVIEW.md](./INFRASTRUCTURE_OVERVIEW.md) §6.2. Compiled defaults in `config.rs` apply only when toml omits the key (spark hours differ).  
**Gate:** `bash scripts/brain-feed-check.sh` is an **operator** check, not a nightly daemon job.

---

## Thesis

A living local Keep improves through **autonomous overnight metabolism between sessions**, not by a vector plugin in every prompt and not by overnight LoRA.

*Memory as Metabolism* names three **jobs**. GZMO already runs those jobs as discrete cron ticks on one writer. The names are labels. They do **not** mean three contiguous UTC blocks, and wall-clock order is **not** TRIAGE → CONSOLIDATE → AUDIT.

```text
Wall clock on the product template (UTC)     Label on that tick
  01:00  DreamEngine REM                     CONSOLIDATE
  01:45  Qdrant vault → honeypot collection  AUDIT (index)
  02:15  SessionDistill                      TRIAGE
  02:30  promote (MetabolismConfig default)  TRIAGE
  02:45  embed (MetabolismConfig default)    TRIAGE (index write)
  03:30  SparkEngine (also 22:30)            CONSOLIDATE
  continuous  ingest watcher                 TRIAGE
```

Do not reorder these ticks to match the paper. Dream at 01:00 already feeds distill at 02:15; spark at 03:30 runs after both.

---

## TRIAGE — eat, verify, refuse

**Jobs:** session distill, inbox ingest, promote, `failure_cases`.

| Tick | Where | What |
|------|--------|------|
| 02:15 UTC | `[session_distill]` `cron_hour/minute` — daemon `SessionDistillEngine` | Chat sessions → atomic claims; Prime verify; promote when the gate passes |
| Continuous | `[ingest]` watcher | Drops under the configured knowledge path → same extract/verify/promote path |
| 02:30 UTC | `[metabolism]` promote default (`config.rs`) | Typed promote pass (same receipts as `gzmo serve`) |
| On refuse | `failure_cases` | `verify_fail` / `gate_refuse` / `promote_rollback` — searchable, no felt-use, not promoted honeypot |

tinyFolder overnight (`tinyfolder-overnight.sh` / timer) is a **Brain Feed satellite**, not a `gzmo-daemon` cron slot. Lab recipes (`synapse-distill-handoff.sh`, `ingest-smoke.sh`) are lab backends, not living law.

---

## CONSOLIDATE — structure, supersede, link

**Jobs:** dream REM, region rewrite on promote, spark.

| Tick / event | Where | What |
|--------------|--------|------|
| 01:00 UTC | `[dreams]` `cron_hour = 1` | `DreamEngine` reads yesterday's episodic log, REM-chunks, extracts after verify |
| On promote | `maybe_region_rewrite` | Incoming truths can supersede a same-entity cluster (`is_latest = 0`). Event-driven, not a clock window |
| 03:30 and 22:30 UTC | `[spark]` `cron_hours = [3, 22]` in the product template | `SparkEngine` associative hypotheses, then verify. Stale-sweetness × refractory damping |

If toml is missing `[spark].cron_hours`, compiled default is `[9, 14, 21]` at minute 17 — that is **not** the living overview. Pin the template on CT101.

---

## AUDIT — index + operator census

**Daemon (living):** Qdrant sync at **01:45 UTC** (`[qdrant] sync_cron_*`) writes promoted vectors to the `honeypot` collection on **CT101** (`:6333`) and **prunes** points that are no longer `is_latest = 1`. Embeddings still come from **VM200** (`:8081`). This is index maintenance, not a 03:30–05:15 block.

**Not daemon on living CT101:**

| Script / loop | Who runs it | Why it is not night law |
|---------------|-------------|-------------------------|
| `gzmo-handoff.sh` 04:00 | Lab assembly backend only (`handoff_backend.is_lab()`) | Calibration suggestion loop; living skips it |
| `felt-use-depth.sh` | Operator / telescope | Census of living mass; soak evidence, not a cron tick |
| `brain-feed-check.sh` | Operator / telescope | Nutrient-path health; dual-writer check |
| Wiki `sync_cron_*` | Wiki engine | Not Brain Feed; do not call Observatory emit AUDIT |

Morning attach still means: one writer, vault/Qdrant in sync, operator census when you need soak proof.

---

## Single living writer

Per [ADR-0003](./ADR-0003-one-instance-metabolism.md) and [ADR-0006](./ADR-0006-owner-control-plane.md):

- All daemon ticks above run only on the host holding the living mutex (**CT101** LXC `.202` today).
- Telescope `gzmo_full` does not start `gzmo serve` while that claim holds.
- Do not add a second overnight writer, an Observatory control plane, or overnight LoRA to “complete” the triad.
