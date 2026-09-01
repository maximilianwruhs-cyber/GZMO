#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INDEX="$ROOT/docs/ADR-INDEX.md"

[[ -f "$INDEX" ]] || { echo "FAIL missing ADR index"; exit 1; }
for n in 0011 0012 0013 0014; do
  file=("$ROOT"/docs/ADR-${n}-*.md)
  [[ -f "${file[0]}" ]] || { echo "FAIL missing ADR-$n"; exit 1; }
  grep -qE '^- \*\*Decision status:\*\* Accepted' "${file[0]}" || {
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

# Metadata header lines must not collapse into one Markdown paragraph.
# Require bullet-list form: each status/lineage label is its own list item.
for n in 0003 0004 0005 0006 0007 0008 0009 0010 0011 0012 0013 0014; do
  file=("$ROOT"/docs/ADR-${n}-*.md)
  [[ -f "${file[0]}" ]] || { echo "FAIL missing ADR-$n"; exit 1; }
  # Adjacent bold meta labels without bullet/hard-break separation (run-on paragraph).
  if grep -nE '^\*\*[^*]+:\*\*' "${file[0]}" | head -1 >/dev/null; then
    # bare **Label:** at column 0 is only OK if previous meta used two-space hard break;
    # prefer bullets — fail any column-0 bold meta in the header block before first ##.
    python3 - "$file" <<'PY' || { echo "FAIL ADR header meta not separated: ${file[0]##*/}"; exit 1; }
import re, sys
from pathlib import Path
p = Path(sys.argv[1])
lines = p.read_text(encoding="utf-8").splitlines()
# header block: after title until first ## or ---
i = 0
if not lines or not lines[0].startswith("# "):
    raise SystemExit(1)
i = 1
while i < len(lines) and lines[i].strip() == "":
    i += 1
meta = []
while i < len(lines):
    s = lines[i]
    if s.strip() == "" or s.startswith("##") or s.strip() == "---":
        break
    meta.append(s)
    i += 1
if len(meta) < 2:
    raise SystemExit(1)
bold = re.compile(r"^(\s*)(\*\*[^*]+:\*\*)")
for idx, s in enumerate(meta):
    m = bold.match(s)
    if not m:
        continue
    # Must be a list item, or (legacy) end previous line with two trailing spaces
    if s.lstrip().startswith("- "):
        continue
    # hard-break form: this line ends with two spaces AND is not first meta? still weak on GH for first
    # Reject bare bold meta lines entirely — bullets required.
    raise SystemExit(1)
# Also reject two consecutive non-empty non-bullet lines that both look like bold labels
for a, b in zip(meta, meta[1:]):
    if re.match(r"^\*\*[^*]+:\*\*", a) and re.match(r"^\*\*[^*]+:\*\*", b):
        raise SystemExit(1)
raise SystemExit(0)
PY
  fi
  # Positive: Decision status appears as a bullet list item when present
  if grep -q 'Decision status:' "${file[0]}"; then
    grep -qE '^- \*\*Decision status:\*\*' "${file[0]}" || {
      echo "FAIL ADR-$n Decision status not bullet-separated"; exit 1;
    }
  fi
done

echo "PASS ADR index"
