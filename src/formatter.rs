pub struct CodeFormatter {
    indent_size: usize,
}

impl CodeFormatter {
    pub fn new() -> Self {
        CodeFormatter {
            indent_size: 2,
        }
    }
    
    pub fn format(&self, code: &str) -> String {
        let lines = code.lines();
        let mut formatted = String::new();
        let mut indent_level = 0;
        let mut in_multiline_string = false;
        
        for line in lines {
            let trimmed = line.trim();
            
            if trimmed.contains("\"\"\"") || trimmed.contains("'''") {
                in_multiline_string = !in_multiline_string;
            }
            
            if in_multiline_string {
                formatted.push_str(line);
                formatted.push('\n');
                continue;
            }
            
            if trimmed.is_empty() {
                formatted.push('\n');
                continue;
            }
            
            if trimmed.starts_with("//") || trimmed.starts_with("#") {
                formatted.push_str(&self.indent(indent_level));
                formatted.push_str(trimmed);
                formatted.push('\n');
                continue;
            }
            
            if trimmed == "end" || trimmed.starts_with("else") || trimmed.starts_with("catch") {
                indent_level = indent_level.saturating_sub(1);
            }
            
            formatted.push_str(&self.indent(indent_level));
            formatted.push_str(trimmed);
            formatted.push('\n');
            
            if self.is_block_start(trimmed) {
                indent_level += 1;
            }
            
            if trimmed == "end" {
                if indent_level > 0 {
                    indent_level -= 1;
                }
            }
        }
        
        formatted
    }
    
    fn indent(&self, level: usize) -> String {
        " ".repeat(level * self.indent_size)
    }
    
    fn is_block_start(&self, line: &str) -> bool {
        let keywords = [
            "def ", "if ", "for ", "while ", "try", "match ", "else",
            "auto_rop ", "heap_exploit ", "kernel_exploit ", "notebook ",
            "taint_analysis ", "symbolic_exec "
        ];
        
        for keyword in &keywords {
            if line.starts_with(keyword) {
                return true;
            }
        }
        
        false
    }
}

pub fn format_file(filename: &str) -> Result<(), String> {
    let code = std::fs::read_to_string(filename)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let formatter = CodeFormatter::new();
    let formatted = formatter.format(&code);
    
    std::fs::write(filename, formatted)
        .map_err(|e| format!("Failed to write formatted file: {}", e))?;
    
    println!("Formatted: {}", filename);
    Ok(())
}
