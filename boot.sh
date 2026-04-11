#!/usr/bin/env bash
# ==============================================================================
# PHANTOM DRIVE IGNITION — GZMO SOVEREIGN OPERATING ENVIRONMENT
#
# Bootstrapper for autonomous USB deployment.
# Implements strict hardware recon, pgid teardown traps, and physical extraction 
# watchdog polling to ensure zero-footprint operation.
# ==============================================================================

set -euo pipefail

# ─── COLOR LOGGING ────────────────────────────────────────────────────────────
GREEN="\032[32m"
YELLOW="\032[33m"
CYAN="\032[36m"
BOLD="\032[1m"
RESET="\032[0m"

log() { echo -e "  ${CYAN}▸${RESET} $1"; }
warn() { echo -e "  ${YELLOW}⚠${RESET} $1"; }
ok() { echo -e "  ${GREEN}✔${RESET} $1"; }
fail() { echo -e "  ${BOLD}⊘ $1${RESET}"; exit 1; }

echo -e "\n${BOLD}╔══════════════════════════════════════════════╗"
echo -e "║          GZMO PHANTOM DRIVE — BOOT           ║"
echo -e "╚══════════════════════════════════════════════╝${RESET}\n"

# ─── 1. PROCESS GROUP & TRAP (THE KILL SWITCH) ────────────────────────────────
# Execution assigns the current shell PID ($$) as the PGID for all children.
PHANTOM_PGID="$$"

trap_cleanup() {
    echo ""
    warn "Termination signal received or execution state failure."
    log "Executing stealth teardown..."
    
    # Ignore incoming signals to prevent recursive trap loops
    trap '' EXIT INT TERM ERR

    # Broadcast SIGTERM to the entire process group
    kill -TERM -- -"${PHANTOM_PGID}" 2>/dev/null || true
    
    # Allow 2 seconds for graceful cache flush and database lock release
    sleep 2
    
    # Broadcast SIGKILL to obliterate any hanging processes
    kill -9 -- -"${PHANTOM_PGID}" 2>/dev/null || true
    
    # Release singleton PID lock
    rm -f "/tmp/gzmo_daemon.pid"
    
    ok "Teardown complete. Host environment sanitized. Phantom Drive inert."
    exit 0
}

trap trap_cleanup EXIT INT TERM ERR

# ─── 1.5. SINGLETON LOCK ──────────────────────────────────────────────────────
PID_FILE="/tmp/gzmo_daemon.pid"
if [ -f "$PID_FILE" ]; then
    OLD_PID=$(cat "$PID_FILE")
    if kill -0 "$OLD_PID" 2>/dev/null; then
        fail "GZMO is already running (PID: $OLD_PID). Highlander rules apply: There can be only one."
    else
        warn "Found stale PID lock ($OLD_PID). Purging..."
        rm -f "$PID_FILE"
    fi
fi
echo $$ > "$PID_FILE"
# ─── 2. PRE-FLIGHT DEPENDENCY CHECKS ──────────────────────────────────────────
log "Running pre-flight checks..."

command -v findmnt >/dev/null || fail "'findmnt' is required for the extraction watchdog."
command -v curl >/dev/null || fail "'curl' is required for health checks."

[ ! -f "gzmo.toml" ] && fail "gzmo.toml not found. Run 'gzmo init' first."
[ ! -d "models" ] && fail "models/ directory is missing."
[ ! -d "memory" ] && fail "memory/ directory is missing."

# Fallback to local development binary if bin/gzmo doesn't exist
GZMO_BIN="./bin/gzmo-static"
if [ ! -f "$GZMO_BIN" ]; then
    if [ -f "./target/x86_64-unknown-linux-musl/release/gzmo" ]; then
        GZMO_BIN="./target/x86_64-unknown-linux-musl/release/gzmo"
    elif [ -f "./target/release/gzmo" ]; then
        GZMO_BIN="./target/release/gzmo"
    elif [ -f "./target/debug/gzmo" ]; then
        GZMO_BIN="./target/debug/gzmo"
    else
        fail "Could not locate compiled gzmo-static binary."
    fi
fi

# Fallback to a system-installed llama-server if USB doesn't have a static one
log "Auditing localized inference binaries..."
export LD_LIBRARY_PATH="./bin:${LD_LIBRARY_PATH:-}"

LLAMA_SERVER_CUDA="./bin/llama-server-cuda"
LLAMA_SERVER_CPU="./bin/llama-server-cpu"
LLAMA_SERVER=""

if [ -f "$LLAMA_SERVER_CUDA" ] && $LLAMA_SERVER_CUDA --help >/dev/null 2>&1; then
    LLAMA_SERVER="$LLAMA_SERVER_CUDA"
    log "CUDA Inference binary test passed. Hardware binding compatible."
elif [ -f "$LLAMA_SERVER_CPU" ]; then
    warn "CUDA binary test failed. Hardware incompatible with CUDA. Falling back to universal CPU binary."
    # Force GPU layers to 0 since we're using the CPU binary
    PHANTOM_NGL=0
    LLAMA_SERVER="$LLAMA_SERVER_CPU"
else
    # Extreme fallback
    if command -v llama-server >/dev/null; then
        LLAMA_SERVER="llama-server"
    else
        fail "No viable llama-server executable found (CUDA or CPU)."
    fi
fi

# ─── 3. HARDWARE RECON & EGPU CHECK ───────────────────────────────────────────
log "Profiling host hardware for deterministic payload assignment..."

PHANTOM_NGL=0
FREE_VRAM_MB=0
FREE_SYSRAM_MB=0

# Safely extract System RAM capacity
if [ -f "/proc/meminfo" ]; then
    FREE_SYSRAM_KB=$(grep MemAvailable /proc/meminfo | awk '{print $2}')
    FREE_SYSRAM_MB=$((FREE_SYSRAM_KB / 1024))
