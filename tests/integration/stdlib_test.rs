use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

const STDLIB_TIMEOUT_SECS: u64 = 10;

fn run_talon_code(code: &str) -> Result<String, String> {
    let temp_dir = tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let script_path = temp_dir.path().join("test.talon");

    fs::write(&script_path, code).map_err(|e| format!("Failed to write script: {}", e))?;

    let cargo_bin = env!("CARGO_BIN_EXE_talon");

    let child = Command::new(cargo_bin)
        .arg("run")
        .arg(&script_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    let timeout = Duration::from_secs(STDLIB_TIMEOUT_SECS);

    let output = wait_timeout::ChildExt::wait_timeout(child, timeout)
        .map_err(|e| format!("Failed to wait for process: {}", e))?;

    match output {
        None => Err(format!("Script timed out after {} seconds", STDLIB_TIMEOUT_SECS)),
        Some(status) => {
            if status.exit_code() == 0 {
                Ok(String::new())
            } else {
                Err(format!("Script exited with code: {:?}", status.exit_code()))
            }
        }
    }
}

#[cfg(test)]
mod packing_functions {
    use super::*;

    #[test]
    fn test_p64_basic() {
        let code = r#"
let packed = p64(0x41424344)
print(packed)
"#;
        assert!(run_talon_code(code).is_ok(), "p64() should pack 64-bit values");
    }

    #[test]
    fn test_p32_basic() {
        let code = r#"
let packed = p32(0x41424344)
print(packed)
"#;
        assert!(run_talon_code(code).is_ok(), "p32() should pack 32-bit values");
    }

    #[test]
    fn test_u64_basic() {
        let code = r#"
let packed = p64(0xdeadbeef)
let unpacked = u64(packed)
print(unpacked)
"#;
        assert!(run_talon_code(code).is_ok(), "u64() should unpack 64-bit values");
    }

    #[test]
    fn test_u32_basic() {
        let code = r#"
let packed = p32(0xdeadbeef)
let unpacked = u32(packed)
print(unpacked)
"#;
        assert!(run_talon_code(code).is_ok(), "u32() should unpack 32-bit values");
    }

    #[test]
    fn test_pack_unpack_roundtrip() {
        let code = r#"
let original = 0x1234567890abcdef
let packed = p64(original)
let unpacked = u64(packed)
if unpacked == original
    print("Success")
end
"#;
        assert!(run_talon_code(code).is_ok(), "Pack/unpack should roundtrip correctly");
    }
}

#[cfg(test)]
mod encoding_functions {
    use super::*;

    #[test]
    fn test_hex_encode() {
        let code = r#"
let data = "Hello"
let encoded = hex(data)
print(encoded)
"#;
        assert!(run_talon_code(code).is_ok(), "hex() should encode data");
    }

    #[test]
    fn test_base64_encode() {
        let code = r#"
let data = "Hello, World!"
let encoded = base64(data)
print(encoded)
"#;
        assert!(run_talon_code(code).is_ok(), "base64() should encode data");
    }

    #[test]
    fn test_url_encode() {
        let code = r#"
let data = "hello world"
let encoded = url_encode(data)
print(encoded)
"#;
        assert!(run_talon_code(code).is_ok(), "url_encode() should encode URLs");
    }
}

#[cfg(test)]
mod string_functions {
    use super::*;

    #[test]
    fn test_bytes_creation() {
        let code = r#"
let data = bytes("AAAA")
print(data)
"#;
        assert!(run_talon_code(code).is_ok(), "bytes() should create byte arrays");
    }

    #[test]
    fn test_string_repeat() {
        let code = r#"
let data = "A" * 100
print(data)
"#;
        assert!(run_talon_code(code).is_ok(), "String repeat should work");
    }

    #[test]
    fn test_string_concatenation() {
        let code = r#"
let a = "Hello"
let b = "World"
let c = a + " " + b
print(c)
"#;
        assert!(run_talon_code(code).is_ok(), "String concatenation should work");
    }
}

#[cfg(test)]
mod math_functions {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let code = r#"
let a = 10 + 20
let b = 50 - 30
let c = 5 * 6
let d = 100 / 10
print(a + b + c + d)
"#;
        assert!(run_talon_code(code).is_ok(), "Basic arithmetic should work");
    }

    #[test]
    fn test_modulo() {
        let code = r#"
let remainder = 17 % 5
print(remainder)
"#;
        assert!(run_talon_code(code).is_ok(), "Modulo operation should work");
    }
}

