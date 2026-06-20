#!/usr/bin/env bash
# arXiv live retrieval skill (Tier 2 network exception).
# Usage:
#   ./skills/skill_arxiv.sh status
#   ./skills/skill_arxiv.sh search --query "cat:cs.AI AND large language model"
#   ./skills/skill_arxiv.sh harvest --set cs.AI --from 2026-01-01
#   ./skills/skill_arxiv.sh fetch --id 2605.16562
#   ./skills/skill_arxiv.sh graph --id arXiv:2605.16562

set -euo pipefail

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SKILLS_DIR/.." && pwd)"
CACHE_DIR="${GZMO_ARXIV_CACHE:-$ROOT_DIR/data/arxiv-cache}"
META_FILE="$CACHE_DIR/metadata.jsonl"
RATE_SLEEP="${ARXIV_RATE_SLEEP:-0.25}"

mkdir -p "$CACHE_DIR"

arxiv_api() {
  local query="$1"
  local max="${2:-10}"
  local encoded
  encoded=$(printf '%s' "$query" | jq -sRr @uri)
  curl -fsSL --max-time 30 \
    "http://export.arxiv.org/api/query?search_query=${encoded}&start=0&max_results=${max}"
}

oai_pmh() {
  local url="$1"
  sleep "$RATE_SLEEP"
  curl -fsSL --max-time 60 "$url"
}

# Map shorthand category (cs.AI) to OAI-PMH setSpec (cs:cs:AI).
oai_set_spec() {
  local raw="$1"
  if [[ "$raw" == *:* ]]; then
    printf '%s' "$raw"
    return
  fi
  if [[ "$raw" == *.* ]]; then
    local subject="${raw%%.*}"
    local category="${raw#*.}"
    printf '%s:%s:%s' "$subject" "$subject" "$category"
    return
  fi
  printf '%s' "$raw"
}

usage() {
  cat <<'EOF'
arXiv skill (live network — Tier 2 exception)

  status
  search --query <expr> [--max N]
  harvest --set <category> [--from YYYY-MM-DD]
  ingest-harvest [--batch-size N] [--max-records N] [--no-ingest]
  fetch --id <arxiv_id>
  graph --id <arxiv_id_or_doi_prefix>
EOF
}

cmd="${1:-status}"
shift || true

case "$cmd" in
  status)
    count=0
    if [[ -f "$META_FILE" ]]; then
      count=$(wc -l < "$META_FILE" | tr -d ' ')
    fi
    jq -n \
      --arg cache_dir "$CACHE_DIR" \
      --arg meta "$META_FILE" \
      --argjson records "$count" \
      '{cache_dir: $cache_dir, metadata_file: $meta, metadata_records: $records, network_tier: "exception"}'
    ;;

  search)
    query=""
    max=10
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --query) query="${2:-}"; shift 2 ;;
        --max) max="${2:-10}"; shift 2 ;;
        *) echo "Unknown arg: $1"; usage; exit 1 ;;
      esac
    done
    if [[ -z "$query" ]]; then
      echo "ERROR: --query required"
      exit 1
    fi
    xml=$(arxiv_api "$query" "$max")
    ARXIV_XML="$xml" python3 -c '
import json, os, xml.etree.ElementTree as ET
ns = {"atom": "http://www.w3.org/2005/Atom", "arxiv": "http://arxiv.org/schemas/atom"}
root = ET.fromstring(os.environ["ARXIV_XML"])
entries = []
for entry in root.findall("atom:entry", ns):
    eid = (entry.findtext("atom:id", default="", namespaces=ns) or "").split("/abs/")[-1]
    title = (entry.findtext("atom:title", default="", namespaces=ns) or "").strip().replace("\n", " ")
    published = entry.findtext("atom:published", default="", namespaces=ns) or ""
    summary = (entry.findtext("atom:summary", default="", namespaces=ns) or "").strip().replace("\n", " ")[:280]
    entries.append({"id": eid, "title": title, "published": published, "summary": summary})
print(json.dumps({"query_ok": True, "count": len(entries), "entries": entries}, ensure_ascii=False))
'
    ;;

  harvest)
    set_name=""
    from_date=""
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --set) set_name="${2:-}"; shift 2 ;;
        --from) from_date="${2:-}"; shift 2 ;;
        *) echo "Unknown arg: $1"; usage; exit 1 ;;
      esac
    done
    if [[ -z "$set_name" ]]; then
      echo "ERROR: --set required (e.g. cs.AI)"
      exit 1
    fi
    oai_set="$(oai_set_spec "$set_name")"
    url="https://oaipmh.arxiv.org/oai?verb=ListRecords&metadataPrefix=arXiv&set=${oai_set}"
    if [[ -n "$from_date" ]]; then
      url="${url}&from=${from_date}"
    fi
    xml_tmp="$(mktemp "${TMPDIR:-/tmp}/arxiv-oai-XXXXXX.xml")"
    trap 'rm -f "$xml_tmp"' EXIT
    oai_pmh "$url" > "$xml_tmp"
    ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    ARXIV_XML_FILE="$xml_tmp" ARXIV_META_FILE="$META_FILE" ARXIV_SET="$set_name" ARXIV_OAI_SET="$oai_set" ARXIV_TS="$ts" python3 -c '
