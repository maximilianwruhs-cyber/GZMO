#!/usr/bin/env python3
"""Merge calibration fields from *-fused.toml into live GZMO-next config.

Full-file copy is unsafe for gzmo-next.toml: fuse emits engine/bench snippets only.
This merges inference defaults, engine.local sampling, chaos, and benchmark
provenance — never clobbering [assembly]/[memory]/ etc.
"""
from __future__ import annotations

import argparse
import re
import shutil
import sys
from datetime import date
from pathlib import Path


def parse_simple_toml(text: str) -> dict[str, dict[str, str]]:
    """Parse flat section → key=value (strings/numbers/bools as raw strings)."""
    sections: dict[str, dict[str, str]] = {}
    current: str | None = None
    for line in text.splitlines():
        s = line.strip()
        if not s or s.startswith("#"):
            continue
        m = re.match(r"^\[([^\]]+)\]$", s)
        if m:
            current = m.group(1)
            sections.setdefault(current, {})
            continue
        if current is None or "=" not in s:
            continue
        key, _, val = s.partition("=")
        sections[current][key.strip()] = val.strip()
    return sections


def set_key_in_section(text: str, section: str, key: str, value: str) -> str:
    """Set key=value inside [section]; insert section before [agent] if missing."""
    header = f"[{section}]"
    lines = text.splitlines(keepends=True)
    start = None
    for i, line in enumerate(lines):
        if line.strip() == header:
            start = i
            break
    if start is None:
        # Insert before [agent] or at end
        insert_at = len(lines)
        for i, line in enumerate(lines):
            if line.strip() == "[agent]":
                insert_at = i
                break
        block = f"\n{header}\n{key} = {value}\n"
        return "".join(lines[:insert_at]) + block + "".join(lines[insert_at:])

    end = len(lines)
    for j in range(start + 1, len(lines)):
        if re.match(r"^\[[^\]]+\]\s*$", lines[j].strip()):
            end = j
            break

    key_re = re.compile(rf"^(\s*){re.escape(key)}\s*=")
    for j in range(start + 1, end):
        if key_re.match(lines[j]):
            indent = key_re.match(lines[j]).group(1)
            lines[j] = f"{indent}{key} = {value}\n"
            return "".join(lines)

    # append key before next section
    lines.insert(end, f"{key} = {value}\n")
    return "".join(lines)


def replace_or_insert_section(text: str, section: str, body: str) -> str:
    """Replace entire [section] body or insert before [agent]."""
    header = f"[{section}]"
    pattern = re.compile(
        rf"(?ms)^\[{re.escape(section)}\]\n.*?(?=^\[|\Z)"
    )
    block = f"{header}\n{body.rstrip()}\n\n"
    if pattern.search(text):
        return pattern.sub(block, text, count=1)
    insert_at = text.find("\n[agent]")
    if insert_at == -1:
        return text.rstrip() + "\n\n" + block
    return text[: insert_at + 1] + block + text[insert_at + 1 :]


def merge(live_path: Path, fused_path: Path, *, dry_run: bool) -> list[str]:
    live_text = live_path.read_text(encoding="utf-8")
    fused = parse_simple_toml(fused_path.read_text(encoding="utf-8"))
    actions: list[str] = []

    if "[assembly]" not in live_text and "[engine.local]" not in live_text:
        raise SystemExit(
            "live config does not look like GZMO-next (no [assembly]/[engine.local]). "
            "Refuse merge — use careful full-file review for legacy configs."
        )

    inf = fused.get("inference.defaults", {})
    if "temperature" in inf:
        live_text = set_key_in_section(live_text, "inference.defaults", "temperature", inf["temperature"])
        live_text = set_key_in_section(live_text, "engine.local", "temperature", inf["temperature"])
        actions.append(f"inference/engine.local temperature ← {inf['temperature']}")
    if "top_p" in inf:
        live_text = set_key_in_section(live_text, "inference.defaults", "top_p", inf["top_p"])
        live_text = set_key_in_section(live_text, "engine.local", "top_p", inf["top_p"])
        actions.append(f"inference/engine.local top_p ← {inf['top_p']}")

    # Lorenz exploration temp must NOT overwrite max_tokens or wipe active_mode.
    eng = fused.get("engine", {})
    if "lorenz_seed" in eng:
        live_text = set_key_in_section(live_text, "engine", "lorenz_seed", eng["lorenz_seed"])
        actions.append(f"engine.lorenz_seed ← {eng['lorenz_seed']}")
    if "strategy" in eng:
        live_text = set_key_in_section(live_text, "engine", "strategy", eng["strategy"])
        actions.append(f"engine.strategy ← {eng['strategy']}")

    chaos = fused.get("chaos", {})
    for key in ("bpm_default", "dt_lorenz"):
        if key in chaos:
            live_text = set_key_in_section(live_text, "chaos", key, chaos[key])
            actions.append(f"chaos.{key} ← {chaos[key]}")

    bf = fused.get("benchmark.fused", {})
    if bf:
        body = "\n".join(f"{k} = {v}" for k, v in bf.items())
        live_text = replace_or_insert_section(live_text, "benchmark.fused", body)
        actions.append("upsert [benchmark.fused]")

    bg = fused.get("benchmark.gate", {})
    if bg:
        body = "\n".join(f"{k} = {v}" for k, v in bg.items())
        live_text = replace_or_insert_section(live_text, "benchmark.gate", body)
        actions.append("upsert [benchmark.gate]")

    # Refresh calibration banner comment if present
    stamp = date.today().isoformat()
    live_text = re.sub(
        r"Calibrated \d{4}-\d{2}-\d{2}:",
        f"Calibrated {stamp}:",
        live_text,
        count=1,
    )

    rapl = fused.get("routing.rapl", {})
    if rapl.get("fits_budget") == "false":
        actions.append("skip [routing.rapl] (fits_budget=false)")
    if "max_tokens" in eng:
        actions.append(f"skip fused engine.max_tokens={eng['max_tokens']} (do not clobber engine.local)")

    if dry_run:
        print("DRY-RUN actions:")
        for a in actions:
            print(f"  - {a}")
        return actions

    backup = live_path.with_name(live_path.name + ".bak-promote")
    shutil.copy2(live_path, backup)
    live_path.write_text(live_text, encoding="utf-8")
    print(f"backup: {backup}")
    print(f"merged {fused_path} → {live_path}")
    for a in actions:
        print(f"  - {a}")
    return actions


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--live", type=Path, required=True)
    ap.add_argument("--fused", type=Path, required=True)
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()
    if not args.live.is_file() or not args.fused.is_file():
        print("live/fused missing", file=sys.stderr)
        return 2
    merge(args.live, args.fused, dry_run=args.dry_run)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
