# Subsystem — Engine Profiles

**Source:** `gzmo-core/src/config.rs` (`EngineSection`, `RoutingConfig`, `TaskKind`)  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Defines dual-profile engine layout (`local` / `cloud` / `sovereign`), active mode switching, and static task→profile mappings. `GatewayRouter` resolves profile names into concrete HTTP endpoints and models.

---

## 2. How it works

### TaskKind enum

```71:94:gzmo-core/src/config.rs
pub enum TaskKind {
    Chat,
    Daemon,
    DreamExtract,
    DreamVerify,
    SparkHypothesis,
    SparkVerify,
    IngestExtract,
    IngestVerify,
    DistillExtract,
    DistillVerify,
    DistillSummary,
}
```

Background vs chat:

```153:155:gzmo-core/src/config.rs
    pub fn is_background(&self) -> bool {
        !matches!(self, Self::Chat)
    }
```

### Engine section (dual-profile)

```1439:1451:gzmo-core/src/config.rs
pub struct EngineSection {
    pub active_mode: EngineMode,
    pub local: Option<EngineProfileConfig>,
    pub cloud: Option<CloudEngineConfig>,
    pub sovereign: Option<EngineProfileConfig>,
    // legacy flat fields fallback
}
```

Active engine resolution:

```1493:1540:gzmo-core/src/config.rs
    pub fn active_engine(&self) -> EngineProfileConfig {
        match self.active_mode {
            EngineMode::Local => { /* local profile or legacy */ }
            EngineMode::Cloud => { /* cloud profile or warn+fallback local */ }
            EngineMode::Sovereign => { /* sovereign or fallback */ }
        }
    }

    pub fn active_engine_for_mode(&self, mode: EngineMode) -> EngineProfileConfig {
        // Pin local/prime without changing active_mode
    }
```

### Routing config (Obolus table)

```1699:1721:gzmo-core/src/config.rs
pub struct RoutingConfig {
    pub default_engine: String,
    pub cloud_first_background: bool,
    pub mappings: HashMap<String, String>,
    pub profiles: HashMap<String, EngineProfileConfig>,
}
```

Example from doc comment:

```1680:1697:gzmo-core/src/config.rs
/// [routing]
/// default_engine = "local"
/// [routing.mappings]
/// dream_extract = "librarian"
/// distill_extract = "librarian"
/// [routing.profiles.librarian]
/// url = "http://192.168.31.110:8083/v1"
```

Resolve task → profile name:

```1731:1734:gzmo-core/src/config.rs
    pub fn resolve(&self, task: TaskKind) -> &str {
        let key = task.to_string();
        self.mappings.get(&key).map(|s| s.as_str()).unwrap_or(&self.default_engine)
    }
```

### CT101 live profile (2026-07-14)

| Field | Value |
|-------|-------|
| `active_mode` | `cloud` |
| Cloud model | `z-ai/glm-5.2` via OpenRouter |
| `reasoning_effort` | `xhigh` |
| Local fallback | Workstation Prime `:8000` |
| Shallow tasks | VM200 librarian `:8083` (when mapped) |

Config authority: `/opt/gzmo/gzmo.toml` (not workspace clone).

---

## 3. Interfaces

| Interface | Example |
|-----------|---------|
| Config file | `/opt/gzmo/gzmo.toml` |
| Mode persist | `GzmoConfig::persist_active_mode(path, mode)` |
| Env overrides | `GZMO_OPENROUTER_KEY`, `GZMO_GEMINI_KEY` via dotenv |
| CLI mode switch | `/mode cloud` in REPL (persists active_mode) |
| Inline profiles | `[routing.profiles.<name>]` |

---

## 4. THINKING nodes

> **THINKING — config.rs:dual-profile layout**
> - *Reviewed:* Legacy flat `[engine]` fields still supported as local fallback.
> - *Insight:* Migration path from single-profile configs without breaking CT101.
> - *Risk / limitation:* Ambiguity when both flat and `engine.local` exist.
> - *Enhancement:* Config linter warning on duplicate profile definitions. [CT101-safe]

> **THINKING — config.rs:cloud fallback fields**
> - *Reviewed:* `CloudEngineConfig` optional Gemini fallback for OpenRouter outages.
> - *Insight:* Nested FallbackGateway at cloud leaf before task-level fallback.
> - *Risk / limitation:* Three-tier fallback chain hard to reason about in logs.
> - *Enhancement:* Structured routing decision log per request. [CT101-safe]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| `gzmo-handoff.sh` | Benchmark gate → apply `*-fused.toml` (lab only) |
| Sovereign | `[engine.sovereign]` :8010 when GGUF ready |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | `gzmo config lint` for profile consistency | [CT101-safe] |
| 2 | Document CT101 authoritative routing table | [CT101-safe] |
| 3 | Per-profile rate limit config | [GZMO-next] |
| 4 | Auto-select librarian vs cloud by token estimate | [GZMO-next] |
