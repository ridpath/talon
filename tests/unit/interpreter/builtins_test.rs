use talon::parser::parse_script;
use talon::interpreter::interpret;

async fn run_script(code: &str) -> Result<(), String> {
    let commands = parse_script(code)?;
    interpret(&commands).await
}

#[tokio::test]
async fn test_p64_builtin() {
    let code = r#"
        let packed = p64(0xdeadbeef)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "p64() should work");
}

#[tokio::test]
async fn test_p32_builtin() {
    let code = r#"
        let packed = p32(0x41414141)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "p32() should work");
}

#[tokio::test]
async fn test_p16_builtin() {
    let code = r#"
        let packed = p16(0x4142)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "p16() should work");
}

#[tokio::test]
async fn test_u64_builtin() {
    let code = r#"
        let packed = p64(0xdeadbeef)
        let unpacked = u64(packed)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "u64() should work");
}

#[tokio::test]
async fn test_u32_builtin() {
    let code = r#"
        let packed = p32(0x41414141)
        let unpacked = u32(packed)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "u32() should work");
}

#[tokio::test]
async fn test_u16_builtin() {
    let code = r#"
        let packed = p16(0x4142)
        let unpacked = u16(packed)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "u16() should work");
}

#[tokio::test]
async fn test_pack64_alias() {
    let code = r#"
        let packed = pack64(0x1234567890)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "pack64() alias should work");
}

#[tokio::test]
async fn test_unpack64_alias() {
    let code = r#"
        let packed = p64(0x1234567890)
        let unpacked = unpack64(packed)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "unpack64() alias should work");
}

#[tokio::test]
async fn test_cyclic_builtin() {
    let code = r#"
        let pattern = cyclic(100)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "cyclic() should work");
}

#[tokio::test]
async fn test_cyclic_find_builtin() {
    let code = r#"
        let pattern = cyclic(1000)
        let offset = cyclic_find(pattern, "aaab")
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "cyclic_find() should work");
}

#[tokio::test]
async fn test_help_builtin() {
    let code = r#"
        help()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "help() should work");
}

#[tokio::test]
async fn test_help_with_function() {
    let code = r#"
        help("p64")
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "help() with function name should work");
}

#[tokio::test]
async fn test_help_search() {
    let code = r#"
        help(search: "rop")
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "help() with search should work");
}

#[tokio::test]
async fn test_p64_missing_arg() {
    let code = r#"
        let packed = p64()
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "p64() without argument should fail");
}

#[tokio::test]
async fn test_p64_invalid_type() {
    let code = r#"
        let packed = p64("not a number")
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "p64() with invalid type should fail");
}

#[tokio::test]
async fn test_u64_missing_arg() {
    let code = r#"
        let unpacked = u64()
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "u64() without argument should fail");
}

#[tokio::test]
async fn test_cyclic_missing_arg() {
    let code = r#"
        let pattern = cyclic()
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "cyclic() without argument should fail");
}

#[tokio::test]
async fn test_cyclic_invalid_type() {
    let code = r#"
        let pattern = cyclic("not a number")
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "cyclic() with invalid type should fail");
}

#[tokio::test]
async fn test_cyclic_find_missing_args() {
    let code = r#"
        let pattern = cyclic(100)
        let offset = cyclic_find(pattern)
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "cyclic_find() with missing args should fail");
}

#[tokio::test]
async fn test_pack_unpack_roundtrip() {
    let code = r#"
        let original = 0x1234567890abcdef
        let packed = p64(original)
        let unpacked = u64(packed)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Pack/unpack roundtrip should work");
}

#[tokio::test]
async fn test_pack32_unpack32_roundtrip() {
    let code = r#"
        let original = 0x12345678
        let packed = p32(original)
        let unpacked = u32(packed)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Pack32/unpack32 roundtrip should work");
}

#[tokio::test]
async fn test_multiple_pack_operations() {
    let code = r#"
        let a = p64(0x1111111111111111)
        let b = p64(0x2222222222222222)
        let c = p64(0x3333333333333333)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Multiple pack operations should work");
}

