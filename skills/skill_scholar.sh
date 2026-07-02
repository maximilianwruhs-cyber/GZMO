#!/usr/bin/env bash
# Google Scholar Labs skill (Tier 2 network exception).
# Agentic literature review via Playwright with verification APIs.
#
# Usage:
#   ./skills/skill_scholar.sh status
#   ./skills/skill_scholar.sh auth-setup
#   ./skills/skill_scholar.sh query --question "How do microplastics affect gut microbiota?"
#   ./skills/skill_scholar.sh followup --session-file session.json --question "Filter to human studies"
#   ./skills/skill_scholar.sh verify --input results.json
#   ./skills/skill_scholar.sh ingest-query --question "..."
#   ./skills/skill_scholar.sh harvest --questions-file topics.txt --max-turns 3

set -euo pipefail

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SKILLS_DIR/.." && pwd)"
SCHOLAR_DIR="$ROOT_DIR/scripts/scholar_labs"
CACHE_DIR="${GZMO_SCHOLAR_CACHE:-$ROOT_DIR/data/scholar-cache}"
AUTH_DIR="${GZMO_SCHOLAR_AUTH:-$ROOT_DIR/playwright/.auth}"
AUTH_FILE="$AUTH_DIR/google_state.json"
RATE_SLEEP="${SCHOLAR_RATE_SLEEP:-3.0}"
VERIFICATION_THRESHOLD="${SCHOLAR_VERIFY_THRESHOLD:-0.85}"

mkdir -p "$CACHE_DIR"
mkdir -p "$CACHE_DIR/raw"
mkdir -p "$CACHE_DIR/sessions"

usage() {
  cat <<'EOF'
Google Scholar Labs skill (live network — Tier 2 exception)

Requires:
  - Python 3 with playwright, beautifulsoup4, httpx, rapidfuzz
  - One-time auth setup: skill_scholar.sh auth-setup

Commands:
  status                    Check auth status and cache stats
  auth-setup                One-time Google authentication (headed browser)

  query --question "..."    Execute a Scholar Labs search query
    [--output file.json] [--hl en|de] [--timeout 30000]

  followup --session-file FILE --question "..."
    Send follow-up query in existing session

  verify --input FILE       Verify results with OpenAlex/S2/Crossref/Unpaywall
    [--output file.json] [--threshold 0.85]

  ingest-query --question "..."
    Query → verify → curated MD → gzmo ingest (single step)

  harvest --questions-file FILE
    Batch query from file (one question per line)
    [--max-turns 3] [--output-dir DIR]

  ingest-harvest --input-dir DIR
    Build curated MD from verified harvest results → gzmo ingest
    [--batch-size 50] [--no-ingest]

  navigator-prompt          Output the Navigator LLM prompt template

Environment:
  GZMO_SCHOLAR_CACHE        Cache directory (default: data/scholar-cache)
  GZMO_SCHOLAR_AUTH         Auth state directory (default: playwright/.auth)
  SCHOLAR_RATE_SLEEP        Seconds between queries (default: 3.0)
  SCHOLAR_VERIFY_THRESHOLD  Similarity threshold (default: 0.85)
EOF
}

# Check Python dependencies
check_python_deps() {
  python3 -c "import playwright, bs4, httpx, rapidfuzz" 2>/dev/null || {
    echo "ERROR: Python dependencies not installed"
    echo "Run: pip install -r $SCHOLAR_DIR/requirements.txt"
    exit 1
  }
}

# Check Playwright browsers
check_playwright() {
  if ! python3 -m playwright install --help >/dev/null 2>&1; then
    echo "ERROR: Playwright not installed"
    exit 1
  fi
}

# Check auth state exists
check_auth() {
  if [[ ! -f "$AUTH_FILE" ]]; then
    echo "ERROR: Google auth state not found at $AUTH_FILE"
    echo "Run: $0 auth-setup"
    exit 1
  fi
}

cmd="${1:-status}"
shift || true

