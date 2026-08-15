#!/usr/bin/env bash
# Stack-watch: monitor GitHub releases + arXiv SOTA for Max's stack
#   agentic dev · airgapped edge devices · SLM fine-tuning frameworks
#
# Writes findings to GZMO data-next/stack-watch/
# Usage:
#   bash ~/github-clone/GZMO/scripts/stack-watch.sh
set -euo pipefail

ROOT="${GZMO_ROOT:-$HOME/github-clone/GZMO}"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/stack-watch"
STATE="$OUT/state.json"
mkdir -p "$OUT"

WATCH_REPOS=(
  "langchain-ai/langgraph:Agent orchestrator"
  "microsoft/autogen:Multi-agent framework"
  "All-Hands-AI/OpenHands:Dev agent"
  "Aider-AI/aider:AI pair programming"
  "unslothai/unsloth:SLM fine-tuning"
  "hiyouga/LLaMA-Factory:SLM tuning"
  "huggingface/trl:Transformer RL"
  "pytorch/torchtune:PyTorch tuning"
  "axolotl-ai-cloud/axolotl:SLM framework"
  "ggml-org/llama.cpp:Edge inference"
  "ollama/ollama:Local LLM"
)

ARXIV_QUERIES=(
  "cat:cs.AI+AND+(agentic+OR+multi-agent+OR+SWE-bench+OR+code-agent)"
  "cat:cs.LG+AND+(LoRA+OR+QLoRA+OR+DoRA+OR+fine-tuning+SLM)"
  "cat:cs.AI+AND+(edge+AI+OR+on-device+LLM+OR+airgap+inference)"
)

stamp()       { date -u +%Y%m%dT%H%M%SZ; }
now_iso()     { date -u +%Y%m%dT%H%M%SZ; }
pl()          { python3 -c "$@"; }

init_state() {
  if [[ ! -f "$STATE" ]]; then
    pl "import json; print(json.dumps({'last_run':None,'releases':{},'papers_seen':[]}))" > "$STATE"
  fi
}

load_state() { cat "$STATE"; }

save_state() { cat > "$STATE"; }

