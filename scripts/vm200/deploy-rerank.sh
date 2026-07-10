#!/usr/bin/env bash
# ─── OBSOLETE ──────────────────────────────────────────────────────────────
# Rerank was consolidated into the main embed server on :8081.
# See deploy-retrieval-layer.sh (updated) and llama-embed.service (updated).
# Both gzmo-embed and gzmo-rerank are served on the same llama-server :8081.
# ───────────────────────────────────────────────────────────────────────────
echo "[!] OBSOLETE — rerank is now consolidated on :8081 (same server as embed)."
echo "    Run deploy-retrieval-layer.sh instead."
echo "    See scripts/vm200/llama-embed.service (updated)."
exit 1
