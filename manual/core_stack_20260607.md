# Core stack seed — provenance

**source_file tag:** `manual/core_stack_20260607.md`
**Seeded:** 2026-06-07 (clean-slate rebuild)
**Origin:** `manual` · **decay_class:** `Structural` · **confidence:** `0.95` · **container:** `obolus`

## What this is

This file is the provenance marker for GZMO's curated **core self-knowledge**: hand-authored, high-trust facts about the machine itself (nodes, services, models, config, memory model, pipeline, engines, paths, how-tos, doctrine, current state).

These facts were injected directly into `vault.db` (semantic_vault + honeypot) via
[`scripts/seed-core-stack.py`](../scripts/seed-core-stack.py), which parses the
`Injected facts:` bullets from the canonical source document:

- **Source of truth / template:** [`docs/CORE_STACK_KNOWLEDGE.md`](../docs/CORE_STACK_KNOWLEDGE.md)

No LLM extraction was used; these are operator-curated facts, not migration-pile data.

## Why direct injection

Per the curation-first / no-automated-migration doctrine, the clean system is
populated first with authoritative, condensed self-knowledge before any other
data. Direct injection (the `seed-cognition-stack.py` pattern) bypasses the
extract/verify pipeline because a human already curated and consolidated the facts.

## How to (re)seed

```bash
cd ~/Projects/_foundation-audit/survey_GZMO

# Preview the parsed facts
scripts/seed-core-stack.py --list

# Dry-run against the vault (no writes)
scripts/seed-core-stack.py --dry-run

# Inject (idempotent: existing facts are skipped by content_norm)
scripts/seed-core-stack.py

# Make recallable
./target/release/gzmo memory embed
scripts/sync-vault-to-qdrant.py --source honeypot
```

## Updating the core

Edit `docs/CORE_STACK_KNOWLEDGE.md` (add or revise cards in the locked
`What / How / Use / Why / Related / Injected facts` format), then re-run the
seeder. It is idempotent on `content_norm`, so only new/changed fact text is added.
