use crate::helpers::{DocGenerator, ErrorHelper, ScriptHelper};
use crate::interpreter::interpret;
use crate::parser::parse_script;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{hint::HistoryHinter, Context, Editor, Helper};
use std::borrow::Cow;
use std::collections::HashMap;

struct TalonCompleter {
    hinter: HistoryHinter,
}

impl Completer for TalonCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let mut candidates = Vec::new();

        let functions = vec![
            "cyclic",
            "cyclic_find",
            "shellcode",
            "rop_find",
            "fmtstr_payload",
            "interactive",
            "disasm",
            "connect",
            "analyze",
            "fuzz",
            "parse",
            "scan",
            "detect",
            "execute",
            "load",
            "print",
            "hex",
            "base64",
            "xor",
            "aes",
            "rsa",
            "socket",
            "http",
            "fetch",
            "send",
            "recv",
        ];

        let keywords = vec![
            "let", "const", "define", "function", "if", "else", "for", "in", "end", "return",
            "match", "case", "try", "catch", "async", "await", "struct", "import", "include",
            "parallel",
        ];

        let commands = vec![
            "help",
            "examples",
            "templates",
            "cheatsheet",
            "history",
            "clear",
            "exit",
            "quit",
            "quickstart",
            "load",
            ":debug",
            ":time",
            ":profile",
            ":inspect",
            ":history",
            ":save",
        ];

        let word_start = line[..pos]
            .rfind(|c: char| c.is_whitespace() || c == '(')
            .map(|i| i + 1)
            .unwrap_or(0);
        let prefix = &line[word_start..pos];

        for func in functions {
            if func.starts_with(prefix) {
                candidates.push(Pair {
                    display: func.to_string(),
                    replacement: func.to_string(),
                });
            }
        }

        for keyword in keywords {
            if keyword.starts_with(prefix) {
                candidates.push(Pair {
                    display: keyword.to_string(),
                    replacement: keyword.to_string(),
                });
            }
        }

        for cmd in commands {
            if cmd.starts_with(prefix) {
                candidates.push(Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                });
            }
        }

        Ok((word_start, candidates))
    }
}

impl Hinter for TalonCompleter {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for TalonCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        false
    }
}

impl Validator for TalonCompleter {
    fn validate(&self, _ctx: &mut ValidationContext) -> Result<ValidationResult, ReadlineError> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Helper for TalonCompleter {}

pub struct REPL {
    history: Vec<String>,
    variables: HashMap<String, String>,
    multiline_buffer: String,
    in_block: bool,
    debug_mode: bool,
}

impl REPL {
    pub fn new() -> Self {
        let mut repl = REPL {
            history: Vec::new(),
            variables: HashMap::new(),
            multiline_buffer: String::new(),
            in_block: false,
            debug_mode: false,
        };
        repl.load_history();
        repl
    }

    fn get_history_file() -> Option<std::path::PathBuf> {
        use directories::BaseDirs;
        if let Some(base_dirs) = BaseDirs::new() {
            Some(base_dirs.home_dir().join(".talon_history"))
        } else {
            None
        }
    }

    fn load_history(&mut self) {
        if let Some(history_file) = Self::get_history_file() {
            if let Ok(content) = std::fs::read_to_string(&history_file) {
                self.history = content.lines().map(|s| s.to_string()).collect();
                if !self.history.is_empty() {
                    println!("Loaded {} command(s) from history", self.history.len());
                }
            }
        }
    }

    fn save_history(&self) {
        if let Some(history_file) = Self::get_history_file() {
            let content = self.history.join("\n");
            if let Err(e) = std::fs::write(&history_file, content) {
                eprintln!("Failed to save history: {}", e);
            }
        }
    }

    fn auto_save_history(&self) {
        if self.history.len() % 10 == 0 && !self.history.is_empty() {
            self.save_history();
        }
    }

