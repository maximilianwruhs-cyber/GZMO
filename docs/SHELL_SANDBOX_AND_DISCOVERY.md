# Shell sandbox & discovery on GZMO-next

**Status:** Partial (2026-07-15)  
**Related:** [00-CAPABILITIES_OVERVIEW Theme 5](./ct101-systems/00-CAPABILITIES_OVERVIEW.md), [ADR-0002 lab-only surfaces](../../little-tools-lab/docs/adr/0002-pedagogy-chaos-scheduler-lab-only.md)

## Shell tool

| Mode | How | Effect |
|------|-----|--------|
| Default allowlist | `gzmo-core/src/tools/shell.rs` | First-token allowlist; `.sh` and `bash script.sh` ok |
| **Strict (GZMO-next)** | automatic when `GZMO_INSTANCE=next` (or `GZMO_SHELL_STRICT=1`) | Blocks `systemctl`, `sudo`, `kill`, … |
| **Docker isolate** | `GZMO_SHELL_DOCKER=1` | `docker run --rm --network none -v $cwd:/work:ro alpine sh -c …` |

gVisor / dedicated sandbox image, discovery lab recipe, and CT101 vault migrate tooling are tracked in [`STRETCH_ITEMS_HANDOFF.md`](./handoff/STRETCH_ITEMS_HANDOFF.md).

## Discovery → honeypot

CT101 discovery publish eval was hardened (stretch **S1**, 2026-07-15). Next does **not**
copy CT101 cycles wholesale.

**Lab recipe (stretch S2):** [`little-tools-lab/scripts/discovery-smoke.sh`](../../little-tools-lab/scripts/discovery-smoke.sh)

```
findings.md / findings.jsonl
  → placeholder / unpublished-garbage gate
  → honeypot-gate
  → optional vault-promote-distill (--live --promote)
  → incremental Qdrant (--ids / --since already shipped)
```

```bash
export GZMO_CLONE_ROOT=/home/gzmo/github-clone
export CARGO_TARGET_DIR=$GZMO_CLONE_ROOT/temp-bench/target
bash little-tools-lab/scripts/discovery-smoke.sh --fixture --meta /tmp/discovery-smoke-meta.json
# live gate + promote into data-next (operator opt-in):
# bash little-tools-lab/scripts/discovery-smoke.sh --live --promote --meta /tmp/discovery-smoke-meta.json
```

Until a weekly scheduler slot is armed (S2b), next also grows memory via chat distill + overnight synapse/dream.
## Incremental Qdrant

- Nightly: `gzmo-scheduler` → `qdrant-vault-sync.sh` (full honeypot).
- On promote (next + vault has Qdrant recall attached): `sync_vault_to_qdrant_filtered(--ids,…,--since,…)` after honeypot-eligible batch.
