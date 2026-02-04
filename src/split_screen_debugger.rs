use crate::gdb_tools::GdbSession;
use crate::time_travel::TimeTravelDebugger;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SplitScreenDebugger {
    gdb_session: Arc<RwLock<Option<GdbSession>>>,
    debugger: Arc<TimeTravelDebugger>,
    source_file: Option<String>,
    source_lines: Vec<String>,
    current_line: usize,
    gdb_output: Vec<String>,
    terminal_height: u16,
    terminal_width: u16,
}

impl SplitScreenDebugger {
    pub fn new(
        gdb_session: Arc<RwLock<Option<GdbSession>>>,
        debugger: Arc<TimeTravelDebugger>,
        source_file: Option<String>,
    ) -> Result<Self, String> {
        let (width, height) = terminal::size()
            .map_err(|e| format!("Failed to get terminal size: {}", e))?;

        let source_lines = if let Some(ref path) = source_file {
            std::fs::read_to_string(path)
                .map(|content| content.lines().map(String::from).collect())
                .unwrap_or_else(|_| vec!["Source file not found".to_string()])
        } else {
            vec!["No source file loaded".to_string()]
        };

        Ok(SplitScreenDebugger {
            gdb_session,
            debugger,
            source_file,
            source_lines,
            current_line: 0,
            gdb_output: vec![],
            terminal_height: height,
            terminal_width: width,
        })
    }

    pub async fn start(&mut self) -> Result<(), String> {
        let mut stdout = io::stdout();

        execute!(stdout, EnterAlternateScreen, Hide)
            .map_err(|e| format!("Failed to enter alternate screen: {}", e))?;

        terminal::enable_raw_mode()
            .map_err(|e| format!("Failed to enable raw mode: {}", e))?;

        self.render().await?;

        loop {
            if event::poll(std::time::Duration::from_millis(100))
                .map_err(|e| format!("Event poll error: {}", e))?
            {
                if let Event::Key(key) = event::read()
                    .map_err(|e| format!("Failed to read event: {}", e))?
                {
                    if !self.handle_key(key).await? {
                        break;
                    }
                }
            }

            self.render().await?;
        }

        execute!(stdout, LeaveAlternateScreen, Show)
            .map_err(|e| format!("Failed to leave alternate screen: {}", e))?;

        terminal::disable_raw_mode()
            .map_err(|e| format!("Failed to disable raw mode: {}", e))?;

        Ok(())
    }

    async fn handle_key(&mut self, key: KeyEvent) -> Result<bool, String> {
        match key.code {
            KeyCode::Char('q') => return Ok(false),
            KeyCode::Char('s') => self.gdb_step().await?,
            KeyCode::Char('c') => self.gdb_continue().await?,
            KeyCode::Char('r') => self.gdb_reverse_step().await?,
            KeyCode::Char('R') => self.gdb_reverse_continue().await?,
            KeyCode::Up => {
                if self.current_line > 0 {
                    self.current_line -= 1;
                }
            }
            KeyCode::Down => {
                if self.current_line < self.source_lines.len().saturating_sub(1) {
                    self.current_line += 1;
                }
            }
            KeyCode::Char('b') => self.set_breakpoint().await?,
            KeyCode::Char('p') => self.print_variables().await?,
            _ => {}
        }
        Ok(true)
    }

    async fn gdb_step(&mut self) -> Result<(), String> {
        let output = {
            let mut gdb_session = self.gdb_session.write().await;
            
            if let Some(ref mut gdb) = *gdb_session {
                gdb.step()?
            } else {
                return Ok(());
            }
        };
        
        self.gdb_output.push(format!("[Step] {}", output));
        self.keep_last_n_lines(20);
        Ok(())
    }

    async fn gdb_continue(&mut self) -> Result<(), String> {
        let output = {
            let mut gdb_session = self.gdb_session.write().await;
            
            if let Some(ref mut gdb) = *gdb_session {
                gdb.continue_exec()?
            } else {
                return Ok(());
            }
        };
        
        self.gdb_output.push(format!("[Continue] {}", output));
        self.keep_last_n_lines(20);
        Ok(())
    }

    async fn gdb_reverse_step(&mut self) -> Result<(), String> {
        let output = self.debugger.gdb_reverse_step().await?;
        self.gdb_output.push(format!("[Reverse Step] {}", output));
        self.keep_last_n_lines(20);
        Ok(())
    }

    async fn gdb_reverse_continue(&mut self) -> Result<(), String> {
        let output = self.debugger.gdb_reverse_continue().await?;
        self.gdb_output.push(format!("[Reverse Continue] {}", output));
        self.keep_last_n_lines(20);
        Ok(())
    }

