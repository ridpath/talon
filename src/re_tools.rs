use crate::ast::RECommand;
use capstone::prelude::*;
use pelite::pe64::{Pe, PeFile};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::Command;
#[cfg(feature = "malware-analysis")]
use yara::Compiler;

/// Binary analysis command router
pub fn handle_re_command(cmd: &RECommand) -> Result<(), String> {
    match cmd {
        RECommand::AnalyzePE(path) => analyze_pe(path),
        RECommand::Disassemble(path) => disassemble_binary(path),
        RECommand::ScanStrings(path) => extract_strings(path),
        RECommand::BinaryDiff { file1, file2 } => diff_binaries(file1, file2),
        RECommand::ImportHash { file } => calc_import_hash(file),
        RECommand::YaraMatch { file, rule } => match_yara(file, rule),
        RECommand::EntropyScan { file } => entropy_report(file),
        RECommand::DisassembleDotNet { assembly } => dotnet_disasm_stub(assembly),
        RECommand::BridgeIDA { script, binary } => ida_bridge_stub(script, binary),
        RECommand::ImphashReal { file } => calc_import_hash(file),
        RECommand::DLLInjectTrace { binary } => {
            println!("[RE] DLL injection trace for: {}", binary);
            Ok(())
        }
        RECommand::GhidraBridgeTrace { project } => {
            println!("[RE] Ghidra bridge trace for: {}", project);
            Ok(())
        }
        RECommand::DetectHollowing { binary } => {
            println!("[RE] Detect process hollowing in: {}", binary);
            Ok(())
        }
        RECommand::DetectVM { binary } => {
            println!("[RE] Detect VM in: {}", binary);
            Ok(())
        }
        RECommand::PatternScan { binary, pattern } => {
            println!("[RE] Pattern scan in {}: {}", binary, pattern);
            Ok(())
        }
    }
}

/// [OK] PE parsing + MZ validation
fn analyze_pe(path: &str) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| e.to_string())?;
    if &data[0..2] == b"MZ" {
        println!("[RE] [OK] MZ header valid");
    } else {
        println!("[RE] [ERROR] Not a valid PE file");
        return Ok(());
    }

    if let Ok(pe) = PeFile::from_bytes(&data) {
        println!("[RE] Sections:");
        for s in pe.section_headers() {
            let name = std::str::from_utf8(&s.Name)
                .unwrap_or("?")
                .trim_matches(char::from(0));
            println!("   ▶ {} - {} bytes", name, s.SizeOfRawData);
        }
    }

    Ok(())
}

/// Capstone disassembler
fn disassemble_binary(path: &str) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| e.to_string())?;
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .build()
        .map_err(|e| format!("{:?}", e))?;
    let insns = cs
        .disasm_all(&data, 0x1000)
        .map_err(|e| format!("{:?}", e))?;
    println!("[RE] Capstone disassembly:");
    for i in insns.iter().take(40) {
        println!(
            "   0x{:08x}: {:<10} {}",
            i.address(),
            i.mnemonic().unwrap_or(""),
            i.op_str().unwrap_or("")
        );
    }
    Ok(())
}

/// String extractor (ASCII, UTF-8, Unicode stub)
fn extract_strings(path: &str) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    for &b in &data {
        if b.is_ascii_graphic() || b == b' ' {
            buf.push(b);
        } else {
            if buf.len() >= 4 {
                println!("[RE] String: {}", String::from_utf8_lossy(&buf));
            }
            buf.clear();
        }
    }
    Ok(())
}

/// Byte-wise diff
fn diff_binaries(file1: &str, file2: &str) -> Result<(), String> {
    let d1 = fs::read(file1).map_err(|e| e.to_string())?;
    let d2 = fs::read(file2).map_err(|e| e.to_string())?;
    if d1 == d2 {
        println!("[RE] [OK] Binaries are identical");
        return Ok(());
    }
    let offset = d1.iter().zip(&d2).position(|(a, b)| a != b).unwrap_or(0);
    println!("[RE] [ERROR] Files differ at byte offset {}", offset);
    Ok(())
}

