use std::fs;

use tempfile::TempDir;

mod elf_tools_tests {
    use super::*;

    fn create_test_elf_with_protections(nx: bool, pie: bool, canary: bool, relro: bool) -> Vec<u8> {
        let mut elf = Vec::new();

        elf.extend_from_slice(&[0x7f, 0x45, 0x4c, 0x46]);
        elf.push(0x02);
        elf.push(0x01);
        elf.push(0x01);
        elf.push(0x00);
        elf.extend_from_slice(&[0x00; 8]);

        if pie {
            elf.extend_from_slice(&[0x03, 0x00]);
        } else {
            elf.extend_from_slice(&[0x02, 0x00]);
        }

        elf.extend_from_slice(&[0x3e, 0x00]);
        elf.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);

        let entry_point: u64 = 0x400080;
        elf.extend_from_slice(&entry_point.to_le_bytes());

        let phoff: u64 = 0x40;
        elf.extend_from_slice(&phoff.to_le_bytes());

        let shoff: u64 = 0x1000;
        elf.extend_from_slice(&shoff.to_le_bytes());

        elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        elf.extend_from_slice(&[0x40, 0x00]);
        elf.extend_from_slice(&[0x38, 0x00]);
        elf.extend_from_slice(&[0x03, 0x00]);
        elf.extend_from_slice(&[0x40, 0x00]);
        elf.extend_from_slice(&[0x05, 0x00]);
        elf.extend_from_slice(&[0x04, 0x00]);

        while elf.len() < phoff as usize {
            elf.push(0x00);
        }

        if relro {
            elf.extend_from_slice(&[0x52, 0x45, 0x4c, 0x52]);
            elf.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
            elf.extend_from_slice(&[0x04, 0x00, 0x00, 0x00]);
            elf.extend_from_slice(&[0x00; 32]);
        }

        elf.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        let flags = if nx { 0x05 } else { 0x07 };
        elf.extend_from_slice(&[flags, 0x00, 0x00, 0x00]);
        elf.extend_from_slice(&[0x00; 32]);

        elf.extend_from_slice(&[0x51, 0xe5, 0x74, 0x64]);
        let stack_flags = if nx { 0x06 } else { 0x07 };
        elf.extend_from_slice(&[stack_flags, 0x00, 0x00, 0x00]);
        elf.extend_from_slice(&[0x00; 32]);

        while elf.len() < 0x200 {
            elf.push(0x90);
        }

        if canary {
            elf.extend_from_slice(b"__stack_chk_fail\0");
        }

        while elf.len() < 0x400 {
            elf.push(0x00);
        }

        elf.extend_from_slice(b".text\0");
        elf.extend_from_slice(b".data\0");
        elf.extend_from_slice(b".bss\0");
        elf.extend_from_slice(b".got\0");
        elf.extend_from_slice(b".plt\0");

        while elf.len() < shoff as usize {
            elf.push(0x00);
        }

        for _ in 0..5 {
            elf.extend_from_slice(&[0x00; 64]);
        }

