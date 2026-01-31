use crate::ast::{
    BlockchainCommand, Command, Control, CryptoCommand, Expr, Literal, OffensiveCommand, TryCatch,
    TypedVar,
};
use std::collections::HashSet;
use std::fmt::Write;

#[derive(Debug, Clone)]
struct PayloadData {
    rop_gadgets: Vec<(String, u64)>,
    shellcode_bytes: Vec<Vec<u8>>,
    addresses: Vec<(String, u64)>,
}

impl PayloadData {
    fn new() -> Self {
        PayloadData {
            rop_gadgets: Vec::new(),
            shellcode_bytes: Vec::new(),
            addresses: Vec::new(),
        }
    }
}

/// Builds a Rust script from a list of commands and compiles it into a binary.
///
/// # Arguments
/// * `commands` - A slice of `Command` objects representing the script.
/// * `static_build` - If true, builds a statically linked binary.
/// * `output_path` - Optional custom output path for the binary.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Ok if successful, Err if the build fails.
pub fn build_script(
    commands: &[Command],
    static_build: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    build_script_with_output(commands, static_build, None)
}

pub fn build_script_with_output(
    commands: &[Command],
    static_build: bool,
    output_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut crate_set = HashSet::new();
    for cmd in commands {
        crate_set.extend(get_required_crates(cmd));
    }

    let payload_data = extract_payloads(commands);
    let cargo_toml = generate_cargo_toml(&crate_set, static_build)?;
    let rust_code = generate_main_rs(commands, &crate_set, &payload_data)?;

    let build_dir = "talon_build";
    std::fs::create_dir_all(format!("{}/src", build_dir))?;
    std::fs::write(format!("{}/Cargo.toml", build_dir), cargo_toml)?;
    std::fs::write(format!("{}/src/main.rs", build_dir), rust_code)?;

    let target_triple = if static_build {
        detect_static_target()?
    } else {
        None
    };

    let mut cargo_args = vec!["build", "--release"];
    if let Some(target) = &target_triple {
        cargo_args.push("--target");
        cargo_args.push(target);
    }

    println!("[*] Building exploit binary...");
    if static_build {
        println!(
            "[*] Static linking enabled: {}",
            target_triple.as_ref().unwrap()
        );
    }

    let status = std::process::Command::new("cargo")
        .current_dir(build_dir)
        .args(&cargo_args)
        .status()?;

    if !status.success() {
        return Err("Cargo build failed".into());
    }

    let source_binary = if let Some(target) = &target_triple {
        format!("{}/target/{}/release/talon_script", build_dir, target)
    } else {
        format!("{}/target/release/talon_script", build_dir)
    };

    if cfg!(target_os = "windows") && !source_binary.ends_with(".exe") {
        let source_with_exe = format!("{}.exe", source_binary);
        if std::path::Path::new(&source_with_exe).exists() {
            let _ = source_binary;
            let source_binary = source_with_exe;
            copy_output_binary(&source_binary, output_path)?;
        } else {
            copy_output_binary(&source_binary, output_path)?;
        }
    } else {
        copy_output_binary(&source_binary, output_path)?;
    }

    Ok(())
}

fn copy_output_binary(
    source: &str,
    output_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let dest = if let Some(path) = output_path {
        path.to_string()
    } else if cfg!(target_os = "windows") {
        "exploit_bin.exe".to_string()
    } else {
        "exploit_bin".to_string()
    };

    std::fs::copy(source, &dest)?;
    println!("[+] Binary compiled: {}", dest);
    println!("[*] Size: {} bytes", std::fs::metadata(&dest)?.len());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dest)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest, perms)?;
    }

    Ok(())
}

fn detect_static_target() -> Result<Option<String>, Box<dyn std::error::Error>> {
    if cfg!(target_os = "linux") {
        check_musl_installed()?;
        Ok(Some("x86_64-unknown-linux-musl".to_string()))
    } else if cfg!(target_os = "windows") {
        Ok(Some("x86_64-pc-windows-msvc".to_string()))
    } else {
        Err("Static linking not supported on this platform".into())
    }
}

fn check_musl_installed() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()?;

    let installed = String::from_utf8_lossy(&output.stdout);
    if !installed.contains("x86_64-unknown-linux-musl") {
        eprintln!("[!] musl target not installed");
        eprintln!("[*] Installing x86_64-unknown-linux-musl...");
        let status = std::process::Command::new("rustup")
            .args(["target", "add", "x86_64-unknown-linux-musl"])
            .status()?;
        if !status.success() {
            return Err("Failed to install musl target".into());
        }
    }
    Ok(())
}

