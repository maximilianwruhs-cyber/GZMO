#!/usr/bin/env python3
"""
Google Scholar Labs HTML Parser

Extracts structured data from Scholar Labs HTML pages.
Handles the specific DOM structure of Google Scholar Labs AI results.
"""

import re
from typing import List, Dict, Optional, Any

try:
    from bs4 import BeautifulSoup
except ImportError:
    print("Error: beautifulsoup4 not installed. Run: pip install beautifulsoup4 lxml")
    raise


def clean_text(text: Optional[str]) -> str:
    """Clean extracted text by removing extra whitespace."""
    if not text:
        return ""
    return re.sub(r'\s+', ' ', text.strip())


def parse_scholar_labs_html(html: str, query: str) -> List[Dict[str, Any]]:
    """
    Parse Google Scholar Labs HTML and extract paper information.

    Args:
        html: Raw HTML content from Scholar Labs
        query: The original query (for context)

    Returns:
        List of dictionaries containing paper metadata
    """
    soup = BeautifulSoup(html, 'lxml')
    results = []

    # Try multiple selector patterns for result containers
    result_containers = (
        soup.select(".scholar-labs-result-container")
        or soup.select("[data-testid='scholar-labs-result']")
        or soup.select(".gs_ai_results .gs_ai_result")
        or soup.select(".gs_r")  # Classic Google Scholar results
        or soup.find_all("div", class_=lambda x: x and "result" in x.lower())
    )

    for container in result_containers:
        paper = parse_paper_container(container)
        if paper.get("title"):  # Only add if we found at least a title
            results.append(paper)

    return results


def parse_paper_container(container) -> Dict[str, Any]:
    """
    Parse a single paper result container.

    Scholar Labs typically provides:
    - Title (linked to paper)
    - Authors
    - Journal/Venue and Year
    - One-line contextual summary (AI-generated)
    - Key findings as bullet points
    - DOI/Link
    """
    paper = {
        "title": None,
        "authors": [],
        "journal": None,
        "year": None,
        "doi": None,
        "url": None,
        "contextual_summary": None,
        "key_findings": [],
        "citation_count": None,
    }

    # Title - try multiple patterns
    title_elem = (
        container.select_one("h3 a")
        or container.select_one(".gs_rt a")
        or container.select_one("a[data-testid='paper-title']")
        or container.select_one(".scholar-labs-title")
        or container.find("a", href=re.compile(r"/scholar\?.*cites"))
    )

    if title_elem:
        paper["title"] = clean_text(title_elem.get_text())
        paper["url"] = title_elem.get("href")
        # Convert relative URLs to absolute
        if paper["url"] and paper["url"].startswith("/"):
            paper["url"] = f"https://scholar.google.com{paper['url']}"

    # Authors - typically follows title
    authors_elem = (
        container.select_one(".gs_a")
        or container.select_one("[data-testid='paper-authors']")
        or container.select_one(".scholar-labs-authors")
    )

    if authors_elem:
        authors_text = clean_text(authors_elem.get_text())
        # Parse "Author A, Author B - Journal, Year - Publisher" pattern
        paper["authors"] = parse_authors(authors_text)
        paper["journal"], paper["year"] = parse_journal_year(authors_text)

    # AI-generated contextual summary
    summary_elem = (
        container.select_one(".gs_ai_summary")
        or container.select_one("[data-testid='ai-summary']")
        or container.select_one(".scholar-labs-summary")
        or container.find("div", class_=lambda x: x and "summary" in x.lower())
    )

    if summary_elem:
        paper["contextual_summary"] = clean_text(summary_elem.get_text())

    # Key findings (bullet points)
    findings_list = (
        container.select(".gs_ai_findings li")
        or container.select("[data-testid='key-finding']")
        or container.select(".scholar-labs-findings li")
        or container.find_all("li", class_=lambda x: x and "finding" in (x or "").lower())
    )

    for finding in findings_list:
        finding_text = clean_text(finding.get_text())
        if finding_text:
            paper["key_findings"].append(finding_text)

    # DOI extraction from various locations
    doi_elem = (
        container.select_one("a[href*='doi.org']")
        or container.select_one("a[href*='doi:']")
        or container.find("a", href=re.compile(r'doi\.org'))
    )

    if doi_elem:
        doi_href = doi_elem.get("href", "")
        # Extract DOI from URL
        doi_match = re.search(r'doi\.org/(10\.\d{4,}/[^\s"<>]+)', doi_href)
        if doi_match:
            paper["doi"] = f"https://doi.org/{doi_match.group(1)}"

    # Citation count
    cites_elem = (
        container.select_one(".gs_fl a[href*='cites']")
        or container.find("a", text=re.compile(r'Cited by \d+'))
        or container.find("a", string=re.compile(r'Cited by \d+'))
    )

    if cites_elem:
        cites_text = cites_elem.get_text()
        cites_match = re.search(r'Cited by (\d+)', cites_text)
        if cites_match:
            paper["citation_count"] = int(cites_match.group(1))

    return paper


def parse_authors(authors_text: str) -> List[str]:
    """
    Extract author names from the authors line.

    Typical format: "Author A, Author B, Author C - Journal, Year"
    """
    if not authors_text:
        return []

    # Split on dash to separate authors from journal/year
    parts = authors_text.split(" - ", 1)
    if not parts:
        return []

    authors_part = parts[0]

    # Split on commas, but watch out for "et al."
    authors = []
    for name in authors_part.split(","):
        name = clean_text(name)
        # Skip empty strings and common non-author text
        if name and name.lower() not in ('', 'et al', 'et al.'):
            authors.append(name)

    return authors


def parse_journal_year(authors_text: str) -> tuple:
    """
    Extract journal name and year from the authors line.

    Returns: (journal_name_or_None, year_or_None)
    """
    if not authors_text:
        return None, None

    journal = None
    year = None

    # Look for 4-digit year (19xx or 20xx)
    year_match = re.search(r'\b(19|20)\d{2}\b', authors_text)
    if year_match:
        year = int(year_match.group())

    # Extract journal - usually between " - " and the year
    journal_match = re.search(r' - ([^-]+?)(?:,|\s*\d{4})', authors_text)
    if journal_match:
        journal = clean_text(journal_match.group(1))

    return journal, year


def main():
    """CLI for testing parser on saved HTML files."""
    import argparse
    import json
    import sys
    from pathlib import Path

    parser = argparse.ArgumentParser(description="Parse saved Scholar Labs HTML")
    parser.add_argument("html_file", type=Path, help="HTML file to parse")
    parser.add_argument("--query", "-q", default="", help="Original query for context")
    parser.add_argument("--output", "-o", type=Path, help="Output JSON file")

    args = parser.parse_args()

    if not args.html_file.exists():
        print(f"Error: File not found: {args.html_file}")
        sys.exit(1)

    html = args.html_file.read_text(encoding='utf-8')
    results = parse_scholar_labs_html(html, args.query)

    output = {
        "source_file": str(args.html_file),
        "query": args.query,
        "result_count": len(results),
        "results": results,
    }

    json_output = json.dumps(output, indent=2, ensure_ascii=False)

    if args.output:
        args.output.write_text(json_output, encoding='utf-8')
        print(f"Results written to: {args.output}")
    else:
        print(json_output)

    print(f"\nParsed {len(results)} papers from HTML.")


if __name__ == "__main__":
    main()
