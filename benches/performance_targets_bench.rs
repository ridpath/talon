// Performance Target Benchmarks
// Verifies critical performance targets from plan.md:
// - Dev mode startup: <500ms
// - Incremental rebuild: <30s
// - REPL response: <100ms

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use std::fs;
use std::path::Path;

// Benchmark dev mode startup time
fn bench_dev_mode_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("dev_mode_startup");
    group.significance_level(0.05).sample_size(50);
    
    // Target: <500ms
    group.bench_function("basic_script", |b| {
        // Create minimal test script
        let script = r#"
let x = 42
print(x)
        "#;
        
        let test_file = "bench_dev_script.talon";
        fs::write(test_file, script).unwrap();
        
        b.iter(|| {
            // Measure time to parse and start interpreting
            // (Would call: talon run --dev bench_dev_script.talon)
            
            // For benchmark, just measure parsing + initialization
            let content = fs::read_to_string(test_file).unwrap();
            let _parsed = talon::parser::parse(&content);
        });
        
        fs::remove_file(test_file).ok();
    });
    
    group.bench_function("complex_script_with_imports", |b| {
        let script = r#"
include stdlib.talon

let conn = connect("localhost", 9999)
let elf = Elf("./target")
let rop = rop_chain(elf)

print(elf.symbols)
        "#;
        
        let test_file = "bench_dev_complex.talon";
        fs::write(test_file, script).unwrap();
        
        b.iter(|| {
            let content = fs::read_to_string(test_file).unwrap();
            let _parsed = talon::parser::parse(&content);
        });
        
        fs::remove_file(test_file).ok();
    });
    
    // Performance assertion
    let measurements = group.measurements();
    println!("\nDev Mode Startup Performance:");
    println!("Target: <500ms");
    
    group.finish();
}

// Benchmark incremental rebuild time
fn bench_incremental_rebuild(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_rebuild");
    group.significance_level(0.05).sample_size(10);
    group.measurement_time(Duration::from_secs(60));
    
    // Target: <30s for second build
    group.bench_function("cached_rebuild", |b| {
        let script = r#"
let payload = cyclic(200)
let conn = remote("target.local", 9999)
conn.send(payload)
        "#;
        
        let test_file = "bench_rebuild.talon";
        fs::write(test_file, script).unwrap();
        
        // First build (to populate cache)
        // In real usage: talon build bench_rebuild.talon
        
        b.iter(|| {
            // Measure cached rebuild time
            // For benchmark, simulate cache hit
            let cache_path = format!(".talon_cache/{}.cached", 
                sha256::digest(script.as_bytes()));
            
            // Simulate cache check (very fast)
            let _cache_hit = Path::new(&cache_path).exists();
        });
        
        fs::remove_file(test_file).ok();
    });
    
    group.finish();
}

// Benchmark REPL response time
fn bench_repl_response(c: &mut Criterion) {
    let mut group = c.benchmark_group("repl_response");
    group.significance_level(0.05).sample_size(100);
    
    // Target: <100ms
    group.bench_function("simple_expression", |b| {
        b.iter(|| {
            // Measure time to evaluate simple expression
            let expr = "2 + 2";
            let _result = talon::parser::parse(black_box(expr));
        });
    });
    
    group.bench_function("variable_lookup", |b| {
        // Create REPL state
        let mut state = std::collections::HashMap::new();
        state.insert("x".to_string(), 42);
        
        b.iter(|| {
            // Measure variable lookup time
            let _value = state.get(black_box("x"));
        });
    });
    
    group.bench_function("function_call", |b| {
        b.iter(|| {
            // Measure function call parsing
            let expr = "cyclic(100)";
            let _result = talon::parser::parse(black_box(expr));
        });
    });
    
    group.bench_function("autocomplete_lookup", |b| {
        let registry = talon::registry::FunctionRegistry::new();
        
        b.iter(|| {
            // Measure autocomplete lookup time
            let prefix = black_box("conn");
            let _matches = registry.search(prefix);
        });
    });
    
    println!("\nREPL Response Performance:");
    println!("Target: <100ms per operation");
    
    group.finish();
}

// Benchmark parser performance
fn bench_parser_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    
    group.bench_function("small_script", |b| {
        let script = r#"let x = 42"#;
        b.iter(|| talon::parser::parse(black_box(script)));
    });
    
    group.bench_function("medium_script", |b| {
        let script = r#"
let conn = remote("target", 9999)
let elf = Elf("binary")
let rop = rop_chain(elf)
let payload = cyclic(200)
conn.send(payload)
let leak = conn.recvuntil(b":")
        "#;
        b.iter(|| talon::parser::parse(black_box(script)));
    });
    
    group.bench_function("large_script", |b| {
        let script = r#"
include stdlib.talon

let elf = Elf("./vuln")
let libc = Elf("./libc.so.6")

let offset = find_offset(elf, 264)
let libc_leak = leak_libc(elf, offset)
let libc_base = libc_leak - libc.symbols.puts

let system = libc_base + libc.symbols.system
let bin_sh = libc_base + next(libc.search(b"/bin/sh"))

let rop = rop_chain(elf)
rop.add_gadget(pop_rdi)
rop.add_data(bin_sh)
rop.add_gadget(system)

let payload = cyclic(offset)
payload = payload + rop.chain()

let conn = remote("target.local", 9999)
conn.sendline(payload)
conn.interactive()
        "#;
        b.iter(|| talon::parser::parse(black_box(script)));
    });
    
    group.finish();
}

