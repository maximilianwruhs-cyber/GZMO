#!/usr/bin/env python3
import json
import os
import urllib.request
import sys
from pathlib import Path
import yaml

EMBED_URL = os.environ.get("EMBED_URL", "http://192.168.31.110:8081/v1/embeddings")
EMBED_MODEL = os.environ.get("EMBED_MODEL", "Qwen3-Embedding-0.6B-Q8_0.gguf")
QDRANT_BASE = os.environ.get("QDRANT_URL", "http://192.168.31.202:6333").rstrip("/")
QDRANT_COLLECTION = os.environ.get("QDRANT_COLLECTION", "honeypot")
QDRANT_URL = f"{QDRANT_BASE}/collections/{QDRANT_COLLECTION}/points/search"

def embed_text(text: str) -> list[float]:
    req_body = {
        "model": EMBED_MODEL,
        "input": text
    }
    req = urllib.request.Request(
        EMBED_URL,
        data=json.dumps(req_body).encode(),
        headers={"Content-Type": "application/json"},
        method="POST"
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        res = json.loads(resp.read().decode())
        return res["data"][0]["embedding"]

def search_qdrant(vector: list[float]) -> list[dict]:
    req_body = {
        "vector": vector,
        "limit": 5,
        "with_payload": True
    }
    req = urllib.request.Request(
        QDRANT_URL,
        data=json.dumps(req_body).encode(),
        headers={"Content-Type": "application/json"},
        method="POST"
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        res = json.loads(resp.read().decode())
        return res["result"]

def main():
    print(f"=== Running Sandbox Retrieval Probes (Qdrant: {QDRANT_COLLECTION}) ===")
    yaml_path = Path(__file__).resolve().parent / "sandbox-probes.yaml"
    if not yaml_path.exists():
        print(f"[FAIL] Could not find sandbox probes YAML at {yaml_path}")
        sys.exit(1)
        
    with open(yaml_path, "r") as f:
        config = yaml.safe_load(f)
        
    probes = config.get("probes", [])
    if not probes:
        print("[FAIL] No probes found in YAML")
        sys.exit(1)
        
    failed = False
    for probe in probes:
        query = probe["query"]
        must_contain = probe["must_contain"]
        min_score = probe.get("min_score", 0.35)
        
        print(f"\nQuery: \"{query}\"")
        try:
            vec = embed_text(query)
            results = search_qdrant(vec)
        except Exception as e:
            print(f"  [FAIL] Error during API calls: {e}")
            failed = True
            continue
            
        print(f"  Top results:")
        found_match = False
        matched_word = None
        for i, r in enumerate(results):
            score = r.get("score", 0.0)
            payload = r.get("payload", {})
            content = payload.get("content", "")
            print(f"    [{i+1}] (Score: {score:.4f}) {content}")
            
            content_lower = content.lower()
            for mc in must_contain:
                if mc.lower() in content_lower and score >= min_score:
                    found_match = True
                    matched_word = mc
                    break
        
        if found_match:
            print(f"  [PASS] Found matching content (contains variant of '{matched_word}')")
        else:
            print(f"  [FAIL] Did not retrieve expected facts containing: {must_contain}")
            failed = True
            
    if failed:
        sys.exit(1)
    else:
        print("\n[SUCCESS] All sandbox retrieval probes passed!")
        sys.exit(0)

if __name__ == "__main__":
    main()
