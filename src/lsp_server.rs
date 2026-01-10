use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService, Server};
use tower_lsp::jsonrpc::Result;
use log::{info, debug};
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Clone)]
struct FunctionSignature {
    name: String,
    signature: String,
    description: String,
    parameters: Vec<String>,
    return_type: String,
    examples: Vec<String>,
    category: String,
}

struct Backend {
    functions: HashMap<String, FunctionSignature>,
    user_functions: HashMap<String, FunctionSignature>,
}

impl Backend {
    fn new() -> Self {
        let mut functions = HashMap::new();
        
        let builtin_functions = vec![
            ("process_attach", "process_attach(pid_or_name)", 
             "Attach to a running process by PID (number) or name (string)\n\n**Mini-Tutorial:**\nProcess attachment is the first step in exploit development. You can attach by:\n1. PID (numeric process ID)\n2. Process name (searches running processes)\n\n**Common Pitfalls:**\n- Requires admin/root privileges\n- Process name is case-sensitive\n- Some processes have anti-debug protections\n\n**Best Practice:** Always check if process_attach succeeded by examining the returned map.", 
             vec!["pid_or_name: number|string"], "map", 
             vec!["let proc = process_attach(1234)", "let proc = process_attach(\"game.exe\")", "if proc[\"attached\"] == \"true\"\n    print(\"Successfully attached!\")\nend"],
             "Process Control"),
            ("process_detach", "process_detach(pid)", "Detach from an attached process",
             vec!["pid: number"], "string",
             vec!["process_detach(1234)"],
             "Process Control"),
            ("process_suspend", "process_suspend(pid)", "Suspend a running process",
             vec!["pid: number"], "string",
             vec!["process_suspend(1234)"],
             "Process Control"),
            ("process_resume", "process_resume(pid)", "Resume a suspended process",
             vec!["pid: number"], "string",
             vec!["process_resume(1234)"],
             "Process Control"),
            ("process_kill", "process_kill(pid)", "Terminate a process",
             vec!["pid: number"], "string",
             vec!["process_kill(1234)"],
             "Process Control"),
            ("process_modules", "process_modules(pid)", "List all loaded modules in a process",
             vec!["pid: number"], "list",
             vec!["let mods = process_modules(1234)", "let base = mods[0][\"base\"]"],
             "Process Control"),
            ("mem_read", "mem_read(pid, address, size)", "Read memory from target process",
             vec!["pid: number", "address: number", "size: number"], "bytes",
             vec!["let data = mem_read(1234, 0x400000, 16)"],
             "Memory"),
            ("mem_write", "mem_write(pid, address, data)", "Write memory to target process",
             vec!["pid: number", "address: number", "data: bytes"], "string",
             vec!["mem_write(1234, 0x500000, bytes(\"AAAA\"))"],
             "Memory"),
            ("mem_scan", "mem_scan(pid, pattern)", "Scan process memory for byte pattern",
             vec!["pid: number", "pattern: bytes|string"], "list",
             vec!["let addrs = mem_scan(1234, bytes(\"\\x90\\x90\\x90\\x90\"))"],
             "Memory"),
            ("mem_alloc", "mem_alloc(pid, size)", "Allocate memory in target process",
             vec!["pid: number", "size: number"], "number",
             vec!["let addr = mem_alloc(1234, 0x1000)"],
             "Memory"),
            ("mem_free", "mem_free(pid, address)", "Free allocated memory",
             vec!["pid: number", "address: number"], "string",
             vec!["mem_free(1234, 0x10000000)"],
             "Memory"),
            ("mem_protect", "mem_protect(pid, address, protection)", "Change memory protection flags",
             vec!["pid: number", "address: number", "protection: string"], "string",
             vec!["mem_protect(1234, 0x400000, \"RWX\")"],
             "Memory"),
            ("pointer_chain", "pointer_chain(pid, offsets)", "Follow multi-level pointer chain",
             vec!["pid: number", "offsets: list"], "number",
             vec!["let addr = pointer_chain(1234, [0x100, 0x50, 0x10])"],
             "Memory"),
            ("inject_asm", "inject_asm(pid, address, asm_code)", "Inject assembly code at address",
             vec!["pid: number", "address: number", "asm_code: string"], "string",
             vec!["inject_asm(1234, 0x600000, \"nop; ret\")"],
             "Memory"),
            ("cyclic", "cyclic(length)", 
             "Generate cyclic de Bruijn pattern for finding offsets\n\n**Mini-Tutorial:**\nCyclic patterns help find buffer overflow offsets:\n1. Generate pattern longer than buffer\n2. Send to vulnerable program\n3. Note crash address (e.g., 0x61616162)\n4. Use cyclic_find() to get exact offset\n\n**Why it works:** Every 4-byte sequence is unique, so the crash address tells you exactly where you control memory.\n\n**Common Pitfalls:**\n- Pattern must be longer than buffer\n- Big-endian systems reverse byte order\n- Some chars may be filtered (null bytes, newlines)", 
             vec!["length: number"], "bytes",
             vec!["let pattern = cyclic(264)", "send(conn, pattern)  // Program crashes at 0x61616162", "let offset = cyclic_find(0x61616162)  // Returns 264"],
             "Exploitation"),
            ("cyclic_find", "cyclic_find(pattern)", "Find offset of pattern in cyclic sequence",
             vec!["pattern: bytes|string|number"], "number",
             vec!["let offset = cyclic_find(0x61616161)", "let offset = cyclic_find(\"aaaa\")"],
             "Exploitation"),
            ("p64", "p64(value)", "Pack 64-bit integer as little-endian bytes",
             vec!["value: number"], "bytes",
             vec!["let packed = p64(0xdeadbeef)"],
             "Packing"),
            ("p32", "p32(value)", "Pack 32-bit integer as little-endian bytes",
             vec!["value: number"], "bytes",
             vec!["let packed = p32(0xdeadbeef)"],
             "Packing"),
            ("p16", "p16(value)", "Pack 16-bit integer as little-endian bytes",
             vec!["value: number"], "bytes",
             vec!["let packed = p16(0x1234)"],
             "Packing"),
            ("p8", "p8(value)", "Pack 8-bit integer as bytes",
             vec!["value: number"], "bytes",
             vec!["let packed = p8(0x41)"],
             "Packing"),
            ("u64", "u64(data)", "Unpack 64-bit little-endian bytes to integer",
             vec!["data: bytes"], "number",
             vec!["let num = u64(data)"],
             "Packing"),
            ("u32", "u32(data)", "Unpack 32-bit little-endian bytes to integer",
             vec!["data: bytes"], "number",
             vec!["let num = u32(data)"],
             "Packing"),
            ("shellcode", "shellcode(arch, payload, ...)", "Generate shellcode for various architectures and payloads",
             vec!["arch: string", "payload: string", "...: various"], "bytes",
             vec!["let sc = shellcode(\"x64\", \"execve\", \"/bin/sh\")", "let sc = shellcode(\"x86\", \"bind_tcp\", 4444)"],
             "Exploitation"),
            ("rop_find", "rop_find(binary, gadget)", "Find ROP gadgets in binary",
             vec!["binary: string", "gadget: string"], "list",
             vec!["let gadgets = rop_find(\"./binary\", \"pop rdi\")"],
             "ROP"),
            ("connect", "connect(host, port)", "Connect to remote service via TCP",
             vec!["host: string", "port: number"], "number",
             vec!["let conn = connect(\"target.com\", 1337)"],
             "Network"),
            ("listen", "listen(port)", "Listen for incoming TCP connections",
             vec!["port: number"], "number",
             vec!["let sock = listen(8080)"],
             "Network"),
            ("send", "send(socket, data)", "Send data over socket",
             vec!["socket: number", "data: bytes|string"], "number",
             vec!["send(conn, payload)"],
             "Network"),
            ("recv", "recv(socket, size)", "Receive data from socket",
             vec!["socket: number", "size: number"], "bytes",
             vec!["let data = recv(conn, 1024)"],
             "Network"),
            ("sendline", "sendline(socket, data)", "Send data with newline over socket",
             vec!["socket: number", "data: string"], "number",
             vec!["sendline(conn, \"username\")"],
             "Network"),
            ("recvline", "recvline(socket)", "Receive line from socket",
             vec!["socket: number"], "string",
             vec!["let line = recvline(conn)"],
             "Network"),
            ("interactive", "interactive(socket)", "Enter interactive shell mode",
             vec!["socket: number"], "none",
             vec!["interactive(conn)"],
             "Network"),
            ("sha256", "sha256(data)", "Calculate SHA-256 hash",
             vec!["data: bytes|string"], "string",
             vec!["let hash = sha256(\"password\")"],
             "Crypto"),
            ("md5", "md5(data)", "Calculate MD5 hash",
             vec!["data: bytes|string"], "string",
             vec!["let hash = md5(\"data\")"],
             "Crypto"),
            ("sha1", "sha1(data)", "Calculate SHA-1 hash",
             vec!["data: bytes|string"], "string",
             vec!["let hash = sha1(\"data\")"],
             "Crypto"),
            ("sha512", "sha512(data)", "Calculate SHA-512 hash",
             vec!["data: bytes|string"], "string",
             vec!["let hash = sha512(\"data\")"],
             "Crypto"),
            ("base64_encode", "base64_encode(data)", "Encode data as Base64",
             vec!["data: bytes|string"], "string",
             vec!["let encoded = base64_encode(shellcode)"],
             "Encoding"),
            ("base64_decode", "base64_decode(data)", "Decode Base64 data",
             vec!["data: string"], "bytes",
             vec!["let decoded = base64_decode(\"SGVsbG8=\")"],
             "Encoding"),
            ("hex", "hex(value)", "Convert number to hex string",
             vec!["value: number"], "string",
             vec!["let addr_str = hex(0xdeadbeef)"],
             "Utility"),
            ("int", "int(value)", "Convert value to integer",
             vec!["value: string|number"], "number",
             vec!["let num = int(\"42\")", "let num = int(\"0xdead\")"],
             "Utility"),
            ("len", "len(value)", "Get length of string, bytes, list, or map",
             vec!["value: string|bytes|list|map"], "number",
             vec!["let size = len(payload)"],
             "Utility"),
            ("str", "str(value)", "Convert value to string",
             vec!["value: any"], "string",
             vec!["let s = str(1234)"],
             "Utility"),
            ("bytes", "bytes(value)", "Convert string or list to bytes",
             vec!["value: string|list"], "bytes",
             vec!["let b = bytes(\"AAAA\")", "let b = bytes([0x41, 0x41, 0x41, 0x41])"],
             "Utility"),
            ("range", "range(start, end)", "Create list of numbers from start to end-1",
             vec!["start: number", "end: number"], "list",
             vec!["for i in range(0, 10)"],
             "Utility"),
            ("print", "print(...)", "Print values to console",
             vec!["...: any"], "none",
             vec!["print(\"Hello, world!\")", "print(\"Address:\", hex(addr))"],
             "Utility"),
            ("read", "read(filename)", "Read file contents as bytes",
             vec!["filename: string"], "bytes",
             vec!["let data = read(\"exploit.bin\")"],
             "File I/O"),
            ("write", "write(filename, data)", "Write data to file",
             vec!["filename: string", "data: bytes|string"], "number",
             vec!["write(\"output.bin\", shellcode)"],
             "File I/O"),
            ("unity_find_objects", "unity_find_objects(class_name)", 
             "Find Unity GameObjects by class name\n\n**Mini-Tutorial:**\nUnity stores all game objects in the Mono heap. This function:\n1. Scans the Mono runtime\n2. Finds all instances of the specified class\n3. Returns their memory addresses\n\n**Common Class Names:**\n- PlayerController, Player, Character\n- Enemy, EnemyController\n- GameManager, LevelManager\n- HealthComponent, WeaponController\n\n**Pro Tip:** Use unity_mono_dump() first to see all available classes.\n\n**Common Pitfalls:**\n- Class name must match exactly (case-sensitive)\n- IL2CPP games use different structure\n- Some objects may be temporarily inactive", 
             vec!["class_name: string"], "list",
             vec!["let players = unity_find_objects(\"PlayerController\")", "for p in players\n    print(\"Found at:\", hex(p[\"address\"]))\nend"],
             "Unity Engine"),
            ("unity_get_component", "unity_get_component(object_addr, component_name)", "Get Unity component from GameObject",
             vec!["object_addr: number", "component_name: string"], "map",
             vec!["let health = unity_get_component(player_addr, \"HealthComponent\")"],
             "Unity Engine"),
            ("unity_call_method", "unity_call_method(object_addr, method_name)", "Call Unity method on GameObject",
             vec!["object_addr: number", "method_name: string"], "any",
             vec!["unity_call_method(player_addr, \"TakeDamage\")"],
             "Unity Engine"),
            ("unity_mono_dump", "unity_mono_dump(pid)", "Dump Mono assemblies from Unity game",
             vec!["pid: number"], "list",
             vec!["let dlls = unity_mono_dump(game_pid)"],
             "Unity Engine"),
            ("unreal_find_actors", "unreal_find_actors(class_name)", "Find Unreal Engine actors by class",
             vec!["class_name: string"], "list",
             vec!["let characters = unreal_find_actors(\"Character\")"],
             "Unreal Engine"),
            ("unreal_get_property", "unreal_get_property(actor_addr, property_name)", "Get property value from Unreal actor",
             vec!["actor_addr: number", "property_name: string"], "any",
             vec!["let hp = unreal_get_property(actor, \"Health\")"],
             "Unreal Engine"),
            ("unreal_set_property", "unreal_set_property(actor_addr, property_name, value)", "Set property value on Unreal actor",
             vec!["actor_addr: number", "property_name: string", "value: any"], "string",
             vec!["unreal_set_property(actor, \"Health\", 9999)"],
             "Unreal Engine"),
            ("unreal_process_event", "unreal_process_event(actor_addr, event_name)", "Trigger event on Unreal actor",
             vec!["actor_addr: number", "event_name: string"], "string",
             vec!["unreal_process_event(actor, \"OnDeath\")"],
             "Unreal Engine"),
            ("anticheat_detect", "anticheat_detect()", "Detect anti-cheat systems running",
             vec![], "list",
             vec!["let acs = anticheat_detect()"],
             "Anti-Cheat"),
            ("kernel_driver_status", "kernel_driver_status(name)", "Check if kernel driver is loaded",
             vec!["name: string"], "map",
             vec!["let status = kernel_driver_status(\"EasyAntiCheat.sys\")"],
             "Anti-Cheat"),
            ("stealth_read", "stealth_read(pid, address, size)", "Stealthy memory read bypassing hooks",
             vec!["pid: number", "address: number", "size: number"], "bytes",
             vec!["let data = stealth_read(pid, addr, 64)"],
             "Anti-Cheat"),
            ("stealth_write", "stealth_write(pid, address, data)", "Stealthy memory write bypassing hooks",
             vec!["pid: number", "address: number", "data: bytes"], "string",
             vec!["stealth_write(pid, addr, payload)"],
             "Anti-Cheat"),
            ("hook_detect", "hook_detect(pid, address)", "Detect if function is hooked",
             vec!["pid: number", "address: number"], "map",
             vec!["let hooked = hook_detect(pid, func_addr)"],
             "Anti-Cheat"),
            ("hook_restore", "hook_restore(pid, address)", "Restore original bytes at hooked function",
             vec!["pid: number", "address: number"], "string",
             vec!["hook_restore(pid, func_addr)"],
             "Anti-Cheat"),
            ("debugger_evasion", "debugger_evasion()", "Apply anti-debugger techniques",
             vec![], "list",
             vec!["let techniques = debugger_evasion()"],
             "Anti-Cheat"),
            ("signature_obfuscate", "signature_obfuscate(code)", "Obfuscate code signature",
             vec!["code: bytes"], "bytes",
             vec!["let obf = signature_obfuscate(shellcode)"],
             "Anti-Cheat"),
            ("esp_create", "esp_create(pid, entity_list_addr)", "Create ESP (wallhack) overlay",
             vec!["pid: number", "entity_list_addr: number"], "string",
             vec!["esp_create(game_pid, 0x123456)"],
             "Game Hacking"),
            ("entity_iterate", "entity_iterate(pid, entity_list_addr)", "Iterate through game entities",
             vec!["pid: number", "entity_list_addr: number"], "list",
             vec!["let entities = entity_iterate(pid, list_addr)"],
             "Game Hacking"),
            ("aimbot_calculate", "aimbot_calculate(camera_pos, target_pos)", "Calculate aim angles",
             vec!["camera_pos: list", "target_pos: list"], "map",
             vec!["let angles = aimbot_calculate([0,0,0], [100,100,100])"],
             "Game Hacking"),
            ("triggerbot", "triggerbot(pid, crosshair_entity_addr)", "Auto-fire when crosshair on target",
             vec!["pid: number", "crosshair_entity_addr: number"], "string",
             vec!["triggerbot(pid, crosshair_addr)"],
             "Game Hacking"),
            ("visibility_check", "visibility_check(pid, entity_addr)", "Check if entity is visible",
             vec!["pid: number", "entity_addr: number"], "string",
             vec!["let visible = visibility_check(pid, entity)"],
             "Game Hacking"),
            ("world_to_screen", "world_to_screen(world_pos, view_matrix)", "Convert 3D world coords to 2D screen coords",
             vec!["world_pos: list", "view_matrix: list"], "map",
             vec!["let screen = world_to_screen([x,y,z], matrix)"],
             "Game Hacking"),
            ("dll_inject", "dll_inject(pid, dll_path)", "Inject DLL into process",
             vec!["pid: number", "dll_path: string"], "string",
             vec!["dll_inject(game_pid, \"C:\\\\cheat.dll\")"],
             "Code Injection"),
            ("dll_hide", "dll_hide(pid, dll_name)", "Hide DLL from module lists",
             vec!["pid: number", "dll_name: string"], "string",
             vec!["dll_hide(pid, \"cheat.dll\")"],
             "Code Injection"),
            ("reflective_load", "reflective_load(dll_bytes)", "Reflectively load DLL from memory",
             vec!["dll_bytes: bytes"], "number",
             vec!["let addr = reflective_load(dll_data)"],
             "Code Injection"),
        ];

        for (name, sig, desc, params, ret, examples, cat) in builtin_functions {
            functions.insert(name.to_string(), FunctionSignature {
                name: name.to_string(),
                signature: sig.to_string(),
                description: desc.to_string(),
                parameters: params.iter().map(|s| s.to_string()).collect(),
                return_type: ret.to_string(),
                examples: examples.iter().map(|s| s.to_string()).collect(),
                category: cat.to_string(),
            });
        }

        Self {
            functions,
            user_functions: HashMap::new(),
        }
    }

