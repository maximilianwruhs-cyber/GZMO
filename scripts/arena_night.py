#!/usr/bin/env python3
"""Nightburst Arena spike: recall + distill → data-next/arena/latest.json (+ RAPL/€)."""

from __future__ import annotations

import json
import os
import subprocess
import time
import urllib.request
from datetime import datetime, timezone
from pathlib import Path


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


class EnergyMeter:
    """Minimal RAPL meter (Obolus-compatible); falls back to duration-only."""

    RAPL_PATHS = [
        Path("/sys/class/powercap/intel-rapl:0/energy_uj"),
        Path("/sys/class/powercap/intel-rapl:1/energy_uj"),
    ]

    def __init__(self) -> None:
        self.active = [p for p in self.RAPL_PATHS if p.exists() and os.access(p, os.R_OK)]
        self._start_uj: dict[str, int] = {}
        self._start_mono = 0.0

    def start(self) -> None:
        self._start_uj = self._read()
        self._start_mono = time.monotonic()

    def _read(self) -> dict[str, int]:
        out: dict[str, int] = {}
        for p in self.active:
            try:
                out[str(p)] = int(p.read_text().strip())
            except (OSError, ValueError):
                out[str(p)] = 0
        return out

    def stop(self) -> dict:
        elapsed = max(0.001, time.monotonic() - self._start_mono)
        if self.active and self._start_uj:
            end = self._read()
            total_uj = 0
            for path, end_val in end.items():
                start_val = self._start_uj.get(path, 0)
                if end_val >= start_val:
                    total_uj += end_val - start_val
                else:
                    total_uj += (2**32 - start_val) + end_val
            joules = total_uj / 1_000_000.0
            return {
                "joules": round(joules, 4),
                "watts_avg": round(joules / elapsed, 2),
                "elapsed_s": round(elapsed, 3),
                "source": "rapl",
            }
        # Fallback estimate (Obolus-compatible): assume ~65W package while busy.
        watts = float(os.environ.get("ARENA_ESTIMATE_WATTS", "65"))
        joules = watts * elapsed
        return {
            "joules": round(joules, 4),
            "watts_avg": round(watts, 2),
            "elapsed_s": round(elapsed, 3),
            "source": "estimate",
        }


def fetch_awattar_c_kwh() -> tuple[float | None, bool]:
    """Return (¢/kWh, live). Uses OBULUS_ELECTRICITY_C_KWH / ARENA_ELECTRICITY_C_KWH as fallback."""
    default = os.environ.get("ARENA_ELECTRICITY_C_KWH") or os.environ.get("OBULUS_ELECTRICITY_C_KWH")
    default_f = float(default) if default else 15.0
    source = os.environ.get("OBULUS_PRICE_SOURCE", os.environ.get("ARENA_PRICE_SOURCE", "awattar"))
    if source != "awattar":
        return default_f, False
    try:
        with urllib.request.urlopen("https://api.awattar.at/v1/marketdata", timeout=5) as resp:
            data = json.loads(resp.read().decode())
        now_ms = datetime.now(timezone.utc).timestamp() * 1000
        for entry in data.get("data", []):
            if entry["start_timestamp"] <= now_ms <= entry["end_timestamp"]:
                return entry["marketprice"] / 10.0, True
        if data.get("data"):
            return data["data"][-1]["marketprice"] / 10.0, True
    except Exception:
        pass
    return default_f, False


def joules_to_eur(joules: float, price_c_kwh: float) -> float:
    kwh = joules / 3_600_000.0
    return round(kwh * price_c_kwh / 100.0, 8)


def main() -> None:
    root = Path(os.environ.get("GZMO_CLONE_ROOT", Path.home() / "github-clone")) / "GZMO"
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

    meter = EnergyMeter()
    price_c, price_live = fetch_awattar_c_kwh()

    started = utc_now()
    meter.start()
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
    energy = meter.stop()
    finished = utc_now()
    total = len(tasks)
    quality = round(hits / max(total, 1), 4)

    joules = energy.get("joules")
    if joules is not None:
        # Soft efficiency curve — stays informative for estimate (~65W) and RAPL.
        # budget_j default covers ~30s @ 65W; override with ARENA_JOULE_BUDGET.
        budget_j = float(os.environ.get("ARENA_JOULE_BUDGET", "2000"))
        efficiency = round(max(0.0, min(1.0, 1.0 / (1.0 + float(joules) / budget_j))), 4)
        energy_proxy = "rapl_joules" if energy.get("source") == "rapl" else "estimate_joules"
        euro = joules_to_eur(float(joules), price_c) if price_c is not None else None
    else:
        efficiency = round(max(0.0, min(1.0, 1.0 - (elapsed_ms / 120_000.0))), 4)
        energy_proxy = "duration_ms"
        euro = None

    z = round(quality * efficiency, 4)

    payload = {
        "schema": "gzmo.arena.nightburst/v1",
        "engine": engine,
        "started": started,
        "finished": finished,
        "elapsed_ms": elapsed_ms,
        "energy_proxy": energy_proxy,
        "joules": joules,
        "watts_avg": energy.get("watts_avg"),
        "energy_source": energy.get("source"),
        "electricity_c_kwh": price_c,
        "electricity_live": price_live,
        "euro_cost": euro,
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
    # Append-only history for €/night aggregate (never overwrite siblings).
    hist = out_dir / "history"
    hist.mkdir(parents=True, exist_ok=True)
    stamp = finished.replace(":", "").replace("-", "")[:15]
    (hist / f"arena-{stamp}.json").write_text(
        json.dumps(payload, indent=2) + "\n", encoding="utf-8"
    )
    euro_line = f"euro_cost = {euro}\n" if euro is not None else "euro_cost = # n/a\n"
    joules_line = f"joules = {joules}\n" if joules is not None else "joules = # n/a\n"
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
            f"{joules_line}"
            f"electricity_c_kwh = {price_c}\n"
            f"{euro_line}"
        ),
        encoding="utf-8",
    )
    print(
        json.dumps(
            {
                "champion": engine,
                "z": z,
                "quality": quality,
                "elapsed_ms": elapsed_ms,
                "joules": joules,
                "euro_cost": euro,
                "electricity_c_kwh": price_c,
                "electricity_live": price_live,
                "energy_source": energy.get("source"),
            },
            indent=2,
        )
    )
    print(f"Wrote {out_dir / 'latest.json'}")


if __name__ == "__main__":
    main()
