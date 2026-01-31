# TALON vs Pwntools Performance Benchmarks

Comprehensive benchmark suite proving TALON's 5x+ performance advantage over pwntools.

## Quick Start

### Linux/macOS/WSL

```bash
cd scripts/benchmarks
chmod +x *.sh
./run_all.sh
```

### Windows (PowerShell)

```powershell
cd scripts\benchmarks
.\run_all.ps1
```

## Requirements

### TALON Benchmarks

- Rust toolchain (stable)
- Cargo

### Pwntools Benchmarks (Optional)

- Python 3.7+
- pwntools: `pip install pwntools`

## Individual Benchmark Runs

### TALON Only

**Linux/macOS/WSL:**

```bash
./bench_talon.sh
```

**Windows:**

```powershell
.\bench_talon.ps1
```

### Pwntools Only

```bash
python3 bench_pwntools.py
```

### Generate Comparison Report

```bash
python3 compare.py
```

## Benchmark Categories

1. **Cyclic Pattern Generation (Mass)**
   - 1,000 to 1,000,000 pattern generations
   - Tests throughput of rapid exploit automation

2. **Cyclic Pattern Generation (Large)**
   - 1 KB to 1 MB pattern sizes
   - Tests memory efficiency and allocation performance

3. **Cyclic Offset Finding**
   - Pattern searching across 1 KB to 100 KB datasets
   - Critical for buffer overflow exploitation

4. **Packing/Unpacking Operations**
   - 1 million u64 pack/unpack operations
   - Tests core primitive performance

5. **ELF Parsing**
   - 1 MB to 20 MB binaries
   - Tests binary analysis speed

6. **ROP Gadget Search**
   - Deep gadget analysis on 1 MB to 20 MB binaries
   - Tests disassembly and pattern matching performance

## Output Files

- `talon_results.txt` - Raw TALON benchmark timings
- `pwntools_results.txt` - Raw pwntools benchmark timings
- `BENCHMARKS.md` - Detailed comparison report with speedup analysis

## Success Criteria

- Average speedup ≥ 5x across all categories
- No category below 3x speedup
- Consistent performance across binary sizes

## Troubleshooting

### TALON benchmarks fail to build

```bash
cargo clean
cargo build --release
cargo bench --bench vs_pwntools_bench
```

### Pwntools not found

```bash
pip install pwntools
```

For Windows users, pwntools requires WSL or may have limited functionality.

### Benchmark results show "N/A"

Ensure both benchmark suites ran successfully before running comparison.

## Performance Optimization

If benchmarks show <5x average speedup:

1. Profile hot paths: `cargo flamegraph --bench vs_pwntools_bench`
2. Check allocations: Review `src/cyclic_tools.rs` and `src/rop_tools.rs`
3. Enable SIMD: Consider vectorization for pattern generation
4. Parallelize: Use rayon for gadget search parallelization

## CI Integration

Add to GitHub Actions:

```yaml
- name: Run Performance Benchmarks
  run: |
    cd scripts/benchmarks
    ./bench_talon.sh
    python3 compare.py
    
- name: Check Performance Regression
  run: |
    python3 scripts/benchmarks/check_regression.py
```

## License

Same as TALON project license.
