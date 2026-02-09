use crate::ast::{
    BlockchainCommand, Command, Control, CryptoCommand, Expr, Literal, OffensiveCommand, TryCatch,
    TypeHint, TypedVar,
};
use crate::build_cache::BuildCache;
use std::collections::HashSet;
use std::fmt::Write;

// OpSec Integration: Syscalls for Windows EDR Bypass
// When building on Windows, use indirect syscalls instead of API calls
#[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
use crate::opsec::syscalls::{SyscallResolver, NtSyscalls};

// Semantic Patching Support Hooks
// These functions provide integration points for binary patching with keystone-engine.
// When keystone is integrated, these hooks will enable runtime assembly generation.

/// Hook for semantic patching - validates that a command can be patched.
///
/// # Arguments
/// * `cmd` - The command to validate.
///
/// # Returns
/// * `bool` - True if the command supports semantic patching.
#[allow(dead_code)]
fn is_patchable_command(cmd: &Command) -> bool {
    matches!(
        cmd,
        Command::Assemble { .. }
            | Command::NopSled { .. }
            | Command::GenerateShellcode(_)
            | Command::Offensive(OffensiveCommand::AssembleSyscall { .. })
    )
}

/// Hook for keystone integration - generates assembly code.
///
/// This function will be expanded when keystone-engine is integrated.
/// For now, it provides a placeholder that can be called from codegen.
///
/// # Arguments
/// * `asm_code` - The assembly code string.
/// * `arch` - The target architecture (x86, x64, arm, etc.).
///
/// # Returns
/// * `Result<Vec<u8>, String>` - The assembled machine code bytes.
#[allow(dead_code)]
fn assemble_with_keystone(asm_code: &str, arch: &str) -> Result<Vec<u8>, String> {
    // Placeholder for keystone integration
    // When keystone is added as a dependency, this will use keystone-engine
    // to assemble the code at build time for static binaries.
    Err(format!(
        "Keystone integration pending: {} (arch: {})",
        asm_code, arch
    ))
}

/// Hook for syscall-based Windows operations (EDR bypass)
///
/// Generates code that uses indirect syscalls instead of Windows API calls.
/// This bypasses user-mode hooks commonly used by EDR solutions.
///
/// # Arguments
/// * `operation` - The type of operation (allocate_memory, create_thread, etc.)
/// * `params` - Parameters for the operation
///
/// # Returns
/// * `String` - Generated Rust code using syscalls
#[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
#[allow(dead_code)]
fn generate_syscall_code(operation: &str, params: &[&str]) -> String {
    match operation {
        "allocate_memory" => {
            let size = params.first().unwrap_or(&"4096");
            let prot = params.get(1).unwrap_or(&"RWX");
            format!(
                r#"
    // EDR Bypass: Use NtAllocateVirtualMemory syscall instead of VirtualAlloc
    use talon::opsec::syscalls::{{SyscallResolver, NtSyscalls}};
    let mut resolver = SyscallResolver::new().expect("Failed to create syscall resolver");
    let syscall_num = resolver.resolve_syscall_number("NtAllocateVirtualMemory")
        .expect("Failed to resolve NtAllocateVirtualMemory");
    let stub = resolver.generate_obfuscated_stub(syscall_num)
        .expect("Failed to generate syscall stub");
    // Memory allocation via syscall: size={}, protection={}
    println!("Using syscall {{}} for memory allocation", syscall_num);
"#,
                size, prot
            )
        }
        "create_thread" => {
            format!(
                r#"
    // EDR Bypass: Use NtCreateThreadEx syscall instead of CreateThread
    use talon::opsec::syscalls::{{SyscallResolver, NtSyscalls}};
    let mut resolver = SyscallResolver::new().expect("Failed to create syscall resolver");
    let syscall_num = resolver.resolve_syscall_number("NtCreateThreadEx")
        .expect("Failed to resolve NtCreateThreadEx");
    let stub = resolver.generate_obfuscated_stub(syscall_num)
        .expect("Failed to generate syscall stub");
    println!("Using syscall {{}} for thread creation", syscall_num);
"#
            )
        }
        "write_memory" => {
            format!(
                r#"
    // EDR Bypass: Use NtWriteVirtualMemory syscall
    use talon::opsec::syscalls::{{SyscallResolver, NtSyscalls}};
    let mut resolver = SyscallResolver::new().expect("Failed to create syscall resolver");
    let syscall_num = resolver.resolve_syscall_number("NtWriteVirtualMemory")
        .expect("Failed to resolve NtWriteVirtualMemory");
    let stub = resolver.generate_obfuscated_stub(syscall_num)
        .expect("Failed to generate syscall stub");
    println!("Using syscall {{}} for memory write", syscall_num);
"#
            )
        }
        _ => String::new(),
    }
}

