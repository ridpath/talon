#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "================================================================"
echo "  TALON vs Pwntools Performance Benchmark Suite"
echo "================================================================"
echo ""

echo "[*] Step 1/3: Running TALON benchmarks..."
echo "-----------------------------------------------------------"
bash bench_talon.sh
echo ""

echo "[*] Step 2/3: Running Pwntools benchmarks..."
echo "-----------------------------------------------------------"
if command -v python3 &> /dev/null; then
    python3 bench_pwntools.py
elif command -v python &> /dev/null; then
    python bench_pwntools.py
else
    echo "[!] Python not found. Skipping pwntools benchmarks."
    echo "[!] Install Python and pwntools to run full comparison."
fi
echo ""

echo "[*] Step 3/3: Generating comparison report..."
echo "-----------------------------------------------------------"
if command -v python3 &> /dev/null; then
    python3 compare.py
elif command -v python &> /dev/null; then
    python compare.py
else
    echo "[!] Cannot generate comparison - Python required"
fi
echo ""

echo "================================================================"
echo "  Benchmark Complete"
echo "================================================================"
echo ""
echo "[+] Results:"
echo "    - TALON results:     talon_results.txt"
echo "    - Pwntools results:  pwntools_results.txt"
echo "    - Comparison report: BENCHMARKS.md"
echo ""

if [ -f "BENCHMARKS.md" ]; then
    echo "[*] Summary from BENCHMARKS.md:"
    echo "-----------------------------------------------------------"
    grep -A 3 "Performance Summary" BENCHMARKS.md || true
    echo "-----------------------------------------------------------"
fi

echo ""
echo "[+] To view full report: cat $SCRIPT_DIR/BENCHMARKS.md"
