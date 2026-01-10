use regex::Regex;
use std::collections::HashMap;
use std::fs;

// ═══════════════════════════════════════════════════════════════════════════
// CTF-SPECIFIC HELPERS - PRODUCTION READY
// ═══════════════════════════════════════════════════════════════════════════

// ────────────────────────────────────────────────────────────────────────────
// FLAG PATTERN EXTRACTION
// ────────────────────────────────────────────────────────────────────────────

pub struct FlagFinder;

impl FlagFinder {
    pub fn find_in_text(text: &str) -> Vec<String> {
        let patterns = vec![
            r"flag\{[^\}]+\}",
            r"FLAG\{[^\}]+\}",
            r"HTB\{[^\}]+\}",
            r"CTF\{[^\}]+\}",
            r"picoCTF\{[^\}]+\}",
            r"[a-f0-9]{32}",
            r"[A-Za-z0-9+/]{32,}={0,2}",
        ];
        
        let mut flags = Vec::new();
        
        for pattern in patterns {
            if let Ok(re) = Regex::new(pattern) {
                for cap in re.find_iter(text) {
                    let flag = cap.as_str().to_string();
                    if !flags.contains(&flag) {
                        flags.push(flag.clone());
                        println!("[FLAG-FINDER] Found: {}", flag);
                    }
                }
            }
        }
        
        flags
    }
    
