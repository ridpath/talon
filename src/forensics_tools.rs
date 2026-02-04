use std::fs::{self, File};
use std::io::{Read, Write, Seek, SeekFrom, BufReader, BufRead};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::process::Command;

pub struct FileCarver {
    signatures: HashMap<&'static str, FileSignature>,
}

#[derive(Clone)]
pub struct FileSignature {
    pub magic: Vec<u8>,
    pub extension: &'static str,
    pub footer: Option<Vec<u8>>,
}

impl FileCarver {
    pub fn new() -> Self {
        let mut signatures = HashMap::new();
        
        signatures.insert("JPEG", FileSignature {
            magic: vec![0xFF, 0xD8, 0xFF],
            extension: "jpg",
            footer: Some(vec![0xFF, 0xD9]),
        });
        
        signatures.insert("PNG", FileSignature {
            magic: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
            extension: "png",
            footer: Some(vec![0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82]),
        });
        
        signatures.insert("GIF87a", FileSignature {
            magic: vec![0x47, 0x49, 0x46, 0x38, 0x37, 0x61],
            extension: "gif",
            footer: Some(vec![0x00, 0x3B]),
        });
        
        signatures.insert("GIF89a", FileSignature {
            magic: vec![0x47, 0x49, 0x46, 0x38, 0x39, 0x61],
            extension: "gif",
            footer: Some(vec![0x00, 0x3B]),
        });
        
        signatures.insert("PDF", FileSignature {
            magic: vec![0x25, 0x50, 0x44, 0x46],
            extension: "pdf",
            footer: Some(vec![0x25, 0x25, 0x45, 0x4F, 0x46]),
        });
        
        signatures.insert("ZIP", FileSignature {
            magic: vec![0x50, 0x4B, 0x03, 0x04],
            extension: "zip",
            footer: None,
        });
        
        signatures.insert("RAR", FileSignature {
            magic: vec![0x52, 0x61, 0x72, 0x21, 0x1A, 0x07],
            extension: "rar",
            footer: None,
        });
        
        signatures.insert("7Z", FileSignature {
            magic: vec![0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C],
            extension: "7z",
            footer: None,
        });
        
        signatures.insert("TAR", FileSignature {
            magic: vec![0x75, 0x73, 0x74, 0x61, 0x72],
            extension: "tar",
            footer: None,
        });
        
        signatures.insert("GZIP", FileSignature {
            magic: vec![0x1F, 0x8B, 0x08],
            extension: "gz",
            footer: None,
        });
        
        signatures.insert("BMP", FileSignature {
            magic: vec![0x42, 0x4D],
            extension: "bmp",
            footer: None,
        });
        
        signatures.insert("EXE", FileSignature {
            magic: vec![0x4D, 0x5A],
            extension: "exe",
            footer: None,
        });
        
        signatures.insert("ELF", FileSignature {
            magic: vec![0x7F, 0x45, 0x4C, 0x46],
            extension: "elf",
            footer: None,
        });
        
        signatures.insert("DOCX", FileSignature {
            magic: vec![0x50, 0x4B, 0x03, 0x04, 0x14, 0x00, 0x06, 0x00],
            extension: "docx",
            footer: None,
        });
        
        signatures.insert("MP4", FileSignature {
            magic: vec![0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70],
            extension: "mp4",
            footer: None,
        });
        
        signatures.insert("AVI", FileSignature {
            magic: vec![0x52, 0x49, 0x46, 0x46],
            extension: "avi",
            footer: None,
        });
        
        FileCarver { signatures }
    }
    