#[tokio::test]
async fn test_mixed_pack_sizes() {
    let code = r#"
        let a = p64(0x1234567890abcdef)
        let b = p32(0x12345678)
        let c = p16(0x1234)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Mixed pack sizes should work");
}

#[tokio::test]
async fn test_pack_zero() {
    let code = r#"
        let zero = p64(0)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Packing zero should work");
}

#[tokio::test]
async fn test_pack_max_values() {
    let code = r#"
        let max64 = p64(0xffffffffffffffff)
        let max32 = p32(0xffffffff)
        let max16 = p16(0xffff)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Packing max values should work");
}

#[tokio::test]
async fn test_unpack_string() {
    let code = r#"
        let bytes = "AAAABBBB"
        let value = u64(bytes)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Unpacking string should work");
}

#[tokio::test]
async fn test_cyclic_small() {
    let code = r#"
        let pattern = cyclic(10)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Small cyclic pattern should work");
}

#[tokio::test]
async fn test_cyclic_large() {
    let code = r#"
        let pattern = cyclic(10000)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Large cyclic pattern should work");
}

#[tokio::test]
async fn test_cyclic_find_not_found() {
    let code = r#"
        let pattern = cyclic(100)
        let offset = cyclic_find(pattern, "ZZZZ")
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "cyclic_find() not found should return null");
}

#[tokio::test]
async fn test_builtin_in_expression() {
    let code = r#"
        let addr = 0x400000
        let payload = p64(addr)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Builtin in expression should work");
}

#[tokio::test]
async fn test_builtin_in_function() {
    let code = r#"
        fn build_rop() {
            let gadget = p64(0xdeadbeef)
            return gadget
        }
        let rop = build_rop()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Builtin in function should work");
}

#[tokio::test]
async fn test_builtin_in_loop() {
    let code = r#"
        let addresses = [0x1000, 0x2000, 0x3000]
        for addr in addresses {
            let packed = p64(addr)
        }
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Builtin in loop should work");
}

#[tokio::test]
async fn test_builtin_chaining() {
    let code = r#"
        let value = u64(p64(0x1234567890))
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Builtin chaining should work");
}

#[tokio::test]
async fn test_builtin_with_hex_literal() {
    let code = r#"
        let addr = p64(0xdeadbeef)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Builtin with hex literal should work");
}

#[tokio::test]
async fn test_builtin_with_variable() {
    let code = r#"
        let base = 0x400000
        let offset = 0x1234
        let packed = p64(base)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "Builtin with variable should work");
}

#[tokio::test]
async fn test_shellcode_builtin() {
    let code = r#"
        let sc = shellcode(arch: "x64", payload: "execve")
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "shellcode() builtin should work");
}

#[tokio::test]
async fn test_shellcode_with_defaults() {
    let code = r#"
        let sc = shellcode()
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "shellcode() with defaults should work");
}

#[tokio::test]
async fn test_shellcode_with_lhost_lport() {
    let code = r#"
        let sc = shellcode(arch: "x64", payload: "reverse_tcp", lhost: "127.0.0.1", lport: 4444)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "shellcode() with lhost/lport should work");
}

#[tokio::test]
async fn test_fmtstr_payload_builtin() {
    let code = r#"
        let payload = fmtstr_payload(offset: 6, writes: {"0x400000": 0xdeadbeef})
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "fmtstr_payload() should work");
}

#[tokio::test]
async fn test_fmtstr_payload_missing_offset() {
    let code = r#"
        let payload = fmtstr_payload(writes: {"0x400000": 0xdeadbeef})
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "fmtstr_payload() without offset should fail");
}

#[tokio::test]
async fn test_disasm_builtin() {
    let code = r#"
        let bytes = 0x9090909090
        disasm(bytes)
    "#;
    let result = run_script(code).await;
    assert!(result.is_ok(), "disasm() should work");
}

#[tokio::test]
async fn test_disasm_missing_arg() {
    let code = r#"
        disasm()
    "#;
    let result = run_script(code).await;
    assert!(result.is_err(), "disasm() without argument should fail");
}