    async fn set_breakpoint(&mut self) -> Result<(), String> {
        let line_num = self.current_line + 1;
        
        {
            let mut gdb_session = self.gdb_session.write().await;
            
            if let Some(ref mut gdb) = *gdb_session {
                gdb.execute(&format!("break {}", line_num))?;
            } else {
                return Ok(());
            }
        }
        
        self.gdb_output.push(format!("[Breakpoint] Set at line {}", line_num));
        self.keep_last_n_lines(20);
        Ok(())
    }

    async fn print_variables(&mut self) -> Result<(), String> {
        let output = {
            let mut gdb_session = self.gdb_session.write().await;
            
            if let Some(ref mut gdb) = *gdb_session {
                gdb.execute("info locals")?
            } else {
                return Ok(());
            }
        };
        
        self.gdb_output.push(format!("[Variables]\n{}", output));
        self.keep_last_n_lines(20);
        Ok(())
    }

    fn keep_last_n_lines(&mut self, n: usize) {
        if self.gdb_output.len() > n {
            self.gdb_output.drain(0..self.gdb_output.len() - n);
        }
    }

    async fn render(&self) -> Result<(), String> {
        let mut stdout = io::stdout();

        execute!(stdout, Clear(ClearType::All))
            .map_err(|e| format!("Failed to clear screen: {}", e))?;

        let split_line = self.terminal_height / 2;

        self.render_source_view(&mut stdout, split_line)?;
        self.render_gdb_view(&mut stdout, split_line)?;
        self.render_status_line(&mut stdout)?;

        stdout.flush()
            .map_err(|e| format!("Failed to flush stdout: {}", e))?;

        Ok(())
    }

    fn render_source_view(&self, stdout: &mut io::Stdout, split_line: u16) -> Result<(), String> {
        execute!(
            stdout,
            MoveTo(0, 0),
            SetForegroundColor(Color::Cyan),
            Print("DSL Source Code"),
            ResetColor
        )
        .map_err(|e| format!("Failed to render source header: {}", e))?;

        let start_line = self.current_line.saturating_sub(split_line.saturating_sub(3) as usize);
        let end_line = (start_line + split_line.saturating_sub(3) as usize).min(self.source_lines.len());

        for (i, line_idx) in (start_line..end_line).enumerate() {
            let y = (i + 2) as u16;
            let line = &self.source_lines[line_idx];
            let is_current = line_idx == self.current_line;

            if is_current {
                execute!(
                    stdout,
                    MoveTo(0, y),
                    SetForegroundColor(Color::Yellow),
                    Print(format!("> {:4} | {}", line_idx + 1, line)),
                    ResetColor
                )
                .map_err(|e| format!("Failed to render current line: {}", e))?;
            } else {
                execute!(
                    stdout,
                    MoveTo(0, y),
                    Print(format!("  {:4} | {}", line_idx + 1, line))
                )
                .map_err(|e| format!("Failed to render source line: {}", e))?;
            }
        }

        Ok(())
    }

    fn render_gdb_view(&self, stdout: &mut io::Stdout, split_line: u16) -> Result<(), String> {
        execute!(
            stdout,
            MoveTo(0, split_line),
            SetForegroundColor(Color::Green),
            Print("GDB Output"),
            ResetColor
        )
        .map_err(|e| format!("Failed to render GDB header: {}", e))?;

        let gdb_lines = split_line + 1;
        let output_height = self.terminal_height.saturating_sub(gdb_lines).saturating_sub(1);
        
        let start_idx = self.gdb_output.len().saturating_sub(output_height as usize);
        
        for (i, line) in self.gdb_output.iter().skip(start_idx).enumerate() {
            let y = gdb_lines + i as u16;
            if y < self.terminal_height - 1 {
                execute!(stdout, MoveTo(0, y), Print(line))
                    .map_err(|e| format!("Failed to render GDB line: {}", e))?;
            }
        }

        Ok(())
    }

    fn render_status_line(&self, stdout: &mut io::Stdout) -> Result<(), String> {
        let status = format!(
            "[s:step c:continue r:rev-step R:rev-cont b:breakpoint p:print-vars q:quit] Line: {}",
            self.current_line + 1
        );

        execute!(
            stdout,
            MoveTo(0, self.terminal_height - 1),
            SetForegroundColor(Color::Blue),
            Print(status),
            ResetColor
        )
        .map_err(|e| format!("Failed to render status line: {}", e))?;

        Ok(())
    }
}

pub async fn start_split_screen_debug(
    gdb_session: Arc<RwLock<Option<GdbSession>>>,
    debugger: Arc<TimeTravelDebugger>,
    source_file: Option<String>,
) -> Result<(), String> {
    let mut split_screen = SplitScreenDebugger::new(gdb_session, debugger, source_file)?;
    split_screen.start().await
}
