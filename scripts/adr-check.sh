#!/usr/bin/env bash
# Deterministic ADR authority / lineage checker.
#   bash scripts/adr-check.sh
# Artifact: data-next/adr-check/latest.json  (schema gzmo.adr.check/v1)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/adr-check"
DOCS="$ROOT/docs"
INDEX="$DOCS/ADR-INDEX.md"
mkdir -p "$OUT"

export ROOT DOCS INDEX OUT
python3 - <<'PY'
from __future__ import annotations

import json
import os
import re
from datetime import datetime, timezone
from pathlib import Path

root = Path(os.environ["ROOT"])
docs = Path(os.environ["DOCS"])
index = Path(os.environ["INDEX"])
out = Path(os.environ["OUT"])

DECISION_VOCAB = {"Proposed", "Accepted", "Rejected", "Superseded"}
IMPL_VOCAB = {"Not started", "In progress", "Implemented", "Retired"}
REQUIRED_HEADINGS_0011_0014 = [
    "## Context",
    "## Decision",
    "## Invariants",
    "## Consequences",
    "## Rejected alternatives",
    "## Verification",
]
FOCUS_NUMS = {"0011", "0012", "0013", "0014"}
ENTRY_DOCS = [
    root / "AGENTS.md",
    root / "MACHINE.md",
    root / "README.md",
    docs / "SPINE_FOCUS.md",
]
LTL_RE = re.compile(r"little-tools-lab/docs/adr/000[12]-")
ADR_FILE_RE = re.compile(r"^ADR-(\d{4})-.+\.md$")
SUPERSEDED_BY_LINE_RE = re.compile(
    r"^\*\*(?:Superseded by|Historical supersedes[^:]*):\*\*\s*(.*)$",
    re.MULTILINE,
)
LINK_RE = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
DECISION_STATUS_RE = re.compile(
    r"^\*\*Decision status:\*\*\s*(\w+)(?:\s|\(|$)", re.MULTILINE
)
IMPL_STATUS_RE = re.compile(
    r"^\*\*Implementation status:\*\*\s*(.+)$", re.MULTILINE
)

checks: list[dict] = []
errors: list[str] = []


def add(check_id: str, ok: bool, detail: str) -> None:
    checks.append({"id": check_id, "ok": ok, "detail": detail})
    if not ok:
        errors.append(f"{check_id}: {detail}")


# --- index present + provenance ---
if not index.is_file():
    add("index-present", False, "docs/ADR-INDEX.md missing")
    payload = {
        "schema": "gzmo.adr.check/v1",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "ok": False,
        "advice": "adr_check_fail",
        "errors": errors,
        "checks": checks,
    }
    (out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(payload, indent=2))
    raise SystemExit(1)

index_text = index.read_text(encoding="utf-8")
add(
    "index-present",
    True,
    "docs/ADR-INDEX.md",
)
add(
    "provenance-0001-0002",
    "ADR-0001/0002 were never issued in GZMO" in index_text,
    "index records GZMO never issued 0001/0002"
    if "ADR-0001/0002 were never issued in GZMO" in index_text
    else "missing 0001/0002 provenance sentence",
)
add(
    "authority-order",
    "Authority order:" in index_text and "ADR-0011 constitutional invariants" in index_text,
    "authority order names ADR-0011 first"
    if "Authority order:" in index_text and "ADR-0011 constitutional invariants" in index_text
    else "authority order incomplete",
)

# --- discover ADR files ---
adr_files: dict[str, Path] = {}
dupes: list[str] = []
for path in sorted(docs.glob("ADR-*.md")):
    m = ADR_FILE_RE.match(path.name)
    if not m:
        continue
    num = m.group(1)
    if num in adr_files:
        dupes.append(num)
    adr_files[num] = path

add(
    "unique-numbers",
    len(dupes) == 0,
    "all ADR numbers unique" if not dupes else f"duplicate ADR numbers: {sorted(set(dupes))}",
)

# Expected lineage span
for num in [f"{n:04d}" for n in range(3, 15)]:
    present = num in adr_files
    add(f"adr-{num}-present", present, str(adr_files[num].name) if present else f"missing ADR-{num}")

# --- per-ADR status vocabulary + focus headings ---
link_targets_needed: list[tuple[str, str]] = []

