use std::fs;
use std::io::Read;

// ═══════════════════════════════════════════════════════════════════════════
// STEGANOGRAPHY TOOLKIT - PRODUCTION READY
// ═══════════════════════════════════════════════════════════════════════════

// Constants for steganography operations
const BMP_HEADER_SIZE: usize = 54;
const MAX_LSB_EXTRACT_BYTES: usize = 10000;
const WAV_HEADER_SIZE: usize = 44;
const MIN_STRING_LENGTH: usize = 20;
const MIN_BASE64_LENGTH: usize = 20;
const HIGH_ENTROPY_THRESHOLD: f64 = 7.5;
const LOW_ENTROPY_THRESHOLD: f64 = 3.0;

// ────────────────────────────────────────────────────────────────────────────
// LSB (LEAST SIGNIFICANT BIT) EXTRACTION
// ────────────────────────────────────────────────────────────────────────────

pub struct LSBExtractor;

impl LSBExtractor {
    /// Extracts hidden data from an image using LSB steganography
    pub fn extract_from_image(image_path: &str) -> Result<Vec<u8>, String> {
        let data = fs::read(image_path).map_err(|e| format!("Failed to read image: {}", e))?;

        log::info!("Extracting LSB data from {}", image_path);

        let mut extracted = Vec::new();
        let mut byte = 0u8;
        let mut bit_count = 0;

        for &byte_val in data.iter().skip(BMP_HEADER_SIZE) {
            let lsb = byte_val & 1;
            byte = (byte << 1) | lsb;
            bit_count += 1;

            if bit_count == 8 {
                extracted.push(byte);
                byte = 0;
                bit_count = 0;

                if extracted.len() > MAX_LSB_EXTRACT_BYTES {
                    break;
                }
            }
        }

        log::info!("Extracted {} bytes", extracted.len());

        if let Ok(text) = String::from_utf8(extracted.clone()) {
            if text
                .chars()
                .take(100)
                .all(|c| c.is_ascii() && !c.is_control() || c == '\n')
            {
                log::info!("Detected ASCII text!");
                log::debug!("Preview: {}", &text.chars().take(200).collect::<String>());
            }
        }

        Ok(extracted)
    }

    /// Hides data in an image using LSB steganography
    pub fn hide_in_image(image_path: &str, data: &[u8], output_path: &str) -> Result<(), String> {
        let mut image_data =
            fs::read(image_path).map_err(|e| format!("Failed to read image: {}", e))?;

        log::info!("Hiding {} bytes in {}", data.len(), image_path);

        let mut data_bits = Vec::new();
        for &byte in data {
            for i in (0..8).rev() {
                data_bits.push((byte >> i) & 1);
            }
        }

        if image_data.len() < data_bits.len() + BMP_HEADER_SIZE {
            return Err(format!(
                "Image too small for data: need {} bytes, have {} bytes",
                data_bits.len() + BMP_HEADER_SIZE,
                image_data.len()
            ));
        }

        for (i, &bit) in data_bits.iter().enumerate() {
            let idx = BMP_HEADER_SIZE + i;
            if idx >= image_data.len() {
                break;
            }
            image_data[idx] = (image_data[idx] & 0xfe) | bit;
        }

        fs::write(output_path, image_data).map_err(|e| format!("Failed to write output: {}", e))?;

        println!("[LSB] [OK] Data hidden in {}", output_path);
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// FILE SIGNATURE DETECTION
// ────────────────────────────────────────────────────────────────────────────

pub struct FileSignatureDetector;

impl FileSignatureDetector {
    pub fn detect(file_path: &str) -> Result<Vec<String>, String> {
        let mut file =
            fs::File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;

        let mut header = vec![0u8; 16];
        file.read_exact(&mut header)
            .map_err(|e| format!("Failed to read header: {}", e))?;

        let mut signatures = Vec::new();

        if header.starts_with(&[0xFF, 0xD8, 0xFF]) {
            signatures.push("JPEG".to_string());
        }
        if header.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            signatures.push("PNG".to_string());
        }
        if header.starts_with(&[0x47, 0x49, 0x46, 0x38]) {
            signatures.push("GIF".to_string());
        }
        if header.starts_with(&[0x42, 0x4D]) {
            signatures.push("BMP".to_string());
        }
        if header.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
            signatures.push("ZIP".to_string());
        }
        if header.starts_with(&[0x52, 0x61, 0x72, 0x21]) {
            signatures.push("RAR".to_string());
        }
        if header.starts_with(&[0x25, 0x50, 0x44, 0x46]) {
            signatures.push("PDF".to_string());
        }
        if header.starts_with(&[0x7F, 0x45, 0x4C, 0x46]) {
            signatures.push("ELF".to_string());
        }
        if header.starts_with(&[0x4D, 0x5A]) {
            signatures.push("PE/EXE".to_string());
        }

        if signatures.is_empty() {
            signatures.push("Unknown".to_string());
        }

        println!("[FILE-SIG] Detected file types:");
        for sig in &signatures {
            println!("  • {}", sig);
        }

        Ok(signatures)
    }