    pub fn carve(&self, image_path: &str, output_dir: &str) -> Result<Vec<PathBuf>, String> {
        log::info!("Starting file carving on {}", image_path);
        
        let data = fs::read(image_path)
            .map_err(|e| format!("Failed to read image: {}", e))?;
        
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
        
        let mut carved_files = Vec::new();
        let mut file_counter = HashMap::new();
        
        for (file_type, sig) in &self.signatures {
            for (offset, _) in data.windows(sig.magic.len())
                .enumerate()
                .filter(|(_, window)| window == sig.magic.as_slice()) {
                
                let file_num = *file_counter.get(file_type).unwrap_or(&0) + 1;
                file_counter.insert(*file_type, file_num);
                
                let output_path = PathBuf::from(output_dir)
                    .join(format!("carved_{}_{}.{}", file_type, file_num, sig.extension));
                
                let end = if let Some(footer) = &sig.footer {
                    self.find_footer(&data, offset, footer).unwrap_or(offset + 10 * 1024 * 1024)
                } else {
                    std::cmp::min(offset + 10 * 1024 * 1024, data.len())
                };
                
                let carved_data = &data[offset..end];
                
                fs::write(&output_path, carved_data)
                    .map_err(|e| format!("Failed to write carved file: {}", e))?;
                
                log::info!("Carved {} at offset 0x{:x} -> {}", file_type, offset, output_path.display());
                
                carved_files.push(output_path);
            }
        }
        
        log::info!("Total files carved: {}", carved_files.len());
        Ok(carved_files)
    }
    
    fn find_footer(&self, data: &[u8], start: usize, footer: &[u8]) -> Option<usize> {
        for i in start..data.len().saturating_sub(footer.len()) {
            if &data[i..i + footer.len()] == footer {
                return Some(i + footer.len());
            }
        }
        None
    }
}

impl Default for FileCarver {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DeletedFileRecovery;

impl DeletedFileRecovery {
    pub fn scan_for_deleted(disk_image: &str) -> Result<Vec<(usize, String)>, String> {
        log::info!("Scanning {} for deleted files", disk_image);
        
        let data = fs::read(disk_image)
            .map_err(|e| format!("Failed to read disk image: {}", e))?;
        
        let mut potential_files = Vec::new();
        let carver = FileCarver::new();
        
        for (file_type, sig) in &carver.signatures {
            for (offset, _) in data.windows(sig.magic.len())
                .enumerate()
                .filter(|(_, window)| window == sig.magic.as_slice()) {
                
                potential_files.push((offset, file_type.to_string()));
                log::debug!("Found potential {} at offset 0x{:x}", file_type, offset);
            }
        }
        
        Ok(potential_files)
    }
    
    pub fn recover_file(disk_image: &str, offset: usize, size: usize, output: &str) -> Result<(), String> {
        let mut file = File::open(disk_image)
            .map_err(|e| format!("Failed to open disk image: {}", e))?;
        
        file.seek(SeekFrom::Start(offset as u64))
            .map_err(|e| format!("Seek failed: {}", e))?;
        
        let mut buffer = vec![0u8; size];
        file.read_exact(&mut buffer)
            .map_err(|e| format!("Read failed: {}", e))?;
        
        fs::write(output, buffer)
            .map_err(|e| format!("Write failed: {}", e))?;
        
        log::info!("Recovered file to {}", output);
        Ok(())
    }
}

pub struct TimelineAnalyzer;

impl TimelineAnalyzer {
    pub fn analyze_directory(dir_path: &str) -> Result<Vec<FileTimestamp>, String> {
        log::info!("Analyzing directory: {}", dir_path);
        
        let mut timestamps = Vec::new();
        
        for entry in walkdir::WalkDir::new(dir_path) {
            let entry = entry.map_err(|e| format!("Walk error: {}", e))?;
            
            if entry.file_type().is_file() {
                let metadata = entry.metadata()
                    .map_err(|e| format!("Metadata error: {}", e))?;
                
                let created = metadata.created().ok();
                let modified = metadata.modified().ok();
                let accessed = metadata.accessed().ok();
                
                timestamps.push(FileTimestamp {
                    path: entry.path().to_path_buf(),
                    created,
                    modified,
                    accessed,
                    size: metadata.len(),
                });
            }
        }
        
        timestamps.sort_by(|a, b| {
            b.modified.unwrap_or(std::time::UNIX_EPOCH)
                .cmp(&a.modified.unwrap_or(std::time::UNIX_EPOCH))
        });
        
        log::info!("Found {} files", timestamps.len());
        Ok(timestamps)
    }
    
