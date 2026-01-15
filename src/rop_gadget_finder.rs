// Advanced ROP gadget finder with semantic analysis
// Native implementation with no external dependencies

use capstone::prelude::*;
use std::fs;

#[derive(Debug, Clone)]
pub struct Gadget {
    pub address: u64,
    pub bytes: Vec<u8>,
    pub instructions: Vec<String>,
    pub category: GadgetCategory,
    pub quality: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GadgetCategory {
    StackPivot,
    LoadRegister,
    StoreMemory,
    ArithmeticOperation,
    Syscall,
    ControlFlow,
    MemoryOperation,
    General,
}

pub struct ROPGadgetFinder {
    cs: Capstone,
    gadgets: Vec<Gadget>,
    base_address: u64,
}

impl ROPGadgetFinder {
    pub fn new(arch: Architecture) -> Result<Self, String> {
        let cs = match arch {
            Architecture::X64 => {
                Capstone::new()
                    .x86()
                    .mode(arch::x86::ArchMode::Mode64)
                    .detail(true)
                    .build()
                    .map_err(|e| format!("Capstone initialization failed: {:?}", e))?
            }
            Architecture::X86 => {
                Capstone::new()
                    .x86()
                    .mode(arch::x86::ArchMode::Mode32)
                    .detail(true)
                    .build()
                    .map_err(|e| format!("Capstone initialization failed: {:?}", e))?
            }
            _ => return Err("Unsupported architecture for ROP analysis".to_string()),
        };
        
        Ok(ROPGadgetFinder {
            cs,
            gadgets: Vec::new(),
            base_address: 0,
        })
    }
    
    pub fn analyze_file(&mut self, path: &str) -> Result<(), String> {
        use colored::Colorize;
        use std::path::Path;
        
        if path.is_empty() {
            return Err("Binary path cannot be empty".to_string());
        }
        
        if !Path::new(path).exists() {
            return Err(format!("File not found: {}", path));
        }
        
        eprintln!("{} Analyzing binary: {}", "[SEARCH]".cyan(), path.green());
        let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
        
        if data.is_empty() {
            return Err("Binary file is empty".to_string());
        }
        
        if data.len() < 100 {
            eprintln!("{} Binary is very small ({} bytes), may not contain useful gadgets", "[WARNING]".yellow(), data.len());
        }
        
        self.analyze_bytes(&data, 0x400000)
    }
    
    pub fn analyze_bytes(&mut self, data: &[u8], base_addr: u64) -> Result<(), String> {
        use colored::Colorize;
        
        if data.is_empty() {
            return Err("Cannot analyze empty data".to_string());
        }
        
        self.base_address = base_addr;
        self.gadgets.clear();
        
        let ret_bytes = [0xc3, 0xc2, 0xcb, 0xca];
        
        for (offset, _) in data.iter().enumerate() {
            if ret_bytes.contains(&data[offset]) {
                self.extract_gadget_at(data, offset, base_addr);
            }
        }
        
        if self.gadgets.is_empty() {
            eprintln!("{} No ROP gadgets found in binary", "[WARNING]".yellow());
            return Ok(());
        }
        
        self.categorize_gadgets();
        self.rank_gadgets();
        
        eprintln!("{} Found {} gadgets", "[OK]".green(), self.gadgets.len().to_string().cyan());
        
        Ok(())
    }
    
