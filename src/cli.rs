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
    let mut static_build = false;
    let mut auto_run = false;
    let mut script_path = None;

    for arg in &args {
        match arg.as_str() {
            "--static" => static_build = true,
            "--run" => auto_run = true,
            val if !val.starts_with("--") && script_path.is_none() => {
                script_path = Some(val.to_string());
            }
            _ => {}
        }
    }

    match args
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .as_slice()
    {
        [_, "build", "--matrix"] => {
            use crate::matrix_builder::MatrixBuilder;
            
            let builder = MatrixBuilder::new();
            builder.build_matrix().expect("Matrix build failed");
            builder.verify_static_linking().expect("Verification failed");
        }

        [_, "build", file] => {
            let script = fs::read_to_string(file).expect("Unable to read script file");
            let cmds = crate::parser::parse_script(&script).expect("Parse failed");
            crate::codegen::build_script(&cmds, static_build).expect("Build failed");

            println!(
                "{}",
                "[BUILD] Binary: ./talon_build/target/release/talon_script".green()
            );

            if auto_run {
                println!("{}", "[RUN] Executing compiled binary...".yellow());
                std::process::Command::new("./talon_build/target/release/talon_script")
                    .status()
                    .expect("Execution failed");
            }
        }

        [_, "build", file, "--evasion-level", level] => {
            let script = fs::read_to_string(file).expect("Unable to read script file");
            let cmds = crate::parser::parse_script(&script).expect("Parse failed");
            
            let evasion_level = match *level {
                "low" | "medium" | "high" => level,
                _ => {
                    eprintln!("{} Invalid evasion level. Use: low, medium, or high", "[ERROR]".red());
                    return;
                }
            };
            
            println!("{} Building with {} evasion level...", "[BUILD]".blue(), evasion_level.yellow());
            
            crate::codegen::build_script(&cmds, static_build).expect("Build failed");
            
            let binary_path = "./talon_build/target/release/talon_script";
            
            #[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
            {
                if let Ok(shellcode) = fs::read(binary_path) {
                    use crate::opsec::polymorphic::PolymorphicEngine;
                    use crate::opsec::polymorphic::MutationStrategy;
                    
                    let strategies = match evasion_level {
                        "low" => vec![MutationStrategy::JunkCodeInsertion],
                        "medium" => vec![
                            MutationStrategy::JunkCodeInsertion,
                            MutationStrategy::InstructionEquivalence,
                        ],
                        "high" => vec![
                            MutationStrategy::JunkCodeInsertion,
                            MutationStrategy::InstructionEquivalence,
                            MutationStrategy::StringEncryption,
                            MutationStrategy::RegisterPermutation,
                        ],
                        _ => vec![],
                    };
                    
                    let junk_density = match evasion_level {
                        "low" => 0.1,
                        "medium" => 0.3,
                        "high" => 0.5,
                        _ => 0.1,
                    };
                    
                    let mut engine = PolymorphicEngine::new(strategies);
                    engine.set_junk_density(junk_density);
                    
                    match engine.mutate(&shellcode) {
                        Ok(mutated) => {
                            let output_path = format!("{}.polymorphic", binary_path);
                            fs::write(&output_path, mutated).expect("Failed to write polymorphic binary");
                            println!("{} Polymorphic binary: {}", "[OK]".green(), output_path);
                            println!("{} Evasion techniques applied: {}", "[INFO]".blue(), engine.strategies.len());
                        }
                        Err(e) => eprintln!("{} Polymorphic transformation failed: {}", "[ERROR]".red(), e),
                    }
                }
            }
            
            #[cfg(not(all(target_os = "windows", feature = "game-hacking-windows")))]
            {
                println!("{} Polymorphic evasion not available (requires Windows + game-hacking-windows feature)", "[WARN]".yellow());
                println!("{} Built binary available at: {}", "[OK]".green(), binary_path);
            }
        }

        [_, "run", file] => {
            let script = fs::read_to_string(file).expect("Unable to read script file");
            let cmds = crate::parser::parse_script(&script).expect("Parse failed");
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(crate::interpreter::interpret(&cmds)).expect("Interpretation failed");
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
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                rt.block_on(crate::interpreter::interpret(&cmds)).expect("Plugin run failed");
            } else if path.ends_with(".so") {
                println!("Dynamic plugin loading not implemented");
            } else {
                println!("Unknown plugin format");
            }
        }

        [_, "fuzz", file] => {
            let script = fs::read_to_string(file).expect("Unable to read script file");
            let cmds = crate::parser::parse_script(&script).expect("Parse failed");
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(crate::interpreter::interpret(&cmds)).expect("Fuzzing run failed");
        }

        [_, "load-template", name] => {
            let path = format!("talon_std/exploit/{}", name);
            let script = fs::read_to_string(&path).expect("Template missing");
            let cmds = crate::parser::parse_script(&script).expect("Parse failed");
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(crate::interpreter::interpret(&cmds)).expect("Template execution failed");
        }

        [_, "list-templates"] => {
            let entries = fs::read_dir("talon_std/exploit").expect("No exploit templates found");
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
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(crate::interpreter::interpret(&cmds)).expect("Execution failed");
        }

        [_, "debug", file] => {
            println!("{} Time-travel debugging mode for: {}", "[DEBUG]".cyan(), file);
            let script = fs::read_to_string(file).expect("Unable to read file");
            let cmds = crate::parser::parse_script(&script).expect("Parse failed");
            
            println!("\n{}", "Time-Travel Debugger".bold().cyan());
            println!("{}", "═".repeat(70));
            println!("\nLoaded script with {} commands", cmds.len());
            println!("\nAST Preview:");
            for (i, cmd) in cmds.iter().take(5).enumerate() {
                println!("  [{}] {:?}", i, cmd);
            }
            if cmds.len() > 5 {
                println!("  ... and {} more commands", cmds.len() - 5);
            }
            
            println!("\n{}", "Debugger Commands:".bold());
            println!("  run           - Execute script with time-travel recording");
            println!("  checkpoints   - List saved checkpoints");
            println!("  ast           - Show full AST");
            println!("  exit          - Exit debugger");
            
            use std::io::{self, Write};
            loop {
                print!("\nDebug> ");
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let choice = input.trim();
                
                match choice {
                    "run" => {
                        println!("{} Executing script with time-travel recording...", "[DEBUG]".blue());
                        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                        match rt.block_on(crate::interpreter::interpret(&cmds)) {
                            Ok(_) => println!("{} Execution completed", "[OK]".green()),
                            Err(e) => eprintln!("{} Execution failed: {}", "[ERROR]".red(), e),
                        }
                    }
                    "checkpoints" => {
                        use crate::build_cache::BuildCache;
                        if let Ok(cache) = BuildCache::new() {
                            let checkpoint_dir = cache.cache_dir.join("checkpoints");
                            if checkpoint_dir.exists() {
                                match fs::read_dir(&checkpoint_dir) {
                                    Ok(entries) => {
                                        let checkpoints: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                                        if checkpoints.is_empty() {
                                            println!("No checkpoints found");
                                        } else {
                                            println!("\n{} Saved checkpoints:", "[INFO]".blue());
                                            for (idx, entry) in checkpoints.iter().enumerate() {
                                                println!("  {}. {}", idx + 1, entry.file_name().to_string_lossy());
                                            }
                                        }
                                    }
                                    Err(e) => eprintln!("{} Failed to list checkpoints: {}", "[ERROR]".red(), e),
                                }
                            } else {
                                println!("No checkpoints directory found");
                            }
                        }
                    }
                    "ast" => {
                        println!("\n{} Full AST:", "[INFO]".blue());
                        for (i, cmd) in cmds.iter().enumerate() {
                            println!("  [{}] {:?}", i, cmd);
                        }
                    }
                    "exit" => {
                        println!("Exiting debugger...");
                        break;
                    }
                    _ => println!("Unknown command. Use: run, checkpoints, ast, exit"),
                }
            }
        }

        [_, "new", "--interactive"] | [_, "new", "-i"] => {
            let gen = TemplateGenerator::new();
            match gen.run_interactive_wizard() {
                Ok(filename) => {
                    println!("{} Generated template: {}", "[OK]".green(), filename);
                }
                Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
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
            println!("\nUsage:");
            println!("  talon new <type> <name>          Generate template from type");
            println!("  talon new --interactive          Interactive template wizard");
            println!("  talon new --list                 List all templates\n");
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

        [_, "oracle", binary_path] => {
            println!("{} Analyzing binary for vulnerabilities...", "[ORACLE]".cyan());
            match crate::oracle::VulnerabilityOracle::new(binary_path) {
                Ok(mut oracle) => {
                    match oracle.analyze_flow() {
                        Ok(reports) => {
                            if reports.is_empty() {
                                println!("{} No vulnerabilities detected", "[ORACLE]".green());
                            } else {
                                println!("\n{} Found {} potential vulnerabilities:\n", "[ORACLE]".yellow(), reports.len());
                                for (idx, report) in reports.iter().enumerate() {
                                    println!("{}. {}", (idx + 1).to_string().yellow().bold(), format!("{}", report.vuln_type).cyan().bold());
                                    println!("   Location: {}", report.location);
                                    println!("   Confidence: {:.1}%", report.confidence * 100.0);
                                    println!("   Exploitability: {:?}", report.exploitability);
                                    println!("   Details: {}", report.details.bright_black());
                                    
                                    if let Some(ref suggested) = report.suggested_exploit {
                                        println!("   Suggested Exploit: {}", suggested.bright_black());
                                    }
                                    println!();
                                }
                            }
                        }
                        Err(e) => eprintln!("{} Analysis failed: {}", "[ERROR]".red(), e),
                    }
                }
                Err(e) => eprintln!("{} Failed to create oracle: {}", "[ERROR]".red(), e),
            }
        }

        [_, "patch", binary_path] => {
            println!("{} Interactive binary patching for: {}", "[PATCH]".cyan(), binary_path);
            use crate::binary_patch::Patch;
            
            match Patch::new(binary_path) {
                Ok(mut patcher) => {
                    patcher.set_dry_run(true);
                    
                    println!("\n{}", "Binary Patcher - Interactive Mode".bold().cyan());
                    println!("{}", "═".repeat(70));
                    println!("\nOptions:");
                    println!("  1. NOP out function call");
                    println!("  2. Replace function call");
                    println!("  3. Insert assembly");
                    println!("  4. Patch bytes");
                    println!("  5. Inject shellcode");
                    println!("  6. Create code cave");
                    println!("  7. Find pattern");
                    println!("  8. Apply patches (disable dry-run)");
                    println!("  9. Show patch history");
                    println!("  0. Exit");
                    
                    use std::io::{self, Write};
                    loop {
                        print!("\nPatch> ");
                        io::stdout().flush().unwrap();
                        
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();
                        let choice = input.trim();
                        
                        match choice {
                            "1" => {
                                print!("Offset (hex): ");
                                io::stdout().flush().unwrap();
                                input.clear();
                                io::stdin().read_line(&mut input).unwrap();
                                if let Ok(offset) = usize::from_str_radix(input.trim().trim_start_matches("0x"), 16) {
                                    print!("Length: ");
                                    io::stdout().flush().unwrap();
                                    input.clear();
                                    io::stdin().read_line(&mut input).unwrap();
                                    if let Ok(length) = input.trim().parse::<usize>() {
                                        match patcher.nop_out(offset, length) {
                                            Ok(_) => println!("{} NOPs inserted", "[OK]".green()),
                                            Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
                                        }
                                    }
                                }
                            }
                            "2" => {
                                print!("Call offset (hex, e.g., 0x401234): ");
                                io::stdout().flush().unwrap();
                                input.clear();
                                io::stdin().read_line(&mut input).unwrap();
                                let offset_str = input.trim();
                                
                                let call_offset = if offset_str.starts_with("0x") {
                                    usize::from_str_radix(&offset_str[2..], 16)
                                } else {
                                    offset_str.parse::<usize>()
                                };
                                
                                match call_offset {
                                    Ok(offset) => {
                                        print!("New function name: ");
                                        io::stdout().flush().unwrap();
                                        input.clear();
                                        io::stdin().read_line(&mut input).unwrap();
                                        let new_fn = input.trim();
                                        
                                        match patcher.replace_call(offset, new_fn) {
                                            Ok(_) => println!("{} Call replaced", "[OK]".green()),
                                            Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
                                        }
                                    }
                                    Err(e) => eprintln!("{} Invalid offset: {}", "[ERROR]".red(), e),
                                }
                            }
                            "7" => {
                                print!("Pattern (hex bytes, space-separated): ");
                                io::stdout().flush().unwrap();
                                input.clear();
                                io::stdin().read_line(&mut input).unwrap();
                                
                                let pattern: Result<Vec<u8>, _> = input.trim()
                                    .split_whitespace()
                                    .map(|b| u8::from_str_radix(b, 16))
                                    .collect();
                                
                                if let Ok(pattern) = pattern {
                                    let offsets = patcher.find_pattern(&pattern);
                                    if offsets.is_empty() {
                                        println!("{} Pattern not found", "[INFO]".cyan());
                                    } else {
                                        println!("{} Found {} occurrence(s):", "[OK]".green(), offsets.len());
                                        for offset in offsets {
                                            println!("  0x{:x}", offset);
                                        }
                                    }
                                }
                            }
                            "8" => {
                                patcher.set_dry_run(false);
                                println!("{} Dry-run mode disabled. Patches will be applied to file.", "[WARNING]".yellow());
                            }
                            "9" => {
                                let ops = patcher.get_operations();
                                if ops.is_empty() {
                                    println!("No patches queued");
                                } else {
                                    println!("\n{} Queued patches:", "[INFO]".blue());
                                    for (idx, op) in ops.iter().enumerate() {
                                        println!("  {}. {:?}", idx + 1, op);
                                    }
                                }
                            }
                            "0" => {
                                println!("Exiting...");
                                break;
                            }
                            _ => println!("Invalid option"),
                        }
                    }
                }
                Err(e) => eprintln!("{} Failed to create patcher: {}", "[ERROR]".red(), e),
            }
        }

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

        [_, "learn"] => {
            TutorialSystem::select_tutorial()
                .unwrap_or_else(|e| eprintln!("{} {}", "[ERROR]".red(), e));
        }

        [_, "learn", "first-blood"] => {
            let tutorial = TutorialSystem::new();
            tutorial
                .start_first_blood()
                .unwrap_or_else(|e| eprintln!("{} {}", "[ERROR]".red(), e));
        }

        [_, "learn", "bandit"] => {
            let tutorial = TutorialSystem::new_bandit();
            tutorial
                .start_bandit()
                .unwrap_or_else(|e| eprintln!("{} {}", "[ERROR]".red(), e));
        }

        [_, "marketplace", "browse"] | [_, "marketplace"] => match ChallengeMarketplace::browse() {
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
        },

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

        [_, "audit", "--ai", binary] | [_, "audit", binary, "--ai"] => {
            println!("{}", "[AI] Initializing ML Oracle...".cyan());
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(async {
                let mut ml_oracle = crate::ml_oracle::MlOracle::new();
                match ml_oracle.initialize().await {
                    Ok(_) => {
                        println!("{}", "[AI] ML Oracle available".green());
                        match crate::oracle::VulnerabilityOracle::new(binary) {
                            Ok(mut oracle) => {
                                match oracle.analyze_with_ai(&ml_oracle).await {
                                    Ok(reports) => {
                                        println!("\n{} Found {} vulnerabilities", "[AUDIT]".green(), reports.len());
                                        for report in reports {
                                            println!("\n{}", "=".repeat(70));
                                            println!("{}: {}", "Type".bold(), report.vuln_type);
                                            println!("{}: {}", "Location".bold(), report.location);
                                            println!("{}: {:.2}", "Confidence".bold(), report.confidence);
                                            println!("{}: {:?}", "Exploitability".bold(), report.exploitability);
                                            println!("\n{}", report.details);
                                        }
                                    }
                                    Err(e) => eprintln!("{} Analysis failed: {}", "[ERROR]".red(), e),
                                }
                            }
                            Err(e) => eprintln!("{} Failed to create oracle: {}", "[ERROR]".red(), e),
                        }
                    }
                    Err(e) => {
                        eprintln!("{} AI features unavailable: {}", "[WARNING]".yellow(), e);
                        eprintln!("Running heuristic analysis only...");
                        match crate::oracle::VulnerabilityOracle::new(binary) {
                            Ok(mut oracle) => {
                                match oracle.analyze_flow() {
                                    Ok(reports) => {
                                        println!("\n{} Found {} vulnerabilities", "[AUDIT]".green(), reports.len());
                                        for report in reports {
                                            println!("\n{}", "=".repeat(70));
                                            println!("{}: {}", "Type".bold(), report.vuln_type);
                                            println!("{}: {}", "Location".bold(), report.location);
                                            println!("{}: {:.2}", "Confidence".bold(), report.confidence);
                                        }
                                    }
                                    Err(e) => eprintln!("{} Analysis failed: {}", "[ERROR]".red(), e),
                                }
                            }
                            Err(e) => eprintln!("{} Failed to create oracle: {}", "[ERROR]".red(), e),
                        }
                    }
                }
            });
        }

        [_, "explain", "--ai", error_msg] => {
            println!("{}", "[AI] Initializing ML Oracle...".cyan());
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(async {
                let mut ml_oracle = crate::ml_oracle::MlOracle::new();
                match ml_oracle.initialize().await {
                    Ok(_) => {
                        println!("{}", "[AI] ML Oracle available".green());
                        match ml_oracle.explain_error(error_msg, "").await {
                            Ok(explanation) => {
                                println!("\n{}", "=".repeat(70));
                                println!("{}", "Error Explanation".bold().cyan());
                                println!("{}", "=".repeat(70));
                                println!("\n{}", explanation);
                            }
                            Err(e) => eprintln!("{} Explanation failed: {}", "[ERROR]".red(), e),
                        }
                    }
                    Err(e) => eprintln!("{} AI features unavailable: {}", "[ERROR]".red(), e),
                }
            });
        }

        [_, "suggest", "--ai", script_file] => {
            println!("{}", "[AI] Initializing AI integration...".cyan());
            if let Ok(script) = fs::read_to_string(script_file) {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                rt.block_on(async {
                    let ai = crate::ai_integration::AiIntegration::new(true);
                    match ai.initialize().await {
                        Ok(_) => {
                            println!("{}", "[AI] AI available".green());
                            match ai.review_exploit(&script).await {
                                Ok(suggestions) => {
                                    println!("\n{}", "=".repeat(70));
                                    println!("{}", "Code Review Suggestions".bold().cyan());
                                    println!("{}", "=".repeat(70));
                                    println!("\n{}", suggestions);
                                }
                                Err(e) => eprintln!("{} Review failed: {}", "[ERROR]".red(), e),
                            }
                        }
                        Err(e) => eprintln!("{} AI features unavailable: {}", "[ERROR]".red(), e),
                    }
                });
            } else {
                eprintln!("{} Failed to read file: {}", "[ERROR]".red(), script_file);
            }
        }

        [_, "fix", "--ai", script_file] => {
            println!("{}", "[AI] Initializing AI integration for script fixing...".cyan());
            if let Ok(script) = fs::read_to_string(script_file) {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                rt.block_on(async {
                    let ai = crate::ai_integration::AiIntegration::new(true);
                    match ai.initialize().await {
                        Ok(_) => {
                            println!("{}", "[AI] AI available".green());
                            match ai.fix_script(&script, "User requested auto-fix").await {
                                Ok(fixed_code) => {
                                    println!("\n{}", "=".repeat(70));
                                    println!("{}", "Fixed Script".bold().green());
                                    println!("{}", "=".repeat(70));
                                    println!("\n{}", fixed_code);
                                    println!("\n{}", "=".repeat(70));
                                    println!("{}", "Save the above code to your .talon file".yellow());
                                }
                                Err(e) => eprintln!("{} Fix failed: {}", "[ERROR]".red(), e),
                            }
                        }
                        Err(e) => eprintln!("{} AI features unavailable: {}", "[ERROR]".red(), e),
                    }
                });
            } else {
                eprintln!("{} Failed to read file: {}", "[ERROR]".red(), script_file);
            }
        }

        [_, "document", "--ai", script_file] => {
            println!("{}", "[AI] Initializing AI integration for documentation generation...".cyan());
            if let Ok(script) = fs::read_to_string(script_file) {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                rt.block_on(async {
                    let ai = crate::ai_integration::AiIntegration::new(true);
                    match ai.initialize().await {
                        Ok(_) => {
                            println!("{}", "[AI] AI available".green());
                            match ai.generate_documentation(&script).await {
                                Ok(documented_code) => {
                                    println!("\n{}", "=".repeat(70));
                                    println!("{}", "Documented Script".bold().cyan());
                                    println!("{}", "=".repeat(70));
                                    println!("\n{}", documented_code);
                                    println!("\n{}", "=".repeat(70));
                                    println!("{}", "Save the above code to your .talon file".yellow());
                                }
                                Err(e) => eprintln!("{} Documentation generation failed: {}", "[ERROR]".red(), e),
                            }
                        }
                        Err(e) => eprintln!("{} AI features unavailable: {}", "[ERROR]".red(), e),
                    }
                });
            } else {
                eprintln!("{} Failed to read file: {}", "[ERROR]".red(), script_file);
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
            if let Some(template) = OneLinerLibrary::get_template(template_name, target, port_num) {
                println!("{}", template);
            } else {
                eprintln!("Unknown template: {}", template_name);
                eprintln!("\nAvailable templates:");
                for tmpl in OneLinerLibrary::list_templates() {
                    eprintln!("  - {}", tmpl);
                }
            }
        }

        [_, "cache", "stats"] => {
            use crate::build_cache::BuildCache;
            match BuildCache::new() {
                Ok(cache) => match cache.get_cache_stats() {
                    Ok(stats) => {
                        println!("\n{}", "Build Cache Statistics:".bold().cyan());
                        println!("  Total entries: {}", stats.total_entries);
                        println!("  Total size: {:.2} MB", stats.total_size_mb());
                        println!("  Cache location: {}", cache.cache_dir.display());
                    }
                    Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
                },
                Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
            }
        }

        [_, "cache", "clean"] => {
            use crate::build_cache::BuildCache;
            match BuildCache::new() {
                Ok(cache) => {
                    println!("{} Cleaning old cache entries...", "[CACHE]".blue());
                    match cache.clean_old_entries(30) {
                        Ok(cleaned) => {
                            println!("{} Cleaned {} old entries", "[OK]".green(), cleaned);
                        }
                        Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
                    }
                }
                Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
            }
        }

        [_, "cache", "clean", days] => {
            use crate::build_cache::BuildCache;
            let max_age: u64 = days.parse().unwrap_or(30);
            match BuildCache::new() {
                Ok(cache) => {
                    println!("{} Cleaning entries older than {} days...", "[CACHE]".blue(), max_age);
                    match cache.clean_old_entries(max_age) {
                        Ok(cleaned) => {
                            println!("{} Cleaned {} old entries", "[OK]".green(), cleaned);
                        }
                        Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
                    }
                }
                Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
            }
        }

        #[cfg(feature = "swarm")]
        [_, "swarm", "deploy", inventory_path] => {
            use crate::cloud::{SwarmController, SwarmConfig};
            use std::path::PathBuf;
            
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(async {
                let config = SwarmConfig::default();
                let controller = SwarmController::new(config).await
                    .expect("Failed to create swarm controller");
                
                let inventory_path = PathBuf::from(inventory_path);
                match controller.load_inventory(&inventory_path).await {
                    Ok(count) => {
                        println!("{} Deployed to {} agents from {}", 
                            "[SWARM]".blue(), count, inventory_path.display());
                    }
                    Err(e) => eprintln!("{} Failed to load inventory: {}", "[ERROR]".red(), e),
                }
            });
        }

        #[cfg(feature = "swarm")]
        [_, "swarm", "run", script_path, "--agents-from", inventory_path] => {
            use crate::cloud::{SwarmController, SwarmConfig, ExecutionRequest, TargetAgents};
            use std::path::PathBuf;
            
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(async {
                let config = SwarmConfig::default();
                let controller = SwarmController::new(config).await
                    .expect("Failed to create swarm controller");
                
                controller.load_inventory(&PathBuf::from(inventory_path)).await
                    .expect("Failed to load inventory");
                
                let request = ExecutionRequest {
                    script_path: PathBuf::from(script_path),
                    target_agents: TargetAgents::All,
                    dry_run: false,
                    timeout_seconds: 300,
                    max_retries: 3,
                };
                
                match controller.execute_script(request).await {
                    Ok(results) => {
                        println!("\n{}", "Swarm Execution Results:".bold().cyan());
                        println!("  Total agents: {}", results.total_agents);
                        println!("  Successful: {}", results.successful);
                        println!("  Failed: {}", results.failed);
                        println!("  Execution time: {}ms", results.execution_time_ms);
                        
                        if results.failed > 0 {
                            println!("\n{}", "Failed Agents:".bold().red());
                            for result in results.results.iter().filter(|r| !r.success) {
                                println!("  - {}: {}", result.target_host, result.error_message);
                            }
                        }
                    }
                    Err(e) => eprintln!("{} Script execution failed: {}", "[ERROR]".red(), e),
                }
            });
        }

        #[cfg(feature = "swarm")]
        [_, "swarm", "status"] => {
            use crate::cloud::{SwarmController, SwarmConfig};
            
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(async {
                let config = SwarmConfig::default();
                let controller = SwarmController::new(config).await
                    .expect("Failed to create swarm controller");
                
                let agents = controller.list_agents().await;
                
                println!("\n{}", "Swarm Agent Status:".bold().cyan());
                println!("{}", "─".repeat(80));
                
                if agents.is_empty() {
                    println!("No agents registered. Use 'talon swarm deploy' to load agents.");
                } else {
                    for agent in agents.iter() {
                        let status = if agent.active { "ACTIVE".green() } else { "INACTIVE".red() };
                        println!(
                            "  {} {} ({}/{})",
                            status,
                            agent.hostname,
                            agent.os,
                            agent.arch
                        );
                        println!("    ID: {}", agent.agent_id);
                        println!("    Capabilities: {}", agent.capabilities.join(", "));
                        if !agent.tags.is_empty() {
                            println!("    Tags: {}", agent.tags.join(", "));
                        }
                        println!();
                    }
                }
            });
        }

        #[cfg(feature = "swarm")]
        [_, "swarm", "results", script_id] => {
            use crate::cloud::{SwarmController, SwarmConfig};
            
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(async {
                let config = SwarmConfig::default();
                let controller = SwarmController::new(config).await
                    .expect("Failed to create swarm controller");
                
                match controller.get_results(script_id).await {
                    Some(results) => {
                        println!("\n{}", "Execution Results:".bold().cyan());
                        println!("{}", "─".repeat(80));
                        
                        for result in results.iter() {
                            let status = if result.success { "SUCCESS".green() } else { "FAILED".red() };
                            println!("  {} {}", status, result.target_host);
                            println!("    Duration: {}ms", result.duration_ms);
                            
                            if !result.success && !result.error_message.is_empty() {
                                println!("    Error: {}", result.error_message);
                            }
                            
                            if !result.loot.is_empty() {
                                println!("    Loot: {} bytes", result.loot.len());
                            }
                            println!();
                        }
                    }
                    None => {
                        eprintln!("{} No results found for script ID: {}", "[ERROR]".red(), script_id);
                    }
                }
            });
        }

        #[cfg(not(feature = "swarm"))]
        [_, "swarm", ..] => {
            eprintln!("{}", "[ERROR] Swarm mode not enabled. Rebuild with --features swarm".red());
        }

        #[cfg(feature = "swarm")]
        [_, "agent", "--connect", primary_endpoint] => {
            use crate::cloud::{Agent, AgentConfig};
            use std::path::PathBuf;
            
            println!("{} Starting agent and connecting to: {}", "[AGENT]".cyan(), primary_endpoint);
            
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(async {
                let config = AgentConfig {
                    primary_endpoint: primary_endpoint.to_string(),
                    agent_id: format!("agent-{}", uuid::Uuid::new_v4()),
                    capabilities: vec![
                        "binary_analysis".to_string(),
                        "network_exploit".to_string(),
                        "rop_chain".to_string(),
                    ],
                    max_concurrent_scripts: 4,
                    heartbeat_interval_secs: 30,
                    reconnect_delay_secs: 5,
                    max_reconnect_attempts: 10,
                };
                
                match Agent::new(config).await {
                    Ok(mut agent) => {
                        println!("{} Agent initialized: {}", "[OK]".green(), agent.agent_id);
                        println!("{} Capabilities: {:?}", "[INFO]".blue(), agent.capabilities);
                        println!("{} Connecting to primary...", "[AGENT]".cyan());
                        
                        match agent.connect().await {
                            Ok(_) => {
                                println!("{} Connected to swarm controller", "[CONNECTED]".green());
                                println!("{} Agent is ready to receive scripts", "[READY]".green());
                                
                                println!("\nPress Ctrl+C to stop agent");
                                
                                match agent.start_heartbeat().await {
                                    Ok(_) => {
                                        println!("{} Agent running. Waiting for scripts...", "[AGENT]".cyan());
                                        
                                        tokio::signal::ctrl_c().await.ok();
                                        println!("\n{} Shutting down agent...", "[AGENT]".yellow());
                                    }
                                    Err(e) => eprintln!("{} Heartbeat failed: {}", "[ERROR]".red(), e),
                                }
                            }
                            Err(e) => eprintln!("{} Connection failed: {}", "[ERROR]".red(), e),
                        }
                    }
                    Err(e) => eprintln!("{} Failed to create agent: {}", "[ERROR]".red(), e),
                }
            });
        }

        #[cfg(not(feature = "swarm"))]
        [_, "agent", ..] => {
            eprintln!("{}", "[ERROR] Agent mode not enabled. Rebuild with --features swarm".red());
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

Cache Management:
  talon cache stats          → Display cache statistics
  talon cache clean          → Clean entries older than 30 days
  talon cache clean <days>   → Clean entries older than specified days

Flags:
  --static                   → Build statically linked binary
  --run                      → Run after successful build
"#
            );
        }
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