        elf
    }

    #[test]
    fn test_elf_basic_loading() {
        let temp_dir = TempDir::new().unwrap();
        let elf_path = temp_dir.path().join("test.elf");

        let elf_data = create_test_elf_with_protections(true, false, false, false);
        fs::write(&elf_path, elf_data).unwrap();

        assert!(elf_path.exists());
        let data = fs::read(&elf_path).unwrap();
        assert_eq!(&data[0..4], &[0x7f, 0x45, 0x4c, 0x46]);
    }

    #[test]
    fn test_elf_protection_detection_nx_enabled() {
        let temp_dir = TempDir::new().unwrap();
        let elf_path = temp_dir.path().join("nx_enabled.elf");

        let elf_data = create_test_elf_with_protections(true, false, false, false);
        fs::write(&elf_path, elf_data).unwrap();

        assert!(elf_path.exists());
    }

    #[test]
    fn test_elf_protection_detection_pie_enabled() {
        let temp_dir = TempDir::new().unwrap();
        let elf_path = temp_dir.path().join("pie_enabled.elf");

        let elf_data = create_test_elf_with_protections(false, true, false, false);
        fs::write(&elf_path, elf_data).unwrap();

        assert!(elf_path.exists());
    }

    #[test]
    fn test_elf_protection_detection_all_enabled() {
        let temp_dir = TempDir::new().unwrap();
        let elf_path = temp_dir.path().join("all_protections.elf");

        let elf_data = create_test_elf_with_protections(true, true, true, true);
        fs::write(&elf_path, elf_data).unwrap();

        assert!(elf_path.exists());
        let data = fs::read(&elf_path).unwrap();
        assert!(data.windows(17).any(|w| w == b"__stack_chk_fail\0"));
    }

    #[test]
    fn test_elf_protection_detection_none() {
        let temp_dir = TempDir::new().unwrap();
        let elf_path = temp_dir.path().join("no_protections.elf");

        let elf_data = create_test_elf_with_protections(false, false, false, false);
        fs::write(&elf_path, elf_data).unwrap();

        assert!(elf_path.exists());
    }

    #[test]
    fn test_elf_header_validation() {
        let temp_dir = TempDir::new().unwrap();

        let invalid_path = temp_dir.path().join("invalid.bin");
        fs::write(&invalid_path, b"NOT AN ELF FILE").unwrap();

        let data = fs::read(&invalid_path).unwrap();
        assert_ne!(&data[0..4], &[0x7f, 0x45, 0x4c, 0x46]);
    }

    #[test]
    fn test_elf_64bit_detection() {
        let temp_dir = TempDir::new().unwrap();
        let elf_path = temp_dir.path().join("test64.elf");

        let elf_data = create_test_elf_with_protections(true, false, false, false);
        fs::write(&elf_path, elf_data).unwrap();

        let data = fs::read(&elf_path).unwrap();
        assert_eq!(data[4], 0x02);
    }

    #[test]
    fn test_elf_architecture_x86_64() {
        let temp_dir = TempDir::new().unwrap();
        let elf_path = temp_dir.path().join("x86_64.elf");

        let elf_data = create_test_elf_with_protections(true, false, false, false);
        fs::write(&elf_path, elf_data).unwrap();

        let data = fs::read(&elf_path).unwrap();
        assert_eq!(data[18], 0x3e);
    }
}

mod binary_analyzer_tests {

    #[test]
    fn test_binary_protections_struct() {
        use talon::binary_analyzer::{BinaryProtections, RelroLevel};

        let protections = BinaryProtections {
            nx: true,
            pie: false,
            relro: RelroLevel::Full,
            canary: true,
            aslr: false,
            fortify: false,
        };

        assert!(protections.nx);
        assert!(!protections.pie);
        assert_eq!(protections.relro, RelroLevel::Full);
        assert!(protections.canary);
    }

    #[test]
    fn test_relro_levels() {
        use talon::binary_analyzer::RelroLevel;

        let none = RelroLevel::None;
        let partial = RelroLevel::Partial;
        let full = RelroLevel::Full;

        assert_eq!(none, RelroLevel::None);
        assert_eq!(partial, RelroLevel::Partial);
        assert_eq!(full, RelroLevel::Full);
        assert_ne!(none, full);
    }

    #[test]
    fn test_section_structure() {
        use talon::binary_analyzer::Section;

        let section = Section {
            name: ".text".to_string(),
            address: 0x400000,
            size: 0x1000,
            permissions: "rx".to_string(),
            is_writable: false,
            is_executable: true,
        };

        assert_eq!(section.name, ".text");
        assert_eq!(section.address, 0x400000);
        assert!(!section.is_writable);
        assert!(section.is_executable);
    }

    #[test]
    fn test_symbol_structure() {
        use talon::binary_analyzer::Symbol;

        let symbol = Symbol {
            name: "main".to_string(),
            address: 0x401000,
            symbol_type: "FUNC".to_string(),
            is_imported: false,
        };

        assert_eq!(symbol.name, "main");
        assert_eq!(symbol.address, 0x401000);
        assert!(!symbol.is_imported);
    }

