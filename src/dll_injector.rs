use std::process::Command;

pub fn inject_dll(pid: u32, dll_path: &str) -> Result<(), String> {
    println!("[INJECTOR] (stub) Would inject DLL {} into PID {}", dll_path, pid);
    Ok(())
}
