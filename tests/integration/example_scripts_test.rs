// Example Script Validation Integration Tests
// Tests all .talon example files to ensure they execute without errors

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use std::sync::Mutex;

const TIMEOUT_SECS: u64 = 30;

lazy_static::lazy_static! {
    static ref TEST_LOCK: Mutex<()> = Mutex::new(());
}

#[derive(Debug, Clone)]
struct ExampleTest {
    name: String,
    path: PathBuf,
    expected_to_pass: bool,
}

impl ExampleTest {
    fn new(path: PathBuf) -> Self {
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        Self {
            name,
            path,
            expected_to_pass: true,
        }
    }
}

fn find_all_examples() -> Vec<ExampleTest> {
    let examples_dir = Path::new("examples");
    
    if !examples_dir.exists() {
        eprintln!("Warning: examples/ directory not found");
        return Vec::new();
    }

    let mut examples = Vec::new();
    
    if let Ok(entries) = fs::read_dir(examples_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("talon") {
                examples.push(ExampleTest::new(path));
            }
        }
    }
    
    examples.sort_by(|a, b| a.name.cmp(&b.name));
    examples
}

fn run_example_with_timeout(example: &ExampleTest) -> Result<(), String> {
    let cargo_bin = env!("CARGO_BIN_EXE_talon");
    
    let child = Command::new(cargo_bin)
        .arg("run")
        .arg(&example.path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    let timeout = Duration::from_secs(TIMEOUT_SECS);
    
    let output = wait_timeout::ChildExt::wait_timeout(child, timeout)
        .map_err(|e| format!("Failed to wait for process: {}", e))?;
    
    match output {
        None => {
            Err(format!("Script timed out after {} seconds", TIMEOUT_SECS))
        }
        Some(status) => {
            let exit_code = status.exit_code();
            if exit_code == 0 {
                Ok(())
            } else {
                Err(format!("Script exited with code: {:?}", exit_code))
            }
        }
    }
}

#[test]
fn test_all_examples_execute() {
    let _lock = TEST_LOCK.lock().unwrap();
    
    let examples = find_all_examples();
    
    if examples.is_empty() {
        panic!("No example .talon files found in examples/ directory");
    }
    
    println!("\nFound {} example scripts to test", examples.len());
    println!("Timeout per script: {}s", TIMEOUT_SECS);
    println!("─────────────────────────────────────");
    
    let mut results = Vec::new();
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    
    for example in &examples {
        print!("Testing: {:40} ... ", example.name);
        
        match run_example_with_timeout(example) {
            Ok(()) => {
                println!("PASS");
                passed += 1;
                results.push((example.name.clone(), true, None));
            }
            Err(e) => {
                if example.expected_to_pass {
                    println!("FAIL: {}", e);
                    failed += 1;
                    results.push((example.name.clone(), false, Some(e)));
                } else {
                    println!("SKIP (expected fail)");
                    skipped += 1;
                }
            }
        }
    }
    
    println!("─────────────────────────────────────");
    println!("Summary: {} passed, {} failed, {} skipped", passed, failed, skipped);
    
    if failed > 0 {
        println!("\nFailed examples:");
        for (name, success, error) in results {
            if !success {
                println!("  - {}: {}", name, error.unwrap_or_default());
            }
        }
        panic!("{} example(s) failed to execute", failed);
    }
}

#[test]
fn test_tutorial_01_basics() {
    let _lock = TEST_LOCK.lock().unwrap();
    let example = ExampleTest::new(PathBuf::from("examples/tutorial_01_basics.talon"));
    run_example_with_timeout(&example).expect("tutorial_01_basics.talon should execute successfully");
}

#[test]
fn test_tutorial_02_exploitation() {
    let _lock = TEST_LOCK.lock().unwrap();
    let example = ExampleTest::new(PathBuf::from("examples/tutorial_02_exploitation.talon"));
    run_example_with_timeout(&example).expect("tutorial_02_exploitation.talon should execute successfully");
}

#[test]
fn test_buffer_overflow_rop() {
    let _lock = TEST_LOCK.lock().unwrap();
    let example = ExampleTest::new(PathBuf::from("examples/01_buffer_overflow_rop.talon"));
    run_example_with_timeout(&example).expect("01_buffer_overflow_rop.talon should execute successfully");
}

#[test]
fn test_format_string_attack() {
    let _lock = TEST_LOCK.lock().unwrap();
    let example = ExampleTest::new(PathBuf::from("examples/02_format_string_attack.talon"));
    run_example_with_timeout(&example).expect("02_format_string_attack.talon should execute successfully");
}

#[test]
fn test_heap_exploitation() {
    let _lock = TEST_LOCK.lock().unwrap();
    let example = ExampleTest::new(PathBuf::from("examples/05_heap_exploitation.talon"));
    run_example_with_timeout(&example).expect("05_heap_exploitation.talon should execute successfully");
}

#[test]
fn test_rop_dsl_showcase() {
    let _lock = TEST_LOCK.lock().unwrap();
    let example = ExampleTest::new(PathBuf::from("examples/rop_dsl_showcase.talon"));
    run_example_with_timeout(&example).expect("rop_dsl_showcase.talon should execute successfully");
}

#[test]
fn test_beginner_ctf_template() {
    let _lock = TEST_LOCK.lock().unwrap();
    let example = ExampleTest::new(PathBuf::from("examples/beginner_ctf_template.talon"));
    run_example_with_timeout(&example).expect("beginner_ctf_template.talon should execute successfully");
}

#[cfg(test)]
mod resource_limits {
    use super::*;
    
    #[test]
    fn test_script_respects_timeout() {
        let _lock = TEST_LOCK.lock().unwrap();
        
        let temp_dir = tempfile::tempdir().unwrap();
        let infinite_loop_script = temp_dir.path().join("infinite_loop.talon");
        
        fs::write(&infinite_loop_script, r#"
# Infinite loop test
let i = 0
while true
    let i = i + 1
end
"#).unwrap();
        
        let example = ExampleTest::new(infinite_loop_script);
        let result = run_example_with_timeout(&example);
        
        assert!(result.is_err(), "Infinite loop should timeout");
        assert!(result.unwrap_err().contains("timed out"), "Error should mention timeout");
    }
    
    #[test]
    fn test_syntax_error_fails_gracefully() {
        let _lock = TEST_LOCK.lock().unwrap();
        
        let temp_dir = tempfile::tempdir().unwrap();
        let syntax_error_script = temp_dir.path().join("syntax_error.talon");
        
        fs::write(&syntax_error_script, r#"
# Invalid syntax
let x = 
let y = "unclosed string
this is not valid Talon syntax !!!
"#).unwrap();
        
        let example = ExampleTest::new(syntax_error_script);
        let result = run_example_with_timeout(&example);
        
        assert!(result.is_err(), "Syntax error should fail");
    }
    
    #[test]
    fn test_empty_script_handling() {
        let _lock = TEST_LOCK.lock().unwrap();
        
        let temp_dir = tempfile::tempdir().unwrap();
        let empty_script = temp_dir.path().join("empty.talon");
        
        fs::write(&empty_script, "").unwrap();
        
        let example = ExampleTest::new(empty_script);
        let result = run_example_with_timeout(&example);
        
        assert!(result.is_err(), "Empty script should fail");
    }
}

#[cfg(test)]
mod example_content_validation {
    use super::*;
    
    #[test]
    fn test_all_examples_have_content() {
        let examples = find_all_examples();
        
        for example in examples {
            let content = fs::read_to_string(&example.path)
                .expect(&format!("Should be able to read {}", example.name));
            
            assert!(!content.trim().is_empty(), 
                "Example {} should not be empty", example.name);
            
            assert!(content.len() > 10, 
                "Example {} seems too short ({}bytes)", example.name, content.len());
        }
    }
    
    #[test]
    fn test_examples_have_comments() {
        let examples = find_all_examples();
        
        let mut uncommented = Vec::new();
        
        for example in examples {
            let content = fs::read_to_string(&example.path).unwrap();
            
            if !content.contains('#') {
                uncommented.push(example.name);
            }
        }
        
        if !uncommented.is_empty() {
            println!("Warning: The following examples have no comments:");
            for name in &uncommented {
                println!("  - {}", name);
            }
        }
    }
}
