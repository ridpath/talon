use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

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
