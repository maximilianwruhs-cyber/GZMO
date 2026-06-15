#!/usr/bin/env python3
"""VM200 retrieval HTTP benchmark — embed, rerank, optional memory_search E2E."""

from __future__ import annotations

import json
import os
import statistics
import subprocess
import sys
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

try:
    import yaml
except ImportError:
    yaml = None

ROOT = Path(__file__).resolve().parents[3]
BENCH_DIR = Path(__file__).resolve().parent
WORKLOADS = BENCH_DIR / "workloads.yaml"


def pct(values: list[float], p: float) -> float:
    if not values:
        return 0.0
    s = sorted(values)
    k = (len(s) - 1) * p
    f = int(k)
    c = min(f + 1, len(s) - 1)
    if f == c:
        return s[f]
    return s[f] + (s[c] - s[f]) * (k - f)


def post_json(url: str, body: dict, timeout: float = 60.0) -> tuple[float, dict]:
    data = json.dumps(body).encode()
    req = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json"}, method="POST")
    t0 = time.perf_counter()
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        out = json.loads(resp.read().decode())
    return (time.perf_counter() - t0) * 1000.0, out


def bench_embed(profile: dict, texts: list[str]) -> dict:
    url = profile["embed_url"].rstrip("/") + "/embeddings"
    model = profile["embed_model"]
    n = int(profile.get("iterations", 20))
    latencies: list[float] = []
    dims = 0
    for i in range(n):
        text = texts[i % len(texts)]
        ms, out = post_json(url, {"model": model, "input": text})
        latencies.append(ms)
        dims = len(out["data"][0]["embedding"])
    return {
        "dims": dims,
        "n": n,
        "p50_ms": round(pct(latencies, 0.50), 2),
        "p90_ms": round(pct(latencies, 0.90), 2),
        "p95_ms": round(pct(latencies, 0.95), 2),
        "p99_ms": round(pct(latencies, 0.99), 2),
        "mean_ms": round(statistics.mean(latencies), 2),
    }


def bench_rerank(profile: dict, query: str, documents: list[str], batch_size: int) -> dict:
    url = profile["rerank_url"].rstrip("/") + "/rerank"
    model = profile["rerank_model"]
    docs = documents[:batch_size] if batch_size < len(documents) else documents * ((batch_size // len(documents)) + 1)
    docs = docs[:batch_size]
    n = max(5, int(profile.get("iterations", 20)) // 4)
    latencies: list[float] = []
    top = 0.0
    for _ in range(n):
        ms, out = post_json(
            url,
            {"model": model, "query": query, "documents": docs, "top_n": min(5, len(docs))},
            timeout=120.0,
        )
        latencies.append(ms)
        r0 = out["results"][0]
        top = float(r0.get("relevance_score", r0.get("score", 0)))
    return {
        "batch_size": batch_size,
        "n": n,
        "top_score": top,
        "p50_ms": round(pct(latencies, 0.50), 2),
        "p95_ms": round(pct(latencies, 0.95), 2),
    }


def vram_mib(host: str, user: str, key: str) -> str:
    key = os.path.expanduser(key)
    cmd = [
        "ssh",
        "-i",
        key,
        "-o",
        "BatchMode=yes",
        f"{user}@{host}",
        "nvidia-smi --query-gpu=memory.used,memory.total --format=csv,noheader",
    ]
    try:
        return subprocess.check_output(cmd, text=True, stderr=subprocess.DEVNULL, timeout=15).strip()
    except Exception as e:
        return f"unavailable ({e})"


def bench_memory_search(queries: list[str]) -> list[dict]:
    gzmo = ROOT / "target" / "release" / "gzmo"
    if not gzmo.exists():
        gzmo = ROOT / "target" / "debug" / "gzmo"
    if not gzmo.exists():
        return [{"error": "gzmo binary missing"}]
    out = []
    for q in queries:
        t0 = time.perf_counter()
        proc = subprocess.run(
            [str(gzmo), "memory", "search", q, "--limit", "5", "--json", "--no-scratch"],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
            timeout=120,
            env={**os.environ, "GZMO_SESSION_ID": "retrieval-bench"},
        )
        ms = (time.perf_counter() - t0) * 1000.0
        hits = 0
        if proc.returncode == 0:
            try:
                hits = len(json.loads(proc.stdout).get("items") or [])
            except json.JSONDecodeError:
                pass
        out.append({"query": q, "wall_ms": round(ms, 2), "hits": hits, "ok": proc.returncode == 0})
    return out


def main() -> int:
    import argparse

    ap = argparse.ArgumentParser(description="VM200 retrieval benchmark")
    ap.add_argument("--profile", required=True, help="Path to profile JSON")
    ap.add_argument("--tag", default="", help="Run tag suffix")
    args = ap.parse_args()

    profile_path = Path(args.profile)
    if not profile_path.is_absolute():
        profile_path = BENCH_DIR / profile_path
    profile = json.loads(profile_path.read_text())
    if yaml and WORKLOADS.exists():
        workloads = yaml.safe_load(WORKLOADS.read_text())
    else:
        workloads = {
            "embed_texts": ["gzmo retrieval probe"],
            "rerank": {"query": "test", "documents": ["a", "b"]},
            "rerank_batch_sizes": [1, 40],
            "memory_search_queries": [],
        }

    ts = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
    run_id = f"{profile.get('name', 'run')}_{ts}"
    if args.tag:
        run_id += f"_{args.tag}"
    run_dir = BENCH_DIR / "runs" / run_id
    run_dir.mkdir(parents=True, exist_ok=True)

    summary = {
        "run_id": run_id,
        "profile": profile.get("name"),
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "embed": bench_embed(profile, workloads["embed_texts"]),
        "rerank": [
            bench_rerank(profile, workloads["rerank"]["query"], workloads["rerank"]["documents"], bs)
            for bs in workloads.get("rerank_batch_sizes", [1, 40])
        ],
        "vram": vram_mib(profile.get("vm_host", ""), profile.get("ssh_user", "maximilian"), profile.get("ssh_key", "~/.ssh/id_sidecar_proxmox")),
    }
    if workloads.get("memory_search_queries"):
        summary["memory_search"] = bench_memory_search(workloads["memory_search_queries"])

    (run_dir / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    (run_dir / "profile.json").write_text(json.dumps(profile, indent=2) + "\n")
    print(json.dumps(summary, indent=2))
    print(f"\n[OK] wrote {run_dir / 'summary.json'}")
    if summary["embed"].get("dims") != 1024:
        print("[FAIL] embed dims != 1024", file=sys.stderr)
        return 1
    for r in summary["rerank"]:
        if abs(r.get("top_score", 0)) < 1e-6:
            print(f"[FAIL] rerank batch {r['batch_size']} near-zero score", file=sys.stderr)
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