    pub fn find_suspicious_timestamps(timestamps: &[FileTimestamp]) -> Vec<&FileTimestamp> {
        let mut suspicious = Vec::new();
        
        for ts in timestamps {
            if let (Some(created), Some(modified)) = (ts.created, ts.modified) {
                if modified < created {
                    suspicious.push(ts);
                    log::warn!("Suspicious: Modified before created: {}", ts.path.display());
                }
            }
            
            if let Some(modified) = ts.modified {
                let now = std::time::SystemTime::now();
                if let Ok(duration) = now.duration_since(modified) {
                    if duration.as_secs() < 3600 {
                        suspicious.push(ts);
                        log::warn!("Recently modified (< 1 hour): {}", ts.path.display());
                    }
                }
            }
        }
        
        suspicious
    }
    
    pub fn generate_timeline_report(timestamps: &[FileTimestamp]) -> String {
        let mut report = String::from("Timeline Analysis Report\n");
        report.push_str(&format!("Total Files: {}\n\n", timestamps.len()));
        
        for ts in timestamps.iter().take(50) {
            report.push_str(&format!("File: {}\n", ts.path.display()));
            if let Some(created) = ts.created {
                report.push_str(&format!("  Created:  {:?}\n", created));
            }
            if let Some(modified) = ts.modified {
                report.push_str(&format!("  Modified: {:?}\n", modified));
            }
            if let Some(accessed) = ts.accessed {
                report.push_str(&format!("  Accessed: {:?}\n", accessed));
            }
            report.push_str(&format!("  Size:     {} bytes\n\n", ts.size));
        }
        
        report
    }
}

#[derive(Debug, Clone)]
pub struct FileTimestamp {
    pub path: PathBuf,
    pub created: Option<std::time::SystemTime>,
    pub modified: Option<std::time::SystemTime>,
    pub accessed: Option<std::time::SystemTime>,
    pub size: u64,
}

pub struct SlackSpaceAnalyzer;

impl SlackSpaceAnalyzer {
    pub fn analyze_file(file_path: &str, block_size: usize) -> Result<Vec<u8>, String> {
        let metadata = fs::metadata(file_path)
            .map_err(|e| format!("Failed to get metadata: {}", e))?;
        
        let file_size = metadata.len() as usize;
        let blocks_needed = (file_size + block_size - 1) / block_size;
        let allocated_size = blocks_needed * block_size;
        let slack_size = allocated_size - file_size;
        
        log::info!("File: {}, Actual: {} bytes, Allocated: {} bytes, Slack: {} bytes", 
            file_path, file_size, allocated_size, slack_size);
        
        let mut file = File::open(file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let mut buffer = vec![0u8; allocated_size];
        let read = file.read(&mut buffer)
            .map_err(|e| format!("Read failed: {}", e))?;
        
        let slack_data = buffer[read..].to_vec();
        
        if slack_data.iter().any(|&b| b != 0) {
            log::warn!("Non-zero data found in slack space");
        }
        
        Ok(slack_data)
    }
    
    pub fn scan_directory_slack(dir_path: &str, block_size: usize) -> Result<HashMap<PathBuf, Vec<u8>>, String> {
        let mut results = HashMap::new();
        
        for entry in walkdir::WalkDir::new(dir_path) {
            let entry = entry.map_err(|e| format!("Walk error: {}", e))?;
            
            if entry.file_type().is_file() {
                if let Ok(slack) = Self::analyze_file(entry.path().to_str().unwrap(), block_size) {
                    if slack.iter().any(|&b| b != 0) {
                        results.insert(entry.path().to_path_buf(), slack);
                    }
                }
            }
        }
        
        Ok(results)
    }
}

pub struct MemoryDumpAnalyzer;

impl MemoryDumpAnalyzer {
    pub fn search_patterns(dump_path: &str, patterns: &[&str]) -> Result<HashMap<String, Vec<usize>>, String> {
        log::info!("Analyzing memory dump: {}", dump_path);
        
        let data = fs::read(dump_path)
            .map_err(|e| format!("Failed to read dump: {}", e))?;
        
        let mut results = HashMap::new();
        
        for pattern in patterns {
            let pattern_bytes = pattern.as_bytes();
            let mut offsets = Vec::new();
            
            for (i, window) in data.windows(pattern_bytes.len()).enumerate() {
                if window == pattern_bytes {
                    offsets.push(i);
                }
            }
            
            if !offsets.is_empty() {
                log::info!("Found '{}' at {} locations", pattern, offsets.len());
                results.insert(pattern.to_string(), offsets);
            }
        }
        
        Ok(results)
    }
    
