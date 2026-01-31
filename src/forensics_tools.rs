use std::fs::{self, File};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

// Forensics Toolkit - Production Ready
// File carving, deleted file recovery, timeline analysis, etc.

pub struct FileCarver {
    signatures: HashMap<&'static str, Vec<u8>>,
}

impl FileCarver {
    pub fn new() -> Self {
        let mut signatures = HashMap::new();

        signatures.insert("JPEG", vec![0xFF, 0xD8, 0xFF]);
        signatures.insert("PNG", vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
        signatures.insert("GIF", vec![0x47, 0x49, 0x46, 0x38]);
        signatures.insert("PDF", vec![0x25, 0x50, 0x44, 0x46]);
        signatures.insert("ZIP", vec![0x50, 0x4B, 0x03, 0x04]);
        signatures.insert("RAR", vec![0x52, 0x61, 0x72, 0x21]);
        signatures.insert("7Z", vec![0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]);
        signatures.insert("TAR", vec![0x75, 0x73, 0x74, 0x61, 0x72]);
        signatures.insert("GZIP", vec![0x1F, 0x8B]);
        signatures.insert("BMP", vec![0x42, 0x4D]);
        signatures.insert("EXE", vec![0x4D, 0x5A]);
        signatures.insert("ELF", vec![0x7F, 0x45, 0x4C, 0x46]);

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

        for (file_type, signature) in &self.signatures {
            for (offset, _) in data.windows(signature.len())
                .enumerate()
                .filter(|(_, window)| window == signature.as_slice()) {

                let file_num = *file_counter.get(file_type).unwrap_or(&0) + 1;
                file_counter.insert(*file_type, file_num);

                let output_path = PathBuf::from(output_dir)
                    .join(format!("carved_{}_{}.{}", file_type, file_num, file_type.to_lowercase()));

                let max_size = 10 * 1024 * 1024;
                let end = std::cmp::min(offset + max_size, data.len());
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
}

pub struct DeletedFileRecovery;

impl DeletedFileRecovery {
    pub fn scan_for_deleted(disk_image: &str) -> Result<Vec<(usize, String)>, String> {
        log::info!("Scanning {} for deleted files", disk_image);

        let data = fs::read(disk_image)
            .map_err(|e| format!("Failed to read disk image: {}", e))?;

        let mut potential_files = Vec::new();
        let carver = FileCarver::new();

        for (file_type, signature) in &carver.signatures {
            for (offset, _) in data.windows(signature.len())
                .enumerate()
                .filter(|(_, window)| window == signature.as_slice()) {

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
}

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
            log::warn!("Non-zero data found in slack space!");
        }

        Ok(slack_data)
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
        let (_, strings) = Self::extract_strings(dump_path, 10)?
            .into_iter()
            .unzip::<_, _, Vec<_>, Vec<_>>();

        let url_regex = regex::Regex::new(r"https?://[^\s<>'\"]+")
            .map_err(|e| format!("Regex error: {}", e))?;

        let mut urls = Vec::new();
        for s in &strings {
            for cap in url_regex.find_iter(s) {
                urls.push(cap.as_str().to_string());
            }
        }

        urls.sort();
        urls.dedup();

        log::info!("Found unique URLs: {}", urls.len());
        Ok(urls)
    }
}

pub struct PCAPAnalyzer;

impl PCAPAnalyzer {
    pub fn analyze_with_tshark(pcap_path: &str) -> Result<(), String> {
        log::info!("Analyzing PCAP with tshark: {}", pcap_path);

        let output = std::process::Command::new("tshark")
            .args(&vec!["-r", pcap_path, "-q", "-z", "conv,tcp", "-z", "conv,udp", "-z", "http,tree"])
            .output();

        match output {
            Ok(out) => {
                println!("{}", String::from_utf8_lossy(&out.stdout));
                Ok(())
            }
            Err(_) => {
                log::warn!("tshark not found using basic analysis");
                Self::basic_analysis(pcap_path)
            }
        }
    }

    fn basic_analysis(pcap_path: &str) -> Result<(), String> {
        let data = fs::read(pcap_path)
            .map_err(|e| format!("Failed to read PCAP: {}", e))?;

        if !data.starts_with(&[0xd4, 0xc3, 0xb2, 0xa1]) &&
           !data.starts_with(&[0xa1, 0xb2, 0xc3, 0xd4]) {
            return Err(String::from("Not a valid PCAP file"));
        }

        log::info!("PCAP file size in bytes: {}", data.len());
        println!("[PCAP] Use tshark or wireshark for full network analysis");
        Ok(())
    }

    pub fn extract_http_objects(pcap_path: &str, output_dir: &str) -> Result<(), String> {
        log::info!("Extracting HTTP objects from PCAP: {}", pcap_path);

        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output dir: {}", e))?;

        let output = std::process::Command::new("tshark")
            .args(&vec!["-r", pcap_path, "--export-objects", &format!("http,{}", output_dir)])
            .output()
            .map_err(|e| format!("tshark execution failed: {}", e))?;

        if output.status.success() {
            println!("[PCAP] HTTP objects extracted to {}", output_dir);
            Ok(())
        } else {
            Err(format!("Extraction failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }
}

pub struct RegistryAnalyzer;

impl RegistryAnalyzer {
    pub fn analyze_hive(hive_path: &str) -> Result<(), String> {
        log::info!("Analyzing registry hive: {}", hive_path);

        let output = std::process::Command::new("reglookup")
            .arg(hive_path)
            .output();

        match output {
            Ok(out) => {
                let result = String::from_utf8_lossy(&out.stdout);
                log::info!("Found registry entry count: {}", result.lines().count());
                Ok(())
            }
            Err(_) => {
                log::warn!("reglookup not found install with apt-get");
                Ok(())
            }
        }
    }
}

pub struct AntiForensicsDetector;

impl AntiForensicsDetector {
    pub fn detect(file_or_dir: &str) -> Result<Vec<String>, String> {
        let mut findings = Vec::new();

        log::info!("Scanning for anti-forensic techniques...");

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
            }
        }

        log::info!("Found suspicious indicator count: {}", findings.len());
        for finding in &findings {
            log::warn!("Anti-forensics detected: {}", finding);
        }

        Ok(findings)
    }
}
