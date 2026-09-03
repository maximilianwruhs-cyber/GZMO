# GZMO — Baseline (2026-09-03)

## Status: Offline workspace baseline GREEN

The full 7-crate workspace compiles, formats, lints, and tests clean on one box
with **no live infrastructure**. Living/production readiness is a *separate*
gate that requires the CT101 home-lab (see "Operational gates" below).

Verified at commit `98755cd` (the tree this doc is committed on top of):

| Gate | Command | Result |
|------|---------|--------|
| Format | `cargo fmt --all --check` | clean (exit 0) |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | 0 warnings (exit 0) |
| Tests | `cargo test --workspace` | **730 passed, 0 failed, 5 ignored** |

Workspace lints (`Cargo.toml`) keep `-D warnings` meaningful: `clippy::correctness`
and `clippy::suspicious` are **deny**; noisy style/pedantic lints are allowed. The
5 ignored tests are gzmo-core live probes (network + Prime `:8000` + OpenRouter key).

---

## Workspace (7 crates)

| Crate | Role | Passing tests |
|-------|------|---------------|
| `gzmo-core` | All business logic — config, LLM gateway, honeypot memory pipeline, ingest, dream, spark, skills, tools, MCP bridge | 370 (+5 ignored live) |
| `gzmo-evolver` | Connected-host repository evolver coordinator (self-development loop) | 170 — lib 115 · repo_loop 54 · bin 1 |
| `evolution-contracts` | Pure domain contracts for evolution candidates, envelopes, evaluation, audit (JSON schemas) | 96 — lib 17 · contracts 68 · schema_snapshots 10 · export_schemas 1 |
| `gzmo-cli` | Thin `gzmo` binary — `main.rs` + `*_cmd.rs` + TUI | 51 |
| `gzmo-chaos` | Lorenz attractor engine | 26 — lib 25 · doctest 1 |
| `eml-core` | Exp-Minus-Log symbolic computation engine — ComplexBall arithmetic, RPN emit, zero-copy execution | 12 |
| `gzmo-scheduler` | Thin cron runner for GZMO-next — ticks every 60s, spawns Little Tools Lab recipe scripts (no engines/vault/LLM/MCP) | 5 |

Total: **730 passing**, 5 ignored (live), 0 failing.

---

## Line-ending invariant

`.gitattributes` pins `* text=auto eol=lf`. The repo targets Linux/WSL; every
tracked shell script runs under bash. A Windows checkout with `core.autocrlf=true`
was rewriting all 258 `.sh` scripts to CRLF, which breaks bash heredocs (syntax
error, exit 2) and previously red-flagged the evolver gate. All committed blobs
are already LF, so the attribute is content-neutral: it only pins the invariant
and stops future Windows checkouts from regressing it.

Verify: `git ls-files --eol` reports `i/lf` for every text file.

---

## Operational gates (require live infra — NOT part of the offline baseline)

Living/production readiness is gated separately and needs the CT101 home-lab
(SSH `ct101`, Prime LLM `:8000`, embeddings `:8081`, Qdrant `:6333`,
`/opt/gzmo` deployment). Run these on the appliance, not in a sandbox:

- `bash scripts/keep-quality-gate.sh` — continuous living quality bar (readiness + felt-use + spark + immune + ripen + lymph + attach + airgap honesty).
- `bash scripts/verify-baseline-green.sh` — M4 eval + production E2E + platform hot-memory + Redis scratch.
- `./scripts/verify-production.sh` — production E2E against live engines.

---

## Reproduce the offline baseline

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run the live probes explicitly when infra is up:

```bash
cargo test -p gzmo-core --test live_cloud_probe -- --ignored --nocapture
```

---

## Environment

- Dual RTX 5070 Ti, Ryzen 9 9950X, 59 GB RAM, Proxmox home lab.
- Works with multiple agent frameworks (herdr, tau, pi, openclaw).

---

## Baseline date

2026-09-03 — offline workspace baseline recorded on top of `98755cd` (`origin/main`).

*730/730 offline tests passing · fmt clean · clippy `-D warnings` clean.*
