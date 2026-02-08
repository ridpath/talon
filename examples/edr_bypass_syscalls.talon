# EDR Bypass using Indirect Syscalls
#
# This example demonstrates how to use indirect syscalls on Windows
# to bypass EDR (Endpoint Detection and Response) user-mode hooks.
#
# Features:
# - Dynamic syscall number resolution from ntdll.dll
# - Direct syscall invocation (bypasses hooked API functions)
# - Hook detection (IAT and inline hooks)
# - Obfuscated syscall stubs with junk instructions
#
# Use Cases:
# - Memory allocation without triggering VirtualAlloc hooks
# - Thread creation without CreateThread detection
# - Process operations bypassing OpenProcess hooks
# - File operations bypassing NtCreateFile hooks
#
# Requirements:
# - Windows OS (x64)
# - Administrator privileges (for some operations)
# - Run on system with or without EDR for comparison

# Initialize syscall resolver
print("[+] Initializing indirect syscall resolver")

# Example 1: Memory Allocation via NtAllocateVirtualMemory
# Standard API: VirtualAlloc (often hooked by EDR)
# Syscall: NtAllocateVirtualMemory (bypasses hooks)
print("\n[*] Example 1: Memory Allocation")
print("Standard approach: VirtualAlloc (HOOKED by EDR)")
print("Evasion approach: NtAllocateVirtualMemory syscall (BYPASS)")

# Example 2: Thread Creation via NtCreateThreadEx
# Standard API: CreateThread (monitored by EDR)
# Syscall: NtCreateThreadEx (undetected)
print("\n[*] Example 2: Thread Creation")
print("Standard approach: CreateThread (DETECTED)")
print("Evasion approach: NtCreateThreadEx syscall (UNDETECTED)")

# Example 3: Process Injection via NtWriteVirtualMemory
# Standard API: WriteProcessMemory (blocked by EDR)
# Syscall: NtWriteVirtualMemory (allowed)
print("\n[*] Example 3: Process Memory Write")
print("Standard approach: WriteProcessMemory (BLOCKED)")
print("Evasion approach: NtWriteVirtualMemory syscall (ALLOWED)")

# Example 4: File Operations via NtCreateFile
# Standard API: CreateFile (logged by EDR)
# Syscall: NtCreateFile (not logged)
print("\n[*] Example 4: File Operations")
print("Standard approach: CreateFile (LOGGED)")
print("Evasion approach: NtCreateFile syscall (STEALTH)")

# Example 5: Hook Detection
# Detect if EDR has hooked common functions
print("\n[*] Example 5: Hook Detection")
print("Checking for inline hooks in ntdll.dll functions")
print("Expected: Hooks detected on systems with EDR")
print("Expected: No hooks on clean systems")

# Example 6: Process Opening via NtOpenProcess
# Standard API: OpenProcess (restricted by EDR)
# Syscall: NtOpenProcess (unrestricted)
print("\n[*] Example 6: Process Opening")
print("Standard approach: OpenProcess (RESTRICTED)")
print("Evasion approach: NtOpenProcess syscall (UNRESTRICTED)")

print("\n[+] Indirect syscall examples complete")
print("[!] Note: This script demonstrates the API. Actual syscall")
print("[!] invocation happens in the compiled binary using assembly stubs.")

# Technical Details:
# 1. SyscallResolver dynamically extracts syscall numbers from ntdll.dll
# 2. Assembly stubs are generated with obfuscation (junk instructions)
# 3. Direct 'syscall' instruction is used (bypasses user-mode hooks)
# 4. Hook detection validates function prologue before syscall extraction
# 5. Supports both standard and obfuscated stub generation

# Common Syscall Numbers (Windows 10 x64 - May vary by build):
# NtAllocateVirtualMemory: 0x18
# NtCreateThreadEx: 0xBD
# NtWriteVirtualMemory: 0x3A
# NtOpenProcess: 0x26
# NtCreateFile: 0x55
#
# Note: These numbers are extracted at runtime to support all Windows versions

# EDR Evasion Patterns:
# Pattern 1: API Unhooking
#   - Detect hook in ntdll function
#   - Extract syscall number
#   - Generate clean syscall stub
#   - Invoke via direct syscall instruction
#
# Pattern 2: Syscall Obfuscation
#   - Add junk instructions before/after syscall
#   - Use varied instruction sequences
#   - Randomize stub layout
#   - Evade signature-based detection
#
# Pattern 3: Hook Validation
#   - Check for JMP/CALL at function start (inline hook)
#   - Validate expected mov r10,rcx pattern
#   - Verify syscall instruction presence
#   - Reject hooked functions

# Integration with Exploit Workflows:
# 1. Shellcode Injection:
#    - Allocate memory: NtAllocateVirtualMemory
#    - Write shellcode: NtWriteVirtualMemory
#    - Create thread: NtCreateThreadEx
#
# 2. DLL Injection:
#    - Open process: NtOpenProcess
#    - Allocate memory: NtAllocateVirtualMemory
#    - Write DLL path: NtWriteVirtualMemory
#    - Create remote thread: NtCreateThreadEx
#
# 3. Process Hollowing:
#    - Create suspended process: NtCreateProcessEx
#    - Unmap memory: NtUnmapViewOfSection
#    - Write payload: NtWriteVirtualMemory
#    - Resume thread: NtResumeThread

print("\n[+] Syscall integration ready for exploit development")
print("[+] Use 'talon build --evasion-level high' for obfuscated stubs")
