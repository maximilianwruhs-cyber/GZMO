# Honest GZMO — Architecture Without Theater

A plain-language description of GZMO's architecture, using honest technical terms instead of theatrical metaphors.

**For the full catalog of theatrical terms, see:** `THEATRICAL_LANGUAGE_CATALOG.md`

---

## What GZMO Actually Is

GZMO is a **parameter modulation system** for LLM inference. It uses deterministic mathematical functions to vary LLM parameters (temperature, max_tokens) based on system state (workload, queue depth, time).

**Not:** A "mood engine" or "organism" or "chaos field"
**Actually:** A parameter scheduler with deterministic state generators

---

## Core Components (Honestly Named)

### 1. StateGenerator (formerly "Lorenz Attractor")

A 3D deterministic sequence generator using Lorenz ODEs:
- `dx/dt = σ(y-x)`
- `dy/dt = x(ρ-z)-y`
- `dz/dt = xy-βz`

**Purpose:** Generate smooth, non-repeating parameter values
**Input:** Initial seed, step count
**Output:** (x, y, z) coordinates used for temperature/max_tokens mapping

**Not:** An "orbit" in "phase space" with "strange attractor" properties
**Actually:** Three coupled differential equations producing deterministic output

### 2. ParameterMutationQueue (formerly "Thought Cabinet")

A delayed processing queue for parameter updates:
- Enqueues requests with category and delay
- Processes at tick intervals
- Batches similar requests for efficiency

**Purpose:** Smooth parameter changes over time (prevent oscillation)
**Not:** "Thoughts crystallizing" or "incubating"
**Actually:** Queue with configurable delay and deduplication

### 3. AdaptiveTempo (formerly "174 BPM Heartbeat")

Adjusts processing frequency based on workload:
- Empty queue: 5000ms intervals (save resources)
- Moderate load: 500ms intervals
- Heavy load: 50ms intervals (minimize latency)

**Purpose:** Balance responsiveness with resource efficiency
**Not:** A "heartbeat" or "rhythm"
**Actually:** Adaptive timer with workload-based adjustment

### 4. EngineCapacity (formerly "alive/dead states")

Four-tier resource throttling:
- **Full:** All features available (80-100% energy)
- **Reduced:** Disable expensive features (50-79% energy)
- **Minimal:** Core only (20-49% energy)
- **Throttled:** Queue requests for later (0-19% energy)

**Purpose:** Graceful degradation under resource constraints
**Not:** "Death and rebirth" of an organism
**Actually:** Feature flagging based on energy level

### 5. ValidatedTempRange (formerly "0.3-1.2 magic numbers")

Temperature range with validation tracking:
- Stores min/max validated via AttractorBench
- Tracks validation source and date
- Supports workload-specific ranges

**Purpose:** Empirically-grounded parameter bounds
**Not:** "Chaos temperature band"
**Actually:** Configurable range with measurement provenance

---

## Data Pipeline (Honestly Described)

### Ingest Flow

```
Raw Documents → Parse → Deduplicate → Store (SQLite)
     ↓
Vector Embed → Index (Qdrant)
     ↓
Query → Retrieve (Qdrant) → Rerank → Return
```

**Not:** "Memory ingestion with crystallization"
**Actually:** Standard RAG pipeline with SQLite + vector DB

### Nightly ETL Flow (formerly "Dream Cycle")

```
1. EXTRACT: LLM processes daily logs → structured facts
2. VERIFY: Confidence threshold filtering
3. PROMOTE: Insert into KG and vault if validated
```

**Not:** "Light → REM → Deep sleep with memory consolidation"
**Actually:** Scheduled batch ETL with LLM extraction

### Parameter Feedback Loop

```
Detector (n-gram similarity) →
    ↓ High similarity detected
Modulator (increase temperature) →
    ↓ Generate with new parameters
Evaluator (diversity metrics) →
    ↓ Record outcome
Learner (correlation analysis)
```

**Not:** "Chaos engine with mood adaptation"
**Actually:** Closed-loop parameter optimization

---

## LLM Integration (Honestly Described)

### Parameter Modulation

