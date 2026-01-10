use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;
use std::collections::HashMap;
use hex; // For decoding byte arrays
use crate::ast::{
    Command, Expr, Literal, Control, OffensiveCommand,
    TypeHint, TypedVar, FunctionDef, MatchArm, MatchBlock, TryCatch, MacroDef,
    SymbolicSpec, AutoExploitSpec, DebugSpec,
    FuzzProtocolSpec, HeapGroomSpec, KernelExploitSpec,
    SolidityAuditSpec, DistributeSpec, TimeTravelSpec,
    TimeTravelAction, ASLRBypassSpec, BinaryDiffSpec, WasmAnalysisSpec,
    ContainerEscapeSpec, CloudExploitSpec, TranslationSpec, DecompileSpec,
    DecompileTarget, AutoPatchSpec, CVEScanSpec, BinarySimilaritySpec,
};

// Define an enum to support both range-based and iterable-based for loops
#[derive(Debug)]
pub enum ForLoop {
    Range { var: String, start: Expr, end: Expr },
    Iterable { var: String, iterable: Expr },
}

#[derive(Parser)]
#[grammar = "lang.pest"]
pub struct LangParser;

pub fn parse_script(input: &str) -> Result<Vec<Command>, String> {
    let pairs = LangParser::parse(Rule::program, input)
        .map_err(|e| {
            let msg = format!("{}", e);
            if msg.contains("expected") {
                format!("[ERROR] Syntax Error: {}\n\nCommon mistakes:\n  - Missing 'end' keyword to close a block\n  - Unclosed string quotes\n  - Missing colon after parameter names\n  - Incorrect indentation", msg)
            } else {
                format!("[ERROR] Parse Error: {}", msg)
            }
        })?;
    let mut commands = Vec::new();
    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::statement => {
                        if let Some(stmt_inner) = inner_pair.into_inner().next() {
                            commands.extend(parse_stmt(stmt_inner)?);
                        }
                    }
                    Rule::command => commands.extend(parse_command(inner_pair.into_inner().next().unwrap())),
                    Rule::EOI => {},
                    _ => eprintln!("[WARN] Skipped node: {:?}", inner_pair.as_rule()),
                }
            }
        }
    }
    Ok(commands)
}

