#!/usr/bin/env python3
"""Living-Keep research intel: draft ideas only. Never writes the vault."""

from __future__ import annotations

import json
import os
import re
import sqlite3
import sys
import urllib.error
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from datetime import datetime, timezone
from pathlib import Path

INTERNAL = {
    "ct101",
    "workstation",
    "vm200",
    "gzmo",
    "brain feed",
    "stigmergy",
}


def output_dir_ok(path: Path, living: Path) -> bool:
    try:
        resolved = path.resolve()
        allowed = (living / "data" / "research-intel").resolve()
        return resolved == allowed or allowed in resolved.parents
    except OSError:
        return False


def resolve_llm(env: dict) -> dict | None:
    key = (env.get("GZMO_OPENROUTER_KEY") or "").strip()
    if key:
        return {
            "provider": "openrouter",
            "url": env.get(
                "OPENROUTER_URL", "https://openrouter.ai/api/v1/chat/completions"
            ),
            "model": env.get("GZMO_RESEARCH_MODEL") or "deepseek/deepseek-v4-flash",
            "api_key": key,
        }
    prime = (env.get("PRIME_CHAT_URL") or "").strip()
    if not prime:
        return None
    base = prime[: -len("/chat/completions")] if prime.endswith("/chat/completions") else prime
    try:
        urllib.request.urlopen(base.rstrip("/") + "/models", timeout=2)
    except (urllib.error.URLError, TimeoutError, OSError, ValueError):
        return None
    return {
        "provider": "prime",
        "url": prime if prime.endswith("/chat/completions") else prime.rstrip("/") + "/chat/completions",
        "model": env.get("PRIME_MODEL") or "local",
        "api_key": "",
    }


def _chat(llm: dict, prompt: str, max_tokens: int, temperature: float) -> str:
    payload = {
        "model": llm["model"],
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": max_tokens,
        "temperature": temperature,
    }
    headers = {"Content-Type": "application/json"}
    if llm.get("api_key"):
        headers["Authorization"] = f"Bearer {llm['api_key']}"
    req = urllib.request.Request(
        llm["url"], data=json.dumps(payload).encode(), headers=headers, method="POST"
    )
    with urllib.request.urlopen(req, timeout=180) as r:
        body = json.loads(r.read().decode())
    return (body["choices"][0]["message"].get("content") or "").strip()


def _json_array(text: str):
    text = re.sub(r"^```(?:json)?|```$", "", text, flags=re.M).strip()
    m = re.search(r"\[.*\]", text, flags=re.S)
    if not m:
        raise ValueError("no JSON array")
    return json.loads(m.group(0))


def sanitize_query(q: str) -> str:
    q = q.lower()
    q = re.sub(r"site:\S+", " ", q)
    for tok in INTERNAL:
        q = q.replace(tok, "")
    return re.sub(r"\s+", " ", q).strip(" ,;-")


def build_lens(repo: Path, living: Path) -> str:
    chunks: list[str] = []
    for rel in (
        "docs/ADR-0007-one-product-living.md",
        "docs/ADR-0003-one-instance-metabolism.md",
        "docs/ADR-0005-flywheel-over-frozen-topology.md",
        "Cargo.toml",
        "gzmo-core/Cargo.toml",
        "gzmo-core/src/metabolism.rs",
        "gzmo-core/src/mcp/serve.rs",
    ):
        p = repo / rel
        if p.is_file():
            chunks.append(f"=== {rel} ===\n{p.read_text(encoding='utf-8', errors='replace')[:2000]}\n")
    vault = living / "data" / "vault.db"
    if vault.is_file():
        try:
            con = sqlite3.connect(f"file:{vault}?mode=ro", uri=True)
            hp = con.execute("SELECT COUNT(*) FROM honeypot WHERE is_latest=1").fetchone()[0]
            sem = con.execute("SELECT COUNT(*) FROM semantic_vault").fetchone()[0]
            con.close()
            chunks.append(f"=== vault census ===\nlatest_honeypot={hp} semantic_vault={sem}\n")
        except sqlite3.Error as e:
            chunks.append(f"=== vault census ===\n(unreadable: {e})\n")
    return "\n".join(chunks)[:16000]


def derive_queries(llm: dict, lens: str) -> list[str]:
    prompt = (
        "You research upgrades for a local living-memory Keep (SQLite vault, "
        "Redis/Qdrant/Neo4j, Rust MCP, overnight distill/spark). "
        "From the stack snapshot, derive 3 search queries for arXiv, GitHub, "
        "Hugging Face, and crates.io. No generic AI news. "
        "Return ONLY a JSON array of exactly 3 strings.\n\n"
        + lens
    )
    raw = _json_array(_chat(llm, prompt, 1024, 0.2))
    queries = [sanitize_query(q) for q in raw if isinstance(q, str) and sanitize_query(q)]
    if len(queries) < 1:
        raise ValueError("no usable queries")
    return queries[:3]


def _terms(q: str, n: int) -> list[str]:
    stop = {"the", "and", "for", "with", "local", "offline", "a", "of", "in", "on"}
    words = [w for w in re.split(r"[^a-zA-Z0-9_]+", q.lower()) if len(w) > 2]
    kept = [w for w in words if w not in stop] or words
    return kept[:n]


