# GZMO — Sovereign Autonomous Agent

> **100% Local · Air-Gapped · Rust**

GZMO is a fully sovereign AI agent that runs entirely on your local hardware. Zero cloud dependencies by default, with optional cloud expansion. It operates in two modes: an **interactive chat REPL** and a **background daemon** with scheduled orchestration, file watchers, and autonomous dream consolidation.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Directory Layout](#directory-layout)
- [Quick Start](#quick-start)
- [Boot Sequence](#boot-sequence)
- [Chat Mode (Interactive)](#chat-mode-interactive)
- [Daemon Mode (Background)](#daemon-mode-background)
- [Configuration — gzmo.toml](#configuration--gzmotoml)
- [Engine Modes — Local vs Cloud](#engine-modes--local-vs-cloud)
- [Slash Commands](#slash-commands)
- [Skills](#skills)
- [Tools (Agent Capabilities)](#tools-agent-capabilities)
- [Memory System](#memory-system)
- [Chaos Engine](#chaos-engine)
- [Building from Source](#building-from-source)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

- **Linux** (tested on Ubuntu/Debian)
- **NVIDIA GPU** (recommended, 6GB+ VRAM) or CPU-only mode
- **Rust toolchain** (for building from source)
- `curl`, `findmnt` (for boot.sh health checks and watchdog)
- Models in `models/` directory (GGUF format)

---

## Directory Layout

```
GZMO_v0.0.1/
├── bin/                    # Static binaries
│   ├── gzmo-static         # Main GZMO binary (what boot.sh launches)
│   ├── llama-server-cuda    # CUDA inference engine
│   └── llama-server-cpu     # CPU fallback inference engine
├── models/                 # GGUF model files (auto-selected by hardware)
├── memory/                 # Episodic memory logs (YYYY-MM-DD.md)
├── data/
│   ├── vault.db            # SQLite semantic knowledge vault
│   └── sessions/           # Saved chat sessions
├── skills/                 # Slash command scripts and skill definitions
├── inbox/                  # Drop files here → auto-ingested by daemon watcher
├── gzmo.toml               # Master configuration (single source of truth)
├── SOUL.md                 # Agent identity and persona definition
├── DREAMS.md               # Nightly dream consolidation output
├── CHAOS_STATE.json        # Live chaos engine telemetry
├── boot.sh                 # Full-stack bootstrapper
├── gzmo-core/              # Core Rust library (engine, tools, memory)
├── gzmo-cli/               # CLI binary (chat REPL, daemon, init)
└── gzmo-chaos/             # Chaos engine (Lorenz attractor, Thought Cabinet)
```

---

## Quick Start

### 1. Boot everything (recommended)

```bash
cd ~/Schreibtisch/GZMO/gzmo_latest/GZMO_v0.0.1
./boot.sh
```

This starts the **inference engine** + **daemon** in one shot. Leave this terminal running.

### 2. Chat with GZMO

Open a **second terminal**:

```bash
cd ~/Schreibtisch/GZMO/gzmo_latest/GZMO_v0.0.1
./bin/gzmo-static
```

That drops you into the interactive REPL. Start typing.

### 3. Stop everything

Press `Ctrl+C` in the boot.sh terminal. The trap handler will:
- Kill the inference engine
- Kill the daemon
- Clean up PID lockfiles
- Sanitize the host environment

---

## Boot Sequence

`boot.sh` performs the following in order:

1. **Singleton lock** — prevents duplicate instances via `/tmp/gzmo_daemon.pid`
2. **Pre-flight checks** — verifies `gzmo.toml`, `models/`, `memory/` exist
3. **Binary resolution** — finds the GZMO binary in priority order:
   - `./bin/gzmo-static` (production static binary)
   - `./target/release/gzmo` (release build)
   - `./target/debug/gzmo` (debug build)
4. **Inference engine selection** — CUDA → CPU → system `llama-server`
5. **Hardware recon** — profiles GPU VRAM and system RAM
6. **Model selection ladder** — automatically picks the best model for your hardware:

   | VRAM | Model | Target |
   |------|-------|--------|
   | 22GB+ | `qwen3.5-35b-a3b` | Enthusiast multi-node |
   | 16GB+ | `qwen2.5-7b-instruct` | High-tier |
   | 6GB+ | `gemma-4-E4B-it` | Mid-tier |
   | 4GB+ | `nemotron-3-nano-4b` | Entry GPU |
   | 8GB+ RAM | `ggml-model-i2_s` (BitNet) | CPU ternary |
   | Fallback | `qwen2.5-0.5b-instruct` | Minimal draft |

7. **Auto-patches** `gzmo.toml` with the selected model
8. **Starts inference engine** on `localhost:1234`
9. **Waits for model to load** into VRAM (up to 120s)
10. **Spawns GZMO daemon** in background
11. **Arms USB extraction watchdog** — auto-teardown if the drive is physically removed

---

## Chat Mode (Interactive)

```bash
./bin/gzmo-static
```

This is the **default command** (no subcommand needed). It launches the interactive REPL with:

- **Chaos engine** — Lorenz attractor modulating LLM temperature in real-time
- **Tool calling** — the agent can read/write files, run shell commands, search the web, and more
- **Session persistence** — conversations are auto-saved on `/quit`
- **SOUL.md hot-reload** — edit the persona file and changes take effect on the next message
- **Vault memory injection** — long-term knowledge automatically loaded into context

### Talking to GZMO

Just type naturally. GZMO has access to tools and will use them autonomously:

```
★ you › What's my disk usage looking like?
```

GZMO will call `shell_exec` or `sys_metrics` internally, analyze the output, and respond.

---

## Daemon Mode (Background)

The daemon runs headless with:

- **Cron-scheduled jobs** — defined in `gzmo.toml` under `[orchestration.jobs]`
- **File watchers** — monitor directories for new files and auto-process them
- **Dream consolidation** — nightly compression of episodic memory into semantic truths
- **Heartbeat** — periodic health checks on the inference engine

The daemon is launched automatically by `boot.sh`. To run it manually:

```bash
./bin/gzmo-static daemon
```

### Default Jobs

| Job | Schedule | Description |
|-----|----------|-------------|
| `sys_janitor` | Every 30 min | Check CPU/RAM/disk, kill suspicious processes |
| `DreamEngine` (`[dreams]`) | 1:00 UTC daily | Gated dream consolidation (`gzmo dream`) |
| `SparkEngine` (`[spark]`) | 09/14/21:17 UTC | Gated serendipitous recall (`gzmo spark`) |

### Knowledge ingest (`[ingest]`)

With `[ingest].enabled`, the daemon uses **IngestEngine** on the watcher directory (see `gzmo.toml` `inbox_ingest`). Drop `.md` / docs there for extract→verify→promote. CLI: `gzmo ingest <path>`.

---

## Configuration — gzmo.toml

The single source of truth for all settings. Key sections:

### Identity

```toml
[identity]
soul_path = "SOUL.md"
```

### Engine (Dual-Mode)

```toml
[engine]
active_mode = "local"    # "local" or "cloud"

[engine.local]
provider    = "local"
url         = "http://localhost:1234/v1"
model       = "gemma-4-E4B-it-Q4_K_M.gguf"
temperature = 0.3
max_tokens  = 8192

[engine.cloud]
provider    = "openrouter"
url         = "https://openrouter.ai/api/v1"
model       = "openrouter/free"
api_key     = "sk-or-..."
```

### API Keys

```toml
[api_keys]
serpapi     = "..."    # Web search (env: GZMO_SERPAPI_KEY)
openrouter  = "..."    # Cloud LLM (env: GZMO_OPENROUTER_KEY)
gemini      = "..."    # Fallback LLM (env: GZMO_GEMINI_KEY)
```

Environment variables take precedence over config values.

### Chaos Engine

```toml
[chaos]
gravity = 9.8
friction = 0.5
seed = 0.506
initial_tension = 0.0
lore_path = "data/lore.toml"
```

---

## Engine Modes — Local vs Cloud

GZMO supports hot-swapping between local and cloud inference at runtime:

```
★ you › /mode cloud     # Switch to cloud (OpenRouter)
★ you › /mode local     # Switch back to local
★ you › /mode           # Show current mode
```

Mode changes are **persisted** to `gzmo.toml` automatically.

If the primary cloud endpoint fails, GZMO falls back to the configured `fallback_*` engine (e.g., Google Gemini API).

---

## Slash Commands

| Command | Description |
|---------|-------------|
| `/quit`, `/exit`, `/q` | Save session and exit |
| `/clear`, `/reset` | Clear conversation context |
| `/new` | Save current session, start fresh |
| `/resume` | Resume most recent session |
| `/save [name]` | Save current session (with optional name) |
| `/load <id\|name>` | Load a saved session |
| `/sessions` | List all saved sessions |
| `/mode [local\|cloud]` | Show or switch engine mode |
| `/stats` | Show session stats, model, mode |
| `/system` | Display current system prompt |
| `/chaos` | Show Chaos Engine state (Lorenz attractor, Thought Cabinet) |
| `/vault` | Show recent entries from semantic vault |
| `/remember <fact>` | Store a fact in the knowledge vault |

### Skills (Slash Commands)

Built-in Rust skills (execute instantly, no LLM call):

| Skill | Description |
|-------|-------------|
| `/dice [NdM]` | Roll dice (e.g., `/dice 2d20`) |
| `/poker` | Draw a poker hand |
| `/quote` | Random quote from lore |
| `/calculate <expr>` | Evaluate a math expression |
| `/sound` | Sound effects and audio |
| `/visual` | Visual effects |
| `/help` | List all available skills |

Additional shell-based skills are in the `skills/` directory and dispatched via `skill_dispatch.sh`.

---

## Tools (Agent Capabilities)

These are tools the LLM can call autonomously during conversation:

| Tool | Description |
|------|-------------|
| `file_read` | Read file contents |
| `file_write` | Write/create files |
| `dir_list` | List directory contents |
| `file_search` | Search for files by name/pattern |
| `shell_exec` | Execute shell commands |
| `web_search` | Search the web (SerpAPI or DuckDuckGo) |
| `web_browse` | Fetch and read web page content |
| `sys_metrics` | Get CPU, RAM, disk usage |
| `sys_kill` | Kill a process by PID |
| `memory_record` | Store information in the semantic vault |
| `memory_search` | Search the semantic vault |

---

## Memory System

GZMO has a three-tier memory architecture:

### 1. Episodic Memory (`memory/YYYY-MM-DD.md`)
- Raw conversation logs and events, written in real-time
- One file per day

### 2. Semantic Vault (`data/vault.db`)
- SQLite database with semantic search
- Long-term facts and knowledge
- Populated by `/remember`, agent tool calls, and dream consolidation

### 3. Session Store (`data/sessions/`)
- Full conversation histories with metadata
- Resumable via `/resume` or `/load`

### Dream Consolidation

Every night at 3 AM (daemon mode), the **autoDream** job:
1. **Light phase** — compresses yesterday's episodic memory
2. **REM phase** — extracts entities via LLM
3. **Deep phase** — writes to Knowledge Graph + Vault
4. Outputs a narrative to `DREAMS.md`

---

## Chaos Engine

GZMO's internal Lorenz attractor simulation drives:

- **LLM temperature** — chaos coordinates modulate creativity in real-time
- **Mood/valence** — subtly colors response tone without explicit mentions
- **Thought Cabinet** — ideas incubate, crystallize, and mutate system parameters
- **Autonomous triggers** — critical tension/energy thresholds fire notifications or skills
- **Lore emission** — random jokes, quotes, and facts from `data/lore.toml`

View live state with `/chaos` in chat mode, or read `CHAOS_STATE.json`.

---

## Building from Source

```bash
# Debug build (faster compilation, larger binary)
cargo build

# Release build (optimized, what boot.sh prefers)
cargo build --release

# Update the static binary that boot.sh uses
cp target/release/gzmo bin/gzmo-static
```

### Workspace Structure

```
Cargo.toml          # Workspace root
├── gzmo-core/      # Core library: config, gateway, tools, memory, orchestrator
├── gzmo-cli/       # Binary: chat REPL, daemon, init, memory dump
└── gzmo-chaos/     # Chaos engine: Lorenz attractor, pulse loop, triggers
```

### Other CLI Commands

```bash
./bin/gzmo-static              # Chat mode (default)
./bin/gzmo-static daemon       # Daemon mode
./bin/gzmo-static init         # Initialize a new GZMO project
./bin/gzmo-static dump         # Export vault to markdown
./bin/gzmo-static memory dump  # Same as above
```

---

## Troubleshooting

### Daemon won't start: "already running" error

```bash
rm -f /tmp/gzmo_rust.pid /tmp/gzmo_daemon.pid
```

Then restart with `./boot.sh`.

### Wrong model logged by daemon

If you see a model mismatch between boot.sh and the daemon log:

1. **Rebuild the binary**: `cargo build --release`
2. **Update the static binary**: `cp target/release/gzmo bin/gzmo-static`
3. Restart `./boot.sh`

The root cause is usually `bin/gzmo-static` being stale (compiled before config changes).

### Engine unreachable / HTTP 503

The model hasn't finished loading into VRAM. Wait for boot.sh to report:
```
✔ Phantom internal inference engine operational. Model fully loaded.
```

### Port 1234 already in use

Another inference engine (LM Studio, Ollama, etc.) is running. boot.sh will detect and reuse it if it's healthy. To force a fresh start:

```bash
# Kill existing engine
pkill -f llama-server
# Then re-run boot.sh
```

### Chat mode shows "Engine OFFLINE"

Your local inference engine isn't running. Either:
- Run `./boot.sh` in another terminal first, or
- Start LM Studio / Ollama on port 1234, or
- Switch to cloud: type `/mode cloud` in chat

### Verbose logging

```bash
RUST_LOG=debug ./bin/gzmo-static
RUST_LOG=info ./bin/gzmo-static daemon
```

---

## License

Sovereign software. All rights reserved.
