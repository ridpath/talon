use std::fs;
use goblin::Object;

#[derive(Debug, Clone)]
pub struct BinaryInfo {
    pub path: String,
    pub arch: String,
    pub bits: u8,
    pub os: String,
    pub endian: String,
    pub protections: BinaryProtections,
    pub pie: bool,
    pub stripped: bool,
    pub has_debug: bool,
}

#[derive(Debug, Clone)]
pub struct BinaryProtections {
    pub nx: bool,
    pub canary: bool,
    pub relro: RelroType,
    pub aslr: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelroType {
    None,
    Partial,
    Full,
}

pub struct TargetDetector;

impl TargetDetector {
    pub fn analyze(path: &str) -> Result<BinaryInfo, String> {
        let data = fs::read(path)
            .map_err(|e| format!("Failed to read binary: {}", e))?;
        
        let obj = Object::parse(&data)
            .map_err(|e| format!("Failed to parse binary: {}", e))?;
        
        match obj {
            Object::Elf(elf) => Self::analyze_elf(path, &elf),
            Object::PE(pe) => Self::analyze_pe(path, &pe),
            Object::Mach(mach) => Self::analyze_mach(path, &mach),
            _ => Err("Unsupported binary format".to_string()),
        }
    }
    
    fn analyze_elf(path: &str, elf: &goblin::elf::Elf) -> Result<BinaryInfo, String> {
        let arch = match elf.header.e_machine {
            0x3E => "x86_64",
            0x03 => "x86",
            0xB7 => "aarch64",
            0x28 => "arm",
            0xF3 => "riscv",
            _ => "unknown",
        }.to_string();
        
        let bits = if elf.is_64 { 64 } else { 32 };
        
        let endian = if elf.little_endian { "little" } else { "big" }.to_string();
        
        let nx = elf.program_headers.iter().any(|ph| {
            ph.p_type == goblin::elf::program_header::PT_GNU_STACK &&
            (ph.p_flags & goblin::elf::program_header::PF_X) == 0
        });
        
        let canary = Self::check_canary_elf(elf);
        
        let relro = Self::check_relro_elf(elf);
        
        let pie = elf.header.e_type == goblin::elf::header::ET_DYN;
        
        let stripped = elf.syms.is_empty();
        
        let has_debug = elf.section_headers.iter().any(|sh| {
            if let Some(name) = elf.shdr_strtab.get_at(sh.sh_name) {
                name.starts_with(".debug")
            } else {
                false
            }
        });
        
        Ok(BinaryInfo {
            path: path.to_string(),
            arch,
            bits,
            os: "linux".to_string(),
            endian,
            protections: BinaryProtections {
                nx,
                canary,
                relro,
                aslr: pie,
            },
            pie,
            stripped,
            has_debug,
        })
    }
    
    fn analyze_pe(path: &str, pe: &goblin::pe::PE) -> Result<BinaryInfo, String> {
        let arch = if pe.is_64 { "x86_64" } else { "x86" }.to_string();
        let bits = if pe.is_64 { 64 } else { 32 };
        
        let nx = pe.header.optional_header
            .map(|oh| {
                (oh.windows_fields.dll_characteristics & 0x0100) != 0
            })
            .unwrap_or(false);
        
        let aslr = pe.header.optional_header
            .map(|oh| {
                (oh.windows_fields.dll_characteristics & 0x0040) != 0
            })
            .unwrap_or(false);
        
        Ok(BinaryInfo {
            path: path.to_string(),
            arch,
            bits,
            os: "windows".to_string(),
            endian: "little".to_string(),
            protections: BinaryProtections {
                nx,
                canary: false,
                relro: RelroType::None,
                aslr,
            },
            pie: aslr,
            stripped: true,
            has_debug: false,
        })
    }
    
    fn analyze_mach(path: &str, _mach: &goblin::mach::Mach) -> Result<BinaryInfo, String> {
        Ok(BinaryInfo {
            path: path.to_string(),
            arch: "x86_64".to_string(),
            bits: 64,
            os: "macos".to_string(),
            endian: "little".to_string(),
            protections: BinaryProtections {
                nx: true,
                canary: false,
                relro: RelroType::None,
                aslr: true,
            },
            pie: true,
            stripped: true,
            has_debug: false,
        })
    }
    
    fn check_canary_elf(elf: &goblin::elf::Elf) -> bool {
        elf.dynsyms.iter().any(|sym| {
            if let Some(name) = elf.dynstrtab.get_at(sym.st_name) {
                name.contains("stack_chk")
            } else {
                false
            }
        })
    }
    
    fn check_relro_elf(elf: &goblin::elf::Elf) -> RelroType {
        let has_relro = elf.program_headers.iter().any(|ph| {
            ph.p_type == goblin::elf::program_header::PT_GNU_RELRO
        });
        
        if !has_relro {
            return RelroType::None;
        }
        
        let has_bind_now = elf.dynamic.as_ref()
            .map(|dyn_info| {
                dyn_info.info.flags_1 & goblin::elf::dynamic::DF_1_NOW != 0
            })
            .unwrap_or(false);
        
        if has_bind_now {
            RelroType::Full
        } else {
            RelroType::Partial
        }
    }
    
    pub fn print_analysis(info: &BinaryInfo) {
        println!("Binary Analysis: {}", info.path);
        println!("  Architecture: {} ({}-bit)", info.arch, info.bits);
        println!("  OS: {}", info.os);
        println!("  Endianness: {}", info.endian);
        println!("  PIE: {}", if info.pie { "Enabled" } else { "Disabled" });
        println!("  Stripped: {}", if info.stripped { "Yes" } else { "No" });
        println!("  Debug Info: {}", if info.has_debug { "Present" } else { "Absent" });
        println!("\nSecurity Protections:");
        println!("  NX: {}", if info.protections.nx { "Enabled" } else { "Disabled" });
        println!("  Canary: {}", if info.protections.canary { "Enabled" } else { "Disabled" });
        println!("  RELRO: {:?}", info.protections.relro);
        println!("  ASLR: {}", if info.protections.aslr { "Enabled" } else { "Disabled" });
    }
    
    pub fn check_system_aslr() -> bool {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = fs::read_to_string("/proc/sys/kernel/randomize_va_space") {
                content.trim() != "0"
            } else {
                true
            }
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            true
        }
    }
}
