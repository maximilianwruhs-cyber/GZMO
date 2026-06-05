"""Shared LLM + text helpers for the M4 eval harness.

Used by faithfulness-judge.py and validate-golden-facts.py so retrieval
faithfulness and golden-fact validation apply the same grounding standard as
the Rust ingest verify gate (gzmo-core/src/memory/kg_extract.rs).
"""

from __future__ import annotations

import json
import os
import re
import unicodedata
import urllib.error
import urllib.request
from difflib import SequenceMatcher
from pathlib import Path
from typing import Any

# Mirrors the verify() system rules in gzmo-core/src/memory/kg_extract.rs so a
# claim judged "supported" here would also pass the ingest verify gate.
VERIFY_RULES = (
    "Rules:\n"
    "1. supported = true ONLY if the SOURCE explicitly states or unambiguously implies the claim.\n"
    "2. Put the exact supporting quote in 'evidence' (at least 12 characters). Empty if unsupported.\n"
    "3. confidence 0.0-1.0 — unsupported => near 0.\n"
    "4. Do NOT use outside knowledge.\n"
    "5. One verdict per candidate index."
)

VERDICTS_SCHEMA = {
    "type": "object",
    "properties": {
        "verdicts": {
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "index": {"type": "integer"},
                    "supported": {"type": "boolean"},
                    "confidence": {"type": "number"},
                    "evidence": {"type": "string"},
                },
                "required": ["index", "supported", "confidence", "evidence"],
                "additionalProperties": False,
            },
        }
    },
    "required": ["verdicts"],
    "additionalProperties": False,
}


def parse_engine_local(project_root: Path) -> tuple[str, str]:
    """Resolve (url, model) from gzmo.toml [engine.local], env-overridable."""
    toml_path = project_root / "gzmo.toml"
    url = os.environ.get("JUDGE_URL", "http://localhost:8000/v1")
    model = os.environ.get("JUDGE_MODEL", "qwen3.6-35b-mtp")
    if not toml_path.is_file():
        return url.rstrip("/"), model
    text = toml_path.read_text(encoding="utf-8")
    block = re.search(r"\[engine\.local\](.*?)(?:\n\[|\Z)", text, re.S)
    if block:
        section = block.group(1)
        m_url = re.search(r'^url\s*=\s*"([^"]+)"', section, re.M)
        m_model = re.search(r'^model\s*=\s*"([^"]+)"', section, re.M)
        if m_url:
            url = m_url.group(1)
        if m_model:
            model = m_model.group(1)
    return url.rstrip("/"), model


def normalize_ws(text: str) -> str:
    return " ".join((text or "").split())


def normalize_quote(text: str) -> str:
    """NFKC + lowercase + whitespace collapse for tolerant quote matching."""
    norm = unicodedata.normalize("NFKC", text or "")
    return " ".join(norm.lower().split())


def evidence_in_source(evidence: str, source: str, min_ratio: float = 0.92) -> bool:
    """True if the quote appears in source, tolerant to whitespace/unicode.

    Falls back to a fuzzy contiguous-match ratio so minor reformatting (smart
    quotes, collapsed spaces) does not reject an otherwise valid quote.
    """
    ev = normalize_quote(evidence)
    src = normalize_quote(source)
    if not ev:
        return False
    if ev in src:
        return True
    # Fuzzy: best contiguous block over a window the size of the evidence.
    if len(ev) < 12 or not src:
        return False
    matcher = SequenceMatcher(None, ev, src, autojunk=False)
    block = matcher.find_longest_match(0, len(ev), 0, len(src))
    if block.size == 0:
        return False
    return (block.size / len(ev)) >= min_ratio


def extract_snippet(source: str, query: str, fact: str, window: int = 4000) -> str:
    """Return a focused window of source around the best query/fact anchor.

    Avoids the "lost in the middle" problem of feeding the first N chars only;
    centers the window on the strongest keyword overlap (LongMemEval-style
    evidence focusing). Falls back to the head of the document.
    """
    if not source:
        return ""
    if len(source) <= window:
        return source
    src_low = source.lower()
    anchors: list[str] = []
    for text in (fact, query):
        for tok in re.findall(r"\w{4,}", (text or "").lower()):
            anchors.append(tok)
    best_pos = -1
    for tok in anchors:
        pos = src_low.find(tok)
        if pos != -1:
            best_pos = pos
            break
    if best_pos == -1:
        return source[:window] + "\n[TRUNCATED]\n"
    start = max(0, best_pos - window // 2)
    end = min(len(source), start + window)
    prefix = "" if start == 0 else "[...]\n"
    suffix = "" if end == len(source) else "\n[...]"
    return prefix + source[start:end] + suffix


def _http_post_json(url: str, body: dict, timeout_s: int) -> dict:
    req = urllib.request.Request(
        f"{url}/chat/completions",
        data=json.dumps(body).encode(),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=timeout_s) as resp:
        return json.loads(resp.read().decode())


def chat_verdicts(
    url: str,
    model: str,
    system: str,
    user: str,
    temperature: float,
    timeout_s: int,
    max_tokens: int = 2048,
    retries: int = 3,
) -> list[dict[str, Any]]:
    """Call the chat endpoint and return parsed `verdicts`.

    Tries strict json_schema first; on transport or parse failure retries, then
    falls back to json_object mode. Raises the last error if all attempts fail.
    """
    base_messages = [
        {"role": "system", "content": system},
        {"role": "user", "content": user},
    ]
    last_err: Exception | None = None
    for attempt in range(retries):
        use_schema = attempt < retries - 1
        if use_schema:
            response_format: dict = {
                "type": "json_schema",
                "json_schema": {
                    "name": "verdicts",
                    "strict": True,
                    "schema": VERDICTS_SCHEMA,
                },
            }
            messages = base_messages
        else:
            response_format = {"type": "json_object"}
            messages = [
                base_messages[0],
                {
                    "role": "user",
                    "content": user
                    + '\n\nReturn ONLY JSON: {"verdicts":[{"index":int,'
                    '"supported":bool,"confidence":float,"evidence":str}]}',
                },
            ]
        body = {
            "model": model,
            "messages": messages,
            "temperature": temperature,
            "max_tokens": max_tokens,
            "response_format": response_format,
        }
        try:
            payload = _http_post_json(url, body, timeout_s)
            content = payload["choices"][0]["message"]["content"]
            parsed = json.loads(content)
            return parsed.get("verdicts", [])
        except (urllib.error.URLError, TimeoutError, json.JSONDecodeError, KeyError) as e:
            last_err = e
            continue
    raise last_err if last_err else RuntimeError("chat_verdicts failed")
