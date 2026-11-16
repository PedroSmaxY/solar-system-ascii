//! Terminal manager using crossterm: raw mode, alternate screen, mouse capture, and input polling.

use crossterm::{
    cursor,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static PANIC_HOOK_SET: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerminalEvent {
    Key(KeyCode, KeyModifiers),
    MouseScrollUp,
    MouseScrollDown,
    MouseDrag { dx: i32, dy: i32 },
    MouseClick { col: u16, row: u16 },
}

pub struct TerminalManager {
    is_raw: bool,
    last_mouse_pos: Option<(u16, u16)>,
    mouse_down_pos: Option<(u16, u16)>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            is_raw: false,
            last_mouse_pos: None,
            mouse_down_pos: None,
        }
    }

    pub fn init(&mut self) {
        let _ = terminal::enable_raw_mode();
        self.is_raw = true;

        if !PANIC_HOOK_SET.swap(true, Ordering::SeqCst) {
            let default_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                let mut out = stdout();
                let _ = execute!(out, DisableMouseCapture, cursor::Show, LeaveAlternateScreen);
                let _ = out.flush();
                let _ = terminal::disable_raw_mode();
                default_hook(info);
            }));
        }

        let mut out = stdout();
        let _ = execute!(out, EnterAlternateScreen, cursor::Hide, EnableMouseCapture);
        let _ = out.flush();
    }

    pub fn restore(&mut self) {
        if self.is_raw {
            let mut out = stdout();
            let _ = execute!(out, DisableMouseCapture, cursor::Show, LeaveAlternateScreen);
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

    pub fn poll_event(&mut self) -> Option<TerminalEvent> {
        if let Ok(true) = event::poll(Duration::from_millis(0)) {
            match event::read() {
                Ok(Event::Key(KeyEvent { code, modifiers, kind, .. })) => {
                    if matches!(kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                        return Some(TerminalEvent::Key(code, modifiers));
                    }
                }
                Ok(Event::Mouse(MouseEvent { kind, column, row, .. })) => {
                    match kind {
                        MouseEventKind::ScrollUp => return Some(TerminalEvent::MouseScrollUp),
                        MouseEventKind::ScrollDown => return Some(TerminalEvent::MouseScrollDown),
                        MouseEventKind::Down(MouseButton::Left) => {
                            self.last_mouse_pos = Some((column, row));
                            self.mouse_down_pos = Some((column, row));
                        }
                        MouseEventKind::Drag(MouseButton::Left) => {
                            if let Some((last_c, last_r)) = self.last_mouse_pos {
                                let dx = column as i32 - last_c as i32;
                                let dy = row as i32 - last_r as i32;
                                self.last_mouse_pos = Some((column, row));
                                if dx.abs() > 0 || dy.abs() > 0 {
                                    self.mouse_down_pos = None;
                                    return Some(TerminalEvent::MouseDrag { dx, dy });
                                }
                            } else {
                                self.last_mouse_pos = Some((column, row));
                            }
                        }
                        MouseEventKind::Up(MouseButton::Left) => {
                            self.last_mouse_pos = None;
                            if let Some((down_c, down_r)) = self.mouse_down_pos.take() {
                                if down_c == column && down_r == row {
                                    return Some(TerminalEvent::MouseClick { col: column, row });
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
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
