# CT101 Qdrant / Embed Ops

**Status:** Living ops scar book (2026-07-19)  
**Related:** [CT101_DEPLOY.md](./CT101_DEPLOY.md), [PORTS.md](../PORTS.md), [ct101-systems/50-memory-data-plane/qdrant-sync-recall.md](../ct101-systems/50-memory-data-plane/qdrant-sync-recall.md)

## Collections

| Collection | Role |
|------------|------|
| **`honeypot`** | Production RAG — sync from SQLite `honeypot WHERE is_latest=1` |
| **`knowledge`** | Legacy vault-era mirror — **read-only**; do not delete without M2 checklist |

Recall looking “empty” while vault is full almost always means: missing embeddings, honeypot gap, or Qdrant orphan/drift — not “vault is dead.”

## Embed backfill loop

Run **on CT101**:

```bash
ssh ct101 'bash /opt/gzmo/current/scripts/ct101-embed-backfill-loop.sh'
# env: EMBED_BATCH=4000 (default), GZMO_CONFIG=/opt/gzmo/gzmo.toml
```

Loop:

1. Count `semantic_vault` rows with null/short embeddings  
2. `gzmo memory embed $BATCH`  
3. Mirror vault embeddings → `honeypot` (same id, `is_latest=1`)  
4. When vault gap is 0 → final mirror + `sync-vault-to-qdrant.py` → collection `honeypot`  
5. `gzmo health` honeypot_qdrant line

Log: `/opt/gzmo/data/embed-backfill.log`

**Stop condition:** vault missing embeddings = 0. Honeypot may still have orphan ids without vault rows — that is a separate prune.

## Orphan prune

After large syncs, Qdrant can hold points not in `honeypot is_latest=1`:

```bash
# Always dry-run first
ssh ct101 'python3 /opt/gzmo/current/scripts/ct101-qdrant-prune-orphans.py --dry-run'
ssh ct101 'python3 /opt/gzmo/current/scripts/ct101-qdrant-prune-orphans.py'
```

Deletes only from collection `honeypot` (default). Never point this at `knowledge` unless you intentionally mean to.

## Failure modes

| Symptom | Likely cause | Action |
|---------|--------------|--------|
| Vault huge, Qdrant tiny | Embed/sync not run | Backfill loop |
| `honeypot_qdrant` red in health | Sidecar down or collection empty | Docker + sync |
| Recall stale after deletes | Orphan points or wrong collection | Dry-run prune; confirm `collection=honeypot` |
| Double collections | Agents query `knowledge` | Config must be `honeypot` |
| Workstation sync against CT101 | Wrong host | Living sync is localhost on CT101 |

## Do not

- Delete collection `knowledge` without the M2 cutover checklist  
- Run prune without `--dry-run` first on a strange count  
- Treat workstation `data-next/` Qdrant as living  
- Expect Redis to hold vectors — Redis is scratch/queue only ([PORTS.md](../PORTS.md))
