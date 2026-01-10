use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct OutputUtils {
    pub colors_enabled: bool,
    pub progress_enabled: bool,
}

impl OutputUtils {
    pub fn new(colors: bool, progress: bool) -> Self {
        OutputUtils {
            colors_enabled: colors,
            progress_enabled: progress,
        }
    }
    
    pub fn success(&self, message: &str) {
        if self.colors_enabled {
            println!("{} {}", "[OK]".green().bold(), message);
        } else {
            println!("[OK] {}", message);
        }
    }
    
    pub fn error(&self, message: &str) {
        if self.colors_enabled {
            eprintln!("{} {}", "[ERROR]".red().bold(), message);
        } else {
            eprintln!("[ERROR] {}", message);
        }
    }
    
    pub fn warning(&self, message: &str) {
        if self.colors_enabled {
            println!("{} {}", "[!]".yellow().bold(), message);
        } else {
            println!("[!] {}", message);
        }
    }
    
    pub fn info(&self, message: &str) {
        if self.colors_enabled {
            println!("{} {}", "[i]".blue().bold(), message);
        } else {
            println!("[i] {}", message);
        }
    }
    
    pub fn exploit(&self, message: &str) {
        if self.colors_enabled {
            println!("{} {}", "[X]".red().bold(), message);
        } else {
            println!("[X] {}", message);
        }
    }
    
    pub fn shell(&self, message: &str) {
        if self.colors_enabled {
            println!("{} {}", "[$]".cyan().bold(), message);
        } else {
            println!("[$] {}", message);
        }
    }
    
    pub fn section(&self, title: &str) {
        if self.colors_enabled {
            println!("\n{}", format!("══════ {} ══════", title).bold().purple());
        } else {
            println!("\n══════ {} ══════", title);
        }
    }
    
    pub fn create_progress_bar(&self, total: u64, message: &str) -> Option<ProgressBar> {
        if !self.progress_enabled {
            return None;
        }
        
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .expect("Invalid progress bar template")
                .progress_chars("#>-")
        );
        pb.set_message(message.to_string());
        Some(pb)
    }
    
    pub fn create_spinner(&self, message: &str) -> Option<ProgressBar> {
        if !self.progress_enabled {
            return None;
        }
        
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Invalid spinner template")
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        Some(pb)
    }
    
    pub fn format_bytes(&self, bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.2} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
    
    pub fn highlight_code(&self, code: &str, language: &str) -> String {
        if !self.colors_enabled {
            return code.to_string();
        }
        
        match language {
            "talon" => self.highlight_talon(code),
            "asm" => self.highlight_asm(code),
            _ => code.to_string(),
        }
    }
    
    fn highlight_talon(&self, code: &str) -> String {
        let keywords = ["let", "const", "if", "else", "for", "in", "end", "function", "return", "connect", "send", "recv"];
        let mut result = code.to_string();
        
        for keyword in &keywords {
            result = result.replace(keyword, &format!("{}", keyword.blue().bold()));
        }
        
        result
    }
    
    fn highlight_asm(&self, code: &str) -> String {
        let mut result = String::new();
        for line in code.lines() {
            if line.trim().starts_with(';') {
                result.push_str(&line.bright_black().to_string());
            } else if line.contains(':') {
                result.push_str(&line.yellow().to_string());
            } else {
                result.push_str(line);
            }
            result.push('\n');
        }
        result
    }
}

impl Default for OutputUtils {
    fn default() -> Self {
        OutputUtils::new(true, true)
    }
}
