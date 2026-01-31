use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

// ═══════════════════════════════════════════════════════════════════════════
// ARCHIVE MANIPULATION TOOLKIT (Cross-Platform)
// ═══════════════════════════════════════════════════════════════════════════

// ────────────────────────────────────────────────────────────────────────────
// ZIP MANIPULATION
// ────────────────────────────────────────────────────────────────────────────

pub struct ZipTools;

impl ZipTools {
    /// Extracts a ZIP archive (cross-platform using native Rust implementation)
    pub fn extract(zip_path: &str, output_dir: &str) -> Result<(), String> {
        // Input validation
        if zip_path.is_empty() || output_dir.is_empty() {
            return Err("ZIP path and output directory cannot be empty".to_string());
        }

        log::info!("Extracting {} to {}", zip_path, output_dir);

        // Use native Rust implementation for cross-platform compatibility
        // External unzip command may not be available on Windows
        Self::extract_native(zip_path, output_dir)
    }

    fn extract_native(zip_path: &str, output_dir: &str) -> Result<(), String> {
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output dir: {}", e))?;

        let file = File::open(zip_path).map_err(|e| format!("Failed to open ZIP: {}", e))?;

        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP: {}", e))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read file {}: {}", i, e))?;

            let outpath = PathBuf::from(output_dir).join(file.name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p)
                        .map_err(|e| format!("Failed to create parent dir: {}", e))?;
                }

                let mut outfile =
                    File::create(&outpath).map_err(|e| format!("Failed to create file: {}", e))?;

                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| format!("Failed to extract file: {}", e))?;
            }

            log::debug!("Extracted: {}", file.name());
        }

        log::info!("Extraction complete");
        Ok(())
    }

    /// Lists contents of a ZIP archive
    pub fn list_contents(zip_path: &str) -> Result<Vec<String>, String> {
        let file = File::open(zip_path).map_err(|e| format!("Failed to open ZIP: {}", e))?;

        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP: {}", e))?;

        let mut contents = Vec::new();

        println!("[ZIP] Contents of {}:", zip_path);
        for i in 0..archive.len() {
            let file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read file {}: {}", i, e))?;

            let name = file.name().to_string();
            let size = file.size();
            let compressed_size = file.compressed_size();

            println!(
                "[ZIP]   {} ({} bytes, compressed: {} bytes)",
                name, size, compressed_size
            );

            contents.push(name);
        }

        Ok(contents)
    }

    pub fn create(files: &[&str], output_zip: &str) -> Result<(), String> {
        println!("[ZIP] Creating archive: {}", output_zip);

        let file = File::create(output_zip).map_err(|e| format!("Failed to create ZIP: {}", e))?;

        let mut zip = zip::ZipWriter::new(file);

        for file_path in files {
            let path = Path::new(file_path);
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| "Invalid filename".to_string())?;

            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            zip.start_file(name, options)
                .map_err(|e| format!("Failed to add file: {}", e))?;

            let mut f = File::open(file_path)
                .map_err(|e| format!("Failed to open {}: {}", file_path, e))?;

            std::io::copy(&mut f, &mut zip).map_err(|e| format!("Failed to write file: {}", e))?;

            println!("[ZIP] Added: {}", name);
        }

        zip.finish()
            .map_err(|e| format!("Failed to finalize ZIP: {}", e))?;

        println!("[ZIP] [OK] Archive created: {}", output_zip);
        Ok(())
    }

    pub fn crack_password(zip_path: &str, wordlist: &str) -> Result<Option<String>, String> {
        println!("[ZIP] Attempting to crack password for {}", zip_path);

        let passwords =
            fs::read_to_string(wordlist).map_err(|e| format!("Failed to read wordlist: {}", e))?;

        let file = File::open(zip_path).map_err(|e| format!("Failed to open ZIP: {}", e))?;

        for (i, password) in passwords.lines().enumerate() {
            let file_clone = file
                .try_clone()
                .map_err(|e| format!("Failed to clone file: {}", e))?;

            if let Ok(mut archive) = zip::ZipArchive::new(file_clone) {
                if let Ok(mut first_file) = archive.by_index(0) {
                    let mut buf = Vec::new();
                    if first_file.read_to_end(&mut buf).is_ok() {
                        println!("[ZIP] [OK] CRACKED! Password: {}", password);
                        return Ok(Some(password.to_string()));
                    }
                }
            }

            if (i + 1) % 1000 == 0 {
                println!("[ZIP] Tested {} passwords...", i + 1);
            }
        }

        println!("[ZIP] [ERROR] Password not found in wordlist");
        Ok(None)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// TAR MANIPULATION
// ────────────────────────────────────────────────────────────────────────────

pub struct TarTools;

impl TarTools {
    pub fn extract(tar_path: &str, output_dir: &str) -> Result<(), String> {
        println!("[TAR] Extracting {} to {}", tar_path, output_dir);

        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output dir: {}", e))?;

        let output = Command::new("tar")
            .args(["-xf", tar_path, "-C", output_dir])
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    println!("[TAR] [OK] Extraction successful");
                    Ok(())
                } else {
                    Err(format!(
                        "tar failed: {}",
                        String::from_utf8_lossy(&out.stderr)
                    ))
                }
            }
            Err(_) => Err("tar command not found".to_string()),
        }
    }

    pub fn create(files: &[&str], output_tar: &str) -> Result<(), String> {
        println!("[TAR] Creating archive: {}", output_tar);

        let mut args = vec!["-cf", output_tar];
        args.extend(files);

        let output = Command::new("tar")
            .args(&args)
            .output()
            .map_err(|e| format!("tar execution failed: {}", e))?;

        if output.status.success() {
            println!("[TAR] [OK] Archive created: {}", output_tar);
            Ok(())
        } else {
            Err(format!(
                "tar failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    pub fn list_contents(tar_path: &str) -> Result<Vec<String>, String> {
        println!("[TAR] Listing contents of {}", tar_path);

        let output = Command::new("tar")
            .args(["-tf", tar_path])
            .output()
            .map_err(|e| format!("tar execution failed: {}", e))?;

        if output.status.success() {
            let contents = String::from_utf8_lossy(&output.stdout);
            let files: Vec<String> = contents.lines().map(|s| s.to_string()).collect();

            for file in &files {
                println!("[TAR]   {}", file);
            }

            Ok(files)
        } else {
            Err(format!(
                "tar failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// GZIP MANIPULATION
// ────────────────────────────────────────────────────────────────────────────

pub struct GzipTools;

impl GzipTools {
    pub fn compress(input_path: &str, output_path: &str) -> Result<(), String> {
        println!("[GZIP] Compressing {} to {}", input_path, output_path);

        let input = fs::read(input_path).map_err(|e| format!("Failed to read input: {}", e))?;

        let output_file =
            File::create(output_path).map_err(|e| format!("Failed to create output: {}", e))?;

        let mut encoder =
            flate2::write::GzEncoder::new(output_file, flate2::Compression::default());
        encoder
            .write_all(&input)
            .map_err(|e| format!("Compression failed: {}", e))?;

        encoder
            .finish()
            .map_err(|e| format!("Failed to finalize: {}", e))?;

        println!("[GZIP] [OK] Compression complete");
        Ok(())
    }

    pub fn decompress(input_path: &str, output_path: &str) -> Result<(), String> {
        println!("[GZIP] Decompressing {} to {}", input_path, output_path);

        let input_file =
            File::open(input_path).map_err(|e| format!("Failed to open input: {}", e))?;

        let mut decoder = flate2::read::GzDecoder::new(input_file);
        let mut output_data = Vec::new();

        decoder
            .read_to_end(&mut output_data)
            .map_err(|e| format!("Decompression failed: {}", e))?;

        fs::write(output_path, output_data)
            .map_err(|e| format!("Failed to write output: {}", e))?;

        println!("[GZIP] [OK] Decompression complete");
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// RAR MANIPULATION
// ────────────────────────────────────────────────────────────────────────────

pub struct RarTools;

impl RarTools {
    pub fn extract(rar_path: &str, output_dir: &str) -> Result<(), String> {
        println!("[RAR] Extracting {} to {}", rar_path, output_dir);

        let output = Command::new("unrar")
            .args(["x", rar_path, output_dir])
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    println!("[RAR] [OK] Extraction successful");
                    Ok(())
                } else {
                    Err(format!(
                        "unrar failed: {}",
                        String::from_utf8_lossy(&out.stderr)
                    ))
                }
            }
            Err(_) => {
                Err("unrar command not found. Install with: apt-get install unrar".to_string())
            }
        }
    }

    pub fn list_contents(rar_path: &str) -> Result<Vec<String>, String> {
        println!("[RAR] Listing contents of {}", rar_path);

        let output = Command::new("unrar")
            .args(["l", rar_path])
            .output()
            .map_err(|e| format!("unrar execution failed: {}", e))?;

        if output.status.success() {
            let contents = String::from_utf8_lossy(&output.stdout);
            let files: Vec<String> = contents
                .lines()
                .filter(|l| !l.is_empty() && !l.starts_with('-'))
                .map(|s| s.trim().to_string())
                .collect();

            for file in &files {
                println!("[RAR]   {}", file);
            }

            Ok(files)
        } else {
            Err(format!(
                "unrar failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 7Z MANIPULATION
// ────────────────────────────────────────────────────────────────────────────

pub struct SevenZipTools;

impl SevenZipTools {
    pub fn extract(archive_path: &str, output_dir: &str) -> Result<(), String> {
        println!("[7Z] Extracting {} to {}", archive_path, output_dir);

        let output = Command::new("7z")
            .args(["x", archive_path, &format!("-o{}", output_dir), "-y"])
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    println!("[7Z] [OK] Extraction successful");
                    Ok(())
                } else {
                    Err(format!(
                        "7z failed: {}",
                        String::from_utf8_lossy(&out.stderr)
                    ))
                }
            }
            Err(_) => {
                Err("7z command not found. Install with: apt-get install p7zip-full".to_string())
            }
        }
    }

    pub fn create(files: &[&str], output_archive: &str) -> Result<(), String> {
        println!("[7Z] Creating archive: {}", output_archive);

        let mut args = vec!["a", output_archive];
        args.extend(files);

        let output = Command::new("7z")
            .args(&args)
            .output()
            .map_err(|e| format!("7z execution failed: {}", e))?;

        if output.status.success() {
            println!("[7Z] [OK] Archive created: {}", output_archive);
            Ok(())
        } else {
            Err(format!(
                "7z failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    pub fn list_contents(archive_path: &str) -> Result<Vec<String>, String> {
        println!("[7Z] Listing contents of {}", archive_path);

        let output = Command::new("7z")
            .args(["l", archive_path])
            .output()
            .map_err(|e| format!("7z execution failed: {}", e))?;

        if output.status.success() {
            let contents = String::from_utf8_lossy(&output.stdout);
            let files: Vec<String> = contents
                .lines()
                .filter(|l| !l.is_empty())
                .map(|s| s.trim().to_string())
                .collect();

            for file in &files {
                println!("[7Z]   {}", file);
            }

            Ok(files)
        } else {
            Err(format!(
                "7z failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// UNIVERSAL ARCHIVE HANDLER
// ────────────────────────────────────────────────────────────────────────────

pub struct ArchiveHandler;

impl ArchiveHandler {
    pub fn auto_extract(archive_path: &str, output_dir: &str) -> Result<(), String> {
        let path = Path::new(archive_path);
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| "No file extension".to_string())?
            .to_lowercase();

        match extension.as_str() {
            "zip" => ZipTools::extract(archive_path, output_dir),
            "tar" => TarTools::extract(archive_path, output_dir),
            "gz" | "gzip" => {
                if archive_path.ends_with(".tar.gz") || archive_path.ends_with(".tgz") {
                    TarTools::extract(archive_path, output_dir)
                } else {
                    let output_file =
                        PathBuf::from(output_dir).join(path.file_stem().unwrap_or_default());
                    GzipTools::decompress(archive_path, output_file.to_str().unwrap())
                }
            }
            "rar" => RarTools::extract(archive_path, output_dir),
            "7z" => SevenZipTools::extract(archive_path, output_dir),
            _ => Err(format!("Unsupported archive format: {}", extension)),
        }
    }

    pub fn identify_type(file_path: &str) -> Result<String, String> {
        let mut file = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;

        let mut header = [0u8; 8];
        file.read_exact(&mut header)
            .map_err(|e| format!("Failed to read header: {}", e))?;

        let archive_type = if header.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
            "ZIP"
        } else if header.starts_with(&[0x52, 0x61, 0x72, 0x21]) {
            "RAR"
        } else if header.starts_with(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]) {
            "7Z"
        } else if header.starts_with(&[0x1F, 0x8B]) {
            "GZIP"
        } else if header.starts_with(b"ustar") {
            "TAR"
        } else {
            "Unknown"
        };

        println!("[ARCHIVE] File type: {}", archive_type);
        Ok(archive_type.to_string())
    }
}
