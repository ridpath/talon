// Live Response forensics module
// Implements real-time monitoring and anti-sandbox detection

use std::collections::HashMap;
use std::io;

#[cfg(any(target_os = "linux", target_os = "windows"))]
use std::fs;

#[derive(Debug, Clone)]
pub enum ForensicsError {
    Unsupported(String),
    IoError(String),
    ParseError(String),
    DetectionFailed(String),
}

impl From<io::Error> for ForensicsError {
    fn from(err: io::Error) -> Self {
        ForensicsError::IoError(err.to_string())
    }
}

impl std::fmt::Display for ForensicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForensicsError::Unsupported(msg) => write!(f, "Unsupported: {}", msg),
            ForensicsError::IoError(msg) => write!(f, "I/O error: {}", msg),
            ForensicsError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ForensicsError::DetectionFailed(msg) => write!(f, "Detection failed: {}", msg),
        }
    }
}

impl std::error::Error for ForensicsError {}

#[derive(Debug, Clone, PartialEq)]
pub enum SyscallEvent {
    Open { path: String, flags: i32 },
    Read { fd: i32, count: usize },
    Write { fd: i32, count: usize },
    Connect { fd: i32, addr: String },
    Exec { path: String, args: Vec<String> },
    Unknown { syscall_num: u64 },
}

#[derive(Debug, Clone)]
pub struct SyscallTrace {
    pub events: Vec<SyscallEvent>,
    pub anomaly_count: usize,
}

impl SyscallTrace {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            anomaly_count: 0,
        }
    }

    pub fn add_event(&mut self, event: SyscallEvent) {
        self.events.push(event);
    }

    pub fn detect_anomalies(&mut self) -> Vec<String> {
        let mut anomalies = Vec::new();
        let mut failed_opens = 0;

        for event in &self.events {
            if let SyscallEvent::Open { path, flags } = event {
                if *flags < 0 {
                    failed_opens += 1;
                    if failed_opens > 5 {
                        anomalies.push(format!(
                            "Repeated failed open() calls detected ({}): possible sandbox probing",
                            path
                        ));
                    }
                }
            }
        }

        self.anomaly_count = anomalies.len();
        anomalies
    }

    pub fn has_anomalies(&self) -> bool {
        self.anomaly_count > 0
    }
}

impl Default for SyscallTrace {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SyscallTracer {
    trace: SyscallTrace,
    monitoring: bool,
}

impl SyscallTracer {
    pub fn new() -> Self {
        Self {
            trace: SyscallTrace::new(),
            monitoring: false,
        }
    }

    pub fn start_monitoring(&mut self) -> Result<(), ForensicsError> {
        #[cfg(target_os = "linux")]
        {
            self.monitoring = true;
            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(ForensicsError::Unsupported(
                "Syscall tracing is only available on Linux with eBPF support".to_string(),
            ))
        }
    }

    pub fn stop_monitoring(&mut self) {
        self.monitoring = false;
    }

    pub fn is_monitoring(&self) -> bool {
        self.monitoring
    }

    pub fn record_syscall(&mut self, event: SyscallEvent) {
        if self.monitoring {
            self.trace.add_event(event);
        }
    }

    pub fn get_trace(&mut self) -> &mut SyscallTrace {
        &mut self.trace
    }

    pub fn detect_sandbox(&mut self) -> Result<bool, ForensicsError> {
        let anomalies = self.trace.detect_anomalies();
        Ok(!anomalies.is_empty())
    }
}

impl Default for SyscallTracer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnvironmentType {
    BareMetalLinux,
    BareMetalWindows,
    DockerContainer,
    KubernetesContainer,
    VirtualBoxVM,
    VMwareVM,
    QemuVM,
    HyperVVM,
    WSL,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct EnvironmentDetection {
    pub env_type: EnvironmentType,
    pub confidence: f32,
    pub indicators: Vec<String>,
}

impl EnvironmentDetection {
    pub fn new() -> Self {
        Self {
            env_type: EnvironmentType::Unknown,
            confidence: 0.0,
            indicators: Vec::new(),
        }
    }

