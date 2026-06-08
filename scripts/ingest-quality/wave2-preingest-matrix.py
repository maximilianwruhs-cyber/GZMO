#!/usr/bin/env python3
import json
import argparse
import sys
import re
from pathlib import Path

ALLOWED_EXTENSIONS = {
    "md", "txt", "yaml", "yml", "json", "toml",
    "py", "rs", "sh", "bash", "html", "htm", "xml", "csv", "tsv", "pdf"
}

def is_binary(path: Path) -> bool:
    if path.suffix.lower() == ".pdf":
        return False
    try:
        with open(path, "rb") as f:
            chunk = f.read(8192)
            return b"\x00" in chunk
    except Exception:
        return True

def main():
    parser = argparse.ArgumentParser(description="Wave-2 Pre-Ingest Matrix Evaluator")
    parser.add_argument("--manifest", required=True, help="Path to the wave2 manifest file")
    parser.add_argument("--out", required=True, help="Path to save the JSON output matrix")
    args = parser.parse_args()

    manifest_path = Path(args.manifest)
    if not manifest_path.exists():
        print(f"Error: manifest not found at {manifest_path}")
        sys.exit(1)

    harness_dir = Path(__file__).parent.resolve()
    yaml_path = harness_dir / "expected.yaml"
    wave1_basenames = set()
    if yaml_path.exists():
        try:
            import yaml
            expected_data = yaml.safe_load(yaml_path.read_text(encoding="utf-8"))
            for fname in expected_data.get("files", {}):
                wave1_basenames.add(Path(fname).name.lower())
        except Exception as e:
            print(f"Warning: could not parse expected.yaml: {e}")

    results = []
    
    total_files = 0
    passed_count = 0
    failed_count = 0
    duplicate_count = 0
    hold_count = 0

    with open(manifest_path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            # Manifest has: path sha256 (path may contain spaces, sha256 is the last word)
            parts = line.rsplit(maxsplit=1)
            file_path_str = parts[0]
            expected_hash = parts[1] if len(parts) > 1 else ""
            file_path = Path(file_path_str)
            
            total_files += 1
            filename = file_path.name
            basename_lower = filename.lower()
            
            s0_pass = True
            s0_reasons = []
            
            # S0: exists, size > 0, not binary
            if not file_path.exists():
                s0_pass = False
                s0_reasons.append("not_found")
                size = 0
            else:
                size = file_path.stat().st_size
                if size == 0:
                    s0_pass = False
                    s0_reasons.append("empty")
                if is_binary(file_path):
                    s0_pass = False
                    s0_reasons.append("binary")
                    
            # S1: size limit, extension check, binary check (from pre-ingest-gate.sh)
            s1_pass = s0_pass
            s1_reasons = []
            if s0_pass:
                if size < 10:
                    s1_pass = False
                    s1_reasons.append("too_small")
                if size > 5242880:
                    s1_pass = False
                    s1_reasons.append("too_large")
                ext = file_path.suffix.lstrip(".").lower()
                if ext not in ALLOWED_EXTENSIONS:
                    s1_pass = False
                    s1_reasons.append("disallowed_extension")

            # S2: extension/class classification (chat vs tech vs neg-control)
            path_str_lower = file_path_str.lower()
            if "chat_history" in path_str_lower or "chat_session" in path_str_lower or "chat" in path_str_lower:
                doc_class = "chat"
            elif "anti_" in path_str_lower or "negative_control" in path_str_lower:
                doc_class = "neg-control"
            else:
                doc_class = "tech"

            # S3: Duplicate of Wave-1 basename
            is_duplicate = basename_lower in wave1_basenames
            
            # Outcome determination:
            # - Failed: failed S0 or S1
            # - Duplicate: S3 duplicate
            # - Hold: passed but class is chat/neg-control or size > 120,000 chars
            # - Go: passed S0-S1, S2 is tech, not duplicate, size <= 120,000 chars
            
            status = "go"
            reason = ""
            
            if not s1_pass:
                status = "fail"
                reason = "S0/S1 validation failed: " + ",".join(s0_reasons + s1_reasons)
                failed_count += 1
            elif is_duplicate:
                status = "duplicate"
                reason = "Duplicate of Wave-1 basename"
                duplicate_count += 1
            elif doc_class in ("chat", "neg-control") or size > 120000:
                status = "hold"
                reasons = []
                if doc_class in ("chat", "neg-control"):
                    reasons.append(f"classified as {doc_class}")
                if size > 120000:
                    reasons.append("size > 120k chars")
                status = "hold"
                reason = "Hold: " + " and ".join(reasons)
                hold_count += 1
            else:
                passed_count += 1

            results.append({
                "path": file_path_str,
                "filename": filename,
                "size_bytes": size,
                "class": doc_class,
                "is_duplicate": is_duplicate,
                "s0_pass": s0_pass,
                "s1_pass": s1_pass,
                "status": status,
                "reason": reason
            })

    summary = {
        "total_files": total_files,
        "go": passed_count,
        "hold": hold_count,
        "duplicate": duplicate_count,
        "fail": failed_count
    }

    output_data = {
        "summary": summary,
        "files": results
    }

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(output_data, indent=2), encoding="utf-8")
    
    print(f"Matrix Evaluation Complete. Total: {total_files} | Go: {passed_count} | Hold: {hold_count} | Duplicate: {duplicate_count} | Fail: {failed_count}")
    print(f"Saved matrix JSON to {out_path}")

if __name__ == "__main__":
    main()
