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

if ! command -v python3 >/dev/null 2>&1; then
  echo "FAIL: python3 is required for scripts/adr-check.sh" >&2
  exit 1
fi

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
    r"^(?:- )?\*\*Superseded by:\*\*\s*(.*)$",
    re.MULTILINE,
)
HISTORICAL_SUPERSEDES_LINE_RE = re.compile(
    r"^(?:- )?\*\*Historical supersedes[^:]*:\*\*\s*(.*)$",
    re.MULTILINE,
)
SUPERSEDES_LINE_RE = re.compile(
    r"^(?:- )?\*\*Supersedes:\*\*\s*(.*)$",
    re.MULTILINE,
)
SPEC_LINE_RE = re.compile(
    r"^(?:- )?\*\*Spec:\*\*\s*(.*)$",
    re.MULTILINE,
)
LINK_RE = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
DECISION_STATUS_RE = re.compile(
    r"^(?:- )?\*\*Decision status:\*\*\s*(\w+)(?:\s|\(|$)", re.MULTILINE
)
IMPL_STATUS_RE = re.compile(
    r"^(?:- )?\*\*Implementation status:\*\*\s*(.+)$", re.MULTILINE
)
META_BOLD_RE = re.compile(r"^(?:- )?(\*\*[^*]+:\*\*)")
INDEX_ROW_RE = re.compile(
    r"^\|\s*(\d{4})\s*\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]*)\|\s*$"
)
ADR_HREF_RE = re.compile(r"(?:^|/)ADR-(\d{4})-")

checks: list[dict] = []
errors: list[str] = []


def add(check_id: str, ok: bool, detail: str) -> None:
    checks.append({"id": check_id, "ok": ok, "detail": detail})
    if not ok:
        errors.append(f"{check_id}: {detail}")


def resolve_repo_target(href: str, base: Path) -> Path | None:
    href_path = href.split("#", 1)[0]
    if not href_path:
        return None
    if href_path.startswith("http://") or href_path.startswith("https://"):
        return None
    if href_path.startswith("./"):
        target = (base / href_path[2:]).resolve()
    elif href_path.startswith("../") or (
        not href_path.startswith("/") and "://" not in href_path
    ):
        target = (base / href_path).resolve()
    else:
        return None
    try:
        target.relative_to(root.resolve())
    except ValueError:
        return None
    if "little-tools-lab" in str(target).replace("\\", "/"):
        return None
    return target


def collect_relative_links(body: str, src: str, base: Path, sink: list[tuple[str, str, Path]]) -> None:
    cleaned = body.strip()
    if cleaned in {"—", "-", "—.", "n/a", "N/A"} or cleaned.startswith("—"):
        return
    for href in LINK_RE.findall(body):
        target = resolve_repo_target(href, base)
        if target is None:
            continue
        sink.append((src, href, target))


def impl_token(raw: str) -> str:
    impl = raw.strip()
    for vocab in sorted(IMPL_VOCAB, key=len, reverse=True):
        if (
            impl == vocab
            or impl.startswith(vocab + " ")
            or impl.startswith(vocab + ";")
            or impl.startswith(vocab + "—")
            or impl.startswith(vocab + "-")
        ):
            return vocab
    return impl


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
link_targets_needed: list[tuple[str, str, Path]] = []
file_meta: dict[str, dict[str, object]] = {}

for num, path in sorted(adr_files.items()):
    text = path.read_text(encoding="utf-8")
    dm = DECISION_STATUS_RE.search(text)
    im = IMPL_STATUS_RE.search(text)
    decision = None
    impl = None
    if not dm:
        add(f"adr-{num}-decision-status", False, "missing Decision status line")
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
        impl = impl_token(im.group(1))
        ok = impl in IMPL_VOCAB
        add(
            f"adr-{num}-implementation-status",
            ok,
            impl if ok else f"invalid implementation status {im.group(1)!r}",
        )

    file_meta[num] = {
        "decision": decision,
        "implementation": impl,
        "superseded_by_targets": set(),
        "path": path,
    }

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

    # Header metadata must render as separate lines (bullet list), not one paragraph.
    header_lines = text.splitlines()
    hi = 1
    while hi < len(header_lines) and header_lines[hi].strip() == "":
        hi += 1
    meta_lines: list[str] = []
    while hi < len(header_lines):
        s = header_lines[hi]
        if s.strip() == "" or s.startswith("##") or s.strip() == "---":
            break
        meta_lines.append(s)
        hi += 1
    bare_runon = False
    for s in meta_lines:
        if META_BOLD_RE.match(s) and not s.lstrip().startswith("- "):
            bare_runon = True
            break
    for a, b in zip(meta_lines, meta_lines[1:]):
        if re.match(r"^\*\*[^*]+:\*\*", a) and re.match(r"^\*\*[^*]+:\*\*", b):
            bare_runon = True
            break
    add(
        f"adr-{num}-meta-separated",
        not bare_runon and len(meta_lines) >= 2,
        "bullet metadata header"
        if not bare_runon and len(meta_lines) >= 2
        else "status/lineage headers would collapse into one Markdown paragraph",
    )

    for sm in SUPERSEDED_BY_LINE_RE.finditer(text):
        body = sm.group(1).strip()
        collect_relative_links(body, f"{num}:Superseded-by", docs, link_targets_needed)
        for href in LINK_RE.findall(body):
            m = ADR_HREF_RE.search(href)
            if m:
                file_meta[num]["superseded_by_targets"].add(m.group(1))  # type: ignore[index]

    for sm in HISTORICAL_SUPERSEDES_LINE_RE.finditer(text):
        collect_relative_links(
            sm.group(1), f"{num}:Historical-supersedes", docs, link_targets_needed
        )

    for sm in SUPERSEDES_LINE_RE.finditer(text):
        collect_relative_links(sm.group(1), f"{num}:Supersedes", docs, link_targets_needed)

    for sm in SPEC_LINE_RE.finditer(text):
        collect_relative_links(sm.group(1), f"{num}:Spec", docs, link_targets_needed)

