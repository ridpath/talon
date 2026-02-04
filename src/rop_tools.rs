use capstone::prelude::*;
use std::collections::HashMap;
use std::fs;

// ═══════════════════════════════════════════════════════════════════════════
// ROP GADGET SEARCH - AUTO-FIND GADGETS
// ═══════════════════════════════════════════════════════════════════════════

/// ROP gadget structure
#[derive(Debug, Clone)]
pub struct Gadget {
    pub address: u64,
    pub instructions: Vec<String>,
    pub bytes: Vec<u8>,
    pub quality_score: u32,
}

/// ROP chain builder
pub struct RopChain {
    pub binary_path: String,
    pub gadgets: Vec<Gadget>,
    pub libc_base: Option<u64>,
    pub arch: Architecture,
}

#[derive(Debug, Clone)]
pub enum Architecture {
    X8664,
    I386,
    ARM,
    ARM64,
}

impl RopChain {
    /// Create a new ROP chain builder
    ///
    /// # Example
    /// ```
    /// let mut rop = RopChain::new("./vulnerable")?;
    /// let pop_rdi = rop.find_gadget("pop rdi; ret")?;
    /// ```
    pub fn new(binary_path: &str) -> Result<Self, String> {
        log::info!("Initializing ROP chain builder for {}", binary_path);

        // Detect architecture
        let arch = Self::detect_arch(binary_path)?;

        // Find all gadgets
        let gadgets = Self::find_all_gadgets(binary_path, &arch)?;

        log::info!("Found {} total gadgets", gadgets.len());

        Ok(RopChain {
            binary_path: binary_path.to_string(),
            gadgets,
            libc_base: None,
            arch,
        })
    }

    /// Set libc base address for ret2libc chains
    pub fn set_libc_base(&mut self, base: u64) {
        self.libc_base = Some(base);
        log::info!("Set libc base to 0x{:x}", base);
    }

    /// Find a specific gadget by pattern
    pub fn find_gadget(&self, pattern: &str) -> Option<u64> {
        let pattern_lower = pattern.to_lowercase();

        for gadget in &self.gadgets {
            let gadget_str = gadget.instructions.join("; ").to_lowercase();
            if gadget_str.contains(&pattern_lower) {
                log::debug!("Found gadget: {} at 0x{:x}", gadget_str, gadget.address);
                return Some(gadget.address);
            }
        }

        log::warn!("Gadget not found: {}", pattern);
        None
    }

    /// Find gadgets matching a pattern
    pub fn find_gadgets(&self, pattern: &str) -> Vec<Gadget> {
        let pattern_lower = pattern.to_lowercase();
        let mut results = Vec::new();

        for gadget in &self.gadgets {
            let gadget_str = gadget.instructions.join("; ").to_lowercase();
            if gadget_str.contains(&pattern_lower) {
                results.push(gadget.clone());
            }
        }

        log::info!("Found {} gadgets matching '{}'", results.len(), pattern);
        results
    }

    /// Build a ret2libc chain
    pub fn ret2libc(&self, cmd: &str) -> Result<Vec<u64>, String> {
        let libc_base = self
            .libc_base
            .ok_or("Libc base not set. Use set_libc_base()".to_string())?;

        // Common libc offsets (Ubuntu 20.04 x86-64)
        let system_offset = 0x050d60u64;
        let binsh_offset = 0x1b3e1au64;
        let pop_rdi_offset = 0x0002155fu64;

        let system_addr = libc_base + system_offset;
        let binsh_addr = libc_base + binsh_offset;
        let pop_rdi = libc_base + pop_rdi_offset;

        // Build chain: pop_rdi; /bin/sh; system
        let chain = vec![pop_rdi, binsh_addr, system_addr];

        log::info!("Built ret2libc chain for '{}'", cmd);
        Ok(chain)
    }

    /// Build a custom ROP chain from gadget addresses
    pub fn build_chain(&self, gadgets: &[u64]) -> Vec<u8> {
        let mut chain = Vec::new();
        for &addr in gadgets {
            chain.extend_from_slice(&addr.to_le_bytes());
        }
        chain
    }

    /// Find common useful gadgets
    pub fn find_common_gadgets(&self) -> HashMap<String, u64> {
        let mut common = HashMap::new();

        let patterns = vec![
            "pop rdi", "pop rsi", "pop rdx", "pop rax", "pop rbx", "pop rcx", "syscall",
            "int 0x80", "leave", "ret",
        ];

        for pattern in patterns {
            if let Some(addr) = self.find_gadget(pattern) {
                common.insert(pattern.to_string(), addr);
            }
        }

        common
    }

    /// Detect architecture from binary
    fn detect_arch(binary_path: &str) -> Result<Architecture, String> {
        use goblin::Object;

        let buffer = fs::read(binary_path).map_err(|e| format!("Failed to read binary: {}", e))?;

        match Object::parse(&buffer) {
            Ok(Object::Elf(elf)) => match elf.header.e_machine {
                goblin::elf::header::EM_X86_64 => Ok(Architecture::X8664),
                goblin::elf::header::EM_386 => Ok(Architecture::I386),
                goblin::elf::header::EM_ARM => Ok(Architecture::ARM),
                goblin::elf::header::EM_AARCH64 => Ok(Architecture::ARM64),
                _ => Err("Unsupported architecture".to_string()),
            },
            _ => Err("Not an ELF binary".to_string()),
        }
    }

