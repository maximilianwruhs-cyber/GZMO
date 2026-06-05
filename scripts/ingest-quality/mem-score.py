#!/usr/bin/env python3
"""MemScore one-liner: dual recall tracks, ingest faithfulness, composite (informational)."""

import json
import sys
from pathlib import Path
from typing import Optional


def load_json(path: Path) -> Optional[dict]:
    if not path.is_file():
        return None
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None


def main() -> None:
    harness_dir = Path(__file__).parent.resolve()
    default_report = harness_dir / "report.json"
    baseline_report = harness_dir / "reports" / "baseline-post-m2.json"
    metrics_path = harness_dir / "reports" / "recall-metrics.json"

    report_path = default_report if default_report.exists() else baseline_report
    if not report_path.exists():
        print(f"Error: No report found at {default_report} or {baseline_report}")
        sys.exit(1)

    report = load_json(report_path)
    if report is None:
        print("Error reading/parsing report")
        sys.exit(1)

    summary = report.get("summary", {})
    mem_score = summary.get("mem_score", {})

    recall_at_5 = mem_score.get("recall_at_5")
    recall_rrf = mem_score.get("recall_at_5_rrf")
    recall_rrf_strict = mem_score.get("recall_at_5_rrf_strict")
    recall_golden = mem_score.get("recall_at_5_golden")
    recall_qdrant = mem_score.get("recall_at_5_qdrant")

    metrics = load_json(metrics_path)
    if metrics and metrics.get("latest"):
        latest = metrics["latest"]
        if recall_rrf is None and "rrf" in latest:
            recall_rrf = latest["rrf"].get("recall_at_5")
        if recall_rrf_strict is None and "rrf_strict" in latest:
            recall_rrf_strict = latest["rrf_strict"].get("recall_at_5")
        if recall_golden is None and "golden" in latest:
            recall_golden = latest["golden"].get("recall_at_5")
        if recall_qdrant is None and "qdrant" in latest:
            recall_qdrant = latest["qdrant"].get("recall_at_5")

    if recall_at_5 is None:
        recall_files = sorted((harness_dir / "reports").glob("recall5-baseline-*.json"), reverse=True)
        if recall_files:
            recall_data = load_json(recall_files[0])
            if recall_data:
                recall_at_5 = recall_data.get("summary", {}).get("mem_score", {}).get("recall_at_5")

    total_ext = summary.get("entities_extracted", 0) + summary.get("relations_extracted", 0)
    total_prom = summary.get("entities_promoted", 0) + summary.get("relations_promoted", 0)
    faithfulness_ingest = total_prom / total_ext if total_ext > 0 else 1.0
    noise_ratio = 1.0 - faithfulness_ingest
    anti_pattern_count = summary.get("anti_entities_found_count", 0)

    faithfulness_judge = mem_score.get("faithfulness_judge")
    faithfulness_context = mem_score.get("faithfulness_context")
    faithfulness_corpus = mem_score.get("faithfulness_corpus")
    judge_latest = harness_dir / "reports" / "faithfulness-judge-latest.json"
    judge_doc = load_json(judge_latest)
    if judge_doc:
        js = judge_doc.get("summary", {})
        if faithfulness_judge is None:
            faithfulness_judge = js.get("faithfulness_judge")
        if faithfulness_context is None:
            faithfulness_context = js.get("faithfulness_context")
        if faithfulness_corpus is None:
            faithfulness_corpus = js.get("faithfulness_corpus")

    # Composite prefers the honest strict recall and the context faithfulness gate.
    recall_for_composite = (
        recall_rrf_strict if recall_rrf_strict is not None
        else recall_rrf if recall_rrf is not None
        else recall_at_5
    )
    faith_for_composite = (
        faithfulness_context if faithfulness_context is not None
        else faithfulness_judge
    )
    anti_ok = 1.0 if anti_pattern_count == 0 else 0.0
    composite = None
    if recall_for_composite is not None:
        if faith_for_composite is not None:
            composite = (
                0.5 * float(recall_for_composite)
                + 0.3 * float(faith_for_composite)
                + 0.1 * (1.0 - noise_ratio)
                + 0.1 * anti_ok
            )
        else:
            composite = (
                0.5 * float(recall_for_composite)
                + 0.3 * faithfulness_ingest
                + 0.2 * anti_ok
            )

    def fmt_recall(val) -> str:
        return f"{val:.3f}" if val is not None else "n/a"

    print(
        "MemScore: "
        f"recall_rrf={fmt_recall(recall_rrf)} | "
        f"recall_rrf_strict={fmt_recall(recall_rrf_strict)} | "
        f"recall_golden={fmt_recall(recall_golden)} | "
        f"recall_qdrant={fmt_recall(recall_qdrant)} | "
        f"faith_context={fmt_recall(faithfulness_context)} | "
        f"faith_corpus={fmt_recall(faithfulness_corpus)} | "
        f"faith_ingest={faithfulness_ingest:.3f} | "
        f"anti_violations={anti_pattern_count}"
    )
    if composite is not None:
        print(f"MemScore_composite(informational)={composite:.3f}")

    if "--verbose" in sys.argv or "-v" in sys.argv:
        print("\n--- MemScore breakdown ---")
        print(f"Report: {report_path}")
        print(f"Recall metrics: {metrics_path}")
        print(f"Golden must-entities: {summary.get('must_entities_recall', 0.0) * 100:.1f}%")
        print(f"Golden must-facts:     {summary.get('must_facts_recall', 0.0) * 100:.1f}%")
        print(
            "Faithfulness judge: "
            "python3 scripts/ingest-quality/faithfulness-judge.py "
            "--mode llm --write-report --merge-mem-score"
        )


if __name__ == "__main__":
    main()
