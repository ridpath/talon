use std::process::Command;

/// Launches a Ghidra RPC headless bridge (assumes Ghidra extension is setup)
pub fn send_to_ghidra(script: &str, binary: &str) -> Result<(), String> {
    let status = Command::new("ghidra_bridge_cli")
        .arg("--binary")
        .arg(binary)
        .arg("--script")
        .arg(script)
        .status()
        .map_err(|e| format!("Ghidra call failed: {}", e))?;

    if status.success() {
        println!("[GHIDRA] [OK] Script executed");
    } else {
        println!("[GHIDRA] [ERROR] Script execution failed");
    }

    Ok(())
}
