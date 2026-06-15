#!/usr/bin/env python3
"""Export Neo4j CONCEPT prerequisite edges to pending YAML graphs for human review.

Usage:
  pip install neo4j
  ./scripts/export-prerequisite-graph.py --out data/pedagogy/graphs/pending
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load_neo4j_config() -> tuple[str, str, str]:
    user = os.environ.get("NEO4J_USER") or os.environ.get("NEO4J_USERNAME", "neo4j")
    password = os.environ.get("NEO4J_PASSWORD", "")
    uri = os.environ.get("NEO4J_URI") or os.environ.get("BOLT_URI")
    toml_path = ROOT / "gzmo.toml"
    if toml_path.is_file():
        try:
            import tomllib

            cfg = tomllib.loads(toml_path.read_text())
            ps = cfg.get("platform_search") or {}
            uri = uri or ps.get("neo4j_uri")
            user = ps.get("neo4j_username") or user
            password = ps.get("neo4j_password") or password
        except Exception:
            pass
    return uri or "bolt://192.168.31.202:7687", user, password


def slug_id(name: str) -> str:
    s = name.strip().lower()
    s = re.sub(r"[^a-z0-9]+", "_", s)
    return s.strip("_") or "concept"


def connected_components(nodes: dict[str, set[str]]) -> list[set[str]]:
    parent: dict[str, str] = {n: n for n in nodes}

    def find(x: str) -> str:
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x

    def union(a: str, b: str) -> None:
        ra, rb = find(a), find(b)
        if ra != rb:
            parent[rb] = ra

    for child, prereqs in nodes.items():
        for p in prereqs:
            union(child, p)
        if not prereqs:
            find(child)

    comps: dict[str, set[str]] = defaultdict(set)
    for n in nodes:
        comps[find(n)].add(n)
    return sorted(comps.values(), key=len, reverse=True)


def make_acyclic(nodes: dict[str, set[str]]) -> dict[str, set[str]]:
    """Drop prerequisite edges that would form cycles (RELATED_TO is noisy)."""
    out = {k: set(v) for k, v in nodes.items()}

    def reachable(adj: dict[str, set[str]], start: str, goal: str) -> bool:
        stack = [start]
        seen = set()
        while stack:
            n = stack.pop()
            if n == goal:
                return True
            if n in seen:
                continue
            seen.add(n)
            stack.extend(adj.get(n, set()) - seen)
        return False

    changed = True
    while changed:
        changed = False
        for node in list(out):
            for p in list(out.get(node, set())):
                if reachable(out, p, node):
                    out[node].discard(p)
                    changed = True
    return out


def write_yaml(domain: str, node_ids: set[str], nodes: dict[str, set[str]], out_path: Path) -> None:
    acyclic = make_acyclic({k: v & node_ids for k, v in nodes.items() if k in node_ids})
    lines = [f"domain: {domain}", "nodes:"]
    for node_id in sorted(node_ids):
        prereqs = sorted(acyclic.get(node_id, set()))
        title = node_id.replace("_", " ").title()
        lines.append(f"  - id: {node_id}")
        lines.append(f"    title: {title}")
        lines.append(
            f"    description: Exported from Neo4j ({len(prereqs)} prereqs in cluster)."
        )
        lines.append("    prerequisites:")
        if prereqs:
            for p in prereqs:
                lines.append(f"      - {p}")
        else:
            lines.append("      []")
        lines.append("    bloom_level: understand")
    out_path.write_text("\n".join(lines) + "\n")


def fetch_edges(session, limit: int) -> list[tuple[str, str]]:
    queries = [
        (
            "PREREQUISITE_OF",
            """
            MATCH (c)-[:PREREQUISITE_OF]->(p)
            WHERE c.name IS NOT NULL AND p.name IS NOT NULL
            RETURN c.name AS child, p.name AS parent
            LIMIT $limit
            """,
        ),
        (
            "RELATED_TO",
            """
            MATCH (c:CONCEPT)-[:RELATED_TO]->(p:CONCEPT)
            WHERE c.name IS NOT NULL AND p.name IS NOT NULL
            RETURN c.name AS child, p.name AS parent
            LIMIT $limit
            """,
        ),
    ]
    edges: list[tuple[str, str]] = []
    for label, cypher in queries:
        for record in session.run(cypher, limit=limit):
            child = record["child"]
            parent = record["parent"]
            if child and parent:
                edges.append((slug_id(str(child)), slug_id(str(parent))))
        if edges:
            print(f"[OK] fetched {len(edges)} edges via {label}")
            break
    return edges


def main() -> int:
    parser = argparse.ArgumentParser(description="Export Neo4j prerequisite graphs")
    parser.add_argument(
        "--out",
        default=str(ROOT / "data" / "pedagogy" / "graphs" / "pending"),
        help="Output directory (review before moving to graphs/)",
    )
    parser.add_argument("--limit", type=int, default=500, help="Max edges to fetch")
    parser.add_argument("--top-clusters", type=int, default=3, help="Largest components to export")
    args = parser.parse_args()

    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    try:
        from neo4j import GraphDatabase
    except ImportError:
        print("[FAIL] pip install neo4j", file=sys.stderr)
        return 1

    uri, user, password = load_neo4j_config()
    if not password:
        print("[WARN] NEO4J_PASSWORD unset — export may fail auth", file=sys.stderr)

    driver = GraphDatabase.driver(uri, auth=(user, password))
    edges: list[tuple[str, str]] = []
    with driver.session() as session:
        edges = fetch_edges(session, args.limit)
    driver.close()

    if not edges:
        print("[WARN] no edges found", file=sys.stderr)
        return 1

    nodes: dict[str, set[str]] = defaultdict(set)
    for child, parent in edges:
        nodes[child].add(parent)
        nodes.setdefault(parent, set())

    comps = connected_components(nodes)
    written = []
    for i, comp in enumerate(comps[: args.top_clusters]):
        if len(comp) < 3:
            continue
        domain = f"generated-neo4j-cluster{i + 1}"
        out_path = out_dir / f"{domain}.yaml"
        write_yaml(domain, comp, nodes, out_path)
        written.append((domain, len(comp), out_path))
        print(f"[OK] cluster {i + 1}: {len(comp)} nodes -> {out_path}")

    meta_path = out_dir / "export-meta.json"
    meta_path.write_text(
        json.dumps(
            {
                "uri": uri,
                "edges": len(edges),
                "nodes": len(nodes),
                "clusters_written": [{"domain": d, "nodes": n, "path": str(p)} for d, n, p in written],
            },
            indent=2,
        )
        + "\n"
    )
    print(f"[OK] meta -> {meta_path}")
    print("Review pending YAML, then: gzmo pedagogy graph validate data/pedagogy/graphs/pending/")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
