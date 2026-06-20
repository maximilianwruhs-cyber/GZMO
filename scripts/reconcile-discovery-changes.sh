#!/usr/bin/env bash
# Local discovery change reconciliation (jules-merge scan/status analogue).
# Detects files claimed by multiple remediation sessions; optional three-way merge.
#
# Usage:
#   reconcile-discovery-changes.sh scan [--json]
#   reconcile-discovery-changes.sh status [--json]
#   reconcile-discovery-changes.sh merge-file --path <file> --ours <path> --theirs <path> [--base <path>] [--dry-run]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SKILLS="${GZMO_SKILLS_ROOT:-$HOME/gzmo_skills}"
TRACKER="${PI_MENTOR_DISCOVERY_TRACKER:-$SKILLS/data/pi-mentor-discovery/remediation-tracker.json}"
SNAPSHOTS="${GZMO_DISCOVERY_SNAPSHOTS:-$SKILLS/data/discovery-implementation/snapshots}"
MANIFEST_DIR="$SKILLS/data/pi-mentor-discovery/implementations"
STAGING="${GZMO_RECONCILE_STAGING:-$SKILLS/data/discovery-implementation/reconcile}"
MANIFEST="$STAGING/manifest.json"

mkdir -p "$STAGING"

cmd_scan() {
  local json=0
  [[ "${1:-}" == "--json" ]] && json=1

  python3 - "$TRACKER" "$SNAPSHOTS" "$MANIFEST_DIR" "$MANIFEST" <<'PY'
import json, sys
from collections import defaultdict
from pathlib import Path

tracker_path, snapshots_dir, impl_dir, out_path = map(Path, sys.argv[1:5])
claimed = defaultdict(set)  # path -> session_ids

# From remediation tracker artifact_paths
if tracker_path.is_file():
    try:
        data = json.loads(tracker_path.read_text())
        for f in data.get("findings", []):
            sid = f.get("discovery_session_id") or "unknown"
            for p in f.get("artifact_paths", []):
                if p:
                    claimed[p].add(sid)
            for p in f.get("written_paths", []):
                if p:
                    claimed[p].add(sid)
    except json.JSONDecodeError:
        pass

# From spawn snapshots
snap_dir = snapshots_dir
if snap_dir.is_dir():
    for snap in snap_dir.glob("*.json"):
        try:
            s = json.loads(snap.read_text())
            sid = s.get("session_id", snap.stem)
            for p in s.get("written_paths", []):
                if p:
                    claimed[p].add(sid)
        except json.JSONDecodeError:
            continue

conflicts = []
clean = []
for path, sessions in sorted(claimed.items()):
    entry = {"path": path, "sessions": sorted(sessions)}
    if len(sessions) > 1:
        conflicts.append(entry)
    else:
        clean.append(entry)

manifest = {
    "scanned_at": __import__("datetime").datetime.now(__import__("datetime").timezone.utc).isoformat().replace("+00:00", "Z"),
    "conflicts": conflicts,
    "clean": clean,
    "conflict_count": len(conflicts),
    "ready": len(conflicts) == 0,
    "pending": [c["path"] for c in conflicts],
}
out_path.write_text(json.dumps(manifest, indent=2) + "\n")
print(json.dumps(manifest, indent=2))
PY

  if [[ $json -eq 0 ]]; then
    echo "Manifest: $MANIFEST"
  fi
}

cmd_status() {
  local json=0
  [[ "${1:-}" == "--json" ]] && json=1
  if [[ ! -f "$MANIFEST" ]]; then
    cmd_scan --json >/dev/null
  fi
  if [[ $json -eq 1 ]]; then
    cat "$MANIFEST"
  else
    python3 - "$MANIFEST" <<'PY'
import json, sys
from pathlib import Path
m = json.loads(Path(sys.argv[1]).read_text())
print(f"conflicts: {m['conflict_count']}  ready: {m['ready']}")
for c in m.get("conflicts", []):
    print(f"  CONFLICT {c['path']} <- {', '.join(c['sessions'])}")
PY
  fi
}

cmd_merge_file() {
  local path="" ours="" theirs="" base="" dry=0
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --path) path="$2"; shift 2 ;;
      --ours) ours="$2"; shift 2 ;;
      --theirs) theirs="$2"; shift 2 ;;
      --base) base="$2"; shift 2 ;;
      --dry-run) dry=1; shift ;;
      *) echo "Unknown: $1"; exit 2 ;;
    esac
  done
  [[ -n "$path" && -n "$ours" && -n "$theirs" ]] || {
    echo "Usage: merge-file --path <rel> --ours <file> --theirs <file> [--base <file>] [--dry-run]"
    exit 2
  }

  if [[ -z "$base" ]]; then
    base="$(mktemp)"
    if git -C "$ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
      rel="${path#"$ROOT"/}"
      rel="${rel#./}"
      if git -C "$ROOT" show "HEAD:$rel" >"$base" 2>/dev/null; then
        :
      else
        echo -n "" >"$base"
      fi
    else
      echo -n "" >"$base"
    fi
    trap 'rm -f "$base"' EXIT
  fi

  out="$STAGING/resolved/$(echo "$path" | tr '/' '_')"
  mkdir -p "$(dirname "$out")"

  if [[ $dry -eq 1 ]]; then
    echo "dry-run: git merge-file $ours $base $theirs"
    echo "  output would be: $out"
    exit 0
  fi

  if git merge-file "$ours" "$base" "$theirs" 2>/dev/null; then
    cp "$ours" "$out"
    echo "{\"path\":\"$path\",\"status\":\"merged\",\"output\":\"$out\"}"
    exit 0
  fi

  markers="$(grep -c '^<<<<<<<' "$ours" 2>/dev/null || echo 0)"
  cp "$ours" "$out"
  echo "{\"path\":\"$path\",\"status\":\"conflict\",\"gitConflictMarkers\":$markers,\"output\":\"$out\"}"
  exit 1
}

CMD="${1:-scan}"
shift || true
case "$CMD" in
  scan) cmd_scan "$@" ;;
  status) cmd_status "$@" ;;
  merge-file) cmd_merge_file "$@" ;;
  *)
    echo "Usage: reconcile-discovery-changes.sh scan|status|merge-file ..."
    exit 2
    ;;
esac