fi

if command -v nvidia-smi >/dev/null; then
    VRAM=$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits | sort -nr | head -n 1)
    if [ -n "$VRAM" ]; then
        FREE_VRAM_MB="$VRAM"
    fi
fi

# Deterministic Hardware-Adaptive Model Selection Ladder
if [ "$FREE_VRAM_MB" -gt 22000 ]; then
    PHANTOM_NGL=99
    TARGET_MODEL="./models/qwen3.5-35b-a3b.Q4_K_M.gguf"
    ok "Enthusiast GPU detected (${FREE_VRAM_MB}MB free). Targeting Qwen 35B Multi-Node."
elif [ "$FREE_VRAM_MB" -gt 16000 ]; then
    PHANTOM_NGL=99
    TARGET_MODEL="./models/qwen2.5-7b-instruct.Q3_K_M.gguf"
    ok "High-tier GPU detected (${FREE_VRAM_MB}MB free). Targeting Qwen 7B."
elif [ "$FREE_VRAM_MB" -gt 6000 ]; then
    PHANTOM_NGL=99
    TARGET_MODEL="./models/gemma-4-E4B-it-Q4_K_M.gguf"
    ok "Mid-tier GPU detected (${FREE_VRAM_MB}MB free). Targeting Gemma 4B."
elif [ "$FREE_VRAM_MB" -gt 4000 ]; then
    PHANTOM_NGL=99
    TARGET_MODEL="./models/nemotron-3-nano-4b.Q4_K_M.gguf"
    ok "Entry GPU detected (${FREE_VRAM_MB}MB free). Targeting Nemotron 4B."
elif [ "$FREE_SYSRAM_MB" -gt 8000 ]; then
    TARGET_MODEL="./models/ggml-model-i2_s.gguf"
    ok "Standard CPU node detected (${FREE_SYSRAM_MB}MB free RAM). Targeting BitNet 1.58 Ternary."
else
    # Lowest possible overhead model
    TARGET_MODEL="./models/qwen2.5-0.5b-instruct.Q8_0.gguf"
    warn "Restricted node detected. Targeting Minimal Draft Engine."
fi

# Override target model if we fell back to CPU engine but hardware thought it had a GPU
if [ "$LLAMA_SERVER" = "$LLAMA_SERVER_CPU" ]; then
    # Must force a CPU-friendly model if we were aiming for Qwen 7B but have no CUDA engine
    TARGET_MODEL="./models/ggml-model-i2_s.gguf"
    warn "Inference engine is CPU-limited. Downgrading payload to Universal CPU Ternary mode."
fi

# Fallback mechanism if the dynamically chosen file isn't physically on the drive
if [ -f "$TARGET_MODEL" ]; then
    MODEL_FILE="$TARGET_MODEL"
else
    warn "$TARGET_MODEL not found. Falling back to emergency model selection."
    MODEL_FILE=$(find ./models -maxdepth 1 -name "*.gguf" | head -n 1)
    [ -z "$MODEL_FILE" ] && fail "No .gguf model found in models/ directory."
fi

ok "Dependencies verified. Locked Payload: $MODEL_FILE"

# ─── 4. PHYSICAL EXTRACTION WATCHDOG ──────────────────────────────────────────
USB_MOUNT_PATH="$(pwd)"
log "Arming mountpoint watchdog on: $USB_MOUNT_PATH"

watchdog_monitor() {
    while true; do
        # findmnt quietly evaluates kernel mount table. 
        # -T resolves the underlying mountpoint (works for both local tests and USBs)
        if ! findmnt -T "$USB_MOUNT_PATH" >/dev/null; then
            echo ""
            echo "[!] EMERGENCY: Physical USB extraction detected by the kernel!"
            # Force script termination, directly triggering trap_cleanup
            kill -TERM "${PHANTOM_PGID}"
            break
        fi
        sleep 3
    done
}
watchdog_monitor &

# ─── 5. IGNITION ──────────────────────────────────────────────────────────────
log "Auditing port 1234 for pre-existing Host nodes..."

# Check if an engine like LM Studio or another llama-server is already running
if curl -s http://127.0.0.1:1234/health >/dev/null || curl -s http://127.0.0.1:1234/v1/models >/dev/null; then
    ok "Host inference engine already operational on port 1234. Bypassing payload ignition."
else
    log "Igniting Phantom LLM Engine ($MODEL_FILE)..."
    # Apply RAM limits to prevent host freezing: --no-mmap avoids swap death on massive files
    $LLAMA_SERVER \
        -m "$MODEL_FILE" \
        -c 4096 \
        --cache-type-k q8_0 \
        --cache-type-v q8_0 \
        -ngl "$PHANTOM_NGL" \
        --host 127.0.0.1 \
        --port 1234 > /dev/null 2>&1 &

    log "Waiting for inference engine to surface..."
    # Give it 30 seconds to boot up
    MAX_WAIT=30
    for ((i=1; i<=MAX_WAIT; i++)); do
        if curl -s http://127.0.0.1:1234/health >/dev/null; then
            break
        fi
        sleep 1
        if [ "$i" -eq "$MAX_WAIT" ]; then
            fail "LLM Engine failed to respond on port 1234."
        fi
    done
    ok "Phantom internal inference engine operational."
fi

log "Spawning GZMO Sovereign Daemon..."
$GZMO_BIN daemon &

ok "Phantom Drive deployment absolute. Operating in stealth."

# ─── 6. PROCESS SYNCHRONIZATION ───────────────────────────────────────────────
# Block main script. If ANY subprocess crashes, wait exits, triggering trap_cleanup
wait -n

fail "A critical sub-process terminated unexpectedly. Initiating teardown."
