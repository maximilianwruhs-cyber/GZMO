#!/usr/bin/env python3
"""import_vault.py — COPY live vault to /tmp, verify counts/dim, upsert into spike Postgres.

Never opens /opt/gzmo/data/vault.db for import (WAL-safe: filesystem copy first).
Talks to Postgres via `docker exec gzmo-pgvector-spike psql` (no pip/psycopg).
Gate G3: FAIL LOUD if counts != 1870 / 1774 / 1747 or embedding dim != 1024.
"""

from __future__ import annotations

import argparse
import os
import shutil
import sqlite3
import struct
import subprocess
import sys
import tempfile

EXPECTED = {"facts": 1870, "honeypot": 1774, "evidence": 1747}
EMBED_DIM = 1024
CONTAINER = "gzmo-pgvector-spike"
LIVE_VAULT = "/opt/gzmo/data/vault.db"
SPIKE_VAULT = "/tmp/vault-spike.db"


def die(msg: str, code: int = 1) -> None:
    print(f"FATAL: {msg}", file=sys.stderr)
    sys.exit(code)


def copy_vault(src: str, dst: str) -> None:
    """Filesystem copy of vault + WAL/SHM sidecar files, then open the copy only."""
    for suffix in ("", "-wal", "-shm"):
        p = src + suffix
        if os.path.exists(p):
            shutil.copy2(p, dst + suffix)
            print(f"copied {p} -> {dst + suffix}")
        elif suffix == "":
            die(f"live vault missing: {src}")
    # Checkpoint the COPY into a single file so read-only open is stable.
    conn = sqlite3.connect(dst)
    conn.execute("PRAGMA wal_checkpoint(TRUNCATE)")
    conn.close()
    for suffix in ("-wal", "-shm"):
        p = dst + suffix
        if os.path.exists(p):
            os.remove(p)


def blob_to_f32(blob: bytes | None) -> list[float] | None:
    if blob is None:
        return None
    if len(blob) % 4 != 0:
        die(f"embedding blob length {len(blob)} not divisible by 4")
    dim = len(blob) // 4
    if dim != EMBED_DIM:
        die(f"embedding dim {dim} != {EMBED_DIM} (blob={len(blob)} bytes)")
    return list(struct.unpack(f"<{dim}f", blob))


def vec_literal(vec: list[float] | None) -> str | None:
    if vec is None:
        return None
    return "[" + ",".join(f"{x:.8g}" for x in vec) + "]"


def sql_quote(s: str | None) -> str:
    if s is None:
        return "NULL"
    return "'" + s.replace("'", "''").replace("\x00", "") + "'"


def psql(sql: str, check: bool = True) -> subprocess.CompletedProcess:
    return subprocess.run(
        [
            "docker",
            "exec",
            "-i",
            CONTAINER,
            "psql",
            "-U",
            "postgres",
            "-v",
            "ON_ERROR_STOP=1",
            "-t",
            "-A",
        ],
        input=sql,
        text=True,
        capture_output=True,
        check=False if not check else False,
    )


def psql_ok(sql: str) -> str:
    r = psql(sql)
    if r.returncode != 0:
        die(f"psql failed:\nSTDOUT:\n{r.stdout}\nSTDERR:\n{r.stderr}\nSQL head:\n{sql[:400]}")
    return (r.stdout or "").strip()


def apply_schema(schema_path: str) -> None:
    with open(schema_path, "r", encoding="utf-8") as f:
        schema = f.read()
    # Drop prior tables for idempotent re-import.
    psql_ok(
        "DROP TABLE IF EXISTS evidence CASCADE;\n"
        "DROP TABLE IF EXISTS honeypot CASCADE;\n"
        "DROP TABLE IF EXISTS facts CASCADE;\n"
    )
    psql_ok(schema)
    print("schema applied")