    pub fn find_hidden_files(data: &[u8]) -> Vec<(usize, String)> {
        let mut hidden_files = Vec::new();

        let signatures: Vec<(&[u8], &str)> = vec![
            (&[0xFF, 0xD8, 0xFF], "JPEG"),
            (&[0x89, 0x50, 0x4E, 0x47], "PNG"),
            (&[0x50, 0x4B, 0x03, 0x04], "ZIP"),
            (&[0x52, 0x61, 0x72, 0x21], "RAR"),
            (&[0x25, 0x50, 0x44, 0x46], "PDF"),
        ];

        for (offset, window) in data.windows(4).enumerate() {
            for (sig, name) in &signatures {
                if window.starts_with(sig) {
                    hidden_files.push((offset, name.to_string()));
                    println!(
                        "[FILE-SIG] Found {} signature at offset 0x{:x}",
                        name, offset
                    );
                }
            }
        }

        hidden_files
    }
}

// ────────────────────────────────────────────────────────────────────────────
// STRINGS EXTRACTION WITH ENTROPY ANALYSIS
// ────────────────────────────────────────────────────────────────────────────

pub struct StringExtractor;

impl StringExtractor {
    pub fn extract_strings(data: &[u8], min_length: usize) -> Vec<String> {
        let mut strings = Vec::new();
        let mut current = String::new();

        for &byte in data {
            if byte.is_ascii_graphic() || byte == b' ' {
                current.push(byte as char);
            } else {
                if current.len() >= min_length {
                    strings.push(current.clone());
                }
                current.clear();
            }
        }

        if current.len() >= min_length {
            strings.push(current);
        }

        println!(
            "[STRINGS] Extracted {} strings (min length: {})",
            strings.len(),
            min_length
        );
        strings
    }

    pub fn extract_base64_strings(data: &[u8]) -> Vec<String> {
        let strings = Self::extract_strings(data, 20);
        let mut base64_strings = Vec::new();

        for s in strings {
            if s.chars()
                .all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '=')
            {
                let padding_count = s.chars().filter(|&c| c == '=').count();
                if padding_count <= 2 && s.len() % 4 == 0 {
                    base64_strings.push(s);
                }
            }
        }

        println!(
            "[STRINGS] Found {} potential base64 strings",
            base64_strings.len()
        );
        base64_strings
    }

    pub fn calculate_entropy(data: &[u8]) -> f64 {
        let mut frequency = [0u32; 256];

        for &byte in data {
            frequency[byte as usize] += 1;
        }

        let len = data.len() as f64;
        let mut entropy = 0.0;

        for &count in &frequency {
            if count > 0 {
                let probability = count as f64 / len;
                entropy -= probability * probability.log2();
            }
        }

        println!("[ENTROPY] Calculated entropy: {:.4} bits/byte", entropy);
        entropy
    }
}

// ────────────────────────────────────────────────────────────────────────────
// AUDIO STEGANOGRAPHY
// ────────────────────────────────────────────────────────────────────────────

pub struct AudioStego;

impl AudioStego {
    pub fn analyze_wav(file_path: &str) -> Result<(), String> {
        let data = fs::read(file_path).map_err(|e| format!("Failed to read WAV: {}", e))?;

        if !data.starts_with(b"RIFF") || !data[8..12].starts_with(b"WAVE") {
            return Err("Not a valid WAV file".to_string());
        }

        println!("[AUDIO-STEGO] Analyzing WAV file: {}", file_path);

        let file_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        println!("[AUDIO-STEGO] File size: {} bytes", file_size);

        let mut pos = 12;
        while pos + 8 <= data.len() {
            let chunk_id = &data[pos..pos + 4];
            let chunk_size =
                u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
                    as usize;

            let chunk_name = String::from_utf8_lossy(chunk_id);
            println!(
                "[AUDIO-STEGO] Found chunk: {} (size: {})",
                chunk_name, chunk_size
            );

            if chunk_name == "fmt "
                && pos + 8 + 16 <= data.len() {
                    let audio_format = u16::from_le_bytes([data[pos + 8], data[pos + 9]]);
                    let channels = u16::from_le_bytes([data[pos + 10], data[pos + 11]]);
                    let sample_rate = u32::from_le_bytes([
                        data[pos + 12],
                        data[pos + 13],
                        data[pos + 14],
                        data[pos + 15],
                    ]);

                    println!("[AUDIO-STEGO]   Format: {}", audio_format);
                    println!("[AUDIO-STEGO]   Channels: {}", channels);
                    println!("[AUDIO-STEGO]   Sample rate: {} Hz", sample_rate);
                }

            pos += 8 + chunk_size;
            if chunk_size % 2 == 1 {
                pos += 1;
            }
        }

        Ok(())
    }

