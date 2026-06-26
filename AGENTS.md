# GZMO — Agent Guide

Sovereign Rust agent: honeypot memory pipeline + local LLM. Read `MACHINE.md` first.

## Repo layout

| Path | Role |
|------|------|
| `gzmo-core/` | All business logic — config, gateway, memory, ingest, dream, spark, MCP |
| `gzmo-cli/` | Thin binary: `main.rs` + `*_cmd.rs` + optional TUI |
| `gzmo-chaos/` | Lorenz attractor engine (separate crate) |
| `gzmo.toml` | **Local** operator config (gitignored — copy from `gzmo.toml.example`) |
| `.env` | **Local** secrets (gitignored — copy from `.env.template`) |
| `memory/` | Episodic logs (runtime, gitignored) |
| `data/vault.db` | SQLite vault (runtime, gitignored) |
| `wiki/` | Git-tracked markdown wiki layer (Obsidian-browsable) — see `WIKI.md` |
| `WIKI.md` | Wiki schema + conventions (how GZMO maintains `wiki/`) |
| `scripts/` | Production ops — prefer these over ad-hoc commands |
| `docs/` | Canonical docs — see `docs/README.md` |

## Conventions

- **Minimize scope** — focused diffs, match existing Rust style in `gzmo-core`.
- **Ponytail ladder** — before new code: YAGNI → reuse → stdlib → native → installed dep → one line → minimum that works. Mark intentional shortcuts with `// ponytail: <ceiling> — <upgrade trigger>`. `/ponytail-review` after fixer workstreams, `/ponytail-debt` monthly. See `.cursor/rules/ponytail.mdc` for the full ladder.
- **ponytail: debt convention** — when taking a shortcut below a rung, tag it: `// ponytail: global Mutex — upgrade when concurrent writers appear`. These are payable debt, not hidden complexity.
- **Secrets** — never in committed files; `.env` + `apply_mcp_env_overrides` in `config.rs`.
- **Two skill systems** — Rust skills in `gzmo-core/src/skills/`, shell skills in `skills/` + `scripts/skill_*.sh`.
- **Engines** — Prime at `:8000`, embeddings VM200 `:8081`, Qdrant LXC101 `:6333`.
- **Pipeline** — extract → verify → promote → vault → honeypot (see `MACHINE.md`).

## Verify changes

```bash
cargo test
cargo clippy --all-targets
./scripts/sovereignty-verify.sh   # ARCH-DIR + Obolus (local, no infra)
./scripts/verify-production.sh    # needs live infra
```

## Commands (discovery pipeline)

Five-phase goal pipeline (Jules automate-github-issues analogue):

```bash
./scripts/run-discovery-goal-pipeline.sh --report <path> [--session-id <id>] [--spawn]
```

Individual phases:

```bash
gzmo kurator plan-from-discovery --report <path> [--spawn]
gzmo kurator approve-plan --plan <plan_dir>          # required before execute (default)
gzmo kurator execute-workstream --plan <dir> --workstream <id> [--spawn]
# Post-fixer: ponytail review pass — catches over-build from autonomous agent work
/ponytail-review   # (in Pi) — delete-list on diff: stdlib:, yagni:, delete:
/ponytail-audit    # periodic — whole-repo over-engineering scan
./scripts/query-discovery-activities.sh summary|failed|open|snapshots
```

Environment:

| Variable | Default | Effect |
|----------|---------|--------|
| `DISCOVERY_PLAN_REQUIRE_APPROVAL` | `1` | Block execute until `plan.json` has `approved_at` |
| `DISCOVERY_INCLUDE_GIT_CONTEXT` | off | Append `git show` + `git log` to plan/fixer briefs |

## Testing

```bash
cargo test -p gzmo-core discovery_plan_agent remediation_snapshot discovery_git_context
./scripts/mechanics-verify.sh
~/gzmo_skills/scripts/discovery-probes/probe-jules-patterns.sh
```

## Boundaries

- **No Jules API** — no `JULES_API_KEY`, `jules.googleapis.com`, or `@google/jules-sdk` in this repo.
- **Skills root** — discovery remediations and probes live under `$GZMO_SKILLS_ROOT` (default `~/gzmo_skills`), never `survey_GZMO/gzmo_skills/`.
- **Parallel spawns** — `plan.json` workstreams must not overlap on `target_paths` (ownership gate in verify).
- **Plan approval** — operator must run `gzmo kurator approve-plan` before `execute-workstream` unless gate disabled.

## Local CI Verification

Optional local GitHub Actions via [act](https://github.com/nektos/act) (pattern from Jules `local-action-verification`):

```bash
./scripts/act/install-act.sh          # once — requires Docker
./scripts/act/run-act.sh "push -j ci" # background + log poll; ACT_TIMEOUT, ACT_POLL
```

Complements `cargo test` and `./scripts/mechanics-verify.sh`. No cloud CI required for sovereignty checks.

### Parallel remediation reconciliation

When multiple discovery spawns touch the same file:

```bash
./scripts/reconcile-discovery-changes.sh scan --json
./scripts/reconcile-discovery-changes.sh status
./scripts/reconcile-discovery-changes.sh merge-file --path <rel> --ours <a> --theirs <b> [--dry-run]
```

Spawn state polling (tracker flush races): `SPAWN_LOAD_POLLING=1` (default), `SPAWN_POLL_INTERVAL_MS`, `SPAWN_POLL_TIMEOUT_MS`.

## Sovereignty + Obolus

- Constitution: `docs/ARCH-DIR-001-GZMO.md`, pointer `ARCH-DIR-001.md`
- Energy governance: `docs/OBOLUS_GOVERNANCE.md`
- Verify: `./scripts/sovereignty-verify.sh`
- New workspace deps require `docs/zero-bloat-reviews/` entry

## Do not touch without reason

- `data/lore.toml` — static chaos lore seed (tracked)
- `SOUL.md` — agent persona (tracked)
- `docs/archive/` — local session notes (gitignored)
