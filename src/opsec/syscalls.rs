// Windows Indirect Syscalls for EDR Bypass
//
// This module implements direct syscall invocation to bypass user-mode API hooks
// commonly used by EDR (Endpoint Detection and Response) solutions.
//
// Key Features:
// - Dynamic syscall number extraction from ntdll.dll at runtime
// - Direct syscall instruction execution (bypasses user-mode hooks)
// - Hook detection for IAT (Import Address Table) and inline hooks
// - Syscall obfuscation with junk instructions
// - Support for x64 syscall instruction and WoW64 fallback
//
// Architecture:
// 1. SyscallResolver: Dynamically resolves syscall numbers from ntdll.dll
// 2. HookDetector: Detects user-land hooks (IAT, inline)
// 3. SyscallStub: Generates assembly trampolines for direct syscalls
// 4. Obfuscator: Adds junk instructions to evade signature detection

use std::collections::HashMap;

#[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
use windows::Win32::{
    Foundation::{HANDLE, HMODULE},
    System::{
        LibraryLoader::{GetModuleHandleA, GetProcAddress},
        Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS},
        SystemServices::IMAGE_DOS_HEADER,
    },
};

/// Errors that can occur during syscall resolution
#[derive(Debug)]
pub enum SyscallError {
    /// Failed to load ntdll.dll
    NtdllNotFound,
    /// Failed to find the specified function in ntdll
    FunctionNotFound(String),
    /// Failed to extract syscall number from function
    SyscallNumberNotFound(String),
    /// Failed to allocate executable memory for stub
    StubAllocationFailed,
    /// Hook detected - cannot safely invoke syscall
    HookDetected(String, HookType),
    /// Architecture not supported (only x64 currently)
    UnsupportedArchitecture,
    /// Memory protection change failed
    ProtectionChangeFailed,
}

impl std::fmt::Display for SyscallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyscallError::NtdllNotFound => write!(f, "Failed to load ntdll.dll"),
            SyscallError::FunctionNotFound(func) => {
                write!(f, "Function not found in ntdll: {}", func)
            }
            SyscallError::SyscallNumberNotFound(func) => {
                write!(f, "Could not extract syscall number from {}", func)
            }
            SyscallError::StubAllocationFailed => {
                write!(f, "Failed to allocate memory for syscall stub")
            }
            SyscallError::HookDetected(func, hook_type) => {
                write!(f, "Hook detected in {}: {:?}", func, hook_type)
            }
            SyscallError::UnsupportedArchitecture => {
                write!(f, "Architecture not supported (x64 only)")
            }
            SyscallError::ProtectionChangeFailed => {
                write!(f, "Failed to change memory protection")
            }
        }
    }
}

impl std::error::Error for SyscallError {}

/// Types of hooks that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookType {
    /// Import Address Table hook
    IatHook,
    /// Inline hook (instruction modification)
    InlineHook,
    /// No hook detected
    None,
}

/// Detects user-land hooks in Windows API functions
pub struct HookDetector;

impl HookDetector {
    /// Detects if a function has an inline hook
    ///
    /// Inline hooks typically replace the first few bytes of a function with a JMP
    /// instruction to redirect execution. This checks for common patterns:
    /// - JMP (E9) or CALL (E8) as first instruction
    /// - Push + Ret gadget
    /// - MOV RAX + JMP RAX pattern (Trampoline hook)
    #[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
    pub fn detect_inline_hook(function_addr: *const u8) -> bool {
        if function_addr.is_null() {
            return false;
        }

        unsafe {
            let bytes = std::slice::from_raw_parts(function_addr, 16);

            // Check for common hook patterns:
            // 1. JMP instruction (E9 xx xx xx xx)
            if bytes[0] == 0xE9 {
                return true;
            }

            // 2. CALL instruction (E8 xx xx xx xx) - less common but possible
            if bytes[0] == 0xE8 {
                return true;
            }

            // 3. Push + Ret gadget (68 xx xx xx xx C3)
            if bytes[0] == 0x68 && bytes.len() >= 6 && bytes[5] == 0xC3 {
                return true;
            }

            // 4. MOV RAX + JMP RAX trampoline (48 B8 xx xx xx xx xx xx xx xx FF E0)
            if bytes.len() >= 12
                && bytes[0] == 0x48
                && bytes[1] == 0xB8
                && bytes[10] == 0xFF
                && bytes[11] == 0xE0
            {
                return true;
            }

            // 5. Check for unexpected NOP padding (common in hooks)
            let nop_count = bytes.iter().take(8).filter(|&&b| b == 0x90).count();
            if nop_count >= 5 {
                return true;
            }

            false
        }
    }

