use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::io::Write;
use talon::cyclic_tools::cyclic;
use talon::elf_tools::ElfContext;
use talon::rop_gadget_finder::{Architecture, ROPGadgetFinder};
use tempfile::NamedTempFile;

fn create_realistic_elf(size: usize) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");

    let mut elf = Vec::new();

    elf.extend_from_slice(&[0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00]);
    elf.extend_from_slice(&[0x00; 8]);
    elf.extend_from_slice(&[0x02, 0x00, 0x3e, 0x00]);
    elf.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x10, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x40, 0x00, 0x38, 0x00, 0x01, 0x00, 0x40, 0x00]);

    let x64_gadgets: Vec<Vec<u8>> = vec![
        vec![0x5f, 0xc3],
        vec![0x5e, 0xc3],
        vec![0x5d, 0xc3],
        vec![0x5b, 0xc3],
        vec![0x58, 0xc3],
        vec![0x59, 0xc3],
        vec![0x5a, 0xc3],
        vec![0x48, 0x89, 0xe5, 0xc3],
        vec![0x48, 0x83, 0xc4, 0x08, 0xc3],
        vec![0x48, 0x8b, 0x05, 0x00, 0x00, 0x00, 0x00, 0xc3],
        vec![0xc3],
    ];

    while elf.len() < size {
        let gadget = &x64_gadgets[elf.len() % x64_gadgets.len()];
        elf.extend_from_slice(gadget);

        for _ in 0..(16 - (gadget.len() % 16)) {
            elf.push(0x90);
        }
    }

    file.write_all(&elf).expect("Failed to write test ELF");
    file.flush().expect("Failed to flush");
    file
}

fn bench_mass_cyclic_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cyclic_generation_mass");
    group.sample_size(10);

    for count in [1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                for i in 0..count {
                    let pattern = cyclic(black_box(200 + (i % 100)));
                    black_box(pattern.len());
                }
            });
        });
    }

    group.finish();
}

fn bench_large_cyclic_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("cyclic_generation_large");

    for size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let pattern = cyclic(black_box(size));
                black_box(pattern.len());
            });
        });
    }

    group.finish();
}

fn bench_deep_gadget_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("gadget_search_deep");
    group.sample_size(10);

    for size_mb in [1, 5, 10, 20].iter() {
        let size = size_mb * 1024 * 1024;
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}MB", size_mb)),
            &size,
            |b, &size| {
                let test_elf = create_realistic_elf(size);
                let path = test_elf.path().to_str().unwrap();

                b.iter(|| {
                    if let Ok(data) = std::fs::read(black_box(path)) {
                        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
                        finder.analyze_bytes(&data, 0x400000).unwrap();
                        let gadgets = finder.get_best_gadgets(10000);
                        black_box(gadgets.len());
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_elf_parsing_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("elf_parsing_large");

    for size_mb in [1, 5, 10, 20].iter() {
        let size = size_mb * 1024 * 1024;
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}MB", size_mb)),
            &size,
            |b, &size| {
                let test_elf = create_realistic_elf(size);
                let path = test_elf.path().to_str().unwrap();

                b.iter(|| {
                    let _result = ElfContext::load(black_box(path));
                    black_box(0);
                });
            },
        );
    }

    group.finish();
}

fn bench_cyclic_find_operations(c: &mut Criterion) {
    use talon::cyclic_tools::cyclic_find_bytes;

    let mut group = c.benchmark_group("cyclic_find");

    for pattern_size in [1_000, 10_000, 100_000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(pattern_size),
            pattern_size,
            |b, &size| {
                let pattern = cyclic(size);
                let target = &pattern[size / 2..size / 2 + 4];

                b.iter(|| {
                    let offset = cyclic_find_bytes(black_box(target));
                    black_box(offset);
                });
            },
        );
    }

    group.finish();
}

fn bench_pattern_matching_gadgets(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_matching_gadgets");

    let test_elf = create_realistic_elf(5 * 1024 * 1024);
    let data = std::fs::read(test_elf.path()).unwrap();
    let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
    finder.analyze_bytes(&data, 0x400000).unwrap();

    let patterns = vec!["ret", "pop", "mov", "jmp", "call"];

    for pattern in patterns.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(pattern),
            pattern,
            |b, &pattern| {
                b.iter(|| {
                    let gadgets = finder.find_gadgets_by_pattern(black_box(pattern));
                    black_box(gadgets.len());
                });
            },
        );
    }

    group.finish();
}

fn bench_packing_unpacking(c: &mut Criterion) {
    use talon::packing_tools::{pack64, unpack64};

    let mut group = c.benchmark_group("packing_operations");

    group.bench_function("pack_1M_u64", |b| {
        b.iter(|| {
            for i in 0..1_000_000 {
                let packed = pack64(black_box(i));
                black_box(packed.len());
            }
        });
    });

    group.bench_function("unpack_1M_u64", |b| {
        let test_data: Vec<Vec<u8>> = (0..1_000_000).map(|i| pack64(i)).collect();

        b.iter(|| {
            for data in &test_data {
                if let Ok(unpacked) = unpack64(black_box(data)) {
                    black_box(unpacked);
                }
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_mass_cyclic_generation,
    bench_large_cyclic_patterns,
    bench_deep_gadget_search,
    bench_elf_parsing_large,
    bench_cyclic_find_operations,
    bench_pattern_matching_gadgets,
    bench_packing_unpacking
);

criterion_main!(benches);