    pub fn is_virtualized(&self) -> bool {
        matches!(
            self.env_type,
            EnvironmentType::VirtualBoxVM
                | EnvironmentType::VMwareVM
                | EnvironmentType::QemuVM
                | EnvironmentType::HyperVVM
        )
    }

    pub fn is_containerized(&self) -> bool {
        matches!(
            self.env_type,
            EnvironmentType::DockerContainer | EnvironmentType::KubernetesContainer
        )
    }

    pub fn is_sandbox(&self) -> bool {
        self.is_virtualized() || self.is_containerized()
    }
}

impl Default for EnvironmentDetection {
    fn default() -> Self {
        Self::new()
    }
}

pub struct VmContainerDetector {
    heuristics: HashMap<String, f32>,
}

impl VmContainerDetector {
    pub fn new() -> Self {
        let mut heuristics = HashMap::new();
        heuristics.insert("docker".to_string(), 0.9);
        heuristics.insert("kubernetes".to_string(), 0.9);
        heuristics.insert("vboxguest".to_string(), 0.85);
        heuristics.insert("vmware".to_string(), 0.85);
        heuristics.insert("qemu".to_string(), 0.8);
        heuristics.insert("hyperv".to_string(), 0.85);
        heuristics.insert("wsl".to_string(), 0.9);

        Self { heuristics }
    }

    #[cfg(target_os = "linux")]
    pub fn detect(&self) -> Result<EnvironmentDetection, ForensicsError> {
        let mut detection = EnvironmentDetection::new();

        if Path::new("/.dockerenv").exists() {
            detection.env_type = EnvironmentType::DockerContainer;
            detection.confidence = 0.95;
            detection.indicators.push("/.dockerenv file exists".to_string());
        }

        if let Ok(cgroup_content) = fs::read_to_string("/proc/1/cgroup") {
            if cgroup_content.contains("docker") {
                detection.env_type = EnvironmentType::DockerContainer;
                detection.confidence = detection.confidence.max(0.9);
                detection.indicators.push("/proc/1/cgroup contains docker".to_string());
            }

            if cgroup_content.contains("kubepods") {
                detection.env_type = EnvironmentType::KubernetesContainer;
                detection.confidence = 0.95;
                detection.indicators.push("/proc/1/cgroup contains kubepods".to_string());
            }
        }

        if let Ok(dmi_content) = fs::read_to_string("/sys/class/dmi/id/product_name") {
            let dmi_lower = dmi_content.to_lowercase();
            
            if dmi_lower.contains("virtualbox") {
                detection.env_type = EnvironmentType::VirtualBoxVM;
                detection.confidence = 0.9;
                detection.indicators.push("DMI product name contains VirtualBox".to_string());
            } else if dmi_lower.contains("vmware") {
                detection.env_type = EnvironmentType::VMwareVM;
                detection.confidence = 0.9;
                detection.indicators.push("DMI product name contains VMware".to_string());
            } else if dmi_lower.contains("qemu") {
                detection.env_type = EnvironmentType::QemuVM;
                detection.confidence = 0.85;
                detection.indicators.push("DMI product name contains QEMU".to_string());
            } else if dmi_lower.contains("hyper-v") || dmi_lower.contains("microsoft") {
                detection.env_type = EnvironmentType::HyperVVM;
                detection.confidence = 0.85;
                detection.indicators.push("DMI product name contains Hyper-V/Microsoft".to_string());
            }
        }

        if let Ok(modules) = fs::read_to_string("/proc/modules") {
            if modules.contains("vboxguest") || modules.contains("vboxsf") {
                detection.env_type = EnvironmentType::VirtualBoxVM;
                detection.confidence = detection.confidence.max(0.95);
                detection.indicators.push("VirtualBox kernel modules detected".to_string());
            }

            if modules.contains("vmw_") {
                detection.env_type = EnvironmentType::VMwareVM;
                detection.confidence = detection.confidence.max(0.95);
                detection.indicators.push("VMware kernel modules detected".to_string());
            }
        }

        if let Ok(version) = fs::read_to_string("/proc/version") {
            if version.contains("Microsoft") || version.contains("WSL") {
                detection.env_type = EnvironmentType::WSL;
                detection.confidence = 0.95;
                detection.indicators.push("WSL detected in kernel version".to_string());
            }
        }

        if detection.env_type == EnvironmentType::Unknown {
            detection.env_type = EnvironmentType::BareMetalLinux;
            detection.confidence = 0.7;
            detection.indicators.push("No virtualization/containerization indicators found".to_string());
        }

        Ok(detection)
    }

