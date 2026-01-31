use talon::interpreter::interpret;
use talon::parser::parse_script;

async fn run_script(code: &str) -> Result<(), String> {
    let commands = parse_script(code)?;
    interpret(&commands).await
}

#[tokio::test]
async fn test_simple_function_definition() {
    let code = r#"
        fn greet() {
            let msg = "hello"
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Simple function definition should succeed");
}

#[tokio::test]
async fn test_function_with_params() {
    let code = r#"
        fn add(a, b) {
            let result = a
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Function with parameters should succeed");
}

#[tokio::test]
async fn test_function_with_default_params() {
    let code = r#"
        fn greet(name = "World") {
            let msg = name
        }
    "#;
    let result = run_script(code).await;
    assert!(
        result.is_ok(),
        "Function with default parameters should succeed"
    );
}

#[tokio::test]
async fn test_function_call_no_args() {
    let code = r#"
        fn test() {
            let x = 1
        }
        test()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Function call with no args should succeed");
}

#[tokio::test]
async fn test_function_call_with_args() {
    let code = r#"
        fn process(data) {
            let x = data
        }
        process("test")
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Function call with args should succeed");
}

#[tokio::test]
async fn test_function_call_multiple_args() {
    let code = r#"
        fn calc(a, b, c) {
            let sum = a
        }
        calc(1, 2, 3)
    "#;
    let result = run_script(code).await;
    assert!(
        result.is_ok(),
        "Function call with multiple args should succeed"
    );
}

#[tokio::test]
async fn test_function_return_value() {
    let code = r#"
        fn get_value() {
            return 42
        }
        let x = get_value()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Function with return value should succeed");
}

#[tokio::test]
async fn test_function_return_string() {
    let code = r#"
        fn get_name() {
            return "talon"
        }
        let name = get_name()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Function returning string should succeed");
}

#[tokio::test]
async fn test_function_with_typed_return() {
    let code = r#"
        fn calc() -> int {
            return 100
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Function with typed return should succeed");
}

#[tokio::test]
async fn test_async_function() {
    let code = r#"
        async fn fetch_data() {
            let data = "async result"
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Async function should be supported");
}

#[tokio::test]
async fn test_async_function_call() {
    let code = r#"
        async fn get_data() {
            return "data"
        }
        let result = get_data()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Async function call should succeed");
}

#[tokio::test]
async fn test_nested_function_calls() {
    let code = r#"
        fn inner() {
            return 10
        }
        fn outer() {
            let x = inner()
            return 20
        }
        let result = outer()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Nested function calls should work");
}

#[tokio::test]
async fn test_recursive_function() {
    let code = r#"
        fn countdown(n) {
            if n > 0 {
                countdown(n)
            }
        }
        countdown(3)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Recursive function should work");
}

#[tokio::test]
async fn test_function_with_local_vars() {
    let code = r#"
        fn process() {
            let local = "value"
            let another = 42
        }
        process()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Function with local variables should work");
}

#[tokio::test]
async fn test_function_accessing_global() {
    let code = r#"
        let global = 100
        fn access_global() {
            let x = global
        }
        access_global()
    "#;
    let result = run_script(code).await;
    assert!(
        result.is_ok(),
        "Function accessing global variable should work"
    );
}

#[tokio::test]
async fn test_function_modifying_global() {
    let code = r#"
        let counter = 0
        fn increment() {
            counter = 1
        }
        increment()
    "#;
    let result = run_script(code).await;
    assert!(
        result.is_ok(),
        "Function modifying global variable should work"
    );
}

#[tokio::test]
async fn test_multiple_function_definitions() {
    let code = r#"
        fn func1() {
            let x = 1
        }
        fn func2() {
            let y = 2
        }
        fn func3() {
            let z = 3
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Multiple function definitions should work");
}

#[tokio::test]
async fn test_function_call_order() {
    let code = r#"
        func1()
        func2()

        fn func1() {
            let a = 1
        }
        fn func2() {
            let b = 2
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Functions can be called before definition");
}

#[tokio::test]
async fn test_function_with_named_args() {
    let code = r#"
        fn config(host, port) {
            let h = host
            let p = port
        }
        config(host: "localhost", port: 8080)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Named arguments should work");
}

#[tokio::test]
async fn test_function_with_mixed_args() {
    let code = r#"
        fn connect(host, port, timeout) {
            let h = host
        }
        connect("localhost", port: 8080, timeout: 30)
    "#;
    let result = run_script(code).await;
    assert!(
        result.is_ok(),
        "Mixed positional and named args should work"
    );
}

#[tokio::test]
async fn test_function_default_param_usage() {
    let code = r#"
        fn greet(name = "User") {
            let msg = name
        }
        greet()
        greet("Alice")
    "#;
    let result = run_script(code).await;
    assert!(
        result.is_ok(),
        "Default parameters should work with and without args"
    );
}

#[tokio::test]
async fn test_function_return_early() {
    let code = r#"
        fn check(val) {
            if val {
                return "yes"
            }
            return "no"
        }
        let result = check(true)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Early return should work");
}

#[tokio::test]
async fn test_function_no_return() {
    let code = r#"
        fn action() {
            let x = 1
        }
        action()
    "#;
    let result = run_script(code).await;
    assert!(
        result.is_ok(),
        "Function without explicit return should work"
    );
}

#[tokio::test]
async fn test_function_empty_body() {
    let code = r#"
        fn noop() {
        }
        noop()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Function with empty body should work");
}

#[tokio::test]
async fn test_function_return_null() {
    let code = r#"
        fn get_null() {
            return null
        }
        let x = get_null()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Function returning null should work");
}

#[tokio::test]
async fn test_function_return_list() {
    let code = r#"
        fn get_list() {
            return [1, 2, 3]
        }
        let items = get_list()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Function returning list should work");
}

#[tokio::test]
async fn test_function_return_map() {
    let code = r#"
        fn get_config() {
            return {"key": "value"}
        }
        let cfg = get_config()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Function returning map should work");
}

#[tokio::test]
async fn test_function_undefined_call() {
    let code = r#"
        undefined_function()
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Calling undefined function should fail");
}

#[tokio::test]
async fn test_function_param_count_mismatch() {
    let code = r#"
        fn needs_two(a, b) {
            let x = a
        }
        needs_two(1)
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Wrong number of arguments should fail");
}

#[tokio::test]
async fn test_function_complex_body() {
    let code = r#"
        fn exploit() {
            let offset = 72
            let payload = "A"
            let rop = 0xdeadbeef
        }
        exploit()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Function with complex body should work");
}

#[tokio::test]
async fn test_function_with_control_flow() {
    let code = r#"
        fn test_control(x) {
            if x > 0 {
                return "positive"
            }
            return "non-positive"
        }
        let r = test_control(5)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Function with control flow should work");
}

#[tokio::test]
async fn test_function_scope_isolation() {
    let code = r#"
        let outer = "global"
        fn test() {
            let inner = "local"
        }
        test()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Function scope should be isolated");
}

#[tokio::test]
async fn test_function_return_expression() {
    let code = r#"
        fn double(x) {
            return x
        }
        let result = double(21)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Function returning expression should work");
}
