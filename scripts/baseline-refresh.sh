#!/usr/bin/env bash
# Regenerate the measured-metrics block in BASELINE.md.
#
#   scripts/baseline-refresh.sh                  rewrite BASELINE.md in place
#   scripts/baseline-refresh.sh --check          exit 1 if BASELINE.md is stale
#   scripts/baseline-refresh.sh --test-log FILE  reuse existing `cargo test` output
#
# Every number in BASELINE.md between the GENERATED markers comes from here.
# CI runs the --check form, so the document cannot drift away from the code
# the way it did between 2026-07-09 and 2026-08-23 (it claimed 133 tests when
# the workspace actually had 384, and 15 warnings when clippy runs -D warnings).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BASELINE="BASELINE.md"
BEGIN_MARK='<!-- BEGIN GENERATED: scripts/baseline-refresh.sh -->'
END_MARK='<!-- END GENERATED -->'

check=0
test_log=""
while [ $# -gt 0 ]; do
  case "$1" in
    --check) check=1 ;;
    --test-log) test_log="${2:?--test-log requires a path}"; shift ;;
    -h|--help) sed -n '2,10p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "baseline-refresh: unknown argument '$1'" >&2; exit 2 ;;
  esac
  shift
done

# ── workspace shape ────────────────────────────────────────────────────────
# `tr -d '\r'` is load-bearing. On a CRLF checkout the trailing carriage return
# lands on the LAST member, so `git ls-files -- crates/eml-core<CR>` matches
# nothing and a whole crate silently vanishes from every metric below.
members=$(sed -n 's/^members = \[\(.*\)\]/\1/p' Cargo.toml | tr -d '"\r ' | tr ',' ' ')
[ -n "$members" ] || { echo "baseline-refresh: could not parse workspace members" >&2; exit 1; }

# Fail loudly rather than under-report: every declared member must resolve to
# at least one tracked file.
for m in $members; do
  if [ -z "$(git ls-files -- "$m" | head -1)" ]; then
    echo "baseline-refresh: workspace member '$m' matches no tracked files" >&2
    exit 1
  fi
done

rs_files=$(git ls-files -- $members | grep '\.rs$' || true)
rs_count=$(printf '%s\n' "$rs_files" | grep -c . || true)
rs_loc=$(printf '%s\n' "$rs_files" | tr '\n' '\0' | xargs -0 cat | wc -l)
crate_count=$(printf '%s\n' $members | grep -c . )
tracked=$(git ls-files | wc -l)
deps=$(grep -c '^name = ' Cargo.lock)

# ── test results ───────────────────────────────────────────────────────────
# Only temp files this script created are cleaned up; a caller-supplied
# --test-log belongs to the caller and is left alone.
owned_test_log=""
rendered=""
cleanup() { rm -f ${owned_test_log:+"$owned_test_log"} ${rendered:+"$rendered"}; }
trap cleanup EXIT

if [ -z "$test_log" ]; then
  test_log="$(mktemp)"
  owned_test_log="$test_log"
  cargo test --all >"$test_log" 2>&1 || true
fi

sum_field() {
  grep -o "[0-9]\+ $1" "$test_log" | awk '{s+=$1} END {print s+0}'
}
t_pass=$(sum_field passed)
t_fail=$(sum_field failed)
t_ignored=$(sum_field ignored)

# ── largest modules ────────────────────────────────────────────────────────
largest=$(printf '%s\n' "$rs_files" | tr '\n' '\0' | xargs -0 wc -l \
          | grep -v ' total$' | sort -rn | head -10 \
          | awk '{printf "| `%s` | %s |\n", $2, $1}')

# NOTE: deliberately no commit hash or timestamp here. A commit cannot contain
# its own hash, so embedding one would make BASELINE.md stale the instant it is
# committed and the CI gate would fail on every push. Use `git log` for dates.

# ── render ─────────────────────────────────────────────────────────────────
block=$(cat <<EOF
$BEGIN_MARK
<!-- Do not edit by hand. Run: scripts/baseline-refresh.sh -->

| Metric | Value |
|---|---|
| Workspace crates | $crate_count |
| Rust source files | $rs_count |
| Rust lines | $rs_loc |
| Tracked files | $tracked |
| Resolved dependencies | $deps |
| Tests passed | $t_pass |
| Tests failed | $t_fail |
| Tests ignored | $t_ignored |

Warnings are not counted: CI runs \`cargo clippy --all-targets -- -D warnings\`,
so a warning is a build failure rather than a number worth tracking.

**Largest modules**

| File | Lines |
|---|---|
$largest
$END_MARK
EOF
)

rendered="$(mktemp)"
awk -v block="$block" -v b="$BEGIN_MARK" -v e="$END_MARK" '
  index($0, b) == 1 { print block; skip = 1; next }
  index($0, e) == 1 { skip = 0; next }
  !skip { print }
' "$BASELINE" >"$rendered"

if ! grep -qF "$BEGIN_MARK" "$BASELINE"; then
  echo "baseline-refresh: $BASELINE is missing the GENERATED markers" >&2
  exit 1
fi

if [ "$check" -eq 1 ]; then
  if diff -u "$BASELINE" "$rendered" >/dev/null; then
    echo "baseline-refresh: BASELINE.md is current"
  else
    echo "baseline-refresh: BASELINE.md is stale. Run scripts/baseline-refresh.sh and commit." >&2
    diff -u "$BASELINE" "$rendered" >&2 || true
    exit 1
  fi
else
  cp "$rendered" "$BASELINE"
  echo "baseline-refresh: wrote $BASELINE ($t_pass passed, $t_fail failed, $t_ignored ignored)"
fi