fn parse_stmt(pair: Pair<Rule>) -> Result<Vec<Command>, String> {
    let span = pair.as_span();
    match pair.as_rule() {
        Rule::include_stmt => {
            let path = pair.into_inner().next()
                .ok_or_else(|| format!("[ERROR] Missing include path at line {}:{}\n\nUsage: include \"path/to/file.talon\"", 
                    span.start_pos().line_col().0, span.start_pos().line_col().1))?
                .as_str().trim_matches('"').to_string();
            Ok(vec![Command::Include { path }])
        }
        Rule::import_stmt => {
            let mut inner = pair.into_inner();
            let module = inner.next()
                .ok_or_else(|| format!("Missing import module at line {}:{}", span.start_pos().line_col().0, span.start_pos().line_col().1))?
                .as_str().trim_matches('"').to_string();
            let items = inner.next().map(|items| {
                items.into_inner().map(|i| i.as_str().to_string()).collect()
            });
            Ok(vec![Command::Import { module, items }])
        }
        Rule::function_def => {
            let mut inner = pair.into_inner();
            let first = inner.next()
                .ok_or_else(|| format!("Missing function name at line {}:{}", span.start_pos().line_col().0, span.start_pos().line_col().1))?;
            let (is_async, name) = if first.as_str() == "async" {
                let name_pair = inner.next()
                    .ok_or_else(|| format!("Missing function name after async at line {}:{}", span.start_pos().line_col().0, span.start_pos().line_col().1))?;
                (true, name_pair.as_str().to_string())
            } else {
                (false, first.as_str().to_string())
            };
            let mut args = Vec::new();
            while let Some(arg) = inner.peek() {
                if arg.as_rule() == Rule::arg_def {
                    let mut arg_parts = arg.into_inner();
                    let arg_name = arg_parts.next().ok_or("Missing arg name")?.as_str().to_string();
                    let next_part = arg_parts.next();
                    let default = if let Some(p) = next_part {
                        if p.as_rule() == Rule::type_hint {
                            arg_parts.next().map(parse_expr)
                        } else {
                            Some(parse_expr(p))
                        }
                    } else {
                        None
                    };
                    args.push((arg_name, default));
                    inner.next();
                } else {
                    break;
                }
            }
            let return_type = if let Some(next) = inner.peek() {
                if next.as_rule() == Rule::type_hint {
                    let r = inner.next().unwrap();
                    Some(match r.as_str() {
                        "int" => TypeHint::Int,
                        "string" => TypeHint::String,
                        "list" => TypeHint::List,
                        "map" => TypeHint::Map,
                        "set" => TypeHint::Set,
                        "bytes" => TypeHint::Bytes,
                        _ => TypeHint::Unknown,
                    })
                } else {
                    None
                }
            } else {
                None
            };
            let mut body = Vec::new();
            for stmt in inner {
                if stmt.as_rule() == Rule::return_stmt {
                    let ret_expr = parse_expr(stmt.into_inner().next().ok_or("Missing return expr")?);
                    body.push(Command::Expr(Expr::Return(Box::new(ret_expr))));
                } else {
                    body.extend(parse_stmt(stmt)?);
                }
            }
            Ok(vec![Command::DefineFunction(FunctionDef { name, args, return_type, body, is_async })])
        }
        Rule::macro_def => {
            let mut inner = pair.into_inner();
            let name = inner.next().ok_or("Missing macro name")?.as_str().to_string();
            let mut args = Vec::new();
            while let Some(arg) = inner.peek() {
                if arg.as_rule() == Rule::ident {
                    args.push(arg.as_str().to_string());
                    inner.next();
                } else {
                    break;
                }
            }
            let mut body = Vec::new();
            for stmt in inner {
                body.extend(parse_stmt(stmt)?);
            }
            Ok(vec![Command::DefineMacro(MacroDef { name, args, body })])
        }
        Rule::call_macro => {
            let mut parts = pair.into_inner();
            let name = parts.next().ok_or("Missing macro name")?.as_str().to_string();
            let args = parts.map(parse_expr).collect();
            Ok(vec![Command::CallMacro { name, args }])
        }
        Rule::call_func => {
            Ok(vec![Command::Expr(parse_expr(pair))])
        }
        Rule::var_decl => {
            let mut parts = pair.into_inner();
            let name = parts.next().ok_or("Missing var name")?.as_str().to_string();
            let next_part = parts.next().ok_or("Missing var value")?;
            let (type_hint, expr_part) = if next_part.as_rule() == Rule::type_hint {
                let hint = match next_part.as_str() {
                    "int" => TypeHint::Int,
                    "string" => TypeHint::String,
                    "list" => TypeHint::List,
                    "map" => TypeHint::Map,
                    "set" => TypeHint::Set,
                    "bytes" => TypeHint::Bytes,
                    _ => TypeHint::Unknown,
                };
                (hint, parts.next().ok_or("Missing var value after type hint")?)
            } else {
                (TypeHint::Unknown, next_part)
            };
            let expr = parse_expr(expr_part);
            Ok(vec![Command::TypedDecl(TypedVar { name, var_type: type_hint, value: expr })])
        }
        Rule::const_decl => {
            let mut parts = pair.into_inner();
            let name = parts.next().ok_or("Missing const name")?.as_str().to_string();
            let expr = parse_expr(parts.next().ok_or("Missing const value")?);
            Ok(vec![Command::ConstDecl { name, value: expr }])
        }
        Rule::destructuring_decl => {
            let mut parts = pair.into_inner();
            let mut vars = Vec::new();
            for part in parts.by_ref() {
                if part.as_rule() == Rule::ident {
                    vars.push(part.as_str().to_string());
                } else {
                    break;
                }
            }
            let expr = parse_expr(parts.next().ok_or("Missing destructuring value")?);
            Ok(vec![Command::DestructuringDecl { vars, value: expr }])
        }
        Rule::assignment => {
            let mut parts = pair.into_inner();
            let name = parts.next().ok_or("Missing assignment name")?.as_str().to_string();
            let expr = parse_expr(parts.next().ok_or("Missing assignment value")?);
            Ok(vec![Command::Assignment { name, value: expr }])
        }
        Rule::struct_def => {
            let mut parts = pair.into_inner();
            let name = parts.next().ok_or("Missing struct name")?.as_str().to_string();
            let mut fields = Vec::new();
            for field_pair in parts {
                let mut inner = field_pair.into_inner();
                let fname = inner.next().ok_or("Missing field name")?.as_str().to_string();
                let ftype = inner.next().ok_or("Missing field type")?.as_str().to_string();
                fields.push((fname, ftype));
            }
            Ok(vec![Command::StructDef { name, fields }])
        }
        Rule::if_stmt => {
            let mut parts = pair.into_inner();
            let condition = parse_expr(parts.next().ok_or("Missing if condition")?);
            let mut then_body = Vec::new();
            let mut else_body = Vec::new();
            for stmt in parts {
                if stmt.as_rule() == Rule::else_stmt {
                    for inner_stmt in stmt.into_inner() {
                        else_body.extend(parse_stmt(inner_stmt)?);
                    }
                } else {
                    then_body.extend(parse_stmt(stmt)?);
                }
            }
            Ok(vec![Command::Control(Control::If { condition, then_body, else_body })])
        }
        Rule::for_stmt => {
            let mut parts = pair.into_inner();
            let var = parts.next().ok_or("Missing loop var")?.as_str().to_string();
            let next = parts.next().ok_or("Missing for loop part")?;
            let iterable = parse_expr(next);
            let mut body = Vec::new();
            for stmt in parts {
                body.extend(parse_stmt(stmt)?);
            }
            Ok(vec![Command::Control(Control::For { var, iterable, body })])
        }
        Rule::while_stmt => {
            let mut parts = pair.into_inner();
            let condition = parse_expr(parts.next().ok_or("Missing while condition")?);
            let mut body = Vec::new();
            for stmt in parts {
                body.extend(parse_stmt(stmt)?);
            }
            Ok(vec![Command::Control(Control::While { condition, body })])
        }
        Rule::break_stmt => {
            Ok(vec![Command::Control(Control::Break)])
        }
        Rule::continue_stmt => {
            Ok(vec![Command::Control(Control::Continue)])
        }
        Rule::parallel_stmt => {
            let mut body = Vec::new();
            for stmt in pair.into_inner() {
                body.extend(parse_stmt(stmt)?);
            }
            Ok(vec![Command::Control(Control::Parallel { body })])
        }
        Rule::match_stmt => {
            let mut parts = pair.into_inner();
            let expr = parse_expr(parts.next().ok_or("Missing match expr")?);
            let mut arms = Vec::new();
            for case in parts {
                if case.as_rule() == Rule::case_stmt {
                    let mut case_parts = case.into_inner();
                    let pattern = parse_expr(case_parts.next().ok_or("Missing case pattern")?);
                    let guard = case_parts.next().and_then(|g| {
                        if g.as_rule() == Rule::guard {
                            Some(parse_expr(g.into_inner().next().unwrap()))
                        } else {
                            None
                        }
                    });
                    let mut body = Vec::new();
                    for stmt in case_parts {
                        body.extend(parse_stmt(stmt)?);
                    }
                    arms.push(MatchArm { pattern, guard, body });
                }
            }
            Ok(vec![Command::Match(MatchBlock { expr, arms })])
        }
        Rule::try_catch_stmt => {
            let parts = pair.into_inner();
            let mut try_body = Vec::new();
            let mut catch_var = String::new();
            let mut catch_body = Vec::new();
            let mut in_catch = false;
            for part in parts {
                if part.as_rule() == Rule::catch_stmt {
                    in_catch = true;
                    let mut catch_parts = part.into_inner();
                    catch_var = catch_parts.next().ok_or("Missing catch var")?.as_str().to_string();
                } else if in_catch {
                    catch_body.extend(parse_stmt(part)?);
                } else {
                    try_body.extend(parse_stmt(part)?);
                }
            }
            Ok(vec![Command::TryCatch(TryCatch { try_body, catch_var, catch_body })])
        }
        Rule::bitwise_op => {
            let mut parts = pair.into_inner();
            let left = parse_expr(parts.next().ok_or("Missing left operand")?);
            let op = parts.next().ok_or("Missing operator")?.as_str().to_string();
            let right = parse_expr(parts.next().ok_or("Missing right operand")?);
            Ok(vec![Command::BitwiseOp { op, left, right }])
        }
        Rule::tool_exec => {
            let mut parts = pair.into_inner();
            let tool = parts.next().ok_or("Missing tool name")?.as_str().to_string();
            let args = parts.map(parse_expr).collect();
            Ok(vec![Command::ToolExec { tool, args }])
        }
        Rule::expr => Ok(vec![Command::Expr(parse_expr(pair))]),
        Rule::command => Ok(parse_command(pair.into_inner().next().unwrap())),
        Rule::statement => {
            let inner = pair.into_inner().next();
            if let Some(inner_pair) = inner {
                parse_stmt(inner_pair)
            } else {
                Ok(vec![])
            }
        }
        _ => Ok(vec![]),
    }
}

