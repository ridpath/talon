use std::collections::HashMap;

#[test]
fn test_type_hint_variants() {
    use talon::ast::TypeHint;
    
    let int_hint = TypeHint::Int;
    let string_hint = TypeHint::String;
    let list_hint = TypeHint::List;
    let map_hint = TypeHint::Map;
    let set_hint = TypeHint::Set;
    let bytes_hint = TypeHint::Bytes;
    let unknown_hint = TypeHint::Unknown;
    let null_hint = TypeHint::Null;
    
    assert!(matches!(int_hint, TypeHint::Int));
    assert!(matches!(string_hint, TypeHint::String));
    assert!(matches!(list_hint, TypeHint::List));
    assert!(matches!(map_hint, TypeHint::Map));
    assert!(matches!(set_hint, TypeHint::Set));
    assert!(matches!(bytes_hint, TypeHint::Bytes));
    assert!(matches!(unknown_hint, TypeHint::Unknown));
    assert!(matches!(null_hint, TypeHint::Null));
}

#[test]
fn test_type_hint_equality() {
    use talon::ast::TypeHint;
    
    assert_eq!(TypeHint::Int, TypeHint::Int);
    assert_ne!(TypeHint::Int, TypeHint::String);
    assert_eq!(TypeHint::Unknown, TypeHint::Unknown);
}