check_releases() {
  local repo="$1" label="$2"
  local key="${repo//\//_}"
  local state="$3"
  local seen_tag
  seen_tag=$(echo "$state" | pl "
import json,sys; s=json.loads(sys.stdin.read())
print(s.get('releases',{}).get('$key',''))
") || seen_tag=""

  local data
  data=$(curl -sfL --max-time 10 -H "Accept: application/vnd.github+json" \
    "https://api.github.com/repos/$repo/releases?per_page=2" 2>/dev/null || echo "[]")

  local count
  count=$(echo "$data" | pl "
import json,sys
try:
    d=json.load(sys.stdin)
    print(len(d) if isinstance(d,list) else 0)
except: print(0)
") || count=0

  [[ "$count" -eq 0 ]] && return

  local latest_tag=""
  local findings=""

  for i in $(seq 0 $((count - 1))); do
    local tag name html_url published
    tag=$(echo "$data" | pl "import json,sys;d=json.load(sys.stdin);print(d[$i].get('tag_name',''))" 2>/dev/null || true)
    name=$(echo "$data" | pl "import json,sys;d=json.load(sys.stdin);print(d[$i].get('name',''))" 2>/dev/null || true)
    html_url=$(echo "$data" | pl "import json,sys;d=json.load(sys.stdin);print(d[$i].get('html_url',''))" 2>/dev/null || true)
    published=$(echo "$data" | pl "import json,sys;d=json.load(sys.stdin);print(d[$i].get('published_at','')[:10])" 2>/dev/null || true)

    [[ -z "$tag" || -z "$published" ]] && continue

    # skip prereleases
    local prerelease
    prerelease=$(echo "$data" | pl "import json,sys;d=json.load(sys.stdin);print(str(d[$i].get('prerelease',False)).lower())" 2>/dev/null || true)
    [[ "$prerelease" == "true" ]] && continue

    if [[ -z "$latest_tag" || "$tag" > "$latest_tag" ]]; then
      latest_tag="$tag"
    fi

    if [[ -n "$seen_tag" && "$tag" != "$seen_tag" ]]; then
      findings+="$label|$name|$tag|$published|$html_url\n"
    fi
  done

  if [[ -n "$findings" ]]; then
    echo -e "$findings" | while IFS='|' read -r lbl name tag date url; do
      echo "• **$lbl** — $name ($tag) — $date"
      echo "  $url"
    done
    echo ""
  fi

  if [[ -n "$latest_tag" && "$latest_tag" != "$seen_tag" ]]; then
    echo "STATE:$key=$latest_tag"
  fi
}

check_arxiv() {
  local query="$1" state="$2"

  local encoded
  encoded=$(pl "
import urllib.parse, sys
print(urllib.parse.quote(sys.argv[1]))
" "$query" 2>/dev/null)

  local since_str
  since_str=$(pl "
from datetime import datetime, timedelta, timezone
d = datetime.now(timezone.utc) - timedelta(days=7)
print(d.strftime('%Y%m%d%H%M%S'))
")

  local full_url="http://export.arxiv.org/api/query?search_query=${query}+AND+submittedDate:[${since_str}0000+TO+*]&start=0&max_results=5&sortBy=submittedDate&sortOrder=descending"
  local xml
  xml=$(curl -sfL --max-time 15 "$full_url" 2>/dev/null || echo "")

  [[ -z "$xml" ]] && return

  pl "
import xml.etree.ElementTree as ET, sys, html

xml_str = '''$xml'''
try:
    root = ET.fromstring(xml_str)
except:
    sys.exit(0)

ns = {'a': 'http://www.w3.org/2005/Atom'}
for entry in root.findall('a:entry', ns):
    title = entry.find('a:title', ns)
    title = title.text.strip().replace('\n',' ')[:100] if title is not None else '?'
    link = entry.find('a:id', ns)
    link = link.text.strip() if link is not None else '?'
    published = entry.find('a:published', ns)
    published = published.text.strip()[:10] if published is not None else '?'
    authors = [a.find('a:name',ns).text for a in entry.findall('a:entry/a:author',ns) if a.find('a:name',ns) is not None]
    if not authors:
        authors = [a.find('a:name',ns).text for a in entry.findall('{http://www.w3.org/2005/Atom}author') if a.find('{http://www.w3.org/2005/Atom}name') is not None]
    a_str = ', '.join(authors[:3]) if authors else ''
    print(f'• {html.unescape(title)} — {a_str}')
    print(f'  {link} ({published})')
    print()
" 2>/dev/null
}

# ─── MAIN ───────────────────────────────────────────────────────
init_state
state=$(load_state)

echo "# Stack Watch — $(now_iso)"
echo ""

# GitHub
echo "## GitHub Releases"
echo ""
state_updates=""
for entry in "${WATCH_REPOS[@]}"; do
  repo="${entry%%:*}"
  label="${entry#*:}"
  [[ "$repo" == "$label" ]] && label="$repo"
  output=$(check_releases "$repo" "$label" "$state" 2>&1 || true)
  findings=$(echo "$output" | grep -v "^STATE:" || true)
  su=$(echo "$output" | grep "^STATE:" || true)
  [[ -n "$findings" ]] && echo "$findings"
  state_updates+="$su"$'\n'
done

# arXiv
echo "## ArXiv (last 7 days)"
echo ""
for query in "${ARXIV_QUERIES[@]}"; do
  cat_short=$(echo "$query" | cut -d+ -f1 | sed 's/cat://' || echo "AI")
  echo "### $cat_short"
  check_arxiv "$query" "$state" || true
  echo ""
done

# Update state
if [[ -n "$state_updates" ]]; then
  while IFS='=' read -r key value; do
    [[ -z "$key" || -z "$value" ]] && continue
    safe_key="${key#STATE:}"
    state=$(echo "$state" | pl "
import json,sys
s = json.load(sys.stdin)
s['releases']['$safe_key'] = '$value'
s['last_run'] = '$(now_iso)'
print(json.dumps(s, indent=2))
" 2>/dev/null || echo "$state")
  done < <(echo "$state_updates")
  save_state <<< "$state"
fi

echo "---"
echo "stack-watch done at $(now_iso)"
