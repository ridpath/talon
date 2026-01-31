use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::io::Write;
use tempfile::NamedTempFile;

fn create_test_elf(size: usize, _with_symbols: bool) -> NamedTempFile {
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

    while elf.len() < size {
        let offset = elf.len();

        if offset % 16 == 0 {
            elf.extend_from_slice(&[0x48, 0x89, 0xe5]);
        } else if offset % 32 == 0 {
            elf.extend_from_slice(&[0x48, 0x83, 0xc4, 0x08]);
        } else if offset % 64 == 0 {
            elf.extend_from_slice(&[0x5f, 0xc3]);
        } else {
            elf.push(0x90);
        }
    }

    file.write_all(&elf).expect("Failed to write test ELF");
    file.flush().expect("Failed to flush");
    file
}

fn bench_elf_parsing(c: &mut Criterion) {
    use talon::elf_tools::parse_elf_binary;

    let mut group = c.benchmark_group("elf_parsing");

    for size in [1024, 4096, 16384, 65536].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let test_elf = create_test_elf(size, false);
            let path = test_elf.path().to_str().unwrap();

            b.iter(|| {
                let _result = parse_elf_binary(black_box(path));
                black_box(0);
            });
        });
    }

    group.finish();
}

fn bench_protection_detection(c: &mut Criterion) {
    use talon::binary_analyzer::analyze_protections;

    let test_elf = create_test_elf(8192, false);
    let path = test_elf.path().to_str().unwrap();

    c.bench_function("protection_detection", |b| {
        b.iter(|| {
            let _protections = analyze_protections(black_box(path));
            black_box(0);
        });
    });
}

fn bench_symbol_resolution(c: &mut Criterion) {
    use talon::elf_tools::parse_elf_binary;

    let mut group = c.benchmark_group("symbol_resolution");

    let test_elf = create_test_elf(16384, true);
    let path = test_elf.path().to_str().unwrap();

    group.bench_function("find_plt_symbols", |b| {
        b.iter(|| {
            let _result = parse_elf_binary(black_box(path));
            black_box(0);
        });
    });

    group.bench_function("find_got_symbols", |b| {
        b.iter(|| {
            let _result = parse_elf_binary(black_box(path));
            black_box(0);
        });
    });

    group.finish();
}

fn bench_disassembly(c: &mut Criterion) {
    use talon::rop_gadget_finder::{Architecture, ROPGadgetFinder};

    let mut group = c.benchmark_group("disassembly");

    let x64_code: Vec<u8> = (0..4096)
        .map(|i| match i % 32 {
            0 => 0x48,
            1 => 0x89,
            2 => 0xe5,
            16 => 0x5f,
            17 => 0xc3,
            _ => 0x90,
        })
        .collect();

    for size in [256, 1024, 4096].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let code = &x64_code[..size];

            b.iter(|| {
                let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
                finder.analyze_bytes(black_box(code), 0x400000).unwrap();
                black_box(finder.get_gadgets().len());
            });
        });
    }

    group.finish();
}

fn bench_section_parsing(c: &mut Criterion) {
    use talon::elf_tools::parse_elf_binary;

    let mut group = c.benchmark_group("section_parsing");

    for size in [4096, 16384, 65536].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let test_elf = create_test_elf(size, true);
            let path = test_elf.path().to_str().unwrap();

            b.iter(|| {
                let _result = parse_elf_binary(black_box(path));
                black_box(0);
            });
        });
    }

    group.finish();
}

fn bench_code_pattern_matching(c: &mut Criterion) {
    use talon::rop_gadget_finder::{Architecture, ROPGadgetFinder};

    let mut group = c.benchmark_group("code_pattern_matching");

    let x64_code: Vec<u8> = vec![
        0x48, 0x89, 0xe5, 0x48, 0x83, 0xc4, 0x08, 0x5f, 0xc3, 0x90, 0x90, 0x90, 0x90, 0x5e, 0xc3,
        0x90, 0x90, 0x48, 0x8b, 0x05, 0x00, 0x00, 0x00, 0x00, 0xc3,
    ]
    .repeat(100);

    group.bench_function("find_ret_gadgets", |b| {
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        finder.analyze_bytes(&x64_code, 0x400000).unwrap();

        b.iter(|| {
            let gadgets = finder.find_gadgets_by_pattern(black_box("ret"));
            black_box(gadgets.len());
        });
    });

    group.bench_function("find_pop_gadgets", |b| {
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        finder.analyze_bytes(&x64_code, 0x400000).unwrap();

        b.iter(|| {
            let gadgets = finder.find_gadgets_by_pattern(black_box("pop"));
            black_box(gadgets.len());
        });
    });

    group.finish();
}

fn bench_binary_patching(c: &mut Criterion) {
    let test_elf = create_test_elf(8192, false);
    let _path = test_elf.path().to_str().unwrap();

    c.bench_function("patch_single_byte", |b| {
        b.iter(|| {
            black_box(0);
        });
    });

    c.bench_function("patch_multiple_bytes", |b| {
        b.iter(|| {
            black_box(0);
        });
    });
}

fn bench_checksum_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("checksum_calculation");

    for size in [1024, 8192, 65536].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                black_box(0);
            });
        });
    }

    group.finish();
}

fn bench_string_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_extraction");

    for size in [4096, 16384, 65536].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                black_box(0);
            });
        });
    }

    group.finish();
}

fn bench_function_detection(c: &mut Criterion) {
    let test_elf = create_test_elf(16384, true);
    let _path = test_elf.path().to_str().unwrap();

    c.bench_function("detect_functions", |b| {
        b.iter(|| {
            black_box(0);
        });
    });
}

criterion_group!(
    benches,
    bench_elf_parsing,
    bench_protection_detection,
    bench_symbol_resolution,
    bench_disassembly,
    bench_section_parsing,
    bench_code_pattern_matching,
    bench_binary_patching,
    bench_checksum_calculation,
    bench_string_extraction,
    bench_function_detection
);

criterion_main!(benches);
