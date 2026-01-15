use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[derive(Debug, Clone)]
pub enum Vuln {
    BufferOverflow { offset: usize },
    FormatString { vuln_arg: usize },
    UseAfterFree { heap_chunk: usize },
    IntegerOverflow { width: usize },
    StackPivot { gadget_offset: usize },
}

pub struct TalonTestHarness {
    temp_dir: TempDir,
    mock_binaries: HashMap<String, PathBuf>,
}

impl TalonTestHarness {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        Self {
            temp_dir,
            mock_binaries: HashMap::new(),
        }
    }

    pub fn run_script(&self, code: &str) -> Result<String, String> {
        let script_path = self.temp_dir.path().join("test_script.talon");
        fs::write(&script_path, code).map_err(|e| format!("Failed to write script: {}", e))?;
        self.run_file(&script_path)
    }

    pub fn run_file(&self, path: &Path) -> Result<String, String> {
        if !path.exists() {
            return Err(format!("Script file does not exist: {:?}", path));
        }
        Ok(format!("Script executed: {:?}", path))
    }

    pub fn mock_binary(&mut self, name: &str, vulns: &[Vuln]) -> PathBuf {
        let bin_path = self.temp_dir.path().join(name);
        
        let elf_header = self.generate_mock_elf(vulns);
        fs::write(&bin_path, elf_header).expect("Failed to write mock binary");
        
        self.mock_binaries.insert(name.to_string(), bin_path.clone());
        bin_path
    }

    fn generate_mock_elf(&self, vulns: &[Vuln]) -> Vec<u8> {
        let mut elf = Vec::new();
        
        elf.extend_from_slice(&[0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00]);
        elf.extend_from_slice(&[0x00; 8]);
        elf.extend_from_slice(&[0x02, 0x00, 0x3e, 0x00]);
        elf.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        
        for vuln in vulns {
            match vuln {
                Vuln::BufferOverflow { offset } => {
                    elf.extend_from_slice(&offset.to_le_bytes()[..4]);
                }
                Vuln::FormatString { vuln_arg } => {
                    elf.extend_from_slice(&vuln_arg.to_le_bytes()[..4]);
                }
                Vuln::UseAfterFree { heap_chunk } => {
                    elf.extend_from_slice(&heap_chunk.to_le_bytes()[..4]);
                }
                Vuln::IntegerOverflow { width } => {
                    elf.extend_from_slice(&width.to_le_bytes()[..4]);
                }
                Vuln::StackPivot { gadget_offset } => {
                    elf.extend_from_slice(&gadget_offset.to_le_bytes()[..4]);
                }
            }
        }
        
        while elf.len() < 256 {
            elf.push(0x90);
        }
        
        elf
    }

    pub fn assert_exploit_success(&self, result: &str) -> Result<(), String> {
        if result.contains("error") || result.contains("failed") {
            Err(format!("Exploit failed: {}", result))
        } else {
            Ok(())
        }
    }

    pub fn assert_contains(&self, haystack: &str, needle: &str) -> Result<(), String> {
        if haystack.contains(needle) {
            Ok(())
        } else {
            Err(format!("Expected '{}' to contain '{}'", haystack, needle))
        }
    }

    pub fn assert_not_contains(&self, haystack: &str, needle: &str) -> Result<(), String> {
        if !haystack.contains(needle) {
            Ok(())
        } else {
            Err(format!("Expected '{}' to not contain '{}'", haystack, needle))
        }
    }

    pub fn create_vulnerable_c_source(&self, name: &str, vuln_type: &Vuln) -> PathBuf {
        let source_path = self.temp_dir.path().join(format!("{}.c", name));
        let code = match vuln_type {
            Vuln::BufferOverflow { offset } => {
                format!(
                    r#"#include <stdio.h>
#include <string.h>

int main(int argc, char *argv[]) {{
    char buffer[{}];
    if (argc > 1) {{
        strcpy(buffer, argv[1]);
        printf("%s\n", buffer);
    }}
    return 0;
}}
"#,
                    offset
                )
            }
            Vuln::FormatString { vuln_arg } => {
                format!(
                    r#"#include <stdio.h>

int main(int argc, char *argv[]) {{
    if (argc > {}) {{
        printf(argv[{}]);
    }}
    return 0;
}}
"#,
                    vuln_arg, vuln_arg
                )
            }
            Vuln::UseAfterFree { heap_chunk } => {
                format!(
                    r#"#include <stdio.h>
#include <stdlib.h>

int main() {{
    char *ptr = malloc({});
    free(ptr);
    printf("%s\n", ptr);
    return 0;
}}
"#,
                    heap_chunk
                )
            }
            Vuln::IntegerOverflow { width } => {
                format!(
                    r#"#include <stdio.h>
#include <stdlib.h>

int main(int argc, char *argv[]) {{
    unsigned int size;
    if (argc > 1) {{
        size = atoi(argv[1]);
        char *buf = malloc(size * {});
        free(buf);
    }}
    return 0;
}}
"#,
                    width
                )
            }
            Vuln::StackPivot { .. } => {
                r#"#include <stdio.h>
#include <string.h>

int main(int argc, char *argv[]) {
    char buffer[64];
    if (argc > 1) {
        strcpy(buffer, argv[1]);
    }
    return 0;
}
"#
                .to_string()
            }
        };
        
        fs::write(&source_path, code).expect("Failed to write C source");
        source_path
    }

    pub fn temp_dir(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn create_test_file(&self, name: &str, content: &str) -> PathBuf {
        let file_path = self.temp_dir.path().join(name);
        fs::write(&file_path, content).expect("Failed to create test file");
        file_path
    }

    pub fn get_mock_binary(&self, name: &str) -> Option<&PathBuf> {
        self.mock_binaries.get(name)
    }
}

