use sha2::{Sha256, Sha512, Digest};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::process::Command;

// ═══════════════════════════════════════════════════════════════════════════
// CRYPTO & HASH CRACKING TOOLKIT - PRODUCTION READY
// ═══════════════════════════════════════════════════════════════════════════

// ────────────────────────────────────────────────────────────────────────────
// HASH IDENTIFICATION
// ────────────────────────────────────────────────────────────────────────────

pub struct HashIdentifier;

impl HashIdentifier {
    pub fn identify(hash: &str) -> Vec<String> {
        let mut possible_types = Vec::new();
        let hash = hash.trim();
        let len = hash.len();
        
        match len {
            32 => {
                if hash.chars().all(|c| c.is_ascii_hexdigit()) {
                    possible_types.push("MD5".to_string());
                    possible_types.push("NTLM".to_string());
                }
            }
            40 => {
                if hash.chars().all(|c| c.is_ascii_hexdigit()) {
                    possible_types.push("SHA-1".to_string());
                }
            }
            64 => {
                if hash.chars().all(|c| c.is_ascii_hexdigit()) {
                    possible_types.push("SHA-256".to_string());
                    possible_types.push("SHA3-256".to_string());
                }
            }
            96 => {
                if hash.chars().all(|c| c.is_ascii_hexdigit()) {
                    possible_types.push("SHA-384".to_string());
                }
            }
            128 => {
                if hash.chars().all(|c| c.is_ascii_hexdigit()) {
                    possible_types.push("SHA-512".to_string());
                    possible_types.push("SHA3-512".to_string());
                }
            }
            _ => {}
        }
        
        if hash.starts_with("$1$") {
            possible_types.push("MD5 (Unix)".to_string());
        } else if hash.starts_with("$2a$") || hash.starts_with("$2b$") || hash.starts_with("$2y$") {
            possible_types.push("bcrypt".to_string());
        } else if hash.starts_with("$5$") {
            possible_types.push("SHA-256 (Unix)".to_string());
        } else if hash.starts_with("$6$") {
            possible_types.push("SHA-512 (Unix)".to_string());
        } else if hash.starts_with("{SHA}") || hash.starts_with("{SSHA}") {
            possible_types.push("LDAP SHA".to_string());
        } else if hash.starts_with("$apr1$") {
            possible_types.push("Apache MD5".to_string());
        }
        
        if hash.contains(":") {
            let parts: Vec<&str> = hash.split(':').collect();
            if parts.len() == 2 {
                if parts[0].len() == 32 && parts[1].len() == 32 {
                    possible_types.push("MD5(pass:salt)".to_string());
                }
            }
        }
        
        if possible_types.is_empty() {
            possible_types.push("Unknown".to_string());
        }
        
        println!("[HASH-ID] Hash length: {}", len);
        println!("[HASH-ID] Possible types:");
        for (i, t) in possible_types.iter().enumerate() {
            println!("  {}. {}", i+1, t);
        }
        
        possible_types
    }
    
