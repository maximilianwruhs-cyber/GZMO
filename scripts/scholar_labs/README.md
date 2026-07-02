# Google Scholar Labs Playwright Driver

Agentic literature review automation for GZMO Sovereign Node (thema_008).

## Overview

This package provides browser-automation tooling for Google Scholar Labs, enabling:
- Semantic conversational search via AI-generated contextual summaries
- Multi-turn follow-up queries for iterative literature refinement
- Cross-verification with OpenAlex, Crossref, Semantic Scholar, and Unpaywall

## Quick Start

### 1. Install Dependencies

```bash
cd /path/to/gzmo/project
pip install -r scripts/scholar_labs/requirements.txt
playwright install chromium
```

### 2. Authenticate

```bash
python scripts/scholar_labs/auth_setup.py
```

This opens a browser for manual Google login. After logging in and verifying access to Scholar Labs, press ENTER to save the session state.

### 3. Run a Query

```bash
python scripts/scholar_labs/query.py \
    --question "How do transformer architectures affect citation graph construction?" \
    --output results.json
```

### 4. Verify Results

```bash
python scripts/scholar_labs/verify.py \
    --input results.json \
    --output verified.json \
    --threshold 0.85
```

### 5. Follow-up Queries

```bash
python scripts/scholar_labs/followup.py \
    --session-file results_session.json \
    --question "Filter these to peer-reviewed studies only"
```

## Module Reference

### auth_setup.py

One-time authentication to save Google session state.

```bash
python auth_setup.py [--auth-dir DIR] [--headless] [--verify-only]
```

### query.py

Execute a Scholar Labs search query.

```bash
python query.py \
    --question "..." \
    [--auth-path PATH] \
    [--hl en|de] \
    [--timeout 30000] \
    [--output file.json] \
    [--rate-sleep 3.0]
```

### parse.py

Parse HTML from Scholar Labs into structured JSON.

```bash
python parse.py saved_page.html [--output parsed.json]
```

### followup.py

Send follow-up questions within an existing session.

```bash
python followup.py \
    --session-file session.json \
    --question "..." \
    [--output file.json]
```

### verify.py

Cross-reference results with academic APIs.

```bash
python verify.py \
    --input results.json \
    [--output verified.json] \
    [--threshold 0.85] \
    [--email user@example.com]
```

## Output Schema

### Query Results

```json
{
  "query": "Original research question",
  "timestamp": "2026-01-15T10:30:00Z",
  "url": "https://scholar.google.com/scholar_labs/search?...",
  "hl": "en",
  "result_count": 10,
  "results": [
    {
      "title": "Paper Title",
      "authors": ["Author A", "Author B"],
      "journal": "Journal Name",
      "year": 2024,
      "doi": "https://doi.org/10.xxxx/xxxxx",
      "url": "https://scholar.google.com/...",
      "contextual_summary": "AI-generated one-line summary...",
      "key_findings": [
        "Specific finding 1",
        "Specific finding 2"
      ],
      "citation_count": 42
    }
  ]
}
```

### Verified Results

Adds `verification` section per paper:

```json
{
  "verification": {
    "status": "verified|tentative|unverified",
    "threshold": 0.85,
    "max_confidence": 0.92,
    "sources": [
      {
        "source": "openalex",
        "confidence": 0.92,
        "openalex_id": "W2741809807",
        "doi": "...",
        "is_oa": true
      }
    ]
  }
}
```

## Rate Limiting

To avoid Google account restrictions:
- Default 3-second sleep between queries
- Do not automate login (use saved auth state only)
- Run auth_setup.py once, reuse the saved session

## Session Management

Query results include a session file that preserves:
- Original query context
- Conversation history for follow-ups
- Raw HTML cache for debugging

Session files are saved to: `data/scholar-cache/sessions/`

## Verification Thresholds

- **Verified** (>= 0.85): High confidence title match
- **Tentative** (0.68-0.84): Moderate match, review recommended
- **Unverified** (< 0.68): Low confidence or no API match

## Troubleshooting

### "Auth state not found"

Run auth_setup.py first to create the session file.

### "Session expired"

Google sessions may expire after extended inactivity. Re-run auth_setup.py.

### "Timeout waiting for results"

Scholar Labs UI may have changed selectors. Check:
1. HTML in cache dir for structure changes
2. Update parse.py selector fallbacks
3. Verify hl=en for stable parsing

### Verification API errors

APIs may rate-limit. Default delays:
- OpenAlex: 0.1s
- Crossref: 0.1s
- Semantic Scholar: 0.1s
- Unpaywall: 0s

## Compliance

This tool operates under GZMO Tier-2 network exceptions:
- scholar.google.com
- api.openalex.org
- api.crossref.org
- api.semanticscholar.org
- api.unpaywall.org

See: gzmo-core/src/compliance.rs
