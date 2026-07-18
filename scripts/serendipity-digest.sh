#!/usr/bin/env bash
# Serendipity digest — spark / nightburst links → markdown for wiki or human review.
set -euo pipefail

ROOT="${GZMO_CLONE_ROOT:-$HOME/github-clone}/GZMO"
DATA="$ROOT/data-next"
OUT_DIR="$DATA/serendipity"
mkdir -p "$OUT_DIR"

exec python3 - "$DATA" "$OUT_DIR" <<'PY'
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

data = Path(sys.argv[1])
out_dir = Path(sys.argv[2])
stamp = datetime.now(timezone.utc).strftime("%Y-%m-%d")
out_path = out_dir / f"digest-{stamp}.md"

def load(path: Path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None

spark = load(data / "spark" / "last-spark-report.json") or {}
# Drop huge embedding blobs from digest view
sel = spark.get("selection") or {}
anchor = sel.get("anchor") or {}
if isinstance(anchor, dict):
    anchor = {k: v for k, v in anchor.items() if k != "embedding"}
recent = sel.get("recent") or sel.get("candidates") or []
if isinstance(recent, list):
    recent = [
        ({k: v for k, v in r.items() if k != "embedding"} if isinstance(r, dict) else r)
        for r in recent[:8]
    ]

dreams = data / "DREAMS.md"
spark_sections = ""
if dreams.is_file():
    text = dreams.read_text(encoding="utf-8", errors="replace")
    m = re.search(r"(?ms)^## Spark\b.*", text)
    if m:
        spark_sections = m.group(0)[:4000]

lines = [
    f"# Serendipity digest — {stamp}",
    "",
    "Overnight-shaped links from spark / dream. No vault rewrite; review then `gzmo wiki push` if useful.",
    "",
    "## Last spark",
    "",
    f"- **date:** {spark.get('date', '—')}",
    f"- **promoted:** {spark.get('promoted')}",
    f"- **skip_reason:** {spark.get('skip_reason') or '—'}",
    f"- **anchor:** {anchor.get('content') if isinstance(anchor, dict) else anchor}",
    "",
]
if recent:
    lines.append("### Nearby / candidates")
    lines.append("")
    for i, r in enumerate(recent, 1):
        if isinstance(r, dict):
            lines.append(f"{i}. {r.get('content') or r.get('id') or r}")
        else:
            lines.append(f"{i}. {r}")
    lines.append("")

if spark_sections:
    lines.append("## Spark sections from DREAMS.md (truncated)")
    lines.append("")
    lines.append("```markdown")
    lines.append(spark_sections.strip())
    lines.append("```")
    lines.append("")

lines.append("## Operator next")
lines.append("")
lines.append("- Promote surprising verified links via chat takeaways / `gzmo session close --takeaway`")
lines.append("- Soft wiki: `gzmo wiki push --origin serendipity-digest` when concepts are ready")
lines.append("")

out_path.write_text("\n".join(lines), encoding="utf-8")
latest = out_dir / "latest.md"
latest.write_text(out_path.read_text(encoding="utf-8"), encoding="utf-8")
print(json.dumps({"digest": str(out_path), "latest": str(latest)}, indent=2))
PY
