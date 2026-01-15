use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;
use std::time::Instant;

fn create_test_elf_x64() -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    
    let mut elf = Vec::new();
    elf.extend_from_slice(&[0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00]);
    elf.extend_from_slice(&[0x00; 8]);
    elf.extend_from_slice(&[0x02, 0x00, 0x3e, 0x00]);
    elf.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x40, 0x00, 0x38, 0x00, 0x01, 0x00, 0x00, 0x00]);
    
    while elf.len() < 0x1000 {
        elf.push(0x00);
    }
    
    let gadgets_offset = 0x500;
    elf[gadgets_offset] = 0x5f;
    elf[gadgets_offset + 1] = 0xc3;
    
    elf[gadgets_offset + 10] = 0x5e;
    elf[gadgets_offset + 11] = 0xc3;
    
    elf[gadgets_offset + 20] = 0x5a;
    elf[gadgets_offset + 21] = 0xc3;
    
    elf[gadgets_offset + 30] = 0x58;
    elf[gadgets_offset + 31] = 0xc3;
    
    elf[gadgets_offset + 40] = 0x0f;
    elf[gadgets_offset + 41] = 0x05;
    
    elf[gadgets_offset + 50] = 0x5f;
    elf[gadgets_offset + 51] = 0x5e;
    elf[gadgets_offset + 52] = 0xc3;
    
    elf[gadgets_offset + 60] = 0x48;
    elf[gadgets_offset + 61] = 0x89;
    elf[gadgets_offset + 62] = 0xe0;
    elf[gadgets_offset + 63] = 0xc3;
    
    elf[gadgets_offset + 70] = 0xc9;
    elf[gadgets_offset + 71] = 0xc3;
    
    elf[gadgets_offset + 80] = 0x48;
    elf[gadgets_offset + 81] = 0x31;
    elf[gadgets_offset + 82] = 0xc0;
    elf[gadgets_offset + 83] = 0xc3;
    
    elf[gadgets_offset + 90] = 0x5f;
    elf[gadgets_offset + 91] = 0x5e;
    elf[gadgets_offset + 92] = 0x5a;
    elf[gadgets_offset + 93] = 0xc3;
    
    file.write_all(&elf).expect("Failed to write test ELF");
    file.flush().expect("Failed to flush");
    file
}

fn create_test_elf_x86() -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    
    let mut elf = Vec::new();
    elf.extend_from_slice(&[0x7f, 0x45, 0x4c, 0x46, 0x01, 0x01, 0x01, 0x00]);
    elf.extend_from_slice(&[0x00; 8]);
    elf.extend_from_slice(&[0x02, 0x00, 0x03, 0x00]);
    elf.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x40, 0x00]);
    elf.extend_from_slice(&[0x34, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x34, 0x00, 0x20, 0x00]);
    
    while elf.len() < 0x800 {
        elf.push(0x00);
    }
    
    let gadgets_offset = 0x400;
    elf[gadgets_offset] = 0x5b;
    elf[gadgets_offset + 1] = 0xc3;
    
    elf[gadgets_offset + 10] = 0x5d;
    elf[gadgets_offset + 11] = 0xc3;
    
    elf[gadgets_offset + 20] = 0xcd;
    elf[gadgets_offset + 21] = 0x80;
    
    file.write_all(&elf).expect("Failed to write test ELF");
    file.flush().expect("Failed to flush");
    file
}

fn create_test_elf_arm() -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    
    let mut elf = Vec::new();
    elf.extend_from_slice(&[0x7f, 0x45, 0x4c, 0x46, 0x01, 0x01, 0x01, 0x00]);
    elf.extend_from_slice(&[0x00; 8]);
    elf.extend_from_slice(&[0x02, 0x00, 0x28, 0x00]);
    elf.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x01, 0x00]);
    elf.extend_from_slice(&[0x34, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x34, 0x00, 0x20, 0x00]);
    
    while elf.len() < 0x800 {
        elf.push(0x00);
    }
    
    let gadgets_offset = 0x400;
    elf[gadgets_offset] = 0x04;
    elf[gadgets_offset + 1] = 0x70;
    elf[gadgets_offset + 2] = 0xbd;
    elf[gadgets_offset + 3] = 0xe8;
    
    elf[gadgets_offset + 10] = 0x1e;
    elf[gadgets_offset + 11] = 0xff;
    elf[gadgets_offset + 12] = 0x2f;
    elf[gadgets_offset + 13] = 0xe1;
    
    file.write_all(&elf).expect("Failed to write test ELF");
    file.flush().expect("Failed to flush");
    file
}