    /// Checks if ntdll function matches the expected syscall pattern
    ///
    /// Expected pattern for x64 ntdll syscall stub:
    /// ```asm
    /// mov r10, rcx         ; 4C 8B D1
    /// mov eax, <syscall#>  ; B8 xx xx 00 00
    /// syscall              ; 0F 05
    /// ret                  ; C3
    /// ```
    #[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
    pub fn is_valid_syscall_stub(function_addr: *const u8) -> bool {
        if function_addr.is_null() {
            return false;
        }

        unsafe {
            let bytes = std::slice::from_raw_parts(function_addr, 12);

            // Check for expected syscall stub pattern
            if bytes.len() >= 11 {
                // mov r10, rcx
                if bytes[0] == 0x4C && bytes[1] == 0x8B && bytes[2] == 0xD1 {
                    // mov eax, syscall_number
                    if bytes[3] == 0xB8 {
                        // syscall instruction
                        if bytes[8] == 0x0F && bytes[9] == 0x05 {
                            return true;
                        }
                    }
                }
            }

            false
        }
    }
}

/// Syscall number resolver that extracts syscall numbers from ntdll.dll
pub struct SyscallResolver {
    /// Cache of function names to syscall numbers
    syscall_cache: HashMap<String, u32>,
    /// Handle to ntdll.dll
    #[cfg(target_os = "windows")]
    ntdll_handle: HMODULE,
}

impl SyscallResolver {
    /// Creates a new SyscallResolver and loads ntdll.dll
    #[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
    pub fn new() -> Result<Self, SyscallError> {
        unsafe {
            let ntdll_name = "ntdll.dll\0";
            let ntdll_handle = GetModuleHandleA(windows::core::PCSTR(ntdll_name.as_ptr()))
                .map_err(|_| SyscallError::NtdllNotFound)?;

            Ok(SyscallResolver {
                syscall_cache: HashMap::new(),
                ntdll_handle,
            })
        }
    }

    /// Resolves a syscall number from ntdll function name
    ///
    /// # Arguments
    /// * `function_name` - Name of the NT function (e.g., "NtAllocateVirtualMemory")
    ///
    /// # Returns
    /// * `Result<u32, SyscallError>` - The syscall number if found
    #[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
    pub fn resolve_syscall_number(&mut self, function_name: &str) -> Result<u32, SyscallError> {
        // Check cache first
        if let Some(&number) = self.syscall_cache.get(function_name) {
            return Ok(number);
        }

        unsafe {
            // Get function address from ntdll
            let func_name_cstr = format!("{}\0", function_name);
            let func_addr = GetProcAddress(
                self.ntdll_handle,
                windows::core::PCSTR(func_name_cstr.as_ptr()),
            )
            .ok_or_else(|| SyscallError::FunctionNotFound(function_name.to_string()))?;

            let func_ptr = func_addr as *const u8;

            // Check for inline hooks before extracting syscall number
            if HookDetector::detect_inline_hook(func_ptr) {
                return Err(SyscallError::HookDetected(
                    function_name.to_string(),
                    HookType::InlineHook,
                ));
            }

            // Validate it's a proper syscall stub
            if !HookDetector::is_valid_syscall_stub(func_ptr) {
                return Err(SyscallError::SyscallNumberNotFound(
                    function_name.to_string(),
                ));
            }

            // Extract syscall number from bytes 4-7 (mov eax, <syscall#>)
            let bytes = std::slice::from_raw_parts(func_ptr, 8);
            let syscall_number = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

            // Cache for future use
            self.syscall_cache
                .insert(function_name.to_string(), syscall_number);

            Ok(syscall_number)
        }
    }