    #[test]
    fn test_dangerous_function_detection() {
        use talon::binary_analyzer::{BinaryAnalyzer, Symbol};

        let symbols = vec![
            Symbol {
                name: "strcpy".to_string(),
                address: 0x400100,
                symbol_type: "FUNC".to_string(),
                is_imported: true,
            },
            Symbol {
                name: "gets".to_string(),
                address: 0x400200,
                symbol_type: "FUNC".to_string(),
                is_imported: true,
            },
            Symbol {
                name: "safe_function".to_string(),
                address: 0x400300,
                symbol_type: "FUNC".to_string(),
                is_imported: false,
            },
        ];

        let dangerous = BinaryAnalyzer::find_dangerous_functions(&symbols);
        assert!(!dangerous.is_empty());
        assert!(dangerous.contains(&"strcpy".to_string()));
        assert!(dangerous.contains(&"gets".to_string()));
        assert!(!dangerous.contains(&"safe_function".to_string()));
    }

    #[test]
    fn test_interesting_function_detection() {
        use talon::binary_analyzer::{BinaryAnalyzer, Symbol};

        let symbols = vec![
            Symbol {
                name: "main".to_string(),
                address: 0x401000,
                symbol_type: "FUNC".to_string(),
                is_imported: false,
            },
            Symbol {
                name: "system".to_string(),
                address: 0x400500,
                symbol_type: "FUNC".to_string(),
                is_imported: true,
            },
            Symbol {
                name: "random_func".to_string(),
                address: 0x400600,
                symbol_type: "FUNC".to_string(),
                is_imported: false,
            },
        ];

        let interesting = BinaryAnalyzer::find_interesting_functions(&symbols);
        assert!(interesting.contains(&"main".to_string()));
        assert!(interesting.contains(&"system".to_string()));
    }

    #[test]
    fn test_writable_section_detection() {
        use talon::binary_analyzer::{BinaryAnalyzer, Section};

        let sections = vec![
            Section {
                name: ".text".to_string(),
                address: 0x400000,
                size: 0x1000,
                permissions: "rx".to_string(),
                is_writable: false,
                is_executable: true,
            },
            Section {
                name: ".data".to_string(),
                address: 0x601000,
                size: 0x100,
                permissions: "rw".to_string(),
                is_writable: true,
                is_executable: false,
            },
            Section {
                name: ".bss".to_string(),
                address: 0x602000,
                size: 0x200,
                permissions: "rw".to_string(),
                is_writable: true,
                is_executable: false,
            },
        ];

        let writable = BinaryAnalyzer::find_writable_sections(&sections);
        assert_eq!(writable.len(), 2);
        assert!(writable.contains(&".data".to_string()));
        assert!(writable.contains(&".bss".to_string()));
        assert!(!writable.contains(&".text".to_string()));
    }

    #[test]
    fn test_binary_analysis_structure() {
        use talon::binary_analyzer::{BinaryAnalysis, BinaryProtections, RelroLevel};

        let analysis = BinaryAnalysis {
            architecture: "x86_64".to_string(),
            os: "Linux".to_string(),
            bitness: 64,
            endianness: "little".to_string(),
            protections: BinaryProtections {
                nx: true,
                pie: true,
                relro: RelroLevel::Full,
                canary: true,
                aslr: true,
                fortify: true,
            },
            sections: vec![],
            symbols: vec![],
            entry_point: 0x400080,
            base_address: 0x400000,
        };

        assert_eq!(analysis.architecture, "x86_64");
        assert_eq!(analysis.bitness, 64);
        assert_eq!(analysis.entry_point, 0x400080);
        assert!(analysis.protections.nx);
        assert!(analysis.protections.pie);
    }
}

mod binary_patch_tests {
    use super::*;
    use talon::binary_patch::BinaryPatcher;

    #[test]
    fn test_patch_bytes_basic() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.bin");
        let output_path = temp_dir.path().join("output.bin");

        let original_data = vec![0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90];
        fs::write(&input_path, &original_data).unwrap();