fn parse_command(pair: Pair<Rule>) -> Vec<Command> {
    match pair.as_rule() {
        Rule::run_cmd => {
            let cmd = pair.into_inner().next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::RunCommand { command: cmd }]
        }
        Rule::load_shellcode_cmd => {
            let path = pair.into_inner().next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::LoadShellcode { path }]
        }
        Rule::execute_shellcode_cmd => vec![Command::ExecuteShellcode],
        Rule::read_file_cmd => {
            let mut parts = pair.into_inner();
            let path = parts.next().unwrap().as_str().trim_matches('"').to_string();
            let var = parts.next().unwrap().as_str().to_string();
            vec![Command::ReadFile { path, var }]
        }
        Rule::write_file_cmd => {
            let mut parts = pair.into_inner();
            let expr = parse_expr(parts.next().unwrap());
            let path = parts.next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::WriteFile { data: expr, path }]
        }
        Rule::format_string_cmd => {
            let mut parts = pair.into_inner();
            let target = parts.next().unwrap().as_str().trim_matches('"').to_string();
            let offset = parts.next().unwrap().as_str().parse::<u32>().unwrap();
            vec![Command::FormatStringExploit { target, offset }]
        }
        Rule::stack_overflow_cmd => {
            let mut parts = pair.into_inner();
            let padding = parts.next().unwrap().as_str().parse::<u32>().unwrap();
            let addr = u64::from_str_radix(parts.next().unwrap().as_str().trim_start_matches("0x"), 16).unwrap();
            vec![Command::StackOverflowExploit { padding, ret_addr: addr }]
        }
        Rule::dump_memory_cmd => {
            let mut parts = pair.into_inner();
            let addr = u64::from_str_radix(parts.next().unwrap().as_str().trim_start_matches("0x"), 16).unwrap();
            let len = parts.next().unwrap().as_str().parse::<u32>().unwrap();
            vec![Command::DumpMemory { address: addr, length: len }]
        }
        Rule::assemble_cmd => {
            let code = pair.into_inner().next().unwrap().as_str().to_string();
            vec![Command::Assemble { code }]
        }
        Rule::beacon_cmd => {
            let mut parts = pair.into_inner();
            let url = parts.next().unwrap().as_str().trim_matches('"').to_string();
            let interval = parts.next().unwrap().as_str().parse::<u64>().unwrap();
            vec![Command::Beacon { url, interval }]
        }
        Rule::download_cmd => {
            let mut parts = pair.into_inner();
            let url = parts.next().unwrap().as_str().trim_matches('"').to_string();
            let path = parts.next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::Download { url, path }]
        }
        Rule::encode_base64_cmd => {
            let expr = parse_expr(pair.into_inner().next().unwrap());
            vec![Command::EncodeBase64 { data: expr }]
        }
        Rule::decode_base64_cmd => {
            let expr = parse_expr(pair.into_inner().next().unwrap());
            vec![Command::DecodeBase64 { data: expr }]
        }
        Rule::offensive_cmd => parse_offensive_command(pair),
        Rule::nop_sled_cmd => {
            let length = pair.into_inner().next().unwrap().as_str().parse::<u32>().unwrap();
            vec![Command::NopSled { length }]
        }
        Rule::heap_spray_cmd => {
            let data = pair.into_inner().next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::HeapSpray { data }]
        }
        Rule::sigrop_chain_cmd => {
            let lib = pair.into_inner().next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::SigropChain { lib }]
        }
        Rule::find_format_offset_cmd => {
            let binary = pair.into_inner().next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::FindFormatOffset { binary }]
        }
        Rule::visualize_heap_cmd => {
            let binary = pair.into_inner().next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::VisualizeHeap { binary }]
        }
        Rule::fuzz_cmd => {
            let mut parts = pair.into_inner();
            let binary = parts.next().unwrap().as_str().trim_matches('"').to_string();
            let seed = parts.next().unwrap().as_str().trim_matches('"').to_string();
            let cycles = parts.next().unwrap().as_str().parse::<u32>().unwrap();
            vec![Command::Fuzz { binary, seed, cycles }]
        }
        
        Rule::symbolic_cmd => {
            let mut parts = pair.into_inner();
            let var_name = parts.next().unwrap().as_str().to_string();
            let var_type = parts.next().unwrap().as_str().to_string();
            let size = parts.next().and_then(|p| p.as_str().parse::<usize>().ok());
            vec![Command::SymbolicExecution(SymbolicSpec { var_name, var_type, size })]
        }
        
        Rule::ai_cmd => {
            let mut parts = pair.into_inner();
            let binary = parts.next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::AutoExploit(AutoExploitSpec {
                binary,
                target_type: "auto".to_string(),
                constraints: vec![],
                objective: "shell".to_string(),
            })]
        }
        
        Rule::debug_cmd => {
            let mut parts = pair.into_inner();
            let binary = parts.next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::DebugAttach(DebugSpec {
                binary,
                breakpoints: vec![],
                watches: vec![],
                on_break: vec![],
            })]
        }
        
        Rule::fuzz_advanced_cmd => {
            let mut parts = pair.into_inner();
            let protocol = parts.next().unwrap().as_str().to_string();
            vec![Command::FuzzProtocol(FuzzProtocolSpec {
                protocol,
                grammar: HashMap::new(),
                coverage_guided: true,
                max_iterations: 1000000,
                crash_triage: true,
            })]
        }
        
        Rule::heap_cmd => {
            let mut parts = pair.into_inner();
            let target_str = parts.next().unwrap().as_str().trim_start_matches("0x");
            let target_addr = u64::from_str_radix(target_str, 16).unwrap_or(0);
            vec![Command::HeapGroom(HeapGroomSpec {
                target_addr,
                spray_size: None,
                spray_count: None,
                free_indices: vec![],
                allocate_size: None,
                allocate_data: None,
            })]
        }
        
        Rule::gadget_cmd => {
            let path = pair.into_inner().next().unwrap().as_str().trim_matches('"').to_string();
            if path.contains("one_gadget") {
                vec![Command::FindOneGadget { libc_path: path }]
            } else {
                vec![Command::FindMagicGadget { pattern: path, constraints: vec![] }]
            }
        }
        
        Rule::kernel_cmd => {
            vec![Command::KernelExploit(KernelExploitSpec {
                auto_detect: true,
                target_cve: None,
                bypass_kaslr: true,
                bypass_smep: true,
                bypass_smap: true,
                disable_selinux: true,
                container_escape: true,
            })]
        }
        
        Rule::scan_cve_cmd => {
            let mut parts = pair.into_inner();
            let target = parts.next().unwrap().as_str().trim_matches('"').to_string();
            
            let mut cve_list = Vec::new();
            let mut suggest_exploit = true;
            let mut generate_poc = true;
            
            for part in parts {
                if part.as_rule() == Rule::assignment {
                    let mut assign_parts = part.into_inner();
                    let key = assign_parts.next().unwrap().as_str();
                    let value_pair = assign_parts.next().unwrap();
                    
                    match key {
                        "check" => {
                            if value_pair.as_rule() == Rule::list {
                                for item in value_pair.into_inner() {
                                    let cve = item.as_str().trim_matches('"').to_string();
                                    cve_list.push(cve);
                                }
                            }
                        }
                        "suggest_exploit" => {
                            suggest_exploit = value_pair.as_str() == "true";
                        }
                        "generate_poc" => {
                            generate_poc = value_pair.as_str() == "true";
                        }
                        _ => {}
                    }
                }
            }
            
            vec![Command::CVEScan(CVEScanSpec {
                target,
                cve_list,
                suggest_exploit,
                generate_poc,
            })]
        }
        
        Rule::similarity_cmd => {
            let mut parts = pair.into_inner();
            let reference = parts.next().unwrap().as_str().trim_matches('"').to_string();
            
            let mut search_in = Vec::new();
            let mut threshold = 0.85;
            let mut output = "text".to_string();
            
            for part in parts {
                if part.as_rule() == Rule::assignment {
                    let mut assign_parts = part.into_inner();
                    let key = assign_parts.next().unwrap().as_str();
                    let value_pair = assign_parts.next().unwrap();
                    
                    match key {
                        "search_in" => {
                            if value_pair.as_rule() == Rule::list {
                                for item in value_pair.into_inner() {
                                    let pattern = item.as_str().trim_matches('"').to_string();
                                    search_in.push(pattern);
                                }
                            }
                        }
                        "threshold" => {
                            threshold = value_pair.as_str().parse::<f64>().unwrap_or(0.85);
                        }
                        "output" => {
                            output = value_pair.as_str().trim_matches('"').to_string();
                        }
                        _ => {}
                    }
                }
            }
            
            vec![Command::BinarySimilarity(BinarySimilaritySpec {
                reference,
                search_in,
                threshold,
                output,
            })]
        }
        
        Rule::chain_cmd => {
            let text = pair.as_str();
            if text.starts_with("connect_to") {
                let mut parts = pair.into_inner();
                let host = parts.next().unwrap().as_str().trim_matches('"').to_string();
                let port = parts.next().unwrap().as_str().parse::<u16>().unwrap_or(0);
                let timeout = parts.next().map(|p| p.as_str().parse::<u64>().unwrap_or(5));
                vec![Command::ChainConnect { host, port, timeout }]
            } else if text.starts_with("send") {
                let mut parts = pair.into_inner();
                let data = parse_expr(parts.next().unwrap());
                vec![Command::ChainSend { data }]
            } else if text.starts_with("receive_until") {
                let mut parts = pair.into_inner();
                let delimiter = parts.next().unwrap().as_str().trim_matches('"').to_string();
                parts.next();
                let max_size = parts.next().unwrap().as_str().parse::<usize>().unwrap_or(4096);
                vec![Command::ChainReceiveUntil { delimiter, max_size }]
            } else if text.starts_with("receive") {
                let mut parts = pair.into_inner();
                let size = parts.next().unwrap().as_str().parse::<usize>().unwrap_or(4096);
                vec![Command::ChainReceive { size }]
            } else if text.starts_with("exploit_leak") {
                let mut parts = pair.into_inner();
                let stage_name = parts.next().unwrap().as_str().trim_matches('"').to_string();
                parts.next();
                let payload = parse_expr(parts.next().unwrap());
                parts.next();
                let offset = parts.next().unwrap().as_str().parse::<usize>().unwrap_or(0);
                parts.next();
                let size = parts.next().unwrap().as_str().parse::<usize>().unwrap_or(8);
                vec![Command::ChainExploitLeak { stage_name, payload, offset, size }]
            } else if text.starts_with("calculate_base") {
                let mut parts = pair.into_inner();
                let leaked_addr = parse_expr(parts.next().unwrap());
                parts.next();
                let offset = u64::from_str_radix(parts.next().unwrap().as_str().trim_start_matches("0x"), 16).unwrap_or(0);
                parts.next();
                let name = parts.next().unwrap().as_str().trim_matches('"').to_string();
                vec![Command::ChainCalculateBase { leaked_addr, offset, name }]
            } else if text.starts_with("bruteforce_aslr") {
                let mut parts = pair.into_inner();
                parts.next();
                let attempts = parts.next().unwrap().as_str().parse::<usize>().unwrap_or(1000);
                parts.next();
                let payload = parse_expr(parts.next().unwrap());
                parts.next();
                let offset = parts.next().unwrap().as_str().parse::<usize>().unwrap_or(0);
                vec![Command::ChainBruteforceASLR { attempts, payload, offset }]
            } else if text.starts_with("interactive") {
                vec![Command::ChainInteractive]
            } else if text.starts_with("save_chain_state") {
                let mut parts = pair.into_inner();
                let path = parts.next().unwrap().as_str().trim_matches('"').to_string();
                vec![Command::ChainSaveState { path }]
            } else if text.starts_with("load_chain_state") {
                let mut parts = pair.into_inner();
                let path = parts.next().unwrap().as_str().trim_matches('"').to_string();
                vec![Command::ChainLoadState { path }]
            } else if text.starts_with("chain_summary") {
                vec![Command::ChainPrintSummary]
            } else {
                vec![]
            }
        }
        
        Rule::safety_cmd => {
            let text = pair.as_str();
            if text.starts_with("set_timeout") {
                let mut parts = pair.into_inner();
                let milliseconds = parts.next().unwrap().as_str().parse::<u64>().unwrap_or(60000);
                vec![Command::SetTimeout { milliseconds }]
            } else if text.starts_with("set_memory_limit") {
                let mut parts = pair.into_inner();
                let megabytes = parts.next().unwrap().as_str().parse::<usize>().unwrap_or(512);
                vec![Command::SetMemoryLimit { megabytes }]
            } else if text.starts_with("set_recursion_limit") {
                let mut parts = pair.into_inner();
                let max_depth = parts.next().unwrap().as_str().parse::<usize>().unwrap_or(1000);
                vec![Command::SetRecursionLimit { max_depth }]
            } else if text.starts_with("enable_strict_mode") {
                vec![Command::EnableStrictMode]
            } else if text.starts_with("disable_strict_mode") {
                vec![Command::DisableStrictMode]
            } else if text.starts_with("get_safety_stats") {
                vec![Command::GetSafetyStats]
            } else if text.starts_with("reset_safety") {
                vec![Command::ResetSafety]
            } else {
                vec![]
            }
        }
        
        Rule::solidity_cmd => {
            let mut parts = pair.into_inner();
            let contract_path = parts.next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::AuditSolidity(SolidityAuditSpec {
                contract_path,
                detect: vec!["reentrancy".to_string()],
                auto_exploit: true,
            })]
        }
        
        Rule::distributed_cmd => {
            let mut parts = pair.into_inner();
            let target_range = parts.next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::DistributeExploit(DistributeSpec {
                target_range,
                threads: 64,
                exploit_type: "auto".to_string(),
                callback: None,
            })]
        }
        
        Rule::timetravel_cmd => {
            let mut parts = pair.into_inner();
            let binary = parts.next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::TimeTravelDebug(TimeTravelSpec {
                binary,
                record_replay: "rr".to_string(),
                actions: vec![TimeTravelAction::ReverseContinue],
            })]
        }
        
        Rule::aslr_cmd => {
            let mut parts = pair.into_inner();
            let binary = parts.next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::BypassASLR(ASLRBypassSpec {
                binary,
                method: "auto".to_string(),
                leak_gadgets: vec![],
            })]
        }
        
        Rule::diff_cmd => {
            let mut parts = pair.into_inner();
            let file1 = parts.next().unwrap().as_str().trim_matches('"').to_string();
            let file2 = parts.next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::BinaryDiff(BinaryDiffSpec {
                file1,
                file2,
                find_patches: true,
                identify_ndays: true,
            })]
        }
        
        Rule::wasm_cmd => {
            let mut parts = pair.into_inner();
            let wasm_path = parts.next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::AnalyzeWasm(WasmAnalysisSpec {
                wasm_path,
                decompile: true,
                find_vulns: vec![],
            })]
        }
        
        Rule::container_cmd => {
            let method_str = pair.into_inner().next().unwrap().as_str();
            vec![Command::ContainerEscape(ContainerEscapeSpec {
                method: vec![method_str.to_string()],
                pivot_target: "host".to_string(),
            })]
        }
        
        Rule::cloud_cmd => {
            let provider = pair.into_inner().next().unwrap().as_str();
            vec![Command::CloudExploit(CloudExploitSpec {
                provider: provider.to_string(),
                ssrf_target: Some("metadata".to_string()),
                extract_creds: true,
                escalate_role: true,
            })]
        }
        
        Rule::translate_cmd => {
            let mut parts = pair.into_inner();
            let from_arch = parts.next().unwrap().as_str().to_string();
            let to_arch = parts.next().unwrap().as_str().to_string();
            vec![Command::TranslateShellcode(TranslationSpec {
                shellcode: vec![],
                from_arch,
                to_arch,
                optimize: true,
            })]
        }
        
        Rule::decompile_cmd => {
            let mut parts = pair.into_inner();
            let target_str = parts.next().unwrap().as_str();
            let target = if target_str.starts_with("0x") {
                let addr = u64::from_str_radix(target_str.trim_start_matches("0x"), 16).unwrap_or(0);
                DecompileTarget::Address(addr)
            } else {
                DecompileTarget::Binary(target_str.trim_matches('"').to_string())
            };
            vec![Command::Decompile(DecompileSpec {
                target,
                output_lang: "c".to_string(),
                annotate: true,
            })]
        }
        
        Rule::patch_cmd => {
            let mut parts = pair.into_inner();
            let file = parts.next().unwrap().as_str().trim_matches('"').to_string();
            vec![Command::AutoPatch(AutoPatchSpec {
                file,
                function: None,
                fix_type: "buffer_overflow".to_string(),
                verify_method: "fuzzing".to_string(),
            })]
        }
        
        _ => vec![Command::Expr(Expr::Literal(Literal::String(format!("[UNHANDLED COMMAND] {}", pair.as_str()))))],
    }
}

