#!/bin/bash

set -e

echo "[*] Running TALON benchmarks..."

cd "$(dirname "$0")/../.."

echo "[*] Building TALON in release mode..."
cargo build --release --quiet 2>&1 | head -20

echo "[*] Running criterion benchmarks..."
cargo bench --bench vs_pwntools_bench --quiet -- --output-format bencher > scripts/benchmarks/talon_results_raw.txt 2>&1 || true

echo "[*] Extracting benchmark results..."
python3 scripts/benchmarks/extract_talon_results.py

echo "[+] TALON benchmark complete"
echo "[+] Results written to scripts/benchmarks/talon_results.txt"