for num, path in sorted(adr_files.items()):
    text = path.read_text(encoding="utf-8")
    dm = DECISION_STATUS_RE.search(text)
    im = IMPL_STATUS_RE.search(text)
    if not dm:
        add(f"adr-{num}-decision-status", False, "missing Decision status line")
        decision = None
    else:
        decision = dm.group(1)
        ok = decision in DECISION_VOCAB
        add(
            f"adr-{num}-decision-status",
            ok,
            f"{decision}" if ok else f"invalid decision status {decision!r}",
        )
    if not im:
        add(f"adr-{num}-implementation-status", False, "missing Implementation status line")
    else:
        impl = im.group(1).strip()
        # Allow trailing notes after vocabulary token
        impl_token = impl
        for vocab in sorted(IMPL_VOCAB, key=len, reverse=True):
            if impl == vocab or impl.startswith(vocab + " ") or impl.startswith(vocab + ";") or impl.startswith(vocab + "—") or impl.startswith(vocab + "-"):
                impl_token = vocab
                break
        ok = impl_token in IMPL_VOCAB
        add(
            f"adr-{num}-implementation-status",
            ok,
            impl_token if ok else f"invalid implementation status {impl!r}",
        )

    if num in FOCUS_NUMS:
        if decision != "Accepted":
            add(f"adr-{num}-accepted", False, f"expected Accepted, got {decision!r}")
        else:
            add(f"adr-{num}-accepted", True, "Accepted")
        missing = [h for h in REQUIRED_HEADINGS_0011_0014 if h not in text]
        add(
            f"adr-{num}-headings",
            not missing,
            "required headings present" if not missing else f"missing headings: {missing}",
        )

    for sm in SUPERSEDED_BY_LINE_RE.finditer(text):
        body = sm.group(1).strip()
        if body in {"—", "-", "—.", "n/a", "N/A"} or body.startswith("—"):
            continue
        for href in LINK_RE.findall(body):
            if href.startswith("http://") or href.startswith("https://"):
                continue
            # strip anchors
            href_path = href.split("#", 1)[0]
            if not href_path:
                continue
            link_targets_needed.append((num, href_path))

# Index table Superseded by links
for href in LINK_RE.findall(index_text):
    if "ADR-00" in href or "ADR-001" in href:
        href_path = href.split("#", 1)[0]
        if href_path:
            link_targets_needed.append(("INDEX", href_path))

# --- superseded-by target existence ---
unresolved = []
for src, href in link_targets_needed:
    # resolve relative to docs/
    if href.startswith("./"):
        target = (docs / href[2:]).resolve()
    elif href.startswith("../"):
        target = (docs / href).resolve()
    elif not href.startswith("/") and "://" not in href:
        target = (docs / href).resolve()
    else:
        continue
    # only enforce in-repo markdown targets under docs or root-relative ADR paths
    try:
        target.relative_to(root.resolve())
    except ValueError:
        # outside repo (e.g. little-tools-lab) — provenance only; skip existence
        continue
    if "little-tools-lab" in str(target).replace("\\", "/"):
        continue
    if not target.is_file():
        unresolved.append(f"{src}→{href}")

add(
    "superseded-by-targets",
    len(unresolved) == 0,
    "all Superseded by targets exist"
    if not unresolved
    else f"missing targets: {unresolved}",
)

# ADR-0006 remains Accepted/Implemented
if "0006" in adr_files:
    t6 = adr_files["0006"].read_text(encoding="utf-8")
    d6 = DECISION_STATUS_RE.search(t6)
    i6 = IMPL_STATUS_RE.search(t6)
    ok6 = (
        d6 is not None
        and d6.group(1) == "Accepted"
        and i6 is not None
        and i6.group(1).strip().startswith("Implemented")
    )
    add(
        "adr-0006-current-runtime",
        ok6,
        "Accepted/Implemented until successor cutover"
        if ok6
        else "ADR-0006 must remain Accepted/Implemented",
    )

# --- entry docs must not depend on inaccessible LTL ADR-0001/0002 ---
ltl_hits = []
for p in ENTRY_DOCS:
    if not p.is_file():
        add(f"entry-{p.name}", False, f"missing entry doc {p}")
        continue
    text = p.read_text(encoding="utf-8")
    if LTL_RE.search(text):
        ltl_hits.append(str(p.relative_to(root)))
    add(f"entry-{p.name}-present", True, str(p.relative_to(root)))

add(
    "no-ltl-authority-in-entry-docs",
    len(ltl_hits) == 0,
    "entry docs free of inaccessible LTL ADR-0001/0002 paths"
    if not ltl_hits
    else f"LTL ADR authority refs in: {ltl_hits}",
)

ok = all(c["ok"] for c in checks)
payload = {
    "schema": "gzmo.adr.check/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": ok,
    "advice": "adr_check_ok" if ok else "adr_check_fail",
    "counts": {
        "pass": sum(1 for c in checks if c["ok"]),
        "fail": sum(1 for c in checks if not c["ok"]),
        "total": len(checks),
    },
    "errors": errors,
    "checks": checks,
    "adrs": sorted(adr_files.keys()),
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps(payload, indent=2))
raise SystemExit(0 if ok else 1)
PY
