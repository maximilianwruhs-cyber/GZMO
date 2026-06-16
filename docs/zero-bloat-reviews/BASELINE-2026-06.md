# Zero-Bloat Baseline — 2026-06

Reference point for `sovereignty-verify.sh` check 7.

## Workspace

- Members: `gzmo-core`, `gzmo-cli`, `gzmo-chaos`
- `[workspace.dependencies]` count: **20**

## Release binary

- Path: `target/release/gzmo`
- Size: **~29.2 MB** (30646776 bytes on 2026-06-16)
- Warn threshold: `compliance.max_binary_mb` (default 80)

## Workspace dependencies (pinned)

anyhow, async-trait, chrono, futures-util, notify, pulldown-cmark, redis, reqwest, reqwest-eventsource, rusqlite, serde, serde_json, serde_yaml, sha2, thiserror, tokio, toml, tracing, tracing-subscriber, uuid

## Obolus snapshot

Run for current top processes:

```bash
gzmo obolus report --since 7d
```

New workspace dependencies after this baseline require a file in this directory.