        let patch = vec![0xCC, 0xCC];
        let result = BinaryPatcher::patch_bytes(
            input_path.to_str().unwrap(),
            2,
            &patch,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_ok());

        let patched_data = fs::read(&output_path).unwrap();
        assert_eq!(patched_data[2], 0xCC);
        assert_eq!(patched_data[3], 0xCC);
        assert_eq!(patched_data[0], 0x90);
    }

    #[test]
    fn test_patch_bytes_out_of_bounds() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.bin");
        let output_path = temp_dir.path().join("output.bin");

        let original_data = vec![0x90; 8];
        fs::write(&input_path, &original_data).unwrap();

        let patch = vec![0xCC; 20];
        let result = BinaryPatcher::patch_bytes(
            input_path.to_str().unwrap(),
            0,
            &patch,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("beyond file"));
    }

    #[test]
    fn test_nop_instructions() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.bin");
        let output_path = temp_dir.path().join("output.bin");

        let original_data = vec![0xCC, 0xCC, 0xCC, 0xCC, 0xCC];
        fs::write(&input_path, &original_data).unwrap();

        let result = BinaryPatcher::nop_instructions(
            input_path.to_str().unwrap(),
            1,
            3,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_ok());

        let patched_data = fs::read(&output_path).unwrap();
        assert_eq!(patched_data[0], 0xCC);
        assert_eq!(patched_data[1], 0x90);
        assert_eq!(patched_data[2], 0x90);
        assert_eq!(patched_data[3], 0x90);
        assert_eq!(patched_data[4], 0xCC);
    }

    #[test]
    fn test_replace_call_instruction() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.bin");
        let output_path = temp_dir.path().join("output.bin");

        let original_data = vec![0x90, 0xE8, 0x00, 0x00, 0x00, 0x00, 0x90];
        fs::write(&input_path, &original_data).unwrap();

        let new_target: u32 = 0xDEADBEEF;
        let result = BinaryPatcher::replace_call(
            input_path.to_str().unwrap(),
            1,
            new_target,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_ok());

        let patched_data = fs::read(&output_path).unwrap();
        assert_eq!(patched_data[1], 0xE8);
        let target = u32::from_le_bytes([
            patched_data[2],
            patched_data[3],
            patched_data[4],
            patched_data[5],
        ]);
        assert_eq!(target, new_target);
    }

    #[test]
    fn test_replace_call_wrong_instruction() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.bin");
        let output_path = temp_dir.path().join("output.bin");

        let original_data = vec![0x90, 0x90, 0x90, 0x90, 0x90];
        fs::write(&input_path, &original_data).unwrap();

        let result = BinaryPatcher::replace_call(
            input_path.to_str().unwrap(),
            1,
            0x12345678,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected CALL"));
    }

    #[test]
    fn test_replace_jump_long() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.bin");
        let output_path = temp_dir.path().join("output.bin");

        let original_data = vec![0xE9, 0x00, 0x00, 0x00, 0x00, 0x90];
        fs::write(&input_path, &original_data).unwrap();

        let new_target: u32 = 0x1000;
        let result = BinaryPatcher::replace_jump(
            input_path.to_str().unwrap(),
            0,
            new_target,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_ok());

        let patched_data = fs::read(&output_path).unwrap();
        assert_eq!(patched_data[0], 0xE9);
        let target = u32::from_le_bytes([
            patched_data[1],
            patched_data[2],
            patched_data[3],
            patched_data[4],
        ]);
        assert_eq!(target, new_target);
    }

    #[test]
    fn test_replace_jump_short() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.bin");
        let output_path = temp_dir.path().join("output.bin");

        let original_data = vec![0xEB, 0x00, 0x90, 0x90];
        fs::write(&input_path, &original_data).unwrap();

        let new_target: u32 = 0x20;
        let result = BinaryPatcher::replace_jump(
            input_path.to_str().unwrap(),
            0,
            new_target,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_ok());

        let patched_data = fs::read(&output_path).unwrap();
        assert_eq!(patched_data[0], 0xEB);
        assert_eq!(patched_data[1], 0x20);
    }

    #[test]
    fn test_patch_string_basic() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.bin");
        let output_path = temp_dir.path().join("output.bin");

        let original_data = b"Hello World! Hello World!";
        fs::write(&input_path, original_data).unwrap();

        let result = BinaryPatcher::patch_string(
            input_path.to_str().unwrap(),
            "Hello",
            "Goodbye",
            output_path.to_str().unwrap(),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        let patched_data = fs::read(&output_path).unwrap();
        let patched_str = String::from_utf8_lossy(&patched_data);
        assert!(patched_str.contains("Goodbye"));
    }

    #[test]
    fn test_patch_string_too_long() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.bin");
        let output_path = temp_dir.path().join("output.bin");

        let original_data = b"Hi";
        fs::write(&input_path, original_data).unwrap();

        let result = BinaryPatcher::patch_string(
            input_path.to_str().unwrap(),
            "Hi",
            "VeryLongString",
            output_path.to_str().unwrap(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("longer"));
    }

    #[test]
    fn test_patch_string_padding() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.bin");
        let output_path = temp_dir.path().join("output.bin");

        let original_data = b"LongString";
        fs::write(&input_path, original_data).unwrap();

        let result = BinaryPatcher::patch_string(
            input_path.to_str().unwrap(),
            "LongString",
            "Short",
            output_path.to_str().unwrap(),
        );

        assert!(result.is_ok());

        let patched_data = fs::read(&output_path).unwrap();
        assert_eq!(&patched_data[0..5], b"Short");
        assert_eq!(patched_data[5], 0x00);
    }
}

