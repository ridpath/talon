use talon::interpreter::interpret;
use talon::parser::parse_script;

async fn run_script(code: &str) -> Result<(), String> {
    let commands = parse_script(code)?;
    interpret(&commands).await
}

#[tokio::test]
async fn test_if_statement() {
    let code = r#"
        let x = 10
        if x > 5 {
            let result = "big"
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "If statement should work");
}

#[tokio::test]
async fn test_if_else() {
    let code = r#"
        let x = 3
        if x > 5 {
            let result = "big"
        } else {
            let result = "small"
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "If-else should work");
}

#[tokio::test]
async fn test_if_elif_else() {
    let code = r#"
        let x = 5
        if x > 10 {
            let result = "big"
        } elif x > 5 {
            let result = "medium"
        } else {
            let result = "small"
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "If-elif-else should work");
}

#[tokio::test]
async fn test_nested_if() {
    let code = r#"
        let x = 10
        let y = 20
        if x > 5 {
            if y > 15 {
                let result = "both large"
            }
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Nested if should work");
}

#[tokio::test]
async fn test_if_with_and_condition() {
    let code = r#"
        let x = 10
        let y = 20
        if x > 5 && y > 15 {
            let result = "both true"
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "If with AND condition should work");
}

#[tokio::test]
async fn test_if_with_or_condition() {
    let code = r#"
        let x = 10
        let y = 2
        if x > 5 || y > 15 {
            let result = "at least one true"
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "If with OR condition should work");
}

#[tokio::test]
async fn test_if_with_not_condition() {
    let code = r#"
        let x = 3
        if !(x > 5) {
            let result = "not greater"
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "If with NOT condition should work");
}

#[tokio::test]
async fn test_while_loop() {
    let code = r#"
        let i = 0
        while i < 5 {
            i = 1
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "While loop should work");
}

#[tokio::test]
async fn test_while_with_break() {
    let code = r#"
        let i = 0
        while i < 100 {
            if i > 5 {
                break
            }
            i = 10
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "While with break should work");
}

#[tokio::test]
async fn test_while_with_continue() {
    let code = r#"
        let i = 0
        while i < 5 {
            i = 10
            if i < 3 {
                continue
            }
            let x = 1
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "While with continue should work");
}

#[tokio::test]
async fn test_for_loop() {
    let code = r#"
        let items = [1, 2, 3, 4, 5]
        for item in items {
            let x = item
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "For loop should work");
}

#[tokio::test]
async fn test_for_loop_with_break() {
    let code = r#"
        let items = [1, 2, 3, 4, 5]
        for item in items {
            if item > 3 {
                break
            }
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "For loop with break should work");
}

#[tokio::test]
async fn test_for_loop_with_continue() {
    let code = r#"
        let items = [1, 2, 3, 4, 5]
        for item in items {
            if item < 3 {
                continue
            }
            let x = item
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "For loop with continue should work");
}

#[tokio::test]
async fn test_nested_loops() {
    let code = r#"
        let outer = [1, 2, 3]
        for i in outer {
            let inner = [4, 5, 6]
            for j in inner {
                let prod = i
            }
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Nested loops should work");
}

#[tokio::test]
async fn test_match_simple() {
    let code = r#"
        let x = 42
        match x {
            42 => {
                let result = "found"
            }
            _ => {
                let result = "not found"
            }
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Simple match should work");
}

#[tokio::test]
async fn test_match_multiple_cases() {
    let code = r#"
        let status = 200
        match status {
            200 => {
                let msg = "OK"
            }
            404 => {
                let msg = "Not Found"
            }
            500 => {
                let msg = "Server Error"
            }
            _ => {
                let msg = "Unknown"
            }
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Match with multiple cases should work");
}

#[tokio::test]
async fn test_match_with_guard() {
    let code = r#"
        let x = 10
        match x {
            n if n > 5 => {
                let result = "big"
            }
            _ => {
                let result = "small"
            }
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Match with guard should work");
}

#[tokio::test]
async fn test_match_string() {
    let code = r#"
        let cmd = "help"
        match cmd {
            "help" => {
                let action = "show_help"
            }
            "exit" => {
                let action = "quit"
            }
            _ => {
                let action = "unknown"
            }
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Match on strings should work");
}

#[tokio::test]
async fn test_if_in_loop() {
    let code = r#"
        let i = 0
        while i < 10 {
            if i > 5 {
                let x = "big"
            }
            i = 20
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "If statement in loop should work");
}

#[tokio::test]
async fn test_loop_in_if() {
    let code = r#"
        let x = 10
        if x > 5 {
            let i = 0
            while i < 3 {
                i = 5
            }
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Loop in if statement should work");
}

#[tokio::test]
async fn test_match_in_loop() {
    let code = r#"
        let items = [1, 2, 3]
        for item in items {
            match item {
                1 => { let x = "one" }
                2 => { let x = "two" }
                _ => { let x = "other" }
            }
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Match in loop should work");
}

#[tokio::test]
async fn test_break_in_nested_loop() {
    let code = r#"
        let outer = [1, 2, 3]
        for i in outer {
            let inner = [4, 5, 6]
            for j in inner {
                if j > 5 {
                    break
                }
            }
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Break in nested loop should work");
}

#[tokio::test]
async fn test_continue_in_nested_loop() {
    let code = r#"
        let outer = [1, 2, 3]
        for i in outer {
            let inner = [4, 5, 6]
            for j in inner {
                if j < 5 {
                    continue
                }
                let x = j
            }
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Continue in nested loop should work");
}

#[tokio::test]
async fn test_complex_condition() {
    let code = r#"
        let a = 5
        let b = 10
        let c = 15
        if (a < b && b < c) || a == 0 {
            let result = "valid"
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Complex conditions should work");
}

#[tokio::test]
async fn test_while_true_with_break() {
    let code = r#"
        let count = 0
        while true {
            count = 1
            if count > 0 {
                break
            }
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "While true with break should work");
}

#[tokio::test]
async fn test_for_empty_list() {
    let code = r#"
        let empty = []
        for item in empty {
            let x = item
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "For loop over empty list should work");
}

#[tokio::test]
async fn test_if_with_comparison_operators() {
    let code = r#"
        let x = 10
        if x == 10 { let a = 1 }
        if x != 5 { let b = 2 }
        if x > 5 { let c = 3 }
        if x < 15 { let d = 4 }
        if x >= 10 { let e = 5 }
        if x <= 10 { let f = 6 }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "All comparison operators should work");
}

#[tokio::test]
async fn test_parallel_block() {
    let code = r#"
        parallel {
            let a = 1
            let b = 2
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Parallel block should work");
}

#[tokio::test]
async fn test_match_default_case() {
    let code = r#"
        let x = 999
        match x {
            1 => { let a = "one" }
            2 => { let a = "two" }
            _ => { let a = "default" }
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Match default case should work");
}

#[tokio::test]
async fn test_conditional_assignment() {
    let code = r#"
        let x = 10
        let result = "init"
        if x > 5 {
            result = "big"
        } else {
            result = "small"
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Conditional assignment should work");
}

#[tokio::test]
async fn test_loop_counter() {
    let code = r#"
        let count = 0
        let i = 0
        while i < 10 {
            count = 1
            i = 20
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Loop counter should work");
}

#[tokio::test]
async fn test_early_return_in_loop() {
    let code = r#"
        fn search(items) {
            for item in items {
                if item == 42 {
                    return "found"
                }
            }
            return "not found"
        }
        let result = search([1, 2, 42, 3])
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Early return in loop should work");
}

#[tokio::test]
async fn test_control_flow_complex() {
    let code = r#"
        let data = [1, 2, 3, 4, 5]
        let result = 0
        for num in data {
            if num < 3 {
                continue
            }
            if num > 4 {
                break
            }
            match num {
                3 => { result = 10 }
                4 => { result = 20 }
                _ => { result = 0 }
            }
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Complex control flow should work");
}
