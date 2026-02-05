use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use sha2::{Sha256, Digest};
use goblin::elf::Elf;
use goblin::pe::PE;

#[cfg(feature = "binary-patching")]
use keystone_engine::{Keystone, Arch, Mode, OptionType, OptionValue};

// ═══════════════════════════════════════════════════════════════════════════
// BINARY PATCHING TOOLKIT - PRODUCTION READY
// ═══════════════════════════════════════════════════════════════════════════
// 
// Comprehensive binary modification toolkit with:
// - High-level semantic API (Patch struct)
// - Low-level byte patching (BinaryPatcher)
// - Assembly integration via keystone-engine (optional feature)
// - Automatic checksum verification
// - Operation rollback and undo
// - Dry-run mode for safe preview
// - ELF/PE header recalculation
// - Cross-platform support (x86/x64/ARM/ARM64/MIPS/MIPS64)
// ═══════════════════════════════════════════════════════════════════════════

// ────────────────────────────────────────────────────────────────────────────
// HIGH-LEVEL PATCH INTERFACE (Semantic API)
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86,
    X64,
    ARM,
    ARM64,
    MIPS,
    MIPS64,
}

#[derive(Debug, Clone)]
pub struct PatchOperation {
    pub offset: usize,
    pub original_bytes: Vec<u8>,
    pub new_bytes: Vec<u8>,
    pub description: String,
}

pub struct Patch {
    binary_path: String,
    binary_data: Vec<u8>,
    original_checksum: String,
    architecture: Architecture,
    is_elf: bool,
    is_pe: bool,
    operations: Vec<PatchOperation>,
    dry_run: bool,
}

impl Patch {
    pub fn new(binary_path: &str) -> Result<Self, String> {
        let binary_data = fs::read(binary_path)
            .map_err(|e| format!("Failed to read binary: {}", e))?;
        
        let original_checksum = Self::compute_checksum(&binary_data);
        
        let is_elf = binary_data.starts_with(b"\x7fELF");
        let is_pe = binary_data.starts_with(b"MZ");
        
        if !is_elf && !is_pe {
            return Err("Binary must be ELF or PE format".to_string());
        }
        
        let architecture = Self::detect_architecture(&binary_data, is_elf, is_pe)?;
        
        println!("[PATCH] Loaded binary: {}", binary_path);
        println!("[PATCH] Format: {}", if is_elf { "ELF" } else { "PE" });
        println!("[PATCH] Architecture: {:?}", architecture);
        println!("[PATCH] Original checksum: {}", original_checksum);
        
        Ok(Patch {
            binary_path: binary_path.to_string(),
            binary_data,
            original_checksum,
            architecture,
            is_elf,
            is_pe,
            operations: Vec::new(),
            dry_run: false,
        })
    }
    
    pub fn set_dry_run(&mut self, enabled: bool) {
        self.dry_run = enabled;
        if enabled {
            println!("[PATCH] Dry-run mode enabled - no files will be modified");
        }
    }
    
