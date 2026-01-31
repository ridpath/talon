pub fn read_process_memory(_pid: u32, _address: usize, _size: usize) -> Result<Vec<u8>, String> {
    #[cfg(target_os = "linux")]
    {
        let mem_path = format!("/proc/{}/mem", pid);
        let mut file =
            File::open(&mem_path).map_err(|e| format!("Failed to open memory: {}", e))?;

        use std::os::unix::fs::FileExt;
        let mut buffer = vec![0u8; size];
        file.read_exact_at(&mut buffer, address as u64)
            .map_err(|e| format!("Failed to read memory: {}", e))?;

        Ok(buffer)
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err("Memory reading not implemented for this platform".to_string())
    }
}

pub fn write_process_memory(_pid: u32, _address: usize, _data: &[u8]) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let mem_path = format!("/proc/{}/mem", pid);
        let mut file = File::options()
            .write(true)
            .open(&mem_path)
            .map_err(|e| format!("Failed to open memory for writing: {}", e))?;

        use std::os::unix::fs::FileExt;
        file.write_all_at(data, address as u64)
            .map_err(|e| format!("Failed to write memory: {}", e))?;

        println!(
            "[MEM] Wrote {} bytes to PID {} at 0x{:x}",
            data.len(),
            pid,
            address
        );
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err("Memory writing not implemented for this platform".to_string())
    }
}

pub fn pattern_scan(haystack: &[u8], pattern: &[u8]) -> Vec<usize> {
    let mut offsets = Vec::new();

    if pattern.is_empty() || haystack.len() < pattern.len() {
        return offsets;
    }

    for i in 0..=(haystack.len() - pattern.len()) {
        if &haystack[i..i + pattern.len()] == pattern {
            offsets.push(i);
        }
    }

    offsets
}

pub fn pattern_scan_wildcard(haystack: &[u8], pattern: &str) -> Vec<usize> {
    let parts: Vec<&str> = pattern.split_whitespace().collect();
    let mut pattern_bytes = Vec::new();
    let mut mask = Vec::new();

    for part in parts {
        if part == "??" {
            pattern_bytes.push(0);
            mask.push(false);
        } else if let Ok(byte) = u8::from_str_radix(part, 16) {
            pattern_bytes.push(byte);
            mask.push(true);
        }
    }

    let mut offsets = Vec::new();

    if pattern_bytes.is_empty() || haystack.len() < pattern_bytes.len() {
        return offsets;
    }

    'outer: for i in 0..=(haystack.len() - pattern_bytes.len()) {
        for (j, &masked) in mask.iter().enumerate() {
            if masked && haystack[i + j] != pattern_bytes[j] {
                continue 'outer;
            }
        }
        offsets.push(i);
    }

    offsets
}

pub fn dump_process_memory(_pid: u32, _output_path: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let maps_path = format!("/proc/{}/maps", pid);
        let maps_content =
            fs::read_to_string(&maps_path).map_err(|e| format!("Failed to read maps: {}", e))?;

        let mut dump_file =
            File::create(output_path).map_err(|e| format!("Failed to create dump file: {}", e))?;

        for line in maps_content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let addr_range = parts[0];
            let range_parts: Vec<&str> = addr_range.split('-').collect();
            if range_parts.len() != 2 {
                continue;
            }

            if let (Ok(start), Ok(end)) = (
                usize::from_str_radix(range_parts[0], 16),
                usize::from_str_radix(range_parts[1], 16),
            ) {
                let size = end - start;
                if let Ok(data) = read_process_memory(pid, start, size) {
                    dump_file.write_all(&data).ok();
                }
            }
        }

        println!("[MEM] Dumped memory from PID {} to {}", pid, output_path);
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err("Memory dumping not implemented for this platform".to_string())
    }
}

pub fn allocate_shellcode_memory(_size: usize) -> Result<*mut u8, String> {
    #[cfg(unix)]
    {
        use std::ptr;

        unsafe {
            let addr = libc::mmap(
                ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            );

            if addr == libc::MAP_FAILED {
                Err("Failed to allocate executable memory".to_string())
            } else {
                Ok(addr as *mut u8)
            }
        }
    }

    #[cfg(not(unix))]
    {
        Err("Memory allocation not implemented for this platform".to_string())
    }
}

pub fn execute_shellcode(shellcode: &[u8]) -> Result<(), String> {
    let mem_ptr = allocate_shellcode_memory(shellcode.len())?;

    unsafe {
        std::ptr::copy_nonoverlapping(shellcode.as_ptr(), mem_ptr, shellcode.len());

        let shellcode_fn: extern "C" fn() = std::mem::transmute(mem_ptr);
        shellcode_fn();
    }

    Ok(())
}

pub fn find_gadget_rop(binary_data: &[u8], gadget_pattern: &str) -> Vec<usize> {
    pattern_scan_wildcard(binary_data, gadget_pattern)
}

pub fn list_process_maps(_pid: u32) -> Result<Vec<String>, String> {
    #[cfg(target_os = "linux")]
    {
        let maps_path = format!("/proc/{}/maps", pid);
        let content =
            fs::read_to_string(&maps_path).map_err(|e| format!("Failed to read maps: {}", e))?;

        Ok(content.lines().map(|s| s.to_string()).collect())
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err("Maps listing not implemented for this platform".to_string())
    }
}
