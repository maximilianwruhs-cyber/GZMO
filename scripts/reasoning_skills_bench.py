#!/usr/bin/env python3
"""Benchmark all GZMO generative skills: thinking on vs off on Prime."""

from __future__ import annotations

import json
import time
import urllib.request
from dataclasses import dataclass

URL = "http://localhost:8000/v1/chat/completions"
MODEL = "gemma-4-26b-a4b-it"

CHAOS_MAX = 1265  # typical chaos cap from live snapshot
PROFILE_MAX = 4096


def contains_any(text: str, needles: list[str]) -> bool:
    lower = text.lower()
    return any(n.lower() in lower for n in needles)


def strip_thinking_channels(text: str) -> str:
    lines = []
    for line in text.splitlines():
        t = line.strip()
        if t.lower() in ("<|channel>thought", "<channel>thought"):
            continue
        for prefix in ("<|channel|>", "<channel|>"):
            if t.startswith(prefix):
                t = t[len(prefix) :].strip()
        if t:
            lines.append(t)
    return "\n".join(lines)


def visible_text(content: str, reasoning: str) -> str:
    return (content.strip() or reasoning).strip()


def line_value(text: str, prefix: str) -> str:
    for line in text.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :].strip()
    return ""


def quality_gate_poem(text: str) -> bool:
    return not contains_any(
        text,
        [
            "seele",
            "schicksal",
            "ewigkeit",
            "tränen",
            "tranen",
            "schatten",
            "flüstern",
            "flustern",
            " soul",
            "fate",
            "eternity",
            "whisper",
            "shadows",
            "tears",
            " dance",
        ],
    )


def quality_gate_joke(text: str) -> bool:
    return not contains_any(
        text,
        [
            "programmier",
            "programming bug",
            "coffee",
            "kaffee",
            "artificial intelligence",
            "chatgpt",
            "dad joke",
            "flachwitz",
            "montagmorgen",
            " wlan",
            " wifi",
            " bug",
        ],
    )


def quality_gate_story(text: str) -> bool:
    return not contains_any(
        text,
        [
            "once upon a time",
            "es war einmal",
            "happily ever after",
            "und sie lebten",
            "moral of the story",
            "lehre des",
            "märchen",
            "marchen",
        ],
    )


def quality_gate_word(text: str) -> bool:
    return text.splitlines() and any(l.startswith("WORD:") for l in text.splitlines()) and not contains_any(
        text,
        ["wordsmith", "neologism of the day", "made-up word:", "fake word:", "lorem ipsum"],
    )


def quality_gate_define(text: str) -> bool:
    return any(l.startswith("DEFINITION:") for l in text.splitlines()) and not contains_any(
        text,
        ["as an ai", "i don't know", "cannot define", "no definition", "lorem ipsum"],
    )


def quality_gate_card(text: str) -> bool:
    slop = ["as an ai", "i cannot", "placeholder", "lorem ipsum", "[card name]", "[mana cost", "[rules text"]
    if any(s in text.lower() for s in slop):
        return False
    for prefix in ["NAME:", "COST:", "TYPE:", "RARITY:", "RULES:"]:
        if not any(l.startswith(prefix) for l in text.splitlines()):
            return False
    if len(line_value(text, "NAME:")) < 3:
        return False
    if "{" not in line_value(text, "COST:"):
        return False
    if len(line_value(text, "RULES:")) < 4:
        return False
    return True


def quality_gate_pkm(text: str) -> bool:
    slop = ["as an ai", "i cannot", "placeholder", "lorem ipsum", "[card name]"]
    if any(s in text.lower() for s in slop):
        return False
    for prefix in ["NAME:", "CATEGORY:", "RARITY:"]:
        if not any(l.startswith(prefix) for l in text.splitlines()):
            return False
    if len(line_value(text, "NAME:")) < 3:
        return False
    for prefix in ["ELEMENT:", "HP:", "STAGE:", "ATTACK1:"]:
        if not any(l.startswith(prefix) for l in text.splitlines()):
            return False
    return True