/// Import hash (SHA-based)
fn calc_import_hash(path: &str) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| e.to_string())?;
    let pe = PeFile::from_bytes(&data).map_err(|e| format!("PE parse error: {}", e))?;
    let mut names = vec![];

    if let Ok(imports) = pe.imports() {
        for desc in imports {
            if let Ok(dll_name) = desc.dll_name() {
                let dll = dll_name.to_string().to_lowercase();
                // Use INT (Import Name Table) to get function names
                if let Ok(int) = desc.int() {
                    for imp in int {
                        // Import is an enum with Hint (bytestring with hint + name)
                        // We can just convert the import to string
                        let import_str = format!("{:?}", imp);
                        if !import_str.is_empty() {
                            names.push(format!("{}!{}", dll, import_str.to_lowercase()));
                        }
                    }
                }
            }
        }
    }

    names.sort();
    let joined = names.join(",");
    let hash = Sha256::digest(joined.as_bytes());
    println!("[RE] Imphash-like (SHA256-8): {}", hex::encode(&hash[..8]));
    Ok(())
}

/// YARA in-memory scanner
#[cfg(feature = "malware-analysis")]
fn match_yara(path: &str, rule: &str) -> Result<(), String> {
    let data = fs::read(path)?;
    let mut compiler = Compiler::new().map_err(|e| e.to_string())?;
    let inline = format!("rule r1 {{ condition: {} }}", rule);
    compiler.add_rules_str(&inline)?;
    let rules = compiler.compile_rules()?;
    let scanner = yara::Scanner::new(&rules)?;
    let matches = scanner.scan_mem(&data)?;

    if matches.is_empty() {
        println!("[RE] [ERROR] No match");
    } else {
        println!("[RE] [OK] YARA matched:");
        for m in matches {
            println!("   ▶ {}", m.identifier);
        }
    }
    Ok(())
}

#[cfg(not(feature = "malware-analysis"))]
fn match_yara(_path: &str, _rule: &str) -> Result<(), String> {
    Err("YARA support not compiled in. Rebuild with --features malware-analysis".to_string())
}

/// Shannon entropy
fn entropy_report(file: &str) -> Result<(), String> {
    let data = fs::read(file).map_err(|e| e.to_string())?;
    let e = calculate_entropy(&data);
    println!("[RE] Entropy: {:.4}", e);
    if e > 7.5 {
        println!("    High entropy (packed/encrypted?)");
    } else if e < 3.0 {
        println!("    Likely plaintext or low-compression");
    } else {
        println!("    Possibly partially obfuscated");
    }
    Ok(())
}

