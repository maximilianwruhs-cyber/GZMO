#!/usr/bin/env python3
"""Assert living research drafts stay off the vault and fail closed without an LLM."""

from datetime import datetime, timezone
from pathlib import Path

from living_research import (
    already_dispatched,
    output_dir_ok,
    resolve_llm,
    resolve_target_file,
    sanitize_query,
)


def test_output_only_under_research_intel():
    living = Path("/home/gzmo/.gzmo-living")
    assert output_dir_ok(living / "data" / "research-intel", living)
    assert not output_dir_ok(living / "data", living)
    assert not output_dir_ok(living / "data" / "vault.db", living)
    assert not output_dir_ok(Path("/tmp/research-intel"), living)


def test_no_llm_is_skip_not_stub():
    assert resolve_llm({}) is None
    assert resolve_llm({"PRIME_CHAT_URL": "http://127.0.0.1:9/v1/chat/completions"}) is None


def test_sanitize_strips_site_prefixes():
    q = sanitize_query("site:arxiv.org rust sqlite hybrid search")
    assert "site:" not in q
    assert "sqlite" in q


def test_resolve_rejects_hallucinated_path_uses_keyword_file():
    repo = Path("/home/gzmo/Projects/GZMO")
    rel = resolve_target_file(
        repo,
        {
            "file_to_touch": "memory/src/vector_store.rs",
            "title": "hybrid search BM25 fusion",
            "summary": "",
            "why": "",
            "integration_point": "",
        },
    )
    assert rel == "gzmo-core/src/memory/recall_rrf.rs"
    assert (repo / rel).is_file()


def test_already_dispatched_skips_same_file_within_week():
    log = Path("/tmp/jules-dispatched-test.jsonl")
    log.write_text(
        '{"ts": 1788100000, "file": "gzmo-core/src/memory/recall_rrf.rs"}\n',
        encoding="utf-8",
    )
    now = datetime.fromtimestamp(1788100000 + 3600, tz=timezone.utc)
    assert already_dispatched(log, "gzmo-core/src/memory/recall_rrf.rs", now)
    assert not already_dispatched(log, "gzmo-core/src/memory/qdrant_recall.rs", now)
    log.unlink()


if __name__ == "__main__":
    test_output_only_under_research_intel()
    test_no_llm_is_skip_not_stub()
    test_sanitize_strips_site_prefixes()
    test_resolve_rejects_hallucinated_path_uses_keyword_file()
    test_already_dispatched_skips_same_file_within_week()
    print("ok")
