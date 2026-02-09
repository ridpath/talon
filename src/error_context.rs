// Suppress deprecation warnings for aes_gcm's GenericArray (transitive dependency)
// This will be resolved when aes_gcm upgrades to generic-array 1.x
#![allow(deprecated)]

use std::collections::HashMap;
use std::fmt;
use std::panic;
use std::sync::{Arc, RwLock};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey, Signature, Verifier, SECRET_KEY_LENGTH};
use rand::RngCore;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DslError {
    pub message: String,
    pub location: SourceLocation,
    pub error_type: ErrorType,
    pub context: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorId(pub String);

impl ErrorId {
    pub fn new(error_type: &ErrorType, index: usize) -> Self {
        let prefix = match error_type {
            ErrorType::Syntax => "E1",
            ErrorType::Runtime => "E2",
            ErrorType::Type => "E3",
            ErrorType::Connection => "E4",
            ErrorType::FileSystem => "E5",
            ErrorType::Network => "E6",
            ErrorType::Binary => "E7",
            ErrorType::Exploitation => "E8",
            ErrorType::Security => "E9",
        };
        ErrorId(format!("{}{:04}", prefix, index))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ErrorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObfuscatedError {
    pub error_id: ErrorId,
    pub timestamp: i64,
    pub encrypted_message: Vec<u8>,
    pub signature: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl ObfuscatedError {
    pub fn to_base64(&self) -> String {
        let serialized = bincode::serialize(self).expect("Failed to serialize obfuscated error");
        base64::encode(&serialized)
    }

    pub fn from_base64(encoded: &str) -> Result<Self, String> {
        let decoded = base64::decode(encoded)
            .map_err(|e| format!("Failed to decode base64: {}", e))?;
        bincode::deserialize(&decoded)
            .map_err(|e| format!("Failed to deserialize error: {}", e))
    }
}

pub struct ProductionErrorContext {
    enabled: bool,
    signing_key: Arc<SigningKey>,
    verifying_key: VerifyingKey,
    encryption_key: Arc<Aes256Gcm>,
    log_file: Arc<RwLock<Option<File>>>,
    error_counter: Arc<RwLock<HashMap<ErrorType, usize>>>,
}

impl Default for ProductionErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ProductionErrorContext {
    pub fn new() -> Self {
        let mut secret_key_bytes = [0u8; SECRET_KEY_LENGTH];
        OsRng.fill_bytes(&mut secret_key_bytes);
        let signing_key = SigningKey::from_bytes(&secret_key_bytes);
        let verifying_key = signing_key.verifying_key();
        
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        let encryption_key = Aes256Gcm::new(&key_bytes.into());

        ProductionErrorContext {
            enabled: false,
            signing_key: Arc::new(signing_key),
            verifying_key,
            encryption_key: Arc::new(encryption_key),
            log_file: Arc::new(RwLock::new(None)),
            error_counter: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn enable(&mut self, log_path: Option<PathBuf>) {
        self.enabled = true;
        
        if let Some(path) = log_path {
            if let Ok(file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                *self.log_file.write().unwrap() = Some(file);
            }
        }
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn obfuscate_error(&self, error: &DslError) -> ObfuscatedError {
        let mut counter = self.error_counter.write().unwrap();
        let index = counter.entry(error.error_type.clone()).or_insert(0);
        *index += 1;
        let error_id = ErrorId::new(&error.error_type, *index);

        let redacted_error = self.redact_source_code(error);

        let serialized_error = bincode::serialize(&redacted_error)
            .expect("Failed to serialize error");

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::clone_from_slice(&nonce_bytes);

        let encrypted_message = self.encryption_key
            .encrypt(&nonce, serialized_error.as_ref())
            .expect("Encryption failed");

        let mut hasher = Sha256::new();
        hasher.update(&encrypted_message);
        hasher.update(nonce_bytes);
        hasher.update(error_id.as_str().as_bytes());
        let message_hash = hasher.finalize();

        let signature = self.signing_key.sign(&message_hash);

        let timestamp = chrono::Utc::now().timestamp();

        ObfuscatedError {
            error_id,
            timestamp,
            encrypted_message,
            signature: signature.to_bytes().to_vec(),
            nonce: nonce_bytes.to_vec(),
        }
    }

    pub fn deobfuscate_error(&self, obfuscated: &ObfuscatedError) -> Result<DslError, String> {
        let mut hasher = Sha256::new();
        hasher.update(&obfuscated.encrypted_message);
        hasher.update(&obfuscated.nonce);
        hasher.update(obfuscated.error_id.as_str().as_bytes());
        let message_hash = hasher.finalize();

        let signature = Signature::from_bytes(
            obfuscated.signature.as_slice().try_into()
                .map_err(|_| "Invalid signature format")?
        );

        self.verifying_key
            .verify(&message_hash, &signature)
            .map_err(|e| format!("Signature verification failed: {}", e))?;

        let nonce = Nonce::clone_from_slice(&obfuscated.nonce);

        let decrypted = self.encryption_key
            .decrypt(&nonce, obfuscated.encrypted_message.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;

        bincode::deserialize(&decrypted)
            .map_err(|e| format!("Failed to deserialize error: {}", e))
    }

    fn redact_source_code(&self, error: &DslError) -> DslError {
        let mut redacted = error.clone();
        
        redacted.location = SourceLocation {
            file: "[REDACTED]".to_string(),
            line: 0,
            column: 0,
        };

        redacted.context = redacted.context.iter()
            .map(|ctx| {
                if ctx.contains(".talon") || ctx.contains("src/") {
                    "[REDACTED]".to_string()
                } else {
                    ctx.clone()
                }
            })
            .collect();

        redacted
    }

    pub fn log_error(&self, error: &DslError) {
        if !self.enabled {
            return;
        }

        let obfuscated = self.obfuscate_error(error);
        let encoded = obfuscated.to_base64();

        if let Ok(mut log_file_guard) = self.log_file.write() {
            if let Some(ref mut log_file) = *log_file_guard {
                let log_entry = format!(
                    "[{}] {} {}\n",
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                    obfuscated.error_id,
                    encoded
                );
                let _ = log_file.write_all(log_entry.as_bytes());
                let _ = log_file.flush();
            }
        }
    }

    pub fn format_for_network(&self, error: &DslError) -> Vec<u8> {
        if !self.enabled {
            return error.to_string().into_bytes();
        }

        let obfuscated = self.obfuscate_error(error);
        let encoded = obfuscated.to_base64();
        
        let network_message = format!(
            "ERROR {} (encrypted)\n{}",
            obfuscated.error_id,
            encoded
        );
        
        network_message.into_bytes()
    }

    pub fn export_verifying_key(&self) -> Vec<u8> {
        self.verifying_key.to_bytes().to_vec()
    }

    pub fn export_verifying_key_base64(&self) -> String {
        base64::encode(self.verifying_key.to_bytes())
    }
}

lazy_static::lazy_static! {
    static ref ERROR_TRANSLATION_MAP: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
    static ref CURRENT_SOURCE_LOCATION: RwLock<SourceLocation> = RwLock::new(SourceLocation::unknown());
    static ref PRODUCTION_ERROR_CONTEXT: RwLock<ProductionErrorContext> = RwLock::new(ProductionErrorContext::new());
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

pub fn enable_production_mode(log_path: Option<PathBuf>) {
    let mut ctx = PRODUCTION_ERROR_CONTEXT.write().unwrap();
    ctx.enable(log_path);
}

pub fn disable_production_mode() {
    let mut ctx = PRODUCTION_ERROR_CONTEXT.write().unwrap();
    ctx.disable();
}

pub fn is_production_mode() -> bool {
    let ctx = PRODUCTION_ERROR_CONTEXT.read().unwrap();
    ctx.is_enabled()
}

pub fn obfuscate_error(error: &DslError) -> ObfuscatedError {
    let ctx = PRODUCTION_ERROR_CONTEXT.read().unwrap();
    ctx.obfuscate_error(error)
}

pub fn deobfuscate_error(obfuscated: &ObfuscatedError) -> Result<DslError, String> {
    let ctx = PRODUCTION_ERROR_CONTEXT.read().unwrap();
    ctx.deobfuscate_error(obfuscated)
}

pub fn log_error_secure(error: &DslError) {
    let ctx = PRODUCTION_ERROR_CONTEXT.read().unwrap();
    ctx.log_error(error);
}

pub fn format_error_for_network(error: &DslError) -> Vec<u8> {
    let ctx = PRODUCTION_ERROR_CONTEXT.read().unwrap();
    ctx.format_for_network(error)
}

pub fn export_verifying_key() -> Vec<u8> {
    let ctx = PRODUCTION_ERROR_CONTEXT.read().unwrap();
    ctx.export_verifying_key()
}

pub fn export_verifying_key_base64() -> String {
    let ctx = PRODUCTION_ERROR_CONTEXT.read().unwrap();
    ctx.export_verifying_key_base64()
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

    #[test]
    fn test_error_id_generation() {
        let runtime_id = ErrorId::new(&ErrorType::Runtime, 1);
        assert_eq!(runtime_id.as_str(), "E20001");

        let network_id = ErrorId::new(&ErrorType::Network, 42);
        assert_eq!(network_id.as_str(), "E60042");

        let syntax_id = ErrorId::new(&ErrorType::Syntax, 999);
        assert_eq!(syntax_id.as_str(), "E10999");
    }

    #[test]
    fn test_production_mode_toggle() {
        disable_production_mode();
        assert!(!is_production_mode());

        enable_production_mode(None);
        assert!(is_production_mode());

        disable_production_mode();
        assert!(!is_production_mode());
    }

    #[test]
    fn test_error_obfuscation_and_deobfuscation() {
        let loc = SourceLocation::new("exploit.talon".to_string(), 42, 10);
        let error = DslError::runtime("Test error message".to_string(), loc)
            .with_context("Additional context from source".to_string());

        let obfuscated = obfuscate_error(&error);

        assert!(obfuscated.error_id.as_str().starts_with("E2"));
        assert!(!obfuscated.encrypted_message.is_empty());
        assert_eq!(obfuscated.signature.len(), 64);
        assert_eq!(obfuscated.nonce.len(), 12);

        let deobfuscated = deobfuscate_error(&obfuscated)
            .expect("Failed to deobfuscate error");

        assert_eq!(deobfuscated.location.file, "[REDACTED]");
        assert_eq!(deobfuscated.location.line, 0);
        assert!(deobfuscated.message.contains("Test error message"));
    }

    #[test]
    fn test_obfuscated_error_base64_encoding() {
        let loc = SourceLocation::new("test.talon".to_string(), 10, 5);
        let error = DslError::network("Connection failed".to_string(), loc);

        let obfuscated = obfuscate_error(&error);
        let encoded = obfuscated.to_base64();

        assert!(!encoded.is_empty());
        assert!(encoded.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='));

        let decoded = ObfuscatedError::from_base64(&encoded)
            .expect("Failed to decode from base64");

        assert_eq!(decoded.error_id.as_str(), obfuscated.error_id.as_str());
        assert_eq!(decoded.encrypted_message, obfuscated.encrypted_message);
        assert_eq!(decoded.signature, obfuscated.signature);
    }

    #[test]
    fn test_source_code_redaction() {
        let loc = SourceLocation::new("/path/to/exploit.talon".to_string(), 100, 15);
        let error = DslError::runtime("Runtime error".to_string(), loc)
            .with_context("Error in exploit.talon at line 100".to_string())
            .with_context("File: src/interpreter.rs".to_string())
            .with_context("Safe context without paths".to_string());

        let obfuscated = obfuscate_error(&error);
        let deobfuscated = deobfuscate_error(&obfuscated)
            .expect("Failed to deobfuscate");

        assert_eq!(deobfuscated.location.file, "[REDACTED]");
        assert_eq!(deobfuscated.location.line, 0);
        assert_eq!(deobfuscated.location.column, 0);

        assert_eq!(deobfuscated.context[0], "[REDACTED]");
        assert_eq!(deobfuscated.context[1], "[REDACTED]");
        assert_eq!(deobfuscated.context[2], "Safe context without paths");
    }

    #[test]
    fn test_format_error_for_network() {
        enable_production_mode(None);

        let loc = SourceLocation::new("network_test.talon".to_string(), 50, 8);
        let error = DslError::network("Connection timeout".to_string(), loc);

        let network_bytes = format_error_for_network(&error);
        let network_string = String::from_utf8_lossy(&network_bytes);

        assert!(network_string.contains("ERROR E6"));
        assert!(network_string.contains("(encrypted)"));
        assert!(!network_string.contains("network_test.talon"));

        disable_production_mode();
    }

    #[test]
    fn test_format_error_for_network_disabled() {
        disable_production_mode();

        let loc = SourceLocation::new("test.talon".to_string(), 10, 5);
        let error = DslError::runtime("Test error".to_string(), loc);

        let network_bytes = format_error_for_network(&error);
        let network_string = String::from_utf8_lossy(&network_bytes);

        assert!(network_string.contains("TALON ERROR"));
        assert!(network_string.contains("test.talon"));
        assert!(!network_string.contains("encrypted"));
    }

    #[test]
    fn test_verifying_key_export() {
        let key_bytes = export_verifying_key();
        assert_eq!(key_bytes.len(), 32);

        let key_base64 = export_verifying_key_base64();
        assert!(!key_base64.is_empty());
        assert!(key_base64.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='));

        let decoded = base64::decode(&key_base64)
            .expect("Failed to decode base64");
        assert_eq!(decoded, key_bytes);
    }

    #[test]
    fn test_signature_verification_failure() {
        let loc = SourceLocation::new("test.talon".to_string(), 1, 1);
        let error = DslError::runtime("Test".to_string(), loc);

        let mut obfuscated = obfuscate_error(&error);
        
        obfuscated.signature[0] ^= 0xFF;

        let result = deobfuscate_error(&obfuscated);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Signature verification failed"));
    }

    #[test]
    fn test_error_counter_increments() {
        let loc = SourceLocation::new("test.talon".to_string(), 1, 1);
        
        let error1 = DslError::runtime("First error".to_string(), loc.clone());
        let obf1 = obfuscate_error(&error1);
        
        let error2 = DslError::runtime("Second error".to_string(), loc.clone());
        let obf2 = obfuscate_error(&error2);

        assert!(obf1.error_id.as_str() < obf2.error_id.as_str());
    }

    #[test]
    fn test_secure_error_logging() {
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join("talon_error_test.log");
        
        if log_path.exists() {
            std::fs::remove_file(&log_path).ok();
        }

        enable_production_mode(Some(log_path.clone()));

        let loc = SourceLocation::new("test.talon".to_string(), 42, 10);
        let error = DslError::runtime("Test logging".to_string(), loc);

        log_error_secure(&error);

        disable_production_mode();

        if log_path.exists() {
            let contents = std::fs::read_to_string(&log_path)
                .expect("Failed to read log file");
            
            assert!(contents.contains("E2"));
            assert!(!contents.contains("test.talon"));
            
            std::fs::remove_file(&log_path).ok();
        }
    }
}
