// OpSec panic handler - removes file paths and debug info from panic messages

use std::panic;

/// Install custom panic handler that sanitizes panic messages
/// Removes file paths, line numbers, and other debug information
pub fn install_sanitized_panic_handler() {
    panic::set_hook(Box::new(|info| {
        #[cfg(debug_assertions)]
        {
            // In debug mode, show full panic info
            eprintln!("Thread panicked: {}", info);
        }
        #[cfg(not(debug_assertions))]
        {
            // In release mode, show sanitized message only
            let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Operation failed".to_string()
            };
            let sanitized = sanitize_message(&msg);
            eprintln!("Fatal error: {}", sanitized);
        }
    }));
}

/// Sanitize message by removing file paths and debug info
fn sanitize_message(msg: &str) -> String {
    let mut sanitized = msg.to_string();
    
    // Remove common path patterns
    // Windows paths: C:\path\to\file.rs
    let re_win = regex::Regex::new(r"[A-Za-z]:\\[^\s:]+\.rs").unwrap();
    sanitized = re_win.replace_all(&sanitized, "[source]").to_string();
    
    // Unix paths: /path/to/file.rs
    let re_unix = regex::Regex::new(r"/[^\s:]+\.rs").unwrap();
    sanitized = re_unix.replace_all(&sanitized, "[source]").to_string();
    
    // Line/column numbers: :123:45
    let re_lines = regex::Regex::new(r":\d+:\d+").unwrap();
    sanitized = re_lines.replace_all(&sanitized, "").to_string();
    
    // Remove src/ references
    sanitized = sanitized.replace("src/", "");
    sanitized = sanitized.replace("src\\", "");
    
    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_windows_path() {
        let msg = "Error at C:\\Users\\test\\project\\src\\main.rs:42:10";
        let sanitized = sanitize_message(msg);
        assert!(!sanitized.contains("C:\\"));
        assert!(!sanitized.contains("main.rs"));
        assert!(!sanitized.contains(":42:10"));
    }

    #[test]
    fn test_sanitize_unix_path() {
        let msg = "Error at /home/user/project/src/lib.rs:123:5";
        let sanitized = sanitize_message(msg);
        assert!(!sanitized.contains("/home"));
        assert!(!sanitized.contains("lib.rs"));
        assert!(!sanitized.contains(":123:5"));
    }

    #[test]
    fn test_sanitize_src_references() {
        let msg = "Error in src/interpreter.rs";
        let sanitized = sanitize_message(msg);
        assert!(!sanitized.contains("src/"));
        assert!(!sanitized.contains("src\\"));
    }

    #[test]
    fn test_sanitize_preserves_useful_info() {
        let msg = "Buffer overflow detected";
        let sanitized = sanitize_message(msg);
        assert_eq!(sanitized, "Buffer overflow detected");
    }
}