# Index table rows: decision/implementation/superseded-by consistency
index_rows: dict[str, dict[str, str]] = {}
for line in index_text.splitlines():
    m = INDEX_ROW_RE.match(line.strip())
    if not m:
        continue
    num, _title, decision_cell, impl_cell, superseded_cell, _notes = m.groups()
    decision_cell = decision_cell.strip()
    impl_cell = impl_cell.strip()
    superseded_cell = superseded_cell.strip()
    index_rows[num] = {
        "decision": decision_cell,
        "implementation": impl_cell,
        "superseded": superseded_cell,
    }
    for href in LINK_RE.findall(superseded_cell):
        target = resolve_repo_target(href, docs)
        if target is not None:
            link_targets_needed.append((f"INDEX:{num}", href, target))

add(
    "index-table-rows",
    set(index_rows) >= set(f"{n:04d}" for n in range(3, 15)),
    "index table covers ADR-0003..0014"
    if set(index_rows) >= set(f"{n:04d}" for n in range(3, 15))
    else f"index table missing rows for {sorted(set(f'{n:04d}' for n in range(3, 15)) - set(index_rows))}",
)

status_drift: list[str] = []
supersede_drift: list[str] = []
for num, row in sorted(index_rows.items()):
    meta = file_meta.get(num)
    if meta is None:
        status_drift.append(f"{num}: index row without ADR file")
        continue
    file_decision = meta.get("decision")
    file_impl = meta.get("implementation")
    if file_decision != row["decision"]:
        status_drift.append(
            f"{num}: decision index={row['decision']!r} file={file_decision!r}"
        )
    if file_impl != impl_token(row["implementation"]):
        status_drift.append(
            f"{num}: implementation index={row['implementation']!r} file={file_impl!r}"
        )

    index_targets = {
        m.group(1)
        for href in LINK_RE.findall(row["superseded"])
        if (m := ADR_HREF_RE.search(href))
    }
    file_targets = set(meta.get("superseded_by_targets") or set())  # type: ignore[arg-type]
    # Index may list successors while historical files omit a Superseded-by line.
    # Require: every index target exists as a file, and if the file declares
    # Superseded-by targets they must match the index set.
    if file_targets and file_targets != index_targets:
        supersede_drift.append(
            f"{num}: superseded-by index={sorted(index_targets)} file={sorted(file_targets)}"
        )
    if row["decision"] == "Superseded" and not index_targets and row["superseded"] not in {"—", "-", "—."}:
        # 0008 is allowed to be Superseded without a concrete successor link.
        if num != "0008":
            supersede_drift.append(f"{num}: Superseded without successor link in index")
    if row["decision"] != "Superseded" and index_targets:
        supersede_drift.append(
            f"{num}: non-Superseded decision has superseded-by targets {sorted(index_targets)}"
        )

add(
    "index-status-consistency",
    len(status_drift) == 0,
    "index decision/implementation match ADR files"
    if not status_drift
    else f"status drift: {status_drift}",
)
add(
    "index-superseded-consistency",
    len(supersede_drift) == 0,
    "index superseded-by targets consistent with ADR files"
    if not supersede_drift
    else f"supersede drift: {supersede_drift}",
)

# --- relative forward/lineage target existence ---
unresolved = []
for src, href, target in link_targets_needed:
    if not target.is_file():
        unresolved.append(f"{src}→{href}")

add(
    "lineage-link-targets",
    len(unresolved) == 0,
    "all Supersedes/Superseded-by/Spec/index targets exist"
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

# --- entry docs: no inaccessible LTL ADR-0001/0002; require ADR-INDEX + ADR-0011 pointers ---
ltl_hits = []
missing_authority_pointer = []
for p in ENTRY_DOCS:
    if not p.is_file():
        add(f"entry-{p.name}", False, f"missing entry doc {p}")
        continue
    text = p.read_text(encoding="utf-8")
    if LTL_RE.search(text):
        ltl_hits.append(str(p.relative_to(root)))
    add(f"entry-{p.name}-present", True, str(p.relative_to(root)))
    has_index = "ADR-INDEX.md" in text
    has_0011 = "ADR-0011" in text
    if not (has_index and has_0011):
        missing_authority_pointer.append(str(p.relative_to(root)))

add(
    "no-ltl-authority-in-entry-docs",
    len(ltl_hits) == 0,
    "entry docs free of inaccessible LTL ADR-0001/0002 paths"
    if not ltl_hits
    else f"LTL ADR authority refs in: {ltl_hits}",
)
add(
    "entry-docs-authority-pointer",
    len(missing_authority_pointer) == 0,
    "entry docs point at ADR-INDEX.md and ADR-0011"
    if not missing_authority_pointer
    else f"missing ADR-INDEX/ADR-0011 pointer in: {missing_authority_pointer}",
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