/// Placeholder for non-Windows platforms
#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
fn generate_syscall_code(_operation: &str, _params: &[&str]) -> String {
    "// Syscalls only available on Windows\n".to_string()
}

/// Builds a Rust script from a list of commands and compiles it into a binary.
///
/// # Arguments
/// * `commands` - A slice of `Command` objects representing the script.
/// * `static_build` - If true, builds a statically linked binary.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Ok if successful, Err if the build fails.
pub fn build_script(
    commands: &[Command],
    static_build: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if commands.is_empty() {
        return Err("Cannot build script with no commands".into());
    }

    let mut crate_set = HashSet::new();
    for cmd in commands {
        crate_set.extend(get_required_crates(cmd));
    }

    let cargo_toml = generate_cargo_toml(&crate_set)
        .map_err(|e| format!("Failed to generate Cargo.toml: {}", e))?;
    let rust_code = generate_main_rs(commands, &crate_set)
        .map_err(|e| format!("Failed to generate Rust code: {}", e))?;

    let cache = BuildCache::new()
        .map_err(|e| format!("Failed to initialize build cache: {}", e))?;
    let cache_key = BuildCache::compute_hash(&cargo_toml, &rust_code);

    let build_dir = "talon_build";
    let binary_name = if cfg!(windows) { "talon_script.exe" } else { "talon_script" };
    let output_binary = if static_build {
        format!("{}/target/x86_64-unknown-linux-musl/release/{}", build_dir, binary_name)
    } else {
        format!("{}/target/release/{}", build_dir, binary_name)
    };

    if cache.check_cache(&cache_key) {
        println!("[CACHE] Cache hit: {}", &cache_key[..16]);
        let parent_dir = std::path::Path::new(&output_binary)
            .parent()
            .ok_or("Invalid output path")?;
        std::fs::create_dir_all(parent_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
        cache.retrieve_cache(&cache_key, std::path::Path::new(&output_binary))
            .map_err(|e| format!("Cache retrieval failed: {}", e))?;
        println!("[BUILD] Binary: {}", output_binary);
        return Ok(());
    }

    println!("[CACHE] Cache miss: {}", &cache_key[..16]);
    
    std::fs::create_dir_all(format!("{}/src", build_dir))
        .map_err(|e| format!("Failed to create build directory: {}", e))?;
    std::fs::write(format!("{}/Cargo.toml", build_dir), &cargo_toml)
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;
    std::fs::write(format!("{}/src/main.rs", build_dir), &rust_code)
        .map_err(|e| format!("Failed to write main.rs: {}", e))?;

    let mut cargo_args = vec!["build", "--release"];
    if static_build {
        cargo_args.push("--target=x86_64-unknown-linux-musl");
    }

    println!("[BUILD] Compiling with: cargo {}", cargo_args.join(" "));
    
    let output = std::process::Command::new("cargo")
        .current_dir(build_dir)
        .args(&cargo_args)
        .output()
        .map_err(|e| format!("Failed to execute cargo: {}", e))?;

    if output.status.success() {
        println!("[BUILD] Binary: {}", output_binary);
        
        if let Err(e) = cache.store_cache(
            &cache_key,
            std::path::Path::new(&output_binary),
            &cargo_toml,
            &rust_code,
        ) {
            eprintln!("[CACHE] Warning: Failed to cache build: {}", e);
        } else {
            println!("[CACHE] Stored in cache: {}", &cache_key[..16]);
        }
        
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("[BUILD] Build failed:");
        eprintln!("{}", stderr);
        if !stdout.is_empty() {
            eprintln!("{}", stdout);
        }
        Err(format!("Cargo build failed with exit code: {:?}", output.status.code()).into())
    }
}

/// Determines if the script requires asynchronous execution.
///
/// # Arguments
/// * `commands` - A slice of `Command` objects.
///
/// # Returns
/// * `bool` - True if async is required, false otherwise.
fn is_async_required(commands: &[Command]) -> bool {
    commands.iter().any(|cmd| {
        matches!(
            cmd,
            Command::Beacon { .. }
                | Command::Download { .. }
                | Command::Control(Control::Parallel { .. })
        )
    })
}

/// Retrieves the list of required crates for a given command.
///
/// # Arguments
/// * `cmd` - A reference to a `Command`.
///
/// # Returns
/// * `Vec<&'static str>` - A vector of crate names.
fn get_required_crates(cmd: &Command) -> Vec<&'static str> {
    match cmd {
        Command::Crypto(crypto) => match crypto {
            CryptoCommand::GenerateECCKeypair { .. } | CryptoCommand::ECDSASign { .. } => {
                vec!["openssl", "hex"]
            }
            CryptoCommand::AESGCMEncrypt { .. } => vec!["aes-gcm", "hex"],
        },
        Command::Blockchain(bc) => match bc {
            BlockchainCommand::ParseABI { .. } => vec!["ethabi"],
            BlockchainCommand::EthCall { .. } => vec!["web3", "tokio", "hex"],
            BlockchainCommand::FetchContract { .. }
            | BlockchainCommand::SourcifyContract { .. } => vec!["reqwest"],
            BlockchainCommand::EVMDisassemble { .. } | BlockchainCommand::ScanReentrancy { .. } => {
                vec!["hex"]
            }
            _ => vec![],
        },
        Command::Offensive(off) => match off {
            OffensiveCommand::AssembleSyscall { .. } | OffensiveCommand::BuildShellcode { .. } => {
                vec!["keystone", "capstone"]
            }
            OffensiveCommand::BridgeIDA { .. } | OffensiveCommand::BridgeGhidra { .. } => {
                vec!["serde", "serde_json"]
            }
            OffensiveCommand::DisassembleDotNet { .. } => vec!["dnfile"],
            _ => vec![],
        },
        Command::Beacon { .. }
        | Command::Download { .. }
        | Command::Control(Control::Parallel { .. }) => vec!["tokio", "reqwest"],
        Command::EncodeBase64 { .. } | Command::DecodeBase64 { .. } => vec!["base64"],
        Command::Reverse(_) => vec!["capstone", "pelite", "yara", "sha2", "hex"],
        Command::Match(_) | Command::Expr(Expr::RegexMatch { .. }) => vec!["regex"],
        Command::BitwiseOp { .. } => vec!["hex"],
        Command::ToolExec { .. } => vec!["std"],
        _ => vec![],
    }
}