    pub fn extract_strings(dump_path: &str, min_length: usize) -> Result<Vec<(usize, String)>, String> {
        let data = fs::read(dump_path)
            .map_err(|e| format!("Failed to read dump: {}", e))?;
        
        let mut strings = Vec::new();
        let mut current = String::new();
        let mut start_offset = 0;
        
        for (i, &byte) in data.iter().enumerate() {
            if byte.is_ascii_graphic() || byte == b' ' {
                if current.is_empty() {
                    start_offset = i;
                }
                current.push(byte as char);
            } else {
                if current.len() >= min_length {
                    strings.push((start_offset, current.clone()));
                }
                current.clear();
            }
        }
        
        log::info!("Extracted {} strings (min length: {})", strings.len(), min_length);
        Ok(strings)
    }
    
    pub fn find_urls(dump_path: &str) -> Result<Vec<String>, String> {
        let strings = Self::extract_strings(dump_path, 10)?;
        
        let url_regex = regex::Regex::new(r"https?://[^\s<>'\"]+")
            .map_err(|e| format!("Regex error: {}", e))?;
        
        let mut urls = Vec::new();
        for (_, s) in &strings {
            for cap in url_regex.find_iter(s) {
                urls.push(cap.as_str().to_string());
            }
        }
        
        urls.sort();
        urls.dedup();
        
        log::info!("Found unique URLs: {}", urls.len());
        Ok(urls)
    }
    
    pub fn find_crypto_keys(dump_path: &str) -> Result<Vec<(usize, String)>, String> {
        let data = fs::read(dump_path)
            .map_err(|e| format!("Failed to read dump: {}", e))?;
        
        let mut keys = Vec::new();
        
        let pem_start = b"-----BEGIN";
        for (i, window) in data.windows(pem_start.len()).enumerate() {
            if window == pem_start {
                if let Some(end) = data[i..].windows(5).position(|w| w == b"-----") {
                    if let Ok(pem_block) = String::from_utf8(data[i..i+end+50].to_vec()) {
                        keys.push((i, pem_block));
                    }
                }
            }
        }
        
        log::info!("Found {} potential crypto keys", keys.len());
        Ok(keys)
    }
    
    pub fn find_credentials(dump_path: &str) -> Result<Vec<(usize, String)>, String> {
        let strings = Self::extract_strings(dump_path, 5)?;
        let mut credentials = Vec::new();
        
        let cred_patterns = vec![
            regex::Regex::new(r"password\s*[:=]\s*\S+").unwrap(),
            regex::Regex::new(r"api[_-]?key\s*[:=]\s*\S+").unwrap(),
            regex::Regex::new(r"token\s*[:=]\s*\S+").unwrap(),
            regex::Regex::new(r"secret\s*[:=]\s*\S+").unwrap(),
        ];
        
        for (offset, s) in strings {
            for pattern in &cred_patterns {
                if let Some(mat) = pattern.find(&s) {
                    credentials.push((offset, mat.as_str().to_string()));
                }
            }
        }
        
        log::info!("Found {} potential credentials", credentials.len());
        Ok(credentials)
    }
}

pub struct PCAPAnalyzer;

impl PCAPAnalyzer {
    pub fn analyze_with_tshark(pcap_path: &str) -> Result<String, String> {
        log::info!("Analyzing PCAP with tshark: {}", pcap_path);
        
        let output = Command::new("tshark")
            .arg("-r")
            .arg(pcap_path)
            .arg("-q")
            .arg("-z")
            .arg("conv,tcp")
            .arg("-z")
            .arg("conv,udp")
            .arg("-z")
            .arg("http,tree")
            .output();
        
        match output {
            Ok(out) => {
                if out.status.success() {
                    Ok(String::from_utf8_lossy(&out.stdout).to_string())
                } else {
                    Err(format!("tshark failed: {}", String::from_utf8_lossy(&out.stderr)))
                }
            }
            Err(_) => {
                log::warn!("tshark not found, using basic analysis");
                Self::basic_analysis(pcap_path)
            }
        }
    }
    