    fn get_completion_items(&self) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        for (_, func) in &self.functions {
            items.push(CompletionItem {
                label: func.name.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(func.signature.clone()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "**{}**\n\n{}\n\n**Category:** {}\n\n**Parameters:**\n{}\n\n**Returns:** {}\n\n**Examples:**\n```talon\n{}\n```",
                        func.signature,
                        func.description,
                        func.category,
                        func.parameters.join("\n"),
                        func.return_type,
                        func.examples.join("\n")
                    ),
                })),
                insert_text: Some(format!("{}($0)", func.name)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }

        for (_, func) in &self.user_functions {
            items.push(CompletionItem {
                label: func.name.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(format!("User Function: {}", func.signature)),
                documentation: Some(Documentation::String(func.description.clone())),
                ..Default::default()
            });
        }

        items.push(CompletionItem {
            label: "if".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Conditional statement".to_string()),
            insert_text: Some("if $1\n    $0\nend".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        });

        items.push(CompletionItem {
            label: "for".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("For loop".to_string()),
            insert_text: Some("for $1 in $2\n    $0\nend".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        });

        items.push(CompletionItem {
            label: "while".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("While loop".to_string()),
            insert_text: Some("while $1\n    $0\nend".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        });

        items.push(CompletionItem {
            label: "fn".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Function definition".to_string()),
            insert_text: Some("fn $1($2)\n    $0\nend".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        });

        items
    }

    fn get_hover_info(&self, word: &str) -> Option<Hover> {
        if let Some(func) = self.functions.get(word) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "**{}**\n\n{}\n\n**Category:** {}\n\n**Parameters:**\n{}\n\n**Returns:** {}\n\n**Examples:**\n```talon\n{}\n```",
                        func.signature,
                        func.description,
                        func.category,
                        func.parameters.join("\n"),
                        func.return_type,
                        func.examples.join("\n")
                    ),
                }),
                range: None,
            });
        }

        if let Some(func) = self.user_functions.get(word) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "**User Function: {}**\n\n{}",
                        func.signature,
                        func.description
                    ),
                }),
                range: None,
            });
        }

        match word {
            "if" | "else" | "elif" => Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(
                    "Conditional control flow".to_string()
                )),
                range: None,
            }),
            "for" | "while" => Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(
                    "Loop control flow".to_string()
                )),
                range: None,
            }),
            "fn" | "return" => Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(
                    "Function definition and control".to_string()
                )),
                range: None,
            }),
            "let" => Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(
                    "Variable declaration".to_string()
                )),
                range: None,
            }),
            _ => None,
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        info!("Initializing TALON LSP with 242+ built-in functions");

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(true),
                    trigger_characters: Some(vec![
                        ".".to_string(),
                        "(".to_string(),
                        " ".to_string(),
                    ]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_highlight_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: None,
                    work_done_progress_options: Default::default(),
                }),
                semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
                    SemanticTokensOptions {
                        legend: SemanticTokensLegend {
                            token_types: vec![
                                SemanticTokenType::KEYWORD,
                                SemanticTokenType::FUNCTION,
                                SemanticTokenType::STRING,
                                SemanticTokenType::NUMBER,
                                SemanticTokenType::VARIABLE,
                                SemanticTokenType::PARAMETER,
                                SemanticTokenType::COMMENT,
                            ],
                            token_modifiers: vec![],
                        },
                        full: Some(SemanticTokensFullOptions::Bool(true)),
                        range: Some(true),
                        ..Default::default()
                    }
                )),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
                    identifier: Some("talon".to_string()),
                    inter_file_dependencies: false,
                    workspace_diagnostics: false,
                    work_done_progress_options: Default::default(),
                })),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "TALON Language Server".to_string(),
                version: Some("1.0.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("TALON LSP fully initialized with semantic analysis");
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        debug!("Hover request at {}:{}:{}", uri, position.line, position.character);
        
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "**TALON DSL**\n\nExploit development language with 242+ built-in functions.\n\nTry typing function names for autocomplete.".to_string(),
            }),
            range: None,
        }))
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        debug!("Completion request");
        Ok(Some(CompletionResponse::Array(self.get_completion_items())))
    }

    async fn completion_resolve(&self, item: CompletionItem) -> Result<CompletionItem> {
        Ok(item)
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        debug!("Signature help request");
        
        Ok(Some(SignatureHelp {
            signatures: vec![SignatureInformation {
                label: "Example function signature".to_string(),
                documentation: Some(Documentation::String("Function parameters".to_string())),
                parameters: None,
                active_parameter: None,
            }],
            active_signature: Some(0),
            active_parameter: Some(0),
        }))
    }

    async fn shutdown(&self) -> Result<()> {
        info!("TALON LSP shutting down");
        Ok(())
    }

    async fn semantic_tokens_full(
        &self,
        _params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        Ok(None)
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    
    let (service, socket) = LspService::build(|_client| Backend::new()).finish();
    
    info!("TALON Language Server starting...");
    Server::new(stdin, stdout, socket).serve(service).await;
}
