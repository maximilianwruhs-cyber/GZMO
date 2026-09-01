#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INDEX="$ROOT/docs/ADR-INDEX.md"

[[ -f "$INDEX" ]] || { echo "FAIL missing ADR index"; exit 1; }
for n in 0011 0012 0013 0014; do
  file=("$ROOT"/docs/ADR-${n}-*.md)
  [[ -f "${file[0]}" ]] || { echo "FAIL missing ADR-$n"; exit 1; }
  grep -q '^\*\*Decision status:\*\* Accepted' "${file[0]}" || {
    echo "FAIL ADR-$n not Accepted"; exit 1;
  }
done

grep -q 'ADR-0001/0002 were never issued in GZMO' "$INDEX" || {
  echo "FAIL missing 0001/0002 provenance"; exit 1;
}
if grep -RIl 'little-tools-lab/docs/adr/000[12]-' \
  "$ROOT/AGENTS.md" "$ROOT/MACHINE.md" "$ROOT/README.md" "$ROOT/docs/SPINE_FOCUS.md"; then
  echo "FAIL active authority depends on inaccessible LTL ADR"; exit 1
fi

echo "PASS ADR index"
