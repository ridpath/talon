use std::collections::HashMap;
use std::fmt;
use std::panic;
use std::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn new(file: String, line: usize, column: usize) -> Self {
        SourceLocation { file, line, column }
    }

    pub fn unknown() -> Self {
        SourceLocation {
            file: "<unknown>".to_string(),
            line: 0,
            column: 0,
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{}", self.file)
        } else {
            write!(f, "{}:{}:{}", self.file, self.line, self.column)
        }
    }
}

#[derive(Debug, Clone)]
pub struct DslError {
    pub message: String,
    pub location: SourceLocation,
    pub error_type: ErrorType,
    pub context: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    Syntax,
    Runtime,
    Type,
    Connection,
    FileSystem,
    Network,
    Binary,
    Exploitation,
    Security,
}

impl DslError {
    pub fn new(message: String, location: SourceLocation, error_type: ErrorType) -> Self {
        DslError {
            message,
            location,
            error_type,
            context: Vec::new(),
        }
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context.push(context);
        self
    }

    pub fn runtime(message: String, location: SourceLocation) -> Self {
        DslError::new(message, location, ErrorType::Runtime)
    }

    pub fn connection(message: String, location: SourceLocation) -> Self {
        DslError::new(message, location, ErrorType::Connection)
    }

    pub fn type_error(message: String, location: SourceLocation) -> Self {
        DslError::new(message, location, ErrorType::Type)
    }

    pub fn network(message: String, location: SourceLocation) -> Self {
        DslError::new(message, location, ErrorType::Network)
    }

    pub fn binary(message: String, location: SourceLocation) -> Self {
        DslError::new(message, location, ErrorType::Binary)
    }

    pub fn exploitation(message: String, location: SourceLocation) -> Self {
        DslError::new(message, location, ErrorType::Exploitation)
    }
}

impl fmt::Display for DslError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[!] TALON ERROR at {}: {}", self.location, self.message)?;
        for ctx in &self.context {
            write!(f, "\n    Context: {}", ctx)?;
        }
        Ok(())
    }
}

impl std::error::Error for DslError {}

lazy_static::lazy_static! {
    static ref ERROR_TRANSLATION_MAP: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
    static ref CURRENT_SOURCE_LOCATION: RwLock<SourceLocation> = RwLock::new(SourceLocation::unknown());
}

pub fn install_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        let location = CURRENT_SOURCE_LOCATION.read().unwrap().clone();
        
        let panic_msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let translated = translate_panic(&panic_msg);
        
        eprintln!("\n[!] TALON ERROR at {}: {}", location, translated);
        
        if location.line > 0 {
            eprintln!("\nError occurred in your TALON script at line {}", location.line);
        }
        
        std::process::exit(1);
    }));
}

pub fn translate_panic(rust_error: &str) -> String {
    let map = ERROR_TRANSLATION_MAP.read().unwrap();
    
    if let Some(translated) = map.get(rust_error) {
        return translated.clone();
    }

    if rust_error.contains("index out of bounds") {
        return "List or array access out of bounds. Check your index value.".to_string();
    }
    
    if rust_error.contains("Cannot borrow") || rust_error.contains("borrow") {
        return "Resource is currently in use by another operation. Ensure connections and resources are not being accessed concurrently.".to_string();
    }
    
    if rust_error.contains("unwrap") || rust_error.contains("None") {
        return "Attempted to use a value that doesn't exist. Check if the variable or resource is properly initialized.".to_string();
    }
    
    if rust_error.contains("parse") {
        return "Failed to parse value. Check the format of your input data.".to_string();
    }
    
    if rust_error.contains("not found") {
        return "Resource not found. Verify the file path, connection, or identifier exists.".to_string();
    }
    
    if rust_error.contains("permission denied") {
        return "Permission denied. Check file permissions or run with appropriate privileges.".to_string();
    }
    
    if rust_error.contains("connection refused") {
        return "Connection refused by target. Verify the host and port are correct and accessible.".to_string();
    }
    
    if rust_error.contains("timeout") {
        return "Operation timed out. The target may be unresponsive or network connectivity is slow.".to_string();
    }

    rust_error.to_string()
}

pub fn register_error_translation(rust_pattern: String, dsl_message: String) {
    let mut map = ERROR_TRANSLATION_MAP.write().unwrap();
    map.insert(rust_pattern, dsl_message);
}