import json, os, xml.etree.ElementTree as ET
from pathlib import Path
meta_path = Path(os.environ["ARXIV_META_FILE"])
root = ET.parse(os.environ["ARXIV_XML_FILE"]).getroot()
ns = {"oai": "http://www.openarchives.org/OAI/2.0/"}
arxiv_ns = "{http://arxiv.org/OAI/arXiv/}"
records = []
for rec in root.findall(".//oai:record", ns):
    header = rec.find("oai:header", ns)
    if header is None:
        continue
    ident = header.findtext("oai:identifier", default="", namespaces=ns)
    datestamp = header.findtext("oai:datestamp", default="", namespaces=ns)
    md = rec.find(f".//{arxiv_ns}arXiv")
    title = md.findtext(f"{arxiv_ns}title", default="") if md is not None else ""
    records.append({"identifier": ident, "datestamp": datestamp, "title": title.strip(), "harvested_at": os.environ["ARXIV_TS"]})
with meta_path.open("a", encoding="utf-8") as f:
    for r in records:
        f.write(json.dumps(r, ensure_ascii=False) + "\n")
print(json.dumps({"harvested": len(records), "set": os.environ["ARXIV_SET"], "oai_set": os.environ.get("ARXIV_OAI_SET", ""), "metadata_file": str(meta_path)}, ensure_ascii=False))
'
    rm -f "$xml_tmp"
    trap - EXIT
    ;;

  ingest-harvest)
    batch_size=300
    max_records=0
    do_ingest=1
    set_label="cs.AI"
    from_date="2026-06-01"
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --batch-size) batch_size="${2:-300}"; shift 2 ;;
        --max-records) max_records="${2:-0}"; shift 2 ;;
        --set) set_label="${2:-cs.AI}"; shift 2 ;;
        --from) from_date="${2:-}"; shift 2 ;;
        --no-ingest) do_ingest=0; shift ;;
        *) echo "Unknown arg: $1"; usage; exit 1 ;;
      esac
    done
    curated_out="$HOME/Schreibtisch/knowledge/curated/thema_004-arxiv-harvest-${set_label//./}.md"
    build_args=(--meta "$META_FILE" --out "$curated_out" --set "$set_label" --batch-size "$batch_size")
    [[ "$max_records" -gt 0 ]] && build_args+=(--max-records "$max_records")
    [[ -n "$from_date" ]] && build_args+=(--from-date "$from_date")
    mapfile -t curated_files < <(
      python3 "$ROOT_DIR/scripts/build-arxiv-harvest-curated.py" "${build_args[@]}" | awk '/^\/.*\.md$/ { print }'
    )
    if [[ ${#curated_files[@]} -eq 0 ]]; then
      echo "ERROR: no curated files produced"
      exit 1
    fi
    if [[ "$do_ingest" -eq 0 ]]; then
      jq -n --argjson files "$(printf '%s\n' "${curated_files[@]}" | jq -R . | jq -s .)" \
        '{curated_files: $files, ingest: false}'
      exit 0
    fi
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
      --arg meta "$META_FILE" \
      '{batches_ingested: $batches, curated_files: $files, metadata_file: $meta}'
    ;;

  fetch)
    id=""
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --id) id="${2:-}"; shift 2 ;;
        *) echo "Unknown arg: $1"; usage; exit 1 ;;
      esac
    done
    id="${id#arXiv:}"
    if [[ -z "$id" ]]; then
      echo "ERROR: --id required"
      exit 1
    fi
    xml=$(arxiv_api "id:$id" 1)
    ARXIV_XML="$xml" python3 -c '
import json, os, xml.etree.ElementTree as ET
ns = {"atom": "http://www.w3.org/2005/Atom"}
root = ET.fromstring(os.environ["ARXIV_XML"])
entry = root.find("atom:entry", ns)
if entry is None:
    print(json.dumps({"found": False}))
    raise SystemExit(0)
data = {
    "found": True,
    "id": (entry.findtext("atom:id", namespaces=ns) or "").split("/abs/")[-1],
    "title": (entry.findtext("atom:title", namespaces=ns) or "").strip(),
    "published": entry.findtext("atom:published", namespaces=ns),
    "pdf": next((l.get("href") for l in entry.findall("atom:link", ns) if l.get("title") == "pdf"), None),
    "summary": (entry.findtext("atom:summary", namespaces=ns) or "").strip()[:500],
}
print(json.dumps(data, ensure_ascii=False))
'
    ;;

  graph)
    id=""
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --id) id="${2:-}"; shift 2 ;;
        *) echo "Unknown arg: $1"; usage; exit 1 ;;
      esac
    done
    id="${id#arXiv:}"
    if [[ -z "$id" ]]; then
      echo "ERROR: --id required"
      exit 1
    fi
    sleep "$RATE_SLEEP"
    body=$(curl -fsSL --max-time 30 "https://api.semanticscholar.org/graph/v1/paper/arXiv:${id}?fields=title,year,citationCount,referenceCount,externalIds")
    echo "$body"
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