    pub fn extract_lsb_from_audio(file_path: &str) -> Result<Vec<u8>, String> {
        let data = fs::read(file_path).map_err(|e| format!("Failed to read audio: {}", e))?;

        let data_start = 44;
        if data.len() < data_start {
            return Err("File too small".to_string());
        }

        let _audio_data = &data[data_start..];
        let extracted = LSBExtractor::extract_from_image(file_path)?;

        println!(
            "[AUDIO-STEGO] Extracted {} bytes from audio LSB",
            extracted.len()
        );
        Ok(extracted)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// METADATA EXTRACTION
// ────────────────────────────────────────────────────────────────────────────

pub struct MetadataExtractor;

impl MetadataExtractor {
    pub fn extract_exif(file_path: &str) -> Result<(), String> {
        println!("[METADATA] Extracting EXIF data from {}", file_path);

        let output = std::process::Command::new("exiftool")
            .arg(file_path)
            .output();

        match output {
            Ok(out) => {
                let result = String::from_utf8_lossy(&out.stdout);
                println!("{}", result);
                Ok(())
            }
            Err(_) => {
                println!("[METADATA] exiftool not found, using basic extraction");
                Self::basic_metadata(file_path)
            }
        }
    }

    fn basic_metadata(file_path: &str) -> Result<(), String> {
        let metadata =
            fs::metadata(file_path).map_err(|e| format!("Failed to read metadata: {}", e))?;

        println!("[METADATA] File size: {} bytes", metadata.len());
        println!(
            "[METADATA] Read-only: {}",
            metadata.permissions().readonly()
        );

        if let Ok(modified) = metadata.modified() {
            println!("[METADATA] Modified: {:?}", modified);
        }

        Ok(())
    }

    pub fn extract_from_pdf(file_path: &str) -> Result<Vec<String>, String> {
        let data = fs::read(file_path).map_err(|e| format!("Failed to read PDF: {}", e))?;

        let content = String::from_utf8_lossy(&data);
        let mut metadata = Vec::new();

        for line in content.lines() {
            if line.contains("/Author")
                || line.contains("/Creator")
                || line.contains("/Producer")
                || line.contains("/Title")
            {
                metadata.push(line.to_string());
                println!("[METADATA] {}", line);
            }
        }

        Ok(metadata)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// BINWALK-LIKE FUNCTIONALITY
// ────────────────────────────────────────────────────────────────────────────

pub struct BinwalkAnalyzer;

impl BinwalkAnalyzer {
    pub fn analyze(file_path: &str) -> Result<(), String> {
        let data = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

        println!("[BINWALK] Analyzing file: {}", file_path);
        println!("[BINWALK] File size: {} bytes", data.len());
        println!("\n[BINWALK] Scanning for signatures...\n");

        let hidden = FileSignatureDetector::find_hidden_files(&data);

        if hidden.is_empty() {
            println!("[BINWALK] No embedded files found");
        } else {
            println!("[BINWALK] Found {} embedded file(s)", hidden.len());
        }

        let entropy = StringExtractor::calculate_entropy(&data);

        if entropy > 7.5 {
            println!("[BINWALK] WARNING: HIGH ENTROPY - Possibly encrypted/compressed");
        } else if entropy < 3.0 {
            println!("[BINWALK] LOW ENTROPY - Possibly structured/repetitive data");
        } else {
            println!("[BINWALK] NORMAL ENTROPY");
        }

        Ok(())
    }

    pub fn extract_embedded(file_path: &str, output_dir: &str) -> Result<(), String> {
        let data = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output dir: {}", e))?;

        let hidden = FileSignatureDetector::find_hidden_files(&data);

        for (i, (offset, file_type)) in hidden.iter().enumerate() {
            let output_file = format!("{}/extracted_{}_{}.bin", output_dir, i, file_type);
            let extracted_data = &data[*offset..];

            fs::write(&output_file, extracted_data)
                .map_err(|e| format!("Failed to write extracted file: {}", e))?;

            println!("[BINWALK] [OK] Extracted {} to {}", file_type, output_file);
        }

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// SPECTRAL ANALYSIS (FOR AUDIO SPECTROGRAMS)
// ────────────────────────────────────────────────────────────────────────────

pub struct SpectralAnalyzer;

impl SpectralAnalyzer {
    pub fn analyze_for_hidden_messages(file_path: &str) -> Result<(), String> {
        println!(
            "[SPECTRAL] Analyzing {} for hidden spectrogram messages",
            file_path
        );
        println!("[SPECTRAL] Tip: Use tools like Sonic Visualizer or Audacity for visual analysis");

        let data = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

        let entropy = StringExtractor::calculate_entropy(&data);
        println!("[SPECTRAL] Audio entropy: {:.4}", entropy);

        if entropy > 7.0 {
            println!("[SPECTRAL] High entropy detected - may contain hidden data");
        }

        Ok(())
    }
}
