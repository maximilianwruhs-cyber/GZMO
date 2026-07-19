#!/usr/bin/env bash
# OKCP memory marketplace spike — export concept bundles; gate inbound write intents.
# GZMO stays the slow compiler; forge/OKCP is the API surface (sibling artifacts only).
#
#   bash scripts/okcp-marketplace.sh              # export bundle
#   bash scripts/okcp-marketplace.sh --intent write   # run concept-gate for external writer
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
WIKI="${GZMO_WIKI_DIR:-$ROOT/wiki}"
OUT="$DATA/okcp-marketplace"
MODE="export"
for a in "$@"; do
  case "$a" in
    --intent) MODE="intent" ;;
    --export) MODE="export" ;;
  esac
done
mkdir -p "$OUT/bundles"
export DATA WIKI OUT ROOT MODE

python3 - <<'PY'
import json, os, hashlib, re
from datetime import datetime, timezone
from pathlib import Path

wiki = Path(os.environ["WIKI"])
out = Path(os.environ["OUT"])
root = Path(os.environ["ROOT"])
data = Path(os.environ["DATA"])
mode = os.environ.get("MODE", "export")
now = datetime.now(timezone.utc)
concepts_dir = wiki / "concepts"
source = "wiki/concepts"

items = []
if concepts_dir.is_dir():
    for p in sorted(concepts_dir.glob("*.md"))[:40]:
        text = p.read_text(encoding="utf-8", errors="replace")
        title = p.stem.replace("-", " ")
        m = re.search(r"^title:\s*[\"']?([^\"'\n]+)", text, re.M | re.I)
        if m:
            title = m.group(1).strip()
        body = text.split("---", 2)[-1] if text.lstrip().startswith("---") else text
        summary = ""
        for line in body.splitlines():
            line = line.strip()
            if line and not line.startswith("#"):
                summary = line[:160]
                break
        digest = hashlib.sha256(text.encode()).hexdigest()[:16]
        items.append({
            "slug": p.stem,
            "title": title,
            "path": str(p.relative_to(root)) if str(p).startswith(str(root)) else str(p),
            "summary": summary,
            "sha256_16": digest,
            "bytes": p.stat().st_size,
            "gate": None,
        })

# Local wiki/concepts is often empty (concepts live on gzmo-next-memory after push).
# Fall back to concept-gate results + wiki-push meta for a marketplace-readable catalog.
if not items:
    source = "concept-gate"
    gate_path = data / "concept-gate" / "latest.json"
    push_path = data / "wiki-push-latest.json"
    gate = {}
    push = {}
    try:
        gate = json.loads(gate_path.read_text(encoding="utf-8"))
    except Exception:
        pass
    try:
        push = json.loads(push_path.read_text(encoding="utf-8"))
    except Exception:
        pass
    for r in (gate.get("results") or [])[:40]:
        path = r.get("path") or ""
        slug = Path(path).stem if path else (r.get("needle") or "unknown")
        title = r.get("query") or slug.replace("-", " ")
        excerpt = (r.get("evidence_excerpt") or "")[:160]
        digest = hashlib.sha256(
            f"{path}|{title}|{r.get('verdict')}|{excerpt}".encode()
        ).hexdigest()[:16]
        items.append({
            "slug": slug,
            "title": title,
            "path": path,
            "summary": excerpt or f"gate={r.get('verdict')}",
            "sha256_16": digest,
            "bytes": None,
            "gate": r.get("verdict"),
        })
    if not items and push.get("concepts_written"):
        source = "wiki-push-meta"
        items.append({
            "slug": "wiki-push-summary",
            "title": "Latest wiki push concept batch",
            "path": "wiki-push-latest.json",
            "summary": f"concepts_written={push.get('concepts_written')} sha={(push.get('commit_sha') or '')[:12]}",
            "sha256_16": hashlib.sha256(str(push.get("commit_sha")).encode()).hexdigest()[:16],
            "bytes": None,
            "gate": None,
        })

bundle = {
    "schema": "gzmo.okcp.bundle/v1",
    "generated_at": now.isoformat(),
    "owner": "gzmo",
    "repo_hint": "gzmo/gzmo-next-memory",
    "scope": "read:concepts",
    "source": source,
    "count": len(items),
    "concepts": items,
    "auth": {
        "read": "public-or-token (operator)",
        "write": "requires PR/review + concept-gate PASS",
        "scopes": ["okcp.concepts.read", "okcp.concepts.write.review"],
    },
    "note": "Export bundle for marketplace consumers; writers go through gate + human merge.",
}
bundle_path = out / "bundles" / f"concepts-{now.strftime('%Y%m%dT%H%M%SZ')}.json"
bundle_path.write_text(json.dumps(bundle, indent=2) + "\n", encoding="utf-8")
(out / "latest-bundle.json").write_text(json.dumps(bundle, indent=2) + "\n", encoding="utf-8")

gate = None
if mode == "intent":
    import subprocess
    proc = subprocess.run(
        ["bash", str(root / "scripts" / "concept-gate-webhook.sh")],
        cwd=str(root),
        capture_output=True,
        text=True,
    )
    try:
        gate = json.loads((Path(os.environ["DATA"]) / "concept-gate" / "webhook-latest.json").read_text())
    except Exception:
        gate = {"verdict": "HOLD" if proc.returncode else "PASS", "gate_exit": proc.returncode}

payload = {
    "schema": "gzmo.okcp.marketplace/v1",
    "generated_at": now.isoformat(),
    "ok": True,
    "mode": mode,
    "bundle": str(bundle_path),
    "concept_count": len(items),
    "write_intent": gate,
    "advice": (
        "merge_ok — run wiki-push-gated"
        if gate and gate.get("verdict") == "PASS"
        else (
            "hold_no_merge"
            if gate
            else "read_bundle_only — use --intent write to gate external writers"
        )
    ),
    "note": "Marketplace spike — no public auth server; scopes documented for OKCP/OKForge later.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# OKCP memory marketplace",
            "",
            f"Bundle concepts: **{len(items)}**",
            f"Mode: {mode}",
            f"Advice: {payload['advice']}",
            "",
            f"Latest bundle: `{bundle_path}`",
            "",
            payload["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({k: payload[k] for k in ("ok", "mode", "concept_count", "advice", "bundle")}, indent=2))
PY