fn calculate_entropy(data: &[u8]) -> f64 {
    let mut freq = [0usize; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let len = data.len() as f64;
    freq.iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum()
}

/// .NET disassembly stub
fn dotnet_disasm_stub(path: &str) -> Result<(), String> {
    println!("[RE] .NET Disassembly (stub): {}", path);
    println!("     Try: `ilspycmd -p {}`", path);
    Ok(())
}

/// IDA remote script stub
fn ida_bridge_stub(script: &str, binary: &str) -> Result<(), String> {
    println!("[RE] IDA Bridge Stub");
    println!("     idat -A -S{} {}", script, binary);
    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════
// ULTIMATE ENHANCEMENTS - DEBUGGER INTEGRATION
// ════════════════════════════════════════════════════════════════════════════

pub struct GDBSession {
    stream: TcpStream,
    breakpoints: Vec<u64>,
}

impl GDBSession {
    pub fn connect(host: &str, port: u16) -> Result<Self, String> {
        let addr = format!("{}:{}", host, port);
        let stream =
            TcpStream::connect(&addr).map_err(|e| format!("GDB connection failed: {}", e))?;

        println!("[GDB] Connected to {}:{}", host, port);
        Ok(GDBSession {
            stream,
            breakpoints: Vec::new(),
        })
    }

    pub fn send_command(&mut self, cmd: &str) -> Result<String, String> {
        let checksum = cmd.bytes().fold(0u8, |acc, b| acc.wrapping_add(b));
        let packet = format!("${}#{:02x}", cmd, checksum);

        self.stream
            .write_all(packet.as_bytes())
            .map_err(|e| format!("GDB send failed: {}", e))?;

        let mut buffer = vec![0u8; 4096];
        let n = self
            .stream
            .read(&mut buffer)
            .map_err(|e| format!("GDB recv failed: {}", e))?;

        Ok(String::from_utf8_lossy(&buffer[..n]).to_string())
    }

    pub fn set_breakpoint(&mut self, addr: u64) -> Result<(), String> {
        let cmd = format!("Z0,{:x},1", addr);
        let response = self.send_command(&cmd)?;

        if response.contains("OK") {
            self.breakpoints.push(addr);
            println!("[GDB] Breakpoint set at 0x{:x}", addr);
            Ok(())
        } else {
            Err(format!("Failed to set breakpoint: {}", response))
        }
    }

    pub fn read_registers(&mut self) -> Result<HashMap<String, u64>, String> {
        let response = self.send_command("g")?;
        let mut regs = HashMap::new();

        let reg_names = vec![
            "rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rbp", "rsp", "rip",
        ];
        let hex_data = response
            .trim_start_matches('$')
            .split('#')
            .next()
            .unwrap_or("");

        for (i, name) in reg_names.iter().enumerate() {
            let offset = i * 16;
            if offset + 16 <= hex_data.len() {
                if let Ok(val) = u64::from_str_radix(&hex_data[offset..offset + 16], 16) {
                    regs.insert(name.to_string(), val);
                }
            }
        }

        Ok(regs)
    }

    pub fn read_memory(&mut self, addr: u64, len: usize) -> Result<Vec<u8>, String> {
        let cmd = format!("m{:x},{:x}", addr, len);
        let response = self.send_command(&cmd)?;

        let hex_data = response
            .trim_start_matches('$')
            .split('#')
            .next()
            .unwrap_or("");
        hex::decode(hex_data).map_err(|e| format!("Memory decode failed: {}", e))
    }

    pub fn continue_execution(&mut self) -> Result<(), String> {
        self.send_command("c")?;
        println!("[GDB] Continuing execution");
        Ok(())
    }

    pub fn single_step(&mut self) -> Result<(), String> {
        self.send_command("s")?;
        println!("[GDB] Single step");
        Ok(())
    }
}

pub struct WinDbgSession {
    stream: TcpStream,
}

impl WinDbgSession {
    pub fn connect(host: &str, port: u16) -> Result<Self, String> {
        let addr = format!("{}:{}", host, port);
        let stream =
            TcpStream::connect(&addr).map_err(|e| format!("WinDbg connection failed: {}", e))?;

        println!("[WINDBG] Connected to {}:{}", host, port);
        Ok(WinDbgSession { stream })
    }

    pub fn execute(&mut self, cmd: &str) -> Result<String, String> {
        let command = format!("{}\n", cmd);
        self.stream
            .write_all(command.as_bytes())
            .map_err(|e| format!("WinDbg send failed: {}", e))?;

        let mut reader = BufReader::new(&self.stream);
        let mut response = String::new();
        reader
            .read_line(&mut response)
            .map_err(|e| format!("WinDbg recv failed: {}", e))?;

        Ok(response)
    }

    pub fn read_memory(&mut self, addr: u64, len: usize) -> Result<Vec<u8>, String> {
        let cmd = format!("db {:x} L{:x}", addr, len);
        let response = self.execute(&cmd)?;

        let mut bytes = Vec::new();
        for line in response.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for part in parts.iter().skip(1) {
                if let Ok(byte) = u8::from_str_radix(part, 16) {
                    bytes.push(byte);
                }
            }
        }

        Ok(bytes)
    }

    pub fn set_breakpoint(&mut self, addr: u64) -> Result<(), String> {
        let cmd = format!("bp {:x}", addr);
        let response = self.execute(&cmd)?;

        if !response.contains("error") {
            println!("[WINDBG] Breakpoint set at 0x{:x}", addr);
            Ok(())
        } else {
            Err(format!("Failed to set breakpoint: {}", response))
        }
    }

    pub fn get_modules(&mut self) -> Result<Vec<String>, String> {
        let response = self.execute("lm")?;
        let modules: Vec<String> = response
            .lines()
            .filter(|l| !l.is_empty())
            .map(|s| s.to_string())
            .collect();

        Ok(modules)
    }
}

// ════════════════════════════════════════════════════════════════════════════
// PDB PARSING & SYMBOL RESOLUTION
// ════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub address: u64,
    pub size: u32,
    pub symbol_type: SymbolType,
}