def verify_sqlite(conn: sqlite3.Connection) -> dict:
    cur = conn.cursor()
    counts = {
        "facts": cur.execute("SELECT COUNT(*) FROM semantic_vault").fetchone()[0],
        "honeypot": cur.execute("SELECT COUNT(*) FROM honeypot").fetchone()[0],
        "evidence": cur.execute("SELECT COUNT(*) FROM evidence").fetchone()[0],
    }
    print(f"sqlite counts: {counts}")
    for k, exp in EXPECTED.items():
        if counts[k] != exp:
            die(f"G3 FAIL: {k} count {counts[k]} != expected {exp}")
    # Dim check on a sample of honeypot embeddings
    rows = cur.execute(
        "SELECT embedding FROM honeypot WHERE embedding IS NOT NULL LIMIT 20"
    ).fetchall()
    if not rows:
        die("G3 FAIL: no honeypot embeddings")
    for (blob,) in rows:
        blob_to_f32(blob)  # dies on mismatch
    latest = cur.execute(
        "SELECT COUNT(*) FROM honeypot WHERE is_latest = 1"
    ).fetchone()[0]
    print(f"honeypot is_latest=1: {latest} (ADR expects 478)")
    print(f"embedding dim verified: {EMBED_DIM}")
    return {**counts, "honeypot_latest": latest, "embedding_dim": EMBED_DIM}


def import_facts(conn: sqlite3.Connection, batch: int = 50) -> int:
    cur = conn.cursor()
    rows = cur.execute(
        "SELECT id, content, embedding, confidence, created_at, source_file FROM semantic_vault"
    ).fetchall()
    n = 0
    buf: list[str] = []
    for id_, content, emb, conf, created, src in rows:
        vec = vec_literal(blob_to_f32(emb) if emb else None)
        emb_sql = "NULL" if vec is None else f"'{vec}'::vector"
        conf_sql = "NULL" if conf is None else str(float(conf))
        stmt = (
            "INSERT INTO facts (id, content, content_norm, embedding, confidence, created_at, source_file) VALUES ("
            f"{sql_quote(id_)}, {sql_quote(content)}, "
            f"to_tsvector('english', coalesce({sql_quote(content)}, '')), "
            f"{emb_sql}, {conf_sql}, {sql_quote(created)}, {sql_quote(src)}"
            ");"
        )
        buf.append(stmt)
        if len(buf) >= batch:
            psql_ok("\n".join(buf))
            n += len(buf)
            buf = []
            if n % 500 == 0:
                print(f"  facts imported {n}/{len(rows)}")
    if buf:
        psql_ok("\n".join(buf))
        n += len(buf)
    return n


def import_honeypot(conn: sqlite3.Connection, batch: int = 50) -> int:
    cur = conn.cursor()
    rows = cur.execute(
        "SELECT id, vault_id, content, embedding, is_latest, supersedes_id, confidence, source_file "
        "FROM honeypot"
    ).fetchall()
    n = 0
    buf: list[str] = []
    for id_, vault_id, content, emb, is_latest, supersedes, conf, src in rows:
        vec = vec_literal(blob_to_f32(emb) if emb else None)
        emb_sql = "NULL" if vec is None else f"'{vec}'::vector"
        conf_sql = "NULL" if conf is None else str(float(conf))
        il = 1 if is_latest else 0
        stmt = (
            "INSERT INTO honeypot (id, vault_id, content, content_norm, embedding, is_latest, "
            "supersedes_id, confidence, source_file) VALUES ("
            f"{sql_quote(id_)}, {sql_quote(vault_id)}, {sql_quote(content)}, "
            f"to_tsvector('english', coalesce({sql_quote(content)}, '')), "
            f"{emb_sql}, {il}, {sql_quote(supersedes)}, {conf_sql}, {sql_quote(src)}"
            ");"
        )
        buf.append(stmt)
        if len(buf) >= batch:
            psql_ok("\n".join(buf))
            n += len(buf)
            buf = []
            if n % 500 == 0:
                print(f"  honeypot imported {n}/{len(rows)}")
    if buf:
        psql_ok("\n".join(buf))
        n += len(buf)
    return n


