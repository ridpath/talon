# TALON Performance Benchmarking Guide

## Quick Start

### Run All Benchmarks

**Linux/macOS:**
```bash
./scripts/run_benchmarks.sh
```

**Windows:**
```powershell
.\scripts\run_benchmarks.ps1
```

### Run Specific Benchmark Suite

```bash
cargo bench --bench parser_bench
cargo bench --bench interpreter_bench
cargo bench --bench binary_analysis_bench
cargo bench --bench rop_bench
```

### Run Specific Test

```bash
cargo bench --bench parser_bench -- expression_parsing
cargo bench --bench interpreter_bench -- control_flow
```

---

## Available Benchmark Suites

### 1. Parser Benchmarks
**File:** `benches/parser_bench.rs`  
**Tests:** 24 functions

Measures parsing performance:
- Expression parsing (literals, arithmetic, function calls)
- Statement parsing (variables, control flow, functions)
- Full script parsing (small, medium, large)
- Error recovery
- Complex nested expressions

**Run:**
```bash
cargo bench --bench parser_bench
```

### 2. Interpreter Benchmarks
**File:** `benches/interpreter_bench.rs`  
**Tests:** 25 functions

Measures runtime execution performance:
- Variable operations
- Control flow (if/while/for)
- Function calls (simple, recursive, nested)
- Builtin functions (p64, u64, hex, bytes, cyclic)
- Array operations
- Exploitation primitives
- Full exploit scripts

**Run:**
```bash
cargo bench --bench interpreter_bench
```

### 3. Binary Analysis Benchmarks
**File:** `benches/binary_analysis_bench.rs`  
**Tests:** 24 functions

Measures binary analysis tool performance:
- ELF parsing (multiple sizes)
- Protection detection (NX, PIE, Canary, RELRO)
- Symbol resolution (PLT, GOT)
- Disassembly
- Section parsing
- Code pattern matching
- Binary patching
- Checksum calculation
- String extraction
- Function detection

**Run:**
```bash
cargo bench --bench binary_analysis_bench
```

### 4. ROP Tools Benchmarks
**File:** `benches/rop_bench.rs`  
**Tests:** 18 functions

Measures ROP gadget finding and chain building:
- Gadget search (multiple binary sizes)
- Pattern search (pop, ret, syscall, mov)
- Chain building (10-1000 gadgets)
- Auto solver
- Gadget finder
- Quality scoring

**Run:**
```bash
cargo bench --bench rop_bench
```

---

## Benchmark Output

### Console Output

Criterion displays results like:
```
test parser_bench/expression_parsing/simple_literal
                        time:   [125.43 ns 126.78 ns 128.21 ns]
                        change: [-2.45% -0.83% +0.91%] (p = 0.35 > 0.05)
                        No change in performance detected.
```

**Fields:**
- **time:** Mean execution time with 95% confidence interval
- **change:** Percentage change from baseline
- **p-value:** Statistical significance (p < 0.05 = significant change)

### HTML Reports

After running benchmarks, view detailed reports:
```bash
# Linux/macOS
xdg-open target/criterion/report/index.html

# macOS
open target/criterion/report/index.html

# Windows
start target/criterion/report/index.html
```

Reports include:
- Line plots of performance over time
- Probability density functions
- Comparison charts
- Detailed statistics

---

## Baseline Management

### Save Current Performance as Baseline

```bash
cargo bench --bench parser_bench -- --save-baseline main
```

### Compare Against Baseline

```bash
cargo bench --bench parser_bench -- --baseline main
```

### List Available Baselines

```bash
ls target/criterion/*/base/
```

---

## Advanced Usage

### Filter Tests by Name

```bash
cargo bench --bench parser_bench -- expression
```

### Set Number of Iterations

```bash
cargo bench --bench parser_bench -- --sample-size 1000
```

### Disable HTML Report Generation

```bash
cargo bench --bench parser_bench -- --noplot
```

### Profile Benchmarks

```bash
cargo bench --bench parser_bench -- --profile-time 10
```

---

## CI/CD Integration

Benchmarks run automatically on:
- Push to `main` or `develop` branches
- Pull requests
- Weekly (Sunday midnight UTC)
- Manual workflow dispatch

