#!/usr/bin/env bash
# Perform a web search and return lightweight results. Usage: ./skills/web_search.sh "query"

QUERY="$*"
if [ -z "$QUERY" ]; then
    echo "Usage: $0 <query>"
    exit 1
fi

# Encode the query
ENCODED=$(echo "$QUERY" | jq -Rr @uri 2>/dev/null)
if [ -z "$ENCODED" ]; then
    # Fallback to pure bash encoding if jq is missing
    ENCODED="${QUERY// /+}"
fi

# We use duckduckgo HTML version for easy parsing without API keys
echo "Gathering search results for: $QUERY"

# Simple curl against DDG lite, extracting the snippets
curl -s -A "Mozilla/5.0 (Windows NT 10.0; Win64; x64)" "https://html.duckduckgo.com/html/?q=${ENCODED}" | \
    grep -oP 'class="result__snippet[^>]*>.*?</a>' | \
    sed -e 's/<[^>]*>//g' -e 's/&amp;/\&/g' -e 's/&quot;/"/g' -e 's/&#x27;/'\''/g' | \
    head -n 5

echo "---"
echo "If you need deeper analysis, use shell_exec to 'curl -sL <exact_url>' or clone repositories directly."