/// Generates the content of the `Cargo.toml` file.
///
/// # Arguments
/// * `crate_set` - A set of crate names.
///
/// # Returns
/// * `Result<String, std::fmt::Error>` - The generated `Cargo.toml` content.
fn generate_cargo_toml(crate_set: &HashSet<&str>) -> Result<String, std::fmt::Error> {
    let mut cargo = String::new();
    writeln!(cargo, "[package]")?;
    writeln!(cargo, "name = \"talon_script\"")?;
    writeln!(cargo, "version = \"0.1.0\"")?;
    writeln!(cargo, "edition = \"2021\"\n")?;
    writeln!(cargo, "[dependencies]")?;
    let mut sorted: Vec<_> = crate_set.iter().copied().collect();
    sorted.sort_unstable();
    for krate in sorted {
        writeln!(cargo, "{} = \"*\"", krate)?;
    }
    Ok(cargo)
}

/// Generates a function definition in Rust code.
///
/// # Arguments
/// * `buf` - The string buffer to write the code into.
/// * `func` - The function definition to generate.
///
/// # Returns
/// * `Result<(), std::fmt::Error>` - Ok if successful, Err if formatting fails.
fn generate_function_def(buf: &mut String, func: &crate::ast::FunctionDef) -> Result<(), std::fmt::Error> {
    let fn_keyword = if func.is_async { "async fn" } else { "fn" };
    
    let params: Vec<String> = func.args.iter().map(|(name, default)| {
        if let Some(default_val) = default {
            format!("{}: impl Into<String> /* default: {} */", name, generate_expr(default_val))
        } else {
            format!("{}: impl Into<String>", name)
        }
    }).collect();
    
    let return_type = match &func.return_type {
        Some(TypeHint::Int) => " -> i64",
        Some(TypeHint::String) => " -> String",
        Some(TypeHint::List) => " -> Vec<String>",
        Some(TypeHint::Map) => " -> HashMap<String, String>",
        Some(TypeHint::Set) => " -> HashSet<String>",
        Some(TypeHint::Bytes) => " -> Vec<u8>",
        Some(_) => " -> String",
        None => "",
    };
    
    writeln!(buf, "{} {}({}){} {{", fn_keyword, func.name, params.join(", "), return_type)?;
    
    for cmd in &func.body {
        generate_command(buf, cmd, func.is_async)?;
    }
    
    if func.return_type.is_some() {
        writeln!(buf, "    Default::default()")?;
    }
    
    writeln!(buf, "}}\n")?;
    Ok(())
}

