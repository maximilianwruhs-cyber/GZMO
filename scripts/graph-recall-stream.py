#!/usr/bin/env python3
"""Neo4j 1-hop hints for RRF graph stream. Prints JSON array of text snippets (stdout)."""
from __future__ import annotations

import json
import os
import re
import sys
from pathlib import Path


def load_repo_dotenv() -> None:
    env_path = Path(__file__).resolve().parents[1] / ".env"
    if not env_path.is_file():
        return
    for line in env_path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, _, val = line.partition("=")
        key = key.strip()
        val = val.strip().strip('"').strip("'")
        if key and key not in os.environ:
            os.environ[key] = val


def tokens_from_query(query: str) -> list[str]:
    out: list[str] = []
    for m in re.finditer(r"\[(\w+):([^\]]+)\]", query):
        out.append(m.group(2).strip())
    for w in re.findall(r"[A-Za-z][A-Za-z0-9_-]{2,}", query):
        if w.lower() not in {"what", "which", "does", "the", "and", "for", "how", "when", "where"}:
            out.append(w)
    for w in query.split():
        wl = w.strip(".,?!:;\"'").lower()
        if len(wl) >= 4 and wl not in out:
            out.append(wl)
    seen: set[str] = set()
    deduped: list[str] = []
    for t in out:
        key = t.lower()
        if key in seen:
            continue
        seen.add(key)
        deduped.append(t)
    return deduped[:8]


def main() -> None:
    load_repo_dotenv()
    query = sys.argv[1] if len(sys.argv) > 1 else ""
    limit = int(sys.argv[2]) if len(sys.argv) > 2 else 30
    hints: list[str] = []

    if not query.strip():
        print(json.dumps(hints))
        return

    try:
        from neo4j import GraphDatabase
    except ImportError:
        print(json.dumps(hints))
        return

    url = os.environ.get("NEO4J_URL", "bolt://192.168.31.202:7687")
    user = os.environ.get("NEO4J_USERNAME", os.environ.get("NEO4J_USER", "neo4j"))
    password = os.environ.get("NEO4J_PASSWORD", os.environ.get("NEO4J_PASS", ""))
    database = os.environ.get("NEO4J_DATABASE", "neo4j")
    if not password:
        print(json.dumps(hints))
        return

    toks = tokens_from_query(query)
    if not toks:
        print(json.dumps(hints))
        return

    driver = GraphDatabase.driver(url, auth=(user, password))
    try:
        with driver.session(database=database) as session:
            for tok in toks:
                rows = session.run(
                    """
                    MATCH (e)
                    WHERE toLower(e.name) CONTAINS toLower($tok)
                    OPTIONAL MATCH (e)-[r]-(n)
                    RETURN e.name AS name,
                           coalesce(e.observations, []) AS obs,
                           n.name AS neighbor,
                           type(r) AS rel
                    LIMIT $lim
                    """,
                    tok=tok,
                    lim=limit,
                )
                for row in rows:
                    name = row.get("name") or ""
                    if name:
                        hints.append(str(name))
                    for obs in row.get("obs") or []:
                        if obs and str(obs) not in hints:
                            hints.append(str(obs)[:500])
                    neighbor = row.get("neighbor")
                    rel = row.get("rel")
                    if neighbor:
                        hints.append(f"{name} {rel or 'RELATED'} {neighbor}")
    finally:
        driver.close()

    # dedupe preserve order
    seen: set[str] = set()
    unique: list[str] = []
    for h in hints:
        key = h.lower()[:200]
        if key in seen:
            continue
        seen.add(key)
        unique.append(h)
    print(json.dumps(unique[:limit]))


if __name__ == "__main__":
    main()
