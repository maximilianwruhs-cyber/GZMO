#!/usr/bin/env bash
# pre-ingest-gate.sh — Validate files before ingestion into GZMO knowledge base.
#
# Usage:
#   pre-ingest-gate.sh <file|directory> [--dry-run] [--quarantine] [--manifest PATH]
#
# Stages:
#   1. Static checks: file size, extension allowlist, binary rejection
#   2. Optional: gzmo ingest --dry-run (if --dry-run flag and binary available)
#
# Exit codes:
#   0 — all files passed
#   1 — one or more files quarantined
#   2 — error (missing args, bad paths)
#
# Quarantine: files failing checks are moved to data/quarantine/ when
# --quarantine flag is passed; otherwise only reported.

set -eo pipefail
export LC_ALL=C

# ─── Configuration ───────────────────────────────────────────────────────
MAX_SIZE_BYTES="${PRE_INGEST_MAX_SIZE:-5242880}"  # 5 MB default
MIN_SIZE_BYTES="${PRE_INGEST_MIN_SIZE:-10}"        # reject empty/stub files

# Extensions allowed for ingestion (lowercase, without dot)
ALLOWED_EXTENSIONS=(
  md txt yaml yml json toml
  py rs sh bash
  html htm xml csv tsv
  pdf          # PDF support via gzmo text extraction
)

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." >/dev/null 2>&1 && pwd)"
QUARANTINE_DIR="$REPO_ROOT/data/quarantine"
GZMO_BIN="$REPO_ROOT/target/release/gzmo"

# ─── Flags ───────────────────────────────────────────────────────────────
DRY_RUN=false
DO_QUARANTINE=false
VERBOSE=false
MANIFEST=""
STAGE2_LIMIT="${PRE_INGEST_STAGE2_LIMIT:-0}"
TARGET=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)     DRY_RUN=true; shift ;;
    --quarantine)  DO_QUARANTINE=true; shift ;;
    --verbose|-v)  VERBOSE=true; shift ;;
    --manifest)
      shift
      MANIFEST="${1:?--manifest requires path}"
      shift
      ;;
    --stage2-limit)
      shift
      STAGE2_LIMIT="${1:?--stage2-limit requires number}"
      shift
      ;;
    --help|-h)
      echo "Usage: pre-ingest-gate.sh <file|directory> [--dry-run] [--quarantine] [--verbose] [--manifest PATH]"
      echo ""
      echo "Validates files against size, extension, and binary checks before ingestion."
      echo "  --dry-run         Run gzmo ingest --dry-run (Stage 2; use --stage2-limit N on dirs)"
      echo "  --stage2-limit N  Cap Stage 2 dry-runs (default: all passed files)"
      echo "  --manifest PATH   Write Stage-1 passed file paths (one per line)"
      echo "  --quarantine      Move failing files to data/quarantine/"
      echo "  --verbose         Print per-file check details"
      echo ""
      echo "Exit: 0 = all pass, 1 = quarantined, 2 = error"
      exit 0
      ;;
    *)
      if [[ -z "$TARGET" ]]; then
        TARGET="$1"
        shift
      else
        echo "Error: unexpected argument: $1" >&2
        exit 2
      fi
      ;;
  esac
done

if [[ -z "$TARGET" ]]; then
  echo "Error: no file or directory specified." >&2
  echo "Usage: pre-ingest-gate.sh <file|directory> [--dry-run] [--quarantine]" >&2
  exit 2
fi

if [[ ! -e "$TARGET" ]]; then
  echo "Error: target does not exist: $TARGET" >&2
  exit 2
fi

# ─── Helpers ─────────────────────────────────────────────────────────────
log_info()  { echo "  ✓ $*"; }
log_warn()  { echo "  ✗ $*"; }
log_debug() { $VERBOSE && echo "    · $*" || true; }

is_binary() {
  local f="$1"
  # Use file(1) to detect binary; also check for NUL bytes
  if file --mime-encoding "$f" 2>/dev/null | grep -qi 'binary'; then
    return 0
  fi
  # Fallback: check first 8KB for NUL bytes
  if head -c 8192 "$f" | grep -qP '\x00'; then
    return 0
  fi
  return 1
}

ext_allowed() {
  local f="$1"
  local ext="${f##*.}"
  ext="${ext,,}"  # lowercase
  for a in "${ALLOWED_EXTENSIONS[@]}"; do
    [[ "$ext" == "$a" ]] && return 0
  done
  return 1
}

quarantine_file() {
  local f="$1"
  local reason="$2"
  if $DO_QUARANTINE; then
    mkdir -p "$QUARANTINE_DIR"
    local basename
    basename="$(basename "$f")"
    local dest="$QUARANTINE_DIR/${basename}"
    # Avoid overwriting existing quarantined files
    if [[ -f "$dest" ]]; then
      dest="${QUARANTINE_DIR}/$(date +%s)_${basename}"
    fi
    mv "$f" "$dest"
    echo "  → Quarantined: $dest ($reason)"
  else
    echo "  → Would quarantine: $f ($reason)"
  fi
}

# ─── Collect files ───────────────────────────────────────────────────────
FILES=()
if [[ -f "$TARGET" ]]; then
  FILES=("$TARGET")
elif [[ -d "$TARGET" ]]; then
  while IFS= read -r -d '' f; do
    FILES+=("$f")
  done < <(find "$TARGET" -type f -print0 | sort -z)
