#!/usr/bin/env python3
"""Ingest chaos.rho_telemetry from Synapse JSONL → multivariate time series."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np

COLUMNS = ("rho_mod", "rho_velocity_ema", "rho_mod_delta", "rho_effective")


def load_synapse(path: Path) -> dict[str, np.ndarray]:
    ticks: list[int] = []
    rows: list[dict[str, float]] = []
    with path.open() as f:
        for line in f:
            ev = json.loads(line)
            if ev.get("event_type") != "chaos.rho_telemetry":
                continue
            d = ev["data"]
            ticks.append(int(d["tick"]))
            rows.append({c: float(d.get(c, 0.0)) for c in COLUMNS})

    if not rows:
        raise ValueError(f"No chaos.rho_telemetry events in {path}")

    tick = np.array(ticks, dtype=np.int64)
    data = {c: np.array([r[c] for r in rows], dtype=np.float64) for c in COLUMNS}
    data["tick"] = tick
    return data


def summary(data: dict[str, np.ndarray]) -> str:
    n = len(data["tick"])
    span_ticks = int(data["tick"][-1] - data["tick"][0])
    dt_ticks = int(np.median(np.diff(data["tick"]))) if n > 1 else 15
    lines = [
        f"samples={n}",
        f"tick_span={span_ticks} (Δt≈{dt_ticks} ticks/event)",
        f"rho_mod range=[{data['rho_mod'].min():.4f}, {data['rho_mod'].max():.4f}]",
    ]
    return "\n".join(lines)


def main() -> None:
    root = Path(__file__).resolve().parents[2]
    path = root / "data" / "Synapse" / "events.jsonl"
    if len(sys.argv) > 1:
        path = Path(sys.argv[1])
    data = load_synapse(path)
    print(summary(data))


if __name__ == "__main__":
    main()