case "$cmd" in
  status)
    # Check Python deps
    py_ok="false"
    if python3 -c "import playwright, bs4, httpx, rapidfuzz" 2>/dev/null; then
      py_ok="true"
    fi

    # Check auth
    auth_ok="false"
    auth_age="null"
    if [[ -f "$AUTH_FILE" ]]; then
      auth_ok="true"
      auth_age=$(($(date +%s) - $(stat -c %Y "$AUTH_FILE" 2>/dev/null || stat -f %m "$AUTH_FILE" 2>/dev/null || echo 0)))
    fi

    # Cache stats
    query_count=0
    if [[ -f "$CACHE_DIR/queries.jsonl" ]]; then
      query_count=$(wc -l < "$CACHE_DIR/queries.jsonl" | tr -d ' ')
    fi

    raw_count=0
    if [[ -d "$CACHE_DIR/raw" ]]; then
      raw_count=$(find "$CACHE_DIR/raw" -type f 2>/dev/null | wc -l | tr -d ' ')
    fi

    session_count=0
    if [[ -d "$CACHE_DIR/sessions" ]]; then
      session_count=$(find "$CACHE_DIR/sessions" -name '*.json' 2>/dev/null | wc -l | tr -d ' ')
    fi

    jq -n \
      --argjson py_ok "$py_ok" \
      --argjson auth_ok "$auth_ok" \
      --argjson auth_age "$auth_age" \
      --arg auth_file "$AUTH_FILE" \
      --arg cache_dir "$CACHE_DIR" \
      --argjson queries "$query_count" \
      --argjson raw_files "$raw_count" \
      --argjson sessions "$session_count" \
      --arg network_tier "exception" \
      '{python_deps: $py_ok, auth: {present: $auth_ok, file: $auth_file, age_seconds: $auth_age}, cache: {dir: $cache_dir, queries: $queries, raw_html: $raw_files, sessions: $sessions}, network_tier: $network_tier}'
    ;;

  auth-setup)
    check_python_deps
    echo "Starting Google authentication setup..."
    echo "A browser window will open. Please log in to Google and verify access to Scholar Labs."
    echo ""
    python3 "$SCHOLAR_DIR/auth_setup.py" --auth-dir "$AUTH_DIR"
    ;;

  query)
    check_python_deps
    check_auth

    question=""
    output=""
    hl="en"
    timeout=30000
    save_session="true"

    while [[ $# -gt 0 ]]; do
      case "$1" in
        --question) question="${2:-}"; shift 2 ;;
        --output) output="${2:-}"; shift 2 ;;
        --hl) hl="${2:-en}"; shift 2 ;;
        --timeout) timeout="${2:-30000}"; shift 2 ;;
        --no-session) save_session="false"; shift ;;
        *) echo "Unknown arg: $1"; usage; exit 1 ;;
      esac
    done

    if [[ -z "$question" ]]; then
      echo "ERROR: --question required"
      exit 1
    fi

    # Generate output path if not specified
    if [[ -z "$output" ]]; then
      timestamp=$(date +%Y%m%d_%H%M%S)
      safe_q=$(echo "$question" | tr -cd '[:alnum:]-' | head -c 40)
      output="$CACHE_DIR/sessions/query_${safe_q}_${timestamp}.json"
    fi

    echo "Querying Scholar Labs..."
    python3 "$SCHOLAR_DIR/query.py" \
      --question "$question" \
      --auth-path "$AUTH_FILE" \
      --cache-dir "$CACHE_DIR" \
      --hl "$hl" \
      --timeout "$timeout" \
      --output "$output" \
      --rate-sleep "$RATE_SLEEP"

    # Save session file for followups
    if [[ "$save_session" == "true" ]]; then
      session_file="${output%.json}_session.json"
      jq --arg session_file "$session_file" '. + {session_file: $session_file}' "$output" > "$session_file"
      echo "Session saved: $session_file"
    fi

    echo "Results: $output"
    ;;

  followup)
    check_python_deps
    check_auth

    session_file=""
    question=""
    output=""

    while [[ $# -gt 0 ]]; do
      case "$1" in
        --session-file) session_file="${2:-}"; shift 2 ;;
        --question) question="${2:-}"; shift 2 ;;
        --output) output="${2:-}"; shift 2 ;;
        *) echo "Unknown arg: $1"; usage; exit 1 ;;
      esac
    done

    if [[ -z "$session_file" || -z "$question" ]]; then
      echo "ERROR: --session-file and --question required"
      exit 1
    fi

    if [[ ! -f "$session_file" ]]; then
      echo "ERROR: Session file not found: $session_file"
      exit 1
    fi

    # Default output based on session file
    if [[ -z "$output" ]]; then
      timestamp=$(date +%Y%m%d_%H%M%S)
      output="${session_file%.json}_followup_${timestamp}.json"
    fi

    echo "Sending follow-up: $question"
    python3 "$SCHOLAR_DIR/followup.py" \
      --session-file "$session_file" \
      --question "$question" \
      --auth-path "$AUTH_FILE" \
      --output "$output" \
      --rate-sleep "$RATE_SLEEP"

    echo "Results: $output"
    ;;

  verify)
    check_python_deps

    input_file=""
    output=""
    threshold="$VERIFICATION_THRESHOLD"
    email="user@example.com"

    while [[ $# -gt 0 ]]; do
      case "$1" in
        --input) input_file="${2:-}"; shift 2 ;;
        --output) output="${2:-}"; shift 2 ;;
        --threshold) threshold="${2:-0.85}"; shift 2 ;;
        --email) email="${2:-}"; shift 2 ;;
        *) echo "Unknown arg: $1"; usage; exit 1 ;;
      esac
    done

    if [[ -z "$input_file" ]]; then
      echo "ERROR: --input required"
      exit 1
    fi

    if [[ ! -f "$input_file" ]]; then
      echo "ERROR: Input file not found: $input_file"
      exit 1
    fi

    # Default output
    if [[ -z "$output" ]]; then
      output="${input_file%.json}_verified.json"
    fi

    echo "Verifying results against academic APIs..."
    echo "Threshold: $threshold"

    python3 "$SCHOLAR_DIR/verify.py" \
      --input "$input_file" \
      --output "$output" \
      --threshold "$threshold" \
      --email "$email"

    echo "Verified results: $output"
    ;;

  ingest-query)
    check_python_deps
    check_auth

    question=""
    hl="en"
    threshold="$VERIFICATION_THRESHOLD"

    while [[ $# -gt 0 ]]; do
      case "$1" in
        --question) question="${2:-}"; shift 2 ;;
        --hl) hl="${2:-en}"; shift 2 ;;
        --threshold) threshold="${2:-0.85}"; shift 2 ;;
        *) echo "Unknown arg: $1"; usage; exit 1 ;;
      esac
    done

    if [[ -z "$question" ]]; then
      echo "ERROR: --question required"
      exit 1
    fi

    timestamp=$(date +%Y%m%d_%H%M%S)
    safe_q=$(echo "$question" | tr -cd '[:alnum:]-' | head -c 40)
    temp_dir="$CACHE_DIR/ingest_tmp_${timestamp}"
    mkdir -p "$temp_dir"

    query_file="$temp_dir/query_${safe_q}.json"
    verified_file="$temp_dir/verified_${safe_q}.json"
    curated_file="$HOME/Schreibtisch/knowledge/curated/thema_008-scholar-harvest-${timestamp}.md"

    echo "=== Step 1: Query Scholar Labs ==="
    python3 "$SCHOLAR_DIR/query.py" \
      --question "$question" \
      --auth-path "$AUTH_FILE" \
      --cache-dir "$CACHE_DIR" \
      --hl "$hl" \
      --output "$query_file" \
      --rate-sleep "$RATE_SLEEP"

    echo ""
    echo "=== Step 2: Verify Results ==="
    python3 "$SCHOLAR_DIR/verify.py" \
      --input "$query_file" \
      --output "$verified_file" \
      --threshold "$threshold"

    echo ""
    echo "=== Step 3: Build Curated Markdown ==="
    # Build curated markdown from verified results
    python3 <<PYEOF