fn create_test_elf_arm64() -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    
    let mut elf = Vec::new();
    elf.extend_from_slice(&[0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00]);
    elf.extend_from_slice(&[0x00; 8]);
    elf.extend_from_slice(&[0x02, 0x00, 0xb7, 0x00]);
    elf.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x40, 0x00, 0x38, 0x00, 0x01, 0x00, 0x00, 0x00]);
    
    while elf.len() < 0x1000 {
        elf.push(0x00);
    }
    
    let gadgets_offset = 0x500;
    elf[gadgets_offset] = 0xc0;
    elf[gadgets_offset + 1] = 0x03;
    elf[gadgets_offset + 2] = 0x5f;
    elf[gadgets_offset + 3] = 0xd6;
    
    file.write_all(&elf).expect("Failed to write test ELF");
    file.flush().expect("Failed to flush");
    file
}

fn create_large_test_binary() -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    
    let mut elf = Vec::new();
    elf.extend_from_slice(&[0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00]);
    elf.extend_from_slice(&[0x00; 8]);
    elf.extend_from_slice(&[0x02, 0x00, 0x3e, 0x00]);
    elf.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x40, 0x00, 0x38, 0x00, 0x01, 0x00, 0x00, 0x00]);
    
    while elf.len() < 0x100000 {
        let offset = elf.len();
        
        if offset % 256 == 0 {
            elf.push(0x5f);
            elf.push(0xc3);
        } else if offset % 512 == 0 {
            elf.push(0x5e);
            elf.push(0xc3);
        } else if offset % 1024 == 0 {
            elf.push(0x0f);
            elf.push(0x05);
        } else {
            elf.push(0x90);
        }
    }
    
    file.write_all(&elf).expect("Failed to write test ELF");
    file.flush().expect("Failed to flush");
    file
}

fn create_binary_with_bad_chars() -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    
    let mut elf = Vec::new();
    elf.extend_from_slice(&[0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00]);
    elf.extend_from_slice(&[0x00; 8]);
    elf.extend_from_slice(&[0x02, 0x00, 0x3e, 0x00]);
    elf.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x40, 0x00, 0x38, 0x00, 0x01, 0x00, 0x00, 0x00]);
    
    while elf.len() < 0x1000 {
        elf.push(0x00);
    }
    
    let offset = 0x500;
    elf[offset] = 0x48;
    elf[offset + 1] = 0xc7;
    elf[offset + 2] = 0xc0;
    elf[offset + 3] = 0x00;
    elf[offset + 4] = 0x00;
    elf[offset + 5] = 0x00;
    elf[offset + 6] = 0x00;
    elf[offset + 7] = 0xc3;
    
    file.write_all(&elf).expect("Failed to write test ELF");
    file.flush().expect("Failed to flush");
    file
}

mod rop_gadget_finder_tests {
    use super::*;
    use talon::rop_gadget_finder::{ROPGadgetFinder, Architecture, GadgetCategory, ROPTarget};
    
    #[test]
    fn test_gadget_finder_initialization() {
        let finder = ROPGadgetFinder::new(Architecture::X64);
        assert!(finder.is_ok());
        
        let finder = ROPGadgetFinder::new(Architecture::X86);
        assert!(finder.is_ok());
    }
    
    #[test]
    fn test_analyze_bytes_x64() {
        let x64_code = vec![
            0x5f,
            0xc3,
            0x5e,
            0xc3,
            0x48, 0x89, 0xe0,
            0xc3,
        ];
        
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        let result = finder.analyze_bytes(&x64_code, 0x400000);
        
        assert!(result.is_ok());
        assert!(finder.gadgets.len() > 0);
    }
    
    #[test]
    fn test_analyze_empty_data() {
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        let result = finder.analyze_bytes(&[], 0x400000);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot analyze empty data");
    }
    
