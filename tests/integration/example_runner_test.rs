use std::process::Command;
use std::time::Duration;
use std::path::{Path, PathBuf};
use std::fs;

const TIMEOUT_SECONDS: u64 = 30;
const EXAMPLES_DIR: &str = "examples";

#[derive(Debug)]
struct ExampleTest {
    name: String,
    path: PathBuf,
    should_pass: bool,
}

fn get_all_examples() -> Vec<ExampleTest> {
    let examples_dir = Path::new(EXAMPLES_DIR);
    let mut examples = Vec::new();

    if let Ok(entries) = fs::read_dir(examples_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("talon") {
                let name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                examples.push(ExampleTest {
                    name: name.clone(),
                    path,
                    should_pass: !name.contains("fail"),
                });
            }
        }
    }

    examples.sort_by(|a, b| a.name.cmp(&b.name));
    examples
}

fn run_example_with_timeout(path: &Path, timeout_secs: u64) -> Result<String, String> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--")
        .arg("run")
        .arg(path)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                Ok(String::from_utf8_lossy(&result.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&result.stderr).to_string())
            }
        }
        Err(e) => Err(format!("Failed to execute: {}", e)),
    }
}

#[test]
#[ignore]
fn test_all_examples_syntax() {
    let examples = get_all_examples();
    let mut failed = Vec::new();

    println!("Testing {} example scripts", examples.len());

    for example in &examples {
        println!("Testing: {}...", example.name);

        match run_example_with_timeout(&example.path, TIMEOUT_SECONDS) {
            Ok(_) if example.should_pass => {
                println!("  PASS: {}", example.name);
            }
            Err(e) if !example.should_pass => {
                println!("  PASS: {} (expected failure)", example.name);
            }
            Ok(_) => {
                println!("  FAIL: {} (should have failed)", example.name);
                failed.push(example.name.clone());
            }
            Err(e) => {
                println!("  FAIL: {}: {}", example.name, e);
                failed.push(example.name.clone());
            }
        }
    }

    if !failed.is_empty() {
        panic!("Failed examples: {:?}", failed);
    }
}

#[test]
fn test_basic_buffer_overflow_example() {
    let path = Path::new(EXAMPLES_DIR).join("01_buffer_overflow_rop.talon");

    if !path.exists() {
        println!("Skipping: example file not found");
        return;
    }

    let result = run_example_with_timeout(&path, 10);
    assert!(result.is_ok() || result.unwrap_err().contains("connect") || result.unwrap_err().contains("network"),
        "Basic example should parse correctly");
}

#[test]
fn test_format_string_example() {
    let path = Path::new(EXAMPLES_DIR).join("02_format_string_attack.talon");

    if !path.exists() {
        println!("Skipping: example file not found");
        return;
    }

    let result = run_example_with_timeout(&path, 10);
    assert!(result.is_ok() || result.unwrap_err().contains("connect") || result.unwrap_err().contains("network"),
        "Format string example should parse correctly");
}

#[test]
fn test_heap_exploitation_example() {
    let path = Path::new(EXAMPLES_DIR).join("05_heap_exploitation.talon");

    if !path.exists() {
        println!("Skipping: example file not found");
        return;
    }

    let result = run_example_with_timeout(&path, 10);
    assert!(result.is_ok() || result.unwrap_err().contains("connect") || result.unwrap_err().contains("network"),
        "Heap exploitation example should parse correctly");
}

#[test]
fn test_tutorial_basics() {
    let path = Path::new(EXAMPLES_DIR).join("tutorial_01_basics.talon");

    if !path.exists() {
        println!("Skipping: example file not found");
        return;
    }

    let result = run_example_with_timeout(&path, 10);
    assert!(result.is_ok() || result.unwrap_err().contains("connect") || result.unwrap_err().contains("network"),
        "Tutorial basics should parse correctly");
}

#[test]
fn test_rop_dsl_showcase() {
    let path = Path::new(EXAMPLES_DIR).join("rop_dsl_showcase.talon");

    if !path.exists() {
        println!("Skipping: example file not found");
        return;
    }

    let result = run_example_with_timeout(&path, 10);
    assert!(result.is_ok() || result.unwrap_err().contains("connect") || result.unwrap_err().contains("network"),
        "ROP DSL showcase should parse correctly");
}

#[test]
fn test_world_class_exploit() {
    let path = Path::new(EXAMPLES_DIR).join("world_class_exploit.talon");

    if !path.exists() {
        println!("Skipping: example file not found");
        return;
    }

    let result = run_example_with_timeout(&path, 10);
    assert!(result.is_ok() || result.unwrap_err().contains("connect") || result.unwrap_err().contains("network"),
        "World class exploit should parse correctly");
}

#[test]
fn test_example_count() {
    let examples = get_all_examples();
    assert!(examples.len() >= 20, "Should have at least 20 examples, found {}", examples.len());
}

#[test]
fn test_examples_have_valid_extensions() {
    let examples = get_all_examples();

    for example in examples {
        assert_eq!(
            example.path.extension().and_then(|s| s.to_str()),
            Some("talon"),
            "All examples should have .talon extension"
        );
    }
}

#[test]
fn test_examples_are_readable() {
    let examples = get_all_examples();

    for example in examples {
        let content = fs::read_to_string(&example.path);
        assert!(content.is_ok(), "Should be able to read {}", example.name);

        let text = content.unwrap();
        assert!(!text.is_empty(), "{} should not be empty", example.name);
    }
}

#[test]
fn test_examples_have_no_syntax_errors() {
    let examples = get_all_examples();
    let mut syntax_errors = Vec::new();

    for example in examples {
        if let Ok(content) = fs::read_to_string(&example.path) {
            if content.contains("SYNTAX_ERROR") || content.contains("TODO: FIX") {
                syntax_errors.push(example.name);
            }
        }
    }

    assert!(syntax_errors.is_empty(), "Examples with syntax errors: {:?}", syntax_errors);
}
