# Skills Bridge — Rust, Shell, gzmo_skills

**Status:** Operator guide (2026-07-19)  
**Sibling detail:** `/home/gzmo/github-clone/gzmo_skills/BRIDGE.md` (may still cite `survey_GZMO` paths)  
**Quality bar:** [SKILL_GOLDEN_STANDARD.md](./SKILL_GOLDEN_STANDARD.md)  
**Backlog:** [DEFERRED_WORK_HANDOFF.md](./DEFERRED_WORK_HANDOFF.md) (TUI/daemon pedagogy parity)

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
| CT101 living | Release tree under `/opt/gzmo/current` + discovery scripts | Paths: [CT101_PATH_AUTHORITY.md](./CT101_PATH_AUTHORITY.md) |
| Workstation next | `GZMO/skills/` via `gzmo-next.toml` | Resolve with `gzmo instance status` → `skills_root` |

## Pedagogy surface gap

| Surface | `maybe_teach` / pedagogy orchestrator |
|---------|----------------------------------------|
| `gzmo chat` | Yes |
| `gzmo tui` | No (slash skills only) |
| `gzmo daemon` | No chat loop — mentor via Unix socket |

Do not assume TUI/daemon teach parity until deferred work lands.

## Legendary packs (research until code re-land)

Front door: [PANTHEON_SKILLS.md](./PANTHEON_SKILLS.md) — ritual/lab only; not CT101 chaos-free mentor KPI.

Archive: [research/pantheon/](./research/pantheon/) (standardization handoff, Card Forge, dice tiers, story V2, Final Pack). Quality bar stays [SKILL_GOLDEN_STANDARD.md](./SKILL_GOLDEN_STANDARD.md).

Würfel / cascade / feat attractor stack are **Unpark Wave 2** — see [PANTHEON_FEAT_RELAND.md](./PANTHEON_FEAT_RELAND.md). On this feature branch: **Slice A full + PKM Forge** — TOML dice corpus, nested wild-magic cascade via `dispatch`, Card Forge, story, and Rust-native `/pkm` (`attractor_common` / `generative`). `/pkm` now participates in cascade dispatch; its shell script is only a thin Rust delegate. Still deferred: daemon `dice_loop`, Slice C chaos IPC. Ghost `DICE_MASTER_*` masters never existed — do not invent them.