#[derive(Debug, Clone)]
pub enum SymbolType {
    Function,
    Data,
    Label,
    Unknown,
}

pub struct PDBParser {
    symbols: HashMap<String, Symbol>,
}

impl PDBParser {
    pub fn new() -> Self {
        PDBParser {
            symbols: HashMap::new(),
        }
    }

    pub fn parse_pdb_cli(&mut self, pdb_path: &str) -> Result<(), String> {
        let output = Command::new("llvm-pdbutil")
            .args(&["dump", "-symbols", pdb_path])
            .output()
            .map_err(|e| format!("Failed to run llvm-pdbutil: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.contains("S_GPROC32") || line.contains("S_LPROC32") {
                if let Some(symbol) = self.parse_symbol_line(line) {
                    self.symbols.insert(symbol.name.clone(), symbol);
                }
            }
        }

        println!(
            "[PDB] Parsed {} symbols from {}",
            self.symbols.len(),
            pdb_path
        );
        Ok(())
    }

    fn parse_symbol_line(&self, line: &str) -> Option<Symbol> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return None;
        }

        let name = parts.last()?.to_string();
        let addr_str = parts
            .iter()
            .find(|s| s.starts_with("0x"))?
            .trim_start_matches("0x");
        let address = u64::from_str_radix(addr_str, 16).ok()?;

        Some(Symbol {
            name,
            address,
            size: 0,
            symbol_type: SymbolType::Function,
        })
    }

    pub fn resolve_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn resolve_address(&self, addr: u64) -> Option<&Symbol> {
        self.symbols.values().find(|s| s.address == addr)
    }

    pub fn list_functions(&self) -> Vec<&Symbol> {
        self.symbols
            .values()
            .filter(|s| matches!(s.symbol_type, SymbolType::Function))
            .collect()
    }
}

pub fn parse_dwarf_symbols(binary_path: &str) -> Result<HashMap<String, u64>, String> {
    let output = Command::new("nm")
        .arg("-C")
        .arg(binary_path)
        .output()
        .map_err(|e| format!("Failed to run nm: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut symbols = HashMap::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            if let Ok(addr) = u64::from_str_radix(parts[0], 16) {
                let name = parts[2..].join(" ");
                symbols.insert(name, addr);
            }
        }
    }

    println!("[DWARF] Parsed {} symbols", symbols.len());
    Ok(symbols)
}

// ════════════════════════════════════════════════════════════════════════════
// CONTROL FLOW GRAPH GENERATION
// ════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub start_addr: u64,
    pub end_addr: u64,
    pub instructions: Vec<String>,
    pub successors: Vec<u64>,
}

#[derive(Debug)]
pub struct ControlFlowGraph {
    pub blocks: HashMap<u64, BasicBlock>,
    pub entry_point: u64,
}

impl ControlFlowGraph {
    pub fn new(entry_point: u64) -> Self {
        ControlFlowGraph {
            blocks: HashMap::new(),
            entry_point,
        }
    }

