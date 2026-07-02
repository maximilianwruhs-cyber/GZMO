#!/usr/bin/env python3
"""
Google Scholar Labs Multi-Turn Orchestrator (Bibliothekars-Agent Runtime)

Executes the full agentic literature review pipeline:
1. Navigator: Transform topic into articulate research question
2. Query: Execute initial Scholar Labs search
3. Gap Evaluator: Analyze results for missing information
4. Follow-up: Iteratively refine with targeted questions (max N turns)
5. Verification: Cross-reference with academic APIs
6. Deduplication: Remove duplicates by DOI/title
7. Batch Emit: Save verified, curated results

This implements the Bibliothekars-Agent concept from the GZMO knowledge base,
providing the first concrete runtime for agentic literature synthesis.

Usage:
    python orchestrate.py \
        --topic "microplastics in fish gut microbiota" \
        --max-turns 3 \
        --output-dir ./results

    python orchestrate.py \
        --question "How do polyethylene microplastics alter zebrafish gut microbiota?" \
        --navigator-prompt ~/gzmo_skills/prompts/research/scholar-navigator.md \
        --max-turns 2
"""

import argparse
import json
import os
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Optional, List, Dict, Any

# Add parent directory to path for imports
SCRIPT_DIR = Path(__file__).parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from scholar_labs.query import query_scholar_labs, get_default_auth_path, get_default_cache_dir
from scholar_labs.verify import verify_paper, DEFAULT_THRESHOLD


def load_navigator_prompt(prompt_path: Optional[Path]) -> str:
    """Load the Navigator prompt template."""
    if prompt_path and prompt_path.exists():
        return prompt_path.read_text(encoding='utf-8')

    # Default prompt
    return """You are the Navigator Agent for Google Scholar Labs.

Transform the user's research topic into an articulate, multi-faceted research
question that leverages the semantic search capabilities of Scholar Labs.

**Input:**
{topic}

**Output:**
Write a specific, grammatically complete research question that includes:
1. The specific technology/entity being studied
2. The specific effect/outcome being measured
3. The specific domain or application context
4. Any relevant temporal or comparative dimensions

**Rules:**
- Output ONLY the transformed question (no explanation, no markdown)
- Use natural language (not Boolean operators)
- Include 3-5 key entities/concepts in the question
- Be specific about relationships ("how does X affect Y" not "X and Y")
- Keep to 1-2 sentences maximum

**Transformed question:**"""


def navigator_transform(topic: str, prompt: str) -> str:
    """
    Use LLM to transform a vague topic into an articulate research question.

    For now, this uses a simple template-based approach.
    In production, this would call the GZMO LLM (Prime) via API.
    """
    # Check if we can call the GZMO CLI
    gzmo_cli = Path.home() / "Projects" / "_foundation-audit" / "survey_GZMO" / "target" / "release" / "gzmo"
    if not gzmo_cli.exists():
        # Try cargo-installed version
        gzmo_cli = Path.home() / ".cargo" / "bin" / "gzmo"

    formatted_prompt = prompt.format(topic=topic)

    # Simple heuristic transformation (fallback when LLM not available)
    # This is a basic implementation - the real version would use LLM
    words = topic.lower().split()

    # Detect domain patterns and expand
    if "microplastic" in topic or "pollution" in topic:
        return f"How do specific polymer types and size distributions of microplastics alter the taxonomic composition, metabolic pathways, and immune response markers in the gut microbiota of freshwater fish species?"

    if "transformer" in topic or "bert" in topic or "llm" in topic:
        return f"How do transformer-based neural network architectures affect the accuracy, completeness, and computational efficiency of automatic citation relationship extraction and knowledge graph construction from academic literature?"

    if "cancer" in topic or "tumor" in topic or "oncology" in topic:
        return f"How do convolutional neural network architectures compare to radiologist assessment in reducing false negatives during early-stage breast cancer detection from mammography screening?"

    if "radiology" in topic or "medical imaging" in topic:
        return f"How do deep learning approaches improve diagnostic accuracy and reduce inter-observer variability in radiological image interpretation for clinical decision support?"

    # Generic expansion
    return f"How do specific mechanisms and variables related to {topic} interact to produce measurable effects in their target systems, and what are the underlying causal pathways?"