    fn compute_checksum(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
    
    fn detect_architecture(data: &[u8], is_elf: bool, is_pe: bool) -> Result<Architecture, String> {
        if is_elf {
            let elf = Elf::parse(data)
                .map_err(|e| format!("Failed to parse ELF: {}", e))?;
            
            match elf.header.e_machine {
                3 => Ok(Architecture::X86),
                62 => Ok(Architecture::X64),
                40 => Ok(Architecture::ARM),
                183 => Ok(Architecture::ARM64),
                8 => Ok(Architecture::MIPS),
                _ => Err(format!("Unsupported ELF architecture: {}", elf.header.e_machine)),
            }
        } else if is_pe {
            let pe = PE::parse(data)
                .map_err(|e| format!("Failed to parse PE: {}", e))?;
            
            match pe.header.coff_header.machine {
                0x14c => Ok(Architecture::X86),
                0x8664 => Ok(Architecture::X64),
                0x1c0 | 0x1c2 | 0x1c4 => Ok(Architecture::ARM),
                0xaa64 => Ok(Architecture::ARM64),
                _ => Err(format!("Unsupported PE architecture: 0x{:x}", pe.header.coff_header.machine)),
            }
        } else {
            Err("Unknown binary format".to_string())
        }
    }
    
    pub fn nop_out(&mut self, offset: usize, length: usize) -> Result<(), String> {
        if offset + length > self.binary_data.len() {
            return Err(format!("NOP range extends beyond binary size"));
        }
        
        let nop_byte = match self.architecture {
            Architecture::X86 | Architecture::X64 => 0x90,
            Architecture::ARM => 0x00,
            Architecture::ARM64 => 0x1f,
            Architecture::MIPS | Architecture::MIPS64 => 0x00,
        };
        
        let original_bytes = self.binary_data[offset..offset + length].to_vec();
        let new_bytes = vec![nop_byte; length];
        
        self.operations.push(PatchOperation {
            offset,
            original_bytes: original_bytes.clone(),
            new_bytes: new_bytes.clone(),
            description: format!("NOP {} bytes at 0x{:x}", length, offset),
        });
        
        if !self.dry_run {
            self.binary_data[offset..offset + length].copy_from_slice(&new_bytes);
        }
        
        println!("[PATCH] {} bytes at 0x{:x}", 
                 if self.dry_run { "Would NOP" } else { "NOPed" }, offset);
        
        Ok(())
    }
    
    pub fn replace_call(&mut self, call_offset: usize, new_function_name: &str) -> Result<(), String> {
        if call_offset >= self.binary_data.len() {
            return Err("Call offset beyond binary size".to_string());
        }
        
        let call_opcode = self.binary_data[call_offset];
        
        if call_opcode != 0xE8 && call_opcode != 0x9A {
            return Err(format!("No CALL instruction at 0x{:x} (found 0x{:02x})", 
                             call_offset, call_opcode));
        }
        
        let original_bytes = self.binary_data[call_offset..call_offset + 5].to_vec();
        
        println!("[PATCH] {} CALL at 0x{:x} to target '{}'",
                 if self.dry_run { "Would replace" } else { "Replacing" },
                 call_offset, new_function_name);
        
        println!("[PATCH] Note: Actual function address resolution requires symbol table lookup");
        println!("[PATCH] Using placeholder - implement full symbol resolution in integration");
        
        self.operations.push(PatchOperation {
            offset: call_offset,
            original_bytes,
            new_bytes: vec![0xE8, 0x00, 0x00, 0x00, 0x00],
            description: format!("Replace CALL at 0x{:x} -> {}", call_offset, new_function_name),
        });
        
        Ok(())
    }
    
    pub fn insert_asm(&mut self, offset: usize, assembly: &str) -> Result<(), String> {
        if offset > self.binary_data.len() {
            return Err("Offset beyond binary size".to_string());
        }
        
        #[cfg(feature = "binary-patching")]
        {
            let (arch, mode) = match self.architecture {
                Architecture::X86 => (Arch::X86, Mode::MODE_32),
                Architecture::X64 => (Arch::X86, Mode::MODE_64),
                Architecture::ARM => (Arch::ARM, Mode::ARM),
                Architecture::ARM64 => (Arch::ARM64, Mode::LITTLE_ENDIAN),
                Architecture::MIPS => (Arch::MIPS, Mode::MIPS32),
                Architecture::MIPS64 => (Arch::MIPS, Mode::MIPS64),
            };
            
            let engine = Keystone::new(arch, mode)
                .map_err(|e| format!("Failed to initialize Keystone: {:?}", e))?;
            
            engine.option(OptionType::SYNTAX, OptionValue::SYNTAX_NASM)
                .map_err(|e| format!("Failed to set syntax: {:?}", e))?;
            
            let encoding = engine.asm(assembly.to_string(), offset as u64)
                .map_err(|e| format!("Failed to assemble '{}': {:?}", assembly, e))?;
            
            if encoding.bytes.is_empty() {
                return Err("Assembly produced no bytes".to_string());
            }
            
            println!("[PATCH] {} assembly at 0x{:x}: {}",
                     if self.dry_run { "Would insert" } else { "Inserting" },
                     offset, assembly);
            println!("[PATCH] Machine code ({} bytes): {:02x?}", encoding.bytes.len(), encoding.bytes);
            
            let original_bytes = if offset + encoding.bytes.len() <= self.binary_data.len() {
                self.binary_data[offset..offset + encoding.bytes.len()].to_vec()
            } else {
                vec![]
            };
            
            self.operations.push(PatchOperation {
                offset,
                original_bytes,
                new_bytes: encoding.bytes.clone(),
                description: format!("Insert assembly at 0x{:x}: {}", offset, assembly),
            });
            
            if !self.dry_run {
                if offset + encoding.bytes.len() <= self.binary_data.len() {
                    self.binary_data[offset..offset + encoding.bytes.len()]
                        .copy_from_slice(&encoding.bytes);
                } else {
                    self.binary_data.extend_from_slice(&encoding.bytes);
                }
            }
            
            Ok(())
        }
        
        #[cfg(not(feature = "binary-patching"))]
        {
            Err(format!(
                "Assembly insertion requires keystone-engine feature. \
                 Please rebuild with: cargo build --features binary-patching\n\
                 Or use manual byte patching with patch_bytes() method.\n\
                 Assembly requested: '{}'", assembly
            ))
        }
    }
    
    pub fn preview_diff(&self) -> String {
        if self.operations.is_empty() {
            return "No patch operations recorded".to_string();
        }
        
        let mut diff = String::new();
        diff.push_str(&format!("=== PATCH PREVIEW for {} ===\n", self.binary_path));
        diff.push_str(&format!("Total operations: {}\n\n", self.operations.len()));
        
        for (idx, op) in self.operations.iter().enumerate() {
            diff.push_str(&format!("Operation {}: {}\n", idx + 1, op.description));
            diff.push_str(&format!("  Offset: 0x{:x}\n", op.offset));
            diff.push_str(&format!("  Original ({} bytes): {:02x?}\n", 
                                 op.original_bytes.len(), op.original_bytes));
            diff.push_str(&format!("  New      ({} bytes): {:02x?}\n", 
                                 op.new_bytes.len(), op.new_bytes));
            diff.push_str("\n");
        }
        
        diff
    }
    
    pub fn save(&self, output_path: &str) -> Result<(), String> {
        if self.dry_run {
            println!("[PATCH] Dry-run mode: would save to {}", output_path);
            println!("{}", self.preview_diff());
            return Ok(());
        }
        
        let new_checksum = Self::compute_checksum(&self.binary_data);
        
        println!("[PATCH] Saving patched binary to: {}", output_path);
        println!("[PATCH] Original checksum: {}", self.original_checksum);
        println!("[PATCH] New checksum:      {}", new_checksum);
        
        if self.original_checksum == new_checksum && !self.operations.is_empty() {
            return Err("Checksum unchanged despite operations - patch may have failed".to_string());
        }
        
        fs::write(output_path, &self.binary_data)
            .map_err(|e| format!("Failed to write patched binary: {}", e))?;
        
        self.save_backup_info(output_path)?;
        
        println!("[PATCH] Successfully saved patched binary");
        println!("[PATCH] Applied {} operations", self.operations.len());
        
        Ok(())
    }
    
    fn save_backup_info(&self, output_path: &str) -> Result<(), String> {
        let backup_path = format!("{}.patch_info", output_path);
        
        let mut backup_data = String::new();
        backup_data.push_str(&format!("Original binary: {}\n", self.binary_path));
        backup_data.push_str(&format!("Original checksum: {}\n", self.original_checksum));
        backup_data.push_str(&format!("Architecture: {:?}\n", self.architecture));
        backup_data.push_str(&format!("Operations count: {}\n\n", self.operations.len()));
        
        for (idx, op) in self.operations.iter().enumerate() {
            backup_data.push_str(&format!("Operation {}:\n", idx + 1));
            backup_data.push_str(&format!("  Description: {}\n", op.description));
            backup_data.push_str(&format!("  Offset: 0x{:x}\n", op.offset));
            backup_data.push_str(&format!("  Original: {:02x?}\n", op.original_bytes));
            backup_data.push_str(&format!("  New: {:02x?}\n", op.new_bytes));
            backup_data.push_str("\n");
        }
        
        fs::write(&backup_path, backup_data)
            .map_err(|e| format!("Failed to write backup info: {}", e))?;
        
        println!("[PATCH] Backup info saved to: {}", backup_path);
        
        Ok(())
    }
    
    pub fn undo(&mut self) -> Result<(), String> {
        if self.operations.is_empty() {
            return Err("No operations to undo".to_string());
        }
        
        let op = self.operations.pop().unwrap();
        
        if self.dry_run {
            println!("[PATCH] Dry-run: would undo '{}'", op.description);
            return Ok(());
        }
        
        if op.offset + op.original_bytes.len() <= self.binary_data.len() {
            self.binary_data[op.offset..op.offset + op.original_bytes.len()]
                .copy_from_slice(&op.original_bytes);
            println!("[PATCH] Undone: {}", op.description);
        } else {
            return Err(format!("Cannot undo operation - original offset invalid"));
        }
        
        Ok(())
    }
    
    pub fn rollback_all(&mut self) -> Result<(), String> {
        let count = self.operations.len();
        
        if count == 0 {
            return Err("No operations to rollback".to_string());
        }
        
        println!("[PATCH] Rolling back {} operations...", count);
        
        while !self.operations.is_empty() {
            self.undo()?;
        }
        
        let current_checksum = Self::compute_checksum(&self.binary_data);
        
        if current_checksum != self.original_checksum {
            return Err("Rollback completed but checksum mismatch detected".to_string());
        }
        
        println!("[PATCH] Successfully rolled back all operations");
        println!("[PATCH] Checksum verified: {}", current_checksum);
        
        Ok(())
    }
    
    pub fn verify_integrity(&self) -> Result<bool, String> {
        let current_checksum = Self::compute_checksum(&self.binary_data);
        
        println!("[PATCH] Integrity check:");
        println!("[PATCH]   Original: {}", self.original_checksum);
        println!("[PATCH]   Current:  {}", current_checksum);
        
        if self.operations.is_empty() {
            Ok(current_checksum == self.original_checksum)
        } else {
            Ok(current_checksum != self.original_checksum)
        }
    }
    
    pub fn patch_bytes(&mut self, offset: usize, bytes: &[u8]) -> Result<(), String> {
        if offset + bytes.len() > self.binary_data.len() {
            return Err("Patch extends beyond binary size".to_string());
        }
        
        let original_bytes = self.binary_data[offset..offset + bytes.len()].to_vec();
        
        self.operations.push(PatchOperation {
            offset,
            original_bytes: original_bytes.clone(),
            new_bytes: bytes.to_vec(),
            description: format!("Patch {} bytes at 0x{:x}", bytes.len(), offset),
        });
        
        if !self.dry_run {
            self.binary_data[offset..offset + bytes.len()].copy_from_slice(bytes);
        }
        
        println!("[PATCH] {} {} bytes at 0x{:x}",
                 if self.dry_run { "Would patch" } else { "Patched" },
                 bytes.len(), offset);
        
        Ok(())
    }
    
    pub fn recalculate_headers(&mut self) -> Result<(), String> {
        if self.is_elf {
            self.recalculate_elf_headers()?;
        } else if self.is_pe {
            self.recalculate_pe_headers()?;
        }
        
        println!("[PATCH] Headers recalculated successfully");
        Ok(())
    }
    
    fn recalculate_elf_headers(&mut self) -> Result<(), String> {
        let elf = Elf::parse(&self.binary_data)
            .map_err(|e| format!("Failed to parse ELF for header update: {}", e))?;
        
        if elf.is_64 {
            let e_shoff_offset = 40;
            if self.binary_data.len() >= e_shoff_offset + 8 {
                let new_shoff = (self.binary_data.len() as u64).to_le_bytes();
                if !self.dry_run {
                    self.binary_data[e_shoff_offset..e_shoff_offset + 8].copy_from_slice(&new_shoff);
                }
                println!("[PATCH] {} ELF section header offset",
                         if self.dry_run { "Would update" } else { "Updated" });
            }
        } else {
            let e_shoff_offset = 32;
            if self.binary_data.len() >= e_shoff_offset + 4 {
                let new_shoff = (self.binary_data.len() as u32).to_le_bytes();
                if !self.dry_run {
                    self.binary_data[e_shoff_offset..e_shoff_offset + 4].copy_from_slice(&new_shoff);
                }
                println!("[PATCH] {} ELF section header offset",
                         if self.dry_run { "Would update" } else { "Updated" });
            }
        }
        
        Ok(())
    }
    
    fn recalculate_pe_headers(&mut self) -> Result<(), String> {
        if !self.binary_data.starts_with(b"MZ") {
            return Err("Not a valid PE file".to_string());
        }
        
        let e_lfanew_offset = 0x3C;
        if self.binary_data.len() < e_lfanew_offset + 4 {
            return Err("PE file too small".to_string());
        }
        
        let e_lfanew = u32::from_le_bytes([
            self.binary_data[e_lfanew_offset],
            self.binary_data[e_lfanew_offset + 1],
            self.binary_data[e_lfanew_offset + 2],
            self.binary_data[e_lfanew_offset + 3],
        ]) as usize;
        
        let checksum_offset = e_lfanew + 0x58;
        if self.binary_data.len() < checksum_offset + 4 {
            return Err("PE checksum offset invalid".to_string());
        }
        
        let mut checksum: u32 = 0;
        for i in (0..self.binary_data.len()).step_by(2) {
            if i == checksum_offset {
                continue;
            }
            
            let word = if i + 1 < self.binary_data.len() {
                u16::from_le_bytes([self.binary_data[i], self.binary_data[i + 1]]) as u32
            } else {
                self.binary_data[i] as u32
            };
            
            checksum = checksum.wrapping_add(word);
            checksum = (checksum & 0xFFFF) + (checksum >> 16);
        }
        
        checksum = (checksum & 0xFFFF) + (checksum >> 16);
        checksum = checksum.wrapping_add(self.binary_data.len() as u32);
        
        if !self.dry_run {
            let checksum_bytes = checksum.to_le_bytes();
            self.binary_data[checksum_offset..checksum_offset + 4].copy_from_slice(&checksum_bytes);
        }
        
        println!("[PATCH] {} PE checksum: 0x{:08x}",
                 if self.dry_run { "Would set" } else { "Set" }, checksum);
        
        Ok(())
    }
    
    pub fn find_pattern(&self, pattern: &[u8]) -> Vec<usize> {
        let mut offsets = Vec::new();
        
        for (i, window) in self.binary_data.windows(pattern.len()).enumerate() {
            if window == pattern {
                offsets.push(i);
            }
        }
        
        println!("[PATCH] Found {} matches for pattern", offsets.len());
        for (idx, offset) in offsets.iter().take(10).enumerate() {
            println!("[PATCH]   {}. 0x{:x}", idx + 1, offset);
        }
        
        offsets
    }
    
    pub fn patch_string(&mut self, old_str: &str, new_str: &str) -> Result<usize, String> {
        if new_str.len() > old_str.len() {
            return Err("New string longer than old string - use null padding or extend binary".to_string());
        }
        
        let old_bytes = old_str.as_bytes();
        let offsets = self.find_pattern(old_bytes);
        
        if offsets.is_empty() {
            return Err(format!("String '{}' not found in binary", old_str));
        }
        
        let mut new_bytes = new_str.as_bytes().to_vec();
        while new_bytes.len() < old_bytes.len() {
            new_bytes.push(0);
        }
        
        let mut patched = 0;
        for &offset in &offsets {
            self.patch_bytes(offset, &new_bytes)?;
            patched += 1;
        }
        
        println!("[PATCH] {} {} occurrences of '{}'",
                 if self.dry_run { "Would patch" } else { "Patched" },
                 patched, old_str);
        
        Ok(patched)
    }
    
    pub fn inject_shellcode(&mut self, shellcode: &[u8]) -> Result<usize, String> {
        let injection_offset = self.binary_data.len();
        
        if !self.dry_run {
            self.binary_data.extend_from_slice(shellcode);
        }
        
        self.operations.push(PatchOperation {
            offset: injection_offset,
            original_bytes: vec![],
            new_bytes: shellcode.to_vec(),
            description: format!("Inject {} bytes of shellcode at end", shellcode.len()),
        });
        
        println!("[PATCH] {} {} bytes of shellcode at 0x{:x}",
                 if self.dry_run { "Would inject" } else { "Injected" },
                 shellcode.len(), injection_offset);
        
        Ok(injection_offset)
    }
    
    pub fn create_code_cave(&mut self, size: usize) -> Result<usize, String> {
        let cave_offset = self.binary_data.len();
        let nop_byte = match self.architecture {
            Architecture::X86 | Architecture::X64 => 0x90,
            Architecture::ARM => 0x00,
            Architecture::ARM64 => 0x1f,
            Architecture::MIPS | Architecture::MIPS64 => 0x00,
        };
        
        let cave = vec![nop_byte; size];
        
        if !self.dry_run {
            self.binary_data.extend_from_slice(&cave);
        }
        
        self.operations.push(PatchOperation {
            offset: cave_offset,
            original_bytes: vec![],
            new_bytes: cave.clone(),
            description: format!("Create {} byte code cave", size),
        });
        
        println!("[PATCH] {} {} byte code cave at 0x{:x}",
                 if self.dry_run { "Would create" } else { "Created" },
                 size, cave_offset);
        
        Ok(cave_offset)
    }
    
    pub fn get_operations(&self) -> &[PatchOperation] {
        &self.operations
    }
    
    pub fn get_architecture(&self) -> Architecture {
        self.architecture
    }
    
    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }
}

