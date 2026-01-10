use colored::*;

pub fn process_escape_sequences(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(&next_ch) = chars.peek() {
                chars.next();
                match next_ch {
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    '\'' => result.push('\''),
                    'x' => {
                        let hex1 = chars.next();
                        let hex2 = chars.next();
                        if let (Some(h1), Some(h2)) = (hex1, hex2) {
                            let hex_str = format!("{}{}", h1, h2);
                            if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                                result.push(byte as char);
                            }
                        }
                    }
                    'u' => {
                        if chars.next() == Some('{') {
                            let mut unicode_hex = String::new();
                            while let Some(ch) = chars.next() {
                                if ch == '}' {
                                    break;
                                }
                                unicode_hex.push(ch);
                            }
                            if let Ok(code_point) = u32::from_str_radix(&unicode_hex, 16) {
                                if let Some(unicode_char) = char::from_u32(code_point) {
                                    result.push(unicode_char);
                                }
                            }
                        }
                    }
                    _ => {
                        result.push('\\');
                        result.push(next_ch);
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }
    
    result
}

pub fn parse_hex_number(input: &str) -> Option<i64> {
    if input.starts_with("0x") || input.starts_with("0X") {
        i64::from_str_radix(&input[2..], 16).ok()
    } else {
        None
    }
}

pub fn parse_number(input: &str) -> Option<i64> {
    if let Some(hex) = parse_hex_number(input) {
        Some(hex)
    } else {
        input.parse::<i64>().ok()
    }
}

pub fn format_parse_error(input: &str, error: &str, line: usize, column: usize) -> String {
    let lines: Vec<&str> = input.lines().collect();
    
    if line == 0 || line > lines.len() {
        return format!("{} {}", "Parse Error:".red().bold(), error);
    }
    
    let error_line = lines[line - 1];
    let prev_line = if line > 1 { Some(lines[line - 2]) } else { None };
    let next_line = if line < lines.len() { Some(lines[line]) } else { None };
    
    let mut output = String::new();
    
    output.push_str(&format!("\n{} {}\n", "Parse Error:".red().bold(), error));
    output.push_str(&format!("{} {}:{}:{}\n\n", "  -->".blue(), "<script>", line, column));
    
    if let Some(prev) = prev_line {
        output.push_str(&format!("{:>4} {} {}\n", (line - 1).to_string().bright_black(), "|".bright_black(), prev.bright_black()));
    }
    
    output.push_str(&format!("{:>4} {} {}\n", line.to_string().blue().bold(), "|".blue(), error_line));
    
    let indent = " ".repeat(6 + column.saturating_sub(1));
    output.push_str(&format!("{}{} {}\n", indent, "^".red().bold(), "error here".red()));
    
    if let Some(next) = next_line {
        output.push_str(&format!("{:>4} {} {}\n", (line + 1).to_string().bright_black(), "|".bright_black(), next.bright_black()));
    }
    
    output.push_str("\n");
    
    if error.contains("expected") {
        output.push_str(&format!("{} Check syntax and ensure proper use of keywords\n", "  Help:".cyan().bold()));
    }
    
    output
}

pub fn suggest_fix(error_msg: &str) -> Option<String> {
    if error_msg.contains("unclosed") {
        Some("Add closing delimiter (bracket, quote, or brace)".to_string())
    } else if error_msg.contains("unexpected token") {
        Some("Check for missing semicolons or mismatched operators".to_string())
    } else if error_msg.contains("undefined") {
        Some("Declare variable with 'let' before using it".to_string())
    } else {
        None
    }
}
