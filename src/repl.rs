use crate::helpers::{DocGenerator, ErrorHelper, ScriptHelper};
use crate::interpreter::{interpret, Value};
use crate::parser::parse_script;
use crate::registry::FunctionRegistry;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{hint::HistoryHinter, Context, Editor, Helper};
use std::borrow::Cow;
use std::collections::HashMap;
use colored::Colorize;

struct TalonCompleter {
    hinter: HistoryHinter,
    registry: FunctionRegistry,
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

        if prefix.contains('.') {
            let parts: Vec<&str> = prefix.split('.').collect();
            if parts.len() == 2 {
                let object_name = parts[0];
                let method_prefix = parts[1];
                
                for func in self.registry.all_functions() {
                    if func.name.starts_with(&format!("{}.", object_name)) 
                        && func.name[object_name.len() + 1..].starts_with(method_prefix) {
                        let method_name = &func.name[object_name.len() + 1..];
                        let display = format!("{}.{} - {}", object_name, method_name, func.signature);
                        candidates.push(Pair {
                            display,
                            replacement: func.name.clone(),
                        });
                    }
                }
            }
        } else {
            for func in self.registry.all_functions() {
                if func.name.starts_with(prefix) {
                    let display = format!("{} - {}", func.name, func.signature);
                    candidates.push(Pair {
                        display,
                        replacement: func.name.clone(),
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
        let keywords = vec![
            "let", "const", "define", "function", "if", "else", "for", "in", "end", "return",
            "match", "case", "try", "catch", "async", "await", "struct", "import", "include",
            "parallel",
        ];
        
        let mut result = String::new();
        let mut chars = line.chars().peekable();
        let mut current_word = String::new();
        let mut in_string = false;
        let mut string_char = ' ';
        
        while let Some(ch) = chars.next() {
            if in_string {
                current_word.push(ch);
                if ch == string_char && (current_word.len() == 1 || !current_word.ends_with("\\")) {
                    result.push_str(&current_word.green().to_string());
                    current_word.clear();
                    in_string = false;
                }
            } else if ch == '"' || ch == '\'' {
                if !current_word.is_empty() {
                    result.push_str(&Self::highlight_word(&current_word, &keywords, &self.registry));
                    current_word.clear();
                }
                in_string = true;
                string_char = ch;
                current_word.push(ch);
            } else if ch.is_whitespace() || ch == '(' || ch == ')' || ch == '{' || ch == '}' || ch == '[' || ch == ']' || ch == ',' || ch == ';' {
                if !current_word.is_empty() {
                    result.push_str(&Self::highlight_word(&current_word, &keywords, &self.registry));
                    current_word.clear();
                }
                result.push(ch);
            } else {
                current_word.push(ch);
            }
        }
        
        if in_string {
            result.push_str(&current_word.green().to_string());
        } else if !current_word.is_empty() {
            result.push_str(&Self::highlight_word(&current_word, &keywords, &self.registry));
        }
        
        Cow::Owned(result)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        true
    }
}

impl TalonCompleter {
    fn highlight_word(word: &str, keywords: &[&str], registry: &FunctionRegistry) -> String {
        if keywords.contains(&word) {
            word.blue().to_string()
        } else if word.parse::<i64>().is_ok() || word.starts_with("0x") {
            word.cyan().to_string()
        } else if registry.get(word).is_some() {
            word.yellow().to_string()
        } else {
            word.to_string()
        }
    }
}

fn pretty_print_bytes(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "b\"\"".to_string();
    }
    
    let mut output = String::new();
    output.push_str(&format!("Bytes ({} bytes):\n", bytes.len()));
    
    for (i, chunk) in bytes.chunks(16).enumerate() {
        output.push_str(&format!("{:08x}  ", i * 16));
        
        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 {
                output.push(' ');
            }
            output.push_str(&format!("{:02x} ", byte));
        }
        
        if chunk.len() < 16 {
            for j in chunk.len()..16 {
                if j == 8 {
                    output.push(' ');
                }
                output.push_str("   ");
            }
        }
        
        output.push_str(" |");
        for byte in chunk {
            let ch = if byte.is_ascii_graphic() || *byte == b' ' {
                *byte as char
            } else {
                '.'
            };
            output.push(ch);
        }
        output.push_str("|\n");
    }
    
    output
}

fn pretty_print_map(map: &HashMap<String, Value>, indent: usize) -> String {
    if map.is_empty() {
        return "{}".to_string();
    }
    
    let mut output = String::new();
    output.push_str("{\n");
    
    let indent_str = "  ".repeat(indent + 1);
    let close_indent = "  ".repeat(indent);
    
    let mut items: Vec<_> = map.iter().collect();
    items.sort_by_key(|(k, _)| k.as_str());
    
    for (i, (key, value)) in items.iter().enumerate() {
        output.push_str(&indent_str);
        output.push_str(&format!("\"{}\": ", key));
        
        match value {
            Value::Map(inner_map) => {
                output.push_str(&pretty_print_map(inner_map, indent + 1));
            }
            Value::List(list) => {
                output.push('[');
                for (j, item) in list.iter().enumerate() {
                    if j > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&format_value(item));
                }
                output.push(']');
            }
            Value::String(s) => {
                output.push_str(&format!("\"{}\"", s));
            }
            Value::Bytes(b) => {
                output.push_str(&format!("b\"{}\"", hex::encode(b)));
            }
            other => {
                output.push_str(&format_value(other));
            }
        }
        
        if i < items.len() - 1 {
            output.push(',');
        }
        output.push('\n');
    }
    
    output.push_str(&close_indent);
    output.push('}');
    
    output
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", s),
        Value::Bytes(b) => format!("0x{}", hex::encode(b)),
        Value::Null => "null".to_string(),
        Value::Map(m) => pretty_print_map(m, 0),
        Value::List(l) => {
            let items: Vec<_> = l.iter().map(format_value).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Set(s) => {
            let items: Vec<_> = s.iter().map(|x| x.as_str()).collect();
            format!("#{{{}}}", items.join(", "))
        }
        Value::SshConnection(id) => format!("SSH({})", id),
    }
}

fn pretty_print_value(value: &Value) {
    match value {
        Value::Bytes(bytes) => {
            println!("{}", pretty_print_bytes(bytes));
        }
        Value::Map(map) => {
            println!("{}", pretty_print_map(map, 0));
        }
        other => {
            println!("{}", format_value(other));
        }
    }
}

impl Validator for TalonCompleter {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult, ReadlineError> {
        let input = ctx.input();
        
