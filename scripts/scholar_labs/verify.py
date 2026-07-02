#!/usr/bin/env python3
"""
Google Scholar Labs Results Verification Layer

Cross-references extracted paper metadata with open academic APIs:
- OpenAlex (primary verification)
- Crossref (DOI resolution)
- Semantic Scholar (AI metadata + embeddings)
- Unpaywall (Open Access PDF resolution)

Uses fuzzy string matching (Levenshtein similarity) to handle title
variations and parsing differences.

Usage:
    python verify.py --input results.json --output verified.json
    python verify.py --input results.json --threshold 0.90
"""

import argparse
import json
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple
from urllib.parse import quote_plus

try:
    import httpx
except ImportError:
    print("Error: httpx not installed. Run: pip install httpx")
    sys.exit(1)

try:
    from rapidfuzz import fuzz
except ImportError:
    print("Error: rapidfuzz not installed. Run: pip install rapidfuzz")
    sys.exit(1)


# Default similarity threshold for title matching
DEFAULT_THRESHOLD = 0.85

# Rate limiting delays (seconds)
OPENALEX_DELAY = 0.1
CROSSREF_DELAY = 0.1
SEMANTIC_SCHOLAR_DELAY = 0.1
UNPAYWALL_DELAY = 0.0  # Unpaywall is more lenient


def levenshtein_similarity(a: str, b: str) -> float:
    """
    Calculate normalized Levenshtein similarity between two strings.

    Returns:
        Similarity ratio between 0.0 and 1.0
    """
    if not a or not b:
        return 0.0
    return fuzz.ratio(a.lower(), b.lower()) / 100.0


def verify_with_openalex(title: str, timeout: int = 30) -> Optional[Dict]:
    """
    Verify paper against OpenAlex API.

    OpenAlex provides:
    - Open bibliographic metadata (CC0)
    - Abstract reconstruction from inverted index
    - Author/institution linking
    - Citation relationships
    """
    try:
        encoded_title = quote_plus(title)
        url = f"https://api.openalex.org/works?search={encoded_title}&per-page=5"

        with httpx.Client(timeout=timeout) as client:
            response = client.get(url)
            response.raise_for_status()
            data = response.json()

        results = data.get("results", [])
        if not results:
            return None

        # Find best title match
        best_match = None
        best_score = 0.0

        for result in results:
            result_title = result.get("display_name", "")
            score = levenshtein_similarity(title, result_title)
            if score > best_score:
                best_score = score
                best_match = result

        if best_match is None:
            return None

        # Extract relevant fields
        work = best_match
        return {
            "source": "openalex",
            "confidence": best_score,
            "openalex_id": work.get("id"),
            "title": work.get("display_name"),
            "authors": [
                auth.get("author", {}).get("display_name", "")
                for auth in work.get("authorships", [])
            ],
            "year": work.get("publication_year"),
            "doi": work.get("doi"),
            "cited_by_count": work.get("cited_by_count"),
            "open_access": work.get("open_access", {}),
            "concepts": [
                c.get("display_name", "")
                for c in work.get("concepts", [])
            ],
        }

    except Exception as e:
        return {"source": "openalex", "error": str(e)}


def verify_with_crossref(doi: Optional[str], timeout: int = 30) -> Optional[Dict]:
    """
    Verify DOI against Crossref API.

    Crossref provides:
    - DOI resolution and validation
    - Bibliographic metadata
    - Reference lists (when available)
    """
    if not doi:
        return None

    # Extract DOI from URL if needed
    if doi.startswith("http"):
        import re
        match = re.search(r'doi\.org/(10\.\d{4,}/[^\s"<>]+)', doi)
        if match:
            doi = match.group(1)
        else:
            return None

    try:
        url = f"https://api.crossref.org/works/{quote_plus(doi)}"

        with httpx.Client(timeout=timeout) as client:
            response = client.get(
                url,
                headers={"User-Agent": "GZMO-Scholar-Verifier/1.0 (mailto:user@example.com)"}
            )
            response.raise_for_status()
            data = response.json()

        work = data.get("message", {})

        return {
            "source": "crossref",
            "doi": work.get("DOI"),
            "title": work.get("title", [None])[0],
            "authors": [
                f"{a.get('given', '')} {a.get('family', '')}".strip()
                for a in work.get("author", [])
            ],
            "year": work.get("published-print", {}).get("date-parts", [[None]])[0][0]
                or work.get("published-online", {}).get("date-parts", [[None]])[0][0],
            "journal": work.get("container-title", [None])[0],
            "publisher": work.get("publisher"),
            "reference_count": work.get("reference-count"),
            "citation_count": work.get("is-referenced-by-count"),
            "type": work.get("type"),
        }

    except Exception as e:
        return {"source": "crossref", "error": str(e)}


