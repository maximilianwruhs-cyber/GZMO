#!/usr/bin/env python3
"""Shared helpers for opportunity discovery scripts."""

from __future__ import annotations

import re
from pathlib import Path
from typing import Any

FRONTMATTER_RE = re.compile(r"^---\n(.*?)\n---\n", re.DOTALL)


def parse_frontmatter(text: str) -> dict[str, Any]:
    m = FRONTMATTER_RE.match(text)
    if not m:
        return {}
    data: dict[str, Any] = {}
    for line in m.group(1).splitlines():
        line = line.strip()
        if not line or line.startswith("#") or ":" not in line:
            continue
        key, val = line.split(":", 1)
        key = key.strip()
        val = val.strip().strip("'\"")
        if val.startswith("[") and val.endswith("]"):
            inner = val[1:-1].strip()
            data[key] = [x.strip().strip("'\"") for x in inner.split(",") if x.strip()] if inner else []
        elif val.isdigit():
            data[key] = int(val)
        elif val.lower() in ("true", "false", "yes", "no"):
            data[key] = val.lower() in ("true", "yes")
        else:
            data[key] = val
    return data


def load_bets(opportunities_dir: Path) -> list[dict[str, Any]]:
    bets: list[dict[str, Any]] = []
    for path in sorted(opportunities_dir.glob("*.md")):
        if path.name.upper() == "README.MD" or path.name == "README.md":
            continue
        text = path.read_text(encoding="utf-8")
        meta = parse_frontmatter(text)
        if not meta.get("id"):
            meta["id"] = path.stem
        meta["path"] = str(path)
        meta["_file"] = path.name
        bets.append(meta)
    return bets


def compute_score(bet: dict[str, Any]) -> int | None:
    if bet.get("status") == "horizon":
        return None
    if "score" in bet and isinstance(bet["score"], int):
        return bet["score"]
    keys = ("uniqueness", "brain_profit", "credit_cost", "attention_cost", "usp_fit")
    if all(isinstance(bet.get(k), int) for k in keys):
        return sum(int(bet[k]) for k in keys)
    return None


def ship_bar(bet: dict[str, Any]) -> bool:
    """True when a bet is eligible to recommend as next craft (not parked/horizon)."""
    if bet.get("status") == "horizon":
        return False
    parked = bet.get("parked")
    if parked is True or (isinstance(parked, str) and parked.lower() in ("true", "yes", "1")):
        return False
    score = compute_score(bet)
    if score is None:
        return False
    return (
        score >= 18
        and int(bet.get("brain_profit") or 0) >= 3
        and int(bet.get("usp_fit") or 0) >= 4
    )


def shipable_priority(bet_row: dict[str, Any]) -> tuple:
    """Prefer active soak/mission bets over candidates when ranking shipable."""
    status = bet_row.get("status") or ""
    # active first (0), then candidate (1), else last
    status_rank = 0 if status == "active" else (1 if status == "candidate" else 2)
    score = int(bet_row.get("score") or 0)
    return (status_rank, -score, bet_row.get("id") or "")
