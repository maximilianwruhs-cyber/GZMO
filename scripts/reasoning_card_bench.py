#!/usr/bin/env python3
"""Compare thinking on vs off for card-forge style prompts on Prime."""

from __future__ import annotations

import json
import re
import urllib.request

URL = "http://localhost:8000/v1/chat/completions"
MODEL = "gemma-4-26b-a4b-it"

SYSTEM = """You are a Magic: The Gathering card designer.
OUTPUT FORMAT (exactly this, no other text):
NAME: [card name]
COST: [mana cost like {2}{R}]
TYPE: [full type line]
RARITY: [Common|Uncommon|Rare|Mythic]
RULES: [rules text]
FLAVOR: [flavor text, max 2 sentences]
PT: [Power/Toughness or NONE]"""

USER = """Forge one original Uncommon red Instant for set GZM.
Attractor state: tick 25800, phase Idle, valence -0.28, rho 27.96, invocation #42.
Make it memorable, mechanically coherent, and worthy of the set."""


def validate_card(text: str, requires_pt: bool = False) -> tuple[bool, list[str]]:
    issues: list[str] = []
    slop = ["as an ai", "i cannot", "placeholder", "lorem ipsum", "[card name]"]
    lower = text.lower()
    for s in slop:
        if s in lower:
            issues.append(f"slop:{s}")
    for prefix in ["NAME:", "COST:", "TYPE:", "RARITY:", "RULES:"]:
        if not any(line.startswith(prefix) for line in text.splitlines()):
            issues.append(f"missing:{prefix}")
    name = next((l.split(":", 1)[1].strip() for l in text.splitlines() if l.startswith("NAME:")), "")
    if len(name) < 3:
        issues.append("name_too_short")
    cost = next((l.split(":", 1)[1].strip() for l in text.splitlines() if l.startswith("COST:")), "")
    if "{" not in cost:
        issues.append("cost_invalid")
    rules = next((l.split(":", 1)[1].strip() for l in text.splitlines() if l.startswith("RULES:")), "")
    if len(rules) < 4:
        issues.append("rules_too_short")
    if requires_pt:
        pt = next((l.split(":", 1)[1].strip() for l in text.splitlines() if l.startswith("PT:")), "")
        if not any(c.isdigit() for c in pt):
            issues.append("pt_invalid")
    return (len(issues) == 0, issues)


def strip_thinking_channels(text: str) -> str:
  lines = []
  for line in text.splitlines():
    t = line.strip()
    if t.lower() in ("<|channel>thought", "<channel>thought"):
      continue
    for prefix in ("<|channel|>", "<channel|>"):
      if t.startswith(prefix):
        t = t[len(prefix):].strip()
    if t:
      lines.append(t)
  return "\n".join(lines)


def call(thinking: bool, max_tokens: int) -> dict:
    if thinking:
        reasoning_format = "auto"
        chat_template_kwargs = {"enable_thinking": True}
        label = "thinking_on"
    else:
        reasoning_format = "none"
        chat_template_kwargs = {"enable_thinking": False}
        label = "thinking_off"

    body = {
        "model": MODEL,
        "messages": [
            {"role": "system", "content": SYSTEM},
            {"role": "user", "content": USER},
        ],
        "temperature": 0.7,
        "top_p": 0.95,
        "max_tokens": max_tokens,
        "reasoning_format": reasoning_format,
        "chat_template_kwargs": chat_template_kwargs,
    }
    req = urllib.request.Request(
        URL,
        data=json.dumps(body).encode(),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=180) as resp:
        data = json.loads(resp.read().decode())

    choice = data["choices"][0]
    msg = choice["message"]
    content = msg.get("content") or ""
    reasoning = msg.get("reasoning_content") or ""
    usage = data.get("usage") or {}

  # Mirror gzmo gateway: prefer content, else reasoning
    visible = content.strip() or reasoning
    cleaned = strip_thinking_channels(visible)

    ok, issues = validate_card(cleaned)
    return {
        "label": label,
        "max_tokens": max_tokens,
        "finish_reason": choice.get("finish_reason"),
        "prompt_tokens": usage.get("prompt_tokens"),
        "completion_tokens": usage.get("completion_tokens"),
        "content_len": len(content),
        "reasoning_len": len(reasoning),
        "cleaned_len": len(cleaned),
        "valid_after_clean": ok,
        "issues": issues,
        "content_preview": content[:400].replace("\n", "\\n"),
        "reasoning_preview": reasoning[:400].replace("\n", "\\n"),
        "cleaned_preview": cleaned[:500].replace("\n", "\\n"),
    }


def main() -> None:
    cases = [
        (True, 1265),
        (True, 2048),
        (True, 4096),
        (False, 512),
        (False, 1024),
    ]
    print("Card-forge reasoning benchmark (Prime @ :8000)\n")
    for thinking, max_tokens in cases:
        try:
            r = call(thinking, max_tokens)
        except Exception as e:
            print(f"FAIL thinking={thinking} max_tokens={max_tokens}: {e}\n")
            continue
        print(
            f"[{r['label']} max={r['max_tokens']}] "
            f"finish={r['finish_reason']} "
            f"out={r['completion_tokens']}tok "
            f"content={r['content_len']}ch reasoning={r['reasoning_len']}ch "
            f"valid={r['valid_after_clean']}"
        )
        if r["issues"]:
            print(f"  issues: {', '.join(r['issues'])}")
        if r["reasoning_len"] > 0:
            print(f"  reasoning: {r['reasoning_preview']}")
        print(f"  cleaned: {r['cleaned_preview']}")
        print()


if __name__ == "__main__":
    main()
