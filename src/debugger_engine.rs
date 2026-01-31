#![allow(clippy::upper_case_acronyms)]

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub id: usize,
    pub location: BreakpointLocation,
    pub condition: Option<String>,
    pub hit_count: usize,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum BreakpointLocation {
    Address(u64),
    Function(String),
    Line(String, usize),
}

#[derive(Debug, Clone)]
pub struct DebuggerState {
    pub breakpoints: HashMap<usize, Breakpoint>,
    pub variables: HashMap<String, String>,
    pub registers: HashMap<String, u64>,
    pub stack_frames: Vec<StackFrame>,
    pub current_line: Option<(String, usize)>,
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: String,
    pub file: String,
    pub line: usize,
    pub locals: HashMap<String, String>,
}

pub struct DebuggerEngine {
    state: Arc<Mutex<DebuggerState>>,
    backend: DebuggerBackend,
    next_breakpoint_id: usize,
}

enum DebuggerBackend {
    GDB { process: std::process::Child },
    LLDB { process: std::process::Child },
    WinDbg { process: std::process::Child },
    Native,
}

impl DebuggerEngine {
    pub fn new(backend_type: &str) -> Result<Self, String> {
        let backend = match backend_type {
            "gdb" => {
                let process = Command::new("gdb")
                    .arg("--interpreter=mi")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to start GDB: {}", e))?;

                DebuggerBackend::GDB { process }
            }
            "lldb" => {
                let process = Command::new("lldb")
                    .arg("--source-quietly")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to start LLDB: {}", e))?;

                DebuggerBackend::LLDB { process }
            }
            "native" => DebuggerBackend::Native,
            _ => return Err(format!("Unsupported debugger backend: {}", backend_type)),
        };

        Ok(DebuggerEngine {
            state: Arc::new(Mutex::new(DebuggerState {
                breakpoints: HashMap::new(),
                variables: HashMap::new(),
                registers: HashMap::new(),
                stack_frames: Vec::new(),
                current_line: None,
            })),
            backend,
            next_breakpoint_id: 1,
        })
    }

    pub fn add_breakpoint(&mut self, location: BreakpointLocation) -> Result<usize, String> {
        let id = self.next_breakpoint_id;
        self.next_breakpoint_id += 1;

        let breakpoint = Breakpoint {
            id,
            location: location.clone(),
            condition: None,
            hit_count: 0,
            enabled: true,
        };

        match &location {
            BreakpointLocation::Address(addr) => {
                self.send_command(&format!("break *0x{:x}", addr))?;
            }
            BreakpointLocation::Function(name) => {
                self.send_command(&format!("break {}", name))?;
            }
            BreakpointLocation::Line(file, line) => {
                self.send_command(&format!("break {}:{}", file, line))?;
            }
        }

        let mut state = self.state.lock().unwrap();
        state.breakpoints.insert(id, breakpoint);

        Ok(id)
    }

    pub fn remove_breakpoint(&mut self, id: usize) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();

        if state.breakpoints.remove(&id).is_some() {
            self.send_command(&format!("delete {}", id))?;
            Ok(())
        } else {
            Err(format!("Breakpoint {} not found", id))
        }
    }

    pub fn step_over(&mut self) -> Result<(), String> {
        self.send_command("next")?;
        self.update_state()?;
        Ok(())
    }

    pub fn step_into(&mut self) -> Result<(), String> {
        self.send_command("step")?;
        self.update_state()?;
        Ok(())
    }

    pub fn step_out(&mut self) -> Result<(), String> {
        self.send_command("finish")?;
        self.update_state()?;
        Ok(())
    }

    pub fn continue_execution(&mut self) -> Result<(), String> {
        self.send_command("continue")?;
        self.update_state()?;
        Ok(())
    }

    pub fn get_variable(&self, name: &str) -> Result<String, String> {
        let state = self.state.lock().unwrap();
        state
            .variables
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Variable {} not found", name))
    }

    pub fn set_variable(&mut self, name: &str, value: &str) -> Result<(), String> {
        self.send_command(&format!("set variable {} = {}", name, value))?;
        self.update_state()?;
        Ok(())
    }

    pub fn get_registers(&self) -> HashMap<String, u64> {
        let state = self.state.lock().unwrap();
        state.registers.clone()
    }

    pub fn get_stack_trace(&self) -> Vec<StackFrame> {
        let state = self.state.lock().unwrap();
        state.stack_frames.clone()
    }

    pub fn evaluate_expression(&mut self, expr: &str) -> Result<String, String> {
        self.send_command(&format!("print {}", expr))?;
        Ok("Expression evaluation result".to_string())
    }

    fn send_command(&self, cmd: &str) -> Result<String, String> {
        match &self.backend {
            DebuggerBackend::GDB { .. } | DebuggerBackend::LLDB { .. } => {
                Ok(format!("Executed: {}", cmd))
            }
            DebuggerBackend::Native => Ok("Native debugger command executed".to_string()),
            _ => Err("Unsupported debugger backend".to_string()),
        }
    }

    fn update_state(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();

        state.registers.insert("rip".to_string(), 0x400000);
        state.registers.insert("rsp".to_string(), 0x7fffffffe000);
        state.registers.insert("rbp".to_string(), 0x7fffffffe010);

        Ok(())
    }

    pub fn get_state(&self) -> DebuggerState {
        self.state.lock().unwrap().clone()
    }
}

impl Drop for DebuggerEngine {
    fn drop(&mut self) {
        match &mut self.backend {
            DebuggerBackend::GDB { process }
            | DebuggerBackend::LLDB { process }
            | DebuggerBackend::WinDbg { process } => {
                let _ = process.kill();
            }
            _ => {}
        }
    }
}

pub fn attach_to_process(pid: u32, backend: &str) -> Result<DebuggerEngine, String> {
    let engine = DebuggerEngine::new(backend)?;
    engine.send_command(&format!("attach {}", pid))?;
    Ok(engine)
}

pub fn launch_and_debug(
    program: &str,
    args: &[&str],
    backend: &str,
) -> Result<DebuggerEngine, String> {
    let engine = DebuggerEngine::new(backend)?;

    let _full_command = format!("{} {}", program, args.join(" "));
    engine.send_command(&format!("file {}", program))?;
    engine.send_command(&format!("run {}", args.join(" ")))?;

    Ok(engine)
}
