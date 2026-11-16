//! Dual-mode ANSI TrueColor frame buffer.
//! Supports:
//! 1. Classic ASCII Mode: char grid + 24-bit TrueColor foreground.
//! 2. High-Res Half-Block Mode: Unicode '▀' with independent foreground (top) and background (bottom)
//!    colors, effectively doubling the vertical pixel resolution.


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
    // Pre-allocated reusable hit mask for raymarching passes
    pub hit_mask: Vec<bool>,
    // Pre-allocated reusable byte buffer for ANSI TrueColor output (zero allocations per frame)
    pub byte_buffer: Vec<u8>,
}

#[inline(always)]
fn push_u8_fast(buf: &mut Vec<u8>, mut n: u8) {
    if n >= 100 {
        buf.push(b'0' + n / 100);
        n %= 100;
        buf.push(b'0' + n / 10);
        n %= 10;
        buf.push(b'0' + n);
    } else if n >= 10 {
        buf.push(b'0' + n / 10);
        n %= 10;
        buf.push(b'0' + n);
    } else {
        buf.push(b'0' + n);
    }
}

#[inline(always)]
fn push_color_seq(buf: &mut Vec<u8>, prefix: &[u8], color: [u8; 3]) {
    buf.extend_from_slice(prefix);
    push_u8_fast(buf, color[0]);
    buf.push(b';');
    push_u8_fast(buf, color[1]);
    buf.push(b';');
    push_u8_fast(buf, color[2]);
    buf.push(b'm');
}

#[inline(always)]
fn color_diff_exceeds(c1: [u8; 3], c2: [u8; 3], thresh: u32) -> bool {
    (c1[0] as i32 - c2[0] as i32).abs() as u32 > thresh
        || (c1[1] as i32 - c2[1] as i32).abs() as u32 > thresh
        || (c1[2] as i32 - c2[2] as i32).abs() as u32 > thresh
}

const ANSI_COLOR_EPSILON: u32 = 3;

