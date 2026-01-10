use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use crate::ast::Command;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub path: PathBuf,
    pub exports: HashMap<String, ModuleExport>,
    pub statements: Vec<Command>,
}

#[derive(Debug, Clone)]
pub enum ModuleExport {
    Function(String),
    Variable(String),
    Type(String),
}

pub struct ModuleLoader {
    loaded_modules: HashMap<String, Module>,
    search_paths: Vec<PathBuf>,
}

impl ModuleLoader {
    pub fn new() -> Self {
        ModuleLoader {
            loaded_modules: HashMap::new(),
            search_paths: vec![
                PathBuf::from("."),
                PathBuf::from("./stdlib"),
                PathBuf::from("./modules"),
            ],
        }
    }

    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    pub fn load_module(&mut self, module_path: &str) -> Result<&Module, String> {
        if self.loaded_modules.contains_key(module_path) {
            return Ok(&self.loaded_modules[module_path]);
        }

        let resolved_path = self.resolve_module_path(module_path)?;
        let source = fs::read_to_string(&resolved_path)
            .map_err(|e| format!("Failed to read module {}: {}", module_path, e))?;

        let statements = crate::parser::parse_script(&source)
            .map_err(|e| format!("Failed to parse module {}: {}", module_path, e))?;

        let exports = self.extract_exports(&statements);

        let module = Module {
            name: module_path.to_string(),
            path: resolved_path,
            exports,
            statements,
        };

        self.loaded_modules.insert(module_path.to_string(), module);
        Ok(&self.loaded_modules[module_path])
    }

    fn resolve_module_path(&self, module_path: &str) -> Result<PathBuf, String> {
        for search_path in &self.search_paths {
            let mut full_path = search_path.join(module_path);
            
            if !full_path.extension().is_some() {
                full_path.set_extension("talon");
            }

            if full_path.exists() {
                return Ok(full_path);
            }
        }

        Err(format!("Module not found: {}", module_path))
    }

    fn extract_exports(&self, _statements: &[Command]) -> HashMap<String, ModuleExport> {
        let mut exports = HashMap::new();
        exports.insert("default".to_string(), ModuleExport::Function("default".to_string()));
        exports
    }

    pub fn get_loaded_modules(&self) -> &HashMap<String, Module> {
        &self.loaded_modules
    }
}

pub fn parse_import_statement(stmt: &str) -> Option<(String, Option<Vec<String>>)> {
    let parts: Vec<&str> = stmt.split_whitespace().collect();
    
    if parts.len() < 2 {
        return None;
    }

    if parts[0] != "import" {
        return None;
    }

    if parts.len() == 2 {
        return Some((parts[1].to_string(), None));
    }

    if parts.len() >= 4 && parts[2] == "from" {
        let symbols: Vec<String> = parts[1]
            .trim_matches(|c| c == '{' || c == '}')
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        return Some((parts[3].to_string(), Some(symbols)));
    }

    Some((parts[1].to_string(), None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_import() {
        assert_eq!(
            parse_import_statement("import rop_utils"),
            Some(("rop_utils".to_string(), None))
        );

        assert_eq!(
            parse_import_statement("import { rop_chain, find_gadgets } from rop_utils"),
            Some((
                "rop_utils".to_string(),
                Some(vec!["rop_chain".to_string(), "find_gadgets".to_string()])
            ))
        );
    }
}
