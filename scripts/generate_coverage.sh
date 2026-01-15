#!/usr/bin/env bash
# TALON Code Coverage Generator (Linux/macOS)
# Usage: ./scripts/generate_coverage.sh [profile]
# Profiles: quick, comprehensive, ci (default: comprehensive)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${PROJECT_ROOT}"

PROFILE="${1:-comprehensive}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
COVERAGE_DIR="${PROJECT_ROOT}/coverage"
REPORT_DIR="${COVERAGE_DIR}/reports/${TIMESTAMP}"

echo "=================================================="
echo "TALON Code Coverage Generator"
echo "=================================================="
echo "Profile: ${PROFILE}"
echo "Coverage Directory: ${COVERAGE_DIR}"
echo "Report Directory: ${REPORT_DIR}"
echo ""

mkdir -p "${REPORT_DIR}"

check_tarpaulin() {
    if ! command -v cargo-tarpaulin &> /dev/null; then
        echo "⚠️  cargo-tarpaulin not found. Installing..."
        cargo install cargo-tarpaulin
    else
        echo "✅ cargo-tarpaulin found"
    fi
}

run_coverage() {
    echo ""
    echo "Running coverage with profile: ${PROFILE}"
    echo "=================================================="
    
    local start_time=$(date +%s)
    
    case "${PROFILE}" in
        quick)
            cargo tarpaulin \
                --out Stdout \
                --out Html \
                --output-dir "${REPORT_DIR}" \
                --timeout 60 \
                --verbose
            ;;
        
        comprehensive)
            cargo tarpaulin \
                --out Html \
                --out Xml \
                --out Lcov \
                --out Json \
                --output-dir "${REPORT_DIR}" \
                --all-features \
                --workspace \
                --timeout 300 \
                --run-types Tests,Doctests \
                --verbose
            ;;
        
        ci)
            cargo tarpaulin \
                --out Xml \
                --output-dir "${REPORT_DIR}" \
                --all-features \
                --workspace \
                --timeout 300 \
                --run-types Tests,Doctests \
                --fail-under 80 \
                --verbose
            ;;
        
        *)
            echo "❌ Unknown profile: ${PROFILE}"
            echo "Available profiles: quick, comprehensive, ci"
            exit 1
            ;;
    esac
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    echo ""
    echo "Coverage generation completed in ${duration}s"
}

generate_summary() {
    echo ""
    echo "=================================================="
    echo "Coverage Summary"
    echo "=================================================="
    
    if [ -f "${REPORT_DIR}/cobertura.xml" ]; then
        local line_rate=$(grep -oP 'line-rate="\K[^"]+' "${REPORT_DIR}/cobertura.xml" | head -1)
        local coverage_percent=$(echo "${line_rate} * 100" | bc)
        echo "Line Coverage: ${coverage_percent}%"
        
        if (( $(echo "${coverage_percent} >= 80" | bc -l) )); then
            echo "✅ Coverage meets target (≥80%)"
        else
            echo "⚠️  Coverage below target (<80%)"
        fi
    fi
    
    if [ -f "${REPORT_DIR}/index.html" ]; then
        echo ""
        echo "HTML Report: ${REPORT_DIR}/index.html"
    fi
    
    if [ -f "${REPORT_DIR}/tarpaulin-report.html" ]; then
        echo "HTML Report: ${REPORT_DIR}/tarpaulin-report.html"
    fi
    
    if [ -f "${REPORT_DIR}/cobertura.xml" ]; then
        echo "XML Report: ${REPORT_DIR}/cobertura.xml"
    fi
    
    if [ -f "${REPORT_DIR}/lcov.info" ]; then
        echo "LCOV Report: ${REPORT_DIR}/lcov.info"
    fi
    
    if [ -f "${REPORT_DIR}/tarpaulin-report.json" ]; then
        echo "JSON Report: ${REPORT_DIR}/tarpaulin-report.json"
    fi
}

symlink_latest() {
    local latest_link="${COVERAGE_DIR}/reports/latest"
    rm -f "${latest_link}"
    ln -sf "${REPORT_DIR}" "${latest_link}"
    echo ""
    echo "Latest report symlinked to: ${latest_link}"
}

open_report() {
    if [ "${PROFILE}" != "ci" ]; then
        echo ""
        read -p "Open HTML report in browser? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            if [ -f "${REPORT_DIR}/tarpaulin-report.html" ]; then
                if command -v xdg-open &> /dev/null; then
                    xdg-open "${REPORT_DIR}/tarpaulin-report.html"
                elif command -v open &> /dev/null; then
                    open "${REPORT_DIR}/tarpaulin-report.html"
                else
                    echo "⚠️  Cannot open browser automatically"
                fi
            fi
        fi
    fi
}

main() {
    check_tarpaulin
    run_coverage
    generate_summary
    symlink_latest
    open_report
    
    echo ""
    echo "=================================================="
    echo "Coverage generation complete!"
    echo "=================================================="
}

main