mod hex_editor_tests {
    use super::*;
    use talon::binary_patch::HexEditor;

    #[test]
    fn test_hex_display() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.bin");

        let data = vec![
            0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64,
        ];
        fs::write(&file_path, &data).unwrap();

        let result = HexEditor::display(file_path.to_str().unwrap(), 0, 11);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hex_search() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.bin");

        let data = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x90, 0x90, 0xDE, 0xAD, 0xBE, 0xEF];
        fs::write(&file_path, &data).unwrap();

        let result = HexEditor::search_hex(file_path.to_str().unwrap(), "DEADBEEF");
        assert!(result.is_ok());

        let offsets = result.unwrap();
        assert_eq!(offsets.len(), 2);
        assert!(offsets.contains(&0));
        assert!(offsets.contains(&6));
    }

    #[test]
    fn test_hex_search_no_match() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.bin");

        let data = vec![0x90; 100];
        fs::write(&file_path, &data).unwrap();

        let result = HexEditor::search_hex(file_path.to_str().unwrap(), "DEADBEEF");
        assert!(result.is_ok());

        let offsets = result.unwrap();
        assert_eq!(offsets.len(), 0);
    }

    #[test]
    fn test_file_comparison_identical() {
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("file1.bin");
        let file2_path = temp_dir.path().join("file2.bin");

        let data = vec![0x90; 100];
        fs::write(&file1_path, &data).unwrap();
        fs::write(&file2_path, &data).unwrap();

        let result =
            HexEditor::compare_files(file1_path.to_str().unwrap(), file2_path.to_str().unwrap());

        assert!(result.is_ok());
        let differences = result.unwrap();
        assert_eq!(differences.len(), 0);
    }

    #[test]
    fn test_file_comparison_different() {
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("file1.bin");
        let file2_path = temp_dir.path().join("file2.bin");

        let data1 = vec![0x90, 0x90, 0x90, 0x90];
        let data2 = vec![0x90, 0xCC, 0x90, 0xCC];
        fs::write(&file1_path, &data1).unwrap();
        fs::write(&file2_path, &data2).unwrap();

        let result =
            HexEditor::compare_files(file1_path.to_str().unwrap(), file2_path.to_str().unwrap());

        assert!(result.is_ok());
        let differences = result.unwrap();
        assert_eq!(differences.len(), 2);
        assert!(differences.contains(&1));
        assert!(differences.contains(&3));
    }
}

mod shellcode_injector_tests {
    use super::*;
    use talon::binary_patch::ShellcodeInjector;

