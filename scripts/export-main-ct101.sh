#!/usr/bin/env bash
# CT101-compatible main.rs (no status_cmd module).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
sed \
  -e '/^mod status_cmd;/d' \
  -e '/Status,/d' \
  -e '/if args\[1\] == "status"/d' \
  -e '/Command::Status/d' \
  -e '/status_cmd::run/d' \
  "$ROOT/gzmo-cli/src/main.rs" > /tmp/main-ct101.rs
echo "Wrote /tmp/main-ct101.rs ($(wc -l </tmp/main-ct101.rs) lines)"
