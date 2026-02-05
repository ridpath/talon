// Production Error Obfuscation Example
// Run with: talon run --production examples/production_error_obfuscation.talon
// 
// This example demonstrates how production mode protects source code from
// network error leakage by encrypting and signing error messages.
//
// Key Features:
// - Ed25519 signature on all error messages (prevents tampering)
// - AES-256-GCM encryption of error content (confidentiality)
// - Source code redaction (file paths and line numbers removed)
// - Error ID system (E0001 vs full text)
// - Secure logging to encrypted file (~/.talon/error.log)

// Connect to a target (simulating network error scenario)
let target = "192.168.1.100"
let port = 9999

print("Attempting connection to " + target + ":" + str(port))

// This will generate a network error
// In production mode, the error will be:
// 1. Encrypted with AES-256-GCM
// 2. Signed with Ed25519
// 3. Source code paths redacted
// 4. Assigned error ID (e.g., E60001)
let conn = connect(target, port)

// Without production mode:
// [!] TALON ERROR at production_error_obfuscation.talon:27:12: Connection refused by target. Verify the host and port are correct and accessible.
//
// With production mode (--production):
// ERROR E60001 (encrypted)
// [base64 encoded encrypted blob]
// No source file paths exposed
// Verifying key printed for decryption

print("If this line executes, connection succeeded")

// The error log at ~/.talon/error.log contains:
// [2024-02-05 14:30:15] E60001 [encrypted base64 blob]
//
// To decrypt the error:
// 1. Use the verifying key printed on startup
// 2. Call error_context::deobfuscate_error() with the blob
// 3. Verify signature with the exported public key