else
  echo "Error: target is neither a file nor a directory: $TARGET" >&2
  exit 2
fi

if [[ ${#FILES[@]} -eq 0 ]]; then
  echo "No files found in: $TARGET"
  exit 0
fi

# ─── Stage 1: Static checks ─────────────────────────────────────────────
echo "═══════════════════════════════════════════════════════════════════"
echo "  PRE-INGEST GATE — Stage 1: Static Validation"
echo "═══════════════════════════════════════════════════════════════════"
echo "Target: $TARGET"
echo "Files to check: ${#FILES[@]}"
echo "Max size: $((MAX_SIZE_BYTES / 1024))KB | Min size: ${MIN_SIZE_BYTES}B"
echo "Quarantine mode: $DO_QUARANTINE"
echo "───────────────────────────────────────────────────────────────────"

PASSED=()
FAILED=()
FAILED_REASONS=()

for f in "${FILES[@]}"; do
  fname="$(basename "$f")"
  fsize=$(stat -c%s "$f" 2>/dev/null || echo 0)

  log_debug "Checking: $fname ($fsize bytes)"

  # Check 1: minimum size
  if [[ "$fsize" -lt "$MIN_SIZE_BYTES" ]]; then
    log_warn "$fname — too small (${fsize}B < ${MIN_SIZE_BYTES}B)"
    FAILED+=("$f")
    FAILED_REASONS+=("too_small")
    quarantine_file "$f" "too_small"
    continue
  fi

  # Check 2: maximum size
  if [[ "$fsize" -gt "$MAX_SIZE_BYTES" ]]; then
    log_warn "$fname — too large (${fsize}B > ${MAX_SIZE_BYTES}B = $((MAX_SIZE_BYTES / 1024))KB)"
    FAILED+=("$f")
    FAILED_REASONS+=("too_large")
    quarantine_file "$f" "too_large"
    continue
  fi

  # Check 3: extension allowlist
  if ! ext_allowed "$fname"; then
    local_ext="${fname##*.}"
    log_warn "$fname — extension not allowed (.${local_ext,,})"
    FAILED+=("$f")
    FAILED_REASONS+=("bad_extension")
    quarantine_file "$f" "bad_extension"
    continue
  fi

  # Check 4: binary rejection (except PDF which has its own extractor)
  local_ext="${fname##*.}"
  local_ext="${local_ext,,}"
  if [[ "$local_ext" != "pdf" ]] && is_binary "$f"; then
    log_warn "$fname — binary file detected"
    FAILED+=("$f")
    FAILED_REASONS+=("binary")
    quarantine_file "$f" "binary"
    continue
  fi

  log_info "$fname — passed (${fsize}B)"
  PASSED+=("$f")
done

echo "───────────────────────────────────────────────────────────────────"
echo "Stage 1 results: ${#PASSED[@]} passed, ${#FAILED[@]} rejected"

# ─── Stage 2: Optional dry-run ──────────────────────────────────────────
STAGE2_PASS=0
STAGE2_FAIL=0

if $DRY_RUN && [[ ${#PASSED[@]} -gt 0 ]]; then
  echo ""
  echo "═══════════════════════════════════════════════════════════════════"
  echo "  PRE-INGEST GATE — Stage 2: Dry-Run Extraction"
  echo "═══════════════════════════════════════════════════════════════════"

  if [[ ! -x "$GZMO_BIN" ]]; then
    echo "  ⚠ gzmo binary not found at $GZMO_BIN — skipping Stage 2"
    echo "  Hint: cargo build --release -p gzmo-cli"
  else
    n=0
    for f in "${PASSED[@]}"; do
      if [[ "$STAGE2_LIMIT" -gt 0 && "$n" -ge "$STAGE2_LIMIT" ]]; then
        break
      fi
      fname="$(basename "$f")"
      echo -n "  Dry-run: $fname ... "
      n=$((n + 1))

      if RUST_LOG=error "$GZMO_BIN" ingest --dry-run "$f" >/dev/null 2>&1; then
        echo "PASS"
        STAGE2_PASS=$((STAGE2_PASS + 1))
      else
        echo "FAIL"
        STAGE2_FAIL=$((STAGE2_FAIL + 1))
        quarantine_file "$f" "dry_run_fail"
      fi
    done
    echo "───────────────────────────────────────────────────────────────────"
    echo "Stage 2 results: $STAGE2_PASS passed, $STAGE2_FAIL failed"
  fi
fi

# ─── Summary ─────────────────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════════════════════════════"
TOTAL_FAIL=$((${#FAILED[@]} + STAGE2_FAIL))
TOTAL_PASS=$((${#PASSED[@]} - STAGE2_FAIL))

if [[ -n "$MANIFEST" ]]; then
  mkdir -p "$(dirname "$MANIFEST")"
  : >"$MANIFEST"
  for f in "${PASSED[@]}"; do
    echo "$f" >>"$MANIFEST"
  done
  echo "Manifest: $MANIFEST (${#PASSED[@]} paths)"
fi

if [[ "$TOTAL_FAIL" -eq 0 ]]; then
  echo "  PRE-INGEST GATE: PASS ($TOTAL_PASS/${#FILES[@]} files ready)"
  echo "═══════════════════════════════════════════════════════════════════"
  exit 0
else
  echo "  PRE-INGEST GATE: QUARANTINE ($TOTAL_FAIL/${#FILES[@]} files rejected)"
  echo "═══════════════════════════════════════════════════════════════════"
  exit 1
fi
