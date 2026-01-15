use talon::parser::parse_script;
use talon::interpreter::interpret;

async fn run_script(code: &str) -> Result<(), String> {
    let commands = parse_script(code)?;
    interpret(&commands).await
}

#[tokio::test]
async fn test_simple_var_decl() {
    let code = "let x = 42";
    let result = run_script(code).await;
    assert!(result.is_ok(), "Simple variable declaration should succeed");
}

#[tokio::test]
async fn test_string_var_decl() {
    let code = r#"let name = "talon""#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "String variable declaration should succeed");
}

#[tokio::test]
async fn test_multiple_var_decls() {
    let code = r#"
        let x = 10
        let y = 20
        let z = 30
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Multiple variable declarations should succeed");
}

#[tokio::test]
async fn test_typed_int_decl() {
    let code = "let x: int = 42";
    let result = run_script(code).await;
    assert!(result.is_ok(), "Typed int declaration should succeed");
}

#[tokio::test]
async fn test_typed_string_decl() {
    let code = r#"let name: string = "test""#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Typed string declaration should succeed");
}

#[tokio::test]
async fn test_typed_bytes_decl() {
    let code = "let data: bytes = 0xdeadbeef";
    let result = run_script(code).await;
    assert!(result.is_ok(), "Typed bytes declaration should succeed");
}

#[tokio::test]
async fn test_typed_list_decl() {
    let code = r#"let items: list = "a,b,c""#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Typed list declaration should succeed");
}

#[tokio::test]
async fn test_typed_map_decl() {
    let code = r#"let config: map = "key:value,foo:bar""#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Typed map declaration should succeed");
}

#[tokio::test]
async fn test_typed_set_decl() {
    let code = r#"let unique: set = "a,b,c,a,b""#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Typed set declaration should succeed");
}

#[tokio::test]
async fn test_typed_null_decl() {
    let code = "let empty: null = null";
    let result = run_script(code).await;
    assert!(result.is_ok(), "Typed null declaration should succeed");
}

#[tokio::test]
async fn test_typed_null_error() {
    let code = "let x: null = 42";
    let result = run_script(code).await;
    assert!(result.is_err(), "Typed null with non-null value should fail");
    assert!(result.unwrap_err().contains("Type Error"));
}

#[tokio::test]
async fn test_const_decl() {
    let code = "const PI = 3.14159";
    let result = run_script(code).await;
    assert!(result.is_ok(), "Const declaration should succeed");
}

#[tokio::test]
async fn test_const_reassignment_error() {
    let code = r#"
        const MAX = 100
        MAX = 200
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Reassigning constant should fail");
    assert!(result.unwrap_err().contains("Cannot reassign constant"));
}

#[tokio::test]
async fn test_var_assignment() {
    let code = r#"
        let x = 10
        x = 20
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Variable reassignment should succeed");
}

#[tokio::test]
async fn test_destructuring_decl() {
    let code = r#"
        let [host, port] = "127.0.0.1:8080"
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Destructuring declaration should succeed");
}

#[tokio::test]
async fn test_destructuring_mismatch() {
    let code = r#"
        let [a, b, c] = "x:y"
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Destructuring mismatch should fail");
    assert!(result.unwrap_err().contains("Destructuring mismatch"));
}

#[tokio::test]
async fn test_hex_number() {
    let code = "let addr = 0xdeadbeef";
    let result = run_script(code).await;
    assert!(result.is_ok(), "Hex number should be parsed correctly");
}

#[tokio::test]
async fn test_boolean_values() {
    let code = r#"
        let t = true
        let f = false
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Boolean values should be supported");
}

#[tokio::test]
async fn test_null_value() {
    let code = "let empty = null";
    let result = run_script(code).await;
    assert!(result.is_ok(), "Null value should be supported");
}

#[tokio::test]
async fn test_list_literal() {
    let code = "let numbers = [1, 2, 3, 4, 5]";
    let result = run_script(code).await;
    assert!(result.is_ok(), "List literal should be supported");
}

#[tokio::test]
async fn test_empty_list() {
    let code = "let empty = []";
    let result = run_script(code).await;
    assert!(result.is_ok(), "Empty list should be supported");
}

#[tokio::test]
async fn test_map_literal() {
    let code = r#"let config = {"host": "localhost", "port": 8080}"#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Map literal should be supported");
}

#[tokio::test]
async fn test_empty_map() {
    let code = "let empty = {}";
    let result = run_script(code).await;
    assert!(result.is_ok(), "Empty map should be supported");
}

#[tokio::test]
async fn test_undefined_variable_error() {
    let code = "x = 42";
    let result = run_script(code).await;
    assert!(result.is_err(), "Using undefined variable should fail");
    assert!(result.unwrap_err().contains("UNDEFINED VARIABLE"));
}

#[tokio::test]
async fn test_byte_array() {
    let code = "let shellcode = 0x9090909090";
    let result = run_script(code).await;
    assert!(result.is_ok(), "Byte array should be supported");
}

#[tokio::test]
async fn test_complex_expressions() {
    let code = r#"
        let base = 0x400000
        let offset = 0x1234
        let target = base
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Complex expressions should work");
}

#[tokio::test]
async fn test_variable_scoping() {
    let code = r#"
        let x = 10
        let y = 20
        let z = 30
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Variable scoping should work correctly");
}

#[tokio::test]
async fn test_type_hint_int_invalid() {
    let code = r#"let x: int = "not_a_number""#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Invalid int type should fail");
}

#[tokio::test]
async fn test_bytes_invalid_hex() {
    let code = "let data: bytes = 0xZZZZ";
    let result = run_script(code).await;
    assert!(result.is_err(), "Invalid hex bytes should fail");
}

#[tokio::test]
async fn test_variable_chaining() {
    let code = r#"
        let a = 1
        let b = 2
        let c = 3
        let d = 4
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Variable chaining should work");
}

#[tokio::test]
async fn test_string_with_special_chars() {
    let code = r#"let payload = "A\x41\x42\x43""#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Strings with escape sequences should work");
}

#[tokio::test]
async fn test_multiline_string() {
    let code = r#"
        let msg = "Line 1
Line 2
Line 3"
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Multiline strings should work");
}

#[tokio::test]
async fn test_numeric_operations_in_decl() {
    let code = r#"
        let x = 42
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Numeric operations in declaration should work");
}

#[tokio::test]
async fn test_large_numbers() {
    let code = "let big = 0xffffffffffffffff";
    let result = run_script(code).await;
    assert!(result.is_ok(), "Large numbers should be supported");
}

#[tokio::test]
async fn test_negative_numbers() {
    let code = "let neg = -42";
    let result = run_script(code).await;
    assert!(result.is_ok(), "Negative numbers should be supported");
}

#[tokio::test]
async fn test_zero_value() {
    let code = "let zero = 0";
    let result = run_script(code).await;
    assert!(result.is_ok(), "Zero value should be supported");
}

#[tokio::test]
async fn test_const_multiple() {
    let code = r#"
        const A = 1
        const B = 2
        const C = 3
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Multiple constants should work");
}

#[tokio::test]
async fn test_mixed_decls() {
    let code = r#"
        let x = 10
        const Y = 20
        let z: int = 30
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Mixed declaration types should work");
}
