use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    // Generate PHF registry code
    generate_registry_phf();

    // Only compile protos when swarm feature is enabled AND protoc is available
    #[cfg(feature = "swarm")]
    {
        // Check if protoc is available in PATH
        if std::env::var("PROTOC").is_ok() || which::which("protoc").is_ok() {
            println!("cargo:info=Compiling proto files with protoc");
            
            tonic_build::configure()
                .build_server(true)
                .build_client(true)
                .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
                .enum_attribute(".", "#[derive(Default)]")
                .server_mod_attribute("talon_swarm_server", "#[derive(Clone)]")
                .compile(
                    &["proto/swarm.proto"],
                    &["proto/"],
                )
                .expect("Failed to compile proto files");
        } else {
            println!("cargo:warning=protoc not found, using pre-generated proto code");
            println!("cargo:warning=Install protoc to regenerate: https://github.com/protocolbuffers/protobuf/releases");
        }
    }
}

fn generate_registry_phf() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("registry_phf.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    write!(
        &mut file,
        "use phf::phf_map;\n\n\
        pub static BUILTIN_REGISTRY: phf::Map<&'static str, usize> = phf_map! {{\n"
    )
    .unwrap();

    // Write the PHF map entries (name -> index)
    let functions = get_builtin_functions();
    for (idx, name) in functions.iter().enumerate() {
        writeln!(&mut file, "    \"{}\" => {},", name, idx).unwrap();
    }

    writeln!(&mut file, "}};\n").unwrap();

    // Write array indices for compile-time validation
    writeln!(&mut file, "pub const BUILTIN_COUNT: usize = {};", functions.len()).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}

fn get_builtin_functions() -> Vec<&'static str> {
    vec![
        // Network
        "connect", "send", "sendline", "recv", "recvline", "recvuntil", "close",
        "interactive", "connect_ssl",
        // Process
        "process", "attach", "gdb",
        // I/O
        "disasm_at", "print", "hex", "unhex",
        // SSH
        "ssh_connect", "ssh_run", "ssh_upload", "ssh_download", "ssh_interactive",
        "connect_ssh", "connect_ssh_pty", "connect_ssh_key",
        "ssh_interactive_start", "ssh_interactive_send", "ssh_interactive_recv",
        "ssh_interactive_close", "ssh_forward", "ssh_interact",
        // Packing
        "b64encode", "b64decode", "p8", "p16", "p32", "p64", "u8", "u16", "u32", "u64",
        "pack", "unpack",
        // Exploitation
        "Elf", "Libc", "ROP", "shellcode", "cyclic", "cyclic_find", "fmtstr_payload",
        "xor", "rol", "ror", "make_elf", "make_pe", "asm", "asm_at", "pwn",
        "rop_find", "parallel_exploit", "flat",
        // Binary Analysis
        "oracle_analyze", "oracle_find_shellcode", "oracle_gadget_density", "oracle_report",
        "disasm", "analyze", "auto_offset",
        // Utilities
        "copy", "remote", "help", "len", "range", "int", "bytes", "str",
        "random_string", "extract_pattern",
        // File I/O
        "read", "write",
        // String Manipulation
        "split", "join", "replace",
        // Debugging
        "debug_attach", "breakpoint", "debug_continue", "debug_step",
        "debug_read_mem", "debug_write_mem", "debug_read_reg", "debug_write_reg",
        // Symbolic
        "symbolic_var", "constrain_no_null", "constrain_alnum",
        "constrain_range", "symbolic_solve",
        // Heap
        "pool_spray", "heap_feng_shui",
        // Kernel
        "token_steal", "process_hide", "rootkit_install", "kaslr_leak",
        "smep_bypass", "kernel_write", "kernel_read",
        // Crypto
        "padding_oracle", "bleichenbacher", "timing_attack", "weak_keys",
        "hash_collision", "aes_padding_attack", "rsa_factorize",
        // Fuzzing
        "fuzz_target", "mutate", "coverage", "corpus_add", "crash_triage",
        // AI
        "generate_exploit",
        // Swarm
        "mass_connect",
        // Mitigation
        "mitigation_analyze", "mitigation_auto_pivot", "mitigation_validate",
        "mitigation_generate_leak",
    ]
}
