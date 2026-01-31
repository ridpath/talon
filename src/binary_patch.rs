use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

// ═══════════════════════════════════════════════════════════════════════════
// BINARY PATCHING TOOLKIT - PRODUCTION READY
// ═══════════════════════════════════════════════════════════════════════════

// ────────────────────────────────────────────────────────────────────────────
// BINARY PATCHER
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
                    data[i + new_bytes.len()..i + old_bytes.len()].fill(0);
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

        let pattern = hex::decode(hex_pattern.replace(" ", ""))
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
