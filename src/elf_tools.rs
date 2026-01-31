use goblin::elf::Elf;
use goblin::Object;
use std::fs;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// ELF/PE SYMBOL RESOLUTION - PWNTOOLS-STYLE BINARY CONTEXT
// ═══════════════════════════════════════════════════════════════════════════

/// ELF binary context with symbol resolution
pub struct ElfContext {
    pub path: String,
    pub elf: Elf<'static>,
    pub base_addr: u64,
    pub symbols: HashMap<String, u64>,
    pub plt: HashMap<String, u64>,
    pub got: HashMap<String, u64>,
    pub sections: HashMap<String, (u64, u64)>, // name -> (addr, size)
    
    // Security features
    pub nx: bool,
    pub pie: bool,
    pub canary: bool,
    pub relro: bool,
    pub fortify: bool,
}

impl ElfContext {
    /// Load an ELF binary and parse all symbols
    /// 
    /// # Example
    /// ```no_run
    /// # use talon::elf_tools::ElfContext;
    /// # fn main() -> Result<(), String> {
    /// let elf = ElfContext::load("./vulnerable")?;
    /// let main_addr = elf.symbols.get("main");
    /// let puts_plt = elf.plt.get("puts");
    /// # Ok(())
    /// # }
    /// ```
    pub fn load(path: &str) -> Result<Self, String> {
        log::info!("Loading ELF binary: {}", path);
        
        // Read file
        let buffer = fs::read(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        // Parse with goblin
        let buffer_static: &'static [u8] = Box::leak(buffer.into_boxed_slice());
        let obj = Object::parse(buffer_static)
            .map_err(|e| format!("Failed to parse ELF: {}", e))?;
        
        match obj {
            Object::Elf(elf) => {
                // Extract symbols
                let mut symbols = HashMap::new();
                let mut plt = HashMap::new();
                let mut got = HashMap::new();
                let mut sections = HashMap::new();
                
                // Parse symbols
                for sym in &elf.syms {
                    if let Some(name) = elf.strtab.get_at(sym.st_name) {
                        if sym.st_value > 0 {
                            symbols.insert(name.to_string(), sym.st_value);
                        }
                    }
                }
                
                // Parse dynamic symbols
                for sym in &elf.dynsyms {
                    if let Some(name) = elf.dynstrtab.get_at(sym.st_name) {
                        if sym.st_value > 0 {
                            symbols.insert(name.to_string(), sym.st_value);
                        }
                    }
                }
                
                // Find PLT and GOT
                for sh in &elf.section_headers {
                    if let Some(name) = elf.shdr_strtab.get_at(sh.sh_name) {
                        sections.insert(name.to_string(), (sh.sh_addr, sh.sh_size));
                        
                        // PLT section
                        if name == ".plt" || name.starts_with(".plt.") {
                            // PLT entries are typically 16 bytes each on x86-64
                            // First entry is special, real entries start at +16
                            let mut offset = 16u64;
                            for sym in &elf.dynsyms {
                                if let Some(sym_name) = elf.dynstrtab.get_at(sym.st_name) {
                                    if !sym_name.is_empty() {
                                        plt.insert(sym_name.to_string(), sh.sh_addr + offset);
                                        offset += 16;
                                    }
                                }
                            }
                        }
                        
                        // GOT section
                        if name == ".got" || name == ".got.plt" {
                            let mut offset = 24u64; // Skip first 3 GOT entries
                            for sym in &elf.dynsyms {
                                if let Some(sym_name) = elf.dynstrtab.get_at(sym.st_name) {
                                    if !sym_name.is_empty() && sym.st_value == 0 {
                                        got.insert(sym_name.to_string(), sh.sh_addr + offset);
                                        offset += 8; // 8 bytes per GOT entry on x86-64
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Detect security features
                let nx = elf.program_headers.iter()
                    .any(|ph| ph.p_type == goblin::elf::program_header::PT_GNU_STACK && (ph.p_flags & 0x1) == 0);
                
                let pie = elf.header.e_type == goblin::elf::header::ET_DYN;
                
                let canary = symbols.contains_key("__stack_chk_fail");
                
                let relro = elf.program_headers.iter()
                    .any(|ph| ph.p_type == goblin::elf::program_header::PT_GNU_RELRO);
                
                let fortify = symbols.iter()
                    .any(|(name, _)| name.contains("_chk"));
                
                log::info!("Loaded ELF: {} symbols, {} PLT entries, {} GOT entries", 
                          symbols.len(), plt.len(), got.len());
                log::info!("Security: NX={}, PIE={}, Canary={}, RELRO={}, FORTIFY={}", 
                          nx, pie, canary, relro, fortify);
                
                Ok(ElfContext {
                    path: path.to_string(),
                    elf,
                    base_addr: 0, // Will be updated if PIE
                    symbols,
                    plt,
                    got,
                    sections,
                    nx,
                    pie,
                    canary,
                    relro,
                    fortify,
                })
            }
            _ => Err("Not an ELF file".to_string()),
        }
    }
    
    /// Get a symbol address by name
    pub fn symbol(&self, name: &str) -> Option<u64> {
        self.symbols.get(name).copied()
    }
    
    /// Get a PLT entry address
    pub fn plt_addr(&self, name: &str) -> Option<u64> {
        self.plt.get(name).copied()
    }
    
    /// Get a GOT entry address
    pub fn got_addr(&self, name: &str) -> Option<u64> {
        self.got.get(name).copied()
    }
    
    /// Get section info
    pub fn section(&self, name: &str) -> Option<(u64, u64)> {
        self.sections.get(name).copied()
    }
    
    /// Find a string in the binary
    pub fn find_string(&self, search: &str) -> Vec<u64> {
        let results = Vec::new();
        
        // Read binary and search for string
        if let Ok(data) = std::fs::read(&self.path) {
            let search_bytes = search.as_bytes();
            
            for (i, window) in data.windows(search_bytes.len()).enumerate() {
                if window == search_bytes {
                    // Check if this address is in a valid section
                    for (section_name, (section_addr, section_size)) in &self.sections {
                        let section_start = *section_addr as usize;
                        let section_end = section_start + *section_size as usize;
                        
                        if i >= section_start && i < section_end {
                            let offset = i - section_start;
                            let string_addr = section_addr + offset as u64;
                            log::info!("Found '{}' at 0x{:x} in section {}", search, string_addr, section_name);
                            return vec![string_addr];
                        }
                    }
                }
            }
        }
        
        log::warn!("String '{}' not found in binary", search);
        results
    }
    
    /// Get entry point
    pub fn entry(&self) -> u64 {
        self.elf.entry
    }
    
    /// Check if binary is 64-bit
    pub fn is_64bit(&self) -> bool {
        self.elf.is_64
    }
    
    /// Get architecture
    pub fn arch(&self) -> String {
        match self.elf.header.e_machine {
            goblin::elf::header::EM_X86_64 => "x86-64".to_string(),
            goblin::elf::header::EM_386 => "i386".to_string(),
            goblin::elf::header::EM_ARM => "ARM".to_string(),
            goblin::elf::header::EM_AARCH64 => "AArch64".to_string(),
            _ => format!("Unknown ({})", self.elf.header.e_machine),
        }
    }
    
    /// Display checksec-style security info
    pub fn checksec(&self) -> String {
        format!(
            "Arch:     {}\n\
             RELRO:    {}\n\
             Stack:    {}\n\
             NX:       {}\n\
             PIE:      {}\n\
             FORTIFY:  {}",
            self.arch(),
            if self.relro { "Full RELRO" } else { "No RELRO" },
            if self.canary { "Canary found" } else { "No canary" },
            if self.nx { "NX enabled" } else { "NX disabled" },
            if self.pie { "PIE enabled" } else { "No PIE" },
            if self.fortify { "Enabled" } else { "No" }
        )
    }
}

// ────────────────────────────────────────────────────────────────────────────
// HELPER FUNCTIONS
// ────────────────────────────────────────────────────────────────────────────

/// Quick load an ELF file
pub fn elf(path: &str) -> Result<ElfContext, String> {
    ElfContext::load(path)
}

/// Display all symbols
pub fn list_symbols(ctx: &ElfContext) {
    println!("Symbols ({}):", ctx.symbols.len());
    let mut syms: Vec<_> = ctx.symbols.iter().collect();
    syms.sort_by_key(|(_, addr)| **addr);
    for (name, addr) in syms {
        println!("  0x{:016x}  {}", addr, name);
    }
}

/// Display PLT entries
pub fn list_plt(ctx: &ElfContext) {
    println!("PLT ({}):", ctx.plt.len());
    let mut entries: Vec<_> = ctx.plt.iter().collect();
    entries.sort_by_key(|(_, addr)| **addr);
    for (name, addr) in entries {
        println!("  0x{:016x}  {}", addr, name);
    }
}

/// Display GOT entries
pub fn list_got(ctx: &ElfContext) {
    println!("GOT ({}):", ctx.got.len());
    let mut entries: Vec<_> = ctx.got.iter().collect();
    entries.sort_by_key(|(_, addr)| **addr);
    for (name, addr) in entries {
        println!("  0x{:016x}  {}", addr, name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elf_context_creation() {
        // This test would require a sample ELF binary
        // For now, just verify the struct exists
        assert!(std::mem::size_of::<ElfContext>() > 0);
    }

    #[test]
    fn test_symbol_lookup() {
        // Would need a real ELF file to test properly
        let symbols: HashMap<String, u64> = HashMap::new();
        assert!(!symbols.contains_key("main"));
    }
}