    fn extract_gadget_at(&mut self, data: &[u8], ret_offset: usize, base_addr: u64) {
        let max_insn_before = 10;
        
        for lookback in 1..=max_insn_before {
            if ret_offset < lookback {
                break;
            }
            
            let start = ret_offset - lookback;
            let end = (ret_offset + 2).min(data.len());
            
            if end <= start {
                continue;
            }
            
            let slice = &data[start..end];
            
            if let Ok(insns) = self.cs.disasm_all(slice, base_addr + start as u64) {
                if insns.len() == 0 {
                    continue;
                }
                
                let last_insn = insns.iter().last();
                if let Some(last) = last_insn {
                    let mnemonic = last.mnemonic().unwrap_or("");
                    if mnemonic == "ret" || mnemonic == "retf" {
                        let instructions: Vec<String> = insns.iter()
                            .map(|i| format!("{} {}", 
                                i.mnemonic().unwrap_or(""), 
                                i.op_str().unwrap_or("")))
                            .collect();
                        
                        if self.is_valid_gadget(&instructions) {
                            let gadget = Gadget {
                                address: base_addr + start as u64,
                                bytes: slice.to_vec(),
                                instructions,
                                category: GadgetCategory::General,
                                quality: 0,
                            };
                            
                            if !self.is_duplicate(&gadget) {
                                self.gadgets.push(gadget);
                            }
                        }
                    }
                }
            }
        }
    }
    
    fn is_valid_gadget(&self, instructions: &[String]) -> bool {
        if instructions.is_empty() {
            return false;
        }
        
        for insn in instructions {
            let lower = insn.to_lowercase();
            
            if lower.contains("call") && !lower.contains("ret") {
                return false;
            }
            
            if lower.contains("jmp") && !lower.contains("ret") {
                return false;
            }
            
            if lower.contains("invalid") {
                return false;
            }
        }
        
        true
    }
    
    fn is_duplicate(&self, new_gadget: &Gadget) -> bool {
        self.gadgets.iter().any(|g| {
            g.address == new_gadget.address || 
            g.instructions == new_gadget.instructions
        })
    }
    
    fn categorize_gadgets(&mut self) {
        let mut categories = Vec::new();
        for gadget in &self.gadgets {
            categories.push(self.categorize_gadget(&gadget.instructions));
        }
        for (gadget, category) in self.gadgets.iter_mut().zip(categories) {
            gadget.category = category;
        }
    }
    
    fn categorize_gadget(&self, instructions: &[String]) -> GadgetCategory {
        let combined = instructions.join(" ").to_lowercase();
        
        if combined.contains("syscall") || combined.contains("int 0x80") || combined.contains("sysenter") {
            return GadgetCategory::Syscall;
        }
        
        if combined.contains("xchg") && (combined.contains("esp") || combined.contains("rsp")) {
            return GadgetCategory::StackPivot;
        }
        
        if combined.contains("leave") {
            return GadgetCategory::StackPivot;
        }
        
        if combined.contains("pop") && instructions.len() <= 4 {
            return GadgetCategory::LoadRegister;
        }
        
        if combined.contains("mov") && combined.contains("[") {
            return GadgetCategory::StoreMemory;
        }
        
        if combined.contains("add") || combined.contains("sub") || 
           combined.contains("xor") || combined.contains("or") {
            return GadgetCategory::ArithmeticOperation;
        }
        
        if combined.contains("jmp") || combined.contains("call") {
            return GadgetCategory::ControlFlow;
        }
        
        GadgetCategory::General
    }
    
    fn rank_gadgets(&mut self) {
        let mut qualities = Vec::new();
        for gadget in &self.gadgets {
            qualities.push(self.calculate_quality(&gadget.instructions, &gadget.category));
        }
        for (gadget, quality) in self.gadgets.iter_mut().zip(qualities) {
            gadget.quality = quality;
        }
        
        self.gadgets.sort_by(|a, b| {
            b.quality.cmp(&a.quality)
                .then_with(|| a.instructions.len().cmp(&b.instructions.len()))
        });
    }
    
    fn calculate_quality(&self, instructions: &[String], category: &GadgetCategory) -> u8 {
        let mut quality: u8 = 100;
        
        quality = quality.saturating_sub((instructions.len() as u8) * 10);
        
        match category {
            GadgetCategory::Syscall => quality = quality.saturating_add(50),
            GadgetCategory::StackPivot => quality = quality.saturating_add(40),
            GadgetCategory::LoadRegister => quality = quality.saturating_add(30),
            GadgetCategory::StoreMemory => quality = quality.saturating_add(25),
            _ => {}
        }
        
        let combined = instructions.join(" ").to_lowercase();
        if combined.contains("bad") || combined.contains("invalid") {
            quality = quality.saturating_sub(50);
        }
        
        quality
    }
    
