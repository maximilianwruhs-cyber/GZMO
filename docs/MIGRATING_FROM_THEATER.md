# Migrating from Theatrical Language — Developer Guide

This guide helps you identify, understand, and replace theatrical language in GZMO code.

**Before reading:** See `THEATRICAL_LANGUAGE_CATALOG.md` for the complete term dictionary.

---

## Why Migrate?

Theatrical language obscures functionality:

**Problem:**
```rust
// What does this actually do?
if thought.crystallize() {
    cabinet.absorb(seed);
}
```

**Solution:**
```rust
// Clear: queue and apply parameter updates
if mutation.apply() {
    queue.enqueue(request);
}
```

---

## Migration Priority Matrix

| Priority | Impact | Example | Effort |
|----------|--------|---------|--------|
| **P0 (Critical)** | Confuses core logic | Struct names, function signatures | High |
| **P1 (High)** | Misleading API | Module names, public functions | Medium |
| **P2 (Medium)** | Internal clarity | Private functions, comments | Low |
| **P3 (Low)** | Documentation only | README, user-facing copy | Optional |

---

## Step-by-Step Migration

### Step 1: Identify Theatrical Terms

Use grep patterns to find theatrical language:

```bash
# Find "thought" terminology
grep -r "Thought\|thought\|crystalliz\|incubat\|absorb" --include="*.rs" src/

# Find "chaos" physics theater
grep -r "chaos\|attractor\|orbit\|phase.*space\|entropy" --include="*.rs" src/

# Find "dream" terminology
grep -r "dream\|sleep\|REM\|consolidat" --include="*.rs" src/

# Find "mood" and emotions
grep -r "mood\|valence\|affective\|emotion" --include="*.rs" src/
```

### Step 2: Map to Honest Equivalents

For each term found, look up in `THEATRICAL_LANGUAGE_CATALOG.md`:

| Found Term | Honest Equivalent | File |
|------------|-------------------|------|
| `ThoughtCabinet` | `ParameterMutationQueue` | `thoughts.rs` |
| `crystallize` | `apply` | `thoughts.rs` |
| `dream` | `nightly_batch_etl` | `dreams.rs` |
| `chaos` | `state_generator` | `chaos.rs` |

### Step 3: Plan Renaming

Use IDE rename refactoring in this order:

1. **Struct/Enum names** (affects all uses)
2. **Function names** (affects call sites)
3. **Field names** (affects accessors)
4. **Variable names** (local scope)
5. **Comments** (documentation)

### Step 4: Maintain Backward Compatibility

Add deprecation aliases for public APIs:

```rust
/// DEPRECATED: Use `ParameterMutationQueue` instead
#[deprecated(since = "0.2.0", note = "Use ParameterMutationQueue")]
pub type ThoughtCabinet = ParameterMutationQueue;
```

---

## Common Migration Patterns

### Pattern 1: Struct Rename

**Before:**
```rust
pub struct ThoughtCabinet {
    slots: Vec<Option<IncubatingThought>>,
    mutations: Mutations,
}

pub struct IncubatingThought {
    category: String,
    ticks_remaining: u64,
}
```

**After:**
```rust
pub struct ParameterMutationQueue {
    queue: Vec<Option<PendingMutation>>,
    modifications: ParameterModifications,
}

pub struct PendingMutation {
    category: String,
    ticks_remaining: u64,
}

// Backward compatibility
#[deprecated(since = "0.2.0", note = "Use ParameterMutationQueue")]
pub type ThoughtCabinet = ParameterMutationQueue;
#[deprecated(since = "0.2.0", note = "Use PendingMutation")]
pub type IncubatingThought = PendingMutation;
```

### Pattern 2: Function Rename

**Before:**
```rust
impl ThoughtCabinet {
    pub fn try_absorb(&mut self, category: &str, text: &str, tick: u64, roll: f64) -> bool {
        // ...
    }
    
    pub fn tick(&mut self) -> Vec<CrystallizationEvent> {
        // ...
    }
}
```

**After:**
```rust
impl ParameterMutationQueue {
    pub fn try_queue(&mut self, category: &str, text: &str, tick: u64, roll: f64) -> bool {
        // ...
    }
    
    pub fn tick(&mut self) -> Vec<AppliedMutation> {
        // ...
    }
    
    // DEPRECATED: Backward compatibility
    #[deprecated(since = "0.2.0", note = "Use try_queue")]
    pub fn try_absorb(&mut self, category: &str, text: &str, tick: u64, roll: f64) -> bool {
        self.try_queue(category, text, tick, roll)
    }
}

#[deprecated(since = "0.2.0", note = "Use AppliedMutation")]
pub type CrystallizationEvent = AppliedMutation;
```

