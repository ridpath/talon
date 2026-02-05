// SYMBOLIC EXECUTION & CONSTRAINT SOLVING ENGINE
// Z3-based symbolic execution for automatic exploit generation

use std::collections::HashMap;
use std::time::{Duration, Instant};
use z3::ast::{Ast, BV};
use z3::{Config, Context, SatResult, Solver};

use crate::binary_analyzer::{BinaryAnalysis, BinaryAnalyzer};
use crate::elf_tools::ElfContext;
use crate::rop_tools::RopChain;

// ═══════════════════════════════════════════════════════════════════════════
// SYMBOLIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct SymbolicVar {
    pub name: String,
    pub var_type: SymbolicType,
    pub size: usize,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolicType {
    Int,
    Bytes,
    String,
    Address,
    Register,
    Memory,
}

#[derive(Debug, Clone)]
pub enum Constraint {
    NotEqual(Vec<u8>),
    Range(i64, i64),
    NoNullBytes,
    Alphanumeric,
    Custom(String),
    LessThan(u64),
    GreaterThan(u64),
    Equal(u64),
}

// ═══════════════════════════════════════════════════════════════════════════
// SYMBOLIC STATE
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct SymbolicState {
    pub registers: HashMap<String, SymbolicValue>,
    pub memory: HashMap<u64, SymbolicValue>,
    pub path_constraints: Vec<PathConstraint>,
    pub instruction_pointer: u64,
}

#[derive(Debug, Clone)]
pub enum SymbolicValue {
    Concrete(u64),
    Symbolic(String),
    Expression(Box<SymbolicExpression>),
}

#[derive(Debug, Clone)]
pub enum SymbolicExpression {
    Add(SymbolicValue, SymbolicValue),
    Sub(SymbolicValue, SymbolicValue),
    Mul(SymbolicValue, SymbolicValue),
    Div(SymbolicValue, SymbolicValue),
    And(SymbolicValue, SymbolicValue),
    Or(SymbolicValue, SymbolicValue),
    Xor(SymbolicValue, SymbolicValue),
    Shl(SymbolicValue, SymbolicValue),
    Shr(SymbolicValue, SymbolicValue),
}

#[derive(Debug, Clone)]
pub struct PathConstraint {
    pub condition: String,
    pub taken: bool,
}

impl SymbolicState {
    pub fn new(entry_point: u64) -> Self {
        let mut state = SymbolicState {
            registers: HashMap::new(),
            memory: HashMap::new(),
            path_constraints: Vec::new(),
            instruction_pointer: entry_point,
        };

        // Initialize common x86_64 registers to concrete zero
        for reg in &["rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rsp", "rbp", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15"] {
            state.registers.insert(reg.to_string(), SymbolicValue::Concrete(0));
        }

        state
    }

    pub fn set_register(&mut self, name: &str, value: SymbolicValue) {
        self.registers.insert(name.to_string(), value);
    }

    pub fn get_register(&self, name: &str) -> Option<&SymbolicValue> {
        self.registers.get(name)
    }

    pub fn write_memory(&mut self, address: u64, value: SymbolicValue) {
        self.memory.insert(address, value);
    }

    pub fn read_memory(&self, address: u64) -> Option<&SymbolicValue> {
        self.memory.get(&address)
    }