pub fn set_source_location(location: SourceLocation) {
    let mut current = CURRENT_SOURCE_LOCATION.write().unwrap();
    *current = location;
}

pub fn get_source_location() -> SourceLocation {
    CURRENT_SOURCE_LOCATION.read().unwrap().clone()
}

pub fn translate_error(rust_error: &str) -> DslError {
    let location = get_source_location();
    let message = translate_panic(rust_error);
    
    if rust_error.contains("index") {
        DslError::runtime(message, location)
    } else if rust_error.contains("connection") || rust_error.contains("Connection") {
        DslError::connection(message, location)
    } else if rust_error.contains("network") || rust_error.contains("Network") {
        DslError::network(message, location)
    } else if rust_error.contains("type") || rust_error.contains("Type") {
        DslError::type_error(message, location)
    } else if rust_error.contains("binary") || rust_error.contains("Binary") {
        DslError::binary(message, location)
    } else if rust_error.contains("exploit") || rust_error.contains("Exploit") {
        DslError::exploitation(message, location)
    } else {
        DslError::runtime(message, location)
    }
}

pub trait ResultExt<T> {
    fn map_err_context(self, location: SourceLocation) -> Result<T, DslError>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn map_err_context(self, location: SourceLocation) -> Result<T, DslError> {
        self.map_err(|e| {
            let error_str = e.to_string();
            let message = translate_panic(&error_str);
            
            if error_str.contains("connection") || error_str.contains("Connection") {
                DslError::connection(message, location)
            } else if error_str.contains("network") || error_str.contains("Network") {
                DslError::network(message, location)
            } else if error_str.contains("type") || error_str.contains("Type") {
                DslError::type_error(message, location)
            } else {
                DslError::runtime(message, location)
            }
        })
    }
}

pub fn init_error_system() {
    install_panic_hook();
    
    register_error_translation(
        "index out of bounds".to_string(),
        "Array or list index is out of valid range".to_string(),
    );
    register_error_translation(
        "Cannot borrow".to_string(),
        "Connection or resource is currently in use by another operation".to_string(),
    );
    register_error_translation(
        "called `Option::unwrap()`".to_string(),
        "Attempted to use a null or uninitialized value".to_string(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location_display() {
        let loc = SourceLocation::new("test.talon".to_string(), 42, 10);
        assert_eq!(format!("{}", loc), "test.talon:42:10");
    }

    #[test]
    fn test_source_location_unknown() {
        let loc = SourceLocation::unknown();
        assert_eq!(format!("{}", loc), "<unknown>");
    }

    #[test]
    fn test_translate_index_error() {
        let translated = translate_panic("index out of bounds: the len is 5 but the index is 10");
        assert!(translated.contains("index"));
        assert!(translated.contains("bounds"));
    }

    #[test]
    fn test_translate_borrow_error() {
        let translated = translate_panic("Cannot borrow as mutable");
        assert!(translated.contains("in use"));
    }

    #[test]
    fn test_translate_connection_error() {
        let translated = translate_panic("connection refused");
        assert!(translated.contains("Connection refused"));
        assert!(translated.contains("target"));
    }

    #[test]
    fn test_dsl_error_creation() {
        let loc = SourceLocation::new("test.talon".to_string(), 10, 5);
        let error = DslError::runtime("Test error".to_string(), loc);
        assert_eq!(error.error_type, ErrorType::Runtime);
        assert!(error.message.contains("Test error"));
    }

    #[test]
    fn test_dsl_error_with_context() {
        let loc = SourceLocation::new("test.talon".to_string(), 10, 5);
        let error = DslError::runtime("Test error".to_string(), loc)
            .with_context("Additional context".to_string());
        assert_eq!(error.context.len(), 1);
        assert_eq!(error.context[0], "Additional context");
    }

    #[test]
    fn test_error_translation_registration() {
        register_error_translation(
            "custom error".to_string(),
            "Custom DSL error message".to_string(),
        );
        let translated = translate_panic("custom error");
        assert_eq!(translated, "Custom DSL error message");
    }

    #[test]
    fn test_source_location_tracking() {
        let loc1 = SourceLocation::new("file1.talon".to_string(), 100, 20);
        set_source_location(loc1.clone());
        let retrieved = get_source_location();
        assert_eq!(retrieved.file, "file1.talon");
        assert_eq!(retrieved.line, 100);
        assert_eq!(retrieved.column, 20);
    }
}