| StateGenerator Output | LLM Parameter | Mapping |
|----------------------|-----------------|---------|
| x coordinate | temperature | normalized to [0.3, 1.2] |
| y coordinate | max_tokens | normalized to [256, 2048] |
| z coordinate | valence bias | normalized to [-1.0, 1.0] |

**Not:** "Chaos-driven mood modulation"
**Actually:** Deterministic parameter assignment from ODE output

### Simplified Pedagogy (formerly "4-Agent Socratic Soul")

Two-LLM-call pipeline:
1. **Evaluator:** Assesses student state (1 call, ~500ms)
2. **Generator:** Produces Socratic response (1 call, ~1500ms)

**Not:** "Diagnoser → Planner → Affective → Tutor agents"
**Actually:** Sequential LLM calls with context passing

---

## Self-Improving Loop (Honestly Described)

### Repetition Detection

Tracks n-gram similarity across recent outputs:
- Sliding window of last N responses
- Jaccard similarity calculation
- Threshold-based stuck detection

**Not:** "Pattern awareness in the chaos field"
**Actually:** String similarity measurement

### Diversity Metrics

| Metric | What It Measures | How |
|----------|------------------|-----|
| Lexical Diversity | Vocabulary variation | unique n-grams / total |
| Pattern Novelty | Repetition vs history | 1 - avg_similarity |
| Latency | Response time | wall clock ms |
| Token Efficiency | Cost per output | tokens / quality_score |

**Not:** "Cognitive assessment of the organism"
**Actually:** Statistical text analysis

### Strategy Learning

Correlates parameters with outcomes:
- Records (temperature, diversity) pairs
- Computes Pearson correlation
- Adjusts parameter ranges based on historical performance

**Not:** "Evolution of the chaos attractor"
**Actually:** Simple regression on historical data

---

## Skill System (Honestly Described)

Skills are Rust functions that:
1. Receive current state snapshot
2. Execute logic (API calls, file I/O, etc.)
3. Optionally emit parameter feedback

**Not:** "Agents in the pantheon invoking chaos magic"
**Actually:** Registered function handlers with state access

### Skill Categories

| Category | What They Do | Examples |
|----------|--------------|----------|
| Information | Retrieve data | fetch_url, search_vault |
| Generation | Create content | generate_text, create_image |
| Analysis | Process data | summarize, classify |
| Utility | System operations | file_write, exec_command |

---

## Configuration (Honestly Described)

### Key Configuration Values

```toml
[parameters]
# Temperature range for LLM sampling
temp_min = 0.3  # Low: precise, deterministic
temp_max = 1.2  # High: creative, variable

# Processing intervals (ms)
tempo_min = 50    # Under heavy load
tempo_max = 5000  # When idle

# Queue configuration
max_queue_depth = 100
batch_threshold = 0.82  # Similarity for batching

# Phase thresholds
tension_idle = 30.0
tension_build = 70.0
```

**Not:** "Chaos physics and organism vital signs"
**Actually:** Standard configuration parameters with documented defaults

---