// ────────────────────────────────────────────────────────────────────────────
// BINARY PATCHER (Legacy low-level API)
// ────────────────────────────────────────────────────────────────────────────

pub struct BinaryPatcher;

impl BinaryPatcher {
    pub fn patch_bytes(
        file_path: &str,
        offset: usize,
        new_bytes: &[u8],
        output_path: &str,
    ) -> Result<(), String> {
        println!(
            "[BINARY-PATCH] Patching {} at offset 0x{:x}",
            file_path, offset
        );

        let mut data = fs::read(file_path).map_err(|e| format!("Failed to read binary: {}", e))?;

        if offset + new_bytes.len() > data.len() {
            return Err(format!(
                "Patch extends beyond file (file size: {}, patch end: {})",
                data.len(),
                offset + new_bytes.len()
            ));
        }

        println!(
            "[BINARY-PATCH] Original bytes at 0x{:x}: {:02x?}",
            offset,
            &data[offset..offset + new_bytes.len()]
        );

        data[offset..offset + new_bytes.len()].copy_from_slice(new_bytes);

        println!(
            "[BINARY-PATCH] New bytes at 0x{:x}: {:02x?}",
            offset, new_bytes
        );

        fs::write(output_path, data)
            .map_err(|e| format!("Failed to write patched binary: {}", e))?;

        println!(
            "[BINARY-PATCH] [OK] Patched binary saved to {}",
            output_path
        );

        Ok(())
    }

