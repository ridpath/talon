use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct EnvironmentInfo {
    pub is_vm: bool,
    pub is_sandboxed: bool,
    pub is_debugged: bool,
    pub vm_indicators: Vec<String>,
    pub sandbox_indicators: Vec<String>,
}

pub fn detect_vm() -> bool {
    detect_vmware() || detect_virtualbox() || detect_qemu() || detect_hyperv()
}

pub fn detect_vmware() -> bool {
    let indicators = vec![
        "/sys/class/dmi/id/product_name",
        "/sys/class/dmi/id/sys_vendor",
        "/sys/class/dmi/id/board_vendor",
    ];
    
    for indicator in indicators {
        if let Ok(content) = fs::read_to_string(indicator) {
            if content.to_lowercase().contains("vmware") {
                println!("[ENV] VMware detected: {}", indicator);
                return true;
            }
        }
    }
    
    if Path::new("/proc/scsi/scsi").exists() {
        if let Ok(content) = fs::read_to_string("/proc/scsi/scsi") {
            if content.to_lowercase().contains("vmware") {
                println!("[ENV] VMware detected via /proc/scsi/scsi");
                return true;
            }
        }
    }
    
    false
}

pub fn detect_virtualbox() -> bool {
    let indicators = vec![
        "/sys/class/dmi/id/product_name",
        "/sys/class/dmi/id/sys_vendor",
    ];
    
    for indicator in indicators {
        if let Ok(content) = fs::read_to_string(indicator) {
            if content.to_lowercase().contains("virtualbox") || 
               content.to_lowercase().contains("vbox") ||
               content.to_lowercase().contains("oracle") {
                println!("[ENV] VirtualBox detected: {}", indicator);
                return true;
            }
        }
    }
    
    if Path::new("/proc/modules").exists() {
        if let Ok(content) = fs::read_to_string("/proc/modules") {
            if content.contains("vboxguest") || content.contains("vboxsf") {
                println!("[ENV] VirtualBox detected via kernel modules");
                return true;
            }
        }
    }
    
    false
}

pub fn detect_qemu() -> bool {
    let indicators = vec![
        "/sys/class/dmi/id/product_name",
        "/sys/class/dmi/id/sys_vendor",
        "/sys/class/dmi/id/chassis_vendor",
    ];
    
    for indicator in indicators {
        if let Ok(content) = fs::read_to_string(indicator) {
            let lower = content.to_lowercase();
            if lower.contains("qemu") || lower.contains("bochs") {
                println!("[ENV] QEMU detected: {}", indicator);
                return true;
            }
        }
    }
    
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        if content.contains("QEMU Virtual CPU") {
            println!("[ENV] QEMU detected via cpuinfo");
            return true;
        }
    }
    
    false
}

pub fn detect_hyperv() -> bool {
    if let Ok(content) = fs::read_to_string("/sys/class/dmi/id/product_name") {
        if content.to_lowercase().contains("hyper-v") || 
           content.to_lowercase().contains("microsoft") {
            println!("[ENV] Hyper-V detected");
            return true;
        }
    }
    
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        if content.contains("hypervisor") {
            println!("[ENV] Hypervisor detected via cpuinfo");
            return true;
        }
    }
    
    false
}

pub fn detect_debugger() -> bool {
    detect_ptrace() || detect_tracerpid()
}

pub fn detect_ptrace() -> bool {
    #[cfg(target_os = "linux")]
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static TRACED: AtomicBool = AtomicBool::new(false);
        
        unsafe {
            if libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0) == -1 {
                TRACED.store(true, Ordering::SeqCst);
                println!("[ENV] Debugger detected via ptrace");
                return true;
            }
        }
    }
    false
}

pub fn detect_tracerpid() -> bool {
    if let Ok(content) = fs::read_to_string("/proc/self/status") {
        for line in content.lines() {
            if line.starts_with("TracerPid:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 && parts[1] != "0" {
                    println!("[ENV] Debugger detected: TracerPid = {}", parts[1]);
                    return true;
                }
            }
        }
    }
    false
}

pub fn detect_sandbox() -> bool {
    detect_cuckoo() || detect_low_resources() || detect_sleep_acceleration()
}

pub fn detect_cuckoo() -> bool {
    let artifacts = vec![
        "/root/.cuckoo",
        "/home/cuckoo",
        "/opt/cuckoo",
    ];
    
    for artifact in artifacts {
        if Path::new(artifact).exists() {
            println!("[ENV] Cuckoo sandbox detected: {}", artifact);
            return true;
        }
    }
    
    if let Ok(content) = fs::read_to_string("/proc/self/cmdline") {
        if content.contains("cuckoo") {
            println!("[ENV] Cuckoo detected in cmdline");
            return true;
        }
    }
    
    false
}

