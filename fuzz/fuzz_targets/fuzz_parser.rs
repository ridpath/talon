#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        if input.len() > 100_000 {
            return;
        }

        if input.chars().filter(|c| c.is_control() && *c != '\n' && *c != '\t').count() > 50 {
            return;
        }

        let _ = talon::parser::parse_script(input);

        if input.len() < 1000 {
            let mut prefixed = String::new();
            prefixed.push_str("let x = ");
            prefixed.push_str(input);
            let _ = talon::parser::parse_script(&prefixed);

            let mut function_wrapped = String::new();
            function_wrapped.push_str("function test()\n");
            function_wrapped.push_str(input);
            function_wrapped.push_str("\nend");
            let _ = talon::parser::parse_script(&function_wrapped);

            let mut if_wrapped = String::new();
            if_wrapped.push_str("if true\n");
            if_wrapped.push_str(input);
            if_wrapped.push_str("\nend");
            let _ = talon::parser::parse_script(&if_wrapped);
        }
    }
});