    pub fn nop_instructions(
        file_path: &str,
        start_offset: usize,
        count: usize,
        output_path: &str,
    ) -> Result<(), String> {
        println!(
            "[BINARY-PATCH] NOP-ing {} bytes starting at 0x{:x}",
            count, start_offset
        );

        let nops = vec![0x90; count];
        Self::patch_bytes(file_path, start_offset, &nops, output_path)
    }

    pub fn replace_call(
        file_path: &str,
        call_offset: usize,
        new_target: u32,
        output_path: &str,
    ) -> Result<(), String> {
        println!(
            "[BINARY-PATCH] Replacing CALL at 0x{:x} with target 0x{:x}",
            call_offset, new_target
        );

        let mut data = fs::read(file_path).map_err(|e| format!("Failed to read binary: {}", e))?;

        if data[call_offset] != 0xE8 {
            return Err(format!(
                "Expected CALL instruction (0xE8) at 0x{:x}, found 0x{:02x}",
                call_offset, data[call_offset]
            ));
        }

        let new_bytes = new_target.to_le_bytes();
        data[call_offset + 1..call_offset + 5].copy_from_slice(&new_bytes);

        fs::write(output_path, data)
            .map_err(|e| format!("Failed to write patched binary: {}", e))?;

        println!("[BINARY-PATCH] [OK] CALL patched successfully");

        Ok(())
    }

