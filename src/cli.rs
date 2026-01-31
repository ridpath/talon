use std::fmt::Write;
use std::fs::{self, File};
use std::io::Write as IoWrite;

use crate::ai_exploit_gen::AIProvider;
use crate::ai_suggestion::{suggest_exploits_for_binary, SuggestionEngine};
use crate::challenge_marketplace::ChallengeMarketplace;
use crate::cheatsheet::CheatSheet;
use crate::completions::CompletionGenerator;
use crate::config::TalonConfig;
use crate::enhanced_binary_diff::EnhancedBinaryDiffer;
use crate::examples::ExampleLibrary;
use crate::exploit_db::ExploitDatabase;
use crate::formatter;
use crate::linter;
use crate::manpages::ManPages;
use crate::one_liners::OneLinerLibrary;
use crate::replay_format::TalonReplay;
use crate::script_translator::ScriptTranslator;
use crate::target_detection::TargetDetector;
use crate::templates::TemplateGenerator;
use crate::tutorial_system::TutorialSystem;
use crate::visualizer::visualize;
use crate::workspace::WorkspaceManager;
use colored::*;

/// Entry CLI dispatcher for TALON DSL
pub fn run(args: Vec<String>) {
    let mut auto_run = false;
    let mut script_path = None;

    for arg in &args {
        match arg.as_str() {
            "--run" => auto_run = true,
            val if !val.starts_with("--") && script_path.is_none() => {
                script_path = Some(val.to_string());
            }
            _ => {}
        }
    }

    match args.get(1).map(|s| s.as_str()) {
        Some("build") if args.len() >= 3 => {
            let mut static_flag = false;
            let mut output_path: Option<String> = None;
            let mut target_file: Option<String> = None;

            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--static" => static_flag = true,
                    "-o" if i + 1 < args.len() => {
                        output_path = Some(args[i + 1].clone());
                        i += 1;
                    }
                    arg if !arg.starts_with('-') && target_file.is_none() => {
                        target_file = Some(arg.to_string());
                    }
                    _ => {}
                }
                i += 1;
            }

            if target_file.is_none() {
                eprintln!("Error: No input file specified");
                return;
            }

            let target_file = target_file.unwrap();
            let script = fs::read_to_string(&target_file).expect("Unable to read script file");
            let cmds = crate::parser::parse_script(&script).expect("Parse failed");
            crate::codegen::build_script_with_output(&cmds, static_flag, output_path.as_deref())
                .expect("Build failed");

            if auto_run {
                let bin_path = output_path.unwrap_or_else(|| {
                    if cfg!(target_os = "windows") {
                        "exploit_bin.exe".to_string()
                    } else {
                        "exploit_bin".to_string()
                    }
                });
                println!("{}", "[*] Executing compiled binary...".yellow());
                std::process::Command::new(&bin_path)
                    .status()
                    .expect("Execution failed");
            }
        }
        _ => match args
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .as_slice()
        {
            [_, "run", file] => {
                let script = fs::read_to_string(file).expect("Unable to read script file");
                let cmds = crate::parser::parse_script(&script).expect("Parse failed");
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(crate::interpreter::interpret(&cmds))
                    .expect("Interpretation failed");
            }

            [_, "wasm", file] => {
                let script = fs::read_to_string(file).expect("Unable to read script file");
                let cmds = crate::parser::parse_script(&script).expect("Parse failed");
                crate::wasm_codegen::emit_wasm(&cmds, "talon.wasm");
                println!("{}", "[WASM] WebAssembly emitted to talon.wasm".green());
            }

            [_, "analyze", "--binary", file] => {
                let script = fs::read_to_string(file).expect("Unable to read script file");
                let cmds = crate::parser::parse_script(&script).expect("Parse failed");
                for c in cmds {
                    if let crate::ast::Command::Reverse(ref r) = c {
                        crate::re_tools::handle_re_command(r)
                            .unwrap_or_else(|e| eprintln!("Error: {}", e));
                    }
                }
            }

            [_, "ast", file] => {
                let script = fs::read_to_string(file).expect("Unable to read file");
                let cmds = crate::parser::parse_script(&script).expect("Parse failed");
                println!("{}", "AST GraphViz Output:".bold().blue());
                visualize(&cmds);
            }

            [_, "doc", file] => {
                let script = fs::read_to_string(file).expect("Unable to read file");
                let cmds = crate::parser::parse_script(&script).expect("Parse failed");
                let mut doc = String::new();
                for cmd in &cmds {
                    writeln!(doc, "- `{:#?}`", cmd).unwrap();
                }
                fs::write("script.md", doc).expect("Doc write failed");
                println!("{}", "Wrote AST doc to script.md".green());
            }

            [_, "repl"] => {
                crate::interpreter::run_repl();
            }

            [_, "quick-ref"] | [_, "quickref"] | [_, "qref"] => {
                print_quick_reference();
            }

            [_, "cheat", topic] => {
                CheatSheet::show(topic);
            }

            [_, "examples", "list"] | [_, "examples"] => {
                let lib = ExampleLibrary::new();
                lib.list();
            }

            [_, "examples", "show", name] => {
                let lib = ExampleLibrary::new();
                lib.show(name);
            }

            [_, "examples", "run", name] => {
                let lib = ExampleLibrary::new();
                lib.run(name).unwrap_or_else(|e| eprintln!("Error: {}", e));
            }

            [_, "examples", "copy", name] => {
                let lib = ExampleLibrary::new();
                lib.copy(name, None)
                    .unwrap_or_else(|e| eprintln!("Error: {}", e));
            }

            [_, "examples", "copy", name, dest] => {
                let lib = ExampleLibrary::new();
                lib.copy(name, Some(dest))
                    .unwrap_or_else(|e| eprintln!("Error: {}", e));
            }

            [_, "workspace", "init", name] => {
                WorkspaceManager::init(name).unwrap_or_else(|e| eprintln!("Error: {}", e));
            }

            [_, "workspace", "add", workspace, challenge] => {
                WorkspaceManager::add(workspace, challenge)
                    .unwrap_or_else(|e| eprintln!("Error: {}", e));
            }

            [_, "workspace", "list", workspace] => {
                WorkspaceManager::list(workspace).unwrap_or_else(|e| eprintln!("Error: {}", e));
            }

            [_, "workspace", "list"] => {
                WorkspaceManager::list_all().unwrap_or_else(|e| eprintln!("Error: {}", e));
            }

            [_, "workspace", "sync", workspace] => {
                WorkspaceManager::sync(workspace).unwrap_or_else(|e| eprintln!("Error: {}", e));
            }

            [_, "fmt", file] => {
                formatter::format_file(file).unwrap_or_else(|e| eprintln!("Error: {}", e));
            }

            [_, "lint", file] => {
                linter::lint_file(file).unwrap_or_else(|e| eprintln!("Error: {}", e));
            }

            [_, "completion"] => {
                println!(
                    r#"# TALON Shell Completion
complete -W "build run wasm repl install analyze doc ast completion plugin fuzz debug vscode stdlib list-templates load-template" talon
"#
                );
            }

            [_, "plugin", path] => {
                if path.ends_with(".talon") {
                    println!("Running plugin script: {}", path);
                    let script = fs::read_to_string(path).expect("Failed to load plugin");
                    let cmds = crate::parser::parse_script(&script).expect("Parse failed");
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(crate::interpreter::interpret(&cmds))
                        .expect("Plugin run failed");
                } else if path.ends_with(".so") {
                    println!("Dynamic plugin loading not implemented");
                } else {
                    println!("Unknown plugin format");
                }
            }

            [_, "fuzz", file] => {
                let script = fs::read_to_string(file).expect("Unable to read script file");
                let cmds = crate::parser::parse_script(&script).expect("Parse failed");
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(crate::interpreter::interpret(&cmds))
                    .expect("Fuzzing run failed");
            }

            [_, "load-template", name] => {
                let path = format!("talon_std/exploit/{}", name);
                let script = fs::read_to_string(&path).expect("Template missing");
                let cmds = crate::parser::parse_script(&script).expect("Parse failed");
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(crate::interpreter::interpret(&cmds))
                    .expect("Template execution failed");
            }

            [_, "list-templates"] => {
                let entries =
                    fs::read_dir("talon_std/exploit").expect("No exploit templates found");
                println!("Available exploit templates:");
                for entry in entries.flatten() {
                    println!("  - {}", entry.file_name().to_string_lossy());
                }
            }

            [_, "install", pkg] => {
                crate::package_manager::install_package(pkg);
            }

            [_, "vscode"] => {
                generate_vscode_syntax();
            }

            [_, "stdlib", file] => {
                let path = format!("talon_std/{}", file);
                let script = fs::read_to_string(&path).expect("Stdlib file not found");
                let cmds = crate::parser::parse_script(&script).expect("Parse failed");
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(crate::interpreter::interpret(&cmds))
                    .expect("Execution failed");
            }

            [_, "debug", file] => {
                let script = fs::read_to_string(file).expect("Unable to read file");
                let cmds = crate::parser::parse_script(&script).expect("Parse failed");
                for (i, cmd) in cmds.iter().enumerate() {
                    println!("[{}] {:?}", i, cmd);
                }
            }

            [_, "new", template_type, name] => {
                let gen = TemplateGenerator::new();
                match gen.generate(template_type, name) {
                    Ok(filename) => {
                        println!("{} Generated template: {}", "[OK]".green(), filename);
                        if let Some(desc) = gen.get_template_description(template_type) {
                            println!("   {}", desc.bright_black());
                        }
                    }
                    Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
                }
            }

            [_, "new", "--list"] | [_, "new"] => {
                let gen = TemplateGenerator::new();
                println!("\n{}", "Available Templates:".bold().cyan());
                for template in gen.list_templates() {
                    if let Some(desc) = gen.get_template_description(&template) {
                        println!("  {} - {}", template.green(), desc.bright_black());
                    }
                }
                println!("\nUsage: talon new <type> <name>\n");
            }

            [_, "db", "search", query] => {
                let db = ExploitDatabase::new();
                let results = db.search(query);
                println!(
                    "\n{} {} result(s) for '{}'",
                    "[DB]".blue(),
                    results.len(),
                    query
                );
                for exploit in results {
                    println!("\n  {} - {}", exploit.cve_id.yellow().bold(), exploit.title);
                    println!("    Platform: {}", exploit.platform);
                    println!("    Type: {}", exploit.exploit_type);
                    println!("    {}", exploit.description.bright_black());
                }
            }

            [_, "db", "list"] => {
                let db = ExploitDatabase::new();
                println!("\n{}", "Exploit Database:".bold().cyan());
                for exploit in db.list_all() {
                    println!(
                        "  {} - {} [{}]",
                        exploit.cve_id.yellow(),
                        exploit.title,
                        exploit.exploit_type.bright_black()
                    );
                }
            }

            [_, "db", "show", cve_id] => {
                let db = ExploitDatabase::new();
                if let Some(exploit) = db.get(cve_id) {
                    println!("\n{}", exploit.cve_id.yellow().bold());
                    println!("Title: {}", exploit.title.bold());
                    println!("Platform: {}", exploit.platform);
                    println!("Type: {}", exploit.exploit_type);
                    println!("\nDescription:");
                    println!("  {}", exploit.description);
                    println!("\nReferences:");
                    for ref_url in &exploit.references {
                        let formatted: String = ref_url.blue().underline().to_string();
                        println!("  - {}", formatted);
                    }
                    if let Some(script) = &exploit.script {
                        println!("\nExploit Script:");
                        let formatted: String = script.bright_black().to_string();
                        println!("{}", formatted);
                    }
                } else {
                    eprintln!("{} CVE not found: {}", "[ERROR]".red(), cve_id);
                }
            }

            [_, "db", "type", exploit_type] => {
                let db = ExploitDatabase::new();
                let results = db.list_by_type(exploit_type);
                println!(
                    "\n{} {} exploit(s) of type '{}'",
                    "[DB]".blue(),
                    results.len(),
                    exploit_type
                );
                for exploit in results {
                    println!("  {} - {}", exploit.cve_id.yellow(), exploit.title);
                }
            }

            [_, "db", "platform", platform] => {
                let db = ExploitDatabase::new();
                let results = db.list_by_platform(platform);
                println!(
                    "\n{} {} exploit(s) for '{}'",
                    "[DB]".blue(),
                    results.len(),
                    platform
                );
                for exploit in results {
                    println!("  {} - {}", exploit.cve_id.yellow(), exploit.title);
                }
            }

            [_, "analyze", binary_path] => match TargetDetector::analyze(binary_path) {
                Ok(info) => TargetDetector::print_analysis(&info),
                Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
            },

            [_, "diff", file1, file2] => match EnhancedBinaryDiffer::diff(file1, file2) {
                Ok(result) => EnhancedBinaryDiffer::print_analysis(&result),
                Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
            },

            [_, "suggest", binary_path] => {
                println!(
                    "\n{} Analyzing binary and suggesting exploits...",
                    "[AI]".cyan().bold()
                );
                match suggest_exploits_for_binary(binary_path) {
                    Ok(suggestions) => {
                        println!(
                            "\n{} Found {} potential exploit(s):\n",
                            "[SUCCESS]".green().bold(),
                            suggestions.len()
                        );
                        for suggestion in suggestions {
                            println!(
                                "{}. {} - {}",
                                suggestion.rank.to_string().yellow().bold(),
                                suggestion.exploit_type.cyan().bold(),
                                suggestion.technique
                            );
                            println!(
                                "   Success Probability: {}%",
                                (suggestion.success_probability * 100.0).to_string().green()
                            );
                            println!("   Complexity: {}", suggestion.complexity);
                            println!("   Description: {}", suggestion.description.bright_black());
                            if !suggestion.prerequisites.is_empty() {
                                println!("   Prerequisites:");
                                for prereq in &suggestion.prerequisites {
                                    println!("     - {}", prereq.bright_black());
                                }
                            }
                            println!();
                        }
                    }
                    Err(e) => eprintln!("{} {}", "[ERROR]".red().bold(), e),
                }
            }

            [_, "suggest", binary_path, "--generate", rank] => {
                let rank_num: usize = rank.parse().unwrap_or(1);
                println!(
                    "\n{} Generating exploit code for suggestion #{}...",
                    "[AI]".cyan().bold(),
                    rank_num
                );
                match suggest_exploits_for_binary(binary_path) {
                    Ok(suggestions) => {
                        if let Some(suggestion) = suggestions.get(rank_num - 1) {
                            let engine = SuggestionEngine::new();
                            match engine.generate_exploit_code(suggestion, binary_path) {
                                Ok(code) => {
                                    println!(
                                        "\n{} Generated Exploit Code:\n",
                                        "[SUCCESS]".green().bold()
                                    );
                                    println!("{}", "─".repeat(70).bright_black());
                                    println!("{}", code);
                                    println!("{}", "─".repeat(70).bright_black());
                                }
                                Err(e) => eprintln!("{} {}", "[ERROR]".red().bold(), e),
                            }
                        } else {
                            eprintln!(
                                "{} Suggestion #{} not found",
                                "[ERROR]".red().bold(),
                                rank_num
                            );
                        }
                    }
                    Err(e) => eprintln!("{} {}", "[ERROR]".red().bold(), e),
                }
            }

            [_, "suggest", binary_path, "--ai", api_key] => {
                println!("\n{} Analyzing with AI (OpenAI)...", "[AI]".cyan().bold());
                let engine = SuggestionEngine::with_ai(api_key.to_string(), AIProvider::OpenAI);
                match engine.analyze_binary(binary_path) {
                    Ok(fingerprint) => {
                        let suggestions = engine.suggest_exploits(&fingerprint);
                        println!(
                            "\n{} AI-Enhanced Suggestions:\n",
                            "[SUCCESS]".green().bold()
                        );
                        for suggestion in suggestions {
                            println!(
                                "{}. {} - {}",
                                suggestion.rank.to_string().yellow().bold(),
                                suggestion.exploit_type.cyan().bold(),
                                suggestion.technique
                            );
                            println!("   {}", suggestion.description);
                            println!();
                        }
                    }
                    Err(e) => eprintln!("{} {}", "[ERROR]".red().bold(), e),
                }
            }

            [_, "config", "init"] => {
                let config = TalonConfig::default();
                match config.save() {
                    Ok(_) => {
                        println!("{} Configuration file created", "[OK]".green());
                        TalonConfig::print_config_location();
                    }
                    Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
                }
            }

            [_, "config", "show"] => {
                let config = TalonConfig::load();
                println!("\n{}", "Current Configuration:".bold().cyan());
                println!("  LM Studio URL: {}", config.lm_studio_url);
                println!("  LM Studio Model: {}", config.lm_studio_model);
                println!("  Verbosity: {}", config.verbosity);
                println!("  Enable Colors: {}", config.enable_colors);
                println!("  Enable Progress Bars: {}", config.enable_progress_bars);
                println!("  Exploit DB URL: {}", config.exploit_db_url);
                println!("  Default Arch: {}", config.default_arch);
                println!("  Default OS: {}", config.default_os);
                println!("  Timeout: {} seconds", config.timeout_seconds);
                println!();
                TalonConfig::print_config_location();
            }

            [_, "config", "edit"] => {
                if let Some(path) = TalonConfig::get_config_path() {
                    println!("Config file: {}", path.display());
                    #[cfg(target_os = "windows")]
                    std::process::Command::new("notepad")
                        .arg(path)
                        .status()
                        .ok();
                    #[cfg(not(target_os = "windows"))]
                    {
                        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
                        std::process::Command::new(editor).arg(path).status().ok();
                    }
                } else {
                    eprintln!(
                        "{} Could not determine config file location",
                        "[ERROR]".red()
                    );
                }
            }

            [_, "man", "--generate"] => match ManPages::generate_all() {
                Ok(_) => println!("{} Man pages generated", "[OK]".green()),
                Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
            },

            [_, "man", topic] => {
                ManPages::display_page(topic);
            }

            [_, "completion", shell] => match CompletionGenerator::install(shell) {
                Ok(_) => println!("{} Shell completion generated", "[OK]".green()),
                Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
            },

            [_, "replay", "play", file] => match TalonReplay::load(file) {
                Ok(replay) => replay
                    .play()
                    .unwrap_or_else(|e| eprintln!("{} {}", "[ERROR]".red(), e)),
                Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
            },

            [_, "replay", "export", file, output] => match TalonReplay::load(file) {
                Ok(replay) => replay
                    .export_to_talon_script(output)
                    .unwrap_or_else(|e| eprintln!("{} {}", "[ERROR]".red(), e)),
                Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
            },

            [_, "tutorial", "first-blood"] | [_, "tutorial"] => {
                let tutorial = TutorialSystem::new();
                tutorial
                    .start_first_blood()
                    .unwrap_or_else(|e| eprintln!("{} {}", "[ERROR]".red(), e));
            }

            [_, "marketplace", "browse"] | [_, "marketplace"] => {
                match ChallengeMarketplace::browse() {
                    Ok(challenges) => {
                        println!("\n{}", "Challenge Marketplace".bold().cyan());
                        println!("{}", "=".repeat(70));
                        for challenge in challenges {
                            println!(
                                "\n{} - {} ({})",
                                challenge.id.yellow(),
                                challenge.title.bold(),
                                challenge.difficulty.green()
                            );
                            println!(
                                "  Author: {} | Rating: {}/5.0 | Downloads: {}",
                                challenge.author, challenge.rating, challenge.downloads
                            );
                            println!("  {}", challenge.description.bright_black());
                        }
                        println!();
                    }
                    Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
                }
            }

            [_, "marketplace", "download", id] => match ChallengeMarketplace::download(id) {
                Ok(file) => println!("{} Downloaded to {}", "[OK]".green(), file),
                Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
            },

            [_, "translate", "from-pwntools", file] => {
                if let Ok(python_code) = fs::read_to_string(file) {
                    match ScriptTranslator::from_pwntools(&python_code) {
                        Ok(talon_code) => println!("{}", talon_code),
                        Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
                    }
                } else {
                    eprintln!("{} Failed to read file", "[ERROR]".red());
                }
            }

            [_, "translate", "to-pwntools", file] => {
                if let Ok(talon_code) = fs::read_to_string(file) {
                    match ScriptTranslator::to_pwntools(&talon_code) {
                        Ok(python_code) => println!("{}", python_code),
                        Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
                    }
                } else {
                    eprintln!("{} Failed to read file", "[ERROR]".red());
                }
            }

            [_, "oneliner", "get-shell", target, port] => {
                let port_num: u16 = port.parse().unwrap_or(1337);
                println!("{}", OneLinerLibrary::get_shell(target, port_num));
            }

            [_, "oneliner", "leak-libc", target, port] => {
                let port_num: u16 = port.parse().unwrap_or(1337);
                println!("{}", OneLinerLibrary::leak_libc(target, port_num));
            }

            [_, "oneliner", "rop-chain", target, port] => {
                let port_num: u16 = port.parse().unwrap_or(1337);
                println!("{}", OneLinerLibrary::rop_chain(target, port_num));
            }

            [_, "template", template_name, target, port] => {
                let port_num: u16 = port.parse().unwrap_or(1337);
                if let Some(template) =
                    OneLinerLibrary::get_template(template_name, target, port_num)
                {
                    println!("{}", template);
                } else {
                    eprintln!("Unknown template: {}", template_name);
                    eprintln!("\nAvailable templates:");
                    for tmpl in OneLinerLibrary::list_templates() {
                        eprintln!("  - {}", tmpl);
                    }
                }
            }

            _ => {
                println!(
                    r#"
TALON CLI – Modular Red Team DSL

Usage:
  talon build <file>         → Compile to native ELF binary
  talon run <file>           → Interpret .talon file
  talon wasm <file>          → Compile to WebAssembly
  talon analyze <file>       → Run static + RE analysis
  talon ast <file>           → Emit DOT/Graphviz for AST
  talon doc <file>           → Write markdown AST doc
  talon repl                 → Start interactive shell
  talon install <package>    → Install stdlib module
  talon fuzz <file>          → Run in-script fuzzer
  talon list-templates       → List exploit templates
  talon load-template <x>    → Load stdlib exploit
  talon stdlib <file>        → Execute stdlib module
  talon plugin <.talon|.so>  → Load plugin script
  talon vscode               → Write VSCode syntax JSON
  talon completion           → Emit shell autocompletion

Flags:
  --static                   → Build statically linked binary
  --run                      → Run after successful build
"#
                );
            }
        },
    }
}

