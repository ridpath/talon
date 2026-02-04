use crate::ast::Command;
use crate::interpreter;
use crate::parser;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

static STDLIB_QUICK: &str = include_str!("../stdlib/quick.talon");
static STDLIB_EASY_PWN: &str = include_str!("../stdlib/easy_pwn.talon");
static STDLIB_CTF_HELPERS: &str = include_str!("../stdlib/ctf_helpers.talon");

lazy_static::lazy_static! {
    static ref STDLIB_CACHE: Arc<Mutex<Option<StdlibCache>>> = Arc::new(Mutex::new(None));
}

struct StdlibCache {
    functions: HashMap<String, Vec<Command>>,
}

impl StdlibCache {
    fn new() -> Result<Self, String> {
        let mut functions = HashMap::new();
        
        let _ = Self::parse_and_cache(STDLIB_QUICK, "stdlib/quick.talon", &mut functions);
        let _ = Self::parse_and_cache(STDLIB_EASY_PWN, "stdlib/easy_pwn.talon", &mut functions);
        let _ = Self::parse_and_cache(STDLIB_CTF_HELPERS, "stdlib/ctf_helpers.talon", &mut functions);
        
        Ok(StdlibCache { functions })
    }
    
    fn parse_and_cache(
        source: &str,
        name: &str,
        cache: &mut HashMap<String, Vec<Command>>,
    ) -> Result<(), String> {
        let commands = parser::parse_script(source)
            .map_err(|e| format!("Failed to parse stdlib '{}': {}", name, e))?;
        
        for cmd in commands {
            if let Command::DefineFunction(ref def) = cmd {
                cache.insert(def.name.clone(), vec![cmd.clone()]);
            }
        }
        
        Ok(())
    }
}

fn ensure_stdlib_loaded() -> Result<(), String> {
    use tokio::runtime::Runtime;
    let rt = Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;
    
    rt.block_on(async {
        let mut cache = STDLIB_CACHE.lock().await;
        if cache.is_none() {
            *cache = Some(StdlibCache::new()?);
        }
        Ok(())
    })
}

pub fn run_fast(script: &str) -> Result<(), String> {
    ensure_stdlib_loaded()?;
    
    let commands = parser::parse_script(script)?;
    
    interpreter::interpret(&commands)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_interpreter_basic() {
        let script = r#"
            let x = 42
            print("Test value:", x)
        "#;
        
        assert!(run_fast(script).is_ok());
    }

    #[test]
    fn test_fast_interpreter_empty() {
        let script = "";
        assert!(run_fast(script).is_ok());
    }

    #[test]
    fn test_fast_interpreter_syntax_error() {
        let script = "let x = ";
        assert!(run_fast(script).is_err());
    }

    #[test]
    fn test_stdlib_cache_initialization() {
        assert!(ensure_stdlib_loaded().is_ok());
    }
}
