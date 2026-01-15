use proptest::prelude::*;

fn parse_and_check(code: &str) -> Result<Vec<talon::ast::Command>, String> {
    talon::parser::parse_script(code)
}

#[test]
fn test_parse_empty_program() {
    let result = parse_and_check("");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_parse_simple_var_decl() {
    let code = "let x = 42";
    let result = parse_and_check(code);
    assert!(result.is_ok());
    let commands = result.unwrap();
    assert_eq!(commands.len(), 1);
}

#[test]
fn test_parse_typed_var_decl() {
    let code = "let x: int = 42";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_const_decl() {
    let code = "const PI = 3.14";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_multiple_var_decls() {
    let code = r#"
        let x = 10
        let y = 20
        let z = 30
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 3);
}

#[test]
fn test_parse_string_literal() {
    let code = r#"let name = "test""#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_hex_number() {
    let code = "let addr = 0xdeadbeef";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_boolean_literals() {
    let code = r#"
        let t = true
        let f = false
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
}

#[test]
fn test_parse_null_literal() {
    let code = "let n = null";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_list_literal() {
    let code = "let lst = [1, 2, 3, 4, 5]";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_empty_list() {
    let code = "let lst = []";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_map_literal() {
    let code = r#"let m = {"key": "value", "num": 42}"#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_set_literal() {
    let code = "let s = #{1, 2, 3}";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_assignment() {
    let code = r#"
        let x = 10
        x = 20
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_compound_assignment() {
    let code = r#"
        let x = 10
        x += 5
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_simple_function() {
    let code = r#"
        define function add(a, b)
            return a + b
        end
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_function_with_defaults() {
    let code = r#"
        define function greet(name = "World")
            return "Hello " + name
        end
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_function_with_type_hints() {
    let code = r#"
        define function add(a: int, b: int): int
            return a + b
        end
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_async_function() {
    let code = r#"
        async define function fetch_data()
            return "data"
        end
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_function_call() {
    let code = r#"print("Hello, world!")"#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_function_call_named_args() {
    let code = r#"connect(host="127.0.0.1", port=8080)"#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_if_statement() {
    let code = r#"
        if x > 10
            print("big")
        end
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_if_else_statement() {
    let code = r#"
        if x > 10
            print("big")
        else
            print("small")
        end
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_for_loop() {
    let code = r#"
        for i in range(0, 10)
            print(i)
        end
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_while_loop() {
    let code = r#"
        while x < 100
            x = x + 1
        end
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_break_continue() {
    let code = r#"
        while true
            break
        end
        for i in range(0, 10)
            continue
        end
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_match_statement() {
    let code = r#"
        match x
            case 1:
                print("one")
            case 2:
                print("two")
        end
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_match_with_guard() {
    let code = r#"
        match x
            case n if n > 0:
                print("positive")
            case 0:
                print("zero")
        end
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_try_catch() {
    let code = r#"
        try
            risky_operation()
        catch e
            print("Error: " + e)
        end
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_macro_def() {
    let code = r#"
        macro debug_print(msg)
            print("[DEBUG] " + msg)
        end
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_macro_call() {
    let code = r#"@debug_print("test")"#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_struct_def() {
    let code = r#"
        struct Point {
            x: int,
            y: int
        }
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_destructuring() {
    let code = "let (x, y, z) = get_coords()";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_include() {
    let code = r#"include "lib/utils.talon""#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_import() {
    let code = r#"import "lib/crypto""#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_binary_operations() {
    let code = r#"
        let a = 1 + 2
        let b = 3 - 4
        let c = 5 * 6
        let d = 7 / 8
        let e = 9 % 10
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 5);
}

#[test]
fn test_parse_comparison_operations() {
    let code = r#"
        let a = x == y
        let b = x != y
        let c = x < y
        let d = x > y
        let e = x <= y
        let f = x >= y
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_logical_operations() {
    let code = r#"
        let a = x and y
        let b = x or y
        let c = not x
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_pack_operations() {
    let code = r#"
        let a = p64(0xdeadbeef)
        let b = p32(0x12345678)
        let c = p16(0xabcd)
        let d = p8(0x42)
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_unpack_operations() {
    let code = r#"
        let a = u64(data)
        let b = u32(data)
        let c = u16(data)
        let d = u8(data)
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_lambda() {
    let code = "let add = fn(x, y) => x + y";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_list_comprehension() {
    let code = "let squares = [x * x for x in range(1, 10)]";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_list_comprehension_with_guard() {
    let code = "let evens = [x for x in range(1, 10) if x % 2 == 0]";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_await_expr() {
    let code = "let result = await fetch_data()";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_env_expr() {
    let code = r#"let home = env("HOME")"#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_parallel_block() {
    let code = r#"
        parallel
            task1()
            task2()
            task3()
        end
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_method_chain() {
    let code = "let result = data.strip().lower().split()";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_index_access() {
    let code = r#"
        let x = arr[0]
        let y = dict["key"]
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_slice() {
    let code = "let sub = arr[1..5]";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_spread_operator() {
    let code = "let combined = [1, 2, ...rest, 5, 6]";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_pipe_operator() {
    let code = "let result = data | strip | lower | split";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_comment_single_line() {
    let code = r#"
        # This is a comment
        let x = 42
        // Another comment
        let y = 10
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_comment_multiline() {
    let code = r#"
        /* This is a
           multiline comment */
        let x = 42
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_nested_expressions() {
    let code = "let result = ((x + y) * (z - w)) / (a + b)";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_complex_program() {
    let code = r#"
        define function exploit(binary, host, port)
            let elf = analyze(binary)
            let rop = build_rop_chain(elf)
            let conn = connect(host, port)
            conn.send(rop)
            let leak = conn.recv()
            return leak
        end

        let target = "./vuln"
        let result = exploit(target, "127.0.0.1", 1337)
        print(hex(result))
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_error_unclosed_string() {
    let code = r#"let x = "unclosed"#;
    let result = parse_and_check(code);
    assert!(result.is_err());
}

#[test]
fn test_parse_error_missing_end() {
    let code = r#"
        if x > 10
            print("big")
    "#;
    let result = parse_and_check(code);
    assert!(result.is_err());
}

#[test]
fn test_parse_error_invalid_syntax() {
    let code = "let = 42";
    let result = parse_and_check(code);
    assert!(result.is_err());
}

#[test]
fn test_parse_byte_array() {
    let code = "let bytes = 0xdeadbeef";
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_multiline_string() {
    let code = r#"
        let text = """
        This is a
        multiline string
        """
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_escape_sequences() {
    let code = r#"let s = "Hello\nWorld\t!""#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_unicode_escape() {
    let code = r#"let s = "Unicode: \u{1F600}""#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_all_type_hints() {
    let code = r#"
        let a: int = 1
        let b: string = "test"
        let c: list = []
        let d: map = {}
        let e: set = #{}
        let f: bytes = 0xdeadbeef
    "#;
    let result = parse_and_check(code);
    assert!(result.is_ok());
}

proptest! {
    #[test]
    fn prop_parse_valid_identifiers(s in "[a-z][a-z0-9_]{0,20}") {
        let code = format!("let {} = 42", s);
        let result = parse_and_check(&code);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn prop_parse_numbers(n in -1000000i64..1000000i64) {
        let code = format!("let x = {}", n);
        let result = parse_and_check(&code);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn prop_parse_hex_numbers(n in 0u64..0xFFFFFFFF) {
        let code = format!("let x = 0x{:x}", n);
        let result = parse_and_check(&code);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn prop_parse_string_literals(s in "[ -~]{0,100}") {
        let escaped = s.replace("\\", "\\\\").replace("\"", "\\\"");
        let code = format!("let x = \"{}\"", escaped);
        let result = parse_and_check(&code);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn prop_parse_binary_ops(a in 0i64..100, b in 1i64..100, op in "[+\\-*/%]") {
        let code = format!("let x = {} {} {}", a, op, b);
        let result = parse_and_check(&code);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn prop_parse_comparison_ops(a in 0i64..100, b in 0i64..100, op in "(==|!=|<|>|<=|>=)") {
        let code = format!("let x = {} {} {}", a, op, b);
        let result = parse_and_check(&code);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn prop_parse_lists(items in prop::collection::vec(0i64..1000, 0..10)) {
        let items_str = items.iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let code = format!("let x = [{}]", items_str);
        let result = parse_and_check(&code);
        prop_assert!(result.is_ok());
    }
}