    pub fn analyze_binary(&mut self, data: &[u8], base_addr: u64) -> Result<(), String> {
        let cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .detail(true)
            .build()
            .map_err(|e| format!("Capstone error: {}", e))?;

        let insns = cs
            .disasm_all(data, base_addr)
            .map_err(|e| format!("Disassembly failed: {}", e))?;

        let mut current_block = BasicBlock {
            start_addr: base_addr,
            end_addr: base_addr,
            instructions: Vec::new(),
            successors: Vec::new(),
        };

        for insn in insns.iter() {
            let mnemonic = insn.mnemonic().unwrap_or("");
            let op_str = insn.op_str().unwrap_or("");
            let inst_str = format!("{} {}", mnemonic, op_str);

            current_block.instructions.push(inst_str.clone());
            current_block.end_addr = insn.address();

            if is_branch_instruction(mnemonic) {
                if let Some(target) = extract_branch_target(&inst_str) {
                    current_block.successors.push(target);
                }

                if !is_unconditional_jump(mnemonic) {
                    current_block
                        .successors
                        .push(insn.address() + insn.bytes().len() as u64);
                }

                self.blocks
                    .insert(current_block.start_addr, current_block.clone());

                current_block = BasicBlock {
                    start_addr: insn.address() + insn.bytes().len() as u64,
                    end_addr: insn.address() + insn.bytes().len() as u64,
                    instructions: Vec::new(),
                    successors: Vec::new(),
                };
            } else if mnemonic == "ret" || mnemonic == "hlt" {
                self.blocks
                    .insert(current_block.start_addr, current_block.clone());

                current_block = BasicBlock {
                    start_addr: insn.address() + insn.bytes().len() as u64,
                    end_addr: insn.address() + insn.bytes().len() as u64,
                    instructions: Vec::new(),
                    successors: Vec::new(),
                };
            }
        }

        if !current_block.instructions.is_empty() {
            self.blocks.insert(current_block.start_addr, current_block);
        }

        println!("[CFG] Generated {} basic blocks", self.blocks.len());
        Ok(())
    }

    pub fn export_dot(&self) -> String {
        let mut dot = String::from("digraph CFG {\n");
        dot.push_str("  node [shape=box];\n");

        for (addr, block) in &self.blocks {
            let label = format!("0x{:x}\\n{} instructions", addr, block.instructions.len());
            dot.push_str(&format!("  \"0x{:x}\" [label=\"{}\"];\n", addr, label));

            for successor in &block.successors {
                dot.push_str(&format!("  \"0x{:x}\" -> \"0x{:x}\";\n", addr, successor));
            }
        }

        dot.push_str("}\n");
        dot
    }

    pub fn find_loops(&self) -> Vec<(u64, u64)> {
        let mut loops = Vec::new();
        let mut visited = HashSet::new();

        for (addr, block) in &self.blocks {
            for successor in &block.successors {
                if successor <= addr && !visited.contains(&(*addr, *successor)) {
                    loops.push((*addr, *successor));
                    visited.insert((*addr, *successor));
                }
            }
        }

        loops
    }
}

fn is_branch_instruction(mnemonic: &str) -> bool {
    matches!(
        mnemonic,
        "je" | "jne" | "jg" | "jl" | "jge" | "jle" | "ja" | "jb" | "jae" | "jbe" | "jmp" | "call"
    )
}

fn is_unconditional_jump(mnemonic: &str) -> bool {
    matches!(mnemonic, "jmp" | "ret")
}

fn extract_branch_target(inst: &str) -> Option<u64> {
    let parts: Vec<&str> = inst.split_whitespace().collect();
    if parts.len() >= 2 {
        let target_str = parts[1].trim_start_matches("0x");
        u64::from_str_radix(target_str, 16).ok()
    } else {
        None
    }
}