def import_evidence(conn: sqlite3.Connection, batch: int = 50) -> int:
    cur = conn.cursor()
    rows = cur.execute(
        "SELECT id, fact_id, evidence_text, char_start, char_end FROM evidence"
    ).fetchall()
    n = 0
    buf: list[str] = []
    for id_, fact_id, etext, cs, ce in rows:
        cs_sql = "NULL" if cs is None else str(int(cs))
        ce_sql = "NULL" if ce is None else str(int(ce))
        stmt = (
            "INSERT INTO evidence (id, fact_id, evidence_text, content_norm, char_start, char_end) VALUES ("
            f"{sql_quote(id_)}, {sql_quote(fact_id)}, {sql_quote(etext)}, "
            f"to_tsvector('english', coalesce({sql_quote(etext)}, '')), "
            f"{cs_sql}, {ce_sql}"
            ");"
        )
        buf.append(stmt)
        if len(buf) >= batch:
            psql_ok("\n".join(buf))
            n += len(buf)
            buf = []
            if n % 500 == 0:
                print(f"  evidence imported {n}/{len(rows)}")
    if buf:
        psql_ok("\n".join(buf))
        n += len(buf)
    return n


def verify_pg(expected: dict) -> dict:
    out = {}
    for table, key in (("facts", "facts"), ("honeypot", "honeypot"), ("evidence", "evidence")):
        n = int(psql_ok(f"SELECT COUNT(*) FROM {table};"))
        out[key] = n
        if n != expected[key]:
            die(f"G3 FAIL: postgres {table} count {n} != {expected[key]}")
    latest = int(psql_ok("SELECT COUNT(*) FROM honeypot WHERE is_latest = 1;"))
    out["honeypot_latest"] = latest
    # Spot-check vector dim via pgvector
    dim = int(
        psql_ok(
            "SELECT vector_dims(embedding) FROM honeypot "
            "WHERE embedding IS NOT NULL LIMIT 1;"
        )
    )
    if dim != EMBED_DIM:
        die(f"G3 FAIL: postgres embedding dim {dim} != {EMBED_DIM}")
    out["embedding_dim"] = dim
    print(f"postgres counts OK: {out}")
    return out


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--schema", default=os.path.join(os.path.dirname(__file__), "schema.sql"))
    ap.add_argument("--live-vault", default=LIVE_VAULT)
    ap.add_argument("--spike-vault", default=SPIKE_VAULT)
    args = ap.parse_args()

    # Ensure container is up
    r = subprocess.run(
        ["docker", "inspect", "-f", "{{.State.Running}}", CONTAINER],
        capture_output=True,
        text=True,
    )
    if r.returncode != 0 or r.stdout.strip() != "true":
        die(f"container {CONTAINER} not running — run docker-run.sh first")

    copy_vault(args.live_vault, args.spike_vault)
    conn = sqlite3.connect(f"file:{args.spike_vault}?mode=ro", uri=True)
    expected = verify_sqlite(conn)

    apply_schema(args.schema)
    print("importing facts...")
    n_f = import_facts(conn)
    print("importing honeypot...")
    n_h = import_honeypot(conn)
    print("importing evidence...")
    n_e = import_evidence(conn)
    conn.close()

    print(f"imported rows: facts={n_f} honeypot={n_h} evidence={n_e}")
    if (n_f, n_h, n_e) != (EXPECTED["facts"], EXPECTED["honeypot"], EXPECTED["evidence"]):
        die(f"G3 FAIL: imported counts {(n_f, n_h, n_e)} != expected tuple")

    pg = verify_pg(EXPECTED)
    summary = {
        "sqlite": expected,
        "imported": {"facts": n_f, "honeypot": n_h, "evidence": n_e},
        "postgres": pg,
        "g3": "PASS",
    }
    out_path = os.path.join(os.path.dirname(__file__), "import_counts.json")
    import json

    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(summary, f, indent=2)
    print(json.dumps(summary, indent=2))
    print("G3 PASS: import counts + dim exact")


if __name__ == "__main__":
    main()