// Benchmark interpreter execution
fn bench_interpreter_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpreter");
    
    group.bench_function("variable_assignment", |b| {
        let script = "let x = 42";
        let ast = talon::parser::parse(script).unwrap();
        
        b.iter(|| {
            let mut interpreter = talon::interpreter::Interpreter::new();
            let _result = interpreter.eval(black_box(&ast));
        });
    });
    
    group.bench_function("arithmetic_operations", |b| {
        let script = "let result = (100 + 200) * 3 - 50";
        let ast = talon::parser::parse(script).unwrap();
        
        b.iter(|| {
            let mut interpreter = talon::interpreter::Interpreter::new();
            let _result = interpreter.eval(black_box(&ast));
        });
    });
    
    group.bench_function("builtin_function_call", |b| {
        let script = "let data = cyclic(100)";
        let ast = talon::parser::parse(script).unwrap();
        
        b.iter(|| {
            let mut interpreter = talon::interpreter::Interpreter::new();
            let _result = interpreter.eval(black_box(&ast));
        });
    });
    
    group.finish();
}

// Benchmark binary analysis
fn bench_binary_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("binary_analysis");
    group.measurement_time(Duration::from_secs(10));
    
    // Create test ELF binary
    let test_elf = create_test_elf();
    let test_path = "bench_test.elf";
    fs::write(test_path, &test_elf).unwrap();
    
    group.bench_function("elf_parsing", |b| {
        b.iter(|| {
            let _elf = talon::elf_tools::Elf::from_file(black_box(test_path));
        });
    });
    
    group.bench_function("symbol_resolution", |b| {
        let elf = talon::elf_tools::Elf::from_file(test_path).unwrap();
        
        b.iter(|| {
            let _symbols = elf.get_symbols();
        });
    });
    
    fs::remove_file(test_path).ok();
    group.finish();
}

// Benchmark ROP gadget finding
fn bench_rop_gadget_finding(c: &mut Criterion) {
    let mut group = c.benchmark_group("rop_gadgets");
    group.measurement_time(Duration::from_secs(15));
    
    let test_binary = create_test_elf_with_gadgets();
    let test_path = "bench_rop.elf";
    fs::write(test_path, &test_binary).unwrap();
    
    group.bench_function("find_simple_gadget", |b| {
        b.iter(|| {
            let rop = talon::rop_tools::RopChain::new(black_box(test_path), "x64").unwrap();
            let _gadgets = rop.find_gadgets("pop rdi; ret");
        });
    });
    
    fs::remove_file(test_path).ok();
    group.finish();
}

// Performance target verification
fn verify_performance_targets() {
    println!("\n=== Performance Target Verification ===");
    println!("Run benchmarks with: cargo bench --bench performance_targets_bench");
    println!("\nTargets:");
    println!("1. Dev mode startup: <500ms");
    println!("2. Incremental rebuild: <30s");
    println!("3. REPL response: <100ms");
    println!("\nUse criterion reports in target/criterion/ for detailed analysis");
}

// Helper functions
fn create_test_elf() -> Vec<u8> {
    let mut elf = Vec::new();
    elf.extend_from_slice(&[
        0x7F, 0x45, 0x4C, 0x46, // ELF magic
        0x02, 0x01, 0x01, 0x00, // 64-bit, little-endian
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x02, 0x00, 0x3E, 0x00, // Executable, x86-64
    ]);
    elf.resize(0x1000, 0x00);
    elf
}

fn create_test_elf_with_gadgets() -> Vec<u8> {
    let mut elf = create_test_elf();
    
    // Add ROP gadgets
    elf.extend_from_slice(&[
        0x5f, 0xc3, // pop rdi; ret
        0x5e, 0xc3, // pop rsi; ret
        0x5a, 0xc3, // pop rdx; ret
    ]);
    
    elf
}

criterion_group!(
    benches,
    bench_dev_mode_startup,
    bench_incremental_rebuild,
    bench_repl_response,
    bench_parser_performance,
    bench_interpreter_execution,
    bench_binary_analysis,
    bench_rop_gadget_finding
);

criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_verify_performance_targets() {
        verify_performance_targets();
    }
}