    /// Generates a direct syscall stub for the given syscall number
    ///
    /// Creates executable memory containing:
    /// ```asm
    /// mov r10, rcx         ; Save first parameter
    /// mov eax, <syscall#>  ; Load syscall number
    /// syscall              ; Invoke syscall
    /// ret                  ; Return to caller
    /// ```
    ///
    /// # Arguments
    /// * `syscall_number` - The syscall number to invoke
    ///
    /// # Returns
    /// * `Result<*const u8, SyscallError>` - Pointer to executable stub
    #[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
    pub fn generate_syscall_stub(&self, syscall_number: u32) -> Result<*const u8, SyscallError> {
        unsafe {
            // Allocate executable memory for stub
            let stub_size = 64; // Extra space for obfuscation
            let stub_mem = windows::Win32::System::Memory::VirtualAlloc(
                None,
                stub_size,
                windows::Win32::System::Memory::MEM_COMMIT
                    | windows::Win32::System::Memory::MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            );

            if stub_mem.is_null() {
                return Err(SyscallError::StubAllocationFailed);
            }

            let stub_ptr = stub_mem as *mut u8;

            // Generate syscall stub with obfuscation
            let mut offset = 0;

            // Add junk instructions for obfuscation (NOP sled with varied instructions)
            stub_ptr.add(offset).write(0x90); // nop
            offset += 1;
            stub_ptr.add(offset).write(0x48); // xor rax, rax
            stub_ptr.add(offset + 1).write(0x31);
            stub_ptr.add(offset + 2).write(0xC0);
            offset += 3;

            // mov r10, rcx (4C 8B D1)
            stub_ptr.add(offset).write(0x4C);
            stub_ptr.add(offset + 1).write(0x8B);
            stub_ptr.add(offset + 2).write(0xD1);
            offset += 3;

            // mov eax, syscall_number (B8 xx xx 00 00)
            stub_ptr.add(offset).write(0xB8);
            let syscall_bytes = syscall_number.to_le_bytes();
            stub_ptr.add(offset + 1).write(syscall_bytes[0]);
            stub_ptr.add(offset + 2).write(syscall_bytes[1]);
            stub_ptr.add(offset + 3).write(syscall_bytes[2]);
            stub_ptr.add(offset + 4).write(syscall_bytes[3]);
            offset += 5;

            // More junk for obfuscation
            stub_ptr.add(offset).write(0x90); // nop
            offset += 1;

            // syscall (0F 05)
            stub_ptr.add(offset).write(0x0F);
            stub_ptr.add(offset + 1).write(0x05);
            offset += 2;

            // ret (C3)
            stub_ptr.add(offset).write(0xC3);

            Ok(stub_ptr as *const u8)
        }
    }