def verify_with_semantic_scholar(
    title: Optional[str],
    doi: Optional[str],
    timeout: int = 30
) -> Optional[Dict]:
    """
    Verify against Semantic Scholar API.

    Semantic Scholar provides:
    - AI-enhanced metadata extraction
    - Citation contexts
    - SPECTER document embeddings
    """
    try:
        # Build query - prefer DOI if available
        if doi:
            import re
            # Extract DOI
            if doi.startswith("http"):
                match = re.search(r'doi\.org/(10\.\d{4,}/[^\s"<>]+)', doi)
                if match:
                    doi = match.group(1)
            paper_id = f"DOI:{doi}"
            url = f"https://api.semanticscholar.org/graph/v1/paper/{quote_plus(paper_id)}?fields=title,authors,year,citationCount,referenceCount,openAccessPdf,fieldsOfStudy,publicationDate,journal"
        elif title:
            encoded_title = quote_plus(title)
            url = f"https://api.semanticscholar.org/graph/v1/paper/search?query={encoded_title}&fields=title,authors,year,citationCount,referenceCount,openAccessPdf,fieldsOfStudy,publicationDate,journal&limit=5"
        else:
            return None

        with httpx.Client(timeout=timeout) as client:
            response = client.get(url)
            response.raise_for_status()
            data = response.json()

        # Handle search results vs direct lookup
        if "data" in data:
            papers = data["data"]
        else:
            papers = [data]

        if not papers:
            return None

        # Find best match if searching by title
        best_paper = papers[0]
        confidence = 1.0

        if title and len(papers) > 1:
            best_score = 0.0
            for paper in papers:
                paper_title = paper.get("title", "")
                score = levenshtein_similarity(title, paper_title)
                if score > best_score:
                    best_score = score
                    best_paper = paper
                    confidence = best_score

        paper = best_paper
        return {
            "source": "semantic_scholar",
            "confidence": confidence,
            "paper_id": paper.get("paperId"),
            "title": paper.get("title"),
            "authors": [
                a.get("name", "")
                for a in paper.get("authors", [])
            ],
            "year": paper.get("year"),
            "citation_count": paper.get("citationCount"),
            "reference_count": paper.get("referenceCount"),
            "open_access_pdf": paper.get("openAccessPdf"),
            "fields_of_study": paper.get("fieldsOfStudy", []),
            "publication_date": paper.get("publicationDate"),
            "journal": paper.get("journal", {}).get("name") if paper.get("journal") else None,
        }

    except Exception as e:
        return {"source": "semantic_scholar", "error": str(e)}


def verify_with_unpaywall(doi: Optional[str], email: str, timeout: int = 30) -> Optional[Dict]:
    """
    Check Open Access status via Unpaywall API.

    Unpaywall provides:
    - OA status (Gold, Green, Hybrid, Bronze)
    - Direct PDF URLs for OA papers
    """
    if not doi:
        return None

    # Extract DOI from URL if needed
    if doi.startswith("http"):
        import re
        match = re.search(r'doi\.org/(10\.\d{4,}/[^\s"<>]+)', doi)
        if match:
            doi = match.group(1)
        else:
            return None

    try:
        url = f"https://api.unpaywall.org/v2/{quote_plus(doi)}?email={quote_plus(email)}"

        with httpx.Client(timeout=timeout) as client:
            response = client.get(url)
            response.raise_for_status()
            data = response.json()

        best_oa = data.get("best_oa_location", {})

        return {
            "source": "unpaywall",
            "doi": data.get("doi"),
            "is_oa": data.get("is_oa"),
            "oa_status": data.get("oa_status"),  # gold, green, hybrid, bronze
            "title": data.get("title"),
            "year": data.get("year"),
            "journal": data.get("journal_name"),
            "publisher": data.get("publisher"),
            "oa_url": best_oa.get("url") if best_oa else None,
            "oa_pdf_url": best_oa.get("url_for_pdf") if best_oa else None,
            "license": best_oa.get("license") if best_oa else None,
        }

    except Exception as e:
        return {"source": "unpaywall", "error": str(e)}