def accept_creative(text: str, max_chars: int, gate) -> bool:
    n = len(text)
    return 0 < n <= max_chars and gate(text)


@dataclass
class SkillCase:
    name: str
    system: str
    user: str
    max_chars: int
    gate: object
    off_max_tokens: int


SKILLS: list[SkillCase] = [
    SkillCase(
        "joke",
        "You are a deadpan comedy engine. Dry observational humor. "
        "FORBIDDEN: programming bugs, coffee, AI jokes, dad jokes. "
        "Max 280 characters. Output ONLY the joke.",
        "Topic: office elevators\nAttractor: tick 25800, phase Idle, valence -0.28, invocation #42\n"
        "Structure: setup → misdirection → punchline. Must be original.",
        280,
        quality_gate_joke,
        512,
    ),
    SkillCase(
        "poem",
        "You are a contemporary poet. Concrete industrial textures: metal, rust, oil. "
        "Ban abstract words (soul, fate, eternity). Max 180 characters. Output ONLY the poem.",
        "Motif: rust\nAttractor: tick 25800, phase Idle, valence -0.28, invocation #42\n"
        "Use concrete sensory details only. No titles or commentary.",
        180,
        quality_gate_poem,
        256,
    ),
    SkillCase(
        "story",
        "You write short stories in Hemingway's sparse, concrete style. "
        "Short declarative sentences. Output ONLY story text.",
        "Keyword: lighthouse\nAttractor: tick 25800, phase Idle, valence -0.28, invocation #42\n"
        "Maximum 500 characters. Complete but unresolved arc. No title or quotes.",
        1000,
        quality_gate_story,
        768,
    ),
    SkillCase(
        "word",
        "You are a neologist. Invent a pronounceable industrial word. Output EXACTLY:\n"
        "WORD: [word] ([pronunciation])\nDEFINITION: [definition]\nEXAMPLE: [sentence]\nNo other text.",
        "Theme: gears\nAttractor: tick 25800, phase Idle, invocation #42",
        512,
        quality_gate_word,
        512,
    ),
    SkillCase(
        "define",
        "You are a scientific lexicographer. Output EXACTLY:\n"
        "WORD: [word]\nPRONUNCIATION: [IPA]\nPART OF SPEECH: [pos]\n"
        "DEFINITION: [definition]\nETYMOLOGY: [etymology]\nUSAGE: [sentence]\nNo other text.",
        "Term: attractor\nAttractor: tick 25800, phase Idle, invocation #42",
        800,
        quality_gate_define,
        1024,
    ),
    SkillCase(
        "card",
        "You are an MTG card designer. Output EXACTLY:\n"
        "NAME:\nCOST:\nTYPE:\nRARITY:\nRULES:\nFLAVOR:\nPT:",
        "Forge one original Uncommon red Instant for set GZM.\n"
        "Attractor: tick 25800, phase Idle, valence -0.28, rho 27.96, invocation #42",
        900,
        quality_gate_card,
        1024,
    ),
    SkillCase(
        "pkm",
        "You are a Pokemon TCG designer. Output EXACTLY:\n"
        "NAME:\nCATEGORY: Pokemon\nELEMENT:\nHP:\nSTAGE:\nRARITY:\nATTACK1:\nFLAVOR:",
        "Forge one original Rare electric Pokemon for set GZM.\n"
        "Attractor: tick 25800, phase Idle, invocation #42",
        900,
        quality_gate_pkm,
        1024,
    ),
]


