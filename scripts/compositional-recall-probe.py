#!/usr/bin/env python3
"""Compositional recall probe — thema_009 / Verified Chain Recall.

Adapts arXiv:2606.24948 Section 5.4 three probes to GZMO's Neo4j + honeypot:

  Probe A — hop-1 fidelity:   query mentions (A, r1) → does `gzmo memory search`
                               surface the intermediate `mid` in top-K?
  Probe B — chain recall:     compositional query "A via r1 mid via r2 B" → does
                               top-K contain a honeypot fact about B (or the chain)?
  Probe C — hop-2 atomic:     query (mid, r2) directly vs average atomic baseline;
                               ratio < 1.0 on hub-heavy chains (paper: 0.26–0.48×).

Chains are mined from Neo4j with leakage control (no direct A->B shortcut in the
training graph). Output JSON to data/discovery-kb-metrics/compositional-recall-{stamp}.json.

Degrades gracefully when Neo4j or the gzmo binary is unavailable (same fail-open
posture as scripts/graph-recall-stream.py).
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GZMO_BIN = os.environ.get("GZMO_BIN", str(ROOT / "target" / "release" / "gzmo"))
OUT_DIR = ROOT / "data" / "discovery-kb-metrics"
DEFAULT_LIMIT = 5
DEFAULT_CHAINS = 3
# Each `gzmo memory search` cold-starts the vault + embedder (~10s on this rig).
SEARCH_TIMEOUT = int(os.environ.get("COMPOSITIONAL_SEARCH_TIMEOUT", "25"))
MAX_WORKERS = int(os.environ.get("COMPOSITIONAL_WORKERS", "4"))


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


def neo4j_driver():
    try:
        from neo4j import GraphDatabase
    except ImportError:
        return None
    url = os.environ.get("NEO4J_URL", "bolt://192.168.31.202:7687")
    user = os.environ.get("NEO4J_USERNAME", os.environ.get("NEO4J_USER", "neo4j"))
    password = os.environ.get("NEO4J_PASSWORD", os.environ.get("NEO4J_PASS", ""))
    if not password:
        return None
    driver = GraphDatabase.driver(url, auth=(user, password))
    # Fail fast on auth/connect errors so the probe doesn't hang the gate.
    try:
        import neo4j as _n
        driver.verify_connectivity()
    except Exception:
        try:
            driver.close()
        except Exception:
            pass
        return None
    return driver


def mine_chains(driver, n_chains: int) -> list[dict]:
    """Top-N two-hop chains (A)-[r1]->(mid)-[r2]->(B) with leakage control.

    Leakage control (Algorithm 1 analogue): discard pairs where B is directly
    reachable from A in one hop, so the chain is genuinely compositional.
    """
    query = """
    MATCH (a)-[r1]->(mid)-[r2]->(b)
    WHERE a.name IS NOT NULL AND mid.name IS NOT NULL AND b.name IS NOT NULL
      AND a.name <> mid.name AND mid.name <> b.name AND a.name <> b.name
      AND NOT EXISTS((a)-[]->(b))
    WITH a, r1, mid, r2, b, count(*) AS support
    WHERE support >= 1
    RETURN a.name AS a, type(r1) AS r1, mid.name AS mid,
           type(r2) AS r2, b.name AS b, support
    ORDER BY support DESC
    LIMIT $n
    """
    out: list[dict] = []
    with driver.session(database=os.environ.get("NEO4J_DATABASE", "neo4j")) as session:
        for row in session.run(query, n=n_chains):
            out.append({
                "a": row["a"], "r1": row["r1"], "mid": row["mid"],
                "r2": row["r2"], "b": row["b"], "support": row["support"],
            })
    return out


def gzmo_search(query: str, limit: int) -> tuple[int, str]:
    if not Path(GZMO_BIN).exists():
        return 127, ""
    try:
        proc = subprocess.run(
            [GZMO_BIN, "memory", "search", query, "--limit", str(limit)],
            capture_output=True, text=True, timeout=SEARCH_TIMEOUT,
        )
        return proc.returncode, proc.stdout
    except subprocess.TimeoutExpired:
        return 124, ""
    except Exception:
        return 1, ""


def hit_in_output(needle: str, output: str) -> bool:
    if not needle:
        return False
    # case-insensitive token substring match on the rendered search output
    pat = re.escape(needle)
    return re.search(pat, output, re.IGNORECASE) is not None


def reciprocal_rank(needle: str, output: str) -> float:
    """MRR over rendered lines: 1/rank of first line containing needle, else 0."""
    if not needle:
        return 0.0
    for idx, line in enumerate(output.splitlines(), start=1):
        if re.search(re.escape(needle), line, re.IGNORECASE):
            return 1.0 / idx
    return 0.0


def run_probes(chains: list[dict], limit: int) -> dict:
    # Build the full query plan up front so we can fan out the cold-start-heavy
    # `gzmo memory search` calls in parallel instead of ~40s each in series.
    plan = []  # (kind, chain_index, query)
    for i, c in enumerate(chains):
        plan.append(("atomic", i, c["b"]))
        plan.append(("hop1", i, f"{c['a']} {c['r1']}"))
        plan.append(("chain", i, f"{c['a']} via {c['r1']} {c['mid']} via {c['r2']} {c['b']}"))
        plan.append(("hop2", i, f"{c['mid']} {c['r2']}"))

    outputs: dict[tuple[str, int], str] = {}
    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as ex:
        futures = {
            ex.submit(gzmo_search, q, limit): (kind, idx)
            for (kind, idx, q) in plan
        }
        for fut in futures:
            kind, idx = futures[fut]
            _, out = fut.result()
            outputs[(kind, idx)] = out

    hop1_rrs, chain_hits, hop2_rrs, atomic_rrs = [], [], [], []

    for i, c in enumerate(chains):
        atomic_rr = reciprocal_rank(c["b"], outputs.get(("atomic", i), ""))
        atomic_rrs.append(atomic_rr)

        hop1_rr = reciprocal_rank(c["mid"], outputs.get(("hop1", i), ""))
        hop1_rrs.append(hop1_rr)

        chain_hit = hit_in_output(c["b"], outputs.get(("chain", i), ""))
        chain_hits.append(1 if chain_hit else 0)

        hop2_rr = reciprocal_rank(c["b"], outputs.get(("hop2", i), ""))
        hop2_rrs.append(hop2_rr)

    per_chain = []
    for c, atomic_rr, hop1_rr, chain_hit, hop2_rr in zip(
        chains, atomic_rrs, hop1_rrs, chain_hits, hop2_rrs
    ):
        hop2_ratio = (hop2_rr / atomic_rr) if atomic_rr > 0 else 0.0
        per_chain.append({
            "chain": c,
            "hop1_rr": round(hop1_rr, 4),
            "chain_hit": chain_hit,
            "hop2_rr": round(hop2_rr, 4),
            "atomic_rr": round(atomic_rr, 4),
            "hop2_atomic_ratio": round(hop2_ratio, 4),
        })

    def mean(xs):
        return round(sum(xs) / len(xs), 4) if xs else 0.0

    return {
        "chains_evaluated": len(chains),
        "hop1_mrr": mean(hop1_rrs),
        "chain_hit_rate": mean(chain_hits),
        "hop2_atomic_mrr": mean(hop2_rrs),
        "atomic_baseline_mrr": mean(atomic_rrs),
        "hop2_atomic_ratio": (
            round(mean(hop2_rrs) / mean(atomic_rrs), 4)
            if mean(atomic_rrs) > 0 else 0.0
        ),
        "per_chain": per_chain,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--chains", type=int, default=DEFAULT_CHAINS)
    ap.add_argument("--limit", type=int, default=DEFAULT_LIMIT)
    ap.add_argument("--out", default=None)
    args = ap.parse_args()

    load_repo_dotenv()
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    driver = neo4j_driver()
    if driver is None:
        print("[SKIP] Neo4j unavailable — no compositional probe run", file=sys.stderr)
        return 0

    chains = mine_chains(driver, args.chains)
    driver.close()
    if not chains:
        print("[SKIP] no compositional chains mined from Neo4j", file=sys.stderr)
        return 0

    metrics = run_probes(chains, args.limit)
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    payload = {
        "generated_at": ts,
        "source": "thema_009 / arXiv:2606.24948",
        **metrics,
    }
    out = Path(args.out) if args.out else OUT_DIR / f"compositional-recall-{stamp}.json"
    out.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    latest = OUT_DIR / "compositional-recall-latest.json"
    if out.parent == OUT_DIR:
        try:
            latest.unlink(missing_ok=True)
            latest.symlink_to(out.name)
        except OSError:
            pass
    print(f"Compositional recall: chains={metrics['chains_evaluated']} "
          f"hop1_mrr={metrics['hop1_mrr']} chain_hit_rate={metrics['chain_hit_rate']} "
          f"hop2_atomic_ratio={metrics['hop2_atomic_ratio']} -> {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