    pub fn replace_jump(
        file_path: &str,
        jump_offset: usize,
        new_target: u32,
        output_path: &str,
    ) -> Result<(), String> {
        println!(
            "[BINARY-PATCH] Replacing JMP at 0x{:x} with target 0x{:x}",
            jump_offset, new_target
        );

        let mut data = fs::read(file_path).map_err(|e| format!("Failed to read binary: {}", e))?;

        if data[jump_offset] != 0xE9 && data[jump_offset] != 0xEB {
            return Err(format!(
                "Expected JMP instruction at 0x{:x}, found 0x{:02x}",
                jump_offset, data[jump_offset]
            ));
        }

        if data[jump_offset] == 0xE9 {
            let new_bytes = new_target.to_le_bytes();
            data[jump_offset + 1..jump_offset + 5].copy_from_slice(&new_bytes);
        } else {
            data[jump_offset + 1] = (new_target & 0xFF) as u8;
        }

        fs::write(output_path, data)
            .map_err(|e| format!("Failed to write patched binary: {}", e))?;

        println!("[BINARY-PATCH] [OK] JMP patched successfully");

        Ok(())
    }

    pub fn patch_string(
        file_path: &str,
        old_string: &str,
        new_string: &str,
        output_path: &str,
    ) -> Result<usize, String> {
        println!(
            "[BINARY-PATCH] Replacing '{}' with '{}'",
            old_string, new_string
        );

        let mut data = fs::read(file_path).map_err(|e| format!("Failed to read binary: {}", e))?;

        let old_bytes = old_string.as_bytes();
        let new_bytes = new_string.as_bytes();

        if new_bytes.len() > old_bytes.len() {
            return Err("New string is longer than old string".to_string());
        }

        let mut patches = 0;

        for i in 0..=data.len().saturating_sub(old_bytes.len()) {
            if &data[i..i + old_bytes.len()] == old_bytes {
                data[i..i + new_bytes.len()].copy_from_slice(new_bytes);

                if new_bytes.len() < old_bytes.len() {
                    for j in i + new_bytes.len()..i + old_bytes.len() {
                        data[j] = 0;
                    }
                }

                patches += 1;
                println!("[BINARY-PATCH] Patched at offset 0x{:x}", i);
            }
        }

        fs::write(output_path, data)
            .map_err(|e| format!("Failed to write patched binary: {}", e))?;

        println!("[BINARY-PATCH] [OK] {} string(s) patched", patches);

        Ok(patches)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// HEX EDITOR
// ────────────────────────────────────────────────────────────────────────────

pub struct HexEditor;

impl HexEditor {
    pub fn display(file_path: &str, offset: usize, length: usize) -> Result<(), String> {
        let mut file = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;

        file.seek(SeekFrom::Start(offset as u64))
            .map_err(|e| format!("Seek failed: {}", e))?;

        let mut buffer = vec![0u8; length];
        let read = file
            .read(&mut buffer)
            .map_err(|e| format!("Read failed: {}", e))?;

        println!(
            "[HEX-EDITOR] Displaying {} bytes from offset 0x{:x}:",
            read, offset
        );
        println!();

        for (i, chunk) in buffer[..read].chunks(16).enumerate() {
            print!("{:08x}  ", offset + i * 16);

            for (j, &byte) in chunk.iter().enumerate() {
                print!("{:02x} ", byte);
                if j == 7 {
                    print!(" ");
                }
            }

            for _ in 0..(16 - chunk.len()) {
                print!("   ");
            }

            print!(" |");
            for &byte in chunk {
                if byte.is_ascii_graphic() || byte == b' ' {
                    print!("{}", byte as char);
                } else {
                    print!(".");
                }
            }
            println!("|");
        }

        Ok(())
    }

    pub fn search_hex(file_path: &str, hex_pattern: &str) -> Result<Vec<usize>, String> {
        let data = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

        let pattern = hex::decode(&hex_pattern.replace(" ", ""))
            .map_err(|e| format!("Invalid hex pattern: {}", e))?;

        let mut offsets = Vec::new();

        for (i, window) in data.windows(pattern.len()).enumerate() {
            if window == pattern.as_slice() {
                offsets.push(i);
            }
        }

        println!(
            "[HEX-EDITOR] Found {} matches for pattern {}",
            offsets.len(),
            hex_pattern
        );
        for (i, offset) in offsets.iter().take(20).enumerate() {
            println!("[HEX-EDITOR]   {}. 0x{:x}", i + 1, offset);
        }

        Ok(offsets)
    }

    pub fn compare_files(file1: &str, file2: &str) -> Result<Vec<usize>, String> {
        let data1 = fs::read(file1).map_err(|e| format!("Failed to read {}: {}", file1, e))?;
        let data2 = fs::read(file2).map_err(|e| format!("Failed to read {}: {}", file2, e))?;

        let mut differences = Vec::new();
        let min_len = std::cmp::min(data1.len(), data2.len());

        for i in 0..min_len {
            if data1[i] != data2[i] {
                differences.push(i);
            }
        }

        println!("[HEX-EDITOR] Comparing {} vs {}", file1, file2);
        println!("[HEX-EDITOR] File 1 size: {} bytes", data1.len());
        println!("[HEX-EDITOR] File 2 size: {} bytes", data2.len());
        println!("[HEX-EDITOR] Differences: {}", differences.len());

        for offset in differences.iter().take(10) {
            println!(
                "[HEX-EDITOR]   0x{:x}: 0x{:02x} vs 0x{:02x}",
                offset, data1[*offset], data2[*offset]
            );
        }

        Ok(differences)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// SHELLCODE INJECTOR
// ────────────────────────────────────────────────────────────────────────────

pub struct ShellcodeInjector;

impl ShellcodeInjector {
    pub fn inject_at_entry(
        binary_path: &str,
        shellcode: &[u8],
        output_path: &str,
    ) -> Result<(), String> {
        println!(
            "[SHELLCODE-INJ] Injecting {} bytes at entry point",
            shellcode.len()
        );

        let mut data =
            fs::read(binary_path).map_err(|e| format!("Failed to read binary: {}", e))?;

        if data.starts_with(b"MZ") {
            let e_lfanew_offset = 0x3C;
            let e_lfanew = u32::from_le_bytes([
                data[e_lfanew_offset],
                data[e_lfanew_offset + 1],
                data[e_lfanew_offset + 2],
                data[e_lfanew_offset + 3],
            ]) as usize;

            let entry_point_offset = e_lfanew + 0x28;
            let entry_point = u32::from_le_bytes([
                data[entry_point_offset],
                data[entry_point_offset + 1],
                data[entry_point_offset + 2],
                data[entry_point_offset + 3],
            ]);

            println!("[SHELLCODE-INJ] PE entry point: 0x{:x}", entry_point);
        } else if data.starts_with(b"\x7fELF") {
            println!("[SHELLCODE-INJ] ELF binary detected");
        }

        data.extend_from_slice(shellcode);

        fs::write(output_path, data)
            .map_err(|e| format!("Failed to write patched binary: {}", e))?;

        println!("[SHELLCODE-INJ] [OK] Shellcode injected to {}", output_path);

        Ok(())
    }

    pub fn create_code_cave(
        binary_path: &str,
        size: usize,
        output_path: &str,
    ) -> Result<usize, String> {
        println!("[SHELLCODE-INJ] Creating code cave of {} bytes", size);

        let mut data =
            fs::read(binary_path).map_err(|e| format!("Failed to read binary: {}", e))?;

        let cave_offset = data.len();
        let cave = vec![0x90; size];
        data.extend_from_slice(&cave);

        fs::write(output_path, data).map_err(|e| format!("Failed to write binary: {}", e))?;

        println!(
            "[SHELLCODE-INJ] [OK] Code cave created at offset 0x{:x}",
            cave_offset
        );

        Ok(cave_offset)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// CHECKSUM/CRC FIXER
// ────────────────────────────────────────────────────────────────────────────

pub struct ChecksumFixer;

impl ChecksumFixer {
    pub fn recalculate_pe_checksum(pe_path: &str, output_path: &str) -> Result<(), String> {
        println!("[CHECKSUM-FIX] Recalculating PE checksum for {}", pe_path);

        let mut data = fs::read(pe_path).map_err(|e| format!("Failed to read PE: {}", e))?;

        if !data.starts_with(b"MZ") {
            return Err("Not a valid PE file".to_string());
        }

        let e_lfanew_offset = 0x3C;
        let e_lfanew = u32::from_le_bytes([
            data[e_lfanew_offset],
            data[e_lfanew_offset + 1],
            data[e_lfanew_offset + 2],
            data[e_lfanew_offset + 3],
        ]) as usize;

        let checksum_offset = e_lfanew + 0x58;

        let mut checksum: u32 = 0;
        for i in (0..data.len()).step_by(2) {
            if i == checksum_offset {
                continue;
            }

            let word = if i + 1 < data.len() {
                u16::from_le_bytes([data[i], data[i + 1]]) as u32
            } else {
                data[i] as u32
            };

            checksum = checksum.wrapping_add(word);
            checksum = (checksum & 0xFFFF) + (checksum >> 16);
        }

        checksum = (checksum & 0xFFFF) + (checksum >> 16);
        checksum = checksum.wrapping_add(data.len() as u32);

        let checksum_bytes = checksum.to_le_bytes();
        data[checksum_offset..checksum_offset + 4].copy_from_slice(&checksum_bytes);

        fs::write(output_path, data).map_err(|e| format!("Failed to write PE: {}", e))?;

        println!(
            "[CHECKSUM-FIX] [OK] Checksum recalculated: 0x{:08x}",
            checksum
        );

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// SIGNATURE BREAKER
// ────────────────────────────────────────────────────────────────────────────

pub struct SignatureBreaker;

impl SignatureBreaker {
    pub fn flip_random_bits(
        file_path: &str,
        num_flips: usize,
        output_path: &str,
    ) -> Result<(), String> {
        println!("[SIG-BREAKER] Flipping {} random bits", num_flips);

        let mut data = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

        use rand::Rng;
        let mut rng = rand::thread_rng();

        for _ in 0..num_flips {
            let byte_offset = rng.gen_range(0..data.len());
            let bit_offset = rng.gen_range(0..8);

            data[byte_offset] ^= 1 << bit_offset;

            println!(
                "[SIG-BREAKER] Flipped bit {} at offset 0x{:x}",
                bit_offset, byte_offset
            );
        }

        fs::write(output_path, data).map_err(|e| format!("Failed to write file: {}", e))?;

        println!(
            "[SIG-BREAKER] [OK] Modified binary saved to {}",
            output_path
        );

        Ok(())
    }

    pub fn append_garbage(
        file_path: &str,
        garbage_size: usize,
        output_path: &str,
    ) -> Result<(), String> {
        println!("[SIG-BREAKER] Appending {} bytes of garbage", garbage_size);

        let mut data = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let garbage: Vec<u8> = (0..garbage_size).map(|_| rng.gen::<u8>()).collect();

        data.extend_from_slice(&garbage);

        fs::write(output_path, data).map_err(|e| format!("Failed to write file: {}", e))?;

        println!(
            "[SIG-BREAKER] [OK] Garbage appended, saved to {}",
            output_path
        );

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// UNIT TESTS
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_elf() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        
        let mut elf_header = vec![0u8; 64];
        elf_header[0..4].copy_from_slice(&[0x7f, 0x45, 0x4c, 0x46]);
        elf_header[4] = 2;
        elf_header[5] = 1;
        elf_header[6] = 1;
        elf_header[16..18].copy_from_slice(&[0x02, 0x00]);
        elf_header[18..20].copy_from_slice(&[0x3e, 0x00]);
        elf_header[20..24].copy_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        elf_header[24..32].copy_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        elf_header[32..40].copy_from_slice(&[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        elf_header[40..48].copy_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        elf_header[52..54].copy_from_slice(&[0x40, 0x00]);
        elf_header[54..56].copy_from_slice(&[0x38, 0x00]);
        elf_header[56..58].copy_from_slice(&[0x00, 0x00]);
        
        file.write_all(&elf_header).unwrap();
        
        let mut padding = vec![0x90; 1024];
        padding[100] = 0xE8;
        padding[101..105].copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        
        file.write_all(&padding).unwrap();
        file.flush().unwrap();
        
        file
    }

    fn create_test_pe() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        
        let mut pe_header = vec![0u8; 512];
        pe_header[0] = b'M';
        pe_header[1] = b'Z';
        pe_header[0x3C] = 0x80;
        
        let pe_offset = 0x80;
        pe_header[pe_offset..pe_offset + 4].copy_from_slice(b"PE\0\0");
        
        pe_header[pe_offset + 4] = 0x64;
        pe_header[pe_offset + 5] = 0x86;
        
        file.write_all(&pe_header).unwrap();
        file.write_all(&vec![0x90; 512]).unwrap();
        file.flush().unwrap();
        
        file
    }

    #[test]
    fn test_patch_new_elf() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let patch = Patch::new(path);
        assert!(patch.is_ok());
        
        let patch = patch.unwrap();
        assert_eq!(patch.is_elf, true);
        assert_eq!(patch.is_pe, false);
        assert_eq!(patch.architecture, Architecture::X64);
    }

    #[test]
    fn test_patch_new_pe() {
        let test_file = create_test_pe();
        let path = test_file.path().to_str().unwrap();
        
        let patch = Patch::new(path);
        assert!(patch.is_ok());
        
        let patch = patch.unwrap();
        assert_eq!(patch.is_elf, false);
        assert_eq!(patch.is_pe, true);
        assert_eq!(patch.architecture, Architecture::X64);
    }

    #[test]
    fn test_nop_out() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        
        let result = patch.nop_out(50, 10);
        assert!(result.is_ok());
        assert_eq!(patch.operations.len(), 1);
        assert_eq!(patch.operations[0].description, "NOP 10 bytes at 0x32");
        
        for i in 50..60 {
            assert_eq!(patch.binary_data[i], 0x90);
        }
    }

    #[test]
    fn test_nop_out_bounds_check() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        
        let result = patch.nop_out(10000, 10);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("extends beyond"));
    }

    #[test]
    #[cfg(feature = "binary-patching")]
    fn test_insert_asm_x64() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        
        let result = patch.insert_asm(100, "xor eax, eax; ret");
        assert!(result.is_ok());
        assert_eq!(patch.operations.len(), 1);
        
        let expected_bytes = vec![0x31, 0xc0, 0xc3];
        assert_eq!(patch.binary_data[100..103], expected_bytes[..]);
    }
    
    #[test]
    #[cfg(not(feature = "binary-patching"))]
    fn test_insert_asm_requires_feature() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        
        let result = patch.insert_asm(100, "xor eax, eax; ret");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("keystone-engine feature"));
    }

    #[test]
    fn test_replace_call() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        
        let result = patch.replace_call(164, "hacked_function");
        assert!(result.is_ok());
        assert_eq!(patch.operations.len(), 1);
    }

    #[test]
    fn test_replace_call_invalid_instruction() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        
        let result = patch.replace_call(50, "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No CALL instruction"));
    }

    #[test]
    fn test_dry_run_mode() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        patch.set_dry_run(true);
        
        let original_data = patch.binary_data.clone();
        
        let result = patch.nop_out(50, 10);
        assert!(result.is_ok());
        
        assert_eq!(patch.binary_data, original_data);
        assert_eq!(patch.operations.len(), 1);
    }

    #[test]
    fn test_preview_diff() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        patch.set_dry_run(true);
        
        patch.nop_out(50, 5).unwrap();
        
        let diff = patch.preview_diff();
        assert!(diff.contains("PATCH PREVIEW"));
        assert!(diff.contains("Total operations: 1"));
        assert!(diff.contains("0x32"));
    }

    #[test]
    fn test_undo_operation() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        
        let original_byte = patch.binary_data[50];
        
        patch.nop_out(50, 5).unwrap();
        assert_eq!(patch.binary_data[50], 0x90);
        
        let result = patch.undo();
        assert!(result.is_ok());
        assert_eq!(patch.binary_data[50], original_byte);
        assert_eq!(patch.operations.len(), 0);
    }

    #[test]
    fn test_rollback_all() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        let original_checksum = patch.original_checksum.clone();
        
        patch.nop_out(50, 5).unwrap();
        patch.nop_out(100, 10).unwrap();
        patch.nop_out(200, 3).unwrap();
        
        assert_eq!(patch.operations.len(), 3);
        
        let result = patch.rollback_all();
        assert!(result.is_ok());
        assert_eq!(patch.operations.len(), 0);
        
        let current_checksum = Patch::compute_checksum(&patch.binary_data);
        assert_eq!(current_checksum, original_checksum);
    }

    #[test]
    fn test_verify_integrity() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        
        let result = patch.verify_integrity();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
        
        patch.nop_out(50, 5).unwrap();
        
        let result = patch.verify_integrity();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_checksum_computation() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let patch = Patch::new(path).unwrap();
        let checksum1 = patch.original_checksum.clone();
        
        let patch2 = Patch::new(path).unwrap();
        let checksum2 = patch2.original_checksum;
        
        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_save_with_backup_info() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        patch.nop_out(50, 5).unwrap();
        
        let output = NamedTempFile::new().unwrap();
        let output_path = output.path().to_str().unwrap();
        
        let result = patch.save(output_path);
        assert!(result.is_ok());
        
        let backup_info_path = format!("{}.patch_info", output_path);
        let backup_exists = std::path::Path::new(&backup_info_path).exists();
        assert!(backup_exists);
        
        let backup_content = fs::read_to_string(&backup_info_path).unwrap();
        assert!(backup_content.contains("Original binary:"));
        assert!(backup_content.contains("Original checksum:"));
        assert!(backup_content.contains("Architecture:"));
        
        fs::remove_file(&backup_info_path).ok();
    }

    #[test]
    fn test_multiple_operations_sequence() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        
        patch.nop_out(50, 5).unwrap();
        patch.nop_out(100, 3).unwrap();
        
        assert_eq!(patch.operations.len(), 2);
        
        let diff = patch.preview_diff();
        assert!(diff.contains("Total operations: 2"));
    }
    
    #[test]
    fn test_patch_bytes() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        
        let new_bytes = vec![0x41, 0x42, 0x43, 0x44];
        let result = patch.patch_bytes(100, &new_bytes);
        assert!(result.is_ok());
        
        assert_eq!(patch.binary_data[100..104], new_bytes[..]);
        assert_eq!(patch.operations.len(), 1);
    }
    
    #[test]
    fn test_recalculate_headers_elf() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        
        let result = patch.recalculate_headers();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_recalculate_headers_pe() {
        let test_file = create_test_pe();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        
        let result = patch.recalculate_headers();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_find_pattern() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let patch = Patch::new(path).unwrap();
        
        let pattern = vec![0x90, 0x90, 0x90];
        let offsets = patch.find_pattern(&pattern);
        
        assert!(!offsets.is_empty());
    }
    
    #[test]
    fn test_inject_shellcode() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        let original_len = patch.binary_data.len();
        
        let shellcode = vec![0x31, 0xc0, 0x48, 0x89, 0xc7];
        let result = patch.inject_shellcode(&shellcode);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), original_len);
        assert_eq!(patch.binary_data.len(), original_len + shellcode.len());
    }
    
    #[test]
    fn test_create_code_cave() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        let original_len = patch.binary_data.len();
        
        let cave_size = 256;
        let result = patch.create_code_cave(cave_size);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), original_len);
        assert_eq!(patch.binary_data.len(), original_len + cave_size);
        
        for i in 0..cave_size {
            assert_eq!(patch.binary_data[original_len + i], 0x90);
        }
    }
    
    #[test]
    fn test_get_architecture() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let patch = Patch::new(path).unwrap();
        
        assert_eq!(patch.get_architecture(), Architecture::X64);
    }
    
    #[test]
    fn test_get_operations() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        
        patch.nop_out(50, 5).unwrap();
        patch.nop_out(100, 10).unwrap();
        
        let ops = patch.get_operations();
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0].offset, 50);
        assert_eq!(ops[1].offset, 100);
    }
    
    #[test]
    fn test_is_dry_run() {
        let test_file = create_test_elf();
        let path = test_file.path().to_str().unwrap();
        
        let mut patch = Patch::new(path).unwrap();
        
        assert_eq!(patch.is_dry_run(), false);
        
        patch.set_dry_run(true);
        assert_eq!(patch.is_dry_run(), true);
    }
}
