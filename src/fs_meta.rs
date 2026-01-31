use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

pub struct FileTimestamps {
    pub modified: SystemTime,
    pub accessed: SystemTime,
    pub created: SystemTime,
}

pub fn get_timestamps(path: &str) -> Result<FileTimestamps, String> {
    let metadata = fs::metadata(path).map_err(|e| format!("Failed to get metadata: {}", e))?;

    Ok(FileTimestamps {
        modified: metadata.modified().unwrap_or(UNIX_EPOCH),
        accessed: metadata.accessed().unwrap_or(UNIX_EPOCH),
        created: metadata.created().unwrap_or(UNIX_EPOCH),
    })
}

pub fn set_timestamps(
    _path: &str,
    _modified: SystemTime,
    _accessed: SystemTime,
) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::os::unix::fs::MetadataExt;

        let path_c = CString::new(path).map_err(|e| format!("Invalid path: {}", e))?;

        let modified_secs = modified
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Invalid time: {}", e))?
            .as_secs() as i64;

        let accessed_secs = accessed
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Invalid time: {}", e))?
            .as_secs() as i64;

        #[repr(C)]
        struct Timeval {
            tv_sec: i64,
            tv_usec: i64,
        }

        #[repr(C)]
        struct Utimbuf {
            actime: i64,
            modtime: i64,
        }

        let times = Utimbuf {
            actime: accessed_secs,
            modtime: modified_secs,
        };

        unsafe {
            if libc::utime(
                path_c.as_ptr(),
                &times as *const Utimbuf as *const libc::utimbuf,
            ) == 0
            {
                println!("[FS] Timestamps updated for {}", path);
                Ok(())
            } else {
                Err("Failed to set timestamps".to_string())
            }
        }
    }

    #[cfg(not(unix))]
    {
        Err("Timestamp manipulation not implemented for this OS".to_string())
    }
}

pub fn timestomp_match(source: &str, target: &str) -> Result<(), String> {
    let source_times = get_timestamps(source)?;
    set_timestamps(target, source_times.modified, source_times.accessed)?;
    println!("[FS] Timestomped {} to match {}", target, source);
    Ok(())
}

pub fn timestomp_clear(path: &str) -> Result<(), String> {
    let epoch = UNIX_EPOCH;
    set_timestamps(path, epoch, epoch)?;
    println!("[FS] Timestamps cleared for {}", path);
    Ok(())
}

pub fn set_file_attributes_hidden(path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x00000002;

        let _file = OpenOptions::new()
            .write(true)
            .attributes(FILE_ATTRIBUTE_HIDDEN)
            .open(path)
            .map_err(|e| format!("Failed to set hidden attribute: {}", e))?;

        println!("[FS] File {} set to hidden", path);
        Ok(())
    }

    #[cfg(unix)]
    {
        if !Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false)
        {
            let parent = Path::new(path).parent().unwrap_or(Path::new("."));
            let filename = Path::new(path).file_name().unwrap();
            let hidden_name = format!(".{}", filename.to_string_lossy());
            let new_path = parent.join(hidden_name);

            fs::rename(path, new_path).map_err(|e| format!("Failed to hide file: {}", e))?;

            println!("[FS] File renamed to hidden");
        }
        Ok(())
    }
}

pub fn read_alternate_data_stream(path: &str, stream_name: &str) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let ads_path = format!("{}:{}", path, stream_name);
        fs::read_to_string(&ads_path).map_err(|e| format!("Failed to read ADS: {}", e))
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("ADS only supported on Windows".to_string())
    }
}

pub fn write_alternate_data_stream(
    path: &str,
    stream_name: &str,
    data: &str,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let ads_path = format!("{}:{}", path, stream_name);
        fs::write(&ads_path, data).map_err(|e| format!("Failed to write ADS: {}", e))?;
        println!("[FS] Written to ADS: {}:{}", path, stream_name);
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("ADS only supported on Windows".to_string())
    }
}