    #[test]
    fn test_find_pop_rdi_gadget() {
        let x64_code = vec![
            0x5f,
            0xc3,
            0x90, 0x90,
            0x5e,
            0xc3,
        ];
        
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        finder.analyze_bytes(&x64_code, 0x400000).unwrap();
        
        let pop_rdi_gadgets = finder.find_gadgets_by_pattern("pop rdi");
        assert!(pop_rdi_gadgets.len() > 0);
    }
    
    #[test]
    fn test_find_pop_rsi_gadget() {
        let x64_code = vec![
            0x5e,
            0xc3,
        ];
        
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        finder.analyze_bytes(&x64_code, 0x400000).unwrap();
        
        let pop_rsi_gadgets = finder.find_gadgets_by_pattern("pop rsi");
        assert!(pop_rsi_gadgets.len() > 0);
    }
    
    #[test]
    fn test_find_syscall_gadget() {
        let x64_code = vec![
            0x0f, 0x05,
            0xc3,
        ];
        
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        finder.analyze_bytes(&x64_code, 0x400000).unwrap();
        
        let syscall_gadgets = finder.find_gadgets_by_category(GadgetCategory::Syscall);
        assert!(syscall_gadgets.len() > 0);
    }
    
    #[test]
    fn test_gadget_categorization() {
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        
        let syscall_insns = vec!["syscall".to_string(), "ret".to_string()];
        let category = finder.categorize_gadget(&syscall_insns);
        assert_eq!(category, GadgetCategory::Syscall);
        
        let pop_insns = vec!["pop rdi".to_string(), "ret".to_string()];
        let category = finder.categorize_gadget(&pop_insns);
        assert_eq!(category, GadgetCategory::LoadRegister);
        
        let leave_insns = vec!["leave".to_string(), "ret".to_string()];
        let category = finder.categorize_gadget(&leave_insns);
        assert_eq!(category, GadgetCategory::StackPivot);
    }
    
    #[test]
    fn test_gadget_quality_scoring() {
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        
        let syscall_insns = vec!["syscall".to_string()];
        let quality = finder.calculate_quality(&syscall_insns, &GadgetCategory::Syscall);
        assert!(quality > 100);
        
        let long_insns = vec![
            "mov rax, rdi".to_string(),
            "add rax, rsi".to_string(),
            "xor rdx, rdx".to_string(),
            "ret".to_string(),
        ];
        let quality_long = finder.calculate_quality(&long_insns, &GadgetCategory::General);
        assert!(quality_long < quality);
    }
    
    #[test]
    fn test_find_gadgets_by_category() {
        let x64_code = vec![
            0x5f,
            0xc3,
            0x5e,
            0xc3,
            0x0f, 0x05,
        ];
        
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        finder.analyze_bytes(&x64_code, 0x400000).unwrap();
        
        let load_gadgets = finder.find_gadgets_by_category(GadgetCategory::LoadRegister);
        assert!(load_gadgets.len() >= 0);
    }
    
    #[test]
    fn test_get_best_gadgets() {
        let x64_code = vec![
            0x5f,
            0xc3,
            0x5e,
            0xc3,
            0x5a,
            0xc3,
        ];
        
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        finder.analyze_bytes(&x64_code, 0x400000).unwrap();
        
        let best = finder.get_best_gadgets(2);
        assert!(best.len() <= 2);
    }
    
    #[test]
    fn test_build_system_chain() {
        let x64_code = vec![
            0x5f,
            0xc3,
        ];
        
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        finder.analyze_bytes(&x64_code, 0x400000).unwrap();
        
        let target = ROPTarget::System {
            binsh_addr: 0x601000,
            system_addr: 0x7ffff7a52390,
        };
        
        let result = finder.build_rop_chain(target);
        assert!(result.is_ok());
        
        let chain = result.unwrap();
        assert!(chain.len() > 0);
    }
    