**Workflow:** `.github/workflows/benchmarks.yml`

### View CI Results

1. Go to GitHub Actions tab
2. Select "Benchmarks" workflow
3. Click on latest run
4. Download benchmark artifacts

### PR Comments

For pull requests, benchmark results are automatically posted as comments showing:
- Performance changes vs. base branch
- Statistical significance
- Regression warnings (if >10% slower)

---

## Performance Targets

| Component | Target | Rationale |
|-----------|--------|-----------|
| Small script parsing | < 1ms | Interactive REPL responsiveness |
| Large script parsing | < 10ms | IDE auto-complete latency |
| Loop (100 iterations) | < 5ms | Script execution speed |
| ELF parsing (64KB) | < 10ms | Binary analysis tools |
| Gadget search (16KB) | < 50ms | ROP chain generation |
| Chain building (100) | < 1ms | Exploit development flow |

---

## Troubleshooting

### Benchmarks Won't Compile

**Issue:** Missing Criterion dependency  
**Solution:**
```bash
cargo build --tests
cargo build --benches
```

### Benchmarks Run Slowly

**Issue:** Debug mode enabled  
**Solution:** Benchmarks automatically use release mode. Verify:
```bash
cargo bench --verbose
```

### Inconsistent Results

**Issue:** System load, background processes  
**Solution:**
- Close unnecessary applications
- Run multiple times and compare
- Use `--sample-size` to increase iterations

### Missing Baseline

**Issue:** No baseline to compare against  
**Solution:**
```bash
cargo bench --bench parser_bench -- --save-baseline main
```

---

## Best Practices

### Before Benchmarking

1. **Close background apps:** Minimize system load
2. **Disable power saving:** Use maximum performance mode
3. **Consistent environment:** Same hardware/OS for comparisons
4. **Warmup system:** Run benchmarks twice, use second result

### Interpreting Results

1. **Check confidence intervals:** Narrow = more reliable
2. **Look for trends:** Single outliers may be noise
3. **Statistical significance:** p < 0.05 indicates real change
4. **Relative vs absolute:** 10% faster may only save microseconds

### When to Optimize

Only optimize if:
- **Regression detected:** Performance degraded >10%
- **Target missed:** Fails to meet performance goals
- **User impact:** Affects interactive responsiveness
- **Competitive need:** Slower than similar tools

---

## Benchmark Development

### Adding New Benchmarks

1. **Create benchmark function:**
```rust
fn bench_my_feature(c: &mut Criterion) {
    c.bench_function("my_feature", |b| {
        b.iter(|| {
            // Code to benchmark
            black_box(my_function());
        });
    });
}
```

2. **Add to criterion_group:**
```rust
criterion_group!(
    benches,
    bench_my_feature,
    bench_other_feature
);
```

3. **Run:**
```bash
cargo bench --bench my_bench
```

### Benchmark Testing Tips

- **Use `black_box()`:** Prevents compiler optimization
- **Setup vs measurement:** Use `iter_batched()` for setup code
- **Parameterized tests:** Use `BenchmarkGroup` for multiple inputs
- **Avoid I/O:** Mock external dependencies
- **Measure what matters:** Focus on user-facing operations

---

## Resources

### Documentation

- Criterion.rs: https://bheisler.github.io/criterion.rs/book/
- Rust Performance Book: https://nnethercote.github.io/perf-book/

### Tools

- **Flamegraph:** Visualize performance hotspots
  ```bash
  cargo install flamegraph
  cargo flamegraph --bench parser_bench
  ```

- **Cachegrind:** Cache profiling
  ```bash
  valgrind --tool=cachegrind target/release/talon
  ```

- **Perf:** Linux performance counters
  ```bash
  perf record cargo bench
  perf report
  ```

---

## Summary

**91 benchmark functions** across 4 critical modules provide comprehensive performance monitoring for TALON. Use these benchmarks to:

- Detect performance regressions
- Guide optimization efforts
- Compare with competing tools
- Make data-driven release decisions

**Quick Commands:**
```bash
# Run all benchmarks
./scripts/run_benchmarks.sh

# Run specific suite
cargo bench --bench parser_bench

# Save baseline
cargo bench -- --save-baseline main

# View HTML reports
open target/criterion/report/index.html
```