    pub fn find_gadgets_by_pattern(&self, pattern: &str) -> Vec<&Gadget> {
        let pattern_lower = pattern.to_lowercase();
        
        self.gadgets.iter()
            .filter(|g| {
                g.instructions.iter()
                    .any(|i| i.to_lowercase().contains(&pattern_lower))
            })
            .collect()
    }
    
    pub fn find_gadgets_by_category(&self, category: GadgetCategory) -> Vec<&Gadget> {
        self.gadgets.iter()
            .filter(|g| g.category == category)
            .collect()
    }
    
    pub fn get_best_gadgets(&self, count: usize) -> Vec<&Gadget> {
        self.gadgets.iter().take(count).collect()
    }
    
    pub fn build_rop_chain(&self, target: ROPTarget) -> Result<Vec<u64>, String> {
        match target {
            ROPTarget::Execve { binsh_addr, libc_base } => {
                self.build_execve_chain(binsh_addr, libc_base)
            }
            ROPTarget::System { binsh_addr, system_addr } => {
                self.build_system_chain(binsh_addr, system_addr)
            }
            ROPTarget::Mprotect { addr, size, prot } => {
                self.build_mprotect_chain(addr, size, prot)
            }
        }
    }
    
    fn build_execve_chain(&self, binsh_addr: u64, _libc_base: u64) -> Result<Vec<u64>, String> {
        let mut chain = Vec::new();
        
        let rdi_gadgets = self.find_gadgets_by_pattern("pop rdi");
        let pop_rdi = rdi_gadgets
            .iter()
            .filter(|g| g.instructions.len() <= 2)
            .next()
            .ok_or("Could not find 'pop rdi; ret' gadget")?;
        chain.push(pop_rdi.address);
        chain.push(binsh_addr);
        
        let rsi_gadgets = self.find_gadgets_by_pattern("pop rsi");
        let pop_rsi = rsi_gadgets
            .iter()
            .filter(|g| g.instructions.len() <= 2)
            .next()
            .ok_or("Could not find 'pop rsi; ret' gadget")?;
        chain.push(pop_rsi.address);
        chain.push(0);
        
        let rdx_gadgets = self.find_gadgets_by_pattern("pop rdx");
        let pop_rdx = rdx_gadgets
            .iter()
            .filter(|g| g.instructions.len() <= 2)
            .next()
            .ok_or("Could not find 'pop rdx; ret' gadget")?;
        chain.push(pop_rdx.address);
        chain.push(0);
        
        let rax_gadgets = self.find_gadgets_by_pattern("pop rax");
        let pop_rax = rax_gadgets
            .iter()
            .filter(|g| g.instructions.len() <= 2)
            .next()
            .ok_or("Could not find 'pop rax; ret' gadget")?;
        chain.push(pop_rax.address);
        chain.push(59);
        
        let syscall_gadgets = self.find_gadgets_by_category(GadgetCategory::Syscall);
        let syscall = syscall_gadgets
            .first()
            .ok_or("Could not find syscall gadget")?;
        chain.push(syscall.address);
        
        Ok(chain)
    }
    
    fn build_system_chain(&self, binsh_addr: u64, system_addr: u64) -> Result<Vec<u64>, String> {
        let mut chain = Vec::new();
        
        let rdi_gadgets = self.find_gadgets_by_pattern("pop rdi");
        let pop_rdi = rdi_gadgets
            .iter()
            .filter(|g| g.instructions.len() <= 2)
            .next()
            .ok_or("Could not find 'pop rdi; ret' gadget")?;
        chain.push(pop_rdi.address);
        chain.push(binsh_addr);
        
        chain.push(system_addr);
        
        Ok(chain)
    }
    