def verify_paper(
    paper: Dict[str, Any],
    threshold: float,
    email: str = "user@example.com",
) -> Dict[str, Any]:
    """
    Verify a single paper against all available verification APIs.

    Args:
        paper: Paper dictionary from Scholar Labs parsing
        threshold: Minimum Levenshtein similarity for title match
        email: Email for Unpaywall API (required)

    Returns:
        Enriched paper dict with verification results
    """
    title = paper.get("title", "")
    doi = paper.get("doi")

    if not title:
        return {**paper, "verification": {"status": "failed", "reason": "No title"}}

    verifications = []

    # OpenAlex (primary)
    openalex = verify_with_openalex(title)
    time.sleep(OPENALEX_DELAY)
    if openalex and not openalex.get("error"):
        verifications.append(openalex)

    # Semantic Scholar (by title or DOI)
    s2 = verify_with_semantic_scholar(title, doi)
    time.sleep(SEMANTIC_SCHOLAR_DELAY)
    if s2 and not s2.get("error"):
        verifications.append(s2)

    # Crossref (if DOI available)
    if doi:
        crossref = verify_with_crossref(doi)
        time.sleep(CROSSREF_DELAY)
        if crossref and not crossref.get("error"):
            verifications.append(crossref)

    # Unpaywall (if DOI available)
    if doi:
        unpaywall = verify_with_unpaywall(doi, email)
        time.sleep(UNPAYWALL_DELAY)
        if unpaywall and not unpaywall.get("error"):
            verifications.append(unpaywall)

    # Determine verification status
    if not verifications:
        status = "unverified"
        max_confidence = 0.0
    else:
        # Use highest confidence from any source
        confidences = [
            v.get("confidence", 0.0)
            for v in verifications
            if "confidence" in v
        ]
        max_confidence = max(confidences) if confidences else 0.0

        if max_confidence >= threshold:
            status = "verified"
        elif max_confidence >= threshold * 0.8:
            status = "tentative"
        else:
            status = "unverified"

    return {
        **paper,
        "verification": {
            "status": status,
            "threshold": threshold,
            "max_confidence": max_confidence,
            "sources": verifications,
        }
    }


def verify_results(
    input_file: Path,
    threshold: float,
    email: str = "user@example.com",
) -> Dict[str, Any]:
    """
    Verify all papers in a results file.

    Args:
        input_file: Path to JSON results file
        threshold: Minimum similarity threshold
        email: Email for Unpaywall API

    Returns:
        Updated results with verification data
    """
    with open(input_file, 'r', encoding='utf-8') as f:
        data = json.load(f)

    query = data.get("query", "")
    results = data.get("results", [])

    print(f"Verifying {len(results)} papers from query: {query[:60]}...")
    print(f"Similarity threshold: {threshold}")

    verified_results = []
    stats = {"verified": 0, "tentative": 0, "unverified": 0, "failed": 0}

    for i, paper in enumerate(results, 1):
        print(f"  [{i}/{len(results)}] Verifying: {paper.get('title', 'Unknown')[:50]}...")

        verified = verify_paper(paper, threshold, email)
        verified_results.append(verified)

        status = verified.get("verification", {}).get("status", "failed")
        stats[status] = stats.get(status, 0) + 1

    # Update data structure
    output = {
        **data,
        "verified_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "threshold": threshold,
        "verification_stats": stats,
        "results": verified_results,
    }

    print(f"\nVerification complete:")
    print(f"  Verified:   {stats['verified']}")
    print(f"  Tentative:  {stats['tentative']}")
    print(f"  Unverified: {stats['unverified']}")
    print(f"  Failed:     {stats['failed']}")

    return output


def main():
    parser = argparse.ArgumentParser(
        description="Verify Scholar Labs results against academic APIs"
    )
    parser.add_argument(
        "--input", "-i",
        type=Path,
        required=True,
        help="Input JSON results file"
    )
    parser.add_argument(
        "--output", "-o",
        type=Path,
        help="Output JSON file (default: stdout)"
    )
    parser.add_argument(
        "--threshold", "-t",
        type=float,
        default=DEFAULT_THRESHOLD,
        help=f"Similarity threshold (default: {DEFAULT_THRESHOLD})"
    )
    parser.add_argument(
        "--email", "-e",
        default="user@example.com",
        help="Email for Unpaywall API (required by their terms)"
    )

    args = parser.parse_args()

    if not args.input.exists():
        print(f"Error: Input file not found: {args.input}")
        sys.exit(1)

    try:
        result = verify_results(args.input, args.threshold, args.email)

        json_output = json.dumps(result, indent=2, ensure_ascii=False)

        if args.output:
            args.output.write_text(json_output, encoding='utf-8')
            print(f"\n✓ Verified results saved to: {args.output}")
        else:
            print("\n" + "=" * 60)
            print(json_output)

        sys.exit(0)

    except Exception as e:
        print(f"\n✗ Error during verification: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
