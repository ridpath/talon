#!/usr/bin/env python3
import time
import sys
import tempfile
import os
from pwn import *

context.log_level = 'error'

def bench_cyclic_generation_mass(iterations):
    results = {}
    
    for count in [1000, 10000, 100000, 1000000]:
        start = time.perf_counter()
        for i in range(count):
            pattern = cyclic(200 + (i % 100))
        end = time.perf_counter()
        results[f'cyclic_gen_mass_{count}'] = end - start
    
    return results

def bench_large_cyclic_patterns():
    results = {}
    
    for size in [1000, 10000, 100000, 1000000]:
        start = time.perf_counter()
        pattern = cyclic(size)
        end = time.perf_counter()
        results[f'cyclic_gen_large_{size}'] = end - start
    
    return results

def bench_cyclic_find():
    results = {}
    
    for pattern_size in [1000, 10000, 100000]:
        pattern = cyclic(pattern_size)
        target = pattern[pattern_size // 2:pattern_size // 2 + 4]
        
        start = time.perf_counter()
        offset = cyclic_find(target)
        end = time.perf_counter()
        results[f'cyclic_find_{pattern_size}'] = end - start
    
    return results

def create_test_elf(size_bytes):
    tf = tempfile.NamedTemporaryFile(delete=False, suffix='.elf')
    
    elf_header = bytes([0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00])
    elf_header += bytes(8)
    elf_header += bytes([0x02, 0x00, 0x3e, 0x00])
    elf_header += bytes([0x01, 0x00, 0x00, 0x00])
    elf_header += bytes([0x00, 0x10, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00])
    elf_header += bytes([0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
    elf_header += bytes([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
    elf_header += bytes([0x00, 0x00, 0x00, 0x00])
    elf_header += bytes([0x40, 0x00, 0x38, 0x00, 0x01, 0x00, 0x40, 0x00])
    
    x64_gadgets = [
        bytes([0x5f, 0xc3]),
        bytes([0x5e, 0xc3]),
        bytes([0x5d, 0xc3]),
        bytes([0x5b, 0xc3]),
        bytes([0x58, 0xc3]),
        bytes([0x59, 0xc3]),
        bytes([0x5a, 0xc3]),
        bytes([0x48, 0x89, 0xe5, 0xc3]),
        bytes([0x48, 0x83, 0xc4, 0x08, 0xc3]),
        bytes([0x48, 0x8b, 0x05, 0x00, 0x00, 0x00, 0x00, 0xc3]),
        bytes([0xc3]),
    ]
    
    data = bytearray(elf_header)
    idx = 0
    
    while len(data) < size_bytes:
        gadget = x64_gadgets[idx % len(x64_gadgets)]
        data.extend(gadget)
        
        padding_len = 16 - (len(gadget) % 16)
        data.extend(bytes([0x90] * padding_len))
        idx += 1
    
    tf.write(bytes(data[:size_bytes]))
    tf.flush()
    tf.close()
    
    return tf.name

def bench_elf_parsing():
    results = {}
    
    for size_mb in [1, 5, 10, 20]:
        size = size_mb * 1024 * 1024
        elf_path = create_test_elf(size)
        
        try:
            start = time.perf_counter()
            e = ELF(elf_path, checksec=False)
            end = time.perf_counter()
            results[f'elf_parsing_{size_mb}MB'] = end - start
        except Exception as ex:
            results[f'elf_parsing_{size_mb}MB'] = -1
        finally:
            os.unlink(elf_path)
    
    return results

def bench_gadget_search():
    results = {}
    
    for size_mb in [1, 5, 10, 20]:
        size = size_mb * 1024 * 1024
        elf_path = create_test_elf(size)
        
        try:
            start = time.perf_counter()
            e = ELF(elf_path, checksec=False)
            rop = ROP(e)
            gadgets = rop.gadgets
            end = time.perf_counter()
            results[f'gadget_search_{size_mb}MB'] = end - start
        except Exception as ex:
            results[f'gadget_search_{size_mb}MB'] = -1
        finally:
            os.unlink(elf_path)
    
    return results

def bench_packing_unpacking():
    results = {}
    
    start = time.perf_counter()
    for i in range(1000000):
        packed = p64(i)
    end = time.perf_counter()
    results['pack_1M_u64'] = end - start
    
    test_data = [p64(i) for i in range(1000000)]
    start = time.perf_counter()
    for data in test_data:
        unpacked = u64(data)
    end = time.perf_counter()
    results['unpack_1M_u64'] = end - start
    
    return results

def main():
    print("[*] Starting pwntools benchmarks...")
    all_results = {}
    
    print("[*] Benchmarking mass cyclic generation...")
    all_results.update(bench_cyclic_generation_mass(1))
    
    print("[*] Benchmarking large cyclic patterns...")
    all_results.update(bench_large_cyclic_patterns())
    
    print("[*] Benchmarking cyclic find...")
    all_results.update(bench_cyclic_find())
    
    print("[*] Benchmarking packing/unpacking...")
    all_results.update(bench_packing_unpacking())
    
    print("[*] Benchmarking ELF parsing...")
    all_results.update(bench_elf_parsing())
    
    print("[*] Benchmarking gadget search...")
    all_results.update(bench_gadget_search())
    
    print("\n[+] Pwntools Benchmark Results:")
    print("=" * 60)
    for name, duration in sorted(all_results.items()):
        if duration >= 0:
            print(f"{name:40} {duration:10.6f}s")
        else:
            print(f"{name:40} {'ERROR':>10}")
    print("=" * 60)
    
    with open('pwntools_results.txt', 'w') as f:
        for name, duration in sorted(all_results.items()):
            f.write(f"{name}:{duration}\n")
    
    print("[+] Results written to pwntools_results.txt")

if __name__ == '__main__':
    main()