    pub fn run(&mut self) {
        println!("\n{}", "═".repeat(60));
        println!("TALON REPL - Interactive Exploit Development Shell");
        println!("{}", "═".repeat(60));
        println!("[COMMANDS]:");
        println!("  help          - Show help");
        println!("  examples      - Show code examples");
        println!("  templates     - List exploit templates");
        println!("  cheatsheet    - Show syntax cheatsheet");
        println!("  history       - Show command history");
        println!("  clear         - Clear screen");
        println!("  exit/quit     - Exit REPL");
        println!("TIP: Press TAB for autocomplete");
        println!("{}\n", "═".repeat(60));

        let helper = TalonCompleter {
            hinter: HistoryHinter {},
        };
        let mut rl = Editor::new().unwrap();
        rl.set_helper(Some(helper));

        if let Some(history_file) = Self::get_history_file() {
            let _ = rl.load_history(&history_file);
        }

        loop {
            let prompt = self.get_prompt();
            let readline = rl.readline(&prompt);

            match readline {
                Ok(input) => {
                    let input = input.trim();

                    if input.is_empty() {
                        continue;
                    }

                    let _ = rl.add_history_entry(input);

                    if self.handle_repl_command(input) {
                        continue;
                    }

                    self.check_multiline(input);

                    if self.in_block {
                        self.multiline_buffer.push_str(input);
                        self.multiline_buffer.push('\n');
                        continue;
                    }

                    let code = if !self.multiline_buffer.is_empty() {
                        let complete = format!("{}{}", self.multiline_buffer, input);
                        self.multiline_buffer.clear();
                        complete
                    } else {
                        input.to_string()
                    };

                    self.history.push(code.clone());
                    self.auto_save_history();
                    self.execute(&code);
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C (Use 'exit' to quit)");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }

        if let Some(history_file) = Self::get_history_file() {
            let _ = rl.save_history(&history_file);
        }
        self.save_history();
        println!("History saved. Goodbye!");
    }

    fn get_prompt(&self) -> String {
        if self.in_block {
            "... ".to_string()
        } else {
            "talon> ".to_string()
        }
    }

    fn check_multiline(&mut self, input: &str) {
        let block_starters = vec!["define", "if", "for", "match", "try", "parallel", "struct"];

        for starter in block_starters {
            if input.starts_with(starter) && !input.contains("end") {
                self.in_block = true;
                return;
            }
        }

        if input == "end" {
            self.in_block = false;
        }
    }

    fn handle_repl_command(&mut self, input: &str) -> bool {
        match input {
            "help" | "?" => {
                self.show_help();
                true
            }
            "examples" => {
                self.show_examples();
                true
            }
            "templates" => {
                self.show_templates();
                true
            }
            "cheatsheet" | "cs" => {
                println!("{}", DocGenerator::generate_cheatsheet());
                true
            }
            "history" | "hist" => {
                self.show_history();
                true
            }
            "clear" | "cls" => {
                print!("\x1B[2J\x1B[1;1H");
                true
            }
            "exit" | "quit" | "q" => {
                self.save_history();
                println!("History saved to ~/.talon_history");
                println!("Goodbye!");
                std::process::exit(0);
            }
            cmd if cmd.starts_with(":debug") => {
                self.handle_debug_command(cmd);
                true
            }
            cmd if cmd.starts_with(":time") => {
                self.handle_time_command(cmd);
                true
            }
            cmd if cmd.starts_with(":profile") => {
                self.handle_profile_command(cmd);
                true
            }
            cmd if cmd.starts_with(":inspect") => {
                self.handle_inspect_command(cmd);
                true
            }
            cmd if cmd.starts_with(":history") => {
                self.handle_history_command(cmd);
                true
            }
            cmd if cmd.starts_with(":save") => {
                self.handle_save_command(cmd);
                true
            }
            cmd if cmd.starts_with("load ") => {
                let template = cmd.strip_prefix("load ").unwrap();
                self.load_template(template);
                true
            }
            cmd if cmd.starts_with("quickstart ") => {
                let exploit_type = cmd.strip_prefix("quickstart ").unwrap();
                println!("{}", ScriptHelper::generate_quick_start(exploit_type));
                true
            }
            _ => false,
        }
    }

    fn show_help(&self) {
        println!("\nTALON REPL Help");
        println!("{}", "─".repeat(60));
        println!("TIP: Interactive Commands:");
        println!("  help           - Show this help");
        println!("  examples       - Show code examples");
        println!("  templates      - List available templates");
        println!("  cheatsheet     - Show syntax reference");
        println!("  quickstart <type> - Generate quickstart (pwn/web3/fuzzing/recon)");
        println!("  load <name>    - Load exploit template");
        println!("  history        - Show command history");
        println!("  clear          - Clear screen");
        println!("  exit           - Exit REPL\n");

        println!("TIP: Advanced REPL Commands:");
        println!("  :debug on/off     - Toggle debug output");
        println!("  :time <cmd>       - Measure execution time");
        println!("  :profile <cmd>    - Performance profiling");
        println!("  :inspect <var>    - Inspect variable details");
        println!("  :history [N]      - Show last N commands");
        println!("  :save <file>      - Save session to file\n");

        println!("TIP: Quick Examples:");
        println!("  let x = 42");
        println!("  connect to \"192.168.1.1\" on port 22");
        println!("  analyze pe file \"malware.exe\"");
        println!("  fuzz file \"input.dat\"\n");

        println!("TIP: Common Mistakes:");
        for (mistake, tip) in ErrorHelper::common_mistakes() {
            println!("  [ERROR] {} → {}", mistake, tip);
        }
        println!();
    }

    fn show_examples(&self) {
        println!("\n[CODE EXAMPLES]");
        println!("{}", "─".repeat(60));

        println!("\n- Basic Example:");
        println!("{}", DocGenerator::generate_example("basic"));

        println!("\n- Exploit Example:");
        println!("{}", DocGenerator::generate_example("exploit"));
    }

    fn show_templates(&self) {
        println!("\nAvailable Templates");
        println!("{}", "─".repeat(60));

        let exploits = ScriptHelper::common_exploits();
        println!("\nExploit Templates:");
        for (name, _) in exploits.iter() {
            println!("  • {} - Load with: load {}", name, name);
        }

        let tasks = ScriptHelper::common_tasks();
        println!("\nTask Templates:");
        for (name, _) in tasks.iter() {
            println!("  • {} - Load with: load {}", name, name);
        }
        println!();
    }

    fn load_template(&mut self, name: &str) {
        let exploits = ScriptHelper::common_exploits();
        let tasks = ScriptHelper::common_tasks();

        if let Some(code) = exploits.get(name) {
            println!("\n[OK] Loaded template: {}", name);
            println!("{}", code);
            self.execute(code);
        } else if let Some(code) = tasks.get(name) {
            println!("\n[OK] Loaded template: {}", name);
            println!("{}", code);
            self.execute(code);
        } else {
            println!(
                "[ERROR] Template '{}' not found. Type 'templates' to see available templates.",
                name
            );
        }
    }

    fn show_history(&self) {
        println!("\nCommand History");
        println!("{}", "─".repeat(60));

        if self.history.is_empty() {
            println!("  (empty)");
        } else {
            for (i, cmd) in self.history.iter().enumerate() {
                println!("  [{}] {}", i + 1, cmd);
            }
        }
        println!();
    }

    fn execute(&mut self, code: &str) {
        match parse_script(code) {
            Ok(commands) => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                match rt.block_on(interpret(&commands)) {
                    Ok(_) => {
                        println!("[OK] Success");
                    }
                    Err(e) => {
                        let enhanced_error = ErrorHelper::suggest_fix(&e);
                        println!("[ERROR] Execution Error:\n{}", enhanced_error);
                    }
                }
            },
            Err(e) => {
                let enhanced_error = ErrorHelper::suggest_fix(&e);
                println!("[ERROR] Parse Error:\n{}", enhanced_error);
            }
        }
    }

    fn handle_debug_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.len() < 2 {
            println!("Debug mode: {}", if self.debug_mode { "ON" } else { "OFF" });
            println!("Usage: :debug on | :debug off");
            return;
        }

        match parts[1] {
            "on" | "true" | "1" => {
                self.debug_mode = true;
                println!("Debug mode enabled - verbose output active");
            }
            "off" | "false" | "0" => {
                self.debug_mode = false;
                println!("Debug mode disabled");
            }
            _ => {
                println!("Invalid option. Use: :debug on | :debug off");
            }
        }
    }

    fn handle_time_command(&mut self, cmd: &str) {
        let code = cmd.strip_prefix(":time").unwrap().trim();
        if code.is_empty() {
            println!("Usage: :time <command>");
            return;
        }

        let start = std::time::Instant::now();
        self.execute(code);
        let duration = start.elapsed();

        println!("\nExecution time: {:?}", duration);
    }

    fn handle_profile_command(&mut self, cmd: &str) {
        let code = cmd.strip_prefix(":profile").unwrap().trim();
        if code.is_empty() {
            println!("Usage: :profile <command>");
            return;
        }

        println!("Profiling execution...");
        let start = std::time::Instant::now();

        self.execute(code);

        let duration = start.elapsed();
        println!("\n=== Performance Profile ===");
        println!("Total time:     {:?}", duration);
        println!("Microseconds:   {}", duration.as_micros());
        println!("Memory impact:  Low (interpreter mode)");
    }

    fn handle_inspect_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.len() < 2 {
            println!("Usage: :inspect <variable>");
            return;
        }

        let var_name = parts[1];
        if let Some(value) = self.variables.get(var_name) {
            println!("\n=== Variable Inspection ===");
            println!("Name:  {}", var_name);
            println!("Value: {}", value);
            println!("Type:  {}", Self::infer_type(value));
            println!("Size:  {} bytes", value.len());
        } else {
            println!("Variable '{}' not found", var_name);
        }
    }

    fn handle_history_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let count = if parts.len() >= 2 {
            parts[1].parse::<usize>().unwrap_or(10)
        } else {
            10
        };

        println!("\n=== Command History (last {}) ===", count);
        let start_idx = if self.history.len() > count {
            self.history.len() - count
        } else {
            0
        };

        for (i, cmd) in self.history.iter().enumerate().skip(start_idx) {
            println!("{:3}: {}", i + 1, cmd);
        }
    }

    fn handle_save_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.len() < 2 {
            println!("Usage: :save <filename.tal>");
            return;
        }

        let filename = parts[1];
        let content = self.history.join("\n");

        match std::fs::write(filename, content) {
            Ok(_) => println!("Session saved to {}", filename),
            Err(e) => println!("Error saving session: {}", e),
        }
    }

    fn infer_type(value: &str) -> &'static str {
        if value.parse::<i64>().is_ok() {
            "Number"
        } else if value.starts_with("0x") {
            "Hex/Address"
        } else if value.starts_with('[') {
            "List/Array"
        } else if value.starts_with('{') {
            "Map/Object"
        } else {
            "String"
        }
    }
}

pub fn run_repl() {
    let mut repl = REPL::new();
    repl.run();
}
