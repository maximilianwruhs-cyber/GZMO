#!/usr/bin/env python3
"""Summarize AttractorBench MAS comparison JSON (text vs latent bridge)."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


def load(path: Path) -> list[dict[str, Any]]:
    data = json.loads(path.read_text())
    return data if isinstance(data, list) else [data]


def summarize(file: Path) -> dict[str, dict[str, float]]:
    rows: dict[str, dict[str, float]] = {}
    for block in load(file):
        mode = block.get("mode", "unknown")
        agg = block.get("aggregate", {})
        rows[mode] = {
            "latency_ms": float(agg.get("latency_ms", [0])[0]),
            "total_tokens": float(agg.get("total_tokens", [0])[0]),
            "success_rate": float(agg.get("success_rate", [0])[0]),
        }
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("text_baseline", type=Path, help="mas_text_baseline.json")
    parser.add_argument("--latent", type=Path, help="mas_latent_compare.json")
    args = parser.parse_args()

    text = summarize(args.text_baseline)
    print(f"## Text-based MAS baseline ({args.text_baseline.name})\n")
    for mode, m in sorted(text.items()):
        print(
            f"- **{mode}**: latency={m['latency_ms']:.0f}ms, "
            f"tokens={m['total_tokens']:.0f}, success={m['success_rate']*100:.0f}%"
        )

    if not args.latent:
        return 0

    latent = summarize(args.latent)
    print(f"\n## Latent bridge run ({args.latent.name})\n")
    for mode, m in sorted(latent.items()):
        print(
            f"- **{mode}**: latency={m['latency_ms']:.0f}ms, "
            f"tokens={m['total_tokens']:.0f}, success={m['success_rate']*100:.0f}%"
        )

    if "single" in text and "two_agent" in text:
        overhead = text["two_agent"]["total_tokens"] - text["single"]["total_tokens"]
        print(f"\n**Handoff token overhead (2-agent vs single)**: {overhead:.0f} tokens")

    if "recursive_mas" in latent and "two_agent" in latent:
        speed = latent["two_agent"]["latency_ms"] / max(latent["recursive_mas"]["latency_ms"], 1)
        print(f"\n**Latency ratio (two_agent / recursive_mas)**: {speed:.2f}x")

    return 0


if __name__ == "__main__":
    sys.exit(main())
