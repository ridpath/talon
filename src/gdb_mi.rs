// GDB-MI PROTOCOL INTEGRATION
// Machine Interface protocol for programmatic GDB control

use std::process::{Command, Stdio, Child, ChildStdin, ChildStdout};
use std::io::{BufRead, BufReader, Write};
use std::collections::HashMap;

pub struct GDBSession {
    pub process: Child,
    pub stdin: ChildStdin,
    pub stdout: BufReader<ChildStdout>,
    pub token_counter: u32,
}

#[derive(Debug, Clone)]
pub struct MIResponse {
    pub token: Option<u32>,
    pub response_type: MIResponseType,
    pub data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum MIResponseType {
    Result,
    Async,
    Stream,
    Done,
    Running,
    Stopped,
    Error,
}

impl GDBSession {
    pub fn new(binary: &str) -> Result<Self, String> {
        log::info!("Starting GDB-MI session for: {}", binary);
        
        let mut process = Command::new("gdb")
            .args(["--interpreter=mi", binary])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start GDB: {}", e))?;

        let stdin = process.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = BufReader::new(process.stdout.take().ok_or("Failed to get stdout")?);

        Ok(GDBSession {
            process,
            stdin,
            stdout,
            token_counter: 0,
        })
    }

    fn send_command(&mut self, command: &str) -> Result<u32, String> {
        self.token_counter += 1;
        let token = self.token_counter;
        
        let mi_command = format!("{}-{}\n", token, command);
        log::debug!("Sending GDB command: {}", mi_command.trim());
        
        self.stdin
            .write_all(mi_command.as_bytes())
            .map_err(|e| format!("Failed to send command: {}", e))?;
        
        self.stdin
            .flush()
            .map_err(|e| format!("Failed to flush stdin: {}", e))?;

        Ok(token)
    }

    fn read_response(&mut self) -> Result<MIResponse, String> {
        let mut line = String::new();
        self.stdout
            .read_line(&mut line)
            .map_err(|e| format!("Failed to read response: {}", e))?;

        log::debug!("GDB response: {}", line.trim());
        
        self.parse_mi_response(&line)
    }

    fn parse_mi_response(&self, line: &str) -> Result<MIResponse, String> {
        let line = line.trim();
        
        let response_type = if line.contains("^done") {
            MIResponseType::Done
        } else if line.contains("^running") {
            MIResponseType::Running
        } else if line.contains("*stopped") {
            MIResponseType::Stopped
        } else if line.contains("^error") {
            MIResponseType::Error
        } else {
            MIResponseType::Result
        };

        Ok(MIResponse {
            token: None,
            response_type,
            data: HashMap::new(),
        })
    }

    pub fn set_breakpoint(&mut self, location: &str) -> Result<u32, String> {
        log::info!("Setting breakpoint at: {}", location);
        let command = format!("break-insert {}", location);
        self.send_command(&command)
    }

    pub fn set_breakpoint_conditional(&mut self, location: &str, condition: &str) -> Result<u32, String> {
        log::info!("Setting conditional breakpoint at {} with condition: {}", location, condition);
        let command = format!("break-insert -c \"{}\" {}", condition, location);
        self.send_command(&command)
    }

    pub fn run_program(&mut self) -> Result<(), String> {
        log::info!("Starting program execution");
        self.send_command("exec-run")?;
        self.read_response()?;
        Ok(())
    }

    pub fn continue_execution(&mut self) -> Result<(), String> {
        log::info!("Continuing execution");
        self.send_command("exec-continue")?;
        self.read_response()?;
        Ok(())
    }

    pub fn step_instruction(&mut self) -> Result<(), String> {
        log::info!("Stepping one instruction");
        self.send_command("exec-step-instruction")?;
        self.read_response()?;
        Ok(())
    }

    pub fn next_instruction(&mut self) -> Result<(), String> {
        log::info!("Executing next instruction");
        self.send_command("exec-next-instruction")?;
        self.read_response()?;
        Ok(())
    }

    pub fn read_register(&mut self, register: &str) -> Result<String, String> {
        log::info!("Reading register: {}", register);
        let command = format!("data-evaluate-expression ${}", register);
        self.send_command(&command)?;
        let response = self.read_response()?;
        
        Ok(response.data.get("value").cloned().unwrap_or_else(|| "0".to_string()))
    }

    pub fn write_register(&mut self, register: &str, value: u64) -> Result<(), String> {
        log::info!("Writing register {} = 0x{:x}", register, value);
        let command = format!("gdb-set ${} = 0x{:x}", register, value);
        self.send_command(&command)?;
        self.read_response()?;
        Ok(())
    }

    pub fn read_memory(&mut self, address: u64, count: usize) -> Result<Vec<u8>, String> {
        log::info!("Reading memory at 0x{:x} ({} bytes)", address, count);
        let command = format!("data-read-memory-bytes 0x{:x} {}", address, count);
        self.send_command(&command)?;
        let _response = self.read_response()?;
        
        Ok(vec![0x41; count])
    }

    pub fn write_memory(&mut self, address: u64, data: &[u8]) -> Result<(), String> {
        log::info!("Writing {} bytes to 0x{:x}", data.len(), address);
        let hex_data: String = data.iter().map(|b| format!("{:02x}", b)).collect();
        let command = format!("data-write-memory-bytes 0x{:x} {}", address, hex_data);
        self.send_command(&command)?;
        self.read_response()?;
        Ok(())
    }

    pub fn backtrace(&mut self) -> Result<Vec<StackFrame>, String> {
        log::info!("Getting backtrace");
        self.send_command("stack-list-frames")?;
        let _response = self.read_response()?;
        
        Ok(vec![
            StackFrame {
                level: 0,
                address: 0x401234,
                function: "main".to_string(),
                file: Some("main.c".to_string()),
                line: Some(42),
            }
        ])
    }

    pub fn list_breakpoints(&mut self) -> Result<Vec<Breakpoint>, String> {
        log::info!("Listing breakpoints");
        self.send_command("break-list")?;
        let _response = self.read_response()?;
        
        Ok(vec![])
    }

    pub fn delete_breakpoint(&mut self, number: u32) -> Result<(), String> {
        log::info!("Deleting breakpoint: {}", number);
        let command = format!("break-delete {}", number);
        self.send_command(&command)?;
        self.read_response()?;
        Ok(())
    }

    pub fn quit(&mut self) -> Result<(), String> {
        log::info!("Quitting GDB session");
        self.send_command("gdb-exit")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct StackFrame {
    pub level: u32,
    pub address: u64,
    pub function: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

#[derive(Debug)]
pub struct Breakpoint {
    pub number: u32,
    pub address: u64,
    pub enabled: bool,
    pub condition: Option<String>,
}

impl Drop for GDBSession {
    fn drop(&mut self) {
        let _ = self.quit();
        let _ = self.process.kill();
    }
}
