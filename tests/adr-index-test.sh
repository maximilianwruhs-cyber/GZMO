#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INDEX="$ROOT/docs/ADR-INDEX.md"

if ! command -v python3 >/dev/null 2>&1; then
  echo "FAIL: python3 is required for tests/adr-index-test.sh" >&2
  exit 1
fi

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

for entry in "$ROOT/AGENTS.md" "$ROOT/MACHINE.md" "$ROOT/README.md" "$ROOT/docs/SPINE_FOCUS.md"; do
  grep -q 'ADR-INDEX.md' "$entry" || {
    echo "FAIL missing ADR-INDEX pointer in ${entry#$ROOT/}"; exit 1;
  }
  grep -q 'ADR-0011' "$entry" || {
    echo "FAIL missing ADR-0011 pointer in ${entry#$ROOT/}"; exit 1;
  }
done

# Metadata header lines must not collapse into one Markdown paragraph.
# Require bullet-list form: each status/lineage label is its own list item.
for n in 0003 0004 0005 0006 0007 0008 0009 0010 0011 0012 0013 0014; do
  file=("$ROOT"/docs/ADR-${n}-*.md)
  [[ -f "${file[0]}" ]] || { echo "FAIL missing ADR-$n"; exit 1; }
  python3 - "$file" <<'PY' || { echo "FAIL ADR header meta not separated: ${file[0]##*/}"; exit 1; }
import re, sys
from pathlib import Path
p = Path(sys.argv[1])
lines = p.read_text(encoding="utf-8").splitlines()
# header block: after title until first ## or ---
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
for s in meta:
    m = bold.match(s)
    if not m:
        continue
    if s.lstrip().startswith("- "):
        continue
    raise SystemExit(1)
for a, b in zip(meta, meta[1:]):
    if re.match(r"^\*\*[^*]+:\*\*", a) and re.match(r"^\*\*[^*]+:\*\*", b):
        raise SystemExit(1)
raise SystemExit(0)
PY
  if grep -q 'Decision status:' "${file[0]}"; then
    grep -qE '^- \*\*Decision status:\*\*' "${file[0]}" || {
      echo "FAIL ADR-$n Decision status not bullet-separated"; exit 1;
    }
  fi
done

# Positive path through the full checker.
bash "$ROOT/scripts/adr-check.sh" >/dev/null

# Negative fixtures: index status drift and dangling forward links must fail.
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
cp -a "$ROOT/docs" "$TMP/docs"
mkdir -p "$TMP"/{scripts,data-next}
cp "$ROOT/scripts/adr-check.sh" "$TMP/scripts/adr-check.sh"
# Minimal entry docs so entry-pointer checks can pass while we break index/links.
for f in AGENTS.md MACHINE.md README.md; do
  cat >"$TMP/$f" <<EOF
# fixture
**Architecture authority:** docs/ADR-INDEX.md ADR-0011
EOF
done
mkdir -p "$TMP/docs"
cat >"$TMP/docs/SPINE_FOCUS.md" <<EOF
# fixture
**Architecture authority:** ADR-INDEX.md ADR-0011
EOF

# 1) Decision status drift between index row and ADR file.
python3 - "$TMP/docs/ADR-INDEX.md" <<'PY'
from pathlib import Path
import re, sys
p = Path(sys.argv[1])
text = p.read_text(encoding="utf-8")
# Flip ADR-0006 Accepted -> Proposed in the table only.
text2, n = re.subn(
    r"(\|\s*0006\s*\|[^|]+\|\s*)Accepted(\s*\|)",
    r"\1Proposed\2",
    text,
    count=1,
)
if n != 1:
    raise SystemExit(f"failed to mutate 0006 decision cell (n={n})")
p.write_text(text2, encoding="utf-8")
PY
if bash "$TMP/scripts/adr-check.sh" >/dev/null 2>"$TMP/drift.err"; then
  echo "FAIL expected index status drift to fail adr-check"; exit 1
fi
grep -q 'index-status-consistency' "$TMP/data-next/adr-check/latest.json" || {
  echo "FAIL drift fixture missing index-status-consistency failure"; exit 1
}

# Restore docs for next negative case.
rm -rf "$TMP/docs"
cp -a "$ROOT/docs" "$TMP/docs"

# 2) Dangling Superseded-by forward link.
python3 - "$TMP/docs" <<'PY'
from pathlib import Path
import re, sys
docs = Path(sys.argv[1])
# Point ADR-0003 supersession at a nonexistent file via index table cell.
index = docs / "ADR-INDEX.md"
text = index.read_text(encoding="utf-8")
text2, n = re.subn(
    r"(\|\s*0003\s*\|[^|]+\|[^|]+\|[^|]+\|\s*)\[[^\]]+\]\([^)]+\)",
    r"\1[ADR-0099](./ADR-0099-does-not-exist.md)",
    text,
    count=1,
)
if n != 1:
    raise SystemExit(f"failed to mutate 0003 superseded link (n={n})")
index.write_text(text2, encoding="utf-8")
PY
if bash "$TMP/scripts/adr-check.sh" >/dev/null 2>"$TMP/dangling.err"; then
  echo "FAIL expected dangling forward link to fail adr-check"; exit 1
fi
grep -Eq 'lineage-link-targets|missing targets' "$TMP/data-next/adr-check/latest.json" || {
  echo "FAIL dangling fixture missing lineage-link-targets failure"; exit 1
}

echo "PASS ADR index"
