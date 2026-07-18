#!/usr/bin/env python3
"""Nightburst Arena spike: recall + distill tasks → data-next/arena/latest.json."""

from __future__ import annotations

import json
import os
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def main() -> None:
    root = Path(os.environ.get("GZMO_CLONE_ROOT", Path.home() / "github-clone")) / "GZMO"
    if "GZMO_CONFIG" in os.environ:
        # Prefer explicit config parent layout
        cfg = Path(os.environ["GZMO_CONFIG"]).resolve()
        if cfg.name.endswith(".toml"):
            pass
    gzmo = os.environ.get(
        "GZMO_BIN",
        str(
            Path(os.environ.get("CARGO_TARGET_DIR", Path.home() / "github-clone/temp-bench/target"))
            / "release"
            / "gzmo"
        ),
    )
    out_dir = Path(os.environ.get("ARENA_OUT_DIR", root / "data-next" / "arena"))
    out_dir.mkdir(parents=True, exist_ok=True)
    engine = os.environ.get("ARENA_ENGINE_LABEL", "prime-local")

    env = os.environ.copy()
    env.setdefault("GZMO_INSTANCE", "next")
    env.setdefault("GZMO_CONFIG", str(root / "config" / "gzmo.toml"))
    env.setdefault("GZMO_ALLOW_LAB_VAULT", "1")

    tasks_spec = [
        ("Quillhorn Cascade", "Quillhorn"),
        ("Meridian-Vesper", "Meridian-Vesper"),
        ("Obsidian-Pip", "Obsidian-Pip"),
        ("Amberglass Ticket-9", "Amberglass"),
        ("Lumen-Attic", "Lumen-Attic"),
        ("Copperfinch-Omega", "Copperfinch"),
    ]

    started = utc_now()
    t0 = time.perf_counter()
    tasks: list[dict] = []
    hits = 0

    for query, needle in tasks_spec:
        t_start = time.perf_counter()
        proc = subprocess.run(
            [gzmo, "memory", "search", query, "--limit", "5", "--no-scratch"],
            capture_output=True,
            text=True,
            env=env,
            check=False,
        )
        ms = int((time.perf_counter() - t_start) * 1000)
        text = (proc.stdout or "") + (proc.stderr or "")
        hit = needle.lower() in text.lower() and proc.returncode == 0
        if hit:
            hits += 1
        tasks.append(
            {
                "task": "recall",
                "query": query,
                "hit": hit,
                "duration_ms": ms,
                "exit": proc.returncode,
            }
        )

    session = root / "data-next" / "sessions" / "burst-proof-c1.json"
    distill_hit = False
    distill_ms = 0
    if session.is_file():
        t_start = time.perf_counter()
        proc = subprocess.run(
            [gzmo, "distill", "burst-proof-c1"],
            capture_output=True,
            text=True,
            env=env,
            check=False,
        )
        distill_ms = int((time.perf_counter() - t_start) * 1000)
        distill_hit = proc.returncode == 0
        if distill_hit:
            hits += 1
        tasks.append(
            {
                "task": "distill",
                "session": "burst-proof-c1",
                "hit": distill_hit,
                "duration_ms": distill_ms,
                "exit": proc.returncode,
            }
        )

    elapsed_ms = int((time.perf_counter() - t0) * 1000)
    finished = utc_now()
    total = len(tasks)
    quality = round(hits / max(total, 1), 4)
    efficiency = round(max(0.0, min(1.0, 1.0 - (elapsed_ms / 120_000.0))), 4)
    z = round(quality * efficiency, 4)

    payload = {
        "schema": "gzmo.arena.nightburst/v1",
        "engine": engine,
        "started": started,
        "finished": finished,
        "elapsed_ms": elapsed_ms,
        "energy_proxy": "duration_ms",
        "joules": None,
        "quality": quality,
        "efficiency": efficiency,
        "z": z,
        "hits": hits,
        "total": total,
        "tasks": tasks,
        "champion": engine,
        "suggestion": {
            "note": "Sibling suggestion only — human promote-fused; do not auto-overwrite gzmo-next.toml",
            "path": "data-next/arena/champion-suggestion.toml",
        },
    }

    (out_dir / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    (out_dir / "champion-suggestion.toml").write_text(
        (
            "# Arena champion suggestion (sibling — not live config)\n"
            f"# Generated {finished}\n"
            "# Review then manually merge engine prefs if desired.\n\n"
            "[arena.champion]\n"
            f'engine_label = "{engine}"\n'
            f"z = {z}\n"
            f"quality = {quality}\n"
            f"efficiency = {efficiency}\n"
            f"elapsed_ms = {elapsed_ms}\n"
        ),
        encoding="utf-8",
    )
    print(json.dumps({"champion": engine, "z": z, "quality": quality, "elapsed_ms": elapsed_ms}, indent=2))
    print(f"Wrote {out_dir / 'latest.json'}")


if __name__ == "__main__":
    main()
