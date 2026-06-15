#!/usr/bin/env python3
"""Extract /dice event pools from dice.rs into data/dice_events.toml (legacy migration aid).

Source of truth after Chapter 4 is data/dice_events.toml — edit that file directly.
Re-run this only when porting from an old inline dice.rs snapshot.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DICE_RS = ROOT / "gzmo-core/src/skills/dice.rs"
OUT = ROOT / "data/dice_events.toml"


def extract_pools(src: str, fn_name: str) -> dict[str, dict]:
    m = re.search(
        rf"fn {fn_name}\(roll: u8, variant: usize\) -> String \{{\n    let pool: &\[&str\] = match roll \{{(.*?)\n        _ =>",
        src,
        re.S,
    )
    if not m:
        raise SystemExit(f"could not find {fn_name} in {DICE_RS}")
    body = m.group(1)
    tiers: dict[str, dict] = {}
    current_tier = ""
    for line in body.splitlines():
        tm = re.match(r"\s*// Tier (\d+): (.+)", line)
        if tm:
            current_tier = tm.group(2).strip()
            continue
        am = re.match(r"\s*(\d+) => &\[", line)
        if am:
            tiers[am.group(1)] = {"tier": current_tier, "variants": []}
            continue
        sm = re.match(r'\s*"((?:[^"\\]|\\.)*)",?\s*$', line)
        if sm and tiers:
            last = list(tiers.keys())[-1]
            tiers[last]["variants"].append(sm.group(1))
    return tiers


def write_toml(d20: dict, d6: dict, path: Path) -> None:
    lines = [
        "# GZMO /dice event corpus — 100 D20 + 18 D6 narrative lines",
        "# Edit here; embedded at compile time (gzmo-core/src/skills/dice_corpus.rs)",
        "# Regenerate from legacy inline dice.rs: scripts/generate-dice-events-toml.py",
        "",
        "[meta]",
        "version = 1",
        f"d20_tiers = {len(d20)}",
        f"d6_tiers = {len(d6)}",
        "",
    ]
    for roll in sorted(d20, key=int):
        t = d20[roll]
        lines.append(f'[d20."{roll}"]')
        lines.append(f'tier = "{t["tier"]}"')
        lines.append("variants = [")
        for v in t["variants"]:
            lines.append(f'  "{v.replace(chr(92), chr(92)*2).replace(chr(34), chr(92)+chr(34))}",')
        lines.append("]")
        lines.append("")
    for roll in sorted(d6, key=int):
        t = d6[roll]
        lines.append(f'[d6."{roll}"]')
        if t["tier"]:
            lines.append(f'tier = "{t["tier"]}"')
        lines.append("variants = [")
        for v in t["variants"]:
            lines.append(f'  "{v.replace(chr(92), chr(92)*2).replace(chr(34), chr(92)+chr(34))}",')
        lines.append("]")
        lines.append("")
    path.write_text("\n".join(lines))


def main() -> int:
    if not DICE_RS.exists():
        print(f"missing {DICE_RS}", file=sys.stderr)
        return 1
    src = DICE_RS.read_text()
    if "fn d20_event" not in src:
        print("dice.rs no longer has inline pools — edit data/dice_events.toml directly", file=sys.stderr)
        return 1
    d20 = extract_pools(src, "d20_event")
    d6 = extract_pools(src, "d6_event")
    OUT.parent.mkdir(parents=True, exist_ok=True)
    write_toml(d20, d6, OUT)
    n20 = sum(len(v["variants"]) for v in d20.values())
    n6 = sum(len(v["variants"]) for v in d6.values())
    print(f"wrote {OUT} ({n20}+{n6}={n20+n6} events)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