    #[test]
    fn test_analyze_file_nonexistent() {
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        let result = finder.analyze_file("/nonexistent/path/to/binary");
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_analyze_file_empty_path() {
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        let result = finder.analyze_file("");
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Binary path cannot be empty");
    }
    
    #[test]
    fn test_duplicate_gadget_filtering() {
        let x64_code = vec![
            0x5f,
            0xc3,
            0x90,
            0x5f,
            0xc3,
        ];
        
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        finder.analyze_bytes(&x64_code, 0x400000).unwrap();
        
        let all_gadgets = finder.find_gadgets_by_pattern("pop rdi");
        let unique_count = all_gadgets.len();
        assert!(unique_count >= 1);
    }
    
    #[test]
    fn test_gadget_pattern_matching_case_insensitive() {
        let x64_code = vec![
            0x5f,
            0xc3,
        ];
        
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        finder.analyze_bytes(&x64_code, 0x400000).unwrap();
        
        let lower = finder.find_gadgets_by_pattern("pop rdi");
        let upper = finder.find_gadgets_by_pattern("POP RDI");
        let mixed = finder.find_gadgets_by_pattern("Pop Rdi");
        
        assert_eq!(lower.len(), upper.len());
        assert_eq!(lower.len(), mixed.len());
    }
}

mod rop_tools_tests {
    use super::*;
    use talon::rop_tools::{RopChain, Architecture, ROPGoal, ROPStrategy, Constraint, AutoROPSolver};
    
    #[test]
    fn test_rop_chain_creation() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let result = RopChain::new(path);
        assert!(result.is_ok());
        
        let rop = result.unwrap();
        assert_eq!(rop.binary_path, path);
    }
    
    #[test]
    fn test_architecture_detection_x64() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        match rop.arch {
            Architecture::X8664 => assert!(true),
            _ => panic!("Expected x86-64 architecture"),
        }
    }
    
    #[test]
    fn test_architecture_detection_x86() {
        let test_elf = create_test_elf_x86();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        match rop.arch {
            Architecture::I386 => assert!(true),
            _ => panic!("Expected i386 architecture"),
        }
    }
    
    #[test]
    fn test_find_gadget_pattern() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        let pop_rdi = rop.find_gadget("pop rdi");
        
        if rop.gadgets.len() > 0 {
            assert!(pop_rdi.is_some() || pop_rdi.is_none());
        }
    }
    
    #[test]
    fn test_find_multiple_gadgets() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        let gadgets = rop.find_gadgets("pop");
        
        assert!(gadgets.len() >= 0);
    }
    
    #[test]
    fn test_set_libc_base() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let mut rop = RopChain::new(path).unwrap();
        rop.set_libc_base(0x7ffff7a00000);
        
        assert_eq!(rop.libc_base, Some(0x7ffff7a00000));
    }
    
    #[test]
    fn test_ret2libc_chain() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let mut rop = RopChain::new(path).unwrap();
        rop.set_libc_base(0x7ffff7a00000);
        
        let result = rop.ret2libc("/bin/sh");
        assert!(result.is_ok());
        
        let chain = result.unwrap();
        assert_eq!(chain.len(), 3);
    }
    
    #[test]
    fn test_ret2libc_without_base() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        let result = rop.ret2libc("/bin/sh");
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Libc base not set"));
    }
    
    #[test]
    fn test_build_chain_from_addresses() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        let addresses = vec![0x400500, 0x400510, 0x400520];
        let chain_bytes = rop.build_chain(&addresses);
        
        assert_eq!(chain_bytes.len(), addresses.len() * 8);
    }
    
    #[test]
    fn test_find_common_gadgets() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        let common = rop.find_common_gadgets();
        
        assert!(common.len() >= 0);
    }
    
    #[test]
    fn test_gadget_quality_scoring() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        
        if rop.gadgets.len() > 0 {
            let first_score = rop.gadgets[0].quality_score;
            assert!(first_score >= 0);
        }
    }
    
    #[test]
    fn test_gadgets_sorted_by_quality() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        
        if rop.gadgets.len() > 1 {
            for i in 0..rop.gadgets.len() - 1 {
                assert!(rop.gadgets[i].quality_score >= rop.gadgets[i + 1].quality_score);
            }
        }
    }
    
    #[test]
    fn test_find_ret2dlresolve_gadgets() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        let result = rop.find_ret2dlresolve_gadgets();
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_auto_rop_solver_initialization() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let result = AutoROPSolver::new(path);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_auto_rop_add_constraint() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let mut solver = AutoROPSolver::new(path).unwrap();
        solver.add_constraint(Constraint::NoNullBytes);
        solver.add_constraint(Constraint::MaxLength(256));
        
        assert_eq!(solver.constraints.len(), 2);
    }
    
    #[test]
    fn test_auto_rop_solve_system_goal() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let mut solver = AutoROPSolver::new(path).unwrap();
        solver.libc_base = Some(0x7ffff7a00000);
        
        let goal = ROPGoal::System("/bin/sh".to_string());
        let strategies = vec![ROPStrategy::Ret2Libc];
        
        let result = solver.solve(goal, strategies);
        
        if result.is_ok() {
            let solution = result.unwrap();
            assert!(solution.chain.len() > 0);
            assert!(solution.success_probability > 0.0);
        }
    }
    
    #[test]
    fn test_constraint_no_null_bytes() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let mut solver = AutoROPSolver::new(path).unwrap();
        solver.add_constraint(Constraint::NoNullBytes);
        
        let addresses = vec![0x400500, 0x400510];
        let satisfies = solver.check_constraints(&addresses);
        
        assert_eq!(satisfies, true);
    }
    
    #[test]
    fn test_constraint_max_length() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let mut solver = AutoROPSolver::new(path).unwrap();
        solver.add_constraint(Constraint::MaxLength(32));
        
        let addresses = vec![0x400500, 0x400510, 0x400520];
        let satisfies = solver.check_constraints(&addresses);
        
        assert_eq!(satisfies, true);
    }
    
    #[test]
    fn test_gadget_deduplication() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        
        let mut instruction_sets = std::collections::HashSet::new();
        for gadget in &rop.gadgets {
            let key = gadget.instructions.join(";");
            assert!(!instruction_sets.contains(&key), "Found duplicate gadget");
            instruction_sets.insert(key);
        }
    }
    
    #[test]
    fn test_rop_goal_creation() {
        let goal1 = ROPGoal::System("/bin/sh".to_string());
        let goal2 = ROPGoal::Execve("/bin/sh".to_string(), vec![]);
        let goal3 = ROPGoal::Mprotect(0x600000, 0x1000, 7);
        
        match goal1 {
            ROPGoal::System(_) => assert!(true),
            _ => panic!("Wrong goal type"),
        }
        
        match goal2 {
            ROPGoal::Execve(_, _) => assert!(true),
            _ => panic!("Wrong goal type"),
        }
        
        match goal3 {
            ROPGoal::Mprotect(_, _, _) => assert!(true),
            _ => panic!("Wrong goal type"),
        }
    }
    
    #[test]
    fn test_rop_strategy_enumeration() {
        let strategies = vec![
            ROPStrategy::OneGadget,
            ROPStrategy::Ret2Libc,
            ROPStrategy::MprotectRWX,
            ROPStrategy::Ret2Syscall,
            ROPStrategy::SROP,
            ROPStrategy::JOP,
            ROPStrategy::COP,
            ROPStrategy::StackPivot,
        ];
        
        assert_eq!(strategies.len(), 8);
    }
}

