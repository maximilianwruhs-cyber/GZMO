#!/usr/bin/env bash
# Idempotent: remove any prior spike container, start pgvector/pgvector:pg16
# bound to 127.0.0.1:5432 only, wait until pg_isready.
set -euo pipefail

NAME=gzmo-pgvector-spike
IMAGE=pgvector/pgvector:pg16

docker rm -f "$NAME" 2>/dev/null || true

# Bind localhost only — CT101 is on the LAN; never expose 0.0.0.0:5432.
docker run -d \
  --name "$NAME" \
  -e POSTGRES_PASSWORD=spike \
  -p 127.0.0.1:5432:5432 \
  "$IMAGE"

echo "Waiting for pg_isready inside $NAME ..."
for i in $(seq 1 60); do
  if docker exec "$NAME" pg_isready -U postgres >/dev/null 2>&1; then
    echo "Postgres ready after ${i}s"
    docker ps --filter "name=$NAME" --format 'table {{.Names}}\t{{.Ports}}\t{{.Status}}'
    exit 0
  fi
  sleep 1
done

echo "ERROR: pg_isready timed out after 60s" >&2
docker logs "$NAME" 2>&1 | tail -40 >&2
exit 1