/// Generates the content of the `main.rs` file.
///
/// # Arguments
/// * `commands` - A slice of `Command` objects.
/// * `crates` - A set of crate names.
///
/// # Returns
/// * `Result<String, std::fmt::Error>` - The generated Rust code.
fn generate_main_rs(
    commands: &[Command],
    crates: &HashSet<&str>,
) -> Result<String, std::fmt::Error> {
    let mut code = String::new();
    let is_async = is_async_required(commands);

    // Standard imports
    writeln!(code, "use std::collections::{{HashMap, HashSet}};")?;
    writeln!(code, "use std::fs;")?;
    writeln!(code, "use std::process::Command as SysCommand;")?;
    writeln!(code, "use std::time::Duration;")?;
    if crates.contains("tokio") {
        writeln!(code, "use tokio::time;")?;
    }
    if crates.contains("reqwest") {
        writeln!(code, "use reqwest;")?;
    }
    if crates.contains("base64") {
        writeln!(code, "use base64;")?;
    }
    if crates.contains("hex") {
        writeln!(code, "use hex;")?;
    }
    if crates.contains("regex") {
        writeln!(code, "use regex::Regex;")?;
    }

    // Define user functions
    for cmd in commands {
        if let Command::DefineFunction(func) = cmd {
            generate_function_def(&mut code, func)?;
        }
    }

    // Entry point
    if is_async {
        writeln!(code, "\n#[tokio::main]")?;
        writeln!(
            code,
            "async fn main() -> Result<(), Box<dyn std::error::Error>> {{"
        )?;
    } else {
        writeln!(code, "\nfn main() {{")?;
    }

    writeln!(
        code,
        "    let mut vars: HashMap<String, String> = HashMap::new();"
    )?;
    for cmd in commands {
        generate_command(&mut code, cmd, is_async)?;
    }

    if is_async {
        writeln!(code, "    Ok(())")?;
    }
    writeln!(code, "}}")?;
    Ok(code)
}