// ════════════════════════════════════════════════════════════════════════════
// DECOMPILER INTEGRATION
// ════════════════════════════════════════════════════════════════════════════

pub struct GhidraDecompiler {
    project_path: String,
}

impl GhidraDecompiler {
    pub fn new(project_path: &str) -> Self {
        GhidraDecompiler {
            project_path: project_path.to_string(),
        }
    }

    pub fn decompile_function(&self, binary: &str, function_addr: u64) -> Result<String, String> {
        let script = format!(
            r#"
import ghidra.app.decompiler.DecompInterface
import ghidra.program.model.address.AddressSet

def decompile_at(addr):
    decompiler = DecompInterface()
    decompiler.openProgram(currentProgram)
    
    func = getFunctionAt(toAddr(addr))
    if func:
        result = decompiler.decompileFunction(func, 30, None)
        return result.getDecompiledFunction().getC()
    return None

print(decompile_at(0x{:x}))
"#,
            function_addr
        );

        let script_path = "/tmp/ghidra_decompile.py";
        fs::write(script_path, script).map_err(|e| format!("Failed to write script: {}", e))?;

        let output = Command::new("analyzeHeadless")
            .args(&[
                &self.project_path,
                "TempProject",
                "-import",
                binary,
                "-postScript",
                script_path,
            ])
            .output()
            .map_err(|e| format!("Ghidra execution failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.contains("undefined") || line.contains("void") {
                return Ok(line.to_string());
            }
        }

        Ok(stdout.to_string())
    }

    pub fn analyze_binary(&self, binary: &str) -> Result<(), String> {
        println!(
            "[GHIDRA] Analyzing {} in project {}",
            binary, self.project_path
        );

        let output = Command::new("analyzeHeadless")
            .args(&[
                &self.project_path,
                "Analysis",
                "-import",
                binary,
                "-analyze",
            ])
            .output()
            .map_err(|e| format!("Ghidra analysis failed: {}", e))?;

        if output.status.success() {
            println!("[GHIDRA] Analysis complete");
            Ok(())
        } else {
            Err(format!(
                "Analysis failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}

pub struct BinaryNinjaDecompiler {
    api_key: Option<String>,
}

impl BinaryNinjaDecompiler {
    pub fn new(api_key: Option<String>) -> Self {
        BinaryNinjaDecompiler { api_key }
    }

    pub fn decompile_function(&self, binary: &str, function_addr: u64) -> Result<String, String> {
        let script = format!(
            r#"
import binaryninja as bn

bv = bn.BinaryViewType.get_view_of_file("{}")
if not bv:
    print("Failed to open binary")
    exit(1)

func = bv.get_function_at(0x{:x})
if func:
    for line in func.hlil:
        print(line)
else:
    print("Function not found at 0x{:x}")
"#,
            binary, function_addr, function_addr
        );

        let script_path = "/tmp/bn_decompile.py";
        fs::write(script_path, script).map_err(|e| format!("Failed to write script: {}", e))?;

        let output = Command::new("python3")
            .arg(script_path)
            .output()
            .map_err(|e| format!("Binary Ninja execution failed: {}", e))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn analyze_binary(&self, binary: &str) -> Result<Vec<String>, String> {
        println!("[BINJA] Analyzing {}", binary);

        let script = format!(
            r#"
import binaryninja as bn

bv = bn.BinaryViewType.get_view_of_file("{}")
if bv:
    for func in bv.functions:
        print(f"{{func.name}} @ 0x{{func.start:x}}")
"#,
            binary
        );

        let script_path = "/tmp/bn_analyze.py";
        fs::write(script_path, script).map_err(|e| format!("Failed to write script: {}", e))?;

        let output = Command::new("python3")
            .arg(script_path)
            .output()
            .map_err(|e| format!("Binary Ninja execution failed: {}", e))?;

        let functions: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();

        println!("[BINJA] Found {} functions", functions.len());
        Ok(functions)
    }
}