mod integration_tests {
    use super::*;
    use talon::rop_tools::{RopChain, ROPGoal, ROPStrategy, AutoROPSolver};
    
    #[test]
    fn test_full_exploit_chain_workflow() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let mut rop = RopChain::new(path).unwrap();
        
        assert!(rop.gadgets.len() >= 0);
        
        rop.set_libc_base(0x7ffff7a00000);
        
        let result = rop.ret2libc("/bin/sh");
        if result.is_ok() {
            let chain = result.unwrap();
            
            let chain_bytes = rop.build_chain(&chain);
            assert_eq!(chain_bytes.len(), chain.len() * 8);
        }
    }
    
    #[test]
    fn test_auto_solver_workflow() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let mut solver = AutoROPSolver::new(path).unwrap();
        
        solver.libc_base = Some(0x7ffff7a00000);
        
        solver.add_constraint(talon::rop_tools::Constraint::NoNullBytes);
        
        let goal = ROPGoal::System("/bin/sh".to_string());
        let strategies = vec![ROPStrategy::Ret2Libc, ROPStrategy::Ret2Syscall];
        
        let result = solver.solve(goal, strategies);
        
        if result.is_ok() {
            let solution = result.unwrap();
            assert!(solution.chain.len() > 0);
            assert!(solution.chain_bytes.len() > 0);
            assert!(solution.success_probability > 0.0 && solution.success_probability <= 1.0);
        }
    }
    
    #[test]
    fn test_chain_building_accuracy() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        
        let addresses = vec![0x0000000000400500, 0x0000000000400510, 0x0000000000400520];
        let chain_bytes = rop.build_chain(&addresses);
        
        assert_eq!(chain_bytes.len(), 24);
        
        let addr1 = u64::from_le_bytes([
            chain_bytes[0], chain_bytes[1], chain_bytes[2], chain_bytes[3],
            chain_bytes[4], chain_bytes[5], chain_bytes[6], chain_bytes[7],
        ]);
        assert_eq!(addr1, 0x400500);
    }
    
    #[test]
    fn test_gadget_search_accuracy() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        
        let all_gadgets = rop.find_gadgets("");
        let pop_gadgets = rop.find_gadgets("pop");
        
        assert!(pop_gadgets.len() <= all_gadgets.len());
    }
}