fn parse_offensive_command(pair: Pair<Rule>) -> Vec<Command> {
    let mut cmds = vec![];
    let cmd = pair.into_inner().next().unwrap();
    match cmd.as_rule() {
        Rule::ghidra_bridge_cmd => {
            let mut parts = cmd.into_inner();
            let script = parts.next().unwrap().as_str().trim_matches('"').to_string();
            let binary = parts.next().unwrap().as_str().trim_matches('"').to_string();
            cmds.push(Command::Offensive(OffensiveCommand::BridgeGhidra { script, binary }));
        }
        Rule::ida_bridge_cmd => {
            let mut parts = cmd.into_inner();
            let script = parts.next().unwrap().as_str().trim_matches('"').to_string();
            let binary = parts.next().unwrap().as_str().trim_matches('"').to_string();
            cmds.push(Command::Offensive(OffensiveCommand::BridgeIDA { script, binary }));
        }
        _ => cmds.push(Command::Expr(Expr::Literal(Literal::String("[UNKNOWN offensive_cmd]".into())))),
    }
    cmds
}

fn parse_expr(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::expr => parse_pipe(pair.into_inner().next().unwrap()),
        Rule::pipe => parse_pipe(pair),
        Rule::comparison => parse_comparison(pair),
        Rule::literal => {
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::quoted_string => Expr::Literal(Literal::String(inner.as_str().trim_matches('"').to_string())),
                Rule::multiline_string => Expr::Literal(Literal::String(inner.as_str().trim_matches('"').replace("\"\"\"", ""))),
                Rule::number => {
                    let num_str = inner.as_str();
                    let value = if num_str.starts_with("0x") || num_str.starts_with("0X") {
                        i64::from_str_radix(&num_str[2..], 16).unwrap_or(0)
                    } else {
                        num_str.parse::<i64>().unwrap_or(0)
                    };
                    Expr::Literal(Literal::Number(value))
                },
                Rule::boolean => Expr::Literal(Literal::Boolean(inner.as_str() == "true")),
                Rule::null => Expr::Literal(Literal::Null),
                Rule::byte_array => Expr::Literal(Literal::ByteArray(inner.as_str().trim_start_matches("0x").to_string())),
                _ => unreachable!(),
            }
        }
        Rule::ident => Expr::Ident(pair.as_str().to_string()),
        Rule::interpolated_string => {
            let mut parts = Vec::new();
            for part in pair.into_inner() {
                if part.as_rule() == Rule::string_part {
                    parts.push(Expr::Literal(Literal::String(part.as_str().to_string())));
                } else if part.as_rule() == Rule::interp_expr {
                    parts.push(parse_expr(part.into_inner().next().unwrap()));
                }
            }
            Expr::InterpolatedString(parts)
        }
        Rule::list => {
            let items = pair.into_inner().map(|item| {
                if item.as_rule() == Rule::list_item {
                    let inner = item.into_inner().next().unwrap();
                    if inner.as_rule() == Rule::spread_expr {
                        let spread_inner = inner.into_inner().next().unwrap();
                        Expr::Spread(Box::new(parse_expr(spread_inner)))
                    } else {
                        parse_expr(inner)
                    }
                } else {
                    parse_expr(item)
                }
            }).collect();
            Expr::List(items)
        }
        Rule::map => {
            let mut map = HashMap::new();
            for pair in pair.into_inner() {
                let mut parts = pair.into_inner();
                let key = parts.next().unwrap().as_str().to_string();
                let value = parse_expr(parts.next().unwrap());
                map.insert(key, value);
            }
            Expr::Map(map)
        }
        Rule::set => Expr::Set(pair.into_inner().map(parse_expr).collect()),
        Rule::bytes_expr => Expr::Bytes(hex::decode(pair.into_inner().next().unwrap().as_str().trim_start_matches("0x")).unwrap_or_default()),
        Rule::lambda => {
            let mut parts = pair.into_inner();
            let arg = parts.next().unwrap().as_str().to_string();
            let body = Box::new(parse_expr(parts.next().unwrap()));
            Expr::Lambda { arg, body }
        }
        Rule::list_comprehension => {
            let mut parts = pair.into_inner();
            let expr = Box::new(parse_expr(parts.next().unwrap()));
            let var = parts.next().unwrap().as_str().to_string();
            let iterable = Box::new(parse_expr(parts.next().unwrap()));
            Expr::ListComprehension { expr, var, iterable }
        }
        Rule::variant => {
            let mut parts = pair.into_inner();
            let name = parts.next().unwrap().as_str().to_string();
            let value = parts.next().map(|p| Box::new(parse_expr(p)));
            Expr::Variant(name, value)
        }
        Rule::env => Expr::Env(pair.into_inner().next().unwrap().as_str().trim_matches('"').to_string()),
        Rule::regex_match => {
            let mut parts = pair.into_inner();
            let regex = parts.next().unwrap().as_str().trim_matches('/').to_string();
            let haystack = Box::new(parse_primary(parts.next().unwrap()));
            Expr::RegexMatch { regex, haystack }
        }
        Rule::await_expr => {
            let expr = Box::new(parse_primary(pair.into_inner().next().unwrap()));
            Expr::Await(expr)
        }
        Rule::call_func => {
            let mut parts = pair.into_inner();
            let name = parts.next().unwrap().as_str().to_string();
            let args = parts.map(|p| {
                if p.as_rule() == Rule::named_arg {
                    let mut arg_parts = p.into_inner();
                    let key = arg_parts.next().unwrap().as_str().to_string();
                    let value = parse_expr(arg_parts.next().unwrap());
                    (Some(key), value)
                } else {
                    (None, parse_expr(p))
                }
            }).collect();
            Expr::Call { name, args }
        }
        Rule::call_macro => {
            let mut parts = pair.into_inner();
            let name = parts.next().unwrap().as_str().to_string();
            let args = parts.map(parse_expr).collect();
            Expr::MacroCall { name, args }
        }
        Rule::pack_expr => {
            let func_name = pair.as_str().split('(').next().unwrap();
            let size = match func_name {
                "p64" => 64,
                "p32" => 32,
                "p16" => 16,
                "p8" => 8,
                _ => 64,
            };
            let value = Box::new(parse_expr(pair.into_inner().next().unwrap()));
            Expr::Pack { size, value }
        }
        Rule::unpack_expr => {
            let func_name = pair.as_str().split('(').next().unwrap();
            let size = match func_name {
                "u64" => 64,
                "u32" => 32,
                "u16" => 16,
                "u8" => 8,
                _ => 64,
            };
            let data = Box::new(parse_expr(pair.into_inner().next().unwrap()));
            Expr::Unpack { size, data }
        }
        Rule::range => {
            let mut parts = pair.into_inner();
            let start_pair = parts.next().unwrap();
            let start_expr = parse_expr(start_pair);
            let end_pair = parts.next().unwrap();
            let end_expr = parse_expr(end_pair);
            
            let start = match start_expr {
                Expr::Literal(Literal::Number(n)) => n,
                _ => 0,
            };
            let end = match end_expr {
                Expr::Literal(Literal::Number(n)) => n,
                _ => 0,
            };
            Expr::Literal(Literal::String(format!("{}..{}", start, end)))
        }
        Rule::logical_or => parse_logical_or(pair),
        Rule::logical_and => parse_logical_and(pair),
        Rule::term => parse_term(pair),
        Rule::factor => parse_factor(pair),
        Rule::unary => parse_unary(pair),
        Rule::primary => parse_primary(pair),
        _ => unreachable!("Unexpected expr rule: {:?}", pair.as_rule()),
    }
}