    pub fn get_hashcat_mode(hash_type: &str) -> Option<u32> {
        match hash_type {
            "MD5" => Some(0),
            "SHA-1" => Some(100),
            "SHA-256" => Some(1400),
            "SHA-512" => Some(1700),
            "NTLM" => Some(1000),
            "bcrypt" => Some(3200),
            "MD5 (Unix)" => Some(500),
            "SHA-256 (Unix)" => Some(7400),
            "SHA-512 (Unix)" => Some(1800),
            "Apache MD5" => Some(1600),
            _ => None,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// HASH CRACKING (HASHCAT/JOHN INTEGRATION)
// ────────────────────────────────────────────────────────────────────────────

pub struct HashCracker;

impl HashCracker {
    pub fn crack_with_hashcat(hash: &str, wordlist: &str, mode: u32) -> Result<String, String> {
        println!("[HASHCAT] Starting hash cracking");
        println!("[HASHCAT] Hash: {}", hash);
        println!("[HASHCAT] Wordlist: {}", wordlist);
        println!("[HASHCAT] Mode: {}", mode);
        
        let hash_file = "/tmp/talon_hash.txt";
        fs::write(hash_file, hash).map_err(|e| format!("Failed to write hash file: {}", e))?;
        
        let output = Command::new("hashcat")
            .args(&[
                "-m", &mode.to_string(),
                "-a", "0",
                hash_file,
                wordlist,
                "--show",
                "--quiet",
            ])
            .output()
            .map_err(|e| format!("Hashcat execution failed: {}. Is hashcat installed?", e))?;
        
        let result = String::from_utf8_lossy(&output.stdout);
        
        if result.contains(":") {
            let parts: Vec<&str> = result.split(':').collect();
            if parts.len() >= 2 {
                let password = parts[1].trim();
                println!("[HASHCAT] CRACKED: {}", password);
                return Ok(password.to_string());
            }
        }
        
        println!("[HASHCAT] Hash not cracked");
        Ok("Not cracked".to_string())
    }
    
    pub fn crack_with_john(hash_file: &str, wordlist: &str) -> Result<String, String> {
        println!("[JOHN] Starting hash cracking");
        println!("[JOHN] Hash file: {}", hash_file);
        println!("[JOHN] Wordlist: {}", wordlist);
        
        let output = Command::new("john")
            .args(&[
                "--wordlist", wordlist,
                hash_file,
            ])
            .output()
            .map_err(|e| format!("John execution failed: {}. Is john installed?", e))?;
        
        let result = String::from_utf8_lossy(&output.stdout);
        println!("[JOHN] Output:\n{}", result);
        
        let show_output = Command::new("john")
            .args(&["--show", hash_file])
            .output()
            .map_err(|e| format!("John show failed: {}", e))?;
        
        let show_result = String::from_utf8_lossy(&show_output.stdout);
        
        if show_result.contains(":") {
            let parts: Vec<&str> = show_result.split(':').collect();
            if parts.len() >= 2 {
                let password = parts[1].trim();
                println!("[JOHN] CRACKED: {}", password);
                return Ok(password.to_string());
            }
        }
        
        println!("[JOHN] Hash not cracked");
        Ok("Not cracked".to_string())
    }
    
    pub fn dictionary_attack(hash: &str, wordlist: &str, hash_type: &str) -> Result<Option<String>, String> {
        println!("[DICT-ATTACK] Starting dictionary attack");
        
        let words = fs::read_to_string(wordlist)
            .map_err(|e| format!("Failed to read wordlist: {}", e))?;
        
        for (i, word) in words.lines().enumerate() {
            let computed_hash = match hash_type {
                "MD5" => format!("{:x}", md5::compute(word)),
                "SHA-256" => {
                    let mut hasher = Sha256::new();
                    hasher.update(word);
                    format!("{:x}", hasher.finalize())
                }
                "SHA-512" => {
                    let mut hasher = Sha512::new();
                    hasher.update(word);
                    format!("{:x}", hasher.finalize())
                }
                _ => continue,
            };
            
            if computed_hash == hash {
                println!("[DICT-ATTACK] FOUND at line {}: {}", i+1, word);
                return Ok(Some(word.to_string()));
            }
            
            if (i + 1) % 10000 == 0 {
                println!("[DICT-ATTACK] Tested {} passwords...", i + 1);
            }
        }
        
        println!("[DICT-ATTACK] Password not found in wordlist");
        Ok(None)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// WORDLIST GENERATION
// ────────────────────────────────────────────────────────────────────────────

pub struct WordlistGenerator;

impl WordlistGenerator {
    pub fn generate_from_website(url: &str, output: &str) -> Result<(), String> {
        println!("[WORDLIST] Generating wordlist from {}", url);
        
        let output_result = Command::new("cewl")
            .args(&[
                "-w", output,
                "-d", "2",
                "-m", "5",
                url,
            ])
            .output()
            .map_err(|e| format!("CeWL execution failed: {}. Is cewl installed?", e))?;
        
        if output_result.status.success() {
            println!("[WORDLIST] Wordlist generated: {}", output);
            Ok(())
        } else {
            Err(format!("CeWL failed: {}", String::from_utf8_lossy(&output_result.stderr)))
        }
    }
    
    pub fn generate_mutations(word: &str) -> Vec<String> {
        let mut mutations = Vec::new();
        
        mutations.push(word.to_string());
        mutations.push(word.to_uppercase());
        mutations.push(word.to_lowercase());
        
        let mut capitalized = word.to_lowercase();
        if let Some(first) = capitalized.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        mutations.push(capitalized);
        
        for year in 2020..=2025 {
            mutations.push(format!("{}{}", word, year));
            mutations.push(format!("{}{}", year, word));
        }
        
        for num in 0..100 {
            mutations.push(format!("{}{}", word, num));
        }
        
        let leet_map: HashMap<char, char> = [
            ('a', '4'), ('e', '3'), ('i', '1'), ('o', '0'), ('s', '5'),
            ('t', '7'), ('l', '1'), ('g', '9'),
        ].iter().cloned().collect();
        
        let mut leet = String::new();
        for c in word.chars() {
            if let Some(&leet_char) = leet_map.get(&c.to_lowercase().next().unwrap()) {
                leet.push(leet_char);
            } else {
                leet.push(c);
            }
        }
        mutations.push(leet);
        
        mutations.push(format!("{}!", word));
        mutations.push(format!("{}@", word));
        mutations.push(format!("{}#", word));
        mutations.push(format!("{}123", word));
        mutations.push(format!("{}!", word));
        
        println!("[WORDLIST] Generated {} mutations for '{}'", mutations.len(), word);
        
        mutations
    }
    
    pub fn generate_combinations(words: &[String], max_length: usize) -> Vec<String> {
        let mut combinations = Vec::new();
        
        for i in 0..words.len() {
            for j in 0..words.len() {
                if i != j {
                    let combo = format!("{}{}", words[i], words[j]);
                    if combo.len() <= max_length {
                        combinations.push(combo.clone());
                        combinations.push(format!("{}_{}", words[i], words[j]));
                        combinations.push(format!("{}-{}", words[i], words[j]));
                    }
                }
            }
        }
        
        println!("[WORDLIST] Generated {} combinations", combinations.len());
        combinations
    }
    
    pub fn common_passwords() -> Vec<&'static str> {
        vec![
            "password", "123456", "12345678", "qwerty", "abc123",
            "monkey", "1234567", "letmein", "trustno1", "dragon",
            "baseball", "111111", "iloveyou", "master", "sunshine",
            "ashley", "bailey", "passw0rd", "shadow", "123123",
            "654321", "superman", "qazwsx", "michael", "football",
            "admin", "root", "toor", "pass", "test",
        ]
    }
    
    pub fn save_wordlist(words: &[String], output_path: &str) -> Result<(), String> {
        let content = words.join("\n");
        fs::write(output_path, content)
            .map_err(|e| format!("Failed to write wordlist: {}", e))?;
        
        println!("[WORDLIST] Saved {} words to {}", words.len(), output_path);
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// PASSWORD ANALYSIS
// ────────────────────────────────────────────────────────────────────────────

pub struct PasswordAnalyzer;

impl PasswordAnalyzer {
    pub fn strength(password: &str) -> u8 {
        let mut score = 0u8;
        
        if password.len() >= 8 {
            score += 20;
        }
        if password.len() >= 12 {
            score += 10;
        }
        if password.len() >= 16 {
            score += 10;
        }
        
        if password.chars().any(|c| c.is_lowercase()) {
            score += 10;
        }
        if password.chars().any(|c| c.is_uppercase()) {
            score += 10;
        }
        if password.chars().any(|c| c.is_numeric()) {
            score += 10;
        }
        if password.chars().any(|c| !c.is_alphanumeric()) {
            score += 20;
        }
        
        let unique_chars = password.chars().collect::<std::collections::HashSet<_>>().len();
        if unique_chars > password.len() / 2 {
            score += 10;
        }
        
        println!("[PASS-ANALYZE] Password strength: {}/100", score);
        println!("[PASS-ANALYZE] Length: {}", password.len());
        println!("[PASS-ANALYZE] Unique chars: {}", unique_chars);
        
        score
    }
    
    pub fn check_common(&self, password: &str) -> bool {
        let common = WordlistGenerator::common_passwords();
        let is_common = common.contains(&password.to_lowercase().as_str());
        
        if is_common {
            println!("[PASS-ANALYZE] WARNING: This is a common password!");
        } else {
            println!("[PASS-ANALYZE] Not in common password list");
        }
        
        is_common
    }
}

// ────────────────────────────────────────────────────────────────────────────
// HASH GENERATION
// ────────────────────────────────────────────────────────────────────────────

pub struct HashGenerator;

impl HashGenerator {
    pub fn md5(data: &str) -> String {
        format!("{:x}", md5::compute(data))
    }
    
    pub fn sha1(data: &str) -> String {
        use sha1::{Sha1, Digest};
        let mut hasher = Sha1::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
    
    pub fn sha256(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
    
    pub fn sha512(data: &str) -> String {
        let mut hasher = Sha512::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
    
    pub fn ntlm(password: &str) -> String {
        let utf16: Vec<u16> = password.encode_utf16().collect();
        let bytes: Vec<u8> = utf16.iter()
            .flat_map(|&c| vec![(c & 0xff) as u8, (c >> 8) as u8])
            .collect();
        
        format!("{:x}", md5::compute(&bytes))
    }
    
    pub fn generate_all(data: &str) -> HashMap<String, String> {
        let mut hashes = HashMap::new();
        
        hashes.insert("MD5".to_string(), Self::md5(data));
        hashes.insert("SHA-1".to_string(), Self::sha1(data));
        hashes.insert("SHA-256".to_string(), Self::sha256(data));
        hashes.insert("SHA-512".to_string(), Self::sha512(data));
        hashes.insert("NTLM".to_string(), Self::ntlm(data));
        
        println!("[HASH-GEN] Generated hashes for input:");
        for (algo, hash) in &hashes {
            println!("  {:<10}: {}", algo, hash);
        }
        
        hashes
    }
}

// ────────────────────────────────────────────────────────────────────────────
// RAINBOW TABLE GENERATOR
// ────────────────────────────────────────────────────────────────────────────

pub struct RainbowTable {
    table: HashMap<String, String>,
}

impl RainbowTable {
    pub fn new() -> Self {
        RainbowTable {
            table: HashMap::new(),
        }
    }
    
    pub fn generate(&mut self, wordlist: &[String], hash_type: &str) {
        println!("[RAINBOW] Generating rainbow table for {} entries", wordlist.len());
        
        for word in wordlist {
            let hash = match hash_type {
                "MD5" => HashGenerator::md5(word),
                "SHA-256" => HashGenerator::sha256(word),
                "SHA-512" => HashGenerator::sha512(word),
                _ => continue,
            };
            
            self.table.insert(hash, word.clone());
        }
        
        println!("[RAINBOW] Generated {} hash->plaintext mappings", self.table.len());
    }
    
    pub fn lookup(&self, hash: &str) -> Option<&String> {
        self.table.get(hash)
    }
    
    pub fn save(&self, output_path: &str) -> Result<(), String> {
        let mut file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        
        for (hash, plaintext) in &self.table {
            writeln!(file, "{}:{}", hash, plaintext)
                .map_err(|e| format!("Write failed: {}", e))?;
        }
        
        println!("[RAINBOW] Saved rainbow table to {}", output_path);
        Ok(())
    }
    
    pub fn load(&mut self, input_path: &str) -> Result<(), String> {
        let content = fs::read_to_string(input_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                self.table.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
        
        println!("[RAINBOW] Loaded {} entries from {}", self.table.len(), input_path);
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// BRUTE FORCE GENERATOR
// ────────────────────────────────────────────────────────────────────────────

pub struct BruteForceGenerator {
    charset: String,
}

impl BruteForceGenerator {
    pub fn new(charset: &str) -> Self {
        BruteForceGenerator {
            charset: charset.to_string(),
        }
    }
    
    pub fn alphanumeric() -> Self {
        Self::new("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789")
    }
    
    pub fn lowercase() -> Self {
        Self::new("abcdefghijklmnopqrstuvwxyz")
    }
    
    pub fn numeric() -> Self {
        Self::new("0123456789")
    }
    
    pub fn generate(&self, min_len: usize, max_len: usize, max_results: usize) -> Vec<String> {
        let mut results = Vec::new();
        let chars: Vec<char> = self.charset.chars().collect();
        
        println!("[BRUTE-FORCE] Generating combinations (min:{}, max:{}, limit:{})", 
            min_len, max_len, max_results);
        
        for len in min_len..=max_len {
            if results.len() >= max_results {
                break;
            }
            
            let mut indices = vec![0; len];
            
            loop {
                let word: String = indices.iter()
                    .map(|&i| chars[i])
                    .collect();
                
                results.push(word);
                
                if results.len() >= max_results {
                    break;
                }
                
                let mut pos = len - 1;
                loop {
                    indices[pos] += 1;
                    if indices[pos] < chars.len() {
                        break;
                    }
                    indices[pos] = 0;
                    if pos == 0 {
                        break;
                    }
                    pos -= 1;
                }
                
                if pos == 0 && indices[0] == 0 && len > min_len {
                    break;
                }
                
                if indices[0] >= chars.len() {
                    break;
                }
            }
        }
        
        println!("[BRUTE-FORCE] Generated {} combinations", results.len());
        results
    }
}
