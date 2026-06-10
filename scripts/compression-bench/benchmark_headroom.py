#!/usr/bin/env python3
import json
import os
import pathlib
import sys
import tiktoken

# Ensure telemetry is off
os.environ["HEADROOM_TELEMETRY"] = "off"

def main():
    from headroom import compress, CompressConfig

    # Initialize tiktoken encoder
    try:
        encoder = tiktoken.get_encoding("cl100k_base")
    except Exception as e:
        print(f"Error loading tiktoken encoder: {e}", file=sys.stderr)
        sys.exit(1)

    bench_dir = pathlib.Path(__file__).parent.resolve()
    fixtures_dir = bench_dir / "fixtures"
    output_dir = bench_dir / "output"
    output_dir.mkdir(parents=True, exist_ok=True)

    results = []
    cfg = CompressConfig(protect_recent=0)

    print(f"Running compression benchmark on fixtures in: {fixtures_dir}")
    print(f"{'Fixture File':<35} | {'Original (tok)':<15} | {'Compressed (tok)':<15} | {'Savings (%)':<12}")
    print("-" * 85)

    all_savings = []

    for path in sorted(fixtures_dir.glob("*")):
        if not path.is_file():
            continue

        raw_content = path.read_text(errors="replace")
        before_tokens = len(encoder.encode(raw_content))

        # Detect route type (JSON or text) for tool/mcp output
        role = "tool"
        name = "tool"
        if path.name.endswith(".json"):
            name = "db"
        elif "grep" in path.name:
            name = "shell"

        try:
            res = compress([{"role": role, "name": name, "content": raw_content}], config=cfg)
            compressed_content = res.messages[0]["content"]
            after_tokens = len(encoder.encode(compressed_content))
        except Exception as e:
            print(f"Error compressing {path.name}: {e}", file=sys.stderr)
            compressed_content = raw_content
            after_tokens = before_tokens

        savings_pct = round(100.0 * (1.0 - (after_tokens / before_tokens)), 2) if before_tokens > 0 else 0.0
        all_savings.append(savings_pct)

        print(f"{path.name:<35} | {before_tokens:<15} | {after_tokens:<15} | {savings_pct:<11}%")

        results.append({
            "file": path.name,
            "before_tokens": before_tokens,
            "headroom_tokens": after_tokens,
            "headroom_savings_pct": savings_pct,
        })

    # Sort savings to calculate median
    all_savings.sort()
    n = len(all_savings)
    if n > 0:
        if n % 2 == 1:
            median_savings = all_savings[n // 2]
        else:
            median_savings = (all_savings[n // 2 - 1] + all_savings[n // 2]) / 2.0
    else:
        median_savings = 0.0

    print("-" * 85)
    print(f"Median savings: {median_savings:.2f}%")

    out_file = output_dir / "results.json"
    with open(out_file, "w") as f:
        json.dump({
            "median_savings_pct": median_savings,
            "results": results
        }, f, indent=2)
    print(f"Saved benchmark results to {out_file}")

if __name__ == "__main__":
    main()