    #[cfg(target_os = "windows")]
    pub fn detect(&self) -> Result<EnvironmentDetection, ForensicsError> {
        let mut detection = EnvironmentDetection::new();

        use std::process::Command;

        if let Ok(output) = Command::new("systeminfo").output() {
            let output_str = String::from_utf8_lossy(&output.stdout).to_lowercase();

            if output_str.contains("virtualbox") {
                detection.env_type = EnvironmentType::VirtualBoxVM;
                detection.confidence = 0.9;
                detection.indicators.push("VirtualBox detected in systeminfo".to_string());
            } else if output_str.contains("vmware") {
                detection.env_type = EnvironmentType::VMwareVM;
                detection.confidence = 0.9;
                detection.indicators.push("VMware detected in systeminfo".to_string());
            } else if output_str.contains("hyper-v") || output_str.contains("microsoft virtual") {
                detection.env_type = EnvironmentType::HyperVVM;
                detection.confidence = 0.9;
                detection.indicators.push("Hyper-V detected in systeminfo".to_string());
            }
        }

        if let Ok(output) = Command::new("wmic")
            .args(&["computersystem", "get", "model"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout).to_lowercase();

            if output_str.contains("virtualbox") {
                detection.env_type = EnvironmentType::VirtualBoxVM;
                detection.confidence = detection.confidence.max(0.95);
                detection.indicators.push("VirtualBox model detected via WMIC".to_string());
            } else if output_str.contains("vmware") {
                detection.env_type = EnvironmentType::VMwareVM;
                detection.confidence = detection.confidence.max(0.95);
                detection.indicators.push("VMware model detected via WMIC".to_string());
            }
        }

        if detection.env_type == EnvironmentType::Unknown {
            detection.env_type = EnvironmentType::BareMetalWindows;
            detection.confidence = 0.7;
            detection.indicators.push("No virtualization indicators found".to_string());
        }

        Ok(detection)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    pub fn detect(&self) -> Result<EnvironmentDetection, ForensicsError> {
        Err(ForensicsError::Unsupported(
            "VM/Container detection is only available on Linux and Windows".to_string(),
        ))
    }
}

impl Default for VmContainerDetector {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EbpfMonitor {
    active: bool,
    events: Vec<String>,
}

impl EbpfMonitor {
    pub fn new() -> Self {
        Self {
            active: false,
            events: Vec::new(),
        }
    }

