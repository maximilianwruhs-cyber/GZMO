#!/usr/bin/env bash
# Tear down spike container + /tmp vault copy. KEEP the pgvector image (airgap asset).
set -euo pipefail

NAME=gzmo-pgvector-spike

echo "== teardown: stop+rm $NAME (keep image) =="
docker stop "$NAME" 2>/dev/null || true
docker rm "$NAME" 2>/dev/null || true

echo "== teardown: rm /tmp vault copy =="
rm -f /tmp/vault-spike.db /tmp/vault-spike.db-wal /tmp/vault-spike.db-shm

echo "== post-teardown checks =="
echo "-- docker ps names --"
docker ps --format '{{.Names}}'
echo "-- 5432 --"
ss -tln | grep 5432 || echo 5432-free
echo "-- image retained --"
docker images pgvector/pgvector:pg16 --format '{{.Repository}}:{{.Tag}} {{.ID}} {{.Size}}'
echo "teardown OK"