    pub fn find_in_file(file_path: &str) -> Result<Vec<String>, String> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        println!("[FLAG-FINDER] Searching in {}", file_path);
        Ok(Self::find_in_text(&content))
    }
    
    pub fn find_in_binary(file_path: &str) -> Result<Vec<String>, String> {
        let data = fs::read(file_path)
            .map_err(|e| format!("Failed to read binary: {}", e))?;
        
        let text = String::from_utf8_lossy(&data);
        println!("[FLAG-FINDER] Searching binary: {}", file_path);
        Ok(Self::find_in_text(&text))
    }
    
    pub fn custom_pattern(text: &str, pattern: &str) -> Result<Vec<String>, String> {
        let re = Regex::new(pattern)
            .map_err(|e| format!("Invalid regex: {}", e))?;
        
        let mut matches = Vec::new();
        
        for cap in re.find_iter(text) {
            matches.push(cap.as_str().to_string());
        }
        
        println!("[FLAG-FINDER] Found {} matches for custom pattern", matches.len());
        Ok(matches)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// CTF CATEGORY IDENTIFICATION
// ────────────────────────────────────────────────────────────────────────────

pub struct CTFCategoryIdentifier;

impl CTFCategoryIdentifier {
    pub fn identify(file_path: &str) -> Result<Vec<String>, String> {
        let data = fs::read(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let mut categories = Vec::new();
        
        if data.starts_with(b"MZ") || data.starts_with(b"\x7fELF") {
            categories.push("Binary Exploitation / Reverse Engineering".to_string());
        }
        
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) || 
           data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            categories.push("Steganography".to_string());
        }
        
        if data.starts_with(b"RIFF") {
            categories.push("Audio Steganography".to_string());
        }
        
        if data.starts_with(b"PK") {
            categories.push("Forensics / Archive Analysis".to_string());
        }
        
        let text = String::from_utf8_lossy(&data);
        
        if text.contains("<?php") || text.contains("<?=") {
            categories.push("Web Exploitation (PHP)".to_string());
        }
        
        if text.contains("<!DOCTYPE") || text.contains("<html") {
            categories.push("Web Exploitation (HTML/JavaScript)".to_string());
        }
        
        if text.contains("contract") && text.contains("function") {
            categories.push("Blockchain / Smart Contract".to_string());
        }
        
        if text.contains("RSA") || text.contains("AES") || text.contains("cipher") {
            categories.push("Cryptography".to_string());
        }
        
        println!("[CTF-CATEGORY] Identified categories:");
        for cat in &categories {
            println!("  • {}", cat);
        }
        
        if categories.is_empty() {
            categories.push("Unknown / Mixed".to_string());
        }
        
        Ok(categories)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// COMMON CTF PATTERNS
// ────────────────────────────────────────────────────────────────────────────

pub struct CTFPatterns;

impl CTFPatterns {
    pub fn check_for_patterns(text: &str) -> HashMap<String, Vec<String>> {
        let mut patterns = HashMap::new();
        
        let base64_regex = Regex::new(r"[A-Za-z0-9+/]{20,}={0,2}").unwrap();
        let base64_matches: Vec<String> = base64_regex.find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect();
        if !base64_matches.is_empty() {
            patterns.insert("Base64 Strings".to_string(), base64_matches);
        }
        
        let hex_regex = Regex::new(r"0x[a-fA-F0-9]{8,}").unwrap();
        let hex_matches: Vec<String> = hex_regex.find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect();
        if !hex_matches.is_empty() {
            patterns.insert("Hex Values".to_string(), hex_matches);
        }
        
        let url_regex = Regex::new(r"https?://[^\s<>]+").unwrap();
        let url_matches: Vec<String> = url_regex.find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect();
        if !url_matches.is_empty() {
            patterns.insert("URLs".to_string(), url_matches);
        }
        
        let ip_regex = Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b").unwrap();
        let ip_matches: Vec<String> = ip_regex.find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect();
        if !ip_matches.is_empty() {
            patterns.insert("IP Addresses".to_string(), ip_matches);
        }
        
        let hash32_regex = Regex::new(r"\b[a-f0-9]{32}\b").unwrap();
        let hash32_matches: Vec<String> = hash32_regex.find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect();
        if !hash32_matches.is_empty() {
            patterns.insert("MD5 Hashes".to_string(), hash32_matches);
        }
        
        let hash40_regex = Regex::new(r"\b[a-f0-9]{40}\b").unwrap();
        let hash40_matches: Vec<String> = hash40_regex.find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect();
        if !hash40_matches.is_empty() {
            patterns.insert("SHA-1 Hashes".to_string(), hash40_matches);
        }
        
        println!("[CTF-PATTERNS] Found {} pattern types", patterns.len());
        for (pattern_type, matches) in &patterns {
            println!("[CTF-PATTERNS] {}: {} matches", pattern_type, matches.len());
        }
        
        patterns
    }
}

// ────────────────────────────────────────────────────────────────────────────
// EXPLOIT TEMPLATE GENERATOR
// ────────────────────────────────────────────────────────────────────────────

pub struct ExploitTemplateGenerator;

impl ExploitTemplateGenerator {
    pub fn pwn_template(binary_name: &str, offset: usize) -> String {
        format!(r#"#!/usr/bin/env python3
from pwn import *

# Configuration
binary = './{}'
context.binary = binary
context.log_level = 'debug'

# Start process
if args.REMOTE:
    p = remote('target.com', 1337)
else:
    p = process(binary)

# Exploit
offset = {}
payload = b'A' * offset
payload += p64(0xdeadbeef)  # RIP

p.sendline(payload)
p.interactive()
"#, binary_name, offset)
    }
    
    pub fn web_template(url: &str) -> String {
        format!(r#"#!/usr/bin/env python3
import requests

url = '{}'
session = requests.Session()

# Test for SQL injection
payloads = [
    "' OR '1'='1",
    "' UNION SELECT NULL--",
    "admin' --"
]

for payload in payloads:
    r = session.get(url, params={{'id': payload}})
    print(f"[+] Testing: {{payload}}")
    print(f"[*] Status: {{r.status_code}}")
    if 'error' in r.text.lower():
        print("[!] Possible SQL injection!")
"#, url)
    }
    
    pub fn crypto_template() -> String {
        r#"#!/usr/bin/env python3
from Crypto.Cipher import AES
from Crypto.Util.Padding import unpad
import base64

# Known values
ciphertext = base64.b64decode('...')
key = b'...'
iv = b'...'

# Decrypt
cipher = AES.new(key, AES.MODE_CBC, iv)
plaintext = unpad(cipher.decrypt(ciphertext), AES.block_size)

print(f"[+] Plaintext: {plaintext.decode()}")
"#.to_string()
    }
    
    pub fn save_template(template_type: &str, output_path: &str) -> Result<(), String> {
        let content = match template_type {
            "pwn" => Self::pwn_template("challenge", 264),
            "web" => Self::web_template("http://target.com/"),
            "crypto" => Self::crypto_template(),
            _ => return Err("Unknown template type".to_string()),
        };
        
        fs::write(output_path, content)
            .map_err(|e| format!("Failed to write template: {}", e))?;
        
        println!("[TEMPLATE] Generated {} template: {}", template_type, output_path);
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// QUICK RECON HELPER
// ────────────────────────────────────────────────────────────────────────────

pub struct CTFRecon;

impl CTFRecon {
    pub fn analyze_challenge(file_or_url: &str) -> Result<(), String> {
        println!("\n[CTF-RECON] Analyzing: {}\n", file_or_url);
        
        if file_or_url.starts_with("http") {
            println!("[CTF-RECON] Web challenge detected");
            println!("[CTF-RECON] Suggested tools:");
            println!("  • Burp Suite / ZAP");
            println!("  • sqlmap");
            println!("  • dirb / gobuster");
            println!("  • nikto");
        } else if std::path::Path::new(file_or_url).exists() {
            let categories = CTFCategoryIdentifier::identify(file_or_url)?;
            
            println!("[CTF-RECON] File analysis complete");
            println!("[CTF-RECON] Suggested approach:");
            
            for category in categories {
                match category.as_str() {
                    cat if cat.contains("Binary") => {
                        println!("  • Run 'file' and 'checksec'");
                        println!("  • Disassemble with Ghidra/IDA");
                        println!("  • Look for buffer overflows");
                        println!("  • Check for shellcode opportunities");
                    }
                    cat if cat.contains("Steganography") => {
                        println!("  • Try steghide, stegsolve");
                        println!("  • Check LSB with zsteg");
                        println!("  • Analyze EXIF data");
                        println!("  • Look for hidden files (binwalk)");
                    }
                    cat if cat.contains("Forensics") => {
                        println!("  • Extract with unzip/tar");
                        println!("  • Check for hidden files");
                        println!("  • Analyze file signatures");
                        println!("  • Search for deleted files");
                    }
                    cat if cat.contains("Cryptography") => {
                        println!("  • Identify hash types");
                        println!("  • Try common ciphers (ROT13, XOR)");
                        println!("  • Check for weak keys");
                        println!("  • Analyze encryption modes");
                    }
                    _ => {}
                }
            }
            
            let flag_results = FlagFinder::find_in_file(file_or_url)?;
            if !flag_results.is_empty() {
                println!("\n[CTF-RECON] FLAGS FOUND IN FILE!");
            }
        } else {
            return Err(format!("File or URL not found: {}", file_or_url));
        }
        
        println!();
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// CTF CHEAT SHEET
// ────────────────────────────────────────────────────────────────────────────

pub struct CTFCheatSheet;

impl CTFCheatSheet {
    pub fn show_category(category: &str) {
        match category.to_lowercase().as_str() {
            "pwn" | "binary" => {
                println!(r#"
╔══════════════════════════════════════════════════════════╗
║           PWN / BINARY EXPLOITATION CHEATSHEET           ║
╚══════════════════════════════════════════════════════════╝

Initial Analysis:
  file binary              # Check file type
  checksec binary          # Check protections
  strings binary | grep flag
  
Disassembly:
  objdump -d binary        # Quick disassembly
  ghidra binary            # Full analysis
  
Debugging:
  gdb binary
  > disas main
  > b *main
  > r < input.txt
  > x/20wx $rsp           # Examine stack
  
Common Vulnerabilities:
  • Buffer Overflow → Overwrite return address
  • Format String → %x %s %n
  • Use After Free → Heap exploitation
  • Integer Overflow → Bypass checks

Exploitation:
  cyclic 200               # Pattern generation
  cyclic -l 0x61616161     # Find offset
  ROPgadget --binary bin   # Find ROP gadgets
"#);
            }
            
            "web" => {
                println!(r#"
╔══════════════════════════════════════════════════════════╗
║              WEB EXPLOITATION CHEATSHEET                 ║
╚══════════════════════════════════════════════════════════╝

Enumeration:
  gobuster dir -u URL -w wordlist
  nikto -h URL
  curl -I URL              # Check headers
  
SQL Injection:
  ' OR '1'='1
  ' UNION SELECT NULL--
  sqlmap -u "URL?id=1" --dump
  
XSS:
  <script>alert(1)</script>
  <img src=x onerror=alert(1)>
  
Command Injection:
  ; id
  | whoami
  `cat /etc/passwd`
  
LFI/RFI:
  ../../../../etc/passwd
  php://filter/resource=index.php
  
Tools:
  • Burp Suite
  • SQLMap
  • XSStrike
  • Gobuster
"#);
            }
            
            "crypto" => {
                println!(r#"
╔══════════════════════════════════════════════════════════╗
║              CRYPTOGRAPHY CHEATSHEET                     ║
╚══════════════════════════════════════════════════════════╝

Hash Identification:
  hashid hash
  hash-identifier
  
Common Ciphers:
  • Caesar/ROT13 → Shift cipher
  • XOR → Guess key length
  • Base64 → Decode iteratively
  • Substitution → Frequency analysis
  
RSA Attacks:
  • Small e → Cube root attack
  • Common modulus → GCD attack
  • Wiener's attack → Small d
  
Tools:
  hashcat -m MODE hash wordlist
  john --wordlist=list hash
  RsaCtfTool.py --publickey pub.pem
"#);
            }
            
            "forensics" => {
                println!(r#"
╔══════════════════════════════════════════════════════════╗
║                FORENSICS CHEATSHEET                      ║
╚══════════════════════════════════════════════════════════╝

File Analysis:
  file unknown
  binwalk file
  foremost file            # Carve files
  strings file | grep flag
  
Image Forensics:
  exiftool image.jpg
  steghide extract -sf image.jpg
  zsteg image.png
  stegsolve image.png
  
Memory Forensics:
  volatility -f dump.raw imageinfo
  volatility -f dump.raw pslist
  
Network Forensics:
  wireshark capture.pcap
  tshark -r capture.pcap
  
Disk Forensics:
  mount -o loop disk.img /mnt
  photorec disk.img
"#);
            }
            
            _ => {
                println!("[CTF-CHEATSHEET] Unknown category. Available: pwn, web, crypto, forensics");
            }
        }
    }
    
    pub fn show_all() {
        println!("\n[CTF-CHEATSHEET] Showing all categories...\n");
        Self::show_category("pwn");
        Self::show_category("web");
        Self::show_category("crypto");
        Self::show_category("forensics");
    }
}

// ────────────────────────────────────────────────────────────────────────────
// AUTOMATED HINTS SYSTEM
// ────────────────────────────────────────────────────────────────────────────

pub struct HintSystem;

impl HintSystem {
    pub fn get_hints(challenge_description: &str) -> Vec<String> {
        let mut hints = Vec::new();
        let desc_lower = challenge_description.to_lowercase();
        
        if desc_lower.contains("overflow") || desc_lower.contains("buffer") {
            hints.push("Try finding the offset with pattern_create/cyclic".to_string());
            hints.push("Check if there's a win() or get_flag() function".to_string());
            hints.push("Look for ROP gadgets if NX is enabled".to_string());
        }
        
        if desc_lower.contains("sql") || desc_lower.contains("database") {
            hints.push("Test for SQL injection with ' OR '1'='1".to_string());
            hints.push("Try UNION-based injection".to_string());
            hints.push("Use sqlmap for automated exploitation".to_string());
        }
        
        if desc_lower.contains("image") || desc_lower.contains("picture") {
            hints.push("Check EXIF data with exiftool".to_string());
            hints.push("Try LSB extraction with zsteg/stegsolve".to_string());
            hints.push("Look for hidden files with binwalk".to_string());
        }
        
        if desc_lower.contains("hash") || desc_lower.contains("password") {
            hints.push("Identify the hash type first".to_string());
            hints.push("Try rockyou.txt wordlist".to_string());
            hints.push("Use hashcat or john the ripper".to_string());
        }
        
        if desc_lower.contains("rsa") || desc_lower.contains("encryption") {
            hints.push("Check for small public exponent".to_string());
            hints.push("Try factordb.com for weak modulus".to_string());
            hints.push("Look for known RSA attacks".to_string());
        }
        
        if hints.is_empty() {
            hints.push("Start with reconnaissance and enumeration".to_string());
            hints.push("Look for common CTF patterns (base64, hex, etc)".to_string());
            hints.push("Search for flags in unexpected places".to_string());
        }
        
        println!("[HINT-SYSTEM] Generated {} hints", hints.len());
        for (i, hint) in hints.iter().enumerate() {
            println!("[HINT-SYSTEM]   {}. {}", i + 1, hint);
        }
        
        hints
    }
}