/// Print quick reference card
fn print_quick_reference() {
    println!(
        r#"
═══════════════════════════════════════════════════════════════════════════
                     TALON DSL - QUICK REFERENCE CARD
═══════════════════════════════════════════════════════════════════════════

PACKING & UNPACKING
───────────────────────────────────────────────────────────────────────────
  addr | p64              Pack 64-bit address (little-endian)
  value | p32             Pack 32-bit value
  data | p16              Pack 16-bit value
  byte | p8               Pack 8-bit value

  u64(bytes)              Unpack 64-bit from bytes
  u32(bytes)              Unpack 32-bit from bytes

SPREAD OPERATOR
───────────────────────────────────────────────────────────────────────────
  [...header, ...body]    Unpack lists into new collection
  let payload = [...rop_chain, ...shellcode, ...footer]

PIPE OPERATOR
───────────────────────────────────────────────────────────────────────────
  addr | p64 | send       Chain operations Unix-style
  leak | u64 | compute_base

RANGES & SLICING
───────────────────────────────────────────────────────────────────────────
  for i in 0..100         Iterate numeric range
  for i in 0..n           Variable upper bound
  data[10..20]            Slice list or string
  payload[0..8]           Extract first 8 elements

ROP CHAIN GENERATION
───────────────────────────────────────────────────────────────────────────
  auto_rop "./binary"
    constraints: [no_nulls, alphanumeric]
    objective: shell
    strategy: ret2libc
  end

HEAP EXPLOITATION
───────────────────────────────────────────────────────────────────────────
  heap_exploit "./binary"
    technique: tcache_poisoning
    target: __malloc_hook
    overwrite_with: system
    glibc_version: "2.35"
  end

SAFETY CONTROLS
───────────────────────────────────────────────────────────────────────────
  enable_strict_mode      Enable all safety checks
  set_timeout 30000       Set 30-second execution limit
  set_max_memory 512      Limit memory to 512 MB
  set_recursion_depth 100 Limit recursion depth
  disable_safety          Disable all safety checks

COMMON PATTERNS
───────────────────────────────────────────────────────────────────────────
  let data = recv 1024           Receive 1024 bytes
  send payload                   Send payload to target

  if canary != expected          Conditional checks
    print "Canary mismatch"
  end

  try                            Error handling
    risky_operation
  catch e
    print "Error: " + e
  end

TEMPLATE GENERATION
───────────────────────────────────────────────────────────────────────────
  talon new buffer-overflow exploit    Classic buffer overflow
  talon new rop chain                  ROP chain builder
  talon new ret2libc attack            Return-to-libc
  talon new heap exploit               Heap exploitation
  talon new kernel privesc             Kernel privilege escalation

USEFUL COMMANDS
───────────────────────────────────────────────────────────────────────────
  talon run script.tal           Execute TALON script
  talon repl                     Start interactive shell
  talon new <type> <name>        Generate exploit template
  talon man <topic>              View manual page
  talon quick-ref                Show this reference (alias: qref)

═══════════════════════════════════════════════════════════════════════════
            For detailed documentation: talon man talon
═══════════════════════════════════════════════════════════════════════════
"#
    );
}

/// Generate VSCode syntax highlighting file
fn generate_vscode_syntax() {
    let syntax = r#"
{
  "name": "Talon",
  "scopeName": "source.talon",
  "patterns": [
    {
      "name": "keyword.control.talon",
      "match": "\\b(if|else|for|define|function|struct|let|connect|generate|exit|end)\\b"
    },
    {
      "name": "string.quoted.double.talon",
      "begin": "\"",
      "end": "\""
    },
    {
      "name": "constant.numeric.talon",
      "match": "\\b\\d+\\b"
    },
    {
      "name": "entity.name.function.talon",
      "match": "\\b(parse|call|disassemble|assemble|build|fuzz|encrypt|sign)\\b"
    }
  ]
}
"#;
    let mut file = File::create("talon.tmLanguage.json").expect("Failed to create syntax file");
    file.write_all(syntax.as_bytes()).expect("Write error");
    println!(
        "{}",
        "[VSCODE] Generated syntax: talon.tmLanguage.json".green()
    );
}