    fn basic_analysis(pcap_path: &str) -> Result<String, String> {
        let data = fs::read(pcap_path)
            .map_err(|e| format!("Failed to read PCAP: {}", e))?;
        
        if !data.starts_with(&[0xd4, 0xc3, 0xb2, 0xa1]) && 
           !data.starts_with(&[0xa1, 0xb2, 0xc3, 0xd4]) {
            return Err(String::from("Not a valid PCAP file"));
        }
        
        log::info!("PCAP file size in bytes: {}", data.len());
        Ok(format!("[PCAP] File size: {} bytes\nUse tshark or wireshark for full network analysis", data.len()))
    }
    
    pub fn extract_http_objects(pcap_path: &str, output_dir: &str) -> Result<(), String> {
        log::info!("Extracting HTTP objects from PCAP: {}", pcap_path);
        
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output dir: {}", e))?;
        
        let export_arg = format!("http,{}", output_dir);
        let output = Command::new("tshark")
            .arg("-r")
            .arg(pcap_path)
            .arg("--export-objects")
            .arg(&export_arg)
            .output()
            .map_err(|e| format!("tshark execution failed: {}", e))?;
        
        if output.status.success() {
            log::info!("HTTP objects extracted to {}", output_dir);
            Ok(())
        } else {
            Err(format!("Extraction failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }
    
    pub fn extract_dns_queries(pcap_path: &str) -> Result<Vec<String>, String> {
        let output = Command::new("tshark")
            .arg("-r")
            .arg(pcap_path)
            .arg("-Y")
            .arg("dns")
            .arg("-T")
            .arg("fields")
            .arg("-e")
            .arg("dns.qry.name")
            .output()
            .map_err(|e| format!("tshark failed: {}", e))?;
        
        if output.status.success() {
            let queries: Vec<String> = String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .map(|s| s.to_string())
                .collect();
            
            log::info!("Found {} DNS queries", queries.len());
            Ok(queries)
        } else {
            Err(format!("DNS extraction failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }
    
    pub fn extract_credentials_from_pcap(pcap_path: &str) -> Result<Vec<String>, String> {
        let output = Command::new("tshark")
            .arg("-r")
            .arg(pcap_path)
            .arg("-Y")
            .arg("http.request.method == \"POST\"")
            .arg("-T")
            .arg("fields")
            .arg("-e")
            .arg("http.file_data")
            .output()
            .map_err(|e| format!("tshark failed: {}", e))?;
        
        if output.status.success() {
            let creds: Vec<String> = String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter(|l| l.contains("password") || l.contains("user"))
                .map(|s| s.to_string())
                .collect();
            
            log::info!("Found {} potential credentials in HTTP POST data", creds.len());
            Ok(creds)
        } else {
            Err(format!("Credential extraction failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }
}

pub struct RegistryAnalyzer;

impl RegistryAnalyzer {
    pub fn analyze_hive(hive_path: &str) -> Result<String, String> {
        log::info!("Analyzing registry hive: {}", hive_path);
        
        let output = Command::new("reglookup")
            .arg(hive_path)
            .output();
        
        match output {
            Ok(out) => {
                if out.status.success() {
                    let result = String::from_utf8_lossy(&out.stdout).to_string();
                    log::info!("Found registry entry count: {}", result.lines().count());
                    Ok(result)
                } else {
                    Err(format!("reglookup failed: {}", String::from_utf8_lossy(&out.stderr)))
                }
            }
            Err(_) => {
                log::warn!("reglookup not found, install with apt-get install reglookup");
                Err(String::from("reglookup not found"))
            }
        }
    }
    
    pub fn extract_run_keys(hive_path: &str) -> Result<Vec<String>, String> {
        let result = Self::analyze_hive(hive_path)?;
        
        let run_keys: Vec<String> = result.lines()
            .filter(|l| l.contains("Run") || l.contains("RunOnce"))
            .map(|s| s.to_string())
            .collect();
        
        log::info!("Found {} Run/RunOnce keys", run_keys.len());
        Ok(run_keys)
    }
    
    pub fn extract_recent_docs(hive_path: &str) -> Result<Vec<String>, String> {
        let result = Self::analyze_hive(hive_path)?;
        
        let recent: Vec<String> = result.lines()
            .filter(|l| l.contains("RecentDocs") || l.contains("Recent"))
            .map(|s| s.to_string())
            .collect();
        
        log::info!("Found {} recent documents entries", recent.len());
        Ok(recent)
    }
}

pub struct AntiForensicsDetector;

impl AntiForensicsDetector {
    pub fn detect(file_or_dir: &str) -> Result<Vec<String>, String> {
        let mut findings = Vec::new();
        
        log::info!("Scanning for anti-forensic techniques");
        
        let timestamps = TimelineAnalyzer::analyze_directory(file_or_dir)?;
        let suspicious = TimelineAnalyzer::find_suspicious_timestamps(&timestamps);
        
        if !suspicious.is_empty() {
            findings.push(format!("Timestamp manipulation detected ({} files)", suspicious.len()));
        }
        
        for entry in walkdir::WalkDir::new(file_or_dir) {
            let entry = entry.map_err(|e| format!("Walk error: {}", e))?;
            
            let name = entry.file_name().to_string_lossy();
            
            if name.starts_with('.') && entry.file_type().is_file() {
                findings.push(format!("Hidden file: {}", entry.path().display()));
            }
            
            if entry.file_type().is_file() {
                let metadata = entry.metadata()
                    .map_err(|e| format!("Metadata error: {}", e))?;
                
                if metadata.len() == 0 {
                    findings.push(format!("Empty file (possible wiping): {}", entry.path().display()));
                }
                
                if let Ok(slack) = SlackSpaceAnalyzer::analyze_file(
                    entry.path().to_str().unwrap(), 512
                ) {
                    if slack.iter().any(|&b| b != 0) {
                        findings.push(format!("Non-zero slack space: {}", entry.path().display()));
                    }
                }
            }
        }
        
        log::info!("Found suspicious indicator count: {}", findings.len());
        for finding in &findings {
            log::warn!("Anti-forensics detected: {}", finding);
        }
        
        Ok(findings)
    }
    
    pub fn detect_timestomp(timestamps: &[FileTimestamp]) -> Vec<String> {
        let mut findings = Vec::new();
        
        for ts in timestamps {
            if let (Some(created), Some(modified)) = (ts.created, ts.modified) {
                if modified < created {
                    findings.push(format!("TIMESTOMP: Modified before created - {}", ts.path.display()));
                }
                
                let year_1980 = std::time::UNIX_EPOCH + std::time::Duration::from_secs(315532800);
                if created < year_1980 || modified < year_1980 {
                    findings.push(format!("TIMESTOMP: Suspicious old timestamp - {}", ts.path.display()));
                }
            }
        }
        
        findings
    }
}

pub struct VolatilityIntegration;

impl VolatilityIntegration {
    pub fn run_plugin(dump_path: &str, plugin: &str, profile: &str) -> Result<String, String> {
        log::info!("Running Volatility plugin: {} on {}", plugin, dump_path);
        
        let output = Command::new("volatility")
            .arg("-f")
            .arg(dump_path)
            .arg("--profile")
            .arg(profile)
            .arg(plugin)
            .output()
            .map_err(|e| format!("Volatility execution failed: {}", e))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(format!("Volatility failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }
    
    pub fn list_processes(dump_path: &str, profile: &str) -> Result<Vec<String>, String> {
        let result = Self::run_plugin(dump_path, "pslist", profile)?;
        
        let processes: Vec<String> = result.lines()
            .skip(2)
            .map(|s| s.to_string())
            .collect();
        
        log::info!("Found {} processes", processes.len());
        Ok(processes)
    }
    
    pub fn dump_process_memory(dump_path: &str, profile: &str, pid: u32, output_dir: &str) -> Result<(), String> {
        let pid_str = pid.to_string();
        let output = Command::new("volatility")
            .arg("-f")
            .arg(dump_path)
            .arg("--profile")
            .arg(profile)
            .arg("memdump")
            .arg("-p")
            .arg(&pid_str)
            .arg("-D")
            .arg(output_dir)
            .output()
            .map_err(|e| format!("Volatility execution failed: {}", e))?;
        
        if output.status.success() {
            log::info!("Process {} memory dumped to {}", pid, output_dir);
            Ok(())
        } else {
            Err(format!("Memory dump failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_file_carver_new() {
        let carver = FileCarver::new();
        assert!(!carver.signatures.is_empty());
        assert!(carver.signatures.contains_key("JPEG"));
        assert!(carver.signatures.contains_key("PNG"));
        assert!(carver.signatures.contains_key("ELF"));
        assert!(carver.signatures.contains_key("PDF"));
        assert!(carver.signatures.contains_key("DOCX"));
    }

    #[test]
    fn test_slack_space_analyzer() {
        let test_file = "test_slack_forensics.tmp";
        fs::write(test_file, b"Hello World Test Data").unwrap();
        
        let result = SlackSpaceAnalyzer::analyze_file(test_file, 512);
        assert!(result.is_ok());
        
        let slack = result.unwrap();
        assert!(slack.len() > 0);
        
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_memory_dump_search() {
        let test_dump = "test_dump_forensics.tmp";
        fs::write(test_dump, b"password123 secret admin token12345").unwrap();
        
        let patterns = vec!["password", "admin", "token"];
        let result = MemoryDumpAnalyzer::search_patterns(test_dump, &patterns);
        
        assert!(result.is_ok());
        let results = result.unwrap();
        assert!(results.contains_key("password"));
        assert!(results.contains_key("admin"));
        assert!(results.contains_key("token"));
        
        fs::remove_file(test_dump).ok();
    }

    #[test]
    fn test_string_extraction() {
        let test_dump = "test_strings_forensics.tmp";
        let test_data = b"Hello\x00World\x00TALON\x00Forensics\x00";
        fs::write(test_dump, test_data).unwrap();
        
        let result = MemoryDumpAnalyzer::extract_strings(test_dump, 4);
        assert!(result.is_ok());
        
        let strings = result.unwrap();
        assert!(!strings.is_empty());
        
        fs::remove_file(test_dump).ok();
    }
    
    #[test]
    fn test_url_extraction() {
        let test_dump = "test_urls_forensics.tmp";
        let test_data = b"Check out https://example.com and http://test.org for more info";
        fs::write(test_dump, test_data).unwrap();
        
        let result = MemoryDumpAnalyzer::find_urls(test_dump);
        assert!(result.is_ok());
        
        let urls = result.unwrap();
        assert!(!urls.is_empty());
        assert!(urls.iter().any(|u| u.contains("example.com")));
        
        fs::remove_file(test_dump).ok();
    }
    
    #[test]
    fn test_file_signature_footer() {
        let carver = FileCarver::new();
        let jpeg_sig = carver.signatures.get("JPEG").unwrap();
        assert!(jpeg_sig.footer.is_some());
        
        let png_sig = carver.signatures.get("PNG").unwrap();
        assert!(png_sig.footer.is_some());
    }
    
    #[test]
    fn test_timeline_analyzer() {
        let test_dir = "test_timeline_forensics";
        fs::create_dir_all(test_dir).unwrap();
        fs::write(format!("{}/file1.txt", test_dir), b"test1").unwrap();
        fs::write(format!("{}/file2.txt", test_dir), b"test2").unwrap();
        
        let result = TimelineAnalyzer::analyze_directory(test_dir);
        assert!(result.is_ok());
        
        let timestamps = result.unwrap();
        assert_eq!(timestamps.len(), 2);
        
        fs::remove_dir_all(test_dir).ok();
    }
    
    #[test]
    fn test_deleted_file_recovery_scan() {
        let test_image = "test_image_forensics.tmp";
        let mut data = vec![0u8; 1024];
        data[100..103].copy_from_slice(&[0xFF, 0xD8, 0xFF]);
        data[500..508].copy_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
        fs::write(test_image, data).unwrap();
        
        let result = DeletedFileRecovery::scan_for_deleted(test_image);
        assert!(result.is_ok());
        
        let files = result.unwrap();
        assert!(files.len() >= 2);
        
        fs::remove_file(test_image).ok();
    }
}