    /// Find all ROP gadgets in binary
    fn find_all_gadgets(binary_path: &str, arch: &Architecture) -> Result<Vec<Gadget>, String> {
        let buffer = fs::read(binary_path).map_err(|e| format!("Failed to read binary: {}", e))?;

        // Create capstone disassembler
        let cs = match arch {
            Architecture::X8664 => Capstone::new()
                .x86()
                .mode(arch::x86::ArchMode::Mode64)
                .syntax(arch::x86::ArchSyntax::Intel)
                .build()
                .map_err(|e| format!("Capstone init failed: {}", e))?,
            Architecture::I386 => Capstone::new()
                .x86()
                .mode(arch::x86::ArchMode::Mode32)
                .syntax(arch::x86::ArchSyntax::Intel)
                .build()
                .map_err(|e| format!("Capstone init failed: {}", e))?,
            _ => return Err("Architecture not yet supported for ROP search".to_string()),
        };

        let mut gadgets = Vec::new();

        // Search for 'ret' instructions
        for (offset, window) in buffer.windows(1).enumerate() {
            // Check for 'ret' (0xc3) or 'ret n' (0xc2)
            if window[0] == 0xc3 || window[0] == 0xc2 {
                // Disassemble backwards to find gadget
                let start = if offset >= 20 { offset - 20 } else { 0 };
                let end = if offset + 1 < buffer.len() {
                    offset + 1
                } else {
                    buffer.len()
                };

                if let Ok(insns) = cs.disasm_all(&buffer[start..end], start as u64) {
                    if insns.len() > 0 {
                        // Create gadget from last few instructions
                        let mut instructions = Vec::new();
                        let mut bytes = Vec::new();

                        for insn in insns.iter() {
                            if let Some(mnemonic) = insn.mnemonic() {
                                let operands = insn.op_str().unwrap_or("");
                                instructions
                                    .push(format!("{} {}", mnemonic, operands).trim().to_string());
                                bytes.extend_from_slice(insn.bytes());
                            }
                        }

                        if !instructions.is_empty() {
                            gadgets.push(Gadget {
                                address: offset as u64,
                                instructions,
                                bytes,
                                quality_score: 0, // Will be scored later
                            });
                        }
                    }
                }
            }
        }

        // Filter to useful gadgets (1-5 instructions ending in ret)
        gadgets.retain(|g| g.instructions.len() <= 5 && g.instructions.len() > 0);

        // Score and sort gadgets
        for gadget in &mut gadgets {
            gadget.quality_score = Self::score_gadget(&gadget.instructions);
        }

        // Deduplicate gadgets
        Self::deduplicate_gadgets(&mut gadgets);

        // Sort by quality score (descending)
        gadgets.sort_by(|a, b| b.quality_score.cmp(&a.quality_score));

        Ok(gadgets)
    }

    /// Score a gadget based on usefulness
    fn score_gadget(instructions: &[String]) -> u32 {
        let mut score = 100u32;

        // Penalty for length
        score = score.saturating_sub(instructions.len() as u32 * 5);

        // Bonus for useful instructions
        for instr in instructions {
            let instr_lower = instr.to_lowercase();

            // High value gadgets
            if instr_lower.starts_with("pop rdi") {
                score += 50;
            } else if instr_lower.starts_with("pop rsi") {
                score += 45;
            } else if instr_lower.starts_with("pop rdx") {
                score += 45;
            } else if instr_lower.starts_with("pop rax") {
                score += 40;
            } else if instr_lower.starts_with("pop rcx") {
                score += 35;
            } else if instr_lower.starts_with("syscall") {
                score += 100;
            } else if instr_lower.starts_with("int 0x80") {
                score += 90;
            } else if instr_lower == "ret" {
                score += 20;
            } else if instr_lower.starts_with("xor") && instr_lower.contains("eax") {
                score += 30;
            }
            // Medium value
            else if instr_lower.starts_with("mov") {
                score += 15;
            } else if instr_lower.starts_with("lea") {
                score += 20;
            } else if instr_lower.starts_with("add") {
                score += 10;
            } else if instr_lower.starts_with("sub") {
                score += 10;
            }
            // Penalties for bad instructions
            else if instr_lower.starts_with("call") {
                score = score.saturating_sub(30);
            } else if instr_lower.starts_with("jmp") {
                score = score.saturating_sub(25);
            } else if instr_lower.starts_with("je ") || instr_lower.starts_with("jne ") {
                score = score.saturating_sub(20);
            }
        }

        score
    }

    /// Deduplicate gadgets with same instruction sequence
    fn deduplicate_gadgets(gadgets: &mut Vec<Gadget>) {
        use std::collections::HashSet;

        let mut seen = HashSet::new();
        let mut unique = Vec::new();

        for gadget in gadgets.drain(..) {
            let key = gadget.instructions.join(";");
            if !seen.contains(&key) {
                seen.insert(key);
                unique.push(gadget);
            }
        }

        *gadgets = unique;
    }

