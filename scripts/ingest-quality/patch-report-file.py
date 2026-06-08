#!/usr/bin/env python3
import json
import subprocess
import sys
from pathlib import Path

# Paths
root = Path(__file__).resolve().parent.parent.parent
report_path = root / "scripts/ingest-quality/report.json"
expected_path = root / "scripts/ingest-quality/expected.yaml"

if len(sys.argv) < 2:
    print("Usage: patch-report-file.py <file_to_eval>")
    sys.exit(1)

file_to_eval = sys.argv[1]
file_name = Path(file_to_eval).name

print(f"[*] Evaluating {file_name}...")
# Run ingest-eval
res = subprocess.run(
    [str(root / "target/release/gzmo"), "ingest-eval", file_to_eval],
    capture_output=True,
    text=True,
    check=True
)

# Parse output (skip any non-JSON logs at startup)
lines = res.stdout.splitlines()
json_start = 0
for idx, line in enumerate(lines):
    if line.strip().startswith("{"):
        json_start = idx
        break
json_str = "\n".join(lines[json_start:])
new_data = json.loads(json_str)
new_file_report = new_data["files"][0]

# Read existing report
with open(report_path, "r") as f:
    report = json.load(f)

# Find and replace the file entry
replaced = False
for idx, file_entry in enumerate(report["files"]):
    if file_entry["file_name"] == file_name:
        report["files"][idx] = new_file_report
        replaced = True
        print(f"[+] Replaced {file_name} in report.json")
        break

if not replaced:
    print(f"[!] Warning: {file_name} not found in report.json files list")

# Recalculate summary metrics
files = report["files"]
total_files = len(files)
golden_files = 0
entities_extracted = sum(f["entities_extracted"] for f in files)
relations_extracted = sum(f["relations_extracted"] for f in files)
entities_promoted = sum(f["entities_promoted"] for f in files)
relations_promoted = sum(f["relations_promoted"] for f in files)

zero_entity_files = sum(1 for f in files if f["entities_promoted"] == 0)
zero_relation_files = sum(1 for f in files if f["relations_promoted"] == 0)

relation_promotion_rate = relations_promoted / relations_extracted if relations_extracted > 0 else 0.0

# Recalculate golden contract details if available
import yaml
with open(expected_path, "r") as f:
    expected = yaml.safe_load(f)

sum_must_entities_total = 0
sum_must_entities_found = 0
sum_must_facts_total = 0
sum_must_facts_found = 0
anti_entities_found_count = 0

# Import rescore_golden helper
sys.path.append(str(root / "scripts/ingest-quality"))
import importlib
rescore_golden = importlib.import_module("rescore-golden")

# For each file in report, if it is in expected.yaml, score it
for f in files:
    fname = f["file_name"]
    # We also need to update its evaluation dict
    if fname in expected["files"]:
        golden_files += 1
        rules = expected["files"][fname]
        
        entities = f.get("verified_entities") or []
        facts = f.get("verified_facts") or []
        relations = rescore_golden.parse_relations(f.get("verified_relations") or [])
        
        missing_e = [m for m in rules.get("must_entities", []) if not rescore_golden.entity_found(m, entities, facts, relations)]
        missing_f = [m for m in rules.get("must_fact_substrings", []) if not rescore_golden.fact_found(m, facts)]
        found_anti = []
        for anti in rules.get("anti_entities", []):
            found_anti.extend(rescore_golden.anti_entity_hits(anti, entities))
            
        must_e_total = len(rules.get("must_entities", []))
        must_e_found = must_e_total - len(missing_e)
        must_f_total = len(rules.get("must_fact_substrings", []))
        must_f_found = must_f_total - len(missing_f)
        
        score_e = must_e_found / must_e_total if must_e_total > 0 else 1.0
        score_f = must_f_found / must_f_total if must_f_total > 0 else 1.0
        anti_penalty = 0.5 if found_anti else 0.0
        score = max(0.5 * score_e + 0.5 * score_f - anti_penalty, 0.0)
        
        f["evaluation"] = {
            "must_entities_total": must_e_total,
            "must_entities_found": must_e_found,
            "must_entities_missing": missing_e,
            "must_facts_total": must_f_total,
            "must_facts_found": must_f_found,
            "must_facts_missing": missing_f,
            "anti_entities_found": found_anti,
            "score": score
        }
        
        sum_must_entities_total += must_e_total
        sum_must_entities_found += must_e_found
        sum_must_facts_total += must_f_total
        sum_must_facts_found += must_f_found
        anti_entities_found_count += len(found_anti)

must_entities_recall = sum_must_entities_found / sum_must_entities_total if sum_must_entities_total > 0 else 0.0
must_facts_recall = sum_must_facts_found / sum_must_facts_total if sum_must_facts_total > 0 else 0.0

report["summary"] = {
    "total_files": total_files,
    "golden_files": golden_files,
    "entities_extracted": entities_extracted,
    "relations_extracted": relations_extracted,
    "entities_promoted": entities_promoted,
    "relations_promoted": relations_promoted,
    "zero_entity_files": zero_entity_files,
    "zero_relation_files": zero_relation_files,
    "relation_promotion_rate": relation_promotion_rate,
    "must_entities_recall": must_entities_recall,
    "must_facts_recall": must_facts_recall,
    "anti_entities_found_count": anti_entities_found_count
}

# Write updated report back
with open(report_path, "w") as f:
    json.dump(report, f, indent=2)
print("[+] report.json summary and file entries updated successfully.")