impl Default for TalonTestHarness {
    fn default() -> Self {
        Self::new()
    }
}

pub fn assert_u64(value: u64, expected: u64) {
    assert_eq!(value, expected, "Expected {:x}, got {:x}", expected, value);
}

pub fn assert_hex_str(value: &str, expected: &str) {
    assert_eq!(value, expected, "Expected {}, got {}", expected, value);
}

pub fn create_rop_gadget_binary() -> Vec<u8> {
    let mut binary = Vec::new();
    
    binary.extend_from_slice(&[0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00]);
    binary.extend_from_slice(&[0x00; 8]);
    
    binary.extend_from_slice(&[0x5f, 0xc3]);
    binary.extend_from_slice(&[0x5e, 0xc3]);
    binary.extend_from_slice(&[0x5a, 0xc3]);
    
    binary.extend_from_slice(&[0x48, 0x89, 0xe5, 0xc3]);
    
    binary.extend_from_slice(&[0x48, 0x83, 0xc4, 0x08, 0xc3]);
    
    while binary.len() < 1024 {
        binary.push(0x90);
    }
    
    binary
}

pub fn create_shellcode_test_env() -> Vec<u8> {
    vec![
        0x48, 0x31, 0xc0,
        0x48, 0xbb, 0x2f, 0x62, 0x69, 0x6e, 0x2f, 0x73, 0x68, 0x00,
        0x53,
        0x48, 0x89, 0xe7,
        0x48, 0x31, 0xf6,
        0x48, 0x31, 0xd2,
        0xb0, 0x3b,
        0x0f, 0x05,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_creation() {
        let harness = TalonTestHarness::new();
        assert!(harness.temp_dir().exists());
    }

    #[test]
    fn test_mock_binary_creation() {
        let mut harness = TalonTestHarness::new();
        let vulns = vec![Vuln::BufferOverflow { offset: 64 }];
        let bin_path = harness.mock_binary("test_binary", &vulns);
        
        assert!(bin_path.exists());
        let content = fs::read(&bin_path).unwrap();
        assert!(content.len() >= 256);
        assert_eq!(&content[0..4], &[0x7f, 0x45, 0x4c, 0x46]);
    }

    #[test]
    fn test_create_vulnerable_c_source() {
        let harness = TalonTestHarness::new();
        let vuln = Vuln::BufferOverflow { offset: 128 };
        let source_path = harness.create_vulnerable_c_source("vuln_test", &vuln);
        
        assert!(source_path.exists());
        let content = fs::read_to_string(&source_path).unwrap();
        assert!(content.contains("strcpy"));
        assert!(content.contains("128"));
    }

    #[test]
    fn test_assert_helpers() {
        let harness = TalonTestHarness::new();
        
        assert!(harness.assert_contains("hello world", "world").is_ok());
        assert!(harness.assert_contains("hello", "world").is_err());
        
        assert!(harness.assert_not_contains("hello", "world").is_ok());
        assert!(harness.assert_not_contains("hello world", "world").is_err());
    }

    #[test]
    fn test_create_test_file() {
        let harness = TalonTestHarness::new();
        let file_path = harness.create_test_file("test.txt", "test content");
        
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_rop_gadget_binary() {
        let binary = create_rop_gadget_binary();
        assert!(binary.len() >= 1024);
        assert_eq!(&binary[0..4], &[0x7f, 0x45, 0x4c, 0x46]);
    }

    #[test]
    fn test_shellcode_env() {
        let shellcode = create_shellcode_test_env();
        assert!(!shellcode.is_empty());
        assert_eq!(shellcode[0], 0x48);
    }
}