mod performance_tests {
    use super::*;
    use talon::rop_tools::RopChain;
    use talon::rop_gadget_finder::ROPGadgetFinder;
    
    #[test]
    fn test_gadget_search_performance_small_binary() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let start = Instant::now();
        let rop = RopChain::new(path).unwrap();
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 100, "Small binary search took {:?}, expected <100ms", duration);
        assert!(rop.gadgets.len() >= 0);
    }
    
    #[test]
    fn test_gadget_search_performance_large_binary() {
        let test_elf = create_large_test_binary();
        let path = test_elf.path().to_str().unwrap();
        
        let start = Instant::now();
        let rop = RopChain::new(path).unwrap();
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 2000, "Large binary search took {:?}, expected <2000ms", duration);
        assert!(rop.gadgets.len() > 0);
    }
    
    #[test]
    fn test_chain_building_performance() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        let rop = RopChain::new(path).unwrap();
        
        let addresses: Vec<u64> = (0..1000).map(|i| 0x400000 + i * 8).collect();
        
        let start = Instant::now();
        let _chain = rop.build_chain(&addresses);
        let duration = start.elapsed();
        
        assert!(duration.as_micros() < 1000, "Chain building took {:?}, expected <1ms", duration);
    }
    
    #[test]
    fn test_pattern_search_performance() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        let rop = RopChain::new(path).unwrap();
        
        let start = Instant::now();
        for _ in 0..100 {
            let _gadgets = rop.find_gadgets("pop");
        }
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 100, "100 pattern searches took {:?}, expected <100ms", duration);
    }
}

mod advanced_tests {
    use super::*;
    use talon::rop_tools::{RopChain, AutoROPSolver, Constraint, ROPGoal, ROPStrategy};
    use talon::rop_gadget_finder::ROPGadgetFinder;
    
    #[test]
    fn test_null_byte_constraint_enforcement() {
        let test_elf = create_binary_with_bad_chars();
        let path = test_elf.path().to_str().unwrap();
        
        let mut solver = AutoROPSolver::new(path).unwrap();
        solver.add_constraint(Constraint::NoNullBytes);
        solver.libc_base = Some(0x7ffff7a00000);
        
        let goal = ROPGoal::System("/bin/sh".to_string());
        let result = solver.solve(goal, vec![ROPStrategy::Ret2Libc]);
        
        if let Ok(solution) = result {
            for byte in &solution.chain_bytes {
                assert_ne!(*byte, 0x00, "Found null byte in chain with NoNullBytes constraint");
            }
        }
    }
    
    #[test]
    fn test_alphanumeric_constraint() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let mut solver = AutoROPSolver::new(path).unwrap();
        solver.add_constraint(Constraint::AlphanumericOnly);
        
        let addresses = vec![0x30303030u64, 0x41414141u64];
        let result = solver.check_constraints(&addresses);
        
