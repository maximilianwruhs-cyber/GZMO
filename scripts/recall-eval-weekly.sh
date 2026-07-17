#!/usr/bin/env bash
# Weekly recall floor → data-next/recall-report.json (GZMO-next scheduler).
# Usage: recall-eval-weekly.sh [OUTPUT_PATH] [VAULT_PATH]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLONE="${GZMO_CLONE_ROOT:-$(cd "$ROOT/.." && pwd)}"
RECALL_ROOT="${RECALL_EVAL_ROOT:-$CLONE/recall-eval}"

OUT="${1:-$ROOT/data-next/recall-report.json}"
VAULT="${2:-$ROOT/data-next/vault.db}"

# Match gzmo-next.toml defaults (VM200 embed); recipes/scheduler may override.
export EMBED_URL="${EMBED_URL:-http://192.168.31.110:8081/v1}"
export EMBED_MODEL="${EMBED_MODEL:-gzmo-embed}"
export QDRANT_URL="${QDRANT_URL:-http://127.0.0.1:6333}"
export QDRANT_COLLECTION="${QDRANT_COLLECTION:-honeypot}"

# recall-eval live.py expects a full embeddings POST URL when EMBED_URL is set.
EMBED_POST="$EMBED_URL"
case "$EMBED_POST" in
  */embeddings) ;;
  */v1) EMBED_POST="${EMBED_POST}/embeddings" ;;
  *) EMBED_POST="${EMBED_POST%/}/v1/embeddings" ;;
esac
export EMBED_URL="$EMBED_POST"

if [[ ! -d "$RECALL_ROOT" ]]; then
  echo "recall-eval-weekly: recall-eval missing at $RECALL_ROOT" >&2
  exit 1
fi

mkdir -p "$(dirname "$OUT")"

LIVE_ARGS=()
if curl -sf "${QDRANT_URL}/collections" >/dev/null 2>&1 \
  && curl -sf "${EMBED_POST%/embeddings}/models" >/dev/null 2>&1; then
  LIVE_ARGS=(--live)
  echo "recall-eval-weekly: live Qdrant + embed"
else
  echo "recall-eval-weekly: fixture mode (Qdrant/embed not both reachable)"
fi

cd "$RECALL_ROOT"
set +e
PYTHONPATH=. python3 -m recall_eval.cli \
  --fixture fixtures/golden-recall.json \
  "${LIVE_ARGS[@]}" \
  -o "$OUT"
RC=$?
set -e

if [[ $RC -ne 0 && ${#LIVE_ARGS[@]} -gt 0 ]]; then
  echo "recall-eval-weekly: live failed (rc=$RC) — falling back to fixture" >&2
  PYTHONPATH=. python3 -m recall_eval.cli \
    --fixture fixtures/golden-recall.json \
    -o "$OUT"
fi

echo "recall-eval-weekly: wrote $OUT (vault hint: $VAULT)"