fn parse_pipe(pair: Pair<Rule>) -> Expr {
    let mut stages = Vec::new();
    for p in pair.into_inner() {
        stages.push(parse_logical_or(p));
    }
    if stages.len() == 1 {
        stages.into_iter().next().unwrap()
    } else {
        Expr::Pipe { stages }
    }
}

fn parse_logical_or(pair: Pair<Rule>) -> Expr {
    let mut parts = pair.into_inner();
    let mut left = parse_logical_and(parts.next().unwrap());
    while let Some(next) = parts.next() {
        let right = parse_logical_and(next);
        left = Expr::BinaryOp { 
            op: "or".to_string(), 
            left: Box::new(left), 
            right: Box::new(right) 
        };
    }
    left
}

fn parse_logical_and(pair: Pair<Rule>) -> Expr {
    let mut parts = pair.into_inner();
    let mut left = parse_comparison(parts.next().unwrap());
    while let Some(next) = parts.next() {
        let right = parse_comparison(next);
        left = Expr::BinaryOp { 
            op: "and".to_string(), 
            left: Box::new(left), 
            right: Box::new(right) 
        };
    }
    left
}

fn parse_comparison(pair: Pair<Rule>) -> Expr {
    let mut parts = pair.into_inner();
    let mut left = parse_term(parts.next().unwrap());
    while let Some(op_pair) = parts.next() {
        if op_pair.as_rule() == Rule::comp_op {
            let right = parse_term(parts.next().unwrap());
            left = Expr::ComparisonOp { 
                op: op_pair.as_str().to_string(), 
                left: Box::new(left), 
                right: Box::new(right) 
            };
        } else {
            left = parse_term(op_pair);
        }
    }
    left
}

