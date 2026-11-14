//! Dual-mode ANSI TrueColor frame buffer.
//! Supports:
//! 1. Classic ASCII Mode: char grid + 24-bit TrueColor foreground.
//! 2. High-Res Half-Block Mode: Unicode '▀' with independent foreground (top) and background (bottom)
//!    colors, effectively doubling the vertical pixel resolution.

use std::fmt::Write as FmtWrite;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderMode {
    Ascii,
    HalfBlock,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub ch: char,
    pub is_text: bool,
}

impl Pixel {
    pub const BLACK: Pixel = Pixel { r: 0, g: 0, b: 0, ch: ' ', is_text: false };

    pub fn new(r: u8, g: u8, b: u8, ch: char) -> Self {
        Self { r, g, b, ch, is_text: false }
    }

    pub fn text(r: u8, g: u8, b: u8, ch: char) -> Self {
        Self { r, g, b, ch, is_text: true }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, ch: ' ', is_text: false }
    }
}

pub struct FrameBuffer {
    pub cols: usize,
    pub lines: usize,
    pub mode: RenderMode,
    // Buffer for pixels: in HalfBlock mode, height is lines * 2
    pub pixels: Vec<Pixel>,
}

impl FrameBuffer {
    pub fn new(cols: usize, lines: usize, mode: RenderMode) -> Self {
        let height = match mode {
            RenderMode::Ascii => lines,
            RenderMode::HalfBlock => lines * 2,
        };
        let pixels = vec![Pixel::BLACK; cols * height];
        Self { cols, lines, mode, pixels }
    }

    pub fn resize(&mut self, cols: usize, lines: usize) {
        if self.cols != cols || self.lines != lines {
            self.cols = cols;
            self.lines = lines;
            let height = match self.mode {
                RenderMode::Ascii => lines,
                RenderMode::HalfBlock => lines * 2,
            };
            self.pixels = vec![Pixel::BLACK; cols * height];
        }
    }

    pub fn set_mode(&mut self, mode: RenderMode) {
        if self.mode != mode {
            self.mode = mode;
            let height = match self.mode {
                RenderMode::Ascii => self.lines,
                RenderMode::HalfBlock => self.lines * 2,
            };
            self.pixels = vec![Pixel::BLACK; self.cols * height];
        }
    }

    #[inline(always)]
    pub fn virtual_height(&self) -> usize {
        match self.mode {
            RenderMode::Ascii => self.lines,
            RenderMode::HalfBlock => self.lines * 2,
        }
    }

    #[inline(always)]
    pub fn set_pixel(&mut self, x: usize, y: usize, pix: Pixel) {
        let v_height = self.virtual_height();
        if x < self.cols && y < v_height {
            self.pixels[y * self.cols + x] = pix;
        }
    }

    /// Render internal grid into an optimized ANSI TrueColor string buffer.
    pub fn flush_to_string(&self) -> String {
        let mut out = String::with_capacity(self.cols * self.lines * 24);
        // Move cursor to top-left
        out.push_str("\x1b[H");

        match self.mode {
            RenderMode::Ascii => {
                let mut curr_fg: Option<[u8; 3]> = None;
                for y in 0..self.lines {
                    for x in 0..self.cols {
                        let p = &self.pixels[y * self.cols + x];
                        if p.ch == ' ' {
                            if curr_fg.is_some() {
                                out.push_str("\x1b[0m");
                                curr_fg = None;
                            }
                            out.push(' ');
                        } else {
                            let fg = [p.r, p.g, p.b];
                            if curr_fg != Some(fg) {
                                let _ = write!(out, "\x1b[38;2;{};{};{}m", fg[0], fg[1], fg[2]);
                                curr_fg = Some(fg);
                            }
                            out.push(p.ch);
                        }
                    }
                    if curr_fg.is_some() {
                        out.push_str("\x1b[0m");
                        curr_fg = None;
                    }
                    if y + 1 < self.lines {
                        out.push_str("\r\n");
                    }
                }
            }
            RenderMode::HalfBlock => {
                let mut curr_fg: Option<[u8; 3]> = None;
                let mut curr_bg: Option<[u8; 3]> = None;

                for y in 0..self.lines {
                    let y_top = y * 2;
                    let y_bot = y * 2 + 1;

                    for x in 0..self.cols {
                        let p_top = &self.pixels[y_top * self.cols + x];
                        let p_bot = &self.pixels[y_bot * self.cols + x];

                        let fg = [p_top.r, p_top.g, p_top.b];
                        let bg = [p_bot.r, p_bot.g, p_bot.b];

                        // If cell is completely black
                        if fg == [0, 0, 0] && bg == [0, 0, 0] && p_top.ch == ' ' && p_bot.ch == ' ' {
                            if curr_fg.is_some() || curr_bg.is_some() {
                                out.push_str("\x1b[0m");
                                curr_fg = None;
                                curr_bg = None;
                            }
                            out.push(' ');
                            continue;
                        }

                        // If HUD or character overlay is present
                        if p_top.is_text {
                            if curr_bg != Some(bg) {
                                if bg == [0, 0, 0] {
                                    out.push_str("\x1b[49m");
                                    curr_bg = None;
                                } else {
                                    let _ = write!(out, "\x1b[48;2;{};{};{}m", bg[0], bg[1], bg[2]);
                                    curr_bg = Some(bg);
                                }
                            }
                            if curr_fg != Some(fg) {
                                let _ = write!(out, "\x1b[38;2;{};{};{}m", fg[0], fg[1], fg[2]);
                                curr_fg = Some(fg);
                            }
                            out.push(p_top.ch);
                            continue;
                        }

                        // Normal Half-Block '▀': foreground is top, background is bottom
                        if curr_fg != Some(fg) {
                            let _ = write!(out, "\x1b[38;2;{};{};{}m", fg[0], fg[1], fg[2]);
                            curr_fg = Some(fg);
                        }
                        if curr_bg != Some(bg) {
                            let _ = write!(out, "\x1b[48;2;{};{};{}m", bg[0], bg[1], bg[2]);
                            curr_bg = Some(bg);
                        }
                        out.push('▀');
                    }

                    if curr_fg.is_some() || curr_bg.is_some() {
                        out.push_str("\x1b[0m");
                        curr_fg = None;
                        curr_bg = None;
                    }
                    if y + 1 < self.lines {
                        out.push_str("\r\n");
                    }
                }
            }
        }

        out
    }
}