    fn build_mprotect_chain(&self, addr: u64, size: u64, prot: u64) -> Result<Vec<u64>, String> {
        let mut chain = Vec::new();
        
        let rdi_gadgets = self.find_gadgets_by_pattern("pop rdi");
        let pop_rdi = rdi_gadgets
            .iter()
            .filter(|g| g.instructions.len() <= 2)
            .next()
            .ok_or("Could not find 'pop rdi; ret' gadget")?;
        chain.push(pop_rdi.address);
        chain.push(addr);
        
        let rsi_gadgets = self.find_gadgets_by_pattern("pop rsi");
        let pop_rsi = rsi_gadgets
            .iter()
            .filter(|g| g.instructions.len() <= 2)
            .next()
            .ok_or("Could not find 'pop rsi; ret' gadget")?;
        chain.push(pop_rsi.address);
        chain.push(size);
        
        let rdx_gadgets = self.find_gadgets_by_pattern("pop rdx");
        let pop_rdx = rdx_gadgets
            .iter()
            .filter(|g| g.instructions.len() <= 2)
            .next()
            .ok_or("Could not find 'pop rdx; ret' gadget")?;
        chain.push(pop_rdx.address);
        chain.push(prot);
        
        let rax_gadgets = self.find_gadgets_by_pattern("pop rax");
        let pop_rax = rax_gadgets
            .iter()
            .filter(|g| g.instructions.len() <= 2)
            .next()
            .ok_or("Could not find 'pop rax; ret' gadget")?;
        chain.push(pop_rax.address);
        chain.push(10);
        
        let syscall_gadgets = self.find_gadgets_by_category(GadgetCategory::Syscall);
        let syscall = syscall_gadgets
            .first()
            .ok_or("Could not find syscall gadget")?;
        chain.push(syscall.address);
        
        Ok(chain)
    }
    
    pub fn print_gadgets(&self, limit: Option<usize>) {
        let count = limit.unwrap_or(self.gadgets.len());
        
        println!("Found {} gadgets (showing top {})", self.gadgets.len(), count);
        println!("{:-<80}", "");
        
        for (i, gadget) in self.gadgets.iter().take(count).enumerate() {
            println!("{:4}. 0x{:016x}: {} (quality: {}, category: {:?})",
                i + 1,
                gadget.address,
                gadget.instructions.join("; "),
                gadget.quality,
                gadget.category
            );
        }
    }
    
    pub fn export_json(&self) -> String {
        let gadgets_data: Vec<_> = self.gadgets.iter().map(|g| {
            serde_json::json!({
                "address": format!("0x{:x}", g.address),
                "instructions": g.instructions,
                "category": format!("{:?}", g.category),
                "quality": g.quality,
                "bytes": hex::encode(&g.bytes)
            })
        }).collect();
        
        serde_json::to_string_pretty(&gadgets_data).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86,
    X64,
    ARM,
    ARM64,
}

pub enum ROPTarget {
    Execve {
        binsh_addr: u64,
        libc_base: u64,
    },
    System {
        binsh_addr: u64,
        system_addr: u64,
    },
    Mprotect {
        addr: u64,
        size: u64,
        prot: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gadget_finder() {
        let x64_code = vec![
            0x5f,                   // pop rdi
            0xc3,                   // ret
            0x5e,                   // pop rsi
            0xc3,                   // ret
            0x48, 0x89, 0xe0,       // mov rax, rsp
            0xc3,                   // ret
        ];
        
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        finder.analyze_bytes(&x64_code, 0x400000).unwrap();
        
        assert!(finder.gadgets.len() > 0);
        
        let pop_rdi = finder.find_gadgets_by_pattern("pop rdi");
        assert!(pop_rdi.len() > 0);
    }

    #[test]
    fn test_categorization() {
        let finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        
        let syscall_gadget = vec!["syscall".to_string(), "ret".to_string()];
        assert_eq!(finder.categorize_gadget(&syscall_gadget), GadgetCategory::Syscall);
        
        let pop_gadget = vec!["pop rdi".to_string(), "ret".to_string()];
        assert_eq!(finder.categorize_gadget(&pop_gadget), GadgetCategory::LoadRegister);
    }
}