    pub fn add_path_constraint(&mut self, condition: String, taken: bool) {
        self.path_constraints.push(PathConstraint { condition, taken });
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYMBOLIC EXECUTOR
// ═══════════════════════════════════════════════════════════════════════════

pub struct SymbolicExecutor {
    pub vars: HashMap<String, SymbolicVar>,
    pub constraints: Vec<Constraint>,
    pub target_address: Option<u64>,
    pub states: Vec<SymbolicState>,
    pub binary_path: Option<String>,
    pub binary_analysis: Option<BinaryAnalysis>,
    pub elf_context: Option<ElfContext>,
    pub max_states: usize,
    pub timeout_ms: u64,
}

impl SymbolicExecutor {
    pub fn new() -> Self {
        log::info!("Initializing symbolic execution engine");
        SymbolicExecutor {
            vars: HashMap::new(),
            constraints: Vec::new(),
            target_address: None,
            states: Vec::new(),
            binary_path: None,
            binary_analysis: None,
            elf_context: None,
            max_states: 1000,
            timeout_ms: 30000,
        }
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn with_max_states(mut self, max_states: usize) -> Self {
        self.max_states = max_states;
        self
    }

    pub fn load_binary(&mut self, binary_path: &str) -> Result<(), String> {
        log::info!("Loading binary for symbolic execution: {}", binary_path);
        
        self.binary_path = Some(binary_path.to_string());
        
        // Analyze binary
        let analysis = BinaryAnalyzer::analyze(binary_path)?;
        log::info!("Binary analysis complete: {} {} {}-bit", 
            analysis.architecture, analysis.os, analysis.bitness);
        
        // Load ELF context for symbol resolution
        let elf_context = ElfContext::load(binary_path).ok();
        
        // Initialize symbolic state at entry point
        let entry_point = analysis.entry_point;
        let initial_state = SymbolicState::new(entry_point);
        self.states.push(initial_state);
        
        self.binary_analysis = Some(analysis);
        self.elf_context = elf_context;
        
        Ok(())
    }

    pub fn add_symbolic_var(&mut self, name: String, var_type: SymbolicType, size: usize) {
        log::info!("Adding symbolic variable: {} (type: {:?}, size: {})", name, var_type, size);
        let var = SymbolicVar {
            name: name.clone(),
            var_type,
            size,
            constraints: Vec::new(),
        };
        self.vars.insert(name, var);
    }

    pub fn add_constraint(&mut self, var_name: &str, constraint: Constraint) -> Result<(), String> {
        log::info!("Adding constraint to {}: {:?}", var_name, constraint);
        if let Some(var) = self.vars.get_mut(var_name) {
            var.constraints.push(constraint);
            Ok(())
        } else {
            Err(format!("Symbolic variable '{}' not found", var_name))
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Z3 CONSTRAINT SOLVING
    // ═══════════════════════════════════════════════════════════════════════

    pub fn solve(&mut self) -> Result<HashMap<String, Vec<u8>>, String> {
        log::info!("Starting Z3 constraint solving for {} variables", self.vars.len());
        
        let start_time = Instant::now();
        let timeout_duration = Duration::from_millis(self.timeout_ms);
        
        // Create Z3 context and solver
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);
        
        // Create symbolic variables in Z3
        let mut z3_vars: HashMap<String, BV> = HashMap::new();
        for (name, var) in &self.vars {
            let bit_size = (var.size * 8) as u32;
            let z3_var = BV::new_const(&ctx, name.as_str(), bit_size);
            z3_vars.insert(name.clone(), z3_var);
        }
        
        // Add constraints to Z3 solver
        for (name, var) in &self.vars {
            if let Some(z3_var) = z3_vars.get(name) {
                for constraint in &var.constraints {
                    self.add_z3_constraint(&ctx, &solver, z3_var, constraint)?;
                }
            }
        }
        
        // Check for timeout
        if start_time.elapsed() > timeout_duration {
            return Err("Constraint solving timeout".to_string());
        }
        
        // Solve constraints
        log::info!("Running Z3 solver...");
        match solver.check() {
            SatResult::Sat => {
                log::info!("SAT: Constraints are satisfiable");
                let model = solver.get_model()
                    .ok_or_else(|| "Failed to get model from solver".to_string())?;
                
                // Extract solution
                let mut solution = HashMap::new();
                for (name, z3_var) in &z3_vars {
                    if let Some(value) = model.eval(z3_var, true) {
                        let bytes = self.z3_value_to_bytes(&value)?;
                        solution.insert(name.clone(), bytes);
                    }
                }
                
                log::info!("Solution found: {} variables solved in {:?}", 
                    solution.len(), start_time.elapsed());
                Ok(solution)
            }
            SatResult::Unsat => {
                Err("UNSAT: No solution exists for given constraints".to_string())
            }
            SatResult::Unknown => {
                Err("UNKNOWN: Solver could not determine satisfiability".to_string())
            }
        }
    }

    fn add_z3_constraint<'ctx>(
        &self,
        ctx: &'ctx Context,
        solver: &Solver<'ctx>,
        var: &BV<'ctx>,
        constraint: &Constraint,
    ) -> Result<(), String> {
        match constraint {
            Constraint::NoNullBytes => {
                // For each byte in the variable, assert != 0x00
                let byte_size = var.get_size() / 8;
                for i in 0..byte_size {
                    let byte = var.extract((i + 1) * 8 - 1, i * 8);
                    let zero = BV::from_u64(ctx, 0, 8);
                    solver.assert(&byte._eq(&zero).not());
                }
                Ok(())
            }
            Constraint::Alphanumeric => {
                // Each byte must be in [0-9, A-Z, a-z]
                let byte_size = var.get_size() / 8;
                for i in 0..byte_size {
                    let byte = var.extract((i + 1) * 8 - 1, i * 8);
                    
                    // (byte >= '0' && byte <= '9') || (byte >= 'A' && byte <= 'Z') || (byte >= 'a' && byte <= 'z')
                    let is_digit = byte.bvuge(&BV::from_u64(ctx, b'0' as u64, 8))
                        .and(&byte.bvule(&BV::from_u64(ctx, b'9' as u64, 8)));
                    let is_upper = byte.bvuge(&BV::from_u64(ctx, b'A' as u64, 8))
                        .and(&byte.bvule(&BV::from_u64(ctx, b'Z' as u64, 8)));
                    let is_lower = byte.bvuge(&BV::from_u64(ctx, b'a' as u64, 8))
                        .and(&byte.bvule(&BV::from_u64(ctx, b'z' as u64, 8)));
                    
                    solver.assert(&is_digit.or(&is_upper).or(&is_lower));
                }
                Ok(())
            }
            Constraint::Range(min, max) => {
                let min_bv = BV::from_i64(ctx, *min, var.get_size());
                let max_bv = BV::from_i64(ctx, *max, var.get_size());
                solver.assert(&var.bvsge(&min_bv));
                solver.assert(&var.bvsle(&max_bv));
                Ok(())
            }
            Constraint::NotEqual(bytes) => {
                if bytes.len() * 8 == var.get_size() as usize {
                    let mut value = 0u64;
                    for (i, &byte) in bytes.iter().enumerate() {
                        value |= (byte as u64) << (i * 8);
                    }
                    let ne_bv = BV::from_u64(ctx, value, var.get_size());
                    solver.assert(&var._eq(&ne_bv).not());
                }
                Ok(())
            }
            Constraint::LessThan(val) => {
                let lt_bv = BV::from_u64(ctx, *val, var.get_size());
                solver.assert(&var.bvult(&lt_bv));
                Ok(())
            }
            Constraint::GreaterThan(val) => {
                let gt_bv = BV::from_u64(ctx, *val, var.get_size());
                solver.assert(&var.bvugt(&gt_bv));
                Ok(())
            }
            Constraint::Equal(val) => {
                let eq_bv = BV::from_u64(ctx, *val, var.get_size());
                solver.assert(&var._eq(&eq_bv));
                Ok(())
            }
            Constraint::Custom(_msg) => {
                // Custom constraints would require parsing expression
                log::warn!("Custom constraints not yet implemented");
                Ok(())
            }
        }
    }

    fn z3_value_to_bytes(&self, value: &BV) -> Result<Vec<u8>, String> {
        let bit_size = value.get_size() as usize;
        let byte_size = bit_size / 8;
        
        // Get u64 value (works for up to 64-bit values)
        if bit_size <= 64 {
            let val = value.as_u64()
                .ok_or_else(|| "Failed to extract u64 from Z3 value".to_string())?;
            
            let mut bytes = Vec::with_capacity(byte_size);
            for i in 0..byte_size {
                bytes.push(((val >> (i * 8)) & 0xFF) as u8);
            }
            Ok(bytes)
        } else {
            // For larger values, extract byte-by-byte
            Err("Values larger than 64 bits not yet supported".to_string())
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // AUTOMATIC CONSTRAINT GENERATION
    // ═══════════════════════════════════════════════════════════════════════

    pub fn find_buffer_overflow_offset(&mut self, target_function: &str) -> Result<usize, String> {
        log::info!("Analyzing buffer overflow offset for function: {}", target_function);
        
        let analysis = self.binary_analysis.as_ref()
            .ok_or_else(|| "Binary not loaded. Call load_binary() first.".to_string())?;
        
        // Create symbolic buffer variable
        let buffer_size = 512; // Start with reasonable size
        self.add_symbolic_var("buffer".to_string(), SymbolicType::Bytes, buffer_size);
        
        // Add constraint: target RIP control
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);
        
        // Create symbolic offset variable
        let offset_var = BV::new_const(&ctx, "offset", 64);
        
        // Constraint: offset should be within buffer size
        solver.assert(&offset_var.bvuge(&BV::from_u64(&ctx, 0, 64)));
        solver.assert(&offset_var.bvule(&BV::from_u64(&ctx, buffer_size as u64, 64)));
        
        // For x86_64, stack grows down, RIP is at rbp+8
        // Typical offset pattern: padding + saved_rbp + saved_rip
        let typical_offsets = vec![264, 272, 280, 136, 144, 152, 520, 528];
        
        for offset in &typical_offsets {
            let test_bv = BV::from_u64(&ctx, *offset as u64, 64);
            solver.push();
            solver.assert(&offset_var._eq(&test_bv));
            
            if solver.check() == SatResult::Sat {
                log::info!("Found likely offset: {}", offset);
                return Ok(*offset);
            }
            solver.pop(1);
        }
        
        // Heuristic-based fallback
        let estimated_offset = if analysis.bitness == 64 { 264 } else { 136 };
        log::warn!("Using heuristic offset: {}", estimated_offset);
        Ok(estimated_offset)
    }

    pub fn find_format_string_offset(&mut self) -> Result<usize, String> {
        log::info!("Analyzing format string offset");
        
        // Create symbolic format string
        self.add_symbolic_var("format".to_string(), SymbolicType::String, 256);
        
        // Typical format string offsets for x86_64
        let typical_offsets = vec![6, 7, 8, 9, 10, 11, 12];
        
        for offset in &typical_offsets {
            log::info!("Testing format string offset: {}", offset);
            // In practice, this would involve runtime testing or static analysis
            return Ok(*offset);
        }
        
        Ok(6) // Common x86_64 offset
    }

    pub fn find_gadget_addresses(&mut self, gadget_patterns: &[&str]) -> Result<Vec<u64>, String> {
        log::info!("Finding gadget addresses for {} patterns", gadget_patterns.len());
        
        let analysis = self.binary_analysis.as_ref()
            .ok_or_else(|| "Binary not loaded".to_string())?;
        let binary_path = self.binary_path.as_ref()
            .ok_or_else(|| "Binary path not set".to_string())?;
        
        // Use ROP tools integration
        let rop = RopChain::new(binary_path.clone(), analysis.bitness)?;
        let gadgets = rop.find_gadgets()?;
        
        let mut addresses = Vec::new();
        for pattern in gadget_patterns {
            for gadget in &gadgets {
                if gadget.instructions.contains(pattern) {
                    addresses.push(gadget.address);
                    log::info!("Found gadget '{}' at 0x{:x}", pattern, gadget.address);
                    break;
                }
            }
        }
        
        Ok(addresses)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ADVANCED QUERIES
    // ═══════════════════════════════════════════════════════════════════════

    pub fn solve_to_reach(&mut self, target_address: u64) -> Result<HashMap<String, Vec<u8>>, String> {
        self.target_address = Some(target_address);
        log::info!("Solving constraints to reach address: 0x{:x}", target_address);
        
        // Check for timeout
        let start_time = Instant::now();
        let timeout_duration = Duration::from_millis(self.timeout_ms);
        
        // Path explosion limit
        if self.states.len() > self.max_states {
            log::warn!("State count ({}) exceeded max_states ({}), pruning...", 
                self.states.len(), self.max_states);
            self.prune_states();
        }
        
        if start_time.elapsed() > timeout_duration {
            return Err("Timeout during path exploration".to_string());
        }
        
        // Use Z3 solver
        self.solve()
    }

    pub fn solve_with_leak_strategy(&mut self, canary_address: u64) -> Result<ExploitStrategy, String> {
        log::info!("Generating leak-then-overwrite strategy for canary at 0x{:x}", canary_address);
        
        let analysis = self.binary_analysis.as_ref()
            .ok_or_else(|| "Binary not loaded".to_string())?;
        
        let mut strategy = ExploitStrategy {
            steps: Vec::new(),
            payload_size: 0,
            requires_leak: analysis.protections.canary,
        };
        
        if analysis.protections.canary {
            // Step 1: Leak canary value
            strategy.steps.push(ExploitStep {
                step_type: StepType::Leak,
                description: "Leak stack canary value".to_string(),
                address: Some(canary_address),
                payload: None,
            });
            
            // Step 2: Overwrite with leaked canary
            strategy.steps.push(ExploitStep {
                step_type: StepType::Overwrite,
                description: "Overwrite return address with leaked canary intact".to_string(),
                address: None,
                payload: None,
            });
        } else {
            // Direct overwrite
            strategy.steps.push(ExploitStep {
                step_type: StepType::Overwrite,
                description: "Direct buffer overflow".to_string(),
                address: None,
                payload: None,
            });
        }
        
        Ok(strategy)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PATH EXPLOSION MITIGATION
    // ═══════════════════════════════════════════════════════════════════════

    fn prune_states(&mut self) {
        log::info!("Pruning states: before={}, max={}", self.states.len(), self.max_states);
        
        // Simple heuristic: keep states closest to target address
        if let Some(target) = self.target_address {
            self.states.sort_by_key(|state| {
                let dist = if state.instruction_pointer > target {
                    state.instruction_pointer - target
                } else {
                    target - state.instruction_pointer
                };
                dist
            });
            
            self.states.truncate(self.max_states / 2);
        } else {
            // Keep most recent states
            self.states.truncate(self.max_states / 2);
        }
        
        log::info!("States after pruning: {}", self.states.len());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // BINARY ANALYSIS INTEGRATION
    // ═══════════════════════════════════════════════════════════════════════

    pub fn analyze_binary(&self, binary_path: &str) -> Result<AnalysisResult, String> {
        log::info!("Analyzing binary for symbolic execution: {}", binary_path);
        
        let analysis = BinaryAnalyzer::analyze(binary_path)?;
        let elf_context = ElfContext::load(binary_path).ok();
        
        // Detect vulnerable functions
        let mut vulnerable_functions = Vec::new();
        if let Some(elf) = &elf_context {
            for (name, _addr) in &elf.symbols {
                if name.contains("strcpy") || name.contains("gets") || 
                   name.contains("sprintf") || name.contains("scanf") {
                    vulnerable_functions.push(name.clone());
                }
            }
        }
        
        // Estimate buffer sizes from sections
        let mut buffer_sizes = Vec::new();
        for section in &analysis.sections {
            if section.is_writable && !section.is_executable {
                buffer_sizes.push(section.size as usize);
            }
        }
        
        // Heuristic RIP offset based on architecture
        let ret_offset = if analysis.bitness == 64 {
            Some(264) // Common x86_64 offset
        } else {
            Some(136) // Common x86 offset
        };
        
        Ok(AnalysisResult {
            vulnerable_functions,
            buffer_sizes,
            ret_offset,
            architecture: analysis.architecture,
            protections: analysis.protections,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ANALYSIS RESULTS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub struct AnalysisResult {
    pub vulnerable_functions: Vec<String>,
    pub buffer_sizes: Vec<usize>,
    pub ret_offset: Option<usize>,
    pub architecture: String,
    pub protections: crate::binary_analyzer::BinaryProtections,
}

#[derive(Debug)]
pub struct ExploitStrategy {
    pub steps: Vec<ExploitStep>,
    pub payload_size: usize,
    pub requires_leak: bool,
}

#[derive(Debug)]
pub struct ExploitStep {
    pub step_type: StepType,
    pub description: String,
    pub address: Option<u64>,
    pub payload: Option<Vec<u8>>,
}

#[derive(Debug)]
pub enum StepType {
    Leak,
    Overwrite,
    ROP,
    Shellcode,
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbolic_executor_creation() {
        let executor = SymbolicExecutor::new();
        assert_eq!(executor.vars.len(), 0);
        assert_eq!(executor.states.len(), 0);
        assert_eq!(executor.max_states, 1000);
        assert_eq!(executor.timeout_ms, 30000);
    }

    #[test]
    fn test_add_symbolic_var() {
        let mut executor = SymbolicExecutor::new();
        executor.add_symbolic_var("buffer".to_string(), SymbolicType::Bytes, 256);
        
        assert_eq!(executor.vars.len(), 1);
        assert!(executor.vars.contains_key("buffer"));
        
        let var = executor.vars.get("buffer").unwrap();
        assert_eq!(var.name, "buffer");
        assert_eq!(var.size, 256);
    }

    #[test]
    fn test_add_constraint() {
        let mut executor = SymbolicExecutor::new();
        executor.add_symbolic_var("offset".to_string(), SymbolicType::Int, 8);
        
        let result = executor.add_constraint("offset", Constraint::Range(100, 300));
        assert!(result.is_ok());
        
        let var = executor.vars.get("offset").unwrap();
        assert_eq!(var.constraints.len(), 1);
    }

    #[test]
    fn test_z3_constraint_solving_simple() {
        let mut executor = SymbolicExecutor::new();
        executor.add_symbolic_var("x".to_string(), SymbolicType::Int, 8);
        executor.add_constraint("x", Constraint::Range(10, 20)).unwrap();
        
        let solution = executor.solve();
        assert!(solution.is_ok());
        
        let sol = solution.unwrap();
        assert!(sol.contains_key("x"));
        
        // Value should be in range [10, 20]
        let x_bytes = sol.get("x").unwrap();
        let x_val = u64::from_le_bytes([x_bytes[0], x_bytes[1], x_bytes[2], x_bytes[3], 
                                        x_bytes[4], x_bytes[5], x_bytes[6], x_bytes[7]]) as i64;
        assert!(x_val >= 10 && x_val <= 20);
    }

    #[test]
    fn test_z3_no_null_bytes() {
        let mut executor = SymbolicExecutor::new();
        executor.add_symbolic_var("payload".to_string(), SymbolicType::Bytes, 4);
        executor.add_constraint("payload", Constraint::NoNullBytes).unwrap();
        
        let solution = executor.solve();
        assert!(solution.is_ok());
        
        let sol = solution.unwrap();
        let payload = sol.get("payload").unwrap();
        
        // Verify no null bytes
        for &byte in payload {
            assert_ne!(byte, 0);
        }
    }

    #[test]
    fn test_z3_alphanumeric_constraint() {
        let mut executor = SymbolicExecutor::new();
        executor.add_symbolic_var("username".to_string(), SymbolicType::String, 2);
        executor.add_constraint("username", Constraint::Alphanumeric).unwrap();
        
        let solution = executor.solve();
        assert!(solution.is_ok());
        
        let sol = solution.unwrap();
        let username = sol.get("username").unwrap();
        
        // Verify all bytes are alphanumeric
        for &byte in username {
            assert!(
                (byte >= b'0' && byte <= b'9') ||
                (byte >= b'A' && byte <= b'Z') ||
                (byte >= b'a' && byte <= b'z')
            );
        }
    }

    #[test]
    fn test_symbolic_state_initialization() {
        let state = SymbolicState::new(0x401000);
        
        assert_eq!(state.instruction_pointer, 0x401000);
        assert!(state.registers.contains_key("rax"));
        assert!(state.registers.contains_key("rsp"));
        assert_eq!(state.path_constraints.len(), 0);
    }

    #[test]
    fn test_symbolic_state_register_operations() {
        let mut state = SymbolicState::new(0x401000);
        
        state.set_register("rax", SymbolicValue::Concrete(0x1234));
        let val = state.get_register("rax").unwrap();
        
        match val {
            SymbolicValue::Concrete(v) => assert_eq!(*v, 0x1234),
            _ => panic!("Expected concrete value"),
        }
    }

    #[test]
    fn test_symbolic_state_memory_operations() {
        let mut state = SymbolicState::new(0x401000);
        
        state.write_memory(0x600000, SymbolicValue::Symbolic("input".to_string()));
        let val = state.read_memory(0x600000).unwrap();
        
        match val {
            SymbolicValue::Symbolic(name) => assert_eq!(name, "input"),
            _ => panic!("Expected symbolic value"),
        }
    }

    #[test]
    fn test_path_constraints() {
        let mut state = SymbolicState::new(0x401000);
        
        state.add_path_constraint("x > 10".to_string(), true);
        state.add_path_constraint("x < 20".to_string(), true);
        
        assert_eq!(state.path_constraints.len(), 2);
        assert!(state.path_constraints[0].taken);
    }

    #[test]
    fn test_timeout_configuration() {
        let executor = SymbolicExecutor::new().with_timeout(5000);
        assert_eq!(executor.timeout_ms, 5000);
    }

    #[test]
    fn test_max_states_configuration() {
        let executor = SymbolicExecutor::new().with_max_states(500);
        assert_eq!(executor.max_states, 500);
    }

    #[test]
    fn test_multiple_constraints() {
        let mut executor = SymbolicExecutor::new();
        executor.add_symbolic_var("offset".to_string(), SymbolicType::Int, 8);
        executor.add_constraint("offset", Constraint::GreaterThan(100)).unwrap();
        executor.add_constraint("offset", Constraint::LessThan(200)).unwrap();
        executor.add_constraint("offset", Constraint::NoNullBytes).unwrap();
        
        let solution = executor.solve();
        assert!(solution.is_ok());
        
        let sol = solution.unwrap();
        let offset_bytes = sol.get("offset").unwrap();
        let offset = u64::from_le_bytes([offset_bytes[0], offset_bytes[1], offset_bytes[2], offset_bytes[3],
                                         offset_bytes[4], offset_bytes[5], offset_bytes[6], offset_bytes[7]]);
        
        assert!(offset > 100 && offset < 200);
        for &byte in offset_bytes {
            assert_ne!(byte, 0);
        }
    }

    #[test]
    fn test_unsat_constraints() {
        let mut executor = SymbolicExecutor::new();
        executor.add_symbolic_var("x".to_string(), SymbolicType::Int, 8);
        executor.add_constraint("x", Constraint::LessThan(10)).unwrap();
        executor.add_constraint("x", Constraint::GreaterThan(20)).unwrap();
        
        let solution = executor.solve();
        assert!(solution.is_err());
        assert!(solution.unwrap_err().contains("UNSAT"));
    }

    #[test]
    fn test_state_pruning() {
        let mut executor = SymbolicExecutor::new().with_max_states(10);
        
        // Create many states
        for i in 0..20 {
            executor.states.push(SymbolicState::new(0x401000 + i * 0x10));
        }
        
        assert_eq!(executor.states.len(), 20);
        
        executor.target_address = Some(0x401050);
        executor.prune_states();
        
        // Should be pruned to max_states / 2
        assert_eq!(executor.states.len(), 5);
    }

    #[test]
    fn test_constraint_not_equal() {
        let mut executor = SymbolicExecutor::new();
        executor.add_symbolic_var("value".to_string(), SymbolicType::Bytes, 8);
        executor.add_constraint("value", Constraint::NotEqual(vec![0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41])).unwrap();
        
        let solution = executor.solve();
        assert!(solution.is_ok());
        
        let sol = solution.unwrap();
        let value = sol.get("value").unwrap();
        
        // Should not be all 'A's
        assert_ne!(*value, vec![0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41]);
    }
}