impl FrameBuffer {
    pub fn new(cols: usize, lines: usize, mode: RenderMode) -> Self {
        let height = match mode {
            RenderMode::Ascii => lines,
            RenderMode::HalfBlock => lines * 2,
        };
        let pixels = vec![Pixel::BLACK; cols * height];
        let hit_mask = vec![false; cols * height];
        let byte_buffer = Vec::with_capacity(cols * lines * 24);
        Self { cols, lines, mode, pixels, hit_mask, byte_buffer }
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
            self.hit_mask = vec![false; cols * height];
            self.byte_buffer = Vec::with_capacity(cols * lines * 24);
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
            self.hit_mask = vec![false; self.cols * height];
            self.byte_buffer = Vec::with_capacity(self.cols * self.lines * 24);
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

    /// Render internal grid directly into the persistent, reusable byte buffer (zero heap allocations).
    pub fn flush_to_bytes(&mut self) -> &[u8] {
        self.byte_buffer.clear();
        // Synchronized output (DEC Mode 2026) to prevent screen tearing on modern terminals
        self.byte_buffer.extend_from_slice(b"\x1b[?2026h\x1b[H");

        match self.mode {
            RenderMode::Ascii => {
                let mut curr_fg: Option<[u8; 3]> = None;
                for y in 0..self.lines {
                    for x in 0..self.cols {
                        let p = &self.pixels[y * self.cols + x];
                        if p.ch == ' ' {
                            if curr_fg.is_some() {
                                self.byte_buffer.extend_from_slice(b"\x1b[0m");
                                curr_fg = None;
                            }
                            self.byte_buffer.push(b' ');
                        } else {
                            let fg = [p.r, p.g, p.b];
                            if curr_fg != Some(fg) {
                                push_color_seq(&mut self.byte_buffer, b"\x1b[38;2;", fg);
                                curr_fg = Some(fg);
                            }
                            let mut utf8_buf = [0u8; 4];
                            let s = p.ch.encode_utf8(&mut utf8_buf);
                            self.byte_buffer.extend_from_slice(s.as_bytes());
                        }
                    }
                    if curr_fg.is_some() {
                        self.byte_buffer.extend_from_slice(b"\x1b[0m");
                        curr_fg = None;
                    }
                    if y + 1 < self.lines {
                        self.byte_buffer.extend_from_slice(b"\r\n");
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

                        // 1. Célula preta (espaço vazio sem texto sobreposto)
                        if fg == [0, 0, 0] && bg == [0, 0, 0] && !p_top.is_text {
                            if curr_fg.is_some() || curr_bg.is_some() {
                                self.byte_buffer.extend_from_slice(b"\x1b[0m");
                                curr_fg = None;
                                curr_bg = None;
                            }
                            self.byte_buffer.push(b' ');
                            continue;
                        }

                        // 2. HUD ou texto de label sobreposto
                        if p_top.is_text {
                            if curr_bg != Some(bg) {
                                if bg == [0, 0, 0] {
                                    self.byte_buffer.extend_from_slice(b"\x1b[49m");
                                    curr_bg = None;
                                } else {
                                    push_color_seq(&mut self.byte_buffer, b"\x1b[48;2;", bg);
                                    curr_bg = Some(bg);
                                }
                            }
                            if curr_fg != Some(fg) {
                                push_color_seq(&mut self.byte_buffer, b"\x1b[38;2;", fg);
                                curr_fg = Some(fg);
                            }
                            let mut utf8_buf = [0u8; 4];
                            let s = p_top.ch.encode_utf8(&mut utf8_buf);
                            self.byte_buffer.extend_from_slice(s.as_bytes());
                            continue;
                        }

                        // 3. Otimização quando fg == bg (ambas as metades têm a mesma cor exata)
                        if fg == bg {
                            let bg_matches = match curr_bg {
                                Some(c) => !color_diff_exceeds(c, bg, ANSI_COLOR_EPSILON),
                                None => false,
                            };
                            let fg_matches = match curr_fg {
                                Some(c) => !color_diff_exceeds(c, fg, ANSI_COLOR_EPSILON),
                                None => false,
                            };
                            if bg_matches {
                                self.byte_buffer.push(b' ');
                                continue;
                            } else if fg_matches {
                                self.byte_buffer.extend_from_slice("█".as_bytes());
                                continue;
                            } else {
                                push_color_seq(&mut self.byte_buffer, b"\x1b[48;2;", bg);
                                curr_bg = Some(bg);
                                self.byte_buffer.push(b' ');
                                continue;
                            }
                        }

                        // 4. Normal Half-Block '▀': foreground é topo, background é base
                        let need_fg = match curr_fg {
                            Some(c) => color_diff_exceeds(c, fg, ANSI_COLOR_EPSILON),
                            None => true,
                        };
                        if need_fg {
                            push_color_seq(&mut self.byte_buffer, b"\x1b[38;2;", fg);
                            curr_fg = Some(fg);
                        }

                        let need_bg = match curr_bg {
                            Some(c) => color_diff_exceeds(c, bg, ANSI_COLOR_EPSILON),
                            None => true,
                        };
                        if need_bg {
                            push_color_seq(&mut self.byte_buffer, b"\x1b[48;2;", bg);
                            curr_bg = Some(bg);
                        }
                        self.byte_buffer.extend_from_slice("▀".as_bytes());
                    }

                    if curr_fg.is_some() || curr_bg.is_some() {
                        self.byte_buffer.extend_from_slice(b"\x1b[0m");
                        curr_fg = None;
                        curr_bg = None;
                    }
                    if y + 1 < self.lines {
                        self.byte_buffer.extend_from_slice(b"\r\n");
                    }
                }
            }
        }

        self.byte_buffer.extend_from_slice(b"\x1b[?2026l");
        &self.byte_buffer
    }

    /// Render internal grid into an ANSI TrueColor string.
    pub fn flush_to_string(&mut self) -> String {
        let bytes = self.flush_to_bytes();
        String::from_utf8_lossy(bytes).into_owned()
    }
}