## System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     GZMO Architecture                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────┐  │
│  │  User Input  │─────▶│   Gateway    │─────▶│   LLM    │  │
│  └──────────────┘      └──────────────┘      └──────────┘  │
│                               │                    ▲       │
│                               ▼                    │       │
│  ┌────────────────────────────────────────────────────┐   │
│  │              Parameter Modulation System             │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │   │
│  │  │ StateGenerator│  │ ParameterQueue │  │ Tempo    │  │   │
│  │  │ (Lorenz ODEs)│  │ (Delay queue)  │  │ (Timer)  │  │   │
│  │  └──────────────┘  └──────────────┘  └──────────┘  │   │
│  └────────────────────────────────────────────────────┘   │
│                               │                           │
│                               ▼                           │
│  ┌────────────────────────────────────────────────────┐   │
│  │              Self-Improving Loop                    │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐         │   │
│  │  │ Detector │ │ Evaluator │ │ Learner  │         │   │
│  │  │(n-gram)  │ │(metrics) │ │(correlate│         │   │
│  │  └──────────┘ └──────────┘ └──────────┘         │   │
│  └────────────────────────────────────────────────────┘   │
│                               │                           │
│                               ▼                           │
│  ┌────────────────────────────────────────────────────┐   │
│  │                  Data Storage                       │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐         │   │
│  │  │  SQLite  │ │ Qdrant   │ │  Files   │         │   │
│  │  │  (Vault) │ │(Vectors) │ │ (Logs)   │         │   │
│  │  └──────────┘ └──────────┘ └──────────┘         │   │
│  └────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Common Operations (Honestly Described)

### Starting the System

```bash
# Start daemon with parameter modulation
gzmo daemon

# Start with telemetry display
gzmo telemetry --watch
```

**Not:** "Awakening the organism"
**Actually:** Spawn background process with timer loop

### Running ETL (formerly "Dream")

```bash
# Process yesterday's logs
gzmo dream 2026-06-25

# Or run on current date
gzmo dream
```

**Not:** "Triggering memory consolidation during REM sleep"
**Actually:** Execute scheduled batch extraction job

### Querying Memory

```bash
# Search vault
gzmo memory "query text"

# Get with context
gzmo memory --context 5 "query"
```

**Not:** "Consulting the crystallized thoughts"
**Actually:** Vector similarity search + SQL query

---

## Performance Characteristics

### Latency Budgets

| Operation | Typical | Worst Case |
|-----------|---------|------------|
| Single LLM call | 500ms | 5000ms |
| Pedagogy (2 calls) | 2000ms | 12000ms |
| Vault search | 50ms | 200ms |
| Vector search | 100ms | 500ms |
| Parameter update | 1ms | 5ms |

**Not:** "Organism response time during various mood states"
**Actually:** Measured latency distributions

### Throughput Limits

- **LLM calls:** Rate-limited by API provider
- **Queue processing:** 1000 items/sec (SQLite)
- **Vector search:** 100 queries/sec (Qdrant)
- **State updates:** 20 updates/sec (tick interval)

---

## Troubleshooting (Honestly Described)

### High Latency

**Symptom:** Responses take >5 seconds

**Causes:**
1. LLM API rate limiting or congestion
2. Heavy queue backlog
3. Large context window processing

**Solutions:**
1. Check API status
2. Reduce tempo_max to process backlog faster
3. Truncate context with --max-context

**Not:** "The organism is exhausted, let it rest"
**Actually:** Profile and reduce workload

### Repetitive Outputs

**Symptom:** LLM generates similar responses repeatedly

**Causes:**
1. Temperature too low
2. Self-improving loop not detecting
3. Input prompts too similar

**Solutions:**
1. Manually increase temp_min
2. Check detector.threshold configuration
3. Add input variation

**Not:** "The chaos field has stagnated"
**Actually:** Increase sampling randomness

### Energy Depletion

**Symptom:** System enters Throttled capacity

**Causes:**
1. Sustained high API call rate
2. Large batch operations
3. Memory leaks (uncommon)

**Solutions:**
1. Wait for regeneration (natural decay)
2. Reduce API call frequency
3. Restart if stuck

**Not:** "The organism has died, await rebirth"
**Actually:** Feature throttling due to load; reduce operations

---

## Glossary of Honest Terms

| When You See | Think | Technical Term |
|--------------|-------|----------------|
| "mood" | parameter state | `ChaosSnapshot` fields |
| "thought" | queued request | `PendingMutation` |
| "dream" | nightly ETL | `DreamEngine::consolidate()` |
| "crystallize" | apply update | `ParameterMutationQueue::apply()` |
| "chaos" | deterministic sequence | `LorenzAttractor` |
| "heartbeat" | timer tick | `PulseLoop::tick()` |
| "organism" | system | GZMO process |
| "pantheon" | registry | `SkillRegistry` |
| "oracle" | predictor | `DiceOracle` skill |
| "spark" | generator | `SparkEngine` |
| "soul" | config | `identity.soul_path` |
| "breath" | update direction | `rho_velocity_ema` sign |

---

## See Also

- `THEATRICAL_LANGUAGE_CATALOG.md` — Complete dictionary of 140+ theatrical terms
- `MIGRATING_FROM_THEATER.md` — Guide for replacing theatrical code
- `ARCHITECTURE.md` — Original theatrical architecture description
- `REMEDIATION_REPORT.md` — Completed remediation work summary