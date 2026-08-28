# Skills Bridge — Rust, Shell, gzmo_skills

**Status:** Operator guide (2026-07-19)  
**Sibling detail:** `/home/gzmo/github-clone/gzmo_skills/BRIDGE.md` (may still cite `survey_GZMO` paths)  
**Quality bar:** [SKILL_GOLDEN_STANDARD.md](./SKILL_GOLDEN_STANDARD.md)  
**Backlog:** [DEFERRED_WORK_HANDOFF.md](./handoff/DEFERRED_WORK_HANDOFF.md) (Wave 2b — chat/TUI `maybe_teach` parity)

## Three trees

| Tree | Role | Used by |
|------|------|---------|
| **Rust registry** | Authoritative slash skills | `gzmo-core/src/skills/` → `dispatch_skill` |
| **Repo `skills/`** | Shell pantheon / next instance | `[skills] directory` in `config/gzmo-next.toml` |
| **`gzmo_skills/` sibling** | CT101 discovery cycles, Pi mentor bridge, timers | Auxiliary — do **not** dual-load the same script from both shell trees |

```
/slash → dispatch_skill()
           ├─ Rust Skill trait → ChaosEvent → PulseLoop → Thought Cabinet (when chaos on)
           └─ shell_bridge (legacy fallback only)
```

**Golden rule:** Rust registry wins. Shell scripts that still exist should delegate to `gzmo chaos skill <cmd>` or the Rust path.

## Living vs next

| Instance | Skills root | Notes |
|----------|-------------|-------|
| CT101 living | Release tree under `/opt/gzmo/current` + discovery scripts | Paths: [CT101_PATH_AUTHORITY.md](./ops/CT101_PATH_AUTHORITY.md) |
| Workstation next | `GZMO/skills/` via `gzmo-next.toml` | Resolve with `gzmo instance status` → `skills_root` |

## Pedagogy surface gap (Wave 2b)

| Surface | `maybe_teach` / pedagogy orchestrator |
|---------|----------------------------------------|
| `gzmo mentor` / mentor IPC | Yes — headless client + optional daemon socket |
| `gzmo chat` | Yes — Wave 2b mentor path before `run_agent_loop` |
| `gzmo tui` | Yes — Wave 2b.1 mentor path before `run_agent_loop` |
| `gzmo daemon` | No chat loop — mentor via Unix socket when wired |

Demable smoke: `bash scripts/verify-mentor.sh`. Chaos `pedagogy_oscillator` (Slice C.1)
stays lab-only — never living overnight / PulseLoop on CT101.

## Legendary packs

Front door: [PANTHEON_SKILLS.md](./PANTHEON_SKILLS.md) — ritual/lab only; not CT101 chaos-free mentor KPI.

Archive: [research/pantheon/](./research/pantheon/) (standardization handoff, Card Forge, dice tiers, story V2, Final Pack). Quality bar stays [SKILL_GOLDEN_STANDARD.md](./SKILL_GOLDEN_STANDARD.md).

**On main:** Slice A full + C.0/C.0.1 — see [PANTHEON_FEAT_RELAND.md](./PANTHEON_FEAT_RELAND.md).
Still deferred: daemon `dice_loop` fire, Slice C.1 pedagogy oscillator. Ghost
`DICE_MASTER_*` masters never existed — do not invent them.