### Pattern 3: Module Rename

**Before:**
```rust
// In lib.rs
pub mod thoughts;
```

**After:**
```rust
// In lib.rs
pub mod mutation_queue;

// Re-export with deprecation
#[deprecated(since = "0.2.0", note = "Use mutation_queue")]
pub mod thoughts {
    pub use super::mutation_queue::*;
}
```

### Pattern 4: Comment Updates

**Before:**
```rust
/// Crystallization impulses on `lorenz_rho_mod`: joke −0.2, quote +0.3
/// Per-tick decay runs in `pulse.rs`.
fn apply_rho_mutation(&mut self, rho_mod: f64) {
    self.rho = 28.0 + rho_mod.clamp(-10.0, 10.0);
}
```

**After:**
```rust
/// Apply permanent rho modification from queued parameter requests.
/// Modifications accumulate and decay over time (see `apply_decay`).
fn apply_rho_mutation(&mut self, rho_mod: f64) {
    self.rho = 28.0 + rho_mod.clamp(-10.0, 10.0);
}
```

---

## File-by-File Migration Guide

### `thoughts.rs` → `mutation_queue.rs`

| Theatrical | Honest | Type |
|------------|--------|------|
| `ThoughtCabinet` | `ParameterMutationQueue` | struct |
| `IncubatingThought` | `PendingMutation` | struct |
| `CrystallizationEvent` | `AppliedMutation` | struct |
| `MutationEffect` | `ParameterEffect` | struct |
| `Mutations` | `ParameterModifications` | struct |
| `try_absorb` | `try_queue` | method |
| `crystallize` | `apply` | method |
| `incubation_period` | `default_delay_ticks` | function |
| `absorb_threshold` | `batch_threshold` | constant |
| `thought_drain_mod` | `queue_overhead` | field |

### `dreams.rs` → `nightly_etl.rs`

| Theatrical | Honest | Type |
|------------|--------|------|
| `DreamEngine` | `NightlyEtlEngine` | struct |
| `DreamReport` | `EtlReport` | struct |
| `DreamEntity` | `ExtractedEntity` | type alias |
| `DreamRelation` | `EntityRelation` | type alias |
| `consolidate` | `run_etl` | method |
| `dream_episodic_source` | `etl_log_source` | function |
| `DREAM_EXTRACT_SYSTEM` | `ETL_EXTRACT_PROMPT` | constant |

### `pulse.rs` Updates

| Theatrical | Honest | Context |
|------------|--------|---------|
| "heartbeat" | "tick interval" | comments |
| "174 BPM" | "adaptive tempo" | documentation |
| "organism" | "system" | error messages |
| "mood" | "parameter state" | comments |
| "breath phase" | "velocity direction" | variable names |
| "crystallized" | "applied" | field names |
| "incubating" | "pending" | field names |

### `chaos.rs` Updates

| Theatrical | Honest | Context |
|------------|--------|---------|
| "attractor" | "state generator" | comments |
| "orbit" | "trajectory" | comments |
| "phase space" | "parameter space" | documentation |
| "cognitive noise" | "perturbation" | method names |
| "strange attractor" | "deterministic sequence" | comments |

---

## Testing Renamed Code

### 1. Compilation Check

```bash
cargo check --all-targets
```

### 2. Run Existing Tests

```bash
cargo test --lib
```

### 3. Check for Deprecation Warnings

```bash
cargo build 2>&1 | grep -i deprecated
```

### 4. Update Test Names

Update test names to use honest terminology:

**Before:**
```rust
#[test]
fn joke_cools_rho() { ... }

#[test]
fn thought_absorbed_into_cabinet() { ... }
```

**After:**
```rust
#[test]
fn low_priority_reduces_rho() { ... }

#[test]
fn mutation_queued() { ... }
```

---

## Migration Checklist

### For Each File Being Modified