        if Self::is_incomplete(input) {
            Ok(ValidationResult::Incomplete)
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}

impl TalonCompleter {
    fn is_incomplete(line: &str) -> bool {
        let mut paren_count = 0;
        let mut bracket_count = 0;
        let mut brace_count = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        let mut escape = false;
        
        for ch in line.chars() {
            if escape {
                escape = false;
                continue;
            }
            
            if ch == '\\' {
                escape = true;
                continue;
            }
            
            if in_string {
                if ch == string_char {
                    in_string = false;
                }
                continue;
            }
            
            match ch {
                '"' | '\'' => {
                    in_string = true;
                    string_char = ch;
                }
                '(' => paren_count += 1,
                ')' => paren_count -= 1,
                '[' => bracket_count += 1,
                ']' => bracket_count -= 1,
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                _ => {}
            }
        }
        
        in_string || paren_count > 0 || bracket_count > 0 || brace_count > 0
    }
    
    fn calculate_indent(lines: &[&str]) -> usize {
        let mut indent = 0;
        let indent_keywords = vec!["define", "if", "for", "match", "try", "parallel", "struct", "case"];
        let dedent_keywords = vec!["end", "else"];
        
        for line in lines {
            let trimmed = line.trim();
            
            for keyword in &indent_keywords {
                if trimmed.starts_with(keyword) {
                    indent += 1;
                    break;
                }
            }
            
            for keyword in &dedent_keywords {
                if trimmed.starts_with(keyword) && indent > 0 {
                    indent -= 1;
                    break;
                }
            }
        }
        
        indent * 2
    }
}

impl Helper for TalonCompleter {}

pub struct REPL {
    history: Vec<String>,
    variables: HashMap<String, String>,
    multiline_buffer: String,
    multiline_lines: Vec<String>,
    in_block: bool,
    debug_mode: bool,
    registry: FunctionRegistry,
}

impl REPL {
    pub fn new() -> Self {
        let mut repl = REPL {
            history: Vec::new(),
            variables: HashMap::new(),
            multiline_buffer: String::new(),
            multiline_lines: Vec::new(),
            in_block: false,
            debug_mode: false,
            registry: FunctionRegistry::new(),
        };
        repl.load_history();
        repl
    }

