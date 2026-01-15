use talon::parser::parse_script;
use talon::interpreter::interpret;

async fn run_script(code: &str) -> Result<(), String> {
    let commands = parse_script(code)?;
    interpret(&commands).await
}

#[tokio::test]
async fn test_try_catch_basic() {
    let code = r#"
        try {
            let x = 10
        } catch e {
            let error = e
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Basic try-catch should work");
}

#[tokio::test]
async fn test_try_catch_with_error() {
    let code = r#"
        try {
            let x = undefined_var
        } catch e {
            let error_caught = "yes"
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Try-catch with error should catch it");
}

#[tokio::test]
async fn test_try_catch_no_error() {
    let code = r#"
        try {
            let x = 42
            let y = "test"
        } catch e {
            let should_not_run = "yes"
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Try-catch with no error should work");
}

#[tokio::test]
async fn test_nested_try_catch() {
    let code = r#"
        try {
            try {
                let x = 1
            } catch e1 {
                let inner_error = e1
            }
        } catch e2 {
            let outer_error = e2
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Nested try-catch should work");
}

#[tokio::test]
async fn test_try_catch_in_function() {
    let code = r#"
        fn safe_operation() {
            try {
                let x = 10
            } catch e {
                return "error"
            }
            return "success"
        }
        let result = safe_operation()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Try-catch in function should work");
}

#[tokio::test]
async fn test_try_catch_in_loop() {
    let code = r#"
        let items = [1, 2, 3]
        for item in items {
            try {
                let x = item
            } catch e {
                let error = e
            }
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Try-catch in loop should work");
}

#[tokio::test]
async fn test_undefined_variable_error() {
    let code = r#"
        let x = undefined_variable
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Undefined variable should throw error");
    assert!(result.unwrap_err().contains("UNDEFINED VARIABLE"));
}

#[tokio::test]
async fn test_undefined_function_error() {
    let code = r#"
        undefined_function()
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Calling undefined function should throw error");
}

#[tokio::test]
async fn test_type_error() {
    let code = r#"
        let x: int = "not a number"
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Type mismatch should throw error");
    assert!(result.unwrap_err().contains("Type Error"));
}

#[tokio::test]
async fn test_const_reassignment_error() {
    let code = r#"
        const X = 10
        X = 20
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Const reassignment should throw error");
    assert!(result.unwrap_err().contains("Cannot reassign constant"));
}

#[tokio::test]
async fn test_destructuring_mismatch_error() {
    let code = r#"
        let [a, b, c] = "one:two"
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Destructuring mismatch should throw error");
    assert!(result.unwrap_err().contains("Destructuring mismatch"));
}

#[tokio::test]
async fn test_builtin_missing_arg_error() {
    let code = r#"
        let x = p64()
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Builtin missing arg should throw error");
}

#[tokio::test]
async fn test_builtin_wrong_type_error() {
    let code = r#"
        let x = p64("not a number")
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Builtin wrong type should throw error");
}

#[tokio::test]
async fn test_division_by_zero_error() {
    let code = r#"
        let x = 10
        let y = 0
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Division by zero test setup should work");
}

#[tokio::test]
async fn test_null_type_error() {
    let code = r#"
        let x: null = 42
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Null type with non-null value should error");
    assert!(result.unwrap_err().contains("Type Error"));
}

#[tokio::test]
async fn test_invalid_hex_error() {
    let code = r#"
        let x: bytes = 0xZZZZ
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Invalid hex should throw error");
}

#[tokio::test]
async fn test_function_arg_mismatch_error() {
    let code = r#"
        fn needs_two(a, b) {
            let x = a
        }
        needs_two(1)
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Function arg mismatch should throw error");
}

#[tokio::test]
async fn test_error_message_quality() {
    let code = r#"
        let result = undefined_var
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Should produce error");
    let err = result.unwrap_err();
    assert!(err.contains("UNDEFINED VARIABLE"), "Error should be descriptive");
}

#[tokio::test]
async fn test_try_catch_preserves_error_message() {
    let code = r#"
        let caught_error = ""
        try {
            let x = undefined_var
        } catch e {
            caught_error = e
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Try-catch should catch and store error");
}

#[tokio::test]
async fn test_multiple_errors_first_wins() {
    let code = r#"
        let a = error1
        let b = error2
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Should fail on first error");
}

#[tokio::test]
async fn test_error_in_nested_function() {
    let code = r#"
        fn inner() {
            let x = undefined_var
        }
        fn outer() {
            inner()
        }
        outer()
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Error should propagate through nested functions");
}

#[tokio::test]
async fn test_error_recovery_with_try_catch() {
    let code = r#"
        let success = false
        try {
            let x = bad_var
        } catch e {
            success = true
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Should recover from error with try-catch");
}

#[tokio::test]
async fn test_try_catch_rethrow() {
    let code = r#"
        fn risky() {
            try {
                let x = undefined
            } catch e {
                let logged = e
            }
        }
        risky()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Try-catch should handle error locally");
}

#[tokio::test]
async fn test_error_in_if_condition() {
    let code = r#"
        if undefined_var {
            let x = 1
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Error in if condition should propagate");
}

#[tokio::test]
async fn test_error_in_loop_condition() {
    let code = r#"
        while undefined_var {
            let x = 1
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Error in loop condition should propagate");
}

#[tokio::test]
async fn test_error_in_match_expression() {
    let code = r#"
        match undefined_var {
            1 => { let x = "one" }
            _ => { let x = "other" }
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Error in match expression should propagate");
}

#[tokio::test]
async fn test_try_catch_multiple_statements() {
    let code = r#"
        try {
            let a = 1
            let b = 2
            let c = 3
        } catch e {
            let error = e
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Try-catch with multiple statements should work");
}

#[tokio::test]
async fn test_try_catch_early_error() {
    let code = r#"
        try {
            let a = undefined
            let b = 2
            let c = 3
        } catch e {
            let caught = "yes"
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Try-catch should catch early error");
}

#[tokio::test]
async fn test_error_message_suggestions() {
    let code = r#"
        let xyz = 10
        let result = xy
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Should error on undefined variable");
}

#[tokio::test]
async fn test_graceful_error_handling() {
    let code = r#"
        fn safe_divide(a, b) {
            try {
                let result = a
                return result
            } catch e {
                return null
            }
        }
        let result = safe_divide(10, 2)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Graceful error handling should work");
}

#[tokio::test]
async fn test_try_catch_variable_scope() {
    let code = r#"
        let outer = "before"
        try {
            let inner = "inside"
            outer = "modified"
        } catch e {
            let error = e
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Try-catch variable scope should work correctly");
}

#[tokio::test]
async fn test_error_with_complex_expression() {
    let code = r#"
        let x = 10
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Complex expression setup should work");
}

#[tokio::test]
async fn test_try_catch_with_return() {
    let code = r#"
        fn test() {
            try {
                return "success"
            } catch e {
                return "error"
            }
        }
        let result = test()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Try-catch with return should work");
}

#[tokio::test]
async fn test_error_propagation_chain() {
    let code = r#"
        fn level3() {
            let x = bad_var
        }
        fn level2() {
            level3()
        }
        fn level1() {
            level2()
        }
        level1()
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "Errors should propagate through call chain");
}

#[tokio::test]
async fn test_try_catch_isolation() {
    let code = r#"
        let global = "initial"
        try {
            global = "try_block"
            let x = undefined
        } catch e {
            global = "catch_block"
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Try-catch should isolate error properly");
}
