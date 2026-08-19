#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ROTATE="$REPO_ROOT/scripts/inbox-rotate.sh"

TMPDIR_TEST="$(mktemp -d)"
trap 'rm -rf "$TMPDIR_TEST"' EXIT

touch "$TMPDIR_TEST/drop-old1.md"
touch "$TMPDIR_TEST/drop-old2.md"
touch "$TMPDIR_TEST/drop-fresh.md"
touch -d "60 days ago" "$TMPDIR_TEST/drop-old1.md"
touch -d "60 days ago" "$TMPDIR_TEST/drop-old2.md"

# --- dry-run test ---
out=$(bash "$ROTATE" --dry-run --older-than 30 "$TMPDIR_TEST")
rc=$?
if [[ $rc -ne 0 ]]; then
  echo "[inbox-rotate-test] FAIL: dry-run exit code $rc"; exit 1
fi
if ! echo "$out" | grep -q "would move 2"; then
  echo "[inbox-rotate-test] FAIL: dry-run did not report 'would move 2'"; exit 1
fi
remaining=$(find "$TMPDIR_TEST" -maxdepth 1 -name 'drop-*.md' | wc -l)
if [[ "$remaining" -ne 3 ]]; then
  echo "[inbox-rotate-test] FAIL: dry-run moved files (found $remaining, expected 3)"; exit 1
fi
if [[ -d "$TMPDIR_TEST/processed" ]]; then
  echo "[inbox-rotate-test] FAIL: dry-run created processed/"; exit 1
fi

# --- apply test ---
out=$(bash "$ROTATE" --apply --older-than 30 "$TMPDIR_TEST")
rc=$?
if [[ $rc -ne 0 ]]; then
  echo "[inbox-rotate-test] FAIL: apply exit code $rc"; exit 1
fi
if ! echo "$out" | grep -q "moved 2"; then
  echo "[inbox-rotate-test] FAIL: apply did not report 'moved 2'"; exit 1
fi
remaining=$(find "$TMPDIR_TEST" -maxdepth 1 -name 'drop-*.md' | wc -l)
if [[ "$remaining" -ne 1 ]]; then
  echo "[inbox-rotate-test] FAIL: expected 1 remaining drop, found $remaining"; exit 1
fi
total=$(find "$TMPDIR_TEST" -name 'drop-*.md' | wc -l)
if [[ "$total" -ne 3 ]]; then
  echo "[inbox-rotate-test] FAIL: files deleted! total=$total, expected 3"; exit 1
fi

echo "[inbox-rotate-test] PASS"
