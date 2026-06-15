#!/usr/bin/env bash
# Incremental Pi knowledge-base reindex (same logic as knowledge-base.ts extension).
set -euo pipefail

FORCE="${1:-false}"
export PI_KB_REINDEX_FORCE="${FORCE}"

python3 <<'PY'
import json, hashlib, os, pathlib, urllib.request, sys

home = pathlib.Path.home()
cfg_path = home / ".pi/agent/knowledge-base.json"
defaults = {
    "docsDir": str(home / "Schreibtisch/knowledge"),
    "embedUrl": "http://192.168.31.110:8081/v1/embeddings",
    "embedModel": "gzmo-embed",
    "qdrantUrl": "http://192.168.31.202:6333",
    "collection": "knowledge",
    "skipDirs": ["archive", ".gzmo_converted"],
}
if cfg_path.exists():
    raw = json.loads(cfg_path.read_text())
    defaults.update({k: raw[k] for k in raw if k in defaults or k == "skipDirs"})
for env, key in [
    ("PI_KB_EMBED_URL", "embedUrl"),
    ("PI_KB_DOCS_DIR", "docsDir"),
    ("PI_KB_QDRANT_URL", "qdrantUrl"),
    ("PI_KB_COLLECTION", "collection"),
]:
    if os.environ.get(env):
        defaults[key] = os.environ[env]

DOCS = pathlib.Path(defaults["docsDir"].replace("~/", str(home) + "/"))
EMBED_URL = defaults["embedUrl"]
MODEL = defaults["embedModel"]
QDRANT = defaults["qdrantUrl"].rstrip("/")
COLLECTION = defaults["collection"]
SKIP = set(defaults["skipDirs"])
STATE_FILE = home / ".pi/agent/knowledge-state.json"
TEXT_EXT = {".md", ".markdown", ".txt", ".rst", ".org", ".py", ".ts", ".tsx", ".js", ".jsx",
            ".json", ".yaml", ".yml", ".toml", ".html", ".css", ".sh", ".bash", ".rs", ".go",
            ".java", ".c", ".cpp", ".h", ".hpp", ".sql", ".cfg", ".ini", ".conf"}
CHUNK, OVER, BATCH = 800, 100, 16
INDEX_VERSION = 2
MAX_BYTES = 2_000_000
FORCE = os.environ.get("PI_KB_REINDEX_FORCE", "false").lower() in ("1", "true", "yes")

def uuid_from(s):
    h = hashlib.md5(s.encode()).hexdigest()
    return f"{h[:8]}-{h[8:12]}-{h[12:16]}-{h[16:20]}-{h[20:32]}"

def chunks(text):
    out, n, i = [], len(text), 0
    while i < n:
        end = min(i + CHUNK, n)
        if end < n:
            nl = text[i:end].rfind("\n")
            if nl > CHUNK * 0.5:
                end = i + nl + 1
        c = text[i:end].strip()
        if c:
            out.append(c)
        if end >= n:
            break
        i = max(0, end - OVER)
    return out

def http(method, url, body=None, timeout=120):
    data = None if body is None else json.dumps(body).encode()
    req = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json"}, method=method)
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.load(r)

def embed_at(url, texts):
    j = http("POST", url, {"model": MODEL, "input": texts})
    return [d["embedding"] for d in sorted(j["data"], key=lambda x: x["index"])]

def embed(texts):
    return embed_at(EMBED_URL, texts)

def walk(root, include_skip):
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if not d.startswith(".")]
        if not include_skip:
            dirnames[:] = [d for d in dirnames if d not in SKIP]
        for f in filenames:
            if pathlib.Path(f).suffix.lower() in TEXT_EXT:
                yield pathlib.Path(dirpath) / f

def index_file(fp):
    rel = str(fp.relative_to(DOCS))
    raw = fp.read_text(encoding="utf-8", errors="ignore")
    chs = chunks(raw)
    http("POST", f"{QDRANT}/collections/{COLLECTION}/points/delete",
         {"filter": {"must": [{"key": "path", "match": {"value": rel}}]}})
    if not chs:
        return 0
    points = []
    for b in range(0, len(chs), BATCH):
        batch = chs[b : b + BATCH]
        vecs = embed(batch)
        for j, text in enumerate(batch):
            idx = b + j
            points.append({"id": uuid_from(f"{rel}#{idx}"), "vector": vecs[j],
                           "payload": {"path": rel, "chunk": idx, "text": text}})
    http("PUT", f"{QDRANT}/collections/{COLLECTION}/points?wait=true", {"points": points})
    return len(chs)

state = json.loads(STATE_FILE.read_text()) if STATE_FILE.exists() else {}
files = list(walk(DOCS, FORCE))
present = {str(f.relative_to(DOCS)) for f in files}
changed, total_chunks = 0, 0
for fp in files:
    rel = str(fp.relative_to(DOCS))
    st = fp.stat()
    if st.st_size > MAX_BYTES:
        continue
    sig = f"{INDEX_VERSION}:{round(st.st_mtime_ns / 1e6)}:{st.st_size}"
    if not FORCE and state.get(rel) == sig:
        continue
    n = index_file(fp)
    state[rel] = sig
    changed += 1
    total_chunks += n
    print(f"  indexed {rel} ({n} chunks)")
removed = 0
for rel in list(state):
    if rel not in present:
        http("POST", f"{QDRANT}/collections/{COLLECTION}/points/delete",
             {"filter": {"must": [{"key": "path", "match": {"value": rel}}]}})
        del state[rel]
        removed += 1
STATE_FILE.write_text(json.dumps(state, indent=2) + "\n")
print(f"Reindex complete: {changed} file(s), {total_chunks} chunk(s), {removed} removed. embed={EMBED_URL}")
PY