#[test]
fn test_type_hint_clone() {
    use talon::ast::TypeHint;
    
    let original = TypeHint::Int;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_typed_var_creation() {
    use talon::ast::{TypedVar, TypeHint, Expr, Literal};
    
    let var = TypedVar {
        name: "test_var".to_string(),
        var_type: TypeHint::Int,
        value: Expr::Literal(Literal::Number(42)),
    };
    
    assert_eq!(var.name, "test_var");
    assert_eq!(var.var_type, TypeHint::Int);
}

#[test]
fn test_function_def_creation() {
    use talon::ast::{FunctionDef, TypeHint, Command, Expr, Literal};
    
    let func = FunctionDef {
        name: "add".to_string(),
        args: vec![
            ("x".to_string(), None),
            ("y".to_string(), Some(Expr::Literal(Literal::Number(0)))),
        ],
        return_type: Some(TypeHint::Int),
        body: vec![
            Command::Expr(Expr::Return(Box::new(Expr::Literal(Literal::Number(42)))))
        ],
        is_async: false,
    };
    
    assert_eq!(func.name, "add");
    assert_eq!(func.args.len(), 2);
    assert!(!func.is_async);
    assert!(func.return_type.is_some());
}

#[test]
fn test_async_function_def() {
    use talon::ast::{FunctionDef, TypeHint};
    
    let func = FunctionDef {
        name: "fetch".to_string(),
        args: vec![],
        return_type: None,
        body: vec![],
        is_async: true,
    };
    
    assert!(func.is_async);
}

#[test]
fn test_match_arm_creation() {
    use talon::ast::{MatchArm, Expr, Literal, Command};
    
    let arm = MatchArm {
        pattern: Expr::Literal(Literal::Number(1)),
        guard: None,
        body: vec![],
    };
    
    assert!(arm.guard.is_none());
    assert_eq!(arm.body.len(), 0);
}

#[test]
fn test_match_arm_with_guard() {
    use talon::ast::{MatchArm, Expr, Literal, Command};
    
    let arm = MatchArm {
        pattern: Expr::Ident("n".to_string()),
        guard: Some(Expr::ComparisonOp {
            op: ">".to_string(),
            left: Box::new(Expr::Ident("n".to_string())),
            right: Box::new(Expr::Literal(Literal::Number(0))),
        }),
        body: vec![],
    };
    
    assert!(arm.guard.is_some());
}

#[test]
fn test_match_block_creation() {
    use talon::ast::{MatchBlock, MatchArm, Expr, Literal};
    
    let block = MatchBlock {
        expr: Expr::Ident("x".to_string()),
        arms: vec![
            MatchArm {
                pattern: Expr::Literal(Literal::Number(1)),
                guard: None,
                body: vec![],
            },
        ],
    };
    
    assert_eq!(block.arms.len(), 1);
}

#[test]
fn test_try_catch_creation() {
    use talon::ast::TryCatch;
    
    let try_catch = TryCatch {
        try_body: vec![],
        catch_var: "e".to_string(),
        catch_body: vec![],
    };
    
    assert_eq!(try_catch.catch_var, "e");
}

#[test]
fn test_macro_def_creation() {
    use talon::ast::MacroDef;
    
    let macro_def = MacroDef {
        name: "debug".to_string(),
        args: vec!["msg".to_string()],
        body: vec![],
    };
    
    assert_eq!(macro_def.name, "debug");
    assert_eq!(macro_def.args.len(), 1);
}

#[test]
fn test_control_if_creation() {
    use talon::ast::{Control, Expr, Literal};
    
    let ctrl = Control::If {
        condition: Expr::Literal(Literal::Boolean(true)),
        then_body: vec![],
        else_body: vec![],
    };
    
    assert!(matches!(ctrl, Control::If { .. }));
}

#[test]
fn test_control_for_creation() {
    use talon::ast::{Control, Expr, Literal};
    
    let ctrl = Control::For {
        var: "i".to_string(),
        iterable: Expr::List(vec![
            Expr::Literal(Literal::Number(1)),
            Expr::Literal(Literal::Number(2)),
            Expr::Literal(Literal::Number(3)),
        ]),
        body: vec![],
    };
    
    assert!(matches!(ctrl, Control::For { .. }));
}

#[test]
fn test_control_while_creation() {
    use talon::ast::{Control, Expr, Literal};
    
    let ctrl = Control::While {
        condition: Expr::Literal(Literal::Boolean(true)),
        body: vec![],
    };
    
    assert!(matches!(ctrl, Control::While { .. }));
}

#[test]
fn test_control_break_continue() {
    use talon::ast::Control;
    
    let break_ctrl = Control::Break;
    let continue_ctrl = Control::Continue;
    
    assert!(matches!(break_ctrl, Control::Break));
    assert!(matches!(continue_ctrl, Control::Continue));
}

#[test]
fn test_control_parallel() {
    use talon::ast::Control;
    
    let parallel = Control::Parallel { body: vec![] };
    
    assert!(matches!(parallel, Control::Parallel { .. }));
}

#[test]
fn test_command_include() {
    use talon::ast::Command;
    
    let cmd = Command::Include {
        path: "lib/utils.talon".to_string(),
    };
    
    assert!(matches!(cmd, Command::Include { .. }));
}

#[test]
fn test_command_import() {
    use talon::ast::Command;
    
    let cmd = Command::Import {
        module: "crypto".to_string(),
        items: Some(vec!["aes".to_string(), "sha256".to_string()]),
    };
    
    assert!(matches!(cmd, Command::Import { .. }));
}

#[test]
fn test_command_var_decl() {
    use talon::ast::{Command, Expr, Literal};
    
    let cmd = Command::VarDecl {
        name: "x".to_string(),
        value: Expr::Literal(Literal::Number(42)),
    };
    
    assert!(matches!(cmd, Command::VarDecl { .. }));
}

#[test]
fn test_command_const_decl() {
    use talon::ast::{Command, Expr, Literal};
    
    let cmd = Command::ConstDecl {
        name: "PI".to_string(),
        value: Expr::Literal(Literal::String("3.14".to_string())),
    };
    
    assert!(matches!(cmd, Command::ConstDecl { .. }));
}

#[test]
fn test_command_assignment() {
    use talon::ast::{Command, Expr, Literal};
    
    let cmd = Command::Assignment {
        name: "x".to_string(),
        value: Expr::Literal(Literal::Number(100)),
    };
    
    assert!(matches!(cmd, Command::Assignment { .. }));
}

#[test]
fn test_command_struct_def() {
    use talon::ast::Command;
    
    let cmd = Command::StructDef {
        name: "Point".to_string(),
        fields: vec![
            ("x".to_string(), "int".to_string()),
            ("y".to_string(), "int".to_string()),
        ],
    };
    
    assert!(matches!(cmd, Command::StructDef { .. }));
}

#[test]
fn test_command_destructuring_decl() {
    use talon::ast::{Command, Expr, Literal};
    
    let cmd = Command::DestructuringDecl {
        vars: vec!["x".to_string(), "y".to_string()],
        value: Expr::List(vec![
            Expr::Literal(Literal::Number(1)),
            Expr::Literal(Literal::Number(2)),
        ]),
    };
    
    assert!(matches!(cmd, Command::DestructuringDecl { .. }));
}

#[test]
fn test_expr_literal_number() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::Literal(Literal::Number(42));
    
    assert!(matches!(expr, Expr::Literal(Literal::Number(42))));
}

#[test]
fn test_expr_literal_string() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::Literal(Literal::String("hello".to_string()));
    
    assert!(matches!(expr, Expr::Literal(Literal::String(_))));
}

#[test]
fn test_expr_literal_boolean() {
    use talon::ast::{Expr, Literal};
    
    let true_expr = Expr::Literal(Literal::Boolean(true));
    let false_expr = Expr::Literal(Literal::Boolean(false));
    
    assert!(matches!(true_expr, Expr::Literal(Literal::Boolean(true))));
    assert!(matches!(false_expr, Expr::Literal(Literal::Boolean(false))));
}