def analyze_gaps(results: List[Dict[str, Any]]) -> Optional[str]:
    """
    Analyze results to identify gaps that need follow-up questions.

    Returns a follow-up question if gaps found, None if results are complete.

    This is a basic heuristic implementation. Production version would use LLM.
    """
    if not results:
        return None

    # Check for missing DOIs (indicates incomplete metadata)
    missing_doi = sum(1 for r in results if not r.get("doi"))
    if missing_doi > len(results) * 0.3:  # More than 30% missing DOIs
        return "For the papers without DOIs, can you provide more complete bibliographic information including journal, year, and full citation?"

    # Check for papers without key findings
    missing_findings = sum(1 for r in results if not r.get("key_findings"))
    if missing_findings > len(results) * 0.3:
        return "Can you extract specific quantitative findings, measurements, or statistical results from the papers that lack detailed key findings?"

    # Check for recent vs older papers (balance temporal coverage)
    years = [r.get("year") for r in results if r.get("year")]
    if years:
        recent = sum(1 for y in years if y and y >= 2020)
        if recent < len(years) * 0.2:  # Less than 20% recent papers
            return "Can you focus on more recent publications from 2020-2025 that address this topic with current methodologies?"

    # Check for clinical/human studies (if medical topic)
    has_clinical = any(
        "clinical" in (r.get("contextual_summary") or "").lower() or
        "human" in (r.get("contextual_summary") or "").lower()
        for r in results
    )
    if not has_clinical:
        return "Can you filter these to focus on studies with human subjects or clinical applications?"

    return None


