use crate::ast::{
    BlockchainCommand, Command, Control, CryptoCommand, Expr, Literal, OffensiveCommand, TryCatch,
    TypeHint, TypedVar,
};
use std::collections::HashSet;
use std::fmt::Write;

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
    let mut crate_set = HashSet::new();
    for cmd in commands {
        crate_set.extend(get_required_crates(cmd));
    }

    let cargo_toml = generate_cargo_toml(&crate_set)?;
    let rust_code = generate_main_rs(commands, &crate_set)?;

    let build_dir = "talon_build";
    std::fs::create_dir_all(format!("{}/src", build_dir))?;
    std::fs::write(format!("{}/Cargo.toml", build_dir), cargo_toml)?;
    std::fs::write(format!("{}/src/main.rs", build_dir), rust_code)?;

    let mut cargo_args = vec!["build", "--release"];
    if static_build {
        cargo_args.push("--target=x86_64-unknown-linux-musl");
    }

    let status = std::process::Command::new("cargo")
        .current_dir(build_dir)
        .args(&cargo_args)
        .status()?;

    if status.success() {
        println!("[BUILD] Binary: {}/target/release/talon_script", build_dir);
        Ok(())
    } else {
        Err("Cargo build failed".into())
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
            writeln!(code, "// [TODO] Define function: {}", func.name)?;
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
        "    let mut vars: HashMap<String, String> = HashMap::new```rust
    new();
"
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