fn parse_term(pair: Pair<Rule>) -> Expr {
    let mut parts = pair.into_inner();
    let mut left = parse_factor(parts.next().unwrap());
    while let Some(op_pair) = parts.next() {
        if op_pair.as_rule() == Rule::add_op {
            let right = parse_factor(parts.next().unwrap());
            left = Expr::BinaryOp { 
                op: op_pair.as_str().to_string(), 
                left: Box::new(left), 
                right: Box::new(right) 
            };
        } else {
            left = parse_factor(op_pair);
        }
    }
    left
}

fn parse_factor(pair: Pair<Rule>) -> Expr {
    let mut parts = pair.into_inner();
    let mut left = parse_unary(parts.next().unwrap());
    while let Some(op_pair) = parts.next() {
        if op_pair.as_rule() == Rule::mul_op {
            let right = parse_unary(parts.next().unwrap());
            left = Expr::BinaryOp { 
                op: op_pair.as_str().to_string(), 
                left: Box::new(left), 
                right: Box::new(right) 
            };
        } else {
            left = parse_unary(op_pair);
        }
    }
    left
}

fn parse_unary(pair: Pair<Rule>) -> Expr {
    let mut parts = pair.into_inner();
    let mut base = parse_primary(parts.next().unwrap());
    
    for postfix in parts {
        if postfix.as_rule() == Rule::postfix {
            let mut post_parts = postfix.into_inner();
            let first = post_parts.next().unwrap();
            
            if first.as_rule() == Rule::ident {
                let method = first.as_str().to_string();
                base = Expr::MethodChain { 
                    base: Box::new(base), 
                    calls: vec![method] 
                };
            } else if first.as_rule() == Rule::slice_range {
                let mut range_parts = first.into_inner();
                let start = Box::new(parse_expr(range_parts.next().unwrap()));
                let end = Box::new(parse_expr(range_parts.next().unwrap()));
                base = Expr::Slice { base: Box::new(base), start, end };
            } else if first.as_rule() == Rule::expr {
                base = Expr::Index { base: Box::new(base), index: Box::new(parse_expr(first)) };
            } else {
                let args = post_parts.map(|p| {
                    if p.as_rule() == Rule::named_arg {
                        let mut arg_parts = p.into_inner();
                        let key = arg_parts.next().unwrap().as_str().to_string();
                        let value = parse_expr(arg_parts.next().unwrap());
                        (Some(key), value)
                    } else {
                        (None, parse_expr(p))
                    }
                }).collect();
                
                if let Expr::Ident(name) = base {
                    base = Expr::Call { name, args };
                }
            }
        }
    }
    
    base
}

fn parse_primary(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::primary => {
            parse_expr(pair.into_inner().next().unwrap())
        }
        Rule::expr => {
            parse_expr(pair)
        }
        _ => parse_expr(pair)
    }
}
