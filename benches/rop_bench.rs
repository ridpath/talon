use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::io::Write;
use tempfile::NamedTempFile;

fn create_bench_elf_x64(size: usize) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");

    let mut elf = Vec::new();
    elf.extend_from_slice(&[0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00]);
    elf.extend_from_slice(&[0x00; 8]);
    elf.extend_from_slice(&[0x02, 0x00, 0x3e, 0x00]);
    elf.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x40, 0x00, 0x38, 0x00, 0x01, 0x00, 0x00, 0x00]);

    while elf.len() < size {
        let offset = elf.len();

        if offset % 256 == 0 {
            elf.extend_from_slice(&[0x5f, 0xc3]);
        } else if offset % 512 == 0 {
            elf.extend_from_slice(&[0x5e, 0xc3]);
        } else if offset % 1024 == 0 {
            elf.extend_from_slice(&[0x0f, 0x05]);
        } else {
            elf.push(0x90);
        }
    }

    file.write_all(&elf).expect("Failed to write test ELF");
    file.flush().expect("Failed to flush");
    file
}

fn bench_gadget_search(c: &mut Criterion) {
    use talon::rop_tools::RopChain;

    let mut group = c.benchmark_group("gadget_search");

    for size in [1024, 4096, 16384, 65536].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let test_elf = create_bench_elf_x64(size);
            let path = test_elf.path().to_str().unwrap();

            b.iter(|| {
                let _rop = RopChain::new(black_box(path)).unwrap();
                black_box(0);
            });
        });
    }

    group.finish();
}

fn bench_pattern_search(c: &mut Criterion) {
    use talon::rop_tools::RopChain;

    let test_elf = create_bench_elf_x64(16384);
    let path = test_elf.path().to_str().unwrap();
    let rop = RopChain::new(path).unwrap();

    let mut group = c.benchmark_group("pattern_search");

    for pattern in ["pop", "ret", "syscall", "mov"].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(pattern),
            pattern,
            |b, &pattern| {
                b.iter(|| {
                    let gadgets = rop.find_gadgets(black_box(pattern));
                    black_box(gadgets.len());
                });
            },
        );
    }

    group.finish();
}

fn bench_chain_building(c: &mut Criterion) {
    use talon::rop_tools::RopChain;

    let test_elf = create_bench_elf_x64(4096);
    let path = test_elf.path().to_str().unwrap();
    let rop = RopChain::new(path).unwrap();

    let mut group = c.benchmark_group("chain_building");

    for chain_length in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(chain_length),
            chain_length,
            |b, &length| {
                let addresses: Vec<u64> = (0..length).map(|i| 0x400000 + i * 8).collect();

                b.iter(|| {
                    let chain = rop.build_chain(black_box(&addresses));
                    black_box(chain.len());
                });
            },
        );
    }

    group.finish();
}

fn bench_auto_solver(c: &mut Criterion) {
    use talon::rop_tools::{AutoROPSolver, ROPGoal, ROPStrategy};

    let test_elf = create_bench_elf_x64(16384);
    let path = test_elf.path().to_str().unwrap();

    c.bench_function("auto_solver_init", |b| {
        b.iter(|| {
            let solver = AutoROPSolver::new(black_box(path)).unwrap();
            black_box(solver.gadget_db.len());
        });
    });

    c.bench_function("auto_solver_solve", |b| {
        let mut solver = AutoROPSolver::new(path).unwrap();
        solver.libc_base = Some(0x7ffff7a00000);

        b.iter(|| {
            let goal = ROPGoal::System("/bin/sh".to_string());
            let strategies = vec![ROPStrategy::Ret2Libc];
            let result = solver.solve(black_box(goal), black_box(strategies));
            black_box(result.is_ok());
        });
    });
}

fn bench_gadget_finder(c: &mut Criterion) {
    use talon::rop_gadget_finder::{Architecture, ROPGadgetFinder};

    let mut group = c.benchmark_group("gadget_finder");

    let x64_code: Vec<u8> = (0..1024)
        .map(|i| {
            if i % 16 == 0 {
                0x5f
            } else if i % 16 == 1 {
                0xc3
            } else {
                0x90
            }
        })
        .collect();

    group.bench_function("analyze_bytes_1kb", |b| {
        b.iter(|| {
            let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
            finder
                .analyze_bytes(black_box(&x64_code), 0x400000)
                .unwrap();
            black_box(finder.gadgets.len());
        });
    });

    group.bench_function("find_by_pattern", |b| {
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        finder.analyze_bytes(&x64_code, 0x400000).unwrap();

        b.iter(|| {
            let gadgets = finder.find_gadgets_by_pattern(black_box("pop"));
            black_box(gadgets.len());
        });
    });

    group.finish();
}

fn bench_gadget_quality_scoring(c: &mut Criterion) {
    use talon::rop_tools::RopChain;

    let test_elf = create_bench_elf_x64(8192);
    let path = test_elf.path().to_str().unwrap();

    c.bench_function("quality_scoring", |b| {
        b.iter(|| {
            let _rop = RopChain::new(black_box(path)).unwrap();
            black_box(0);
        });
    });
}

criterion_group!(
    benches,
    bench_gadget_search,
    bench_pattern_search,
    bench_chain_building,
    bench_auto_solver,
    bench_gadget_finder,
    bench_gadget_quality_scoring
);

criterion_main!(benches);