/// Generates Rust code for a single command.
///
/// # Arguments
/// * `buf` - The string buffer to write the code into.
/// * `cmd` - The command to generate code for.
/// * `is_async` - Whether the script requires async execution.
///
/// # Returns
/// * `Result<(), std::fmt::Error>` - Ok if successful, Err if formatting fails.
fn generate_command(
    buf: &mut String,
    cmd: &Command,
    is_async: bool,
) -> Result<(), std::fmt::Error> {
    match cmd {
        Command::RunCommand { command } => {
            writeln!(buf, "    println!(\"[RUN] {}\");", command)?;
            writeln!(buf, "    let out = SysCommand::new(\"sh\").arg(\"-c\").arg(\"{}\").output().expect(\"Exec fail\");", command)?;
            writeln!(
                buf,
                "    println!(\"{{}}\", String::from_utf8_lossy(&out.stdout));"
            )?;
        }
        Command::TypedDecl(TypedVar {
            name,
            var_type,
            value,
        }) => {
            let rust_type = match var_type {
                TypeHint::Int => "i64",
                TypeHint::String => "String",
                TypeHint::List => "Vec<String>",
                TypeHint::Map => "HashMap<String, String>",
                TypeHint::Set => "HashSet<String>",
                TypeHint::Bytes => "Vec<u8>",
                _ => "String",
            };
            writeln!(
                buf,
                "    let {}: {} = {};",
                name,
                rust_type,
                generate_expr(value)
            )?;
        }
        Command::Download { url, path } => {
            if is_async {
                writeln!(buf, "    let data = reqwest::Client::new().get(\"{}\").send().await?.bytes().await?;", url)?;
            } else {
                writeln!(
                    buf,
                    "    let data = reqwest::blocking::Client::new().get(\"{}\").send()?.bytes()?;",
                    url
                )?;
            }
            writeln!(
                buf,
                "    fs::write(\"{}\", data).expect(\"Failed to write file\");",
                path
            )?;
        }
        Command::Beacon { url, interval } => {
            if is_async {
                writeln!(buf, "    tokio::spawn(async move {{")?;
                writeln!(buf, "        let client = reqwest::Client::new();")?;
                writeln!(buf, "        loop {{")?;
                writeln!(
                    buf,
                    "            let _ = client.get(\"{}\").send().await;",
                    url
                )?;
                writeln!(
                    buf,
                    "            time::sleep(Duration::from_secs({})).await;",
                    interval
                )?;
                writeln!(buf, "        }}")?;
                writeln!(buf, "    }});")?;
            } else {
                writeln!(buf, "    std::thread::spawn(move || {{")?;
                writeln!(
                    buf,
                    "        let client = reqwest::blocking::Client::new();"
                )?;
                writeln!(buf, "        loop {{")?;
                writeln!(buf, "            let _ = client.get(\"{}\").send();", url)?;
                writeln!(
                    buf,
                    "            std::thread::sleep(Duration::from_secs({}));",
                    interval
                )?;
                writeln!(buf, "        }}")?;
                writeln!(buf, "    }});")?;
            }
        }
        Command::WriteFile { data, path } => {
            writeln!(
                buf,
                "    fs::write(\"{}\", {}).expect(\"Write fail\");",
                path,
                generate_expr(data)
            )?;
        }
        Command::ReadFile { path, var } => {
            writeln!(
                buf,
                "    let data = fs::read_to_string(\"{}\").expect(\"Read fail\");",
                path
            )?;
            writeln!(buf, "    vars.insert(\"{}\".to_string(), data);", var)?;
        }
        Command::VarDecl { name, value } => {
            writeln!(
                buf,
                "    vars.insert(\"{}\".to_string(), {}.to_string());",
                name,
                generate_expr(value)
            )?;
        }
        Command::Expr(expr) => {
            writeln!(
                buf,
                "    println!(\"[EXPR] {{}}\", {});",
                generate_expr(expr)
            )?;
        }
        Command::Control(Control::If {
            condition,
            then_body,
            else_body,
        }) => {
            writeln!(buf, "    if {:?} {{", condition)?;
            for inner in then_body {
                generate_command(buf, inner, is_async)?;
            }
            writeln!(buf, "    }} else {{")?;
            for inner in else_body {
                generate_command(buf, inner, is_async)?;
            }
            writeln!(buf, "    }}")?;
        }
        Command::Control(Control::For {
            var,
            iterable,
            body,
        }) => {
            writeln!(buf, "    for {} in {:?} {{", var, iterable)?;
            for inner in body {
                generate_command(buf, inner, is_async)?;
            }
            writeln!(buf, "    }}")?;
        }
        Command::Control(Control::Parallel { body }) => {
            if is_async {
                writeln!(buf, "    let handles = vec![")?;
                for cmd in body {
                    writeln!(buf, "        tokio::spawn(async move {{")?;
                    generate_command(buf, cmd, true)?;
                    writeln!(buf, "        }}),")?;
                }
                writeln!(buf, "    ];")?;
                writeln!(buf, "    for h in handles {{ h.await?; }}")?;
            } else {
                for cmd in body {
                    generate_command(buf, cmd, false)?;
                }
            }
        }
        Command::CallFunction { name, args } => {
            let arg_strs: Vec<String> = args
                .iter()
                .enumerate()
                .map(|(i, (kopt, v))| {
                    if let Some(k) = kopt {
                        format!("let {} = {};", k, generate_expr(v))
                    } else {
                        format!("let arg{} = {};", i, generate_expr(v))
                    }
                })
                .collect();

            for l in &arg_strs {
                writeln!(buf, "    {}", l)?;
            }

            let call_args = (0..args.len())
                .map(|i| format!("arg{}", i))
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(buf, "    {}({});", name, call_args)?;
        }
        Command::ToolExec { tool, args } => {
            let joined_args = args
                .iter()
                .map(generate_expr)
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(buf, "    let output = SysCommand::new(\"{}\").args(vec![{}]).output().expect(\"Tool exec failed\");", tool, joined_args)?;
            writeln!(
                buf,
                "    println!(\"[TOOL] {{}}\", String::from_utf8_lossy(&output.stdout));"
            )?;
        }
        Command::TryCatch(TryCatch {
            try_body,
            catch_var,
            catch_body,
        }) => {
            writeln!(
                buf,
                "    let result: Result<(), Box<dyn std::error::Error>> = (|| {{"
            )?;
            for stmt in try_body {
                generate_command(buf, stmt, is_async)?;
            }
            writeln!(buf, "        Ok(())")?;
            writeln!(buf, "    }})();")?;

            writeln!(buf, "    match result {{")?;
            writeln!(buf, "        Ok(_) => (),")?;
            writeln!(buf, "        Err({}) => {{", catch_var)?;
            for stmt in catch_body {
                generate_command(buf, stmt, is_async)?;
            }
            writeln!(buf, "        }}")?;
            writeln!(buf, "    }}")?;
        }
        Command::Sleep(seconds) => {
            if is_async {
                writeln!(buf, "    tokio::time::sleep(Duration::from_secs({})).await;", seconds)?;
            } else {
                writeln!(buf, "    std::thread::sleep(Duration::from_secs({}));", seconds)?;
            }
        }
        Command::Connect { ip, port } => {
            writeln!(buf, "    let socket = std::net::TcpStream::connect(format!(\"{}:{}\", \"{}\", {})).expect(\"Failed to connect\");", ip, port, ip, port)?;
            writeln!(buf, "    println!(\"[CONNECT] Connected to {}:{}\");", ip, port)?;
        }
        Command::EncodeBase64 { data } => {
            writeln!(buf, "    let encoded = base64::encode({});", generate_expr(data))?;
            writeln!(buf, "    println!(\"[BASE64] {{}}\", encoded);")?;
        }
        Command::DecodeBase64 { data } => {
            writeln!(buf, "    let decoded = base64::decode({}).expect(\"Invalid base64\");", generate_expr(data))?;
            writeln!(buf, "    println!(\"[BASE64] Decoded {{}} bytes\", decoded.len());")?;
        }
        Command::Assignment { name, value } => {
            writeln!(buf, "    vars.insert(\"{}\".to_string(), {}.to_string());", name, generate_expr(value))?;
        }
        Command::ConstDecl { name, value } => {
            writeln!(buf, "    const {}: &str = \"{}\";", name.to_uppercase(), generate_expr(value))?;
        }
        Command::Include { path } => {
            writeln!(buf, "    // Include directive for: {}", path)?;
        }
        Command::Import { module, items } => {
            if let Some(item_list) = items {
                writeln!(buf, "    // Import {{{}}} from {}", item_list.join(", "), module)?;
            } else {
                writeln!(buf, "    // Import {} (all items)", module)?;
            }
        }
        Command::Control(Control::While { condition, body }) => {
            writeln!(buf, "    while {:?} {{", condition)?;
            for inner in body {
                generate_command(buf, inner, is_async)?;
            }
            writeln!(buf, "    }}")?;
        }
        Command::Control(Control::Break) => {
            writeln!(buf, "    break;")?;
        }
        Command::Control(Control::Continue) => {
            writeln!(buf, "    continue;")?;
        }
        Command::BitwiseOp { op, left, right } => {
            let rust_op = match op.as_str() {
                "and" | "&" => "&",
                "or" | "|" => "|",
                "xor" | "^" => "^",
                "shl" | "<<" => "<<",
                "shr" | ">>" => ">>",
                _ => op.as_str(),
            };
            writeln!(buf, "    let result = ({}) {} ({});", generate_expr(left), rust_op, generate_expr(right))?;
            writeln!(buf, "    println!(\"[BITWISE] {{}}\", result);")?;
        }
        Command::AntiDebugCheck => {
            writeln!(buf, "    #[cfg(target_os = \"linux\")]")?;
            writeln!(buf, "    {{")?;
            writeln!(buf, "        use std::fs;")?;
            writeln!(buf, "        let status = fs::read_to_string(\"/proc/self/status\").unwrap_or_default();")?;
            writeln!(buf, "        if status.contains(\"TracerPid:\\t0\") {{")?;
            writeln!(buf, "            println!(\"[DEBUG] No debugger detected\");")?;
            writeln!(buf, "        }} else {{")?;
            writeln!(buf, "            println!(\"[DEBUG] Debugger detected!\");")?;
            writeln!(buf, "        }}")?;
            writeln!(buf, "    }}")?;
        }
        Command::ExitIfDebugger => {
            writeln!(buf, "    #[cfg(target_os = \"linux\")]")?;
            writeln!(buf, "    {{")?;
            writeln!(buf, "        use std::fs;")?;
            writeln!(buf, "        let status = fs::read_to_string(\"/proc/self/status\").unwrap_or_default();")?;
            writeln!(buf, "        if !status.contains(\"TracerPid:\\t0\") {{")?;
            writeln!(buf, "            std::process::exit(1);")?;
            writeln!(buf, "        }}")?;
            writeln!(buf, "    }}")?;
        }
        Command::NopSled { length } => {
            writeln!(buf, "    let nop_sled = vec![0x90u8; {} as usize];", length)?;
            writeln!(buf, "    println!(\"[NOP] Generated {{}} byte NOP sled\", nop_sled.len());")?;
        }
        Command::XorDecode(key) => {
            writeln!(buf, "    let mut decoded = Vec::new();")?;
            writeln!(buf, "    for byte in &vars.get(\"encoded\").unwrap_or(&String::new()).as_bytes() {{")?;
            writeln!(buf, "        decoded.push(byte ^ {});", key)?;
            writeln!(buf, "    }}")?;
            writeln!(buf, "    println!(\"[XOR] Decoded {{}} bytes with key {{}}\", decoded.len(), {});", key)?;
        }
        Command::DefineFunction(_) => {
            // Already handled in generate_main_rs before entry point
        }
        Command::CallMacro { name, args } => {
            let arg_strs: Vec<String> = args.iter().map(generate_expr).collect();
            writeln!(buf, "    // Macro call: {}!({})", name, arg_strs.join(", "))?;
        }
        Command::DefineMacro(macro_def) => {
            writeln!(buf, "    // Macro definition: {}", macro_def.name)?;
        }
        Command::StructDef { name, fields } => {
            writeln!(buf, "    // Struct definition: {} with {} fields", name, fields.len())?;
        }
        Command::DestructuringDecl { vars, value } => {
            writeln!(buf, "    // Destructuring: let [{}] = {:?};", vars.join(", "), value)?;
        }
        Command::Match(match_block) => {
            writeln!(buf, "    // Pattern matching on: {:?}", match_block.expr)?;
            writeln!(buf, "    match {:?} {{", match_block.expr)?;
            for arm in &match_block.arms {
                writeln!(buf, "        {:?} => {{", arm.pattern)?;
                for cmd in &arm.body {
                    generate_command(buf, cmd, is_async)?;
                }
                writeln!(buf, "        }}")?;
            }
            writeln!(buf, "    }}")?;
        }
        Command::Assemble { code } => {
            writeln!(buf, "    // Assembly code: {}", code)?;
            writeln!(buf, "    // NOTE: Assembly requires keystone-engine integration")?;
            writeln!(buf, "    // Use interpreter mode for runtime assembly")?;
        }
        Command::GenerateShellcode(spec) => {
            writeln!(buf, "    // Generate shellcode: {:?}", spec)?;
            writeln!(buf, "    // NOTE: Shellcode generation available in interpreter mode")?;
        }
        Command::LoadShellcode { path } => {
            writeln!(buf, "    let shellcode = fs::read(\"{}\")?;", path)?;
            writeln!(buf, "    println!(\"[SHELLCODE] Loaded {{}} bytes from {{}}\", shellcode.len(), \"{}\");", path)?;
        }
        Command::ExecuteShellcode => {
            writeln!(buf, "    // NOTE: Shellcode execution requires runtime interpreter")?;
            writeln!(buf, "    // Static binary cannot execute shellcode directly")?;
        }
        Command::DumpMemory { address, length } => {
            writeln!(buf, "    // Dump memory at address 0x{:x} for {} bytes", address, length)?;
            writeln!(buf, "    // NOTE: Memory operations require runtime debugging")?;
        }
        Command::Hash(hash_target) => {
            writeln!(buf, "    // Hash operation: {:?}", hash_target)?;
            writeln!(buf, "    use sha2::{{Sha256, Digest}};")?;
            writeln!(buf, "    let mut hasher = Sha256::new();")?;
            writeln!(buf, "    hasher.update(b\"data\");")?;
            writeln!(buf, "    let result = hasher.finalize();")?;
            writeln!(buf, "    println!(\"[HASH] {{:x}}\", result);")?;
        }
        Command::ScanSubnet(subnet) => {
            writeln!(buf, "    println!(\"[SCAN] Scanning subnet: {}\");", subnet)?;
            writeln!(buf, "    // NOTE: Network scanning requires async runtime")?;
        }
        Command::Reverse(_rev_cmd) => {
            writeln!(buf, "    // Reverse engineering command")?;
            writeln!(buf, "    // NOTE: RE features available in interpreter mode")?;
        }
        Command::Crypto(_crypto_cmd) => {
            writeln!(buf, "    // Cryptographic operation")?;
            writeln!(buf, "    // NOTE: Crypto features available in interpreter mode")?;
        }
        Command::Blockchain(_bc_cmd) => {
            writeln!(buf, "    // Blockchain operation")?;
            writeln!(buf, "    // NOTE: Blockchain features available in interpreter mode")?;
        }
        Command::Offensive(_off_cmd) => {
            writeln!(buf, "    // Offensive security operation")?;
            writeln!(buf, "    // NOTE: Advanced offensive features available in interpreter mode")?;
        }
        Command::Toolchain(_tool_cmd) => {
            writeln!(buf, "    // Toolchain operation")?;
            writeln!(buf, "    // NOTE: Toolchain features available in interpreter mode")?;
        }
        Command::CTF(_ctf_cmd) => {
            writeln!(buf, "    // CTF operation")?;
            writeln!(buf, "    // NOTE: CTF features available in interpreter mode")?;
        }
        Command::FormatStringExploit { target, offset } => {
            writeln!(buf, "    println!(\"[FMTSTR] Target: {}, Offset: {}\");", target, offset)?;
            writeln!(buf, "    // NOTE: Format string exploitation available in interpreter mode")?;
        }
        Command::StackOverflowExploit { padding, ret_addr } => {
            writeln!(buf, "    println!(\"[EXPLOIT] Padding: {}, Return address: 0x{:x}\");", padding, ret_addr)?;
            writeln!(buf, "    let payload = vec![b'A'; {} as usize];", padding)?;
            writeln!(buf, "    println!(\"[EXPLOIT] Payload size: {{}} bytes\", payload.len());")?;
        }
        Command::HeapSpray { data } => {
            writeln!(buf, "    println!(\"[HEAP] Spraying: {}\");", data)?;
            writeln!(buf, "    // NOTE: Heap manipulation available in interpreter mode")?;
        }
        Command::FindFormatOffset { binary } => {
            writeln!(buf, "    println!(\"[FMTSTR] Finding offset for: {}\");", binary)?;
        }
        Command::VisualizeHeap { binary } => {
            writeln!(buf, "    println!(\"[HEAP] Visualizing heap for: {}\");", binary)?;
        }
        Command::SigropChain { lib } => {
            writeln!(buf, "    println!(\"[ROP] SIGROP chain for: {}\");", lib)?;
        }
        Command::Fuzz { binary, seed, cycles } => {
            writeln!(buf, "    println!(\"[FUZZ] Target: {}, Seed: {}, Cycles: {}\");", binary, seed, cycles)?;
            writeln!(buf, "    // NOTE: Fuzzing engine available in interpreter mode")?;
        }
        Command::FuzzProtocol(_spec) => {
            writeln!(buf, "    // Protocol fuzzing")?;
            writeln!(buf, "    // NOTE: Protocol fuzzing available in interpreter mode")?;
        }
        Command::SymbolicExecution(_spec) => {
            writeln!(buf, "    // Symbolic execution")?;
            writeln!(buf, "    // NOTE: Symbolic execution available in interpreter mode")?;
        }
        Command::SolveConstraints { target, constraints } => {
            writeln!(buf, "    println!(\"[SYMBOLIC] Solving for target: 0x{:x}\");", target)?;
            writeln!(buf, "    println!(\"[SYMBOLIC] Constraints: {:?}\");", constraints)?;
        }
        Command::AutoExploit(_spec) => {
            writeln!(buf, "    // Automatic exploitation")?;
            writeln!(buf, "    // NOTE: Auto-exploit available in interpreter mode")?;
        }
        Command::DebugAttach(_spec) => {
            writeln!(buf, "    // Debug attach")?;
            writeln!(buf, "    // NOTE: Debugging available in interpreter mode")?;
        }
        Command::HeapGroom(_spec) => {
            writeln!(buf, "    // Heap grooming")?;
            writeln!(buf, "    // NOTE: Heap manipulation available in interpreter mode")?;
        }
        Command::BruteFtp { ip, user, pass_list_path } => {
            writeln!(buf, "    println!(\"[BRUTE] FTP {}@{} with list: {}\");", user, ip, pass_list_path)?;
            writeln!(buf, "    // NOTE: Network operations available in interpreter mode")?;
        }
        _ => {
            writeln!(buf, "    // Command not yet implemented in codegen: {:?}", cmd)?;
            writeln!(buf, "    // NOTE: This command is available in interpreter mode")?;
        }
    }
    Ok(())
}

