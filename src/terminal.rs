//! Terminal manager using crossterm: raw mode, alternate screen, and input polling.

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout, Write};
use std::time::Duration;

pub struct TerminalManager {
    is_raw: bool,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self { is_raw: false }
    }

    pub fn init(&mut self) {
        let _ = terminal::enable_raw_mode();
        self.is_raw = true;

        let mut out = stdout();
        let _ = execute!(out, EnterAlternateScreen, cursor::Hide);
        let _ = out.flush();
    }

    pub fn restore(&mut self) {
        if self.is_raw {
            let mut out = stdout();
            let _ = execute!(out, cursor::Show, LeaveAlternateScreen);
            let _ = out.flush();
            let _ = terminal::disable_raw_mode();
            self.is_raw = false;
        }
    }

    pub fn size() -> (usize, usize) {
        match terminal::size() {
            Ok((w, h)) => (w as usize, h as usize),
            Err(_) => (120, 40),
        }
    }

    pub fn poll_key() -> Option<KeyCode> {
        if let Ok(true) = event::poll(Duration::from_millis(0)) {
            if let Ok(Event::Key(KeyEvent { code, kind, .. })) = event::read() {
                if kind == KeyEventKind::Press {
                    return Some(code);
                }
            }
        }
        None
    }
}

impl Drop for TerminalManager {
    fn drop(&mut self) {
        self.restore();
    }
}