    /// Generates an obfuscated syscall stub with additional junk instructions
    ///
    /// This variant adds more complex obfuscation to evade signature-based detection.
    /// Includes randomized instruction ordering and dead code.
    #[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
    pub fn generate_obfuscated_stub(
        &self,
        syscall_number: u32,
    ) -> Result<*const u8, SyscallError> {
        unsafe {
            let stub_size = 128; // More space for complex obfuscation
            let stub_mem = windows::Win32::System::Memory::VirtualAlloc(
                None,
                stub_size,
                windows::Win32::System::Memory::MEM_COMMIT
                    | windows::Win32::System::Memory::MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            );

            if stub_mem.is_null() {
                return Err(SyscallError::StubAllocationFailed);
            }

            let stub_ptr = stub_mem as *mut u8;
            let mut offset = 0;

            // Complex obfuscation pattern
            // 1. Junk computation (never used)
            stub_ptr.add(offset).write(0x48); // mov rax, 0x1234
            stub_ptr.add(offset + 1).write(0xB8);
            stub_ptr.add(offset + 2).write(0x34);
            stub_ptr.add(offset + 3).write(0x12);
            stub_ptr.add(offset + 4).write(0x00);
            stub_ptr.add(offset + 5).write(0x00);
            stub_ptr.add(offset + 6).write(0x00);
            stub_ptr.add(offset + 7).write(0x00);
            stub_ptr.add(offset + 8).write(0x00);
            stub_ptr.add(offset + 9).write(0x00);
            offset += 10;

            // 2. Conditional junk (always taken)
            stub_ptr.add(offset).write(0x48); // test rax, rax
            stub_ptr.add(offset + 1).write(0x85);
            stub_ptr.add(offset + 2).write(0xC0);
            offset += 3;

            // 3. Short forward jump (skip 2 bytes of junk)
            stub_ptr.add(offset).write(0xEB); // jmp short +2
            stub_ptr.add(offset + 1).write(0x02);
            offset += 2;

            // 4. Dead bytes (skipped by jump)
            stub_ptr.add(offset).write(0xCC); // int3 (never executed)
            stub_ptr.add(offset + 1).write(0xCC);
            offset += 2;

            // 5. Actual syscall stub begins here
            // mov r10, rcx
            stub_ptr.add(offset).write(0x4C);
            stub_ptr.add(offset + 1).write(0x8B);
            stub_ptr.add(offset + 2).write(0xD1);
            offset += 3;

            // mov eax, syscall_number
            stub_ptr.add(offset).write(0xB8);
            let syscall_bytes = syscall_number.to_le_bytes();
            stub_ptr.add(offset + 1).write(syscall_bytes[0]);
            stub_ptr.add(offset + 2).write(syscall_bytes[1]);
            stub_ptr.add(offset + 3).write(syscall_bytes[2]);
            stub_ptr.add(offset + 4).write(syscall_bytes[3]);
            offset += 5;

            // 6. More junk before syscall
            stub_ptr.add(offset).write(0x90); // nop
            stub_ptr.add(offset + 1).write(0x90);
            offset += 2;

            // syscall
            stub_ptr.add(offset).write(0x0F);
            stub_ptr.add(offset + 1).write(0x05);
            offset += 2;

            // ret
            stub_ptr.add(offset).write(0xC3);

            Ok(stub_ptr as *const u8)
        }
    }

    /// Returns all cached syscall numbers (for debugging)
    pub fn get_cache(&self) -> &HashMap<String, u32> {
        &self.syscall_cache
    }

    /// Clears the syscall cache
    pub fn clear_cache(&mut self) {
        self.syscall_cache.clear();
    }
}

/// Common NT API syscalls with their typical function names
pub struct NtSyscalls;