pub fn enumerate_ads(path: &str) -> Result<Vec<String>, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        let output = Command::new("powershell")
            .args(["-Command", &format!("Get-Item '{}' -Stream *", path)])
            .output()
            .map_err(|e| format!("Failed to enumerate ADS: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let streams: Vec<String> = stdout
            .lines()
            .filter(|l| !l.is_empty())
            .map(|s| s.to_string())
            .collect();

        Ok(streams)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("ADS only supported on Windows".to_string())
    }
}

pub fn get_file_inode(_path: &str) -> Result<u64, String> {
    #[cfg(unix)]
    {
        let metadata = fs::metadata(path).map_err(|e| format!("Failed to get metadata: {}", e))?;
        Ok(metadata.ino())
    }

    #[cfg(not(unix))]
    {
        Err("Inode information only available on Unix systems".to_string())
    }
}

pub fn secure_delete(path: &str, passes: u32) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|e| format!("Failed to get file metadata: {}", e))?;

    let file_size = metadata.len() as usize;

    for pass in 0..passes {
        let mut file = OpenOptions::new()
            .write(true)
            .open(path)
            .map_err(|e| format!("Failed to open file for overwrite: {}", e))?;

        let pattern = match pass % 3 {
            0 => vec![0x00; file_size],
            1 => vec![0xFF; file_size],
            _ => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                (0..file_size).map(|_| rng.gen::<u8>()).collect()
            }
        };

        file.write_all(&pattern)
            .map_err(|e| format!("Failed to overwrite file: {}", e))?;
        file.sync_all()
            .map_err(|e| format!("Failed to sync file: {}", e))?;
    }

    fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))?;

    println!("[FS] Securely deleted {} with {} passes", path, passes);
    Ok(())
}

pub fn shred_file(path: &str) -> Result<(), String> {
    secure_delete(path, 7)
}

pub fn list_open_files() -> Result<Vec<String>, String> {
    #[cfg(target_os = "linux")]
    {
        let pid = std::process::id();
        let fd_path = format!("/proc/{}/fd", pid);

        let entries =
            fs::read_dir(&fd_path).map_err(|e| format!("Failed to read fd directory: {}", e))?;

        let mut files = Vec::new();
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(link) = fs::read_link(entry.path()) {
                    files.push(link.to_string_lossy().to_string());
                }
            }
        }

        Ok(files)
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err("Open file listing only implemented for Linux".to_string())
    }
}

pub fn clear_file_slack_space(path: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let metadata = file
        .metadata()
        .map_err(|e| format!("Failed to get metadata: {}", e))?;

    let file_size = metadata.len();
    let block_size = 4096u64;
    let slack_size = block_size - (file_size % block_size);

    if slack_size > 0 && slack_size < block_size {
        let zeros = vec![0u8; slack_size as usize];
        file.write_all(&zeros)
            .map_err(|e| format!("Failed to clear slack space: {}", e))?;

        println!(
            "[FS] Cleared {} bytes of slack space in {}",
            slack_size, path
        );
    }

    Ok(())
}

pub fn clone_file_metadata(source: &str, target: &str) -> Result<(), String> {
    let source_metadata =
        fs::metadata(source).map_err(|e| format!("Failed to read source metadata: {}", e))?;

    let permissions = source_metadata.permissions();
    fs::set_permissions(target, permissions)
        .map_err(|e| format!("Failed to set permissions: {}", e))?;

    timestomp_match(source, target)?;

    println!("[FS] Cloned metadata from {} to {}", source, target);
    Ok(())
}

pub fn create_decoy_file(path: &str, size_mb: u32) -> Result<(), String> {
    let size_bytes = (size_mb as usize) * 1024 * 1024;
    let mut file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;

    use rand::Rng;
    let mut rng = rand::thread_rng();
    let chunk_size = 1024 * 1024;
    let chunks = size_bytes / chunk_size;

    for _ in 0..chunks {
        let data: Vec<u8> = (0..chunk_size).map(|_| rng.gen::<u8>()).collect();
        file.write_all(&data)
            .map_err(|e| format!("Failed to write decoy data: {}", e))?;
    }

    println!("[FS] Created decoy file {} ({} MB)", path, size_mb);
    Ok(())
}
