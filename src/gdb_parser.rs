use std::collections::HashMap;
use std::process::Command;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct GdbRegisterState {
    pub registers: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct GdbBacktrace {
    pub frames: Vec<GdbFrame>,
}

#[derive(Debug, Clone)]
pub struct GdbFrame {
    pub level: usize,
    pub address: u64,
    pub function: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GdbCrashInfo {
    pub signal: String,
    pub crash_address: Option<u64>,
    pub instruction_pointer: u64,
    pub registers: HashMap<String, u64>,
    pub backtrace: Vec<GdbFrame>,
}

pub struct GdbParser;

impl GdbParser {
    pub fn run_and_parse(binary: &str, input: Option<&[u8]>) -> Result<GdbCrashInfo, String> {
        let temp_input = if let Some(data) = input {
            let temp_path = std::env::temp_dir().join("talon_gdb_input");
            std::fs::write(&temp_path, data)
                .map_err(|e| format!("Failed to write temp input: {}", e))?;
            Some(temp_path)
        } else {
            None
        };

        let mut gdb_commands = vec![
            "set pagination off".to_string(),
            "set confirm off".to_string(),
        ];

        if let Some(ref input_file) = temp_input {
            gdb_commands.push(format!("run < {}", input_file.display()));
        } else {
            gdb_commands.push("run".to_string());
        }

        gdb_commands.extend(vec![
            "info registers".to_string(),
            "backtrace".to_string(),
            "quit".to_string(),
        ]);

        let gdb_script = gdb_commands.join("\n");
        let script_path = std::env::temp_dir().join("talon_gdb_script.gdb");
        std::fs::write(&script_path, gdb_script)
            .map_err(|e| format!("Failed to write GDB script: {}", e))?;

        let output = Command::new("gdb")
            .arg("-batch")
            .arg("-x")
            .arg(&script_path)
            .arg(binary)
            .output()
            .map_err(|e| format!("Failed to run GDB: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let full_output = format!("{}\n{}", stdout, stderr);

        Self::parse_crash_info(&full_output)
    }

    pub fn parse_crash_info(output: &str) -> Result<GdbCrashInfo, String> {
        let signal = Self::parse_signal(output)?;
        let crash_address = Self::parse_crash_address(output);
        let registers = Self::parse_registers(output);
        let instruction_pointer = registers.get("rip")
            .or_else(|| registers.get("eip"))
            .or_else(|| registers.get("pc"))
            .copied()
            .unwrap_or(0);
        let backtrace = Self::parse_backtrace(output);

        Ok(GdbCrashInfo {
            signal,
            crash_address,
            instruction_pointer,
            registers,
            backtrace,
        })
    }

    fn parse_signal(output: &str) -> Result<String, String> {
        for line in output.lines() {
            if line.contains("SIGSEGV") {
                return Ok("SIGSEGV".to_string());
            } else if line.contains("SIGABRT") {
                return Ok("SIGABRT".to_string());
            } else if line.contains("SIGILL") {
                return Ok("SIGILL".to_string());
            } else if line.contains("SIGFPE") {
                return Ok("SIGFPE".to_string());
            } else if line.contains("exited normally") {
                return Ok("EXIT".to_string());
            }
        }
        Err("No crash or signal detected".to_string())
    }

    fn parse_crash_address(output: &str) -> Option<u64> {
        let addr_re = Regex::new(r"0x([0-9a-fA-F]+)").ok()?;
        
        for line in output.lines() {
            if line.contains("Segmentation fault") || line.contains("SIGSEGV") {
                if let Some(cap) = addr_re.captures(line) {
                    if let Ok(addr) = u64::from_str_radix(&cap[1], 16) {
                        return Some(addr);
                    }
                }
            }
        }
        None
    }

    pub fn parse_registers(output: &str) -> HashMap<String, u64> {
        let mut registers = HashMap::new();
        let reg_re = Regex::new(r"(\w+)\s+0x([0-9a-fA-F]+)").unwrap();
        
        let mut in_registers = false;
        for line in output.lines() {
            if line.contains("info registers") || line.trim().starts_with("rax") || line.trim().starts_with("eax") {
                in_registers = true;
            }
            
            if line.contains("backtrace") || line.starts_with("#") {
                in_registers = false;
            }
            
            if in_registers {
                for cap in reg_re.captures_iter(line) {
                    let reg_name = cap[1].to_lowercase();
                    if let Ok(value) = u64::from_str_radix(&cap[2], 16) {
                        registers.insert(reg_name, value);
                    }
                }
            }
        }
        
        registers
    }

    pub fn parse_backtrace(output: &str) -> Vec<GdbFrame> {
        let mut frames = Vec::new();
        let frame_re = Regex::new(r"#(\d+)\s+0x([0-9a-fA-F]+)\s+in\s+([^\s(]+)").unwrap();
        
        for line in output.lines() {
            if let Some(cap) = frame_re.captures(line) {
                if let (Ok(level), Ok(addr)) = (
                    cap[1].parse::<usize>(),
                    u64::from_str_radix(&cap[2], 16)
                ) {
                    frames.push(GdbFrame {
                        level,
                        address: addr,
                        function: cap[3].to_string(),
                        args: Vec::new(),
                    });
                }
            }
        }
        
        frames
    }

    pub fn quick_run(binary: &str, args: &[&str]) -> Result<GdbCrashInfo, String> {
        let mut gdb_commands = vec![
            "set pagination off".to_string(),
            "set confirm off".to_string(),
        ];

        if args.is_empty() {
            gdb_commands.push("run".to_string());
        } else {
            gdb_commands.push(format!("run {}", args.join(" ")));
        }

        gdb_commands.extend(vec![
            "info registers".to_string(),
            "backtrace".to_string(),
            "quit".to_string(),
        ]);

        let gdb_script = gdb_commands.join("\n");
        let script_path = std::env::temp_dir().join("talon_gdb_quick.gdb");
        std::fs::write(&script_path, gdb_script)
            .map_err(|e| format!("Failed to write GDB script: {}", e))?;

        let output = Command::new("gdb")
            .arg("-batch")
            .arg("-x")
            .arg(&script_path)
            .arg(binary)
            .output()
            .map_err(|e| format!("Failed to run GDB: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let full_output = format!("{}\n{}", stdout, stderr);

        Self::parse_crash_info(&full_output)
    }

    pub fn get_crash_offset(binary: &str, pattern: &[u8]) -> Result<usize, String> {
        let crash_info = Self::run_and_parse(binary, Some(pattern))?;
        
        let crash_value = crash_info.instruction_pointer;
        
        crate::cyclic_tools::cyclic_find(crash_value)
            .ok_or_else(|| format!("Could not find offset for crash at 0x{:x}", crash_value))
    }

    pub fn extract_register(output: &str, reg_name: &str) -> Option<u64> {
        let registers = Self::parse_registers(output);
        registers.get(reg_name).copied()
    }
}

pub fn gdb_run(binary: &str) -> Result<GdbCrashInfo, String> {
    GdbParser::run_and_parse(binary, None)
}

pub fn gdb_run_with_input(binary: &str, input: &[u8]) -> Result<GdbCrashInfo, String> {
    GdbParser::run_and_parse(binary, Some(input))
}

pub fn gdb_run_args(binary: &str, args: &[&str]) -> Result<GdbCrashInfo, String> {
    GdbParser::quick_run(binary, args)
}

pub fn gdb_get_registers(binary: &str) -> Result<HashMap<String, u64>, String> {
    let crash_info = GdbParser::run_and_parse(binary, None)?;
    Ok(crash_info.registers)
}

pub fn gdb_get_backtrace(binary: &str) -> Result<Vec<GdbFrame>, String> {
    let crash_info = GdbParser::run_and_parse(binary, None)?;
    Ok(crash_info.backtrace)
}

pub fn gdb_auto_offset(binary: &str, pattern_size: usize) -> Result<usize, String> {
    let pattern = crate::cyclic_tools::cyclic(pattern_size);
    GdbParser::get_crash_offset(binary, &pattern)
}
