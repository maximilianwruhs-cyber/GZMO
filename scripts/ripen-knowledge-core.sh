#!/usr/bin/env bash
# M5 weekly inquest: ripen honeypot → knowledge_core (see ripen-knowledge-core.py).
# Preview by default; pass --commit after operator review to write the core DB.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec python3 "${ROOT}/scripts/ripen-knowledge-core.py" "$@"