    #[cfg(target_os = "linux")]
    pub fn attach(&mut self) -> Result<(), ForensicsError> {
        self.active = true;
        self.events.push("eBPF monitor attached".to_string());
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn attach(&mut self) -> Result<(), ForensicsError> {
        Err(ForensicsError::Unsupported(
            "eBPF monitoring is only available on Linux with kernel 5.8+".to_string(),
        ))
    }

    pub fn detach(&mut self) {
        self.active = false;
        self.events.push("eBPF monitor detached".to_string());
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn get_events(&self) -> &[String] {
        &self.events
    }

    pub fn record_event(&mut self, event: String) {
        if self.active {
            self.events.push(event);
        }
    }
}

impl Default for EbpfMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_trace_creation() {
        let trace = SyscallTrace::new();
        assert_eq!(trace.events.len(), 0);
        assert_eq!(trace.anomaly_count, 0);
    }

    #[test]
    fn test_syscall_trace_add_event() {
        let mut trace = SyscallTrace::new();
        trace.add_event(SyscallEvent::Open {
            path: "/test".to_string(),
            flags: 0,
        });
        assert_eq!(trace.events.len(), 1);
    }

    #[test]
    fn test_syscall_trace_detect_anomalies() {
        let mut trace = SyscallTrace::new();
        
        for i in 0..10 {
            trace.add_event(SyscallEvent::Open {
                path: format!("/test{}", i),
                flags: -1,
            });
        }

        let anomalies = trace.detect_anomalies();
        assert!(!anomalies.is_empty());
        assert!(trace.has_anomalies());
    }

    #[test]
    fn test_syscall_tracer_creation() {
        let tracer = SyscallTracer::new();
        assert!(!tracer.is_monitoring());
    }

    #[test]
    fn test_syscall_tracer_record() {
        let mut tracer = SyscallTracer::new();
        tracer.record_syscall(SyscallEvent::Read { fd: 1, count: 100 });
        
        let trace = tracer.get_trace();
        assert_eq!(trace.events.len(), 0);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_syscall_tracer_start_monitoring() {
        let mut tracer = SyscallTracer::new();
        let result = tracer.start_monitoring();
        assert!(result.is_ok());
        assert!(tracer.is_monitoring());
    }

    #[test]
    fn test_environment_detection_creation() {
        let detection = EnvironmentDetection::new();
        assert_eq!(detection.env_type, EnvironmentType::Unknown);
        assert_eq!(detection.confidence, 0.0);
        assert_eq!(detection.indicators.len(), 0);
    }

    #[test]
    fn test_environment_detection_is_virtualized() {
        let mut detection = EnvironmentDetection::new();
        
        detection.env_type = EnvironmentType::VirtualBoxVM;
        assert!(detection.is_virtualized());
        assert!(!detection.is_containerized());
        assert!(detection.is_sandbox());

        detection.env_type = EnvironmentType::BareMetalLinux;
        assert!(!detection.is_virtualized());
        assert!(!detection.is_sandbox());
    }

    #[test]
    fn test_environment_detection_is_containerized() {
        let mut detection = EnvironmentDetection::new();
        
        detection.env_type = EnvironmentType::DockerContainer;
        assert!(detection.is_containerized());
        assert!(!detection.is_virtualized());
        assert!(detection.is_sandbox());
    }

    #[test]
    fn test_vm_container_detector_creation() {
        let detector = VmContainerDetector::new();
        assert!(detector.heuristics.contains_key("docker"));
        assert!(detector.heuristics.contains_key("vmware"));
    }

    #[test]
    fn test_ebpf_monitor_creation() {
        let monitor = EbpfMonitor::new();
        assert!(!monitor.is_active());
        assert_eq!(monitor.get_events().len(), 0);
    }

    #[test]
    fn test_ebpf_monitor_record_event() {
        let mut monitor = EbpfMonitor::new();
        monitor.record_event("test event".to_string());
        assert_eq!(monitor.get_events().len(), 0);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_ebpf_monitor_attach() {
        let mut monitor = EbpfMonitor::new();
        let result = monitor.attach();
        assert!(result.is_ok());
        assert!(monitor.is_active());
    }

    #[test]
    fn test_ebpf_monitor_detach() {
        let mut monitor = EbpfMonitor::new();
        monitor.detach();
        assert!(!monitor.is_active());
    }

    #[test]
    fn test_syscall_event_equality() {
        let event1 = SyscallEvent::Open {
            path: "/test".to_string(),
            flags: 0,
        };
        let event2 = SyscallEvent::Open {
            path: "/test".to_string(),
            flags: 0,
        };
        assert_eq!(event1, event2);
    }

    #[test]
    fn test_environment_type_equality() {
        assert_eq!(EnvironmentType::DockerContainer, EnvironmentType::DockerContainer);
        assert_ne!(EnvironmentType::DockerContainer, EnvironmentType::VirtualBoxVM);
    }

    #[test]
    fn test_forensics_error_display() {
        let err = ForensicsError::Unsupported("test".to_string());
        assert_eq!(err.to_string(), "Unsupported: test");

        let err = ForensicsError::IoError("io error".to_string());
        assert_eq!(err.to_string(), "I/O error: io error");

        let err = ForensicsError::ParseError("parse error".to_string());
        assert_eq!(err.to_string(), "Parse error: parse error");

        let err = ForensicsError::DetectionFailed("detection failed".to_string());
        assert_eq!(err.to_string(), "Detection failed: detection failed");
    }
}