def _fetch_json(url: str, timeout: int = 20):
    req = urllib.request.Request(url, headers={"User-Agent": "gzmo-living-research/1.0"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read().decode())


def fetch_sources(queries: list[str], db_path: Path, max_per: int) -> dict:
    db_path.parent.mkdir(parents=True, exist_ok=True)
    con = sqlite3.connect(db_path)
    con.execute("CREATE TABLE IF NOT EXISTS seen (key TEXT PRIMARY KEY, first_seen TEXT, title TEXT)")
    seen = {r[0] for r in con.execute("SELECT key FROM seen")}
    now = datetime.now(timezone.utc).isoformat()
    results, errors = [], {}

    def remember(key: str, title: str) -> bool:
        if not key or key in seen:
            return False
        seen.add(key)
        con.execute(
            "INSERT OR IGNORE INTO seen (key, first_seen, title) VALUES (?,?,?)",
            (key, now, title[:200]),
        )
        return True

    def arxiv(q: str) -> list[dict]:
        out = []
        query = " AND ".join(f"all:{t}" for t in _terms(q, 3))
        u = (
            "http://export.arxiv.org/api/query?search_query="
            + urllib.parse.quote(query)
            + f"&start=0&max_results={max_per}&sortBy=submittedDate&sortOrder=descending"
        )
        req = urllib.request.Request(u, headers={"User-Agent": "gzmo-living-research/1.0"})
        with urllib.request.urlopen(req, timeout=25) as r:
            root = ET.fromstring(r.read())
        ns = {"a": "http://www.w3.org/2005/Atom"}
        for e in root.findall("a:entry", ns):
            key = (e.findtext("a:id", "", ns) or "").strip()
            title = re.sub(r"\s+", " ", e.findtext("a:title", "", ns) or "").strip()
            if not remember(key, title):
                continue
            out.append(
                {
                    "source": "arxiv",
                    "id": key,
                    "title": title,
                    "url": key,
                    "published": (e.findtext("a:published", "", ns) or "")[:10],
                    "summary": re.sub(r"\s+", " ", e.findtext("a:summary", "", ns) or "").strip()[:500],
                }
            )
        return out

    def github(q: str) -> list[dict]:
        out = []
        try:
            d = _fetch_json(
                "https://api.github.com/search/repositories?q="
                + urllib.parse.quote(" ".join(_terms(q, 4)))
                + f"&sort=stars&per_page={max_per}"
            )
        except Exception as e:
            errors["github"] = str(e)
            return out
        for it in d.get("items", []):
            key = it.get("full_name", "")
            if not remember(key, key):
                continue
            out.append(
                {
                    "source": "github",
                    "id": key,
                    "title": f"{key} — {it.get('description') or ''}"[:200],
                    "url": it.get("html_url", key),
                    "published": (it.get("pushed_at") or "")[:10],
                    "summary": f"stars={it.get('stargazers_count', 0)} lang={it.get('language') or '-'}",
                }
            )
        return out

    def hf(q: str) -> list[dict]:
        out = []
        try:
            d = _fetch_json(
                "https://huggingface.co/api/models?search="
                + urllib.parse.quote(" ".join(_terms(q, 3)))
                + f"&sort=trendingScore&limit={max_per}"
            )
        except Exception as e:
            errors["huggingface"] = str(e)
            return out
        for it in d:
            key = it.get("id", "")
            if not remember(key, key):
                continue
            out.append(
                {
                    "source": "huggingface",
                    "id": key,
                    "title": key,
                    "url": f"https://huggingface.co/{key}",
                    "published": (it.get("lastModified") or "")[:10],
                    "summary": f"likes={it.get('likes', 0)} downloads={it.get('downloads', 0)}",
                }
            )
        return out

    def crates(q: str) -> list[dict]:
        out = []
        try:
            d = _fetch_json(
                "https://crates.io/api/v1/crates?q="
                + urllib.parse.quote(" ".join(_terms(q, 3)))
                + f"&per_page={max_per}&sort=recent-downloads"
            )
        except Exception as e:
            errors["crates.io"] = str(e)
            return out
        for it in d.get("crates", []):
            key = f"crates.io/{it.get('id', '')}"
            if not remember(key, key):
                continue
            out.append(
                {
                    "source": "crates.io",
                    "id": key,
                    "title": f"{it.get('id')} — {it.get('description') or ''}"[:200],
                    "url": f"https://crates.io/crates/{it.get('id')}",
                    "published": (it.get("updated_at") or "")[:10],
                    "summary": f"downloads={it.get('downloads', 0)}",
                }
            )
        return out

    for q in queries:
        for name, fn in (
            ("arxiv", arxiv),
            ("github", github),
            ("huggingface", hf),
            ("crates.io", crates),
        ):
            try:
                got = fn(q)
                for item in got:
                    item["query"] = q
                results.extend(got)
            except Exception as e:
                errors.setdefault(name, str(e))
    con.commit()
    con.close()
    return {"findings": results, "errors": errors, "seen_total": len(seen)}


def evaluate(llm: dict, queries: list[str], findings: list[dict], top_n: int) -> list[dict]:
    if not findings:
        return []
    listing = "\n".join(
        f"- [{i}] ({f['source']}) {f['title']} | {f.get('summary', '')[:200]}"
        for i, f in enumerate(findings)
    )
    prompt = (
        "You draft Keep upgrades (performance or usability). "
        "For each finding: benefit=true only if it would change a concrete file "
        "(vault, metabolism, MCP, spark, embeddings, sidecars). "
        "Generic AI news → benefit=false. "
        "Return ONLY JSON array of {index, benefit, why, integration_point, file_to_touch}.\n\n"
        f"Queries: {json.dumps(queries)}\n\nFindings:\n{listing}"
    )
    evaluated = _json_array(_chat(llm, prompt, 1600, 0.1))
    items = []
    for i, f in enumerate(findings):
        ev = next((e for e in evaluated if e.get("index") == i), None) or {}
        items.append(
            {
                **f,
                "benefit": bool(ev.get("benefit")),
                "why": ev.get("why", ""),
                "integration_point": ev.get("integration_point", ""),
                "file_to_touch": ev.get("file_to_touch", ""),
            }
        )
    items.sort(key=lambda x: (not x["benefit"], x["source"]))
    return items[: max(top_n, len(items))]


def write_drafts(out: Path, living: Path, stamp: str, queries: list[str], items: list[dict], fetch_errors: dict, eval_err: str | None) -> None:
    if not output_dir_ok(out, living):
        raise SystemExit(f"refuse: output {out} is not living research-intel")
    out.mkdir(parents=True, exist_ok=True)
    now = datetime.now(timezone.utc).isoformat()
    payload = {
        "schema": "gzmo.living_research_intel/v1",
        "generated_at": now,
        "ok": True,
        "vault_written": False,
        "queries": queries,
        "findings": items,
        "fetch_errors": fetch_errors,
        "eval_error": eval_err,
    }
    (out / f"research-intel-{stamp}.json").write_text(json.dumps(payload, indent=2) + "\n")
    md = [
        f"# living research-intel — {stamp}",
        "",
        "Drafts only. Not ingested. Not a PR.",
        "",
        f"queries: {json.dumps(queries)}",
        f"findings: {len(items)} benefit={sum(1 for i in items if i.get('benefit'))} eval_error={eval_err or 'none'}",
        "",
        "## Top drafts",
        "",
    ]
    for t in [i for i in items if i.get("benefit")][:3] or items[:3]:
        md.append(f"### {t['title']}")
        md.append(f"- source: {t['source']} · {t.get('url')}")
        if t.get("why"):
            md.append(f"- why: {t['why']}")
        if t.get("integration_point"):
            md.append(f"- idea: {t['integration_point']}")
        if t.get("file_to_touch"):
            md.append(f"- file: {t['file_to_touch']}")
        md.append("")
    (out / f"research-intel-{stamp}.md").write_text("\n".join(md) + "\n", encoding="utf-8")
    (out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n")
    (out / "latest.md").write_text("\n".join(md) + "\n", encoding="utf-8")


def main(argv: list[str] | None = None) -> int:
    import argparse

    p = argparse.ArgumentParser(description="Living Keep research drafts (no vault writes)")
    p.add_argument("--living-home", default=os.environ.get("GZMO_LIVING_HOME", str(Path.home() / ".gzmo-living")))
    p.add_argument("--repo", default=str(Path(__file__).resolve().parents[1]))
    p.add_argument("--out", default="")
    p.add_argument("--max-per-source", type=int, default=int(os.environ.get("RESEARCH_INTEL_MAX_PER_SOURCE", "2")))
    p.add_argument("--top", type=int, default=int(os.environ.get("RESEARCH_INTEL_TOP", "3")))
    args = p.parse_args(argv)

    living = Path(args.living_home)
    out = Path(args.out) if args.out else living / "data" / "research-intel"
    if not output_dir_ok(out, living):
        print(f"[!] refuse output path {out}", file=sys.stderr)
        return 2

    llm = resolve_llm(os.environ)
    if llm is None:
        print("[!] no live LLM (OpenRouter key or reachable Prime) — skip", file=sys.stderr)
        return 2

    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    lens = build_lens(Path(args.repo), living)
    try:
        queries = derive_queries(llm, lens)
    except Exception as e:
        print(f"[!] query derivation failed: {e}", file=sys.stderr)
        return 2

    fetch = fetch_sources(queries, out / "seen.db", args.max_per_source)
    eval_err = None
    try:
        items = evaluate(llm, queries, fetch.get("findings") or [], args.top)
    except Exception as e:
        eval_err = str(e)
        items = [
            {**f, "benefit": False, "why": "", "integration_point": "", "file_to_touch": ""}
            for f in (fetch.get("findings") or [])
        ]
    write_drafts(out, living, stamp, queries, items, fetch.get("errors") or {}, eval_err)
    print(json.dumps({"ok": True, "out": str(out / "latest.md"), "findings": len(items), "vault_written": False}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
