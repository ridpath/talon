use crate::parser::parse_script;
use crate::ast::Command;
use std::collections::HashSet;

pub struct Linter {
    issues: Vec<LintIssue>,
}

#[derive(Debug, Clone)]
pub struct LintIssue {
    severity: String,
    line: usize,
    message: String,
    suggestion: Option<String>,
}

impl Linter {
    pub fn new() -> Self {
        Linter {
            issues: Vec::new(),
        }
    }
    
    pub fn lint_file(&mut self, filename: &str) -> Result<Vec<LintIssue>, String> {
        let code = std::fs::read_to_string(filename)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        self.lint_code(&code)
    }
    
    pub fn lint_code(&mut self, code: &str) -> Result<Vec<LintIssue>, String> {
        self.issues.clear();
        
        self.check_unused_variables(code);
        self.check_hardcoded_secrets(code);
        self.check_dangerous_functions(code);
        self.check_missing_error_handling(code);
        self.check_unreachable_code(code);
        
        if let Ok(cmds) = parse_script(code) {
            self.check_ast(&cmds);
        }
        
        Ok(self.issues.clone())
    }
    
    fn check_unused_variables(&mut self, code: &str) {
        let mut defined_vars = HashSet::new();
        let mut used_vars = HashSet::new();
        
        for (_line_num, line) in code.lines().enumerate() {
            if line.trim().starts_with("let ") {
                if let Some(var_name) = line.trim().strip_prefix("let ") {
                    if let Some(name) = var_name.split('=').next() {
                        defined_vars.insert(name.trim().to_string());
                    }
                }
            }
            
            for var in &defined_vars {
                if line.contains(var) && !line.trim().starts_with("let ") {
                    used_vars.insert(var.clone());
                }
            }
        }
        
        for var in defined_vars.difference(&used_vars) {
            self.issues.push(LintIssue {
                severity: "warning".to_string(),
                line: 0,
                message: format!("Unused variable: {}", var),
                suggestion: Some(format!("Remove unused variable '{}' or use it in your code", var)),
            });
        }
    }
    
    fn check_hardcoded_secrets(&mut self, code: &str) {
        let patterns = [
            ("password", "Hardcoded password detected"),
            ("api_key", "Hardcoded API key detected"),
            ("secret", "Hardcoded secret detected"),
            ("token", "Hardcoded token detected"),
        ];
        
        for (line_num, line) in code.lines().enumerate() {
            for (pattern, msg) in &patterns {
                if line.to_lowercase().contains(pattern) && line.contains("=") {
                    self.issues.push(LintIssue {
                        severity: "error".to_string(),
                        line: line_num + 1,
                        message: msg.to_string(),
                        suggestion: Some("Use environment variables or configuration files for sensitive data".to_string()),
                    });
                }
            }
        }
    }
    
    fn check_dangerous_functions(&mut self, code: &str) {
        let dangerous = [
            ("system(", "Direct system() call can be dangerous"),
            ("exec(", "Direct exec() call can be dangerous"),
            ("eval(", "eval() can execute arbitrary code"),
        ];
        
        for (line_num, line) in code.lines().enumerate() {
            for (pattern, msg) in &dangerous {
                if line.contains(pattern) {
                    self.issues.push(LintIssue {
                        severity: "warning".to_string(),
                        line: line_num + 1,
                        message: msg.to_string(),
                        suggestion: Some("Ensure input is properly validated and sanitized".to_string()),
                    });
                }
            }
        }
    }
    
    fn check_missing_error_handling(&mut self, code: &str) {
        let mut in_try_block = false;
        let mut try_depth = 0;
        
        for (line_num, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("try") {
                in_try_block = true;
                try_depth += 1;
            } else if trimmed == "end" && in_try_block {
                try_depth -= 1;
                if try_depth == 0 {
                    in_try_block = false;
                }
            }
            
            let risky_ops = ["connect", "send", "recv", "http_", "malloc", "free"];
            
            for op in &risky_ops {
                if trimmed.contains(op) && !in_try_block && !trimmed.starts_with("//") {
                    self.issues.push(LintIssue {
                        severity: "info".to_string(),
                        line: line_num + 1,
                        message: format!("Operation '{}' without error handling", op),
                        suggestion: Some("Consider wrapping in try/catch block".to_string()),
                    });
                    break;
                }
            }
        }
    }
    
    fn check_unreachable_code(&mut self, code: &str) {
        let mut after_return = false;
        
        for (line_num, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("return") {
                after_return = true;
                continue;
            }
            
            if trimmed == "end" {
                after_return = false;
            }
            
            if after_return && !trimmed.is_empty() && !trimmed.starts_with("//") {
                self.issues.push(LintIssue {
                    severity: "error".to_string(),
                    line: line_num + 1,
                    message: "Unreachable code after return statement".to_string(),
                    suggestion: Some("Remove code after return or restructure logic".to_string()),
                });
            }
        }
    }
    
    fn check_ast(&mut self, _commands: &[Command]) {
        // Future: AST-level checks
    }
    
    pub fn print_issues(&self) {
        if self.issues.is_empty() {
            println!("No issues found!");
            return;
        }
        
        println!("\nLint Issues:\n{}", "=".repeat(80));
        
        for issue in &self.issues {
            let severity_icon = match issue.severity.as_str() {
                "error" => "[ERROR]",
                "warning" => "⚠",
                "info" => "ℹ",
                _ => "·",
            };
            
            println!("{} [{}] Line {}: {}", 
                severity_icon,
                issue.severity.to_uppercase(),
                if issue.line > 0 { issue.line.to_string() } else { "?".to_string() },
                issue.message
            );
            
            if let Some(suggestion) = &issue.suggestion {
                println!("   Suggestion: {}", suggestion);
            }
            println!();
        }
        
        let errors = self.issues.iter().filter(|i| i.severity == "error").count();
        let warnings = self.issues.iter().filter(|i| i.severity == "warning").count();
        let infos = self.issues.iter().filter(|i| i.severity == "info").count();
        
        println!("{}", "=".repeat(80));
        println!("Summary: {} errors, {} warnings, {} infos", errors, warnings, infos);
    }
}

pub fn lint_file(filename: &str) -> Result<(), String> {
    let mut linter = Linter::new();
    linter.lint_file(filename)?;
    linter.print_issues();
    Ok(())
}