    fn get_history_file() -> Option<std::path::PathBuf> {
        use directories::BaseDirs;
        if let Some(base_dirs) = BaseDirs::new() {
            let talon_dir = base_dirs.home_dir().join(".talon");
            if !talon_dir.exists() {
                let _ = std::fs::create_dir_all(&talon_dir);
            }
            Some(talon_dir.join("history"))
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
            registry: FunctionRegistry::new(),
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

                    if self.in_block || TalonCompleter::is_incomplete(input) {
                        self.multiline_buffer.push_str(input);
                        self.multiline_buffer.push('\n');
                        self.multiline_lines.push(input.to_string());
                        continue;
                    }

                    let code = if !self.multiline_buffer.is_empty() {
                        let complete = format!("{}{}", self.multiline_buffer, input);
                        self.multiline_buffer.clear();
                        self.multiline_lines.clear();
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
        if self.in_block || !self.multiline_buffer.is_empty() {
            let lines_ref: Vec<&str> = self.multiline_lines.iter().map(|s| s.as_str()).collect();
            let indent = TalonCompleter::calculate_indent(&lines_ref);
            format!("{}... ", " ".repeat(indent))
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
        if input.starts_with("help(") && input.ends_with(')') {
            let func_name = input.trim_start_matches("help(").trim_end_matches(')').trim_matches('"').trim_matches('\'');
            self.show_function_help(func_name);
            return true;
        }
        
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
                println!("History saved to ~/.talon/history");
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

    fn show_function_help(&self, func_name: &str) {
        if let Some(func) = self.registry.get(func_name) {
            println!("\n{}", "═".repeat(60));
            println!("Function: {}", func.name);
            println!("{}", "─".repeat(60));
            println!("Signature: {}", func.signature);
            println!("Description: {}", func.description);
            if !func.examples.is_empty() {
                println!("\nExamples:");
                for example in &func.examples {
                    println!("{}", example);
                }
            }
            if !func.related.is_empty() {
                println!("\nRelated functions:");
                for related in &func.related {
                    println!("  - {}", related);
                }
            }
            println!("{}", "═".repeat(60));
        } else {
            println!("Function '{}' not found. Type 'help' for general help.", func_name);
            
            let similar = self.registry.search(func_name);
            if !similar.is_empty() {
                println!("\nDid you mean one of these?");
                for func in similar.iter().take(5) {
                    println!("  - {}", func.name);
                }
            }
        }
    }
    
    fn show_help(&self) {
        println!("\nTALON REPL Help");
        println!("{}", "─".repeat(60));
        println!("TIP: Interactive Commands:");
        println!("  help              - Show this help");
        println!("  help(\"function\")  - Show detailed help for a function");
        println!("  examples          - Show code examples");
        println!("  templates         - List available templates");
        println!("  cheatsheet        - Show syntax reference");
        println!("  quickstart <type> - Generate quickstart (pwn/web3/fuzzing/recon)");
        println!("  load <name>       - Load exploit template");
        println!("  history           - Show command history");
        println!("  clear             - Clear screen");
        println!("  exit              - Exit REPL\n");

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
        println!("  fuzz file \"input.dat\"");
        println!("  help(\"connect\")  - Get help on connect function\n");

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_incomplete_balanced() {
        assert!(!TalonCompleter::is_incomplete("let x = 42"));
        assert!(!TalonCompleter::is_incomplete("connect(\"host\", 22)"));
        assert!(!TalonCompleter::is_incomplete("[1, 2, 3]"));
        assert!(!TalonCompleter::is_incomplete("{\"key\": \"value\"}"));
    }

    #[test]
    fn test_is_incomplete_unbalanced_parens() {
        assert!(TalonCompleter::is_incomplete("connect(\"host\""));
        assert!(TalonCompleter::is_incomplete("func(arg1, arg2"));
        assert!(TalonCompleter::is_incomplete("((nested)"));
    }

    #[test]
    fn test_is_incomplete_unbalanced_brackets() {
        assert!(TalonCompleter::is_incomplete("[1, 2, 3"));
        assert!(TalonCompleter::is_incomplete("let arr = ["));
        assert!(TalonCompleter::is_incomplete("[[nested]"));
    }

    #[test]
    fn test_is_incomplete_unbalanced_braces() {
        assert!(TalonCompleter::is_incomplete("{\"key\": \"value\""));
        assert!(TalonCompleter::is_incomplete("let obj = {"));
        assert!(TalonCompleter::is_incomplete("{{nested}"));
    }

    #[test]
    fn test_is_incomplete_unclosed_string() {
        assert!(TalonCompleter::is_incomplete("let s = \"hello"));
        assert!(TalonCompleter::is_incomplete("let s = 'world"));
        assert!(!TalonCompleter::is_incomplete("let s = \"hello\""));
        assert!(!TalonCompleter::is_incomplete("let s = 'world'"));
    }

    #[test]
    fn test_is_incomplete_escaped_quotes() {
        assert!(!TalonCompleter::is_incomplete("let s = \"hello\\\"world\""));
        assert!(!TalonCompleter::is_incomplete("let s = 'it\\'s'"));
    }

    #[test]
    fn test_calculate_indent_basic() {
        let lines = vec!["define function test()"];
        let indent = TalonCompleter::calculate_indent(&lines);
        assert_eq!(indent, 2);
    }

    #[test]
    fn test_calculate_indent_nested() {
        let lines = vec![
            "define function test()",
            "  if x > 0",
        ];
        let indent = TalonCompleter::calculate_indent(&lines);
        assert_eq!(indent, 4);
    }

    #[test]
    fn test_calculate_indent_with_end() {
        let lines = vec![
            "define function test()",
            "  let x = 1",
            "end",
        ];
        let indent = TalonCompleter::calculate_indent(&lines);
        assert_eq!(indent, 0);
    }

    #[test]
    fn test_calculate_indent_multiple_blocks() {
        let lines = vec![
            "define function test()",
            "  for i in 0..10",
            "    if i > 5",
        ];
        let indent = TalonCompleter::calculate_indent(&lines);
        assert_eq!(indent, 6);
    }

    #[test]
    fn test_repl_creation() {
        let repl = REPL::new();
        assert_eq!(repl.history.len(), 0);
        assert!(!repl.in_block);
        assert!(!repl.debug_mode);
        assert_eq!(repl.multiline_buffer, "");
        assert_eq!(repl.multiline_lines.len(), 0);
    }

    #[test]
    fn test_repl_prompt_normal() {
        let repl = REPL::new();
        assert_eq!(repl.get_prompt(), "talon> ");
    }

    #[test]
    fn test_repl_prompt_multiline() {
        let mut repl = REPL::new();
        repl.in_block = true;
        repl.multiline_lines.push("define function test()".to_string());
        let prompt = repl.get_prompt();
        assert!(prompt.starts_with("  "));
        assert!(prompt.ends_with("... "));
    }

    #[test]
    fn test_format_value_number() {
        let value = Value::Number(42);
        assert_eq!(format_value(&value), "42");
    }

    #[test]
    fn test_format_value_string() {
        let value = Value::String("hello".to_string());
        assert_eq!(format_value(&value), "\"hello\"");
    }

    #[test]
    fn test_format_value_bytes() {
        let value = Value::Bytes(vec![0x41, 0x42, 0x43]);
        assert_eq!(format_value(&value), "0x414243");
    }

    #[test]
    fn test_format_value_null() {
        let value = Value::Null;
        assert_eq!(format_value(&value), "null");
    }

    #[test]
    fn test_format_value_list() {
        let value = Value::List(vec![
            Value::Number(1),
            Value::Number(2),
            Value::Number(3),
        ]);
        assert_eq!(format_value(&value), "[1, 2, 3]");
    }

    #[test]
    fn test_format_value_set() {
        let mut set = std::collections::HashSet::new();
        set.insert("a".to_string());
        set.insert("b".to_string());
        let value = Value::Set(set);
        let result = format_value(&value);
        assert!(result.starts_with("#{"));
        assert!(result.ends_with('}'));
        assert!(result.contains('a') && result.contains('b'));
    }

    #[test]
    fn test_pretty_print_bytes_empty() {
        let bytes = vec![];
        let result = pretty_print_bytes(&bytes);
        assert_eq!(result, "b\"\"");
    }

    #[test]
    fn test_pretty_print_bytes_basic() {
        let bytes = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f];
        let result = pretty_print_bytes(&bytes);
        assert!(result.contains("Bytes (5 bytes)"));
        assert!(result.contains("48 65 6c 6c 6f"));
        assert!(result.contains("Hello"));
    }

    #[test]
    fn test_pretty_print_bytes_non_printable() {
        let bytes = vec![0x00, 0x01, 0x02, 0x03];
        let result = pretty_print_bytes(&bytes);
        assert!(result.contains("00 01 02 03"));
        assert!(result.contains("...."));
    }

    #[test]
    fn test_pretty_print_map_empty() {
        let map = HashMap::new();
        let result = pretty_print_map(&map, 0);
        assert_eq!(result, "{}");
    }

    #[test]
    fn test_pretty_print_map_simple() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::String("value".to_string()));
        let result = pretty_print_map(&map, 0);
        assert!(result.contains("\"key\""));
        assert!(result.contains("\"value\""));
    }

    #[test]
    fn test_pretty_print_map_nested() {
        let mut inner_map = HashMap::new();
        inner_map.insert("inner".to_string(), Value::Number(42));
        
        let mut map = HashMap::new();
        map.insert("outer".to_string(), Value::Map(inner_map));
        
        let result = pretty_print_map(&map, 0);
        assert!(result.contains("\"outer\""));
        assert!(result.contains("\"inner\""));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_infer_type_number() {
        assert_eq!(REPL::infer_type("42"), "Number");
        assert_eq!(REPL::infer_type("-123"), "Number");
    }

    #[test]
    fn test_infer_type_hex() {
        assert_eq!(REPL::infer_type("0xdeadbeef"), "Hex/Address");
        assert_eq!(REPL::infer_type("0x1234"), "Hex/Address");
    }

    #[test]
    fn test_infer_type_list() {
        assert_eq!(REPL::infer_type("[1, 2, 3]"), "List/Array");
    }

    #[test]
    fn test_infer_type_map() {
        assert_eq!(REPL::infer_type("{key: value}"), "Map/Object");
    }

    #[test]
    fn test_infer_type_string() {
        assert_eq!(REPL::infer_type("hello"), "String");
        assert_eq!(REPL::infer_type("some text"), "String");
    }
}
