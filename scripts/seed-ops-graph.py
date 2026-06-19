#!/usr/bin/env python3
"""Seed operational graph entities: ROUTING_RULE + DAEMON_STATE (+ SKILL registry).

Closes the Pillar B schema gap: distill promotes domain entities but runtime ops
types were missing from Neo4j. Idempotent MERGE via neo4j driver.

Usage:
  seed-ops-graph.py [--dry-run] [--gzmo-root PATH]
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

try:
    import tomllib
except ImportError:
    import tomli as tomllib  # type: ignore


SKILL_NAMES = [
    "dice", "sound", "poker", "quote", "calculate", "visual", "joke", "poem",
    "story", "word", "define", "card", "pkm", "transform", "language",
    "stabilize", "ops", "learn", "discover", "help",
]


def load_chaos_state(data_dir: Path) -> dict:
    path = data_dir / "CHAOS_STATE.json"
    if not path.is_file():
        return {}
    try:
        return json.loads(path.read_text())
    except (json.JSONDecodeError, OSError):
        return {}


def daemon_active() -> bool:
    try:
        out = subprocess.run(
            ["systemctl", "--user", "is-active", "gzmo-daemon.service"],
            capture_output=True,
            text=True,
            timeout=5,
        )
        return out.stdout.strip() == "active"
    except (OSError, subprocess.TimeoutExpired):
        return False


def build_entities(gzmo_root: Path) -> tuple[list[dict], list[dict]]:
    cfg = tomllib.loads((gzmo_root / "gzmo.toml").read_text())
    routing = cfg.get("routing", {})
    mappings = routing.get("mappings", {})
    default_engine = routing.get("default_engine", "local")
    data_dir = gzmo_root / "data"
    chaos = load_chaos_state(data_dir)

    now = datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")
    entities: list[dict] = []
    relations: list[dict] = []

    daemon_name = "gzmo-daemon"
    daemon_obs = [
        f"[seeded_at] {now}",
        f"[default_engine] {default_engine}",
        f"[use_librarian] {cfg.get('use_librarian', False)}",
        f"[cloud_first_background] {routing.get('cloud_first_background', False)}",
        f"[systemd] {'active' if daemon_active() else 'inactive'}",
    ]
    if chaos:
        daemon_obs.extend([
            f"[tick] {chaos.get('tick', '?')}",
            f"[phase] {chaos.get('phase', '?')}",
            f"[rho_effective] {chaos.get('rho_effective', '?')}",
            f"[thoughts_crystallized] {chaos.get('thoughts_crystallized', '?')}",
            f"[tension] {chaos.get('tension', '?')}",
        ])
    entities.append({
        "name": daemon_name,
        "type": "DAEMON_STATE",
        "observations": daemon_obs,
    })

    for key, profile in sorted(mappings.items()):
        rule_name = f"routing-{key.replace('_', '-')}"
        entities.append({
            "name": rule_name,
            "type": "ROUTING_RULE",
            "observations": [
                f"[mapping_key] {key}",
                f"[profile] {profile}",
                f"[default_engine] {default_engine}",
                f"[seeded_at] {now}",
            ],
        })
        relations.append({
            "source": daemon_name,
            "target": rule_name,
            "relationType": "ROUTES_VIA",
        })
        profile_node = f"profile-{profile}"
        entities.append({
            "name": profile_node,
            "type": "ROUTING_PROFILE",
            "observations": [
                f"[profile_name] {profile}",
                f"[engine_url] http://localhost:8000/v1",
                f"[seeded_at] {now}",
            ],
        })
        relations.append({
            "source": rule_name,
            "target": profile_node,
            "relationType": "USES_PROFILE",
        })

    for skill in SKILL_NAMES:
        skill_name = f"skill-{skill}"
        entities.append({
            "name": skill_name,
            "type": "SKILL",
            "observations": [
                f"[slash_command] /{skill}",
                f"[registry] gzmo-core/src/skills/registry.rs",
                f"[seeded_at] {now}",
            ],
        })
        relations.append({
            "source": daemon_name,
            "target": skill_name,
            "relationType": "DISPATCHES",
        })

    return entities, relations


def merge_graph(
    url: str,
    user: str,
    password: str,
    entities: list[dict],
    relations: list[dict],
    dry_run: bool,
) -> dict:
    if dry_run:
        return {
            "dry_run": True,
            "entities": len(entities),
            "relations": len(relations),
        }

    from neo4j import GraphDatabase

    driver = GraphDatabase.driver(url, auth=(user, password))
    created_entities = 0
    created_relations = 0

    with driver.session() as session:
        for etype, group in _group_by(entities, "type").items():
            session.run(
                f"""
                UNWIND $entities AS entity
                MERGE (e:Memory {{name: entity.name}})
                SET e.type = entity.type
                SET e:`{etype}`
                FOREACH (obs_text IN entity.observations |
                    MERGE (o:Observation {{text: obs_text, entity_name: entity.name}})
                    ON CREATE SET o.created_at = datetime(), o.source = "seed-ops-graph"
                    MERGE (e)-[:HAS_OBSERVATION]->(o)
                )
                """,
                entities=group,
            )
            created_entities += len(group)

        for rtype, group in _group_by(relations, "relationType").items():
            session.run(
                f"""
                UNWIND $relations AS relation
                MATCH (from:Memory {{name: relation.source}}), (to:Memory {{name: relation.target}})
                MERGE (from)-[r:`{rtype}`]->(to)
                """,
                relations=group,
            )
            created_relations += len(group)

        counts = session.run(
            """
            MATCH (n)
            WHERE n:ROUTING_RULE OR n:DAEMON_STATE OR n:SKILL
            WITH labels(n) AS ls, count(*) AS c
            UNWIND ls AS l
            WITH l, sum(c) AS total
            WHERE l IN ['ROUTING_RULE', 'DAEMON_STATE', 'SKILL']
            RETURN l AS label, total AS count
            """
        ).data()

    driver.close()
    return {
        "entities_upserted": created_entities,
        "relations_upserted": created_relations,
        "label_counts": {r["label"]: r["count"] for r in counts},
    }


def _group_by(items: list[dict], key: str) -> dict[str, list[dict]]:
    out: dict[str, list[dict]] = {}
    for item in items:
        out.setdefault(item[key], []).append(item)
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description="Seed ROUTING_RULE + DAEMON_STATE in Neo4j")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--gzmo-root", default=os.environ.get("GZMO_ROOT", ""))
    parser.add_argument("--json-out", default="")
    args = parser.parse_args()

    gzmo_root = Path(args.gzmo_root or Path(__file__).resolve().parents[1])
    entities, relations = build_entities(gzmo_root)

    url = os.environ.get("NEO4J_URL", "bolt://192.168.31.202:7687")
    user = os.environ.get("NEO4J_USERNAME", "neo4j")
    password = os.environ.get("NEO4J_PASSWORD", "")

    if not args.dry_run and not password:
        print("[!] NEO4J_PASSWORD required (or set in remediation-env)", file=sys.stderr)
        return 1

    result = {
        "script": "seed-ops-graph",
        "timestamp": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "gzmo_root": str(gzmo_root),
        "entity_count": len(entities),
        "relation_count": len(relations),
    }

    try:
        merge_result = merge_graph(url, user, password, entities, relations, args.dry_run)
        result.update(merge_result)
        result["verdict"] = "ok"
    except Exception as exc:
        result["error"] = str(exc)
        result["verdict"] = "failed"
        print(json.dumps(result, indent=2))
        return 1

    print(json.dumps(result, indent=2))
    if args.json_out:
        Path(args.json_out).write_text(json.dumps(result, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