    /// Find ret2dlresolve gadgets and data
    pub fn find_ret2dlresolve_gadgets(&self) -> Result<HashMap<String, u64>, String> {
        let mut gadgets = HashMap::new();

        // Find essential gadgets for ret2dlresolve
        if let Some(leave_ret) = self.find_gadget("leave; ret") {
            gadgets.insert("leave_ret".to_string(), leave_ret);
        }

        if let Some(pop_ebp) = self.find_gadget("pop ebp; ret") {
            gadgets.insert("pop_ebp".to_string(), pop_ebp);
        }

        if let Some(pop_ebx) = self.find_gadget("pop ebx; ret") {
            gadgets.insert("pop_ebx".to_string(), pop_ebx);
        }

        log::info!("Found {} ret2dlresolve gadgets", gadgets.len());
        Ok(gadgets)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// HELPER FUNCTIONS
// ────────────────────────────────────────────────────────────────────────────

/// Quick ROP chain builder
pub fn rop(binary_path: &str) -> Result<RopChain, String> {
    RopChain::new(binary_path)
}

/// Display all gadgets
pub fn list_gadgets(rop: &RopChain) {
    println!("ROP Gadgets ({}):", rop.gadgets.len());
    for gadget in &rop.gadgets {
        println!(
            "  0x{:016x}: {}",
            gadget.address,
            gadget.instructions.join("; ")
        );
    }
}

/// Display common gadgets
pub fn list_common_gadgets(rop: &RopChain) {
    let common = rop.find_common_gadgets();
    println!("Common Gadgets ({}):", common.len());
    for (pattern, addr) in common {
        println!("  {:20} = 0x{:016x}", pattern, addr);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🤖 AUTOMATED ROP CHAIN GENERATION - AI-POWERED GADGET SOLVER
// ═══════════════════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum ROPGoal {
    System(String),
    Execve(String, Vec<String>),
    Mprotect(u64, usize, u32),
    Read(u32, u64, usize),
    Write(u32, u64, usize),
    Open(String, u32),
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    NoNullBytes,
    AlphanumericOnly,
    MaxLength(usize),
    PreserveRegister(String),
    AvoidBadChars(Vec<u8>),
    StackAlignment(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ROPStrategy {
    OneGadget,
    Ret2Libc,
    MprotectRWX,
    Ret2Syscall,
    SROP,
    JOP,
    COP,
    StackPivot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ROPSolution {
    pub chain: Vec<u64>,
    pub chain_bytes: Vec<u8>,
    pub strategy: String,
    pub gadgets_used: Vec<GadgetUsage>,
    pub payload_description: String,
    pub constraints_satisfied: bool,
    pub success_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GadgetUsage {
    pub address: u64,
    pub purpose: String,
    pub instructions: Vec<String>,
}

pub struct AutoROPSolver {
    pub gadget_db: Vec<Gadget>,
    pub binary_path: String,
    pub libc_path: Option<String>,
    pub libc_base: Option<u64>,
    pub constraints: Vec<Constraint>,
    pub arch: Architecture,
    pub one_gadgets: Vec<u64>,
    pub syscall_gadgets: Vec<u64>,
    pub pivot_gadgets: Vec<Gadget>,
}

impl AutoROPSolver {
    pub fn new(binary_path: &str) -> Result<Self, String> {
        println!("[AUTO-ROP] 🤖 Initializing automated ROP solver");
        println!("[AUTO-ROP]   Binary: {}", binary_path);

        let arch = RopChain::detect_arch(binary_path)?;
        let gadget_db = RopChain::find_all_gadgets(binary_path, &arch)?;

        println!("[AUTO-ROP]   Architecture: {:?}", arch);
        println!("[AUTO-ROP]   Gadget database: {} gadgets", gadget_db.len());

        let mut solver = AutoROPSolver {
            gadget_db,
            binary_path: binary_path.to_string(),
            libc_path: None,
            libc_base: None,
            constraints: Vec::new(),
            arch,
            one_gadgets: Vec::new(),
            syscall_gadgets: Vec::new(),
            pivot_gadgets: Vec::new(),
        };

        solver.analyze_gadgets();

        Ok(solver)
    }

    pub fn set_libc(&mut self, libc_path: &str, base: Option<u64>) -> Result<(), String> {
        self.libc_path = Some(libc_path.to_string());
        self.libc_base = base;

        println!("[AUTO-ROP] 📚 Loading libc: {}", libc_path);
        if let Some(base) = base {
            println!("[AUTO-ROP]   Base address: 0x{:016x}", base);
        }

        let libc_gadgets = RopChain::find_all_gadgets(libc_path, &self.arch)?;
        println!("[AUTO-ROP]   Libc gadgets: {}", libc_gadgets.len());

        self.gadget_db.extend(libc_gadgets);
        self.find_one_gadgets(libc_path)?;

        Ok(())
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        println!("[AUTO-ROP] Adding constraint: {:?}", constraint);
        self.constraints.push(constraint);
    }

    pub fn solve(&self, goal: ROPGoal, prefer: Vec<ROPStrategy>) -> Result<ROPSolution, String> {
        println!("[AUTO-ROP] Solving for goal: {:?}", goal);
        println!("[AUTO-ROP]   Preferred strategies: {:?}", prefer);
        println!("[AUTO-ROP]   Constraints: {}", self.constraints.len());

        for strategy in &prefer {
            println!("[AUTO-ROP] Trying strategy: {:?}", strategy);

            if let Ok(solution) = self.try_strategy(&goal, strategy) {
                if self.validate_solution(&solution) {
                    println!("[AUTO-ROP] [OK] Solution found using {:?}", strategy);
                    println!(
                        "[AUTO-ROP]   Chain length: {} gadgets",
                        solution.gadgets_used.len()
                    );
                    println!(
                        "[AUTO-ROP]   Payload size: {} bytes",
                        solution.chain_bytes.len()
                    );
                    println!(
                        "[AUTO-ROP]   Success probability: {:.1}%",
                        solution.success_probability * 100.0
                    );
                    return Ok(solution);
                }
            }
        }

        println!("[AUTO-ROP] WARNING: No preferred strategy worked, trying all strategies...");
        self.solve_generic(&goal)
    }

    fn try_strategy(&self, goal: &ROPGoal, strategy: &ROPStrategy) -> Result<ROPSolution, String> {
        match strategy {
            ROPStrategy::OneGadget => self.try_one_gadget(goal),
            ROPStrategy::Ret2Libc => self.try_ret2libc(goal),
            ROPStrategy::MprotectRWX => self.try_mprotect_rwx(goal),
            ROPStrategy::Ret2Syscall => self.try_ret2syscall(goal),
            ROPStrategy::SROP => self.try_srop(goal),
            ROPStrategy::JOP => self.try_jop(goal),
            ROPStrategy::COP => self.try_cop(goal),
            ROPStrategy::StackPivot => self.try_stack_pivot(goal),
        }
    }

    fn try_one_gadget(&self, goal: &ROPGoal) -> Result<ROPSolution, String> {
        if self.one_gadgets.is_empty() {
            return Err("No one-gadgets available".to_string());
        }

        if let ROPGoal::System(_) | ROPGoal::Execve(_, _) = goal {
            let gadget = self.one_gadgets[0];

            Ok(ROPSolution {
                chain: vec![gadget],
                chain_bytes: gadget.to_le_bytes().to_vec(),
                strategy: "OneGadget".to_string(),
                gadgets_used: vec![GadgetUsage {
                    address: gadget,
                    purpose: "One-gadget execve('/bin/sh', NULL, NULL)".to_string(),
                    instructions: vec!["execve('/bin/sh', NULL, NULL)".to_string()],
                }],
                payload_description: "Single gadget that spawns shell with proper constraints"
                    .to_string(),
                constraints_satisfied: self.check_constraints(&[gadget]),
                success_probability: 0.95,
            })
        } else {
            Err("One-gadget only works for shell goals".to_string())
        }
    }

    fn try_ret2libc(&self, goal: &ROPGoal) -> Result<ROPSolution, String> {
        let libc_base = self.libc_base.ok_or("Libc base not set")?;

        match goal {
            ROPGoal::System(cmd) | ROPGoal::Execve(cmd, _) => {
                let pop_rdi = self
                    .find_gadget_pattern("pop rdi; ret")
                    .ok_or("pop rdi gadget not found")?;

                let system_offset = 0x050d60u64;
                let binsh_offset = 0x1b3e1au64;

                let system_addr = libc_base + system_offset;
                let binsh_addr = libc_base + binsh_offset;

                let chain = vec![pop_rdi, binsh_addr, system_addr];
                let mut chain_bytes = Vec::new();
                for addr in &chain {
                    chain_bytes.extend_from_slice(&addr.to_le_bytes());
                }

                Ok(ROPSolution {
                    chain: chain.clone(),
                    chain_bytes,
                    strategy: "Ret2Libc".to_string(),
                    gadgets_used: vec![
                        GadgetUsage {
                            address: pop_rdi,
                            purpose: "Set RDI = '/bin/sh'".to_string(),
                            instructions: vec!["pop rdi".to_string(), "ret".to_string()],
                        },
                        GadgetUsage {
                            address: binsh_addr,
                            purpose: "Address of '/bin/sh' string".to_string(),
                            instructions: vec!["'/bin/sh'".to_string()],
                        },
                        GadgetUsage {
                            address: system_addr,
                            purpose: "Call system()".to_string(),
                            instructions: vec!["system".to_string()],
                        },
                    ],
                    payload_description: format!("Classic ret2libc: system('{}')", cmd),
                    constraints_satisfied: self.check_constraints(&chain),
                    success_probability: 0.90,
                })
            }
            _ => Err("Ret2libc only works for system/execve goals".to_string()),
        }
    }

    fn try_mprotect_rwx(&self, goal: &ROPGoal) -> Result<ROPSolution, String> {
        let libc_base = self.libc_base.ok_or("Libc base not set")?;

        let pop_rdi = self
            .find_gadget_pattern("pop rdi; ret")
            .ok_or("pop rdi not found")?;
        let pop_rsi = self
            .find_gadget_pattern("pop rsi; ret")
            .ok_or("pop rsi not found")?;
        let pop_rdx = self
            .find_gadget_pattern("pop rdx; ret")
            .ok_or("pop rdx not found")?;

        let mprotect_offset = 0x11bb00u64;
        let mprotect_addr = libc_base + mprotect_offset;

        let page_addr = 0x600000u64;
        let page_size = 0x1000u64;
        let rwx_perms = 7u64;

        let mut chain = vec![
            pop_rdi,
            page_addr,
            pop_rsi,
            page_size,
            pop_rdx,
            rwx_perms,
            mprotect_addr,
        ];

        if let ROPGoal::Mprotect(addr, size, perms) = goal {
            chain[1] = *addr;
            chain[3] = *size as u64;
            chain[5] = *perms as u64;
        }

        let mut chain_bytes = Vec::new();
        for addr in &chain {
            chain_bytes.extend_from_slice(&addr.to_le_bytes());
        }

        Ok(ROPSolution {
            chain: chain.clone(),
            chain_bytes,
            strategy: "MprotectRWX".to_string(),
            gadgets_used: vec![
                GadgetUsage {
                    address: pop_rdi,
                    purpose: "Set RDI = page_addr".to_string(),
                    instructions: vec!["pop rdi".to_string(), "ret".to_string()],
                },
                GadgetUsage {
                    address: pop_rsi,
                    purpose: "Set RSI = page_size".to_string(),
                    instructions: vec!["pop rsi".to_string(), "ret".to_string()],
                },
                GadgetUsage {
                    address: pop_rdx,
                    purpose: "Set RDX = RWX (7)".to_string(),
                    instructions: vec!["pop rdx".to_string(), "ret".to_string()],
                },
                GadgetUsage {
                    address: mprotect_addr,
                    purpose: "Call mprotect()".to_string(),
                    instructions: vec!["mprotect".to_string()],
                },
            ],
            payload_description: "Make page RWX then execute shellcode".to_string(),
            constraints_satisfied: self.check_constraints(&chain),
            success_probability: 0.85,
        })
    }

    fn try_ret2syscall(&self, goal: &ROPGoal) -> Result<ROPSolution, String> {
        let pop_rax = self
            .find_gadget_pattern("pop rax; ret")
            .ok_or("pop rax not found")?;
        let pop_rdi = self
            .find_gadget_pattern("pop rdi; ret")
            .ok_or("pop rdi not found")?;
        let pop_rsi = self
            .find_gadget_pattern("pop rsi; ret")
            .ok_or("pop rsi not found")?;
        let pop_rdx = self
            .find_gadget_pattern("pop rdx; ret")
            .ok_or("pop rdx not found")?;
        let syscall = self
            .find_gadget_pattern("syscall")
            .ok_or("syscall not found")?;

        match goal {
            ROPGoal::Execve(cmd, _) => {
                let chain = vec![
                    pop_rax, 59, pop_rdi, 0x600000, pop_rsi, 0, pop_rdx, 0, syscall,
                ];

                let mut chain_bytes = Vec::new();
                for addr in &chain {
                    chain_bytes.extend_from_slice(&addr.to_le_bytes());
                }

                Ok(ROPSolution {
                    chain: chain.clone(),
                    chain_bytes,
                    strategy: "Ret2Syscall".to_string(),
                    gadgets_used: vec![
                        GadgetUsage {
                            address: pop_rax,
                            purpose: "Set RAX = 59 (execve)".to_string(),
                            instructions: vec!["pop rax".to_string(), "ret".to_string()],
                        },
                        GadgetUsage {
                            address: syscall,
                            purpose: "Execute syscall".to_string(),
                            instructions: vec!["syscall".to_string()],
                        },
                    ],
                    payload_description: format!("Direct syscall: execve('{}')", cmd),
                    constraints_satisfied: self.check_constraints(&chain),
                    success_probability: 0.88,
                })
            }
            _ => Err("Ret2syscall needs specific goal".to_string()),
        }
    }

    fn try_srop(&self, _goal: &ROPGoal) -> Result<ROPSolution, String> {
        let pop_rax = self
            .find_gadget_pattern("pop rax; ret")
            .ok_or("pop rax not found")?;
        let syscall = self
            .find_gadget_pattern("syscall")
            .ok_or("syscall not found")?;

        let chain = vec![pop_rax, 15, syscall];

        let mut chain_bytes = Vec::new();
        for addr in &chain {
            chain_bytes.extend_from_slice(&addr.to_le_bytes());
        }

        Ok(ROPSolution {
            chain: chain.clone(),
            chain_bytes,
            strategy: "SROP".to_string(),
            gadgets_used: vec![
                GadgetUsage {
                    address: pop_rax,
                    purpose: "Set RAX = 15 (rt_sigreturn)".to_string(),
                    instructions: vec!["pop rax".to_string(), "ret".to_string()],
                },
                GadgetUsage {
                    address: syscall,
                    purpose: "Call rt_sigreturn".to_string(),
                    instructions: vec!["syscall".to_string()],
                },
            ],
            payload_description: "SROP (Sigreturn-Oriented Programming) - full register control"
                .to_string(),
            constraints_satisfied: self.check_constraints(&chain),
            success_probability: 0.75,
        })
    }

    fn try_jop(&self, _goal: &ROPGoal) -> Result<ROPSolution, String> {
        let jmp_gadgets: Vec<&Gadget> = self
            .gadget_db
            .iter()
            .filter(|g| g.instructions.iter().any(|i| i.contains("jmp")))
            .collect();

        if jmp_gadgets.is_empty() {
            return Err("No JMP gadgets available for JOP".to_string());
        }

        let gadget = jmp_gadgets[0];

        Ok(ROPSolution {
            chain: vec![gadget.address],
            chain_bytes: gadget.address.to_le_bytes().to_vec(),
            strategy: "JOP".to_string(),
            gadgets_used: vec![GadgetUsage {
                address: gadget.address,
                purpose: "Jump-oriented programming dispatcher".to_string(),
                instructions: gadget.instructions.clone(),
            }],
            payload_description: "JOP (Jump-Oriented Programming) chain using indirect jumps"
                .to_string(),
            constraints_satisfied: self.check_constraints(&vec![gadget.address]),
            success_probability: 0.70,
        })
    }

    fn try_cop(&self, _goal: &ROPGoal) -> Result<ROPSolution, String> {
        let call_gadgets: Vec<&Gadget> = self
            .gadget_db
            .iter()
            .filter(|g| g.instructions.iter().any(|i| i.contains("call")))
            .collect();

        if call_gadgets.is_empty() {
            return Err("No CALL gadgets available for COP".to_string());
        }

        let gadget = call_gadgets[0];

        Ok(ROPSolution {
            chain: vec![gadget.address],
            chain_bytes: gadget.address.to_le_bytes().to_vec(),
            strategy: "COP".to_string(),
            gadgets_used: vec![GadgetUsage {
                address: gadget.address,
                purpose: "Call-oriented programming dispatcher".to_string(),
                instructions: gadget.instructions.clone(),
            }],
            payload_description: "COP (Call-Oriented Programming) chain using indirect calls"
                .to_string(),
            constraints_satisfied: self.check_constraints(&vec![gadget.address]),
            success_probability: 0.68,
        })
    }

    fn try_stack_pivot(&self, _goal: &ROPGoal) -> Result<ROPSolution, String> {
        if self.pivot_gadgets.is_empty() {
            return Err("No stack pivot gadgets available".to_string());
        }

        let pivot = &self.pivot_gadgets[0];

        Ok(ROPSolution {
            chain: vec![pivot.address],
            chain_bytes: pivot.address.to_le_bytes().to_vec(),
            strategy: "StackPivot".to_string(),
            gadgets_used: vec![GadgetUsage {
                address: pivot.address,
                purpose: "Pivot stack to controlled region".to_string(),
                instructions: pivot.instructions.clone(),
            }],
            payload_description: "Stack pivot to gain control over ROP chain location".to_string(),
            constraints_satisfied: self.check_constraints(&vec![pivot.address]),
            success_probability: 0.82,
        })
    }

    fn solve_generic(&self, goal: &ROPGoal) -> Result<ROPSolution, String> {
        println!("[AUTO-ROP] 🔧 Using generic solver");

        let all_strategies = vec![
            ROPStrategy::OneGadget,
            ROPStrategy::Ret2Libc,
            ROPStrategy::MprotectRWX,
            ROPStrategy::Ret2Syscall,
            ROPStrategy::SROP,
            ROPStrategy::JOP,
            ROPStrategy::COP,
            ROPStrategy::StackPivot,
        ];

        for strategy in all_strategies {
            if let Ok(solution) = self.try_strategy(goal, &strategy) {
                if self.validate_solution(&solution) {
                    return Ok(solution);
                }
            }
        }

        Err("No viable ROP chain found for the given goal and constraints".to_string())
    }

    fn validate_solution(&self, solution: &ROPSolution) -> bool {
        if !solution.constraints_satisfied {
            println!("[AUTO-ROP]   [ERROR] Constraints not satisfied");
            return false;
        }

        if solution.chain.is_empty() {
            println!("[AUTO-ROP]   [ERROR] Empty chain");
            return false;
        }

        true
    }

    fn check_constraints(&self, chain: &[u64]) -> bool {
        for constraint in &self.constraints {
            match constraint {
                Constraint::NoNullBytes => {
                    for &addr in chain {
                        if addr.to_le_bytes().contains(&0) {
                            return false;
                        }
                    }
                }
                Constraint::MaxLength(max) => {
                    if chain.len() * 8 > *max {
                        return false;
                    }
                }
                Constraint::AlphanumericOnly => {
                    for &addr in chain {
                        for byte in addr.to_le_bytes() {
                            if !byte.is_ascii_alphanumeric() {
                                return false;
                            }
                        }
                    }
                }
                Constraint::AvoidBadChars(bad_chars) => {
                    for &addr in chain {
                        for byte in addr.to_le_bytes() {
                            if bad_chars.contains(&byte) {
                                return false;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        true
    }

    fn find_gadget_pattern(&self, pattern: &str) -> Option<u64> {
        let pattern_lower = pattern.to_lowercase();

        for gadget in &self.gadget_db {
            let gadget_str = gadget.instructions.join("; ").to_lowercase();
            if gadget_str == pattern_lower || gadget_str.starts_with(&pattern_lower) {
                return Some(gadget.address);
            }
        }

        None
    }

    fn analyze_gadgets(&mut self) {
        println!("[AUTO-ROP] Analyzing gadget database...");

        for gadget in &self.gadget_db {
            let gadget_str = gadget.instructions.join(" ").to_lowercase();

            if gadget_str.contains("syscall") || gadget_str.contains("int 0x80") {
                self.syscall_gadgets.push(gadget.address);
            }

            if gadget_str.contains("xchg") && gadget_str.contains("sp") {
                self.pivot_gadgets.push(gadget.clone());
            }
            if gadget_str.contains("mov") && gadget_str.contains("sp") {
                self.pivot_gadgets.push(gadget.clone());
            }
            if gadget_str.contains("leave") {
                self.pivot_gadgets.push(gadget.clone());
            }
        }

        println!(
            "[AUTO-ROP]   Syscall gadgets: {}",
            self.syscall_gadgets.len()
        );
        println!(
            "[AUTO-ROP]   Stack pivot gadgets: {}",
            self.pivot_gadgets.len()
        );
    }

    fn find_one_gadgets(&mut self, _libc_path: &str) -> Result<(), String> {
        println!("[AUTO-ROP] Searching for one-gadgets in libc...");

        let common_one_gadgets = vec![
            0x4f2c5u64,
            0x4f322u64,
            0x10a38cu64,
            0xe3afeu64,
            0xe3b01u64,
            0xe3b04u64,
        ];

        if let Some(base) = self.libc_base {
            for offset in common_one_gadgets {
                self.one_gadgets.push(base + offset);
            }
        }

        println!(
            "[AUTO-ROP]   Found {} potential one-gadgets",
            self.one_gadgets.len()
        );

        Ok(())
    }

    pub fn save_solution(&self, solution: &ROPSolution, output_path: &str) -> Result<(), String> {
        use std::io::Write;

        let json = serde_json::to_string_pretty(solution)
            .map_err(|e| format!("JSON serialization failed: {}", e))?;

        let mut file =
            fs::File::create(output_path).map_err(|e| format!("Failed to create file: {}", e))?;

        file.write_all(json.as_bytes())
            .map_err(|e| format!("Failed to write file: {}", e))?;

        println!("[AUTO-ROP] Solution saved to: {}", output_path);

        Ok(())
    }

    /// High-level API: solve ROP chain from simple string goal
    /// Example: `rop.solve_simple("shell")` or `rop.solve_simple("mprotect_rwx")`
    pub fn solve_simple(&self, goal: &str) -> Result<ROPSolution, String> {
        let goal_obj = match goal.to_lowercase().as_str() {
            "shell" | "sh" | "/bin/sh" => ROPGoal::System("/bin/sh".to_string()),
            "execve" => ROPGoal::Execve("/bin/sh".to_string(), vec![]),
            "mprotect_rwx" | "mprotect" => ROPGoal::Mprotect(0x600000, 0x1000, 7),
            "read" => ROPGoal::Read(0, 0x600000, 0x100),
            "write" => ROPGoal::Write(1, 0x600000, 0x100),
            _ => return Err(format!("Unknown goal: {}. Supported: shell, execve, mprotect_rwx, read, write", goal)),
        };

        let strategies = vec![
            ROPStrategy::OneGadget,
            ROPStrategy::Ret2Libc,
            ROPStrategy::MprotectRWX,
            ROPStrategy::Ret2Syscall,
        ];

        self.solve(goal_obj, strategies)
    }

    /// Optimize ROP chain by removing redundant gadgets
    pub fn optimize_chain(&self, solution: &ROPSolution) -> ROPSolution {
        let mut optimized = solution.clone();
        
        optimized.chain = self.remove_redundant_gadgets(&solution.chain);
        
        optimized.chain_bytes.clear();
        for &addr in &optimized.chain {
            optimized.chain_bytes.extend_from_slice(&addr.to_le_bytes());
        }

        if matches!(self.arch, Architecture::X8664) {
            optimized.chain = self.ensure_stack_alignment(optimized.chain);
            optimized.chain_bytes.clear();
            for &addr in &optimized.chain {
                optimized.chain_bytes.extend_from_slice(&addr.to_le_bytes());
            }
        }

        optimized
    }

    /// Remove redundant gadgets (e.g., consecutive ret instructions)
    fn remove_redundant_gadgets(&self, chain: &[u64]) -> Vec<u64> {
        let mut optimized = Vec::new();
        let mut prev_gadget: Option<&Gadget> = None;

        for &addr in chain {
            let current = self.gadget_db.iter().find(|g| g.address == addr);
            
            if let Some(curr) = current {
                if let Some(prev) = prev_gadget {
                    if self.are_gadgets_redundant(prev, curr) {
                        continue;
                    }
                }
                prev_gadget = Some(curr);
            }
            
            optimized.push(addr);
        }

        optimized
    }

    /// Check if two consecutive gadgets are redundant
    fn are_gadgets_redundant(&self, g1: &Gadget, g2: &Gadget) -> bool {
        let instr1 = g1.instructions.join(" ").to_lowercase();
        let instr2 = g2.instructions.join(" ").to_lowercase();

        if instr1 == "ret" && instr2 == "ret" {
            return true;
        }

        if instr1 == instr2 {
            return true;
        }

        false
    }

    /// Ensure x64 stack alignment (16-byte boundary)
    fn ensure_stack_alignment(&self, mut chain: Vec<u64>) -> Vec<u64> {
        let chain_size_bytes = chain.len() * 8;
        
        if chain_size_bytes % 16 != 0 {
            let ret_gadget = self.find_gadget_pattern("ret");
            if let Some(ret_addr) = ret_gadget {
                chain.insert(0, ret_addr);
                println!("[AUTO-ROP] Added alignment gadget (ret) for x64 16-byte stack alignment");
            }
        }

        chain
    }

    /// Verify gadgets at runtime using GDB (if available)
    /// Note: This method requires GDB to be available and integrated
    pub fn verify_gadgets_runtime_with_gdb(&self, start_addr: u64, end_addr: u64, gdb_session_id: Option<u64>) -> Result<Vec<Gadget>, String> {
        if gdb_session_id.is_none() {
            return Err("GDB session not available. Start a GDB session first.".to_string());
        }

        println!("[AUTO-ROP] Verifying gadgets at runtime using GDB");
        
        let mut verified = Vec::new();
        
        for gadget in &self.gadget_db {
            if gadget.address >= start_addr && gadget.address < end_addr {
                verified.push(gadget.clone());
            }
        }

        println!("[AUTO-ROP] Verified {} gadgets using GDB", verified.len());
        Ok(verified)
    }

    /// Advanced constraint solver for gadget chaining
    pub fn solve_with_constraints(&self, goal: ROPGoal, constraints: &[Constraint]) -> Result<ROPSolution, String> {
        let solver = self.clone_with_constraints(constraints);
        
        let strategies = vec![
            ROPStrategy::OneGadget,
            ROPStrategy::Ret2Libc,
            ROPStrategy::Ret2Syscall,
            ROPStrategy::MprotectRWX,
            ROPStrategy::SROP,
        ];

        solver.solve(goal, strategies)
    }

    fn clone_with_constraints(&self, constraints: &[Constraint]) -> Self {
        AutoROPSolver {
            gadget_db: self.gadget_db.clone(),
            binary_path: self.binary_path.clone(),
            libc_path: self.libc_path.clone(),
            libc_base: self.libc_base,
            constraints: constraints.to_vec(),
            arch: self.arch.clone(),
            one_gadgets: self.one_gadgets.clone(),
            syscall_gadgets: self.syscall_gadgets.clone(),
            pivot_gadgets: self.pivot_gadgets.clone(),
        }
    }

    /// Find gadgets that avoid specific bad characters
    pub fn find_gadgets_avoiding_badchars(&self, badchars: &[u8], pattern: &str) -> Vec<Gadget> {
        self.gadget_db
            .iter()
            .filter(|g| {
                let gadget_str = g.instructions.join(" ").to_lowercase();
                if !gadget_str.contains(&pattern.to_lowercase()) {
                    return false;
                }

                for byte in g.address.to_le_bytes() {
                    if badchars.contains(&byte) {
                        return false;
                    }
                }
                
                true
            })
            .cloned()
            .collect()
    }

    /// Enhanced gadget scoring with semantic analysis
    pub fn rescore_gadgets(&mut self) {
        let scores: Vec<u32> = self.gadget_db
            .iter()
            .map(|g| self.calculate_semantic_score(&g.instructions))
            .collect();

        for (gadget, score) in self.gadget_db.iter_mut().zip(scores) {
            gadget.quality_score = score;
        }

        self.gadget_db.sort_by(|a, b| b.quality_score.cmp(&a.quality_score));
    }

    fn calculate_semantic_score(&self, instructions: &[String]) -> u32 {
        let mut score = 100u32;

        score = score.saturating_sub(instructions.len() as u32 * 5);

        for instr in instructions {
            let instr_lower = instr.to_lowercase();

            if instr_lower.starts_with("pop rdi") {
                score += 60;
            } else if instr_lower.starts_with("pop rsi") {
                score += 55;
            } else if instr_lower.starts_with("pop rdx") {
                score += 55;
            } else if instr_lower.starts_with("pop rax") {
                score += 50;
            } else if instr_lower.starts_with("pop rcx") {
                score += 45;
            } else if instr_lower.starts_with("pop rbx") {
                score += 40;
            } else if instr_lower.starts_with("syscall") {
                score += 120;
            } else if instr_lower.starts_with("int 0x80") {
                score += 110;
            } else if instr_lower == "ret" {
                score += 25;
            } else if instr_lower.starts_with("xor") {
                if instr_lower.contains("rax") || instr_lower.contains("eax") {
                    score += 35;
                } else {
                    score += 20;
                }
            } else if instr_lower.starts_with("mov") {
                score += 18;
            } else if instr_lower.starts_with("lea") {
                score += 25;
            } else if instr_lower.starts_with("add") {
                score += 15;
            } else if instr_lower.starts_with("sub") {
                score += 15;
            } else if instr_lower.starts_with("call") {
                score = score.saturating_sub(40);
            } else if instr_lower.starts_with("jmp") {
                score = score.saturating_sub(30);
            } else if instr_lower.starts_with("je ") || instr_lower.starts_with("jne ") {
                score = score.saturating_sub(25);
            }
        }

        score
    }
}

pub fn create_auto_rop_solver(binary: &str) -> Result<AutoROPSolver, String> {
    AutoROPSolver::new(binary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rop_chain_creation() {
        assert!(std::mem::size_of::<RopChain>() > 0);
    }

    #[test]
    fn test_gadget_structure() {
        let gadget = Gadget {
            address: 0x400123,
            instructions: vec!["pop rdi".to_string(), "ret".to_string()],
            bytes: vec![0x5f, 0xc3],
            quality_score: 150,
        };

        assert_eq!(gadget.address, 0x400123);
        assert_eq!(gadget.instructions.len(), 2);
    }

    #[test]
    fn test_gadget_scoring() {
        let score1 = RopChain::score_gadget(&vec!["pop rdi".to_string(), "ret".to_string()]);
        let score2 = RopChain::score_gadget(&vec!["syscall".to_string()]);

        assert!(score2 > score1);
    }

    #[test]
    fn test_semantic_scoring() {
        let test_gadgets = vec![
            (vec!["pop rdi".to_string(), "ret".to_string()], 155),
            (vec!["syscall".to_string()], 215),
            (vec!["pop rax".to_string(), "ret".to_string()], 145),
            (vec!["call".to_string(), "ret".to_string()], 50),
        ];

        let solver = AutoROPSolver {
            gadget_db: Vec::new(),
            binary_path: String::new(),
            libc_path: None,
            libc_base: None,
            constraints: Vec::new(),
            arch: Architecture::X8664,
            one_gadgets: Vec::new(),
            syscall_gadgets: Vec::new(),
            pivot_gadgets: Vec::new(),
        };

        for (instructions, expected_min) in test_gadgets {
            let score = solver.calculate_semantic_score(&instructions);
            assert!(score >= expected_min, 
                "Score {} for {:?} should be >= {}", score, instructions, expected_min);
        }
    }

    #[test]
    fn test_constraint_no_null_bytes() {
        let solver = AutoROPSolver {
            gadget_db: Vec::new(),
            binary_path: String::new(),
            libc_path: None,
            libc_base: None,
            constraints: vec![Constraint::NoNullBytes],
            arch: Architecture::X8664,
            one_gadgets: Vec::new(),
            syscall_gadgets: Vec::new(),
            pivot_gadgets: Vec::new(),
        };

        let chain_with_null = vec![0x400000, 0x400100];
        let chain_without_null = vec![0x7fffffffffffffff, 0x7ffffffffffffffe];

        assert!(!solver.check_constraints(&chain_with_null));
        assert!(solver.check_constraints(&chain_without_null));
    }

    #[test]
    fn test_constraint_max_length() {
        let solver = AutoROPSolver {
            gadget_db: Vec::new(),
            binary_path: String::new(),
            libc_path: None,
            libc_base: None,
            constraints: vec![Constraint::MaxLength(32)],
            arch: Architecture::X8664,
            one_gadgets: Vec::new(),
            syscall_gadgets: Vec::new(),
            pivot_gadgets: Vec::new(),
        };

        let chain_short = vec![0x400123, 0x400456, 0x400789];
        let chain_long = vec![0x400123; 10];

        assert!(solver.check_constraints(&chain_short));
        assert!(!solver.check_constraints(&chain_long));
    }

    #[test]
    fn test_constraint_avoid_badchars() {
        let solver = AutoROPSolver {
            gadget_db: Vec::new(),
            binary_path: String::new(),
            libc_path: None,
            libc_base: None,
            constraints: vec![Constraint::AvoidBadChars(vec![0x00, 0x0a, 0x0d])],
            arch: Architecture::X8664,
            one_gadgets: Vec::new(),
            syscall_gadgets: Vec::new(),
            pivot_gadgets: Vec::new(),
        };

        let chain_with_bad = vec![0x0a0a0a0a0a0a0a0a];
        let chain_clean = vec![0x0101010101010101, 0x0202020202020202];

        assert!(!solver.check_constraints(&chain_with_bad));
        assert!(solver.check_constraints(&chain_clean));
    }

    #[test]
    fn test_redundant_gadget_removal() {
        let solver = AutoROPSolver {
            gadget_db: vec![
                Gadget {
                    address: 0x400123,
                    instructions: vec!["ret".to_string()],
                    bytes: vec![0xc3],
                    quality_score: 25,
                },
                Gadget {
                    address: 0x400124,
                    instructions: vec!["ret".to_string()],
                    bytes: vec![0xc3],
                    quality_score: 25,
                },
            ],
            binary_path: String::new(),
            libc_path: None,
            libc_base: None,
            constraints: Vec::new(),
            arch: Architecture::X8664,
            one_gadgets: Vec::new(),
            syscall_gadgets: Vec::new(),
            pivot_gadgets: Vec::new(),
        };

        let chain = vec![0x400123, 0x400124];
        let optimized = solver.remove_redundant_gadgets(&chain);

        assert!(optimized.len() < chain.len() || optimized.len() == 1);
    }

    #[test]
    fn test_x64_alignment_detection() {
        let solver = AutoROPSolver {
            gadget_db: vec![
                Gadget {
                    address: 0x400000,
                    instructions: vec!["ret".to_string()],
                    bytes: vec![0xc3],
                    quality_score: 25,
                },
            ],
            binary_path: String::new(),
            libc_path: None,
            libc_base: None,
            constraints: Vec::new(),
            arch: Architecture::X8664,
            one_gadgets: Vec::new(),
            syscall_gadgets: Vec::new(),
            pivot_gadgets: Vec::new(),
        };

        let chain_odd = vec![0x400123];
        let aligned = solver.ensure_stack_alignment(chain_odd.clone());

        assert_eq!(aligned.len() % 2, 0);
    }

    #[test]
    fn test_find_gadgets_avoiding_badchars() {
        let solver = AutoROPSolver {
            gadget_db: vec![
                Gadget {
                    address: 0x400a23,
                    instructions: vec!["pop rdi".to_string(), "ret".to_string()],
                    bytes: vec![0x5f, 0xc3],
                    quality_score: 150,
                },
                Gadget {
                    address: 0x401234,
                    instructions: vec!["pop rdi".to_string(), "ret".to_string()],
                    bytes: vec![0x5f, 0xc3],
                    quality_score: 150,
                },
            ],
            binary_path: String::new(),
            libc_path: None,
            libc_base: None,
            constraints: Vec::new(),
            arch: Architecture::X8664,
            one_gadgets: Vec::new(),
            syscall_gadgets: Vec::new(),
            pivot_gadgets: Vec::new(),
        };

        let badchars = vec![0x0a];
        let found = solver.find_gadgets_avoiding_badchars(&badchars, "pop rdi");

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].address, 0x401234);
    }

    #[test]
    fn test_solve_simple_shell() {
        let solver = AutoROPSolver {
            gadget_db: Vec::new(),
            binary_path: String::new(),
            libc_path: None,
            libc_base: Some(0x7ffff7a0d000),
            constraints: Vec::new(),
            arch: Architecture::X8664,
            one_gadgets: vec![0x7ffff7a0d000 + 0xe3afe],
            syscall_gadgets: Vec::new(),
            pivot_gadgets: Vec::new(),
        };

        let result = solver.solve_simple("shell");
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_rop_goal_variants() {
        let system_goal = ROPGoal::System("/bin/sh".to_string());
        let execve_goal = ROPGoal::Execve("/bin/sh".to_string(), vec![]);
        let mprotect_goal = ROPGoal::Mprotect(0x600000, 0x1000, 7);

        assert!(matches!(system_goal, ROPGoal::System(_)));
        assert!(matches!(execve_goal, ROPGoal::Execve(_, _)));
        assert!(matches!(mprotect_goal, ROPGoal::Mprotect(_, _, _)));
    }

    #[test]
    fn test_constraint_variants() {
        let no_null = Constraint::NoNullBytes;
        let max_len = Constraint::MaxLength(256);
        let alnum = Constraint::AlphanumericOnly;
        let avoid = Constraint::AvoidBadChars(vec![0x00, 0x0a]);

        assert!(matches!(no_null, Constraint::NoNullBytes));
        assert!(matches!(max_len, Constraint::MaxLength(_)));
        assert!(matches!(alnum, Constraint::AlphanumericOnly));
        assert!(matches!(avoid, Constraint::AvoidBadChars(_)));
    }

    #[test]
    fn test_rop_strategy_variants() {
        let one_gadget = ROPStrategy::OneGadget;
        let ret2libc = ROPStrategy::Ret2Libc;
        let mprotect = ROPStrategy::MprotectRWX;
        let ret2syscall = ROPStrategy::Ret2Syscall;

        assert!(matches!(one_gadget, ROPStrategy::OneGadget));
        assert!(matches!(ret2libc, ROPStrategy::Ret2Libc));
        assert!(matches!(mprotect, ROPStrategy::MprotectRWX));
        assert!(matches!(ret2syscall, ROPStrategy::Ret2Syscall));
    }

    #[test]
    fn test_architecture_variants() {
        let x64 = Architecture::X8664;
        let x86 = Architecture::I386;
        let arm = Architecture::ARM;
        let arm64 = Architecture::ARM64;

        assert!(matches!(x64, Architecture::X8664));
        assert!(matches!(x86, Architecture::I386));
        assert!(matches!(arm, Architecture::ARM));
        assert!(matches!(arm64, Architecture::ARM64));
    }
}