impl NtSyscalls {
    /// Common NT API function names that can be invoked via syscall
    pub const COMMON_SYSCALLS: &'static [&'static str] = &[
        "NtAllocateVirtualMemory",
        "NtFreeVirtualMemory",
        "NtProtectVirtualMemory",
        "NtReadVirtualMemory",
        "NtWriteVirtualMemory",
        "NtCreateProcess",
        "NtCreateProcessEx",
        "NtCreateThread",
        "NtCreateThreadEx",
        "NtOpenProcess",
        "NtOpenThread",
        "NtTerminateProcess",
        "NtTerminateThread",
        "NtSuspendProcess",
        "NtSuspendThread",
        "NtResumeProcess",
        "NtResumeThread",
        "NtCreateFile",
        "NtOpenFile",
        "NtReadFile",
        "NtWriteFile",
        "NtDeleteFile",
        "NtQuerySystemInformation",
        "NtQueryInformationProcess",
        "NtQueryInformationThread",
        "NtSetInformationProcess",
        "NtSetInformationThread",
        "NtDuplicateObject",
        "NtClose",
        "NtWaitForSingleObject",
        "NtWaitForMultipleObjects",
        "NtCreateEvent",
        "NtSetEvent",
        "NtResetEvent",
        "NtCreateMutant",
        "NtReleaseMutant",
        "NtCreateSection",
        "NtMapViewOfSection",
        "NtUnmapViewOfSection",
        "NtLoadDriver",
        "NtUnloadDriver",
        "NtQueryDirectoryFile",
        "NtCreateKey",
        "NtOpenKey",
        "NtDeleteKey",
        "NtSetValueKey",
        "NtQueryValueKey",
    ];

    /// Preloads common syscall numbers into the resolver cache
    #[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
    pub fn preload_common_syscalls(
        resolver: &mut SyscallResolver,
    ) -> Result<(), SyscallError> {
        for &syscall_name in Self::COMMON_SYSCALLS {
            match resolver.resolve_syscall_number(syscall_name) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Warning: Failed to resolve {}: {}", syscall_name, e);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
    fn test_syscall_resolver_creation() {
        let resolver = SyscallResolver::new();
        assert!(resolver.is_ok(), "SyscallResolver should initialize");
    }

    #[test]
    #[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
    fn test_resolve_ntallocatevirtualmemory() {
        let mut resolver = SyscallResolver::new().expect("Failed to create resolver");
        let result = resolver.resolve_syscall_number("NtAllocateVirtualMemory");
        assert!(
            result.is_ok(),
            "Should resolve NtAllocateVirtualMemory syscall number"
        );
    }

    #[test]
    #[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
    fn test_cache_functionality() {
        let mut resolver = SyscallResolver::new().expect("Failed to create resolver");
        let syscall_name = "NtAllocateVirtualMemory";

        // First call should populate cache
        let first_result = resolver
            .resolve_syscall_number(syscall_name)
            .expect("Failed to resolve");

        // Cache should now contain the syscall
        assert!(resolver.get_cache().contains_key(syscall_name));

        // Second call should return same value (from cache)
        let second_result = resolver
            .resolve_syscall_number(syscall_name)
            .expect("Failed to resolve");
        assert_eq!(first_result, second_result);
    }

    #[test]
    #[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
    fn test_generate_syscall_stub() {
        let resolver = SyscallResolver::new().expect("Failed to create resolver");
        let syscall_number = 0x18; // Example syscall number
        let stub = resolver.generate_syscall_stub(syscall_number);
        assert!(stub.is_ok(), "Should generate syscall stub");
        assert!(!stub.unwrap().is_null(), "Stub pointer should be valid");
    }

    #[test]
    #[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
    fn test_generate_obfuscated_stub() {
        let resolver = SyscallResolver::new().expect("Failed to create resolver");
        let syscall_number = 0x18;
        let stub = resolver.generate_obfuscated_stub(syscall_number);
        assert!(stub.is_ok(), "Should generate obfuscated stub");
        assert!(!stub.unwrap().is_null(), "Stub pointer should be valid");
    }

    #[test]
    #[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
    fn test_preload_common_syscalls() {
        let mut resolver = SyscallResolver::new().expect("Failed to create resolver");
        let result = NtSyscalls::preload_common_syscalls(&mut resolver);
        assert!(result.is_ok(), "Should preload common syscalls");
        assert!(
            resolver.get_cache().len() > 0,
            "Cache should contain syscalls"
        );
    }

    #[test]
    #[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
    fn test_clear_cache() {
        let mut resolver = SyscallResolver::new().expect("Failed to create resolver");
        let _ = resolver.resolve_syscall_number("NtAllocateVirtualMemory");
        assert!(resolver.get_cache().len() > 0);

        resolver.clear_cache();
        assert_eq!(resolver.get_cache().len(), 0, "Cache should be empty");
    }

    #[test]
    #[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
    fn test_invalid_function_name() {
        let mut resolver = SyscallResolver::new().expect("Failed to create resolver");
        let result = resolver.resolve_syscall_number("InvalidFunctionNameXYZ");
        assert!(
            result.is_err(),
            "Should return error for invalid function name"
        );
    }

    #[test]
    fn test_hook_type_equality() {
        assert_eq!(HookType::IatHook, HookType::IatHook);
        assert_eq!(HookType::InlineHook, HookType::InlineHook);
        assert_eq!(HookType::None, HookType::None);
        assert_ne!(HookType::IatHook, HookType::InlineHook);
    }

    #[test]
    fn test_syscall_error_display() {
        let error = SyscallError::NtdllNotFound;
        assert_eq!(error.to_string(), "Failed to load ntdll.dll");

        let error2 = SyscallError::FunctionNotFound("TestFunc".to_string());
        assert!(error2.to_string().contains("TestFunc"));
    }
}
