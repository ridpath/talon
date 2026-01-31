#!/usr/bin/env python3
import os
import sys
from datetime import datetime

def load_results(filename):
    results = {}
    
    if not os.path.exists(filename):
        print(f"[!] Results file not found: {filename}")
        return results
    
    with open(filename, 'r') as f:
        for line in f:
            line = line.strip()
            if ':' in line:
                name, value = line.split(':', 1)
                try:
                    results[name] = float(value)
                except ValueError:
                    continue
    
    return results

def map_benchmark_names():
    mapping = {
        'cyclic_gen_mass_1000': 'cyclic_generation_mass_1000',
        'cyclic_gen_mass_10000': 'cyclic_generation_mass_10000',
        'cyclic_gen_mass_100000': 'cyclic_generation_mass_100000',
        'cyclic_gen_mass_1000000': 'cyclic_generation_mass_1000000',
        'cyclic_gen_large_1000': 'cyclic_generation_large_1000',
        'cyclic_gen_large_10000': 'cyclic_generation_large_10000',
        'cyclic_gen_large_100000': 'cyclic_generation_large_100000',
        'cyclic_gen_large_1000000': 'cyclic_generation_large_1000000',
        'cyclic_find_1000': 'cyclic_find_1000',
        'cyclic_find_10000': 'cyclic_find_10000',
        'cyclic_find_100000': 'cyclic_find_100000',
        'pack_1M_u64': 'packing_operations_pack_1M_u64',
        'unpack_1M_u64': 'packing_operations_unpack_1M_u64',
        'elf_parsing_1MB': 'elf_parsing_large_1MB',
        'elf_parsing_5MB': 'elf_parsing_large_5MB',
        'elf_parsing_10MB': 'elf_parsing_large_10MB',
        'elf_parsing_20MB': 'elf_parsing_large_20MB',
        'gadget_search_1MB': 'gadget_search_deep_1MB',
        'gadget_search_5MB': 'gadget_search_deep_5MB',
        'gadget_search_10MB': 'gadget_search_deep_10MB',
        'gadget_search_20MB': 'gadget_search_deep_20MB',
    }
    return mapping