/// Generates Rust code for an expression.
///
/// # Arguments
/// * `expr` - The expression to generate code for.
///
/// # Returns
/// * `String` - The generated Rust expression code.
fn generate_expr(expr: &Expr) -> String {
    match expr {
        Expr::Literal(Literal::Number(n)) => n.to_string(),
        Expr::Literal(Literal::String(s)) => {
            let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{}\".to_string()", escaped)
        }
        Expr::Literal(Literal::ByteArray(s)) => {
            format!("hex::decode(\"{}\").expect(\"Invalid hex string\")", s)
        }
        Expr::Ident(id) => format!("vars.get(\"{}\").cloned().unwrap_or_default()", id),
        Expr::BinaryOp { op, left, right } => {
            format!("({} {} {})", generate_expr(left), op, generate_expr(right))
        }
        Expr::RegexMatch { regex, haystack } => {
            let safe_regex = regex.replace('\\', "\\\\").replace('"', "\\\"");
            format!(
                "Regex::new(r\"{}\").expect(\"Invalid regex\").is_match(&{})",
                safe_regex,
                generate_expr(haystack)
            )
        }
        Expr::InterpolatedString(parts) => {
            let joined = parts
                .iter()
                .map(generate_expr)
                .collect::<Vec<_>>()
                .join(" + ");
            format!("({})", joined)
        }
        _ => format!("\"[unsupported expr: {:?}]\"", expr),
    }
}
