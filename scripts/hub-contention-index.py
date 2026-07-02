#!/usr/bin/env python3
"""Hub contention index — thema_009 / VCR.

Computes Neo4j degree for entity names referenced in the honeypot and writes a
contention-tier cache consumed by vault.rs RRF recall. Implements (without
superposition math) the paper's finding that high-degree facts are intrinsically
harder to retrieve even as standalone atomic queries (hop-2 atomic difficulty).

Cache: data/hub-contention-cache.json
  { "entities": { "<Name>": {"degree": int, "tier": "low|med|high"} },
    "generated_at": "...", "thresholds": {"med": int, "high": int} }

Uses batched aggregation query for efficiency (single round-trip vs 12k+).
"""
from __future__ import annotations

import json
import os
import re
import sqlite3
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VAULT_DB = os.environ.get("GZMO_VAULT_DB", str(ROOT / "data" / "vault.db"))
OUT = ROOT / "data" / "hub-contention-cache.json"
DEFAULT_MED = int(os.environ.get("HUB_TIER_MED", "4"))
DEFAULT_HIGH = int(os.environ.get("HUB_TIER_HIGH", "8"))


def load_repo_dotenv() -> None:
    env_path = ROOT / ".env"
    if not env_path.is_file():
        return
    for line in env_path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, _, val = line.partition("=")
        if key.strip() and key.strip() not in os.environ:
            os.environ[key.strip()] = val.strip().strip('"').strip("'")


def honeypot_entity_names(max_entities: int = 2000) -> set[str]:
    """Extract [CATEGORY:Name] bracket tags from honeypot content.
    
    Limits to top-N by confidence to keep Neo4j query fast.
    """
    names: set[str] = set()
    if not Path(VAULT_DB).exists():
        return names
    try:
        con = sqlite3.connect(VAULT_DB)
        # Prioritize high-confidence facts and extract bracket-style tags
        rows = con.execute(
            """SELECT content, confidence 
               FROM honeypot 
               WHERE is_latest = 1 AND content LIKE '%[%:%]%'
               ORDER BY confidence DESC, recall_count DESC
               LIMIT ?""",
            (max_entities * 2,),
        ).fetchall()
        
        for content, _ in rows:
            for m in re.finditer(r"\[\w+:\s*([^\]]{3,100})\]", content or ""):
                names.add(m.group(1).strip())
            if len(names) >= max_entities:
                break
                
        con.close()
    except sqlite3.Error as e:
        print(f"[WARN] SQLite error: {e}", file=sys.stderr)
    return names


def fetch_degrees_batched(names: set[str], timeout: int = 45) -> dict[str, int]:
    """Fetch degrees for all names in a single batched Cypher query."""
    if not names:
        return {}
    try:
        from neo4j import GraphDatabase
    except ImportError:
        return {}
    
    url = os.environ.get("NEO4J_URL", "bolt://192.168.31.202:7687")
    user = os.environ.get("NEO4J_USERNAME", os.environ.get("NEO4J_USER", "neo4j"))
    password = os.environ.get("NEO4J_PASSWORD", os.environ.get("NEO4J_PASS", ""))
    if not password:
        return {}
    
    driver = GraphDatabase.driver(url, auth=(user, password))
    try:
        driver.verify_connectivity()
    except Exception:
        driver.close()
        return {}
    
    # Smaller batch for faster query - hub detection only needs high-degree nodes
    name_list = list(names)[:1500]
    
    out: dict[str, int] = {}
    try:
        with driver.session(database=os.environ.get("NEO4J_DATABASE", "neo4j")) as session:
            # Use exact match only for speed - avoids full scan with CONTAINS
            result = session.run(
                """
                UNWIND $names AS name
                MATCH (e {name: name})
                RETURN e.name AS matched_name, COUNT { (e)--() } AS deg
                """,
                names=name_list,
            )
            for record in result:
                matched = record.get("matched_name")
                deg = record.get("deg")
                if matched and deg is not None:
                    out[matched] = int(deg)
                    out[matched.lower()] = int(deg)
    except Exception as e:
        print(f"[WARN] Neo4j query error: {e}", file=sys.stderr)
    finally:
        driver.close()
    
    return out


def tier(deg: int, med: int, high: int) -> str:
    if deg >= high:
        return "high"
    if deg >= med:
        return "med"
    return "low"


def main() -> int:
    load_repo_dotenv()
    names = honeypot_entity_names()
    print(f"[INFO] Extracted {len(names)} potential entity names from honeypot", file=sys.stderr)
    
    degrees = fetch_degrees_batched(names)
    print(f"[INFO] Retrieved degrees for {len(degrees)//2} unique entities from Neo4j", file=sys.stderr)
    
    # Build entities dict with proper casing from original names
    entities: dict[str, dict] = {}
    for name in names:
        deg = degrees.get(name) or degrees.get(name.lower()) or 0
        if deg > 0:
            entities[name] = {
                "degree": deg,
                "tier": tier(deg, DEFAULT_MED, DEFAULT_HIGH),
            }
    
    payload = {
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "thresholds": {"med": DEFAULT_MED, "high": DEFAULT_HIGH},
        "entity_count": len(entities),
        "entities": entities,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    
    high = sum(1 for v in entities.values() if v["tier"] == "high")
    med = sum(1 for v in entities.values() if v["tier"] == "med")
    print(f"Hub contention index: {len(entities)} entities (high={high}, med={med}) -> {OUT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