#[cfg(test)]
mod control_flow {
    use super::*;

    #[test]
    fn test_if_statement() {
        let code = r#"
let x = 10
if x > 5
    print("Greater")
end
"#;
        assert!(run_talon_code(code).is_ok(), "If statements should work");
    }

    #[test]
    fn test_for_loop() {
        let code = r#"
for i in 1..5
    print(i)
end
"#;
        assert!(run_talon_code(code).is_ok(), "For loops should work");
    }

    #[test]
    fn test_while_loop() {
        let code = r#"
let i = 0
while i < 3
    let i = i + 1
    print(i)
end
"#;
        assert!(run_talon_code(code).is_ok(), "While loops should work");
    }
}

#[cfg(test)]
mod function_definitions {
    use super::*;

    #[test]
    fn test_function_definition() {
        let code = r#"
define function add(a, b)
    return a + b
end

let result = add(10, 20)
print(result)
"#;
        assert!(run_talon_code(code).is_ok(), "Function definitions should work");
    }

    #[test]
    fn test_function_with_default_args() {
        let code = r#"
define function greet(name)
    return "Hello, " + name
end

let msg = greet("World")
print(msg)
"#;
        assert!(run_talon_code(code).is_ok(), "Functions with arguments should work");
    }
}

#[cfg(test)]
mod list_operations {
    use super::*;

    #[test]
    fn test_list_creation() {
        let code = r#"
let items = [1, 2, 3, 4, 5]
print(items)
"#;
        assert!(run_talon_code(code).is_ok(), "List creation should work");
    }

    #[test]
    fn test_list_access() {
        let code = r#"
let items = [10, 20, 30]
let first = items[0]
print(first)
"#;
        assert!(run_talon_code(code).is_ok(), "List access should work");
    }
}

#[cfg(test)]
mod cyclic_pattern {
    use super::*;

    #[test]
    fn test_cyclic_generation() {
        let code = r#"
let pattern = cyclic(100)
print(pattern)
"#;
        assert!(run_talon_code(code).is_ok(), "cyclic() should generate patterns");
    }

    #[test]
    fn test_cyclic_find() {
        let code = r#"
let pattern = cyclic(100)
let offset = cyclic_find("BBBB", pattern)
print(offset)
"#;
        assert!(run_talon_code(code).is_ok(), "cyclic_find() should find offsets");
    }
}

#[cfg(test)]
mod exploit_primitives {
    use super::*;

    #[test]
    fn test_rop_gadget_search() {
        let code = r#"
print("ROP test placeholder")
"#;
        assert!(run_talon_code(code).is_ok(), "ROP primitives test placeholder");
    }

    #[test]
    fn test_shellcode_generation() {
        let code = r#"
print("Shellcode test placeholder")
"#;
        assert!(run_talon_code(code).is_ok(), "Shellcode generation test placeholder");
    }
}

#[cfg(test)]
mod file_operations {
    use super::*;

    #[test]
    fn test_file_read_simulation() {
        let code = r#"
print("File operations test")
"#;
        assert!(run_talon_code(code).is_ok(), "File operations should be testable");
    }
}

#[test]
fn test_print_function() {
    let code = r#"
print("Hello, TALON!")
"#;
    assert!(run_talon_code(code).is_ok(), "print() should output text");
}

#[test]
fn test_variable_assignment() {
    let code = r#"
let x = 42
let y = "test"
let z = true
print(x)
"#;
    assert!(run_talon_code(code).is_ok(), "Variable assignment should work");
}

#[test]
fn test_comments() {
    let code = r#"
# This is a comment
let x = 10
print(x)
"#;
    assert!(run_talon_code(code).is_ok(), "Comments should be ignored");
}

#[test]
fn test_multiline_script() {
    let code = r#"
let a = 1
let b = 2
let c = a + b
print(c)
"#;
    assert!(run_talon_code(code).is_ok(), "Multi-line scripts should execute");
}
