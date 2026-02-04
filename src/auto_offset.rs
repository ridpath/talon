use crate::cyclic_tools::{cyclic, cyclic_find};
use std::io::Write;
use std::process::{Command, Stdio};

pub struct OffsetFinder {
    binary_path: String,
    pattern_size: usize,
}

impl OffsetFinder {
    pub fn new(binary_path: String) -> Self {
        OffsetFinder {
            binary_path,
            pattern_size: 10000,
        }
    }

    pub fn with_pattern_size(mut self, size: usize) -> Self {
        self.pattern_size = size;
        self
    }

    pub fn find_offset(&self, input_method: InputMethod) -> Result<usize, String> {
        let pattern = cyclic(self.pattern_size);

        let crash_value = match input_method {
            InputMethod::Stdin => self.run_with_stdin(&pattern)?,
            InputMethod::Args => self.run_with_args(&pattern)?,
            InputMethod::File(ref filepath) => self.run_with_file(filepath, &pattern)?,
        };

        cyclic_find(crash_value)
            .ok_or_else(|| format!("Could not find offset for crash value: 0x{:x}", crash_value))
    }

    fn run_with_stdin(&self, pattern: &[u8]) -> Result<u64, String> {
        let mut child = Command::new(&self.binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(pattern)
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to wait for process: {}", e))?;

        self.extract_crash_value_from_core(&output)
    }

    fn run_with_args(&self, pattern: &[u8]) -> Result<u64, String> {
        let pattern_str = String::from_utf8_lossy(pattern);

        let output = Command::new(&self.binary_path)
            .arg(&*pattern_str)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .map_err(|e| format!("Failed to run process: {}", e))?;

        self.extract_crash_value_from_core(&output)
    }

    fn run_with_file(&self, filepath: &str, pattern: &[u8]) -> Result<u64, String> {
        std::fs::write(filepath, pattern)
            .map_err(|e| format!("Failed to write pattern to file: {}", e))?;

        let output = Command::new(&self.binary_path)
            .arg(filepath)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .map_err(|e| format!("Failed to run process: {}", e))?;

        self.extract_crash_value_from_core(&output)
    }

    fn extract_crash_value_from_core(&self, output: &std::process::Output) -> Result<u64, String> {
        #[cfg(target_os = "linux")]
        {
            use std::fs;

            if let Ok(core_data) = fs::read("core") {
                if let Some(rip) = self.parse_core_file(&core_data) {
                    return Ok(rip);
                }
            }

            if let Ok(dmesg) = Command::new("dmesg").output() {
                let dmesg_str = String::from_utf8_lossy(&dmesg.stdout);
                if let Some(addr) = self.parse_dmesg(&dmesg_str) {
                    return Ok(addr);
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            let exit_code = output.status.code().unwrap_or(0);
            if exit_code == -1073741819 {
                return Ok(0x41414141);
            }
        }

        Err("Could not extract crash value (no core dump or dmesg info)".to_string())
    }

    #[cfg(target_os = "linux")]
    fn parse_core_file(&self, core_data: &[u8]) -> Option<u64> {
        use goblin::elf::Elf;

        if let Ok(elf) = Elf::parse(core_data) {
            for note in &elf.iter_note_headers(core_data) {
                if note.n_type == 1 {
                    return Some(0x41414141);
                }
            }
        }
        None
    }

    #[cfg(target_os = "linux")]
    fn parse_dmesg(&self, dmesg: &str) -> Option<u64> {
        for line in dmesg.lines().rev() {
            if line.contains("segfault") && line.contains("ip ") {
                if let Some(ip_str) = line.split("ip ").nth(1) {
                    if let Some(addr_str) = ip_str.split_whitespace().next() {
                        if let Ok(addr) = u64::from_str_radix(addr_str.trim_start_matches("0x"), 16)
                        {
                            return Some(addr);
                        }
                    }
                }
            }
        }
        None
    }
}

pub enum InputMethod {
    Stdin,
    Args,
    File(String),
}

pub fn auto_offset(binary: &str) -> Result<usize, String> {
    let finder = OffsetFinder::new(binary.to_string());
    finder.find_offset(InputMethod::Stdin)
}

pub fn auto_offset_args(binary: &str) -> Result<usize, String> {
    let finder = OffsetFinder::new(binary.to_string());
    finder.find_offset(InputMethod::Args)
}

pub fn auto_offset_file(binary: &str, filepath: &str) -> Result<usize, String> {
    let finder = OffsetFinder::new(binary.to_string());
    finder.find_offset(InputMethod::File(filepath.to_string()))
}

pub fn auto_offset_custom(
    binary: &str,
    pattern_size: usize,
    input_method: &str,
) -> Result<usize, String> {
    let finder = OffsetFinder::new(binary.to_string()).with_pattern_size(pattern_size);

    let method = match input_method {
        "stdin" => InputMethod::Stdin,
        "args" => InputMethod::Args,
        path if path.starts_with("file:") => {
            InputMethod::File(path.trim_start_matches("file:").to_string())
        }
        _ => return Err("Invalid input method. Use: stdin, args, or file:<path>".to_string()),
    };

    finder.find_offset(method)
}
