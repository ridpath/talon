use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use talon::parser::TalonParser;

fn bench_expression_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("expression_parsing");
    
    let expressions = vec![
        ("simple_literal", "42"),
        ("string_literal", r#""hello world""#),
        ("simple_arithmetic", "1 + 2 * 3"),
        ("complex_arithmetic", "(10 + 20) * (30 - 15) / 5"),
        ("function_call", "print(\"hello\")"),
        ("nested_function", "p64(u64(leak) + 0x1234)"),
        ("array_indexing", "array[0][1][2]"),
        ("method_chain", "rop.find(\"pop rdi\").build().execute()"),
    ];
    
    for (name, expr) in expressions.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), expr, |b, &expr| {
            b.iter(|| {
                let parser = TalonParser::new();
                let result = parser.parse_expression(black_box(expr));
                black_box(result.is_ok());
            });
        });
    }
    
    group.finish();
}

fn bench_statement_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("statement_parsing");
    
    let statements = vec![
        ("variable_declaration", "let x = 42"),
        ("if_statement", "if x > 10 { print(x) }"),
        ("while_loop", "while x < 100 { x = x + 1 }"),
        ("for_loop", "for i in range(0, 10) { print(i) }"),
        ("function_definition", "fn add(a, b) { return a + b }"),
        ("return_statement", "return 42"),
    ];
    
    for (name, stmt) in statements.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), stmt, |b, &stmt| {
            b.iter(|| {
                let parser = TalonParser::new();
                let result = parser.parse_statement(black_box(stmt));
                black_box(result.is_ok());
            });
        });
    }
    
    group.finish();
}

fn bench_full_script_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_script_parsing");
    
    let small_script = r#"
        let x = 42
        let y = x + 10
        print(y)
    "#;
    
    let medium_script = r#"
        fn exploit(target, port) {
            let conn = connect(target, port)
            let leak = conn.recv(8)
            let base = u64(leak) - 0x1234
            let rop = p64(base + 0x5678) + p64(base + 0x9abc)
            conn.send(rop)
            return conn.recv()
        }
        
        let result = exploit("localhost", 1337)
        print(hex(result))
    "#;
    
    let large_script = r#"
        fn calculate_libc_base(leak, offset) {
            return leak - offset
        }
        
        fn build_rop_chain(base, gadgets) {
            let chain = bytes("")
            for gadget in gadgets {
                chain = chain + p64(base + gadget)
            }
            return chain
        }
        
        fn exploit_buffer_overflow(target, port) {
            let conn = connect(target, port)
            
            let pattern = cyclic(200)
            conn.send(pattern)
            
            let crash = conn.recv()
            let offset = cyclic_find(crash)
            
            let leak_payload = bytes("A" * offset) + p64(0x400000)
            conn.send(leak_payload)
            
            let leak = u64(conn.recv(8))
            let libc_base = calculate_libc_base(leak, 0x21910)
            
            let gadgets = [0x1234, 0x5678, 0x9abc]
            let rop = build_rop_chain(libc_base, gadgets)
            
            let final_payload = bytes("A" * offset) + rop
            conn.send(final_payload)
            
            conn.interactive()
        }
        
        exploit_buffer_overflow("challenge.ctf", 1337)
    "#;
    
    group.bench_function("small_script", |b| {
        b.iter(|| {
            let parser = TalonParser::new();
            let result = parser.parse(black_box(small_script));
            black_box(result.is_ok());
        });
    });
    
    group.bench_function("medium_script", |b| {
        b.iter(|| {
            let parser = TalonParser::new();
            let result = parser.parse(black_box(medium_script));
            black_box(result.is_ok());
        });
    });
    
    group.bench_function("large_script", |b| {
        b.iter(|| {
            let parser = TalonParser::new();
            let result = parser.parse(black_box(large_script));
            black_box(result.is_ok());
        });
    });
    
    group.finish();
}

fn bench_error_recovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_recovery");
    
    let invalid_scripts = vec![
        ("missing_paren", "let x = (1 + 2"),
        ("invalid_token", "let x = @#$%"),
        ("incomplete_statement", "let x = "),
    ];
    
    for (name, script) in invalid_scripts.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), script, |b, &script| {
            b.iter(|| {
                let parser = TalonParser::new();
                let result = parser.parse(black_box(script));
                black_box(result.is_err());
            });
        });
    }
    
    group.finish();
}

fn bench_complex_expressions(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_expressions");
    
    for nesting_level in [5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(nesting_level),
            nesting_level,
            |b, &level| {
                let expr = (0..level)
                    .map(|_| "(")
                    .collect::<String>()
                    + "42"
                    + &(0..level).map(|_| ")").collect::<String>();
                
                b.iter(|| {
                    let parser = TalonParser::new();
                    let result = parser.parse_expression(black_box(&expr));
                    black_box(result.is_ok());
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_expression_parsing,
    bench_statement_parsing,
    bench_full_script_parsing,
    bench_error_recovery,
    bench_complex_expressions
);

criterion_main!(benches);