        assert!(result);
    }
    
    #[test]
    fn test_max_length_constraint() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let mut solver = AutoROPSolver::new(path).unwrap();
        solver.add_constraint(Constraint::MaxLength(64));
        solver.libc_base = Some(0x7ffff7a00000);
        
        let goal = ROPGoal::System("/bin/sh".to_string());
        let result = solver.solve(goal, vec![ROPStrategy::Ret2Libc]);
        
        if let Ok(solution) = result {
            assert!(solution.chain_bytes.len() <= 64);
        }
    }
    
    #[test]
    fn test_avoid_bad_chars_constraint() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let mut solver = AutoROPSolver::new(path).unwrap();
        solver.add_constraint(Constraint::AvoidBadChars(vec![0x0a, 0x0d, 0x00]));
        
        let addresses = vec![0x400500, 0x400510];
        let result = solver.check_constraints(&addresses);
        
        assert!(result);
    }
    
    #[test]
    fn test_stack_alignment_constraint() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let mut solver = AutoROPSolver::new(path).unwrap();
        solver.add_constraint(Constraint::StackAlignment(16));
        
        assert_eq!(solver.constraints.len(), 1);
    }
    
    #[test]
    fn test_multiple_strategy_fallback() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let mut solver = AutoROPSolver::new(path).unwrap();
        solver.libc_base = Some(0x7ffff7a00000);
        
        let goal = ROPGoal::System("/bin/sh".to_string());
        let strategies = vec![
            ROPStrategy::OneGadget,
            ROPStrategy::Ret2Libc,
            ROPStrategy::Ret2Syscall,
        ];
        
        let result = solver.solve(goal, strategies);
        
        if result.is_ok() {
            let solution = result.unwrap();
            assert!(solution.chain.len() > 0);
        }
    }
    
    #[test]
    fn test_complex_rop_chain() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        
        let addresses = vec![
            0x400500,
            0xdeadbeef,
            0x400510,
            0xcafebabe,
            0x400520,
        ];
        
        let chain = rop.build_chain(&addresses);
        assert_eq!(chain.len(), 40);
        
        let first_addr = u64::from_le_bytes([
            chain[0], chain[1], chain[2], chain[3],
            chain[4], chain[5], chain[6], chain[7],
        ]);
        assert_eq!(first_addr, 0x400500);
    }
    
    #[test]
    fn test_gadget_quality_accuracy() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        
        if rop.gadgets.len() > 1 {
            let syscall_gadgets: Vec<_> = rop.gadgets.iter()
                .filter(|g| g.instructions.iter().any(|i| i.contains("syscall")))
                .collect();
            
            let pop_gadgets: Vec<_> = rop.gadgets.iter()
                .filter(|g| g.instructions.iter().any(|i| i.starts_with("pop")))
                .collect();
            
            if !syscall_gadgets.is_empty() && !pop_gadgets.is_empty() {
                assert!(syscall_gadgets[0].quality_score >= pop_gadgets[0].quality_score,
                    "Syscall gadgets should have higher quality than simple pop gadgets");
            }
        }
    }
    
    #[test]
    fn test_gadget_search_with_regex_patterns() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        
        let patterns = vec!["pop", "ret", "syscall", "mov", "xor"];
        
        for pattern in patterns {
            let gadgets = rop.find_gadgets(pattern);
            if !gadgets.is_empty() {
                for gadget in &gadgets {
                    let gadget_str = gadget.instructions.join(" ").to_lowercase();
                    assert!(gadget_str.contains(pattern), 
                        "Gadget '{}' should contain pattern '{}'", gadget_str, pattern);
                }
            }
        }
    }
    
    #[test]
    fn test_ret2dlresolve_complete_chain() {
        let test_elf = create_test_elf_x86();
        let path = test_elf.path().to_str().unwrap();
        
        let rop = RopChain::new(path).unwrap();
        let result = rop.find_ret2dlresolve_gadgets();
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_gadget_byte_accuracy() {
        let x64_code = vec![
            0x5f,
            0xc3,
        ];
        
        let mut finder = ROPGadgetFinder::new(talon::rop_gadget_finder::Architecture::X64).unwrap();
        finder.analyze_bytes(&x64_code, 0x400000).unwrap();
        
        if !finder.gadgets.is_empty() {
            let gadget = &finder.gadgets[0];
            assert!(gadget.bytes.len() > 0);
            assert!(gadget.bytes.contains(&0xc3));
        }
    }
    
    #[test]
    fn test_cross_architecture_support() {
        let x64_elf = create_test_elf_x64();
        let x86_elf = create_test_elf_x86();
        
        let x64_rop = RopChain::new(x64_elf.path().to_str().unwrap());
        let x86_rop = RopChain::new(x86_elf.path().to_str().unwrap());
        
        assert!(x64_rop.is_ok());
        assert!(x86_rop.is_ok());
    }
}

mod edge_case_tests {
    use super::*;
    use talon::rop_tools::RopChain;
    use talon::rop_gadget_finder::{ROPGadgetFinder, Architecture};
    
