#!/usr/bin/env bash
# Verify that every relative Markdown link in tracked .md files resolves.
#
#   scripts/check-doc-links.sh              fail on any link not in the baseline
#   scripts/check-doc-links.sh --list       print every broken link, exit 0
#   scripts/check-doc-links.sh --write-baseline   record current breakage
#
# This repository inherited 152 broken relative links, most of them pointing
# into sibling repositories that only exist in the author's multi-repo
# workspace. Fixing them all is a separate project, so they are recorded in
# docs/link-baseline.txt and CI fails only on links that are *newly* broken.
# Shrink the baseline over time; never grow it.
#
# Only relative targets are checked. External URLs, mailto:, and pure
# "#anchor" fragments are out of scope; so are links inside fenced code
# blocks, which are usually illustrative rather than real paths.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BASELINE_FILE="docs/link-baseline.txt"
mode="check"
case "${1:-}" in
  --list) mode="list" ;;
  --write-baseline) mode="write" ;;
  "") ;;
  *) echo "check-doc-links: unknown argument '$1'" >&2; exit 2 ;;
esac

broken=0
report=""

while IFS= read -r file; do
  dir="$(dirname "$file")"
  # Strip fenced code blocks, then pull the target out of every ]( ... ) link.
  awk '
    /^[[:space:]]*```/ { fence = !fence; next }
    !fence { print }
  ' "$file" \
  | grep -o ']([^)]*)' \
  | sed 's/^](//; s/)$//' \
  | while IFS= read -r target; do
      [ -n "$target" ] || continue
      case "$target" in
        http://*|https://*|mailto:*|\#*|'<'*) continue ;;
      esac
      # Drop any #fragment and ?query suffix.
      path="${target%%#*}"
      path="${path%%\?*}"
      [ -n "$path" ] || continue
      # Resolve relative to the containing directory.
      if [ "${path#/}" != "$path" ]; then
        resolved=".${path}"
      else
        resolved="$dir/$path"
      fi
      if [ ! -e "$resolved" ]; then
        printf '%s -> %s\n' "$file" "$target"
      fi
    done
done < <(git ls-files '*.md') > /tmp/gzmo-broken-links.$$ || true

report="$(sort -u /tmp/gzmo-broken-links.$$)"
rm -f /tmp/gzmo-broken-links.$$
broken="$(printf '%s' "$report" | grep -c . || true)"

if [ "$mode" = "list" ]; then
  echo "check-doc-links: $broken broken relative link(s)"
  [ "$broken" -gt 0 ] && printf '%s\n' "$report"
  exit 0
fi

if [ "$mode" = "write" ]; then
  mkdir -p "$(dirname "$BASELINE_FILE")"
  {
    echo "# Known-broken relative Markdown links, recorded by"
    echo "# scripts/check-doc-links.sh --write-baseline"
    echo "#"
    echo "# CI fails on any broken link NOT listed here. Entries should only"
    echo "# ever be removed. If you add one, you are committing a broken link."
    printf '%s\n' "$report"
  } >"$BASELINE_FILE"
  echo "check-doc-links: recorded $broken known-broken link(s) in $BASELINE_FILE"
  exit 0
fi

known="$(mktemp)"; current="$(mktemp)"
trap 'rm -f "$known" "$current"' EXIT
grep -v '^#' "$BASELINE_FILE" 2>/dev/null | grep . | sort -u >"$known" || true
printf '%s\n' "$report" | grep . | sort -u >"$current" || true

new="$(comm -13 "$known" "$current")"
fixed="$(comm -23 "$known" "$current")"
new_count="$(printf '%s' "$new" | grep -c . || true)"
fixed_count="$(printf '%s' "$fixed" | grep -c . || true)"

if [ "$fixed_count" -gt 0 ]; then
  echo "check-doc-links: $fixed_count baseline entry/entries now resolve."
  echo "  Shrink the baseline: scripts/check-doc-links.sh --write-baseline"
fi

if [ "$new_count" -eq 0 ]; then
  echo "check-doc-links: no new broken links ($broken known, see $BASELINE_FILE)"
  exit 0
fi

echo "check-doc-links: $new_count NEW broken relative link(s):" >&2
printf '%s\n' "$new" >&2
exit 1