import json
from datetime import datetime

with open("$verified_file", "r") as f:
    data = json.load(f)

query = data.get("query", "")
results = data.get("results", [])
verified_at = data.get("verified_at", datetime.utcnow().isoformat() + "Z")

md = f"""---
title: thema_008-scholar-harvest-{timestamp}
created: {datetime.utcnow().isoformat()}Z
source: google_scholar_labs
query: {query}
verification_threshold: {threshold}
verified_at: {verified_at}
---

# thema_008 Scholar Labs Harvest

**Query:** {query}

**Harvested:** {datetime.utcnow().strftime("%Y-%m-%d %H:%M UTC")}

**Verified:** {verified_at}

**Threshold:** {threshold}

---

## Papers

"""

for i, paper in enumerate(results, 1):
    title = paper.get("title", "Unknown")
    if not title:
        continue

    authors = ", ".join(paper.get("authors", [])) or "Unknown"
    journal = paper.get("journal", "Unknown")
    year = paper.get("year", "Unknown")
    doi = paper.get("doi", "")
    url = paper.get("url", "")
    summary = paper.get("contextual_summary", "")
    findings = paper.get("key_findings", [])

    verif = paper.get("verification", {})
    verif_status = verif.get("status", "unknown")
    verif_conf = verif.get("max_confidence", 0.0)

    md += f"""### {i}. {title}

**Authors:** {authors}

**Journal:** {journal} ({year})

**DOI:** {doi or "N/A"}

**URL:** {url or "N/A"}

**Verification:** {verif_status} (confidence: {verif_conf:.2f})

**AI Summary:** {summary or "N/A"}

**Key Findings:**
"""
    if findings:
        for finding in findings:
            md += f"- {finding}\n"
    else:
        md += "- No key findings extracted\n"

    md += "\n---\n\n"

