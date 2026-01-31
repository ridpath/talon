#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        if input.len() > 50_000 {
            return;
        }

        if let Ok(commands) = talon::parser::parse_script(input) {
            use talon::interpreter::Interpreter;

            let mut interp = Interpreter::new();
            interp.set_timeout(std::time::Duration::from_millis(100));

            let _ = interp.execute(commands);

            if input.len() < 500 {
                let mut safe_interp = Interpreter::new();
                safe_interp.enable_sandbox_mode();
                let _ = safe_interp.execute(commands.clone());
            }
        }
    }
});