#[test]
fn test_expr_literal_null() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::Literal(Literal::Null);
    
    assert!(matches!(expr, Expr::Literal(Literal::Null)));
}

#[test]
fn test_expr_ident() {
    use talon::ast::Expr;
    
    let expr = Expr::Ident("variable_name".to_string());
    
    assert!(matches!(expr, Expr::Ident(_)));
}

#[test]
fn test_expr_binary_op() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::BinaryOp {
        op: "+".to_string(),
        left: Box::new(Expr::Literal(Literal::Number(1))),
        right: Box::new(Expr::Literal(Literal::Number(2))),
    };
    
    assert!(matches!(expr, Expr::BinaryOp { .. }));
}

#[test]
fn test_expr_comparison_op() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::ComparisonOp {
        op: ">".to_string(),
        left: Box::new(Expr::Literal(Literal::Number(5))),
        right: Box::new(Expr::Literal(Literal::Number(3))),
    };
    
    assert!(matches!(expr, Expr::ComparisonOp { .. }));
}

#[test]
fn test_expr_bitwise_op() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::BitwiseOp {
        op: "&".to_string(),
        left: Box::new(Expr::Literal(Literal::Number(0xFF))),
        right: Box::new(Expr::Literal(Literal::Number(0x0F))),
    };
    
    assert!(matches!(expr, Expr::BitwiseOp { .. }));
}

#[test]
fn test_expr_list() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::List(vec![
        Expr::Literal(Literal::Number(1)),
        Expr::Literal(Literal::Number(2)),
        Expr::Literal(Literal::Number(3)),
    ]);
    
    assert!(matches!(expr, Expr::List(_)));
}

#[test]
fn test_expr_empty_list() {
    use talon::ast::Expr;
    
    let expr = Expr::List(vec![]);
    
    if let Expr::List(items) = expr {
        assert_eq!(items.len(), 0);
    } else {
        panic!("Expected List variant");
    }
}

#[test]
fn test_expr_map() {
    use talon::ast::{Expr, Literal};
    
    let mut map = HashMap::new();
    map.insert("key".to_string(), Expr::Literal(Literal::String("value".to_string())));
    
    let expr = Expr::Map(map);
    
    assert!(matches!(expr, Expr::Map(_)));
}

#[test]
fn test_expr_set() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::Set(vec![
        Expr::Literal(Literal::Number(1)),
        Expr::Literal(Literal::Number(2)),
        Expr::Literal(Literal::Number(3)),
    ]);
    
    assert!(matches!(expr, Expr::Set(_)));
}

#[test]
fn test_expr_bytes() {
    use talon::ast::Expr;
    
    let expr = Expr::Bytes(vec![0xde, 0xad, 0xbe, 0xef]);
    
    assert!(matches!(expr, Expr::Bytes(_)));
}

#[test]
fn test_expr_lambda() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::Lambda {
        arg: "x".to_string(),
        body: Box::new(Expr::BinaryOp {
            op: "*".to_string(),
            left: Box::new(Expr::Ident("x".to_string())),
            right: Box::new(Expr::Literal(Literal::Number(2))),
        }),
    };
    
    assert!(matches!(expr, Expr::Lambda { .. }));
}

#[test]
fn test_expr_call() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::Call {
        name: "print".to_string(),
        args: vec![(None, Expr::Literal(Literal::String("hello".to_string())))],
    };
    
    assert!(matches!(expr, Expr::Call { .. }));
}

#[test]
fn test_expr_call_with_named_args() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::Call {
        name: "connect".to_string(),
        args: vec![
            (Some("host".to_string()), Expr::Literal(Literal::String("127.0.0.1".to_string()))),
            (Some("port".to_string()), Expr::Literal(Literal::Number(8080))),
        ],
    };
    
    assert!(matches!(expr, Expr::Call { .. }));
}

#[test]
fn test_expr_index() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::Index {
        base: Box::new(Expr::Ident("arr".to_string())),
        index: Box::new(Expr::Literal(Literal::Number(0))),
    };
    
    assert!(matches!(expr, Expr::Index { .. }));
}

#[test]
fn test_expr_slice() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::Slice {
        base: Box::new(Expr::Ident("arr".to_string())),
        start: Box::new(Expr::Literal(Literal::Number(0))),
        end: Box::new(Expr::Literal(Literal::Number(10))),
    };
    
    assert!(matches!(expr, Expr::Slice { .. }));
}

#[test]
fn test_expr_pack() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::Pack {
        size: 64,
        value: Box::new(Expr::Literal(Literal::Number(0xdeadbeef))),
    };
    
    assert!(matches!(expr, Expr::Pack { .. }));
}

#[test]
fn test_expr_unpack() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::Unpack {
        size: 64,
        data: Box::new(Expr::Bytes(vec![0xef, 0xbe, 0xad, 0xde])),
    };
    
    assert!(matches!(expr, Expr::Unpack { .. }));
}

