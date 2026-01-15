#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        if input.len() > 10_000 {
            return;
        }
        
        if let Ok(commands) = talon::parser::parse_script(input) {
            for cmd in &commands {
                let _ = format!("{:?}", cmd);
                
                let serialized = serde_json::to_string(cmd);
                if let Ok(json) = serialized {
                    let _ = serde_json::from_str::<talon::ast::Command>(&json);
                }
            }
            
            let _ = talon::ast::optimize_ast(commands.clone());
            let _ = talon::ast::validate_types(&commands);
        }
    }
});
