#!/usr/bin/env python3
"""Assert living research drafts stay off the vault and fail closed without an LLM."""

from pathlib import Path

from living_research import output_dir_ok, resolve_llm, sanitize_query


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


if __name__ == "__main__":
    test_output_only_under_research_intel()
    test_no_llm_is_skip_not_stub()
    test_sanitize_strips_site_prefixes()
    print("ok")
