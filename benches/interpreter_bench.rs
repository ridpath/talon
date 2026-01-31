use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use talon::interpreter::interpret;
use talon::parser::parse_script;

fn bench_variable_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("variable_operations");

    let scripts = vec![
        ("simple_assignment", "let x = 42"),
        ("arithmetic_assignment", "let x = 10 + 20 * 3"),
        ("string_concatenation", r#"let s = "hello" + " " + "world""#),
        ("array_creation", "let arr = [1, 2, 3, 4, 5]"),
    ];

    for (name, script) in scripts.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), script, |b, &script| {
            b.iter(|| {
                let parser = TalonParser::new();
                let ast = parser.parse(script).unwrap();
                let mut interpreter = Interpreter::new();
                black_box(interpreter.eval(&ast));
            });
        });
    }

    group.finish();
}

fn bench_control_flow(c: &mut Criterion) {
    let mut group = c.benchmark_group("control_flow");

    let if_script = r#"
        let x = 100
        if x > 50 {
            x = x * 2
        } else {
            x = x + 10
        }
    "#;

    group.bench_function("if_else", |b| {
        b.iter(|| {
            let commands = parse_script(black_box(if_script)).unwrap();
            black_box(
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(interpret(&commands)),
            );
        });
    });

    let while_script = r#"
        let i = 0
        while i < 100 {
            i = i + 1
        }
    "#;

    group.bench_function("while_loop_100", |b| {
        b.iter(|| {
            let commands = parse_script(black_box(while_script)).unwrap();
            black_box(
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(interpret(&commands)),
            );
        });
    });

    let for_script = r#"
        let sum = 0
        for i in range(0, 100) {
            sum = sum + i
        }
    "#;

    group.bench_function("for_loop_100", |b| {
        b.iter(|| {
            let commands = parse_script(black_box(for_script)).unwrap();
            black_box(
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(interpret(&commands)),
            );
        });
    });

    group.finish();
}

fn bench_function_calls(c: &mut Criterion) {
    let mut group = c.benchmark_group("function_calls");

    let simple_function = r#"
        fn add(a, b) {
            return a + b
        }
        let result = add(10, 20)
    "#;

    group.bench_function("simple_function", |b| {
        b.iter(|| {
            let commands = parse_script(black_box(simple_function)).unwrap();
            black_box(
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(interpret(&commands)),
            );
        });
    });

    let recursive_function = r#"
        fn factorial(n) {
            if n <= 1 {
                return 1
            }
            return n * factorial(n - 1)
        }
        let result = factorial(10)
    "#;

    group.bench_function("recursive_factorial_10", |b| {
        b.iter(|| {
            let commands = parse_script(black_box(recursive_function)).unwrap();
            black_box(
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(interpret(&commands)),
            );
        });
    });

    let nested_calls = r#"
        fn inner(x) { return x + 1 }
        fn middle(x) { return inner(x) + 2 }
        fn outer(x) { return middle(x) + 3 }
        let result = outer(10)
    "#;

    group.bench_function("nested_calls", |b| {
        b.iter(|| {
            let commands = parse_script(black_box(nested_calls)).unwrap();
            black_box(
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(interpret(&commands)),
            );
        });
    });

    group.finish();
}

fn bench_builtin_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("builtin_functions");

    let scripts = vec![
        ("p64_single", "let packed = p64(0xdeadbeef)"),
        (
            "p64_chain",
            "let chain = p64(0x1111) + p64(0x2222) + p64(0x3333)",
        ),
        (
            "u64_unpack",
            r#"let value = u64("\x01\x02\x03\x04\x05\x06\x07\x08")"#,
        ),
        ("hex_conversion", "let h = hex(0xdeadbeef)"),
        ("bytes_creation", r#"let b = bytes("A" * 100)"#),
        ("cyclic_pattern", "let pattern = cyclic(200)"),
    ];

    for (name, script) in scripts.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), script, |b, &script| {
            b.iter(|| {
                let parser = TalonParser::new();
                let ast = parser.parse(script).unwrap();
                let mut interpreter = Interpreter::new();
                black_box(interpreter.eval(&ast));
            });
        });
    }

    group.finish();
}

fn bench_array_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("array_operations");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("array_creation", size),
            size,
            |b, &size| {
                let script = format!(
                    "let arr = [{}]",
                    (0..size)
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                b.iter(|| {
                    let parser = TalonParser::new();
                    let ast = parser.parse(black_box(&script)).unwrap();
                    let mut interpreter = Interpreter::new();
                    black_box(interpreter.eval(&ast));
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("array_indexing", size),
            size,
            |b, &size| {
                let script = format!(
                    "let arr = [{}]\nlet x = arr[{}]",
                    (0..size)
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    size / 2
                );

                b.iter(|| {
                    let parser = TalonParser::new();
                    let ast = parser.parse(black_box(&script)).unwrap();
                    let mut interpreter = Interpreter::new();
                    black_box(interpreter.eval(&ast));
                });
            },
        );
    }

    group.finish();
}

fn bench_exploitation_primitives(c: &mut Criterion) {
    let mut group = c.benchmark_group("exploitation_primitives");

    let rop_chain = r#"
        let base = 0x7ffff7a00000
        let pop_rdi = base + 0x1234
        let bin_sh = base + 0x5678
        let system = base + 0x9abc
        let chain = p64(pop_rdi) + p64(bin_sh) + p64(system)
    "#;

    group.bench_function("rop_chain_construction", |b| {
        b.iter(|| {
            let commands = parse_script(black_box(rop_chain)).unwrap();
            black_box(
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(interpret(&commands)),
            );
        });
    });

    let payload_building = r#"
        let offset = 72
        let padding = bytes("A" * offset)
        let overflow = p64(0x400000) + p64(0x400008) + p64(0x400010)
        let payload = padding + overflow
    "#;

    group.bench_function("payload_building", |b| {
        b.iter(|| {
            let commands = parse_script(black_box(payload_building)).unwrap();
            black_box(
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(interpret(&commands)),
            );
        });
    });

    group.finish();
}

fn bench_full_exploit_scripts(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_exploit_scripts");

    let buffer_overflow_exploit = r#"
        fn exploit(target, port) {
            let offset = 72
            let libc_base = 0x7ffff7a00000
            let system = libc_base + 0x4f440
            let bin_sh = libc_base + 0x1b3e9a
            let pop_rdi = libc_base + 0x2155f

            let padding = bytes("A" * offset)
            let rop = p64(pop_rdi) + p64(bin_sh) + p64(system)
            let payload = padding + rop

            return payload
        }

        let final_payload = exploit("localhost", 1337)
    "#;

    group.bench_function("buffer_overflow_exploit", |b| {
        b.iter(|| {
            let commands = parse_script(black_box(buffer_overflow_exploit)).unwrap();
            black_box(
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(interpret(&commands)),
            );
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_variable_operations,
    bench_control_flow,
    bench_function_calls,
    bench_builtin_functions,
    bench_array_operations,
    bench_exploitation_primitives,
    bench_full_exploit_scripts
);

criterion_main!(benches);