fn extract_payloads(commands: &[Command]) -> PayloadData {
    let mut data = PayloadData::new();

    for cmd in commands {
        match cmd {
            Command::GenerateShellcode(_spec) => {
                data.shellcode_bytes.push(vec![]);
            }
            Command::LoadShellcode { path } => {
                if let Ok(bytes) = std::fs::read(path) {
                    data.shellcode_bytes.push(bytes);
                }
            }
            Command::Offensive(OffensiveCommand::BuildShellcode { asm: _, .. }) => {
                data.shellcode_bytes.push(vec![]);
            }
            Command::VarDecl {
                name: _,
                value: Expr::Literal(Literal::ByteArray(hex_str)),
            } => {
                if let Ok(bytes) = hex::decode(hex_str) {
                    data.shellcode_bytes.push(bytes);
                }
            }
            _ => {}
        }
    }

    data
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
/// * `static_build` - Whether to configure for static linking.
///
/// # Returns
/// * `Result<String, std::fmt::Error>` - The generated `Cargo.toml` content.
fn generate_cargo_toml(
    crate_set: &HashSet<&str>,
    static_build: bool,
) -> Result<String, std::fmt::Error> {
    let mut cargo = String::new();
    writeln!(cargo, "[package]")?;
    writeln!(cargo, "name = \"talon_script\"")?;
    writeln!(cargo, "version = \"0.1.0\"")?;
    writeln!(cargo, "edition = \"2021\"\n")?;

    if static_build {
        writeln!(cargo, "[profile.release]")?;
        writeln!(cargo, "opt-level = \"z\"")?;
        writeln!(cargo, "lto = true")?;
        writeln!(cargo, "codegen-units = 1")?;
        writeln!(cargo, "panic = \"abort\"")?;
        writeln!(cargo, "strip = true\n")?;
    }

    writeln!(cargo, "[dependencies]")?;
    let mut sorted: Vec<_> = crate_set.iter().copied().collect();
    sorted.sort_unstable();
    for krate in sorted {
        writeln!(cargo, "{} = \"*\"", krate)?;
    }
    Ok(cargo)
}

/// Generates the content of the `main.rs` file.
///
/// # Arguments
/// * `commands` - A slice of `Command` objects.
/// * `crates` - A set of crate names.
/// * `payload_data` - Embedded payload data (shellcode, ROP gadgets, addresses).
///
/// # Returns
/// * `Result<String, std::fmt::Error>` - The generated Rust code.
fn generate_main_rs(
    commands: &[Command],
    crates: &HashSet<&str>,
    payload_data: &PayloadData,
) -> Result<String, std::fmt::Error> {
    let mut code = String::new();
    let is_async = is_async_required(commands);

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

    if !payload_data.shellcode_bytes.is_empty()
        || !payload_data.rop_gadgets.is_empty()
        || !payload_data.addresses.is_empty()
    {
        writeln!(code, "\n")?;

        for (idx, shellcode) in payload_data.shellcode_bytes.iter().enumerate() {
            write!(code, "const SHELLCODE_{}: &[u8] = &[", idx)?;
            for (i, byte) in shellcode.iter().enumerate() {
                if i > 0 {
                    write!(code, ", ")?;
                }
                write!(code, "0x{:02x}", byte)?;
            }
            writeln!(code, "];")?;
        }

        for (name, addr) in &payload_data.rop_gadgets {
            writeln!(
                code,
                "const ROP_{}: u64 = 0x{:x};",
                name.to_uppercase(),
                addr
            )?;
        }

        for (name, addr) in &payload_data.addresses {
            writeln!(
                code,
                "const ADDR_{}: u64 = 0x{:x};",
                name.to_uppercase(),
                addr
            )?;
        }
    }

    for cmd in commands {
        if let Command::DefineFunction(func) = cmd {
            writeln!(code, "// [TODO] Define function: {}", func.name)?;
        }
    }

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
            var_type: _,
            value,
        }) => {
            let expr_str = generate_expr(value);
            if matches!(value, Expr::Literal(Literal::String(_))) {
                writeln!(
                    buf,
                    "    vars.insert(\"{}\".to_string(), {}.to_string());",
                    name, expr_str
                )?;
            } else {
                writeln!(
                    buf,
                    "    vars.insert(\"{}\".to_string(), format!(\"{{}}\", {}));",
                    name, expr_str
                )?;
            }
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
            let expr_str = generate_expr(value);
            if matches!(value, Expr::Literal(Literal::String(_))) {
                writeln!(
                    buf,
                    "    vars.insert(\"{}\".to_string(), {}.to_string());",
                    name, expr_str
                )?;
            } else {
                writeln!(
                    buf,
                    "    vars.insert(\"{}\".to_string(), format!(\"{{}}\", {}));",
                    name, expr_str
                )?;
            }
        }
        Command::Expr(expr) => match expr {
            Expr::Call { name, args } => match name.as_str() {
                "print" => {
                    if args.len() == 1 {
                        writeln!(
                            buf,
                            "    println!(\"{{}}\", {});",
                            generate_expr(&args[0].1)
                        )?;
                    } else {
                        for (_, arg) in args {
                            writeln!(buf, "    println!(\"{{}}\", {});", generate_expr(arg))?;
                        }
                    }
                }
                "hex" => {
                    let arg_str = if !args.is_empty() {
                        generate_expr(&args[0].1)
                    } else {
                        "0".to_string()
                    };
                    writeln!(buf, "    println!(\"{{:x}}\", {});", arg_str)?;
                }
                "len" => {
                    let arg_str = if !args.is_empty() {
                        generate_expr(&args[0].1)
                    } else {
                        "\"\".to_string()".to_string()
                    };
                    writeln!(buf, "    println!(\"{{}}\", {}.len());", arg_str)?;
                }
                _ => {
                    writeln!(buf, "    // [UNHANDLED FUNCTION CALL] {}", name)?;
                }
            },
            _ => {
                writeln!(
                    buf,
                    "    println!(\"[EXPR] {{}}\", {});",
                    generate_expr(expr)
                )?;
            }
        },
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
        _ => writeln!(buf, "    // [UNHANDLED] {:?}", cmd)?,
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
            let escaped = s
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t");
            format!("\"{}\"", escaped)
        }
        Expr::Literal(Literal::ByteArray(s)) => {
            format!("hex::decode(\"{}\").expect(\"Invalid hex string\")", s)
        }
        Expr::Ident(id) => format!("vars.get(\"{}\").cloned().unwrap_or_default()", id),
        Expr::BinaryOp { op, left, right } => match op.as_str() {
            "*" => {
                if matches!(**left, Expr::Literal(Literal::String(_))) {
                    format!(
                        "{}.repeat({} as usize)",
                        generate_expr(left),
                        generate_expr(right)
                    )
                } else if matches!(**right, Expr::Literal(Literal::String(_))) {
                    format!(
                        "{}.repeat({} as usize)",
                        generate_expr(right),
                        generate_expr(left)
                    )
                } else {
                    format!("({} {} {})", generate_expr(left), op, generate_expr(right))
                }
            }
            "+" => {
                if matches!(**left, Expr::Literal(Literal::String(_)))
                    || matches!(**right, Expr::Literal(Literal::String(_)))
                {
                    format!(
                        "format!(\"{{}}{{}}\", {}, {})",
                        generate_expr(left),
                        generate_expr(right)
                    )
                } else {
                    format!("({} + {})", generate_expr(left), generate_expr(right))
                }
            }
            _ => format!("({} {} {})", generate_expr(left), op, generate_expr(right)),
        },
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
                .join(" + &");
            format!("({})", joined)
        }
        Expr::Call { name, args } => match name.as_str() {
            "hex" => {
                let arg_str = if !args.is_empty() {
                    generate_expr(&args[0].1)
                } else {
                    "0".to_string()
                };
                format!("format!(\"{{:x}}\", {})", arg_str)
            }
            "len" => {
                let arg_str = if !args.is_empty() {
                    generate_expr(&args[0].1)
                } else {
                    "\"\".to_string()".to_string()
                };
                format!("{}.len()", arg_str)
            }
            _ => format!("/* unsupported function: {} */\"\"", name),
        },
        Expr::Index { base, index } => {
            format!(
                "vars.get(&format!(\"{{}}\", {}))[{}]",
                generate_expr(base),
                generate_expr(index)
            )
        }
        _ => format!("/* unsupported expr: {:?} */\"\"", expr),
    }
}