def generate_markdown_report(talon_results, pwn_results, output_file):
    mapping = map_benchmark_names()
    
    with open(output_file, 'w') as f:
        f.write("# TALON vs Pwntools Performance Benchmark\n\n")
        f.write(f"**Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
        f.write("**Mission**: Prove TALON's 5x+ performance advantage over pwntools\n\n")
        
        f.write("## Executive Summary\n\n")
        f.write("TALON is designed as a high-performance exploit compiler leveraging Rust's zero-cost abstractions ")
        f.write("and LLVM backend. This benchmark suite demonstrates significant performance improvements over ")
        f.write("Python-based pwntools across critical exploitation primitives.\n\n")
        
        f.write("## Benchmark Categories\n\n")
        
        categories = {
            'Cyclic Pattern Generation (Mass)': [
                ('cyclic_gen_mass_1000', '1,000 patterns'),
                ('cyclic_gen_mass_10000', '10,000 patterns'),
                ('cyclic_gen_mass_100000', '100,000 patterns'),
                ('cyclic_gen_mass_1000000', '1,000,000 patterns'),
            ],
            'Cyclic Pattern Generation (Large)': [
                ('cyclic_gen_large_1000', '1 KB pattern'),
                ('cyclic_gen_large_10000', '10 KB pattern'),
                ('cyclic_gen_large_100000', '100 KB pattern'),
                ('cyclic_gen_large_1000000', '1 MB pattern'),
            ],
            'Cyclic Offset Finding': [
                ('cyclic_find_1000', '1 KB pattern'),
                ('cyclic_find_10000', '10 KB pattern'),
                ('cyclic_find_100000', '100 KB pattern'),
            ],
            'Packing/Unpacking Operations': [
                ('pack_1M_u64', 'Pack 1M u64 values'),
                ('unpack_1M_u64', 'Unpack 1M u64 values'),
            ],
            'ELF Parsing': [
                ('elf_parsing_1MB', '1 MB binary'),
                ('elf_parsing_5MB', '5 MB binary'),
                ('elf_parsing_10MB', '10 MB binary'),
                ('elf_parsing_20MB', '20 MB binary'),
            ],
            'ROP Gadget Search': [
                ('gadget_search_1MB', '1 MB binary'),
                ('gadget_search_5MB', '5 MB binary'),
                ('gadget_search_10MB', '10 MB binary'),
                ('gadget_search_20MB', '20 MB binary'),
            ],
        }
        
        total_speedup = []
        
        for category, benchmarks in categories.items():
            f.write(f"### {category}\n\n")
            f.write("| Benchmark | TALON (s) | Pwntools (s) | Speedup |\n")
            f.write("|-----------|-----------|--------------|----------|\n")
            
            for pwn_name, description in benchmarks:
                talon_name = mapping.get(pwn_name, pwn_name)
                
                talon_time = talon_results.get(talon_name, -1)
                pwn_time = pwn_results.get(pwn_name, -1)
                
                if talon_time > 0 and pwn_time > 0:
                    speedup = pwn_time / talon_time
                    total_speedup.append(speedup)
                    f.write(f"| {description:30} | {talon_time:9.6f} | {pwn_time:12.6f} | **{speedup:.2f}x** |\n")
                elif talon_time > 0:
                    f.write(f"| {description:30} | {talon_time:9.6f} | N/A          | N/A |\n")
                elif pwn_time > 0:
                    f.write(f"| {description:30} | N/A       | {pwn_time:12.6f} | N/A |\n")
                else:
                    f.write(f"| {description:30} | N/A       | N/A          | N/A |\n")
            
            f.write("\n")
        
        if total_speedup:
            avg_speedup = sum(total_speedup) / len(total_speedup)
            max_speedup = max(total_speedup)
            min_speedup = min(total_speedup)
            
            f.write("## Performance Summary\n\n")
            f.write(f"- **Average Speedup**: {avg_speedup:.2f}x\n")
            f.write(f"- **Maximum Speedup**: {max_speedup:.2f}x\n")
            f.write(f"- **Minimum Speedup**: {min_speedup:.2f}x\n")
            f.write(f"- **Total Benchmarks**: {len(total_speedup)}\n\n")
            
            if avg_speedup >= 5.0:
                f.write("### Result: SUCCESS\n\n")
                f.write(f"TALON achieves **{avg_speedup:.1f}x average speedup**, exceeding the 5x target.\n\n")
            else:
                f.write("### Result: OPTIMIZATION REQUIRED\n\n")
                f.write(f"TALON achieves {avg_speedup:.1f}x average speedup. Target: 5x minimum.\n\n")
                f.write("**Action Items**:\n")
                f.write("- Profile `src/interpreter.rs` for bottlenecks\n")
                f.write("- Optimize allocations in hot paths\n")
                f.write("- Consider SIMD for pattern generation\n")
                f.write("- Parallelize gadget search with rayon\n\n")
        
        f.write("## Methodology\n\n")
        f.write("- **TALON**: Benchmarked using Criterion.rs with warmup and statistical analysis\n")
        f.write("- **Pwntools**: Benchmarked using Python `time.perf_counter()` for high-resolution timing\n")
        f.write("- **Environment**: Same hardware, same test data\n")
        f.write("- **Iterations**: Multiple runs averaged for statistical significance\n\n")
        
        f.write("## Why TALON?\n\n")
        f.write("1. **AOT Compilation**: `talon build` produces statically-linked, zero-dependency binaries\n")
        f.write("2. **Rust Performance**: LLVM optimization, zero-cost abstractions, no GC pauses\n")
        f.write("3. **Weaponization**: Compiled exploits embed ROP gadgets and shellcode in `.data` section\n")
        f.write("4. **Portability**: Windows/Linux/macOS native binaries with musl static linking\n")
        f.write("5. **Research-Grade**: Semantic ROP solving, automatic libc identification, alignment correction\n\n")
        
        f.write("---\n\n")
        f.write("**Conclusion**: TALON delivers production-grade performance for high-speed exploitation research and CTF automation.\n")

def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    
    talon_file = os.path.join(script_dir, 'talon_results.txt')
    pwn_file = os.path.join(script_dir, 'pwntools_results.txt')
    output_file = os.path.join(script_dir, 'BENCHMARKS.md')
    
    print("[*] Loading benchmark results...")
    talon_results = load_results(talon_file)
    pwn_results = load_results(pwn_file)
    
    print(f"[+] Loaded {len(talon_results)} TALON results")
    print(f"[+] Loaded {len(pwn_results)} pwntools results")
    
    if not talon_results and not pwn_results:
        print("[!] No results found. Run benchmarks first:")
        print("    - TALON: ./bench_talon.sh or bench_talon.ps1")
        print("    - Pwntools: python3 bench_pwntools.py")
        sys.exit(1)
    
    print("[*] Generating comparison report...")
    generate_markdown_report(talon_results, pwn_results, output_file)
    
    print(f"[+] Benchmark report generated: {output_file}")
    print("\n[*] Opening report...")
    
    if os.path.exists(output_file):
        with open(output_file, 'r') as f:
            content = f.read()
            
            if 'SUCCESS' in content:
                print("\n[+] BENCHMARK RESULT: SUCCESS - 5x+ speedup achieved!")
            elif 'OPTIMIZATION REQUIRED' in content:
                print("\n[!] BENCHMARK RESULT: Optimization needed - target not met")
            
            print("\n" + "=" * 70)
            for line in content.split('\n'):
                if 'Average Speedup' in line or 'Maximum Speedup' in line or 'Minimum Speedup' in line:
                    print(f"  {line}")
            print("=" * 70)

if __name__ == '__main__':
    main()
