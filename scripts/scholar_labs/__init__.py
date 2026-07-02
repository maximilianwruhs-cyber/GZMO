"""
Google Scholar Labs Playwright Driver Package

Agentic literature review automation for GZMO Sovereign Node.

This package provides:
- auth_setup: One-time Google authentication
- query: Execute Scholar Labs queries
- parse: Extract structured data from HTML
- followup: Multi-turn conversation handling
- verify: Cross-reference with OpenAlex, Crossref, S2, Unpaywall

Usage:
    from scripts.scholar_labs.query import query_scholar_labs
    from scripts.scholar_labs.verify import verify_paper

See README.md for detailed usage instructions.
"""

__version__ = "0.1.0"
__all__ = [
    "auth_setup",
    "query",
    "parse",
    "followup",
    "verify",
]