- [ ] Identify all theatrical terms (use grep patterns)
- [ ] Map to honest equivalents (catalog lookup)
- [ ] Rename structs/enums with deprecation aliases
- [ ] Rename functions with deprecation wrappers
- [ ] Update field names
- [ ] Update variable names
- [ ] Update comments
- [ ] Update doc strings
- [ ] Update test names
- [ ] Run `cargo check`
- [ ] Run `cargo test`
- [ ] Verify deprecation warnings work
- [ ] Update CHANGELOG.md

### Project-Wide

- [ ] Update `lib.rs` re-exports
- [ ] Update `README.md` references
- [ ] Update API documentation
- [ ] Update user-facing error messages
- [ ] Update configuration file examples
- [ ] Update CLI help text
- [ ] Test backward compatibility

---

## Dealing with Resistance

### "The theatrical language has vibe/worldbuilding"

**Response:** Vibe belongs in user-facing copy, not internal APIs. Keep theatrical terms in:
- Marketing materials
- User documentation (if desired)
- CLI output formatting
- Log message flavor text

Remove from:
- Struct/function names
- API documentation
- Internal comments
- Architecture descriptions

### "It's too much work to rename everything"

**Response:** Prioritize:
1. P0: Rename core structs (1 day)
2. P1: Rename public APIs (2 days)
3. P2/P3: Incremental comment updates (ongoing)

Use deprecation aliases to maintain compatibility during transition.

### "The metaphors help me understand the system"

**Response:** If you need metaphors to understand it, the implementation isn't clear. After honest renaming:
- Read `HONEST_GZMO.md` for plain-language architecture
- Check `THEATRICAL_LANGUAGE_CATALOG.md` for mappings
- Use honest terms to build accurate mental models

---

## Before/After Examples

### Function Documentation

**Before:**
```rust
/// Attempt to absorb a lore/skill item as an unprocessed thought.
/// Returns true if absorbed (slot available and chaos roll passes threshold).
///
/// Fused thoughts produce amplified mutations on crystallize.
pub fn try_absorb(&mut self, category: &str, text: &str, tick: u64, roll: f64) -> bool
```

**After:**
```rust
/// Attempt to queue a parameter mutation request.
/// Returns true if queued (capacity available and probability threshold met).
///
/// Similar requests are batched for efficiency and applied after delay.
pub fn try_queue(&mut self, category: &str, text: &str, tick: u64, probability: f64) -> bool
```

### Error Messages

**Before:**
```rust
bail!("The organism is exhausted and cannot process more thoughts. \
       Await rebirth or reduce cognitive load.");
```

**After:**
```rust
bail!("System capacity exceeded. \
       Reduce workload or wait for processing queue to clear.");
```

### Log Messages

**Before:**
```rust
info!(
    category = %category,
    text = %text.chars().take(40).collect::<String>(),
    "🧠 Thought absorbed into cabinet"
);
```

**After:**
```rust
info!(
    category = %category,
    text = %text.chars().take(40).collect::<String>(),
    "🧠 Parameter mutation queued"
);
```

---

## Tools and Scripts

### Automated Term Finder

```bash
#!/bin/bash
# find_theater.sh - Find theatrical terms in codebase

cd "$(dirname "$0")/.."

echo "=== Category: Vitalism ==="
grep -rn "alive\|deaths\|heartbeat\|organism\|breath\|rebirth" \
  --include="*.rs" src/ | head -20

echo "=== Category: Cognitive ==="
grep -rn "ThoughtCabinet\|crystalliz\|incubat\|absorb\|dream\|consolidat" \
  --include="*.rs" src/ | head -20

echo "=== Category: Physics ==="
grep -rn "chaos\|attractor\|orbit\|phase space\|entropy\|turbulence" \
  --include="*.rs" src/ | head -20
```

### Renaming Checklist Generator

```bash
#!/bin/bash
# generate_checklist.sh - Create migration checklist

FILE=$1

echo "=== Migration Checklist for $FILE ==="
echo ""
echo "Structs/Enums:"
grep -n "pub struct\|pub enum" "$FILE" | while read line; do
  echo "  [ ] $(echo $line | cut -d: -f2-)"
done

echo ""
echo "Functions:"
grep -n "pub fn\|pub async fn" "$FILE" | while read line; do
  echo "  [ ] $(echo $line | cut -d: -f2-)"
done

echo ""
echo "Run: cargo check && cargo test"
```

---

## See Also

- `THEATRICAL_LANGUAGE_CATALOG.md` — Complete term dictionary
- `HONEST_GZMO.md` — Architecture without theater
- `REMEDIATION_REPORT.md` — Completed migration examples