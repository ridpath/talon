#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "========================================="
echo "TALON Performance Benchmarking Suite"
echo "========================================="
echo ""

OUTPUT_DIR="benchmark-results"
mkdir -p "$OUTPUT_DIR"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$OUTPUT_DIR/benchmark_report_$TIMESTAMP.md"

echo "# TALON Benchmark Report" > "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "**Date:** $(date)" >> "$REPORT_FILE"
echo "**Platform:** $(uname -s) $(uname -m)" >> "$REPORT_FILE"
echo "**Rust Version:** $(rustc --version)" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

run_benchmark() {
    local bench_name=$1
    local bench_file=$2
    
    echo ""
    echo "Running $bench_name benchmarks..."
    echo "======================================"
    
    cargo bench --bench "$bench_file" -- --output-format bencher | tee "$OUTPUT_DIR/${bench_file}_${TIMESTAMP}.txt"
    
    echo "" >> "$REPORT_FILE"
    echo "## $bench_name" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo '```' >> "$REPORT_FILE"
    cat "$OUTPUT_DIR/${bench_file}_${TIMESTAMP}.txt" | grep "test" | head -30 >> "$REPORT_FILE"
    echo '```' >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
}

run_benchmark "Parser" "parser_bench"
run_benchmark "Interpreter" "interpreter_bench"
run_benchmark "Binary Analysis" "binary_analysis_bench"
run_benchmark "ROP Tools" "rop_bench"

echo ""
echo "========================================="
echo "Performance Summary"
echo "========================================="

echo "" >> "$REPORT_FILE"
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "## Summary" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "All benchmarks completed successfully. Results are stored in:" >> "$REPORT_FILE"
echo "- Report: \`$REPORT_FILE\`" >> "$REPORT_FILE"
echo "- Detailed results: \`$OUTPUT_DIR/\`" >> "$REPORT_FILE"
echo "- Criterion HTML reports: \`target/criterion/\`" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo ""
echo "✓ All benchmarks completed successfully"
echo ""
echo "Results saved to:"
echo "  - Report: $REPORT_FILE"
echo "  - Raw data: $OUTPUT_DIR/"
echo "  - HTML reports: target/criterion/"
echo ""
echo "To view HTML reports, open target/criterion/report/index.html in a browser"
echo ""

if command -v xdg-open > /dev/null 2>&1; then
    read -p "Open HTML report in browser? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        xdg-open target/criterion/report/index.html
    fi
elif command -v open > /dev/null 2>&1; then
    read -p "Open HTML report in browser? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        open target/criterion/report/index.html
    fi
fi