with open("$curated_file", "w", encoding="utf-8") as f:
    f.write(md)

print(f"Curated markdown written: {curated_file}")
PYEOF

    echo ""
    echo "=== Step 4: GZMO Ingest ==="
    gzmo_bin="${GZMO_BIN:-$ROOT_DIR/target/release/gzmo}"
    if [[ ! -x "$gzmo_bin" ]]; then
      echo "Building gzmo release..."
      (cd "$ROOT_DIR" && cargo build -p gzmo-cli --release)
    fi

    "$gzmo_bin" ingest "$curated_file"

    # Cleanup
    rm -rf "$temp_dir"

    echo ""
    echo "=== Ingest Complete ==="
    jq -n \
      --arg query "$question" \
      --arg curated "$curated_file" \
      --arg threshold "$threshold" \
      '{query: $query, curated_file: $curated, threshold: $threshold, status: "ingested"}'
    ;;

  harvest)
    check_python_deps
    check_auth

    questions_file=""
    max_turns=1
    output_dir=""

    while [[ $# -gt 0 ]]; do
      case "$1" in
        --questions-file) questions_file="${2:-}"; shift 2 ;;
        --max-turns) max_turns="${2:-1}"; shift 2 ;;
        --output-dir) output_dir="${2:-}"; shift 2 ;;
        *) echo "Unknown arg: $1"; usage; exit 1 ;;
      esac
    done

    if [[ -z "$questions_file" ]]; then
      echo "ERROR: --questions-file required"
      exit 1
    fi

    if [[ ! -f "$questions_file" ]]; then
      echo "ERROR: Questions file not found: $questions_file"
      exit 1
    fi

    # Default output dir
    if [[ -z "$output_dir" ]]; then
      timestamp=$(date +%Y%m%d_%H%M%S)
      output_dir="$CACHE_DIR/harvest_${timestamp}"
    fi
    mkdir -p "$output_dir"

    echo "Harvesting questions from: $questions_file"
    echo "Max turns per query: $max_turns"
    echo "Output directory: $output_dir"
    echo ""

    total=0
    success=0
    failed=0

    while IFS= read -r question; do
      # Skip empty lines and comments
      [[ -z "$question" || "$question" =~ ^# ]] && continue

      total=$((total + 1))
      echo "[$total] Query: $question"

      safe_q=$(echo "$question" | tr -cd '[:alnum:]-' | head -c 40)
      query_file="$output_dir/query_${safe_q}.json"

      if python3 "$SCHOLAR_DIR/query.py" \
        --question "$question" \
        --auth-path "$AUTH_FILE" \
        --cache-dir "$CACHE_DIR" \
        --output "$query_file" \
        --rate-sleep "$RATE_SLEEP" 2>&1; then

        success=$((success + 1))
        echo "  ✓ Saved: $query_file"

        # Handle follow-up turns if max_turns > 1
        if [[ $max_turns -gt 1 ]]; then
          echo "  (Follow-up turns not yet implemented in batch mode)"
        fi
      else
        failed=$((failed + 1))
        echo "  ✗ Failed"
      fi

      echo ""
    done < "$questions_file"

    echo "=== Harvest Complete ==="
    jq -n \
      --arg output_dir "$output_dir" \
      --argjson total "$total" \
      --argjson success "$success" \
      --argjson failed "$failed" \
      '{output_dir: $output_dir, total: $total, success: $success, failed: $failed}'
    ;;

  ingest-harvest)
    check_python_deps

    input_dir=""
    batch_size=50
    do_ingest=1

    while [[ $# -gt 0 ]]; do
      case "$1" in
        --input-dir) input_dir="${2:-}"; shift 2 ;;
        --batch-size) batch_size="${2:-50}"; shift 2 ;;
        --no-ingest) do_ingest=0; shift ;;
        *) echo "Unknown arg: $1"; usage; exit 1 ;;
      esac
    done

    if [[ -z "$input_dir" ]]; then
      echo "ERROR: --input-dir required (directory with verified JSON files)"
      exit 1
    fi

    if [[ ! -d "$input_dir" ]]; then
      echo "ERROR: Input directory not found: $input_dir"
      exit 1
    fi

    # Find all verified JSON files
    mapfile -t verified_files < <(find "$input_dir" -name "*verified*.json" -type f 2>/dev/null)

    if [[ ${#verified_files[@]} -eq 0 ]]; then
      # Try with query files
      mapfile -t verified_files < <(find "$input_dir" -name "query_*.json" -type f 2>/dev/null)
    fi

    if [[ ${#verified_files[@]} -eq 0 ]]; then
      echo "ERROR: No verified JSON files found in $input_dir"
      exit 1
    fi

    echo "Found ${#verified_files[@]} result files to process"

    # Process each verified file
    curated_files=()
    for verified_file in "${verified_files[@]}"; do
      echo "Processing: $verified_file"

      # Determine output name
      base_name=$(basename "$verified_file" .json)
      curated_out="$HOME/Schreibtisch/knowledge/curated/thema_008-scholar-harvest-${base_name}.md"

      # Build curated markdown
      python3 "$ROOT_DIR/scripts/build-scholar-harvest-curated.py" \
        --input "$verified_file" \
        --out "$curated_out" \
        --batch-size "$batch_size"

      # Find produced curated files
      mapfile -t produced < <(find "$HOME/Schreibtisch/knowledge/curated" \
        -name "thema_008-scholar-harvest-${base_name}*.md" -newer "$verified_file" 2>/dev/null)

      curated_files+=("${produced[@]}")
    done

    if [[ ${#curated_files[@]} -eq 0 ]]; then
      echo "ERROR: No curated files produced"
      exit 1
    fi

    if [[ "$do_ingest" -eq 0 ]]; then
      jq -n --argjson files "$(printf '%s\n' "${curated_files[@]}" | jq -R . | jq -s .)" \
        '{curated_files: $files, ingest: false}'
      exit 0
    fi

    # Run gzmo ingest
    gzmo_bin="${GZMO_BIN:-$ROOT_DIR/target/release/gzmo}"
    if [[ ! -x "$gzmo_bin" ]]; then
      echo "Building gzmo release..."
      (cd "$ROOT_DIR" && cargo build -p gzmo-cli --release)
    fi

    promoted=0
    for f in "${curated_files[@]}"; do
      echo "=== gzmo ingest: $f ==="
      (cd "$ROOT_DIR" && "$gzmo_bin" ingest "$f")
      promoted=$((promoted + 1))
    done

    jq -n \
      --argjson batches "$promoted" \
      --argjson files "$(printf '%s\n' "${curated_files[@]}" | jq -R . | jq -s .)" \
      --arg input_dir "$input_dir" \
      '{batches_ingested: $batches, curated_files: $files, input_dir: $input_dir}'
    ;;

  navigator-prompt)
    cat <<'NAVEOF'
# Navigator Agent Prompt for Scholar Labs

You are the Navigator Agent for Google Scholar Labs. Your task is to transform
vague research topics into articulate, multi-faceted research questions that
leverage the semantic search capabilities of Scholar Labs.

## Input Format
The user will provide a topic like:
- "Find papers on AI in radiology"
- "microplastics and fish gut microbiota"
- "transformer architectures for citation graphs"

## Output Format
Transform the input into a specific, grammatically complete research question
that includes:
1. The specific technology/entity being studied
2. The specific effect/outcome being measured
3. The specific domain or application context
4. Any relevant temporal or comparative dimensions

## Examples

**Input:** "Find papers on AI in radiology."
**Output:** "How do convolutional neural networks compare to radiologist
assessment in reducing false negatives during early-stage breast cancer
mammography screening?"

**Input:** "microplastics AND gut microbiota AND fish"
**Output:** "How do specific polymer types of microplastics, such as
polyethylene and polystyrene, alter the taxonomic composition and metabolic
pathways of the gut microbiota in freshwater fish species?"

**Input:** "transformers for citation graphs"
**Output:** "How do transformer-based language model architectures affect
the accuracy and completeness of automatic citation relationship extraction
in large-scale academic knowledge graphs?"

## Rules
- Output ONLY the transformed question (no explanation, no markdown)
- Use natural language (not Boolean operators)
- Include 3-5 key entities/concepts in the question
- Be specific about relationships ("how does X affect Y" not "X and Y")
- Keep to 1-2 sentences maximum
NAVEOF
    ;;

  help|-h|--help)
    usage
    ;;

  *)
    echo "Unknown command: $cmd"
    usage
    exit 1
    ;;
esac
