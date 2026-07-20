---
id: ct101-brain-feed-sync
title: One-command Brain Feed script sync to CT101
status: candidate
score: 19
uniqueness: 2
brain_profit: 3
credit_cost: 5
attention_cost: 5
usp_fit: 4
stack_ids: []
created: 2026-07-20
updated: 2026-07-20
---

# CT101 Brain Feed sync

## Why rare

Not rare as rsync — rare as **operator pain that blocks nutrient scripts from reaching the living host** after merge (rsync drops `+x`, stale `/opt/gzmo/current/scripts`).

## Brain profit

New Brain Feed / keep-quality scripts on `main` actually run on the living box without a rebuild.

## Done when

`scripts/ct101-brain-feed-sync.sh` (or sibling) rsyncs the CT101_DEPLOY script/doc set, restores `+x`, dual-writer-safe (no daemon restart); documented one-liner; discovery/brain-feed still GREEN.