def call(system: str, user: str, thinking: bool, max_tokens: int) -> dict:
    if thinking:
        reasoning_format = "auto"
        chat_template_kwargs = {"enable_thinking": True}
    else:
        reasoning_format = "none"
        chat_template_kwargs = {"enable_thinking": False}

    body = {
        "model": MODEL,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        "temperature": 0.7,
        "top_p": 0.95,
        "max_tokens": max_tokens,
        "reasoning_format": reasoning_format,
        "chat_template_kwargs": chat_template_kwargs,
    }
    started = time.time()
    req = urllib.request.Request(
        URL,
        data=json.dumps(body).encode(),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=180) as resp:
        data = json.loads(resp.read().decode())
    elapsed = time.time() - started

    choice = data["choices"][0]
    msg = choice["message"]
    content = msg.get("content") or ""
    reasoning = msg.get("reasoning_content") or ""
    usage = data.get("usage") or {}
    visible = visible_text(content, reasoning)
    cleaned = strip_thinking_channels(visible)

    return {
        "thinking": thinking,
        "max_tokens": max_tokens,
        "elapsed_s": round(elapsed, 2),
        "finish_reason": choice.get("finish_reason"),
        "completion_tokens": usage.get("completion_tokens", 0),
        "content_len": len(content),
        "reasoning_len": len(reasoning),
        "cleaned_len": len(cleaned),
        "cleaned": cleaned,
        "reasoning_preview": reasoning[:220].replace("\n", " "),
        "content_preview": content[:220].replace("\n", " "),
    }


def evaluate(skill: SkillCase, result: dict) -> tuple[bool, list[str]]:
    cleaned = result["cleaned"]
    issues: list[str] = []
    if not cleaned:
        issues.append("empty")
    if len(cleaned) > skill.max_chars:
        issues.append(f"too_long>{skill.max_chars}")
    if result["finish_reason"] == "length":
        issues.append("truncated")
    if result["reasoning_len"] > 0 and result["content_len"] == 0:
        issues.append("reasoning_only")
    if cleaned and not skill.gate(cleaned):
        issues.append("quality_gate")
    return (len(issues) == 0, issues)


def main() -> None:
    print(f"Generative skills benchmark — {MODEL} @ {URL}\n")
    rows = []
    for skill in SKILLS:
        print(f"=== {skill.name} (max_chars={skill.max_chars}) ===")
        for label, thinking, max_tok in [
            ("chaos_on", True, CHAOS_MAX),
            ("thinking_on_4k", True, PROFILE_MAX),
            ("thinking_off", False, skill.off_max_tokens),
        ]:
            try:
                r = call(skill.system, skill.user, thinking, max_tok)
                ok, issues = evaluate(skill, r)
                status = "PASS" if ok else "FAIL"
                print(
                    f"  [{label:16}] {status:4} {r['elapsed_s']:5.1f}s "
                    f"finish={r['finish_reason']:<6} out={r['completion_tokens']:4}tok "
                    f"content={r['content_len']:4} reasoning={r['reasoning_len']:5} "
                    f"clean={r['cleaned_len']:4}"
                )
                if issues:
                    print(f"    issues: {', '.join(issues)}")
                if r["reasoning_len"] > 0 and r["content_len"] == 0:
                    print(f"    reasoning: {r['reasoning_preview']}")
                if r["cleaned"]:
                    preview = r["cleaned"][:180].replace("\n", " | ")
                    print(f"    output: {preview}")
                rows.append((skill.name, label, ok, r["elapsed_s"], issues))
            except Exception as e:
                print(f"  [{label:16}] ERROR {e}")
                rows.append((skill.name, label, False, 0.0, [str(e)]))
        print()

    print("Summary")
    print("-" * 72)
    for skill_name in [s.name for s in SKILLS]:
        subset = [r for r in rows if r[0] == skill_name]
        chaos = next((r for r in subset if r[1] == "chaos_on"), None)
        off = next((r for r in subset if r[1] == "thinking_off"), None)
        chaos_s = "PASS" if chaos and chaos[2] else "FAIL"
        off_s = "PASS" if off and off[2] else "FAIL"
        print(f"  {skill_name:8} chaos_on={chaos_s:4}  thinking_off={off_s:4}")


if __name__ == "__main__":
    main()
