#[cfg(test)]
mod tests {
    use talon::interpreter::interpret;
    use talon::parser::parse_script;

    async fn run_script(code: &str) -> Result<(), String> {
        let commands = parse_script(code)?;
        interpret(&commands).await
    }

    #[tokio::test]
    async fn test_two_level_dot_notation() {
        let code = r#"
            let obj = {"level1": {"level2": 42}}
            let val = obj.level1.level2
        "#;
        let result = run_script(code).await;
        assert!(
            result.is_ok(),
            "Two-level dot notation should work: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_three_level_dot_notation() {
        let code = r#"
            let obj = {"a": {"b": {"c": "success"}}}
            let val = obj.a.b.c
        "#;
        let result = run_script(code).await;
        assert!(
            result.is_ok(),
            "Three-level dot notation should work: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_four_level_dot_notation() {
        let code = r#"
            let obj = {"w": {"x": {"y": {"z": 999}}}}
            let val = obj.w.x.y.z
        "#;
        let result = run_script(code).await;
        assert!(
            result.is_ok(),
            "Four-level dot notation should work: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_five_level_dot_notation() {
        let code = r#"
            let deep = {"a": {"b": {"c": {"d": {"e": "deep_value"}}}}}
            let val = deep.a.b.c.d.e
        "#;
        let result = run_script(code).await;
        assert!(
            result.is_ok(),
            "Five-level dot notation should work: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_elf_symbols_pattern() {
        let code = r#"
            let elf = {"symbols": {"main": 0x401000, "puts": 0x401050}}
            let main_addr = elf.symbols.main
        "#;
        let result = run_script(code).await;
        assert!(
            result.is_ok(),
            "elf.symbols.main pattern should work: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_mixed_bracket_and_dot() {
        let code = r#"
            let obj = {"data": {"values": [10, 20, 30]}}
            let val = obj.data.values[1]
        "#;
        let result = run_script(code).await;
        assert!(
            result.is_ok(),
            "Mixed bracket and dot should work: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_dot_notation_with_numbers() {
        let code = r#"
            let config = {"server": {"port": 8080, "host": "localhost"}}
            let port = config.server.port
        "#;
        let result = run_script(code).await;
        assert!(
            result.is_ok(),
            "Dot notation with numbers should work: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_nested_map_construction_and_access() {
        let code = r#"
            let nested = {
                "level1": {
                    "level2": {
                        "level3": {
                            "value": "found it"
                        }
                    }
                }
            }
            let result = nested.level1.level2.level3.value
        "#;
        let result = run_script(code).await;
        assert!(
            result.is_ok(),
            "Nested map construction and access should work: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_error_missing_key_nested() {
        let code = r#"
            let obj = {"a": {"b": 42}}
            let val = obj.a.c
        "#;
        let result = run_script(code).await;
        assert!(result.is_err(), "Missing nested key should error");
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("KEY NOT FOUND") || err_msg.contains("not in map"),
            "Error should mention missing key: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_error_non_map_type() {
        let code = r#"
            let obj = {"a": 42}
            let val = obj.a.b
        "#;
        let result = run_script(code).await;
        assert!(result.is_err(), "Accessing field on non-map should error");
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("TYPE ERROR") || err_msg.contains("map"),
            "Error should mention type error: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_error_missing_intermediate_key() {
        let code = r#"
            let obj = {"a": {"b": 42}}
            let val = obj.x.y.z
        "#;
        let result = run_script(code).await;
        assert!(result.is_err(), "Missing intermediate key should error");
    }

    #[tokio::test]
    async fn test_assignment_through_nested_dot() {
        let code = r#"
            let obj = {"a": {"b": 0}}
            obj.a.b = 100
        "#;
        let result = run_script(code).await;
        assert!(
            result.is_ok(),
            "Assignment through nested dot notation should work: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_function_call_with_nested_access() {
        let code = r#"
            fn get_value() {
                return {"inner": {"val": 123}}
            }
            let x = get_value().inner.val
        "#;
        let result = run_script(code).await;
        assert!(
            result.is_ok(),
            "Function call with nested access should work: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_complex_nested_structure() {
        let code = r#"
            let system = {
                "network": {
                    "interfaces": {
                        "eth0": {
                            "ipv4": "192.168.1.100",
                            "ipv6": "fe80::1"
                        }
                    }
                }
            }
            let ip = system.network.interfaces.eth0.ipv4
        "#;
        let result = run_script(code).await;
        assert!(
            result.is_ok(),
            "Complex nested structure should work: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_underscore_keys_nested() {
        let code = r#"
            let obj = {"my_key": {"nested_key": "value"}}
            let val = obj.my_key.nested_key
        "#;
        let result = run_script(code).await;
        assert!(
            result.is_ok(),
            "Underscore keys in nested access should work: {:?}",
            result.err()
        );
    }
}