    #[test]
    fn test_inject_at_entry_elf() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.elf");
        let output_path = temp_dir.path().join("output.elf");

        let mut elf_data = vec![0x7f, 0x45, 0x4c, 0x46];
        elf_data.extend_from_slice(&[0x00; 100]);
        fs::write(&input_path, &elf_data).unwrap();

        let shellcode = vec![0x90, 0x90, 0x90];
        let result = ShellcodeInjector::inject_at_entry(
            input_path.to_str().unwrap(),
            &shellcode,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_ok());

        let output_data = fs::read(&output_path).unwrap();
        assert!(output_data.len() > elf_data.len());
    }

    #[test]
    fn test_create_code_cave() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.bin");
        let output_path = temp_dir.path().join("output.bin");

        let original_data = vec![0xCC; 100];
        fs::write(&input_path, &original_data).unwrap();

        let cave_size = 256;
        let result = ShellcodeInjector::create_code_cave(
            input_path.to_str().unwrap(),
            cave_size,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_ok());
        let cave_offset = result.unwrap();

        let output_data = fs::read(&output_path).unwrap();
        assert_eq!(output_data.len(), original_data.len() + cave_size);
        assert_eq!(cave_offset, original_data.len());

        for i in cave_offset..cave_offset + cave_size {
            assert_eq!(output_data[i], 0x90);
        }
    }
}

mod checksum_fixer_tests {
    use super::*;
    use talon::binary_patch::ChecksumFixer;

    fn create_minimal_pe() -> Vec<u8> {
        let mut pe = vec![0x4D, 0x5A];
        pe.extend_from_slice(&[0x00; 58]);

        let e_lfanew: u32 = 0x80;
        pe[0x3C..0x40].copy_from_slice(&e_lfanew.to_le_bytes());

        while pe.len() < e_lfanew as usize {
            pe.push(0x00);
        }

        pe.extend_from_slice(b"PE\0\0");

        while pe.len() < (e_lfanew as usize + 0x60) {
            pe.push(0x00);
        }

        pe
    }

    #[test]
    fn test_pe_checksum_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("test.exe");
        let output_path = temp_dir.path().join("test_fixed.exe");

        let pe_data = create_minimal_pe();
        fs::write(&input_path, &pe_data).unwrap();

        let result = ChecksumFixer::recalculate_pe_checksum(
            input_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
        );

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_pe_checksum_invalid_file() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("not_pe.bin");
        let output_path = temp_dir.path().join("output.bin");

        fs::write(&input_path, b"NOT A PE FILE").unwrap();

        let result = ChecksumFixer::recalculate_pe_checksum(
            input_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not a valid PE"));
    }
}

mod signature_breaker_tests {
    use super::*;
    use talon::binary_patch::SignatureBreaker;

    #[test]
    fn test_flip_random_bits() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.bin");
        let output_path = temp_dir.path().join("output.bin");

        let original_data = vec![0x00; 100];
        fs::write(&input_path, &original_data).unwrap();

        let result = SignatureBreaker::flip_random_bits(
            input_path.to_str().unwrap(),
            5,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_ok());

        let output_data = fs::read(&output_path).unwrap();
        assert_eq!(output_data.len(), original_data.len());

        let mut differences = 0;
        for i in 0..original_data.len() {
            if output_data[i] != original_data[i] {
                differences += 1;
            }
        }
        assert!(differences > 0 && differences <= 5);
    }

    #[test]
    fn test_append_garbage() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.bin");
        let output_path = temp_dir.path().join("output.bin");

        let original_data = vec![0xCC; 50];
        fs::write(&input_path, &original_data).unwrap();

        let garbage_size = 128;
        let result = SignatureBreaker::append_garbage(
            input_path.to_str().unwrap(),
            garbage_size,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_ok());

        let output_data = fs::read(&output_path).unwrap();
        assert_eq!(output_data.len(), original_data.len() + garbage_size);

        for i in 0..original_data.len() {
            assert_eq!(output_data[i], 0xCC);
        }
    }
}