def deduplicate_results(results: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """Remove duplicate papers based on DOI or title similarity."""
    seen_dois = set()
    seen_titles = []
    unique_results = []

    for result in results:
        # Check DOI
        doi = result.get("doi")
        if doi:
            doi_norm = doi.lower().replace("https://doi.org/", "").replace("http://doi.org/", "")
            if doi_norm in seen_dois:
                continue
            seen_dois.add(doi_norm)

        # Check title similarity
        title = result.get("title", "").lower()
        is_duplicate = False
        for seen_title in seen_titles:
            # Simple word overlap check (production: use rapidfuzz)
            title_words = set(title.split())
            seen_words = set(seen_title.split())
            if len(title_words) > 0:
                overlap = len(title_words & seen_words) / len(title_words)
                if overlap > 0.8:  # 80% word overlap
                    is_duplicate = True
                    break

        if not is_duplicate:
            seen_titles.append(title)
            unique_results.append(result)

    return unique_results


def run_orchestrator(
    topic: str,
    initial_question: Optional[str],
    navigator_prompt: str,
    max_turns: int,
    auth_path: Path,
    cache_dir: Path,
    output_dir: Path,
    threshold: float,
    rate_sleep: float,
) -> Dict[str, Any]:
    """
    Run the full multi-turn orchestration pipeline.

    Args:
        topic: The research topic (used if no initial_question)
        initial_question: Pre-formulated question (skips Navigator)
        navigator_prompt: The Navigator prompt template
        max_turns: Maximum follow-up turns
        auth_path: Path to Playwright auth state
        cache_dir: Cache directory
        output_dir: Output directory for results
        threshold: Verification similarity threshold
        rate_sleep: Seconds to sleep between queries

    Returns:
        Final results dictionary
    """
    output_dir.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.utcnow().isoformat() + "Z"

    # Step 1: Navigator (if no initial question provided)
    if initial_question:
        question = initial_question
        print(f"Using provided question: {question}")
    else:
        print(f"Navigator: Transforming topic '{topic}' into research question...")
        question = navigator_transform(topic, navigator_prompt)
        print(f"Navigator output: {question}")

    # Step 2: Initial Query
    print(f"\n[Turn 1/{max_turns + 1}] Initial query...")
    initial_results = query_scholar_labs(
        question=question,
        auth_path=auth_path,
        cache_dir=cache_dir,
        save_raw=True,
    )

    all_results = initial_results.get("results", [])
    print(f"  Initial results: {len(all_results)} papers")

    session_info = {
        "question": question,
        "timestamp": timestamp,
        "turns": []
    }

    # Steps 3-4: Gap Analysis and Follow-up (iterative)
    current_turn = 1
    for turn in range(max_turns):
        current_turn += 1

        # Analyze gaps
        print(f"\n[Gap Analysis] Checking for missing information...")
        followup_question = analyze_gaps(all_results)

        if not followup_question:
            print(f"  No significant gaps found. Stopping at turn {current_turn}.")
            break

        print(f"  Gap found: {followup_question[:80]}...")

        # Send follow-up
        print(f"\n[Turn {current_turn}/{max_turns + 1}] Follow-up: {followup_question[:60]}...")

        # For now, we need to re-run query with modified question
        # In production, this would use the followup.py session mechanism
        modified_question = f"{question} {followup_question}"

        followup_results = query_scholar_labs(
            question=modified_question,
            auth_path=auth_path,
            cache_dir=cache_dir,
            save_raw=True,
        )

        new_results = followup_results.get("results", [])
        print(f"  Follow-up results: {len(new_results)} papers")

        # Merge results
        all_results.extend(new_results)
        all_results = deduplicate_results(all_results)
        print(f"  After deduplication: {len(all_results)} unique papers")

        session_info["turns"].append({
            "turn": turn + 1,
            "followup_question": followup_question,
            "new_results": len(new_results),
            "total_unique": len(all_results),
        })

        if rate_sleep > 0:
            time.sleep(rate_sleep)

    # Step 5: Verification
    print(f"\n[Verification] Cross-referencing {len(all_results)} papers with academic APIs...")
    verified_results = []

    for i, paper in enumerate(all_results, 1):
        print(f"  [{i}/{len(all_results)}] {paper.get('title', 'Unknown')[:50]}...")
        verified = verify_paper(paper, threshold, email="user@example.com")
        verified_results.append(verified)

    # Step 6: Compile final output
    final_output = {
        "topic": topic,
        "original_question": question,
        "timestamp": datetime.utcnow().isoformat() + "Z",
        "session": session_info,
        "turns_executed": len(session_info["turns"]) + 1,
        "result_count": len(verified_results),
        "verification_threshold": threshold,
        "results": verified_results,
    }

    # Save results
    output_file = output_dir / f"orchestrated_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}.json"
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(final_output, f, indent=2, ensure_ascii=False)

    print(f"\n✓ Orchestration complete. Results saved to: {output_file}")

    # Stats
    verified_count = sum(
        1 for r in verified_results
        if r.get("verification", {}).get("status") == "verified"
    )
    print(f"  Verified papers: {verified_count}/{len(verified_results)}")

    return final_output


def main():
    parser = argparse.ArgumentParser(
        description="Multi-turn Scholar Labs orchestrator (Bibliothekars-Agent runtime)"
    )
    parser.add_argument(
        "--topic", "-t",
        help="Research topic (for Navigator transformation)"
    )
    parser.add_argument(
        "--question", "-q",
        help="Pre-formulated research question (skips Navigator)"
    )
    parser.add_argument(
        "--navigator-prompt",
        type=Path,
        help="Path to Navigator prompt template"
    )
    parser.add_argument(
        "--max-turns",
        type=int,
        default=3,
        help="Maximum follow-up turns (default: 3)"
    )
    parser.add_argument(
        "--auth-path",
        type=Path,
        default=None,
        help="Path to Playwright auth state JSON"
    )
    parser.add_argument(
        "--cache-dir",
        type=Path,
        default=None,
        help="Cache directory"
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("./scholar_orchestration_results"),
        help="Output directory (default: ./scholar_orchestration_results)"
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=DEFAULT_THRESHOLD,
        help=f"Verification similarity threshold (default: {DEFAULT_THRESHOLD})"
    )
    parser.add_argument(
        "--rate-sleep",
        type=float,
        default=3.0,
        help="Seconds between queries for rate limiting (default: 3.0)"
    )

    args = parser.parse_args()

    # Validate inputs
    if not args.topic and not args.question:
        print("ERROR: Either --topic or --question required")
        sys.exit(1)

    # Set defaults
    auth_path = args.auth_path or get_default_auth_path()
    cache_dir = args.cache_dir or get_default_cache_dir()

    # Check auth
    if not auth_path.exists():
        print(f"ERROR: Auth state not found at {auth_path}")
        print("Run: python auth_setup.py")
        sys.exit(1)

    # Load navigator prompt
    navigator_prompt = load_navigator_prompt(args.navigator_prompt)

    try:
        result = run_orchestrator(
            topic=args.topic or args.question,
            initial_question=args.question,
            navigator_prompt=navigator_prompt,
            max_turns=args.max_turns,
            auth_path=auth_path,
            cache_dir=cache_dir,
            output_dir=args.output_dir,
            threshold=args.threshold,
            rate_sleep=args.rate_sleep,
        )

        sys.exit(0)

    except Exception as e:
        print(f"\n✗ Orchestration failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
