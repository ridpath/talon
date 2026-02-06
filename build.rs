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
        write!(&mut file, "    \"{}\" => {},\n", name, idx).unwrap();
    }

    write!(&mut file, "}};\n\n").unwrap();

    // Write array indices for compile-time validation
    write!(&mut file, "pub const BUILTIN_COUNT: usize = {};\n", functions.len()).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}

fn get_builtin_functions() -> Vec<&'static str> {
    vec![
        "connect", "send", "sendline", "recv", "recvline", "recvuntil", "close",
        "interactive", "connect_ssl", "process", "attach", "gdb", "disasm_at",
        "print", "hex", "unhex", "b64encode", "b64decode", "p8", "p16", "p32",
        "p64", "u8", "u16", "u32", "u64", "Elf", "Libc", "ROP", "shellcode",
        "cyclic", "cyclic_find", "fmtstr_payload", "xor", "rol", "ror", "make_elf",
        "make_pe", "asm", "asm_at", "pwn", "ssh_connect", "ssh_run", "ssh_upload",
        "ssh_download", "ssh_interactive", "connect_ssh", "connect_ssh_pty",
        "ssh_interactive_start", "ssh_interactive_send", "ssh_interactive_recv",
        "ssh_interactive_close", "pack", "unpack", "rop_find", "parallel_exploit",
        "copy", "remote", "help", "debug_attach", "breakpoint", "debug_continue",
        "debug_step", "debug_read_mem", "debug_write_mem", "debug_read_reg",
        "debug_write_reg", "symbolic_var", "constrain_no_null", "constrain_alnum",
        "constrain_range", "symbolic_solve", "pool_spray", "heap_feng_shui",
        "token_steal", "process_hide", "rootkit_install", "kaslr_leak",
        "smep_bypass", "kernel_write", "kernel_read", "padding_oracle",
        "bleichenbacher", "timing_attack", "weak_keys", "hash_collision",
        "aes_padding_attack", "rsa_factorize", "fuzz_target", "mutate", "coverage",
        "corpus_add", "crash_triage", "generate_exploit", "oracle_analyze",
        "oracle_find_shellcode", "oracle_gadget_density", "oracle_report",
        "mass_connect", "mitigation_analyze", "mitigation_auto_pivot",
        "mitigation_validate",
    ]
}