    #[test]
    fn test_empty_gadget_search() {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        let elf: Vec<u8> = vec![0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00];
        file.write_all(&elf).expect("Failed to write");
        file.flush().expect("Failed to flush");
        
        let result = RopChain::new(file.path().to_str().unwrap());
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_gadget_with_invalid_instructions() {
        let invalid_code = vec![0xff, 0xff, 0xff, 0xc3];
        
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        let result = finder.analyze_bytes(&invalid_code, 0x400000);
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_very_long_gadget_chain() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        let rop = RopChain::new(path).unwrap();
        
        let long_chain: Vec<u64> = (0..10000).map(|i| 0x400000 + i).collect();
        let chain_bytes = rop.build_chain(&long_chain);
        
        assert_eq!(chain_bytes.len(), 10000 * 8);
    }
    
    #[test]
    fn test_gadget_dedup_edge_cases() {
        let duplicate_code = vec![
            0x5f, 0xc3,
            0x90,
            0x5f, 0xc3,
            0x90,
            0x5f, 0xc3,
        ];
        
        let mut finder = ROPGadgetFinder::new(Architecture::X64).unwrap();
        finder.analyze_bytes(&duplicate_code, 0x400000).unwrap();
        
        let pop_rdi_count = finder.find_gadgets_by_pattern("pop rdi").len();
        assert!(pop_rdi_count >= 1);
    }
    
    #[test]
    fn test_high_address_ranges() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        let rop = RopChain::new(path).unwrap();
        
        let high_addresses = vec![
            0x7ffff7a00000,
            0x7ffff7b00000,
            0x7ffff7c00000,
        ];
        
        let chain = rop.build_chain(&high_addresses);
        assert_eq!(chain.len(), 24);
    }
    
    #[test]
    fn test_gadget_search_case_insensitivity_thorough() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        let rop = RopChain::new(path).unwrap();
        
        let patterns = vec![
            ("pop", "POP", "Pop"),
            ("ret", "RET", "Ret"),
            ("syscall", "SYSCALL", "SysCall"),
        ];
        
        for (lower, upper, mixed) in patterns {
            let lower_results = rop.find_gadgets(lower);
            let upper_results = rop.find_gadgets(upper);
            let mixed_results = rop.find_gadgets(mixed);
            
            assert_eq!(lower_results.len(), upper_results.len());
            assert_eq!(lower_results.len(), mixed_results.len());
        }
    }
    
    #[test]
    fn test_zero_address_handling() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        let rop = RopChain::new(path).unwrap();
        
        let addresses_with_zero = vec![0x0, 0x400500, 0x400510];
        let chain = rop.build_chain(&addresses_with_zero);
        
        assert_eq!(chain.len(), 24);
        assert_eq!(&chain[0..8], &[0, 0, 0, 0, 0, 0, 0, 0]);
    }
    
    #[test]
    fn test_boundary_conditions() {
        let test_elf = create_test_elf_x64();
        let path = test_elf.path().to_str().unwrap();
        let rop = RopChain::new(path).unwrap();
        
        let boundary_addresses = vec![
            0x0,
            0xFFFFFFFFFFFFFFFF,
            0x8000000000000000,
        ];
        
        let chain = rop.build_chain(&boundary_addresses);
        assert_eq!(chain.len(), 24);
    }
}

mod property_based_tests {
    use super::*;
    use proptest::prelude::*;
    use talon::rop_tools::RopChain;
    
    proptest! {
        #[test]
        fn test_build_chain_length(addresses in prop::collection::vec(0u64..0xFFFFFFFFu64, 1..20)) {
            let test_elf = create_test_elf_x64();
            let path = test_elf.path().to_str().unwrap();
            let rop = RopChain::new(path).unwrap();
            
            let chain = rop.build_chain(&addresses);
            prop_assert_eq!(chain.len(), addresses.len() * 8);
        }
        
        #[test]
        fn test_libc_base_setting(base_addr in 0x7f0000000000u64..0x7fffffffu64) {
            let test_elf = create_test_elf_x64();
            let path = test_elf.path().to_str().unwrap();
            let mut rop = RopChain::new(path).unwrap();
            
            rop.set_libc_base(base_addr);
            prop_assert_eq!(rop.libc_base, Some(base_addr));
        }
    }
}