#[test]
fn test_expr_return() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::Return(Box::new(Expr::Literal(Literal::Number(42))));
    
    assert!(matches!(expr, Expr::Return(_)));
}

#[test]
fn test_expr_await() {
    use talon::ast::Expr;
    
    let expr = Expr::Await(Box::new(Expr::Call {
        name: "fetch".to_string(),
        args: vec![],
    }));
    
    assert!(matches!(expr, Expr::Await(_)));
}

#[test]
fn test_expr_spread() {
    use talon::ast::Expr;
    
    let expr = Expr::Spread(Box::new(Expr::Ident("items".to_string())));
    
    assert!(matches!(expr, Expr::Spread(_)));
}

#[test]
fn test_expr_pipe() {
    use talon::ast::Expr;
    
    let expr = Expr::Pipe {
        stages: vec![
            Expr::Ident("data".to_string()),
            Expr::Ident("strip".to_string()),
            Expr::Ident("lower".to_string()),
        ],
    };
    
    assert!(matches!(expr, Expr::Pipe { .. }));
}

#[test]
fn test_expr_list_comprehension() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::ListComprehension {
        expr: Box::new(Expr::BinaryOp {
            op: "*".to_string(),
            left: Box::new(Expr::Ident("x".to_string())),
            right: Box::new(Expr::Ident("x".to_string())),
        }),
        var: "x".to_string(),
        iterable: Box::new(Expr::Call {
            name: "range".to_string(),
            args: vec![
                (None, Expr::Literal(Literal::Number(1))),
                (None, Expr::Literal(Literal::Number(10))),
            ],
        }),
    };
    
    assert!(matches!(expr, Expr::ListComprehension { .. }));
}

#[test]
fn test_expr_env() {
    use talon::ast::Expr;
    
    let expr = Expr::Env("HOME".to_string());
    
    assert!(matches!(expr, Expr::Env(_)));
}

#[test]
fn test_literal_byte_array() {
    use talon::ast::Literal;
    
    let lit = Literal::ByteArray("deadbeef".to_string());
    
    assert!(matches!(lit, Literal::ByteArray(_)));
}

#[test]
fn test_command_clone() {
    use talon::ast::{Command, Expr, Literal};
    
    let original = Command::VarDecl {
        name: "x".to_string(),
        value: Expr::Literal(Literal::Number(42)),
    };
    
    let cloned = original.clone();
    
    assert!(matches!(cloned, Command::VarDecl { .. }));
}

#[test]
fn test_expr_clone() {
    use talon::ast::{Expr, Literal};
    
    let original = Expr::Literal(Literal::Number(42));
    let cloned = original.clone();
    
    assert!(matches!(cloned, Expr::Literal(Literal::Number(42))));
}

#[test]
fn test_complex_nested_expr() {
    use talon::ast::{Expr, Literal};
    
    let expr = Expr::BinaryOp {
        op: "+".to_string(),
        left: Box::new(Expr::BinaryOp {
            op: "*".to_string(),
            left: Box::new(Expr::Literal(Literal::Number(2))),
            right: Box::new(Expr::Literal(Literal::Number(3))),
        }),
        right: Box::new(Expr::BinaryOp {
            op: "/".to_string(),
            left: Box::new(Expr::Literal(Literal::Number(8))),
            right: Box::new(Expr::Literal(Literal::Number(4))),
        }),
    };
    
    assert!(matches!(expr, Expr::BinaryOp { .. }));
}

#[test]
fn test_shellcode_spec() {
    use talon::ast::ShellcodeSpec;
    
    let spec = ShellcodeSpec {
        os: "linux".to_string(),
        payload_type: "reverse_shell".to_string(),
        lhost: Some("192.168.1.1".to_string()),
        lport: Some(4444),
    };
    
    assert_eq!(spec.os, "linux");
    assert_eq!(spec.lhost, Some("192.168.1.1".to_string()));
}

#[test]
fn test_all_command_variants_are_clonable() {
    use talon::ast::{Command, Expr, Literal};
    
    let commands = vec![
        Command::Include { path: "test.talon".to_string() },
        Command::Import { module: "crypto".to_string(), items: None },
        Command::VarDecl { name: "x".to_string(), value: Expr::Literal(Literal::Number(1)) },
        Command::ConstDecl { name: "C".to_string(), value: Expr::Literal(Literal::Number(1)) },
        Command::Assignment { name: "x".to_string(), value: Expr::Literal(Literal::Number(2)) },
        Command::Sleep(1000),
        Command::ExecuteShellcode,
        Command::AntiDebugCheck,
    ];
    
    for cmd in commands {
        let _ = cmd.clone();
    }
}
