#!/usr/bin/env python3
import json
import pathlib
import sys

def main():
    bench_dir = pathlib.Path(__file__).parent.resolve()
    rust_results_file = bench_dir / "output" / "rust_results.json"
    
    if not rust_results_file.exists():
        print(f"Error: {rust_results_file} not found. Run cargo test first.", file=sys.stderr)
        sys.exit(1)
        
    with open(rust_results_file, "r") as f:
        results = json.load(f)
        
    print(f"Loaded {len(results)} benchmark results from Rust compressor")
    print(f"{'Fixture File':<35} | {'Original (tok)':<15} | {'Compressed (tok)':<15} | {'Savings (%)':<12}")
    print("-" * 85)
    
    savings = []
    probes_passed = 0
    total_probes = 0
    
    # Define probe checks for specific files to verify semantic correctness
    probes = {
        "gzmo_memory_search.txt": ["RECALL", "TRUNCATED TO BUDGET"],
        "read_file_rust.txt": ["AgentLoopConfig", "TRUNCATED TO BUDGET"],
        "wiki_search.txt": ["drive-research-hermes-compression-and-bol-architecture", "TRUNCATED TO BUDGET"],
        "mcp_status.json": ["healthy", "memory_mcp"],
        "subagent_summary.txt": ["Research on local Redis integration", "TRUNCATED TO BUDGET"],
        "web_search.txt": ["LLMLingua", "TRUNCATED TO BUDGET"]
    }
    
    for r in results:
        file = r["file"]
        before = r["before_tokens"]
        after = r["after_tokens"]
        pct = r["savings_pct"]
        text = r["text"]
        
        savings.append(pct)
        print(f"{file:<35} | {before:<15} | {after:<15} | {pct:<11.2f}%")
        
        if file in probes:
            for probe in probes[file]:
                total_probes += 1
                if probe in text:
                    probes_passed += 1
                else:
                    print(f"  [FAIL] Missing probe '{probe}' in compressed {file}", file=sys.stderr)
                    
    savings.sort()
    n = len(savings)
    if n > 0:
        if n % 2 == 1:
            median_savings = savings[n // 2]
        else:
            median_savings = (savings[n // 2 - 1] + savings[n // 2]) / 2.0
    else:
        median_savings = 0.0
        
    print("-" * 85)
    print(f"Median savings: {median_savings:.2f}%")
    
    fidelity_pct = (probes_passed / total_probes) * 100.0 if total_probes > 0 else 100.0
    print(f"Probe fidelity: {probes_passed}/{total_probes} ({fidelity_pct:.1f}%)")
    
    # Save combined results
    out_file = bench_dir / "output" / "results.json"
    with open(out_file, "w") as f:
        json.dump({
            "median_savings_pct": median_savings,
            "fidelity_pct": fidelity_pct,
            "results": results
        }, f, indent=2)
        
    failed = False
    if median_savings < 40.0:
        print(f"Error: Median savings {median_savings:.2f}% is less than the required 40%", file=sys.stderr)
        failed = True
    if fidelity_pct < 90.0:
        print(f"Error: Probe fidelity {fidelity_pct:.1f}% is less than the required 90%", file=sys.stderr)
        failed = True
        
    if failed:
        sys.exit(1)
    else:
        print("All compression verification gates passed successfully!")
        sys.exit(0)

if __name__ == "__main__":
    main()
