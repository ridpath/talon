#[cfg(test)]
mod tests {
    use talon::ast::{Command, Expr, Literal, TypedVar};
    use talon::parser::parse_script;

    #[test]
    fn test_dot_notation_parsing() {
        let code = r#"let val = obj.field"#;
        let result = parse_script(code);
        assert!(result.is_ok(), "Dot notation should parse correctly");

        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);

        if let Command::TypedDecl(TypedVar { name, value, .. }) = &commands[0] {
            assert_eq!(name, "val");
            match value {
                Expr::Index { base, index } => {
                    assert!(matches!(**base, Expr::Ident(ref s) if s == "obj"));
                    assert!(
                        matches!(**index, Expr::Literal(Literal::String(ref s)) if s == "field")
                    );
                }
                _ => panic!(
                    "Expected Index expression for dot notation, got: {:?}",
                    value
                ),
            }
        } else {
            panic!("Expected TypedDecl command");
        }
    }

    #[test]
    fn test_nested_dot_notation() {
        let code = r#"let val = obj.a.b.c"#;
        let result = parse_script(code);
        assert!(result.is_ok(), "Nested dot notation should parse correctly");

        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);

        if let Command::TypedDecl(TypedVar { name, value, .. }) = &commands[0] {
            assert_eq!(name, "val");

            let mut depth = 0;
            let mut current = value;
            while let Expr::Index { base, index } = current {
                depth += 1;
                assert!(matches!(**index, Expr::Literal(Literal::String(_))));
                current = base;
            }

            assert_eq!(depth, 3, "Should have 3 levels of nesting");
            assert!(matches!(current, Expr::Ident(ref s) if s == "obj"));
        } else {
            panic!("Expected TypedDecl command");
        }
    }

    #[test]
    fn test_dot_notation_with_underscore() {
        let code = r#"let val = obj.my_field"#;
        let result = parse_script(code);
        assert!(result.is_ok());

        let commands = result.unwrap();
        if let Command::TypedDecl(TypedVar { value, .. }) = &commands[0] {
            if let Expr::Index { index, .. } = value {
                assert!(
                    matches!(**index, Expr::Literal(Literal::String(ref s)) if s == "my_field")
                );
            }
        }
    }

    #[test]
    fn test_mixed_dot_and_bracket_notation() {
        let code = r#"let val = obj.field["key"]"#;
        let result = parse_script(code);
        assert!(result.is_ok(), "Mixed notation should parse correctly");

        let commands = result.unwrap();
        if let Command::TypedDecl(TypedVar { value, .. }) = &commands[0] {
            if let Expr::Index { base, index } = value {
                assert!(matches!(**index, Expr::Literal(Literal::String(ref s)) if s == "key"));
                assert!(matches!(**base, Expr::Index { .. }));
            }
        }
    }
}
