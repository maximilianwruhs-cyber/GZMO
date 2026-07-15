# Assembly Guard — Inline vs Lab Backend Enforcement

**Source:** `gzmo-core/src/assembly.rs`, boot logging in `gzmo-cli/src/daemon_cmd.rs`  
**Parent:** [120-two-stack-boundary/SYSTEM.md](./SYSTEM.md)

---

## Capability

Prevents **accidental lab graft** on CT101: even if `gzmo.toml` sets `[assembly].dream = "lab"` (etc.), the legacy daemon runs **inline** gzmo-core engines unless `GZMO_INSTANCE=next`. Provides shared `run_lab_script()` for GZMO-next and `gzmo assemble` CLI.

---

## How it works

### Instance check

```30:33:github-clone/GZMO/gzmo-core/src/assembly.rs
pub fn instance_is_next() -> bool {
    std::env::var("GZMO_INSTANCE").is_ok_and(|v| v == "next")
}
```

### Effective backend guardrail

```62:71:github-clone/GZMO/gzmo-core/src/assembly.rs
impl AssemblyConfig {
    pub fn effective(&self, configured: AssemblyBackend) -> AssemblyBackend {
        if configured.is_lab() && !instance_is_next() {
            return AssemblyBackend::Inline;
        }
        configured
    }
}
```

### Assembly config slices

```48:60:github-clone/GZMO/gzmo-core/src/assembly.rs
pub struct AssemblyConfig {
    pub distill: AssemblyBackend,
    pub dream: AssemblyBackend,
    pub spark: AssemblyBackend,
    pub ops_health: AssemblyBackend,
    pub config_handoff: AssemblyBackend,
}
```

Default: all `Inline` ([test confirms](github-clone/GZMO/gzmo-core/src/assembly.rs)).

### Daemon boot resolution

```61:77:github-clone/GZMO/gzmo-cli/src/daemon_cmd.rs
    let asm = &config.assembly;
    let distill_backend = asm.effective(asm.distill);
    let dream_backend = asm.effective(asm.dream);
    let spark_backend = asm.effective(asm.spark);
    let ops_backend = asm.effective(asm.ops_health);
    let handoff_backend = asm.effective(asm.config_handoff);
    info!(
        instance = %std::env::var("GZMO_INSTANCE").unwrap_or_else(|_| "legacy".into()),
        distill = distill_backend.label(),
        dream = dream_backend.label(),
        // ...
        "Assembly backends resolved"
    );
```

On CT101: journalctl shows all `inline` regardless of TOML lab flags.

### Lab script runner

```88:109:github-clone/GZMO/gzmo-core/src/assembly.rs
pub fn run_lab_script(script: &str, args: &[&str]) -> Result<()> {
    let path = script_path(script);
    if !path.is_file() {
        anyhow::bail!("lab script not found: {}", path.display());
    }
    let mut cmd = Command::new("bash");
    cmd.arg(&path);
    // ...
}
```

`lab_root()` resolves via `LITTLE_TOOLS_LAB_ROOT` → `GZMO_CLONE_ROOT/little-tools-lab` → `/home/gzmo/github-clone/little-tools-lab`.

### Handoff apply target (GZMO-next only)

```41:45:github-clone/GZMO/gzmo-core/src/assembly.rs
pub fn handoff_apply_target() -> Option<PathBuf> {
    let config = PathBuf::from(std::env::var("GZMO_CONFIG").ok()?);
    let stem = config.file_stem()?.to_string_lossy().into_owned();
    Some(config.with_file_name(format!("{stem}-fused.toml")))
}
```

Never overwrites live instance config — fused output is sibling file for operator review.

---

## Interfaces

| Interface | CT101 | GZMO-next |
|-----------|-------|-----------|
| `GZMO_INSTANCE` | unset / `legacy` | `next` (required for lab) |
| `GZMO_CONFIG` | `/opt/gzmo/gzmo.toml` | `config/gzmo-next.toml` |
| `LITTLE_TOOLS_LAB_ROOT` | unused on CT101 | lab recipe root |
| `[assembly]` TOML | ignored for lab (forced inline) | must be `lab` for scheduler |

---

## THINKING nodes

> **THINKING — assembly.rs:effective guard**
> - *Reviewed:* Single choke point — `is_lab() && !instance_is_next()` → Inline.
> - *Insight:* Defense in depth with CT101_BOUNDARY policy; config typos can't graft lab.
> - *Risk / limitation:* Setting `GZMO_INSTANCE=next` on CT101 by mistake would enable lab subprocesses.
> - *Enhancement:* Hostname guard: refuse `next` on CT101 LXC. [CT101-safe]

> **THINKING — assembly.rs:module doc comment**
> - *Reviewed:* File header states "Not wired into CT101 legacy daemon".
> - *Insight:* Partially superseded — daemon *does* call `effective()` but still forces inline.
> - *Risk / limitation:* Comment may confuse readers — guard is wired, lab is not activated.
> - *Enhancement:* Update module doc to "lab backends gated by GZMO_INSTANCE". [CT101-safe]

> **THINKING — assembly.rs:handoff_apply_target**
> - *Reviewed:* Fused config lands as `gzmo-next-fused.toml`, not in-place overwrite.
> - *Insight:* Prevents `[assembly]`/`[memory]` clobber from partial handoff merge.
> - *Risk / limitation:* Operator must manually promote fused file — easy to forget.
> - *Enhancement:* `gzmo config promote-fused` with diff preview. [GZMO-next]

---

## Advancement

| CT101 | GZMO-next |
|-------|-----------|
| `effective()` always Inline | All loops lab via scheduler |
| `run_lab_script` unused in hot path | Sole execution path for cognition |
| No `[assembly]` edits on CT101 | `[assembly] all lab` in gzmo-next.toml |

---

## Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Hostname guard against `GZMO_INSTANCE=next` on CT101 | [CT101-safe] |
| 2 | Pre-commit lint: no `assembly = "lab"` in CT101 config template | [CT101-safe] |
| 3 | Clarify assembly.rs module documentation | [CT101-safe] |
| 4 | `gzmo config promote-fused` workflow | [GZMO-next] |
| 5 | Log warning if TOML requests lab on legacy instance | [CT101-safe] |