pub fn detect_low_resources() -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 1 {
                        if let Ok(mem_kb) = parts[1].parse::<u64>() {
                            let mem_gb = mem_kb / 1024 / 1024;
                            if mem_gb < 2 {
                                println!("[ENV] Low memory detected: {} GB", mem_gb);
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    
    if let Ok(output) = Command::new("nproc").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(cores) = stdout.trim().parse::<u32>() {
            if cores < 2 {
                println!("[ENV] Low CPU cores detected: {}", cores);
                return true;
            }
        }
    }
    
    false
}

pub fn detect_sleep_acceleration() -> bool {
    let sleep_duration = Duration::from_secs(2);
    let start = Instant::now();
    std::thread::sleep(sleep_duration);
    let elapsed = start.elapsed();
    
    let diff = if elapsed > sleep_duration {
        elapsed - sleep_duration
    } else {
        sleep_duration - elapsed
    };
    
    if diff > Duration::from_millis(500) {
        println!("[ENV] Sleep acceleration detected: {:?} vs {:?}", elapsed, sleep_duration);
        return true;
    }
    
    false
}

pub fn detect_wine() -> bool {
    std::env::var("WINEPREFIX").is_ok() ||
    std::env::var("WINEDLLOVERRIDES").is_ok() ||
    Path::new("/proc/self/exe").read_link()
        .map(|p| p.to_string_lossy().contains("wine"))
        .unwrap_or(false)
}

pub fn check_hostname() -> Result<String, String> {
    Command::new("hostname")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .map_err(|e| e.to_string())
}

pub fn suspicious_hostname() -> bool {
    if let Ok(hostname) = check_hostname() {
        let suspicious = vec!["sandbox", "malware", "cuckoo", "analysis", "virus", "sample"];
        let lower = hostname.to_lowercase();
        
        for pattern in suspicious {
            if lower.contains(pattern) {
                println!("[ENV] Suspicious hostname: {}", hostname);
                return true;
            }
        }
    }
    false
}

pub fn check_timezone() -> Result<String, String> {
    std::env::var("TZ")
        .or_else(|_| fs::read_to_string("/etc/timezone").map(|s| s.trim().to_string()))
        .map_err(|e| e.to_string())
}

pub fn check_locale() -> Result<String, String> {
    std::env::var("LANG")
        .map_err(|e| e.to_string())
}

pub fn detect_user_interaction() -> bool {
    let mouse_activity = Path::new("/dev/input/mice").exists();
    let keyboard_activity = Path::new("/dev/input/by-path").read_dir()
        .map(|mut d| d.any(|e| e.ok().map(|e| e.path().to_string_lossy().contains("kbd")).unwrap_or(false)))
        .unwrap_or(false);
    
    mouse_activity && keyboard_activity
}

pub fn comprehensive_check() -> EnvironmentInfo {
    let mut info = EnvironmentInfo {
        is_vm: false,
        is_sandboxed: false,
        is_debugged: false,
        vm_indicators: Vec::new(),
        sandbox_indicators: Vec::new(),
    };
    
    if detect_vmware() {
        info.is_vm = true;
        info.vm_indicators.push("VMware".to_string());
    }
    
    if detect_virtualbox() {
        info.is_vm = true;
        info.vm_indicators.push("VirtualBox".to_string());
    }
    
    if detect_qemu() {
        info.is_vm = true;
        info.vm_indicators.push("QEMU".to_string());
    }
    
    if detect_hyperv() {
        info.is_vm = true;
        info.vm_indicators.push("Hyper-V".to_string());
    }
    
    if detect_debugger() {
        info.is_debugged = true;
    }
    
    if detect_cuckoo() {
        info.is_sandboxed = true;
        info.sandbox_indicators.push("Cuckoo".to_string());
    }
    
    if detect_low_resources() {
        info.is_sandboxed = true;
        info.sandbox_indicators.push("Low Resources".to_string());
    }
    
    if detect_sleep_acceleration() {
        info.is_sandboxed = true;
        info.sandbox_indicators.push("Sleep Acceleration".to_string());
    }
    
    if suspicious_hostname() {
        info.is_sandboxed = true;
        info.sandbox_indicators.push("Suspicious Hostname".to_string());
    }
    
    info
}

pub fn exit_if_detected() {
    let info = comprehensive_check();
    
    if info.is_vm || info.is_sandboxed || info.is_debugged {
        println!("[ENV] Hostile environment detected! Exiting...");
        std::process::exit(1);
    }
}

pub fn sleep_evasion(seconds: u64) {
    std::thread::sleep(Duration::from_secs(seconds));
}

pub fn jitter_sleep(min_secs: u64, max_secs: u64) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let sleep_time = rng.gen_range(min_secs..=max_secs);
    std::thread::sleep(Duration::from_secs(sleep_time));
}
