use crate::{get_byte, write_byte};
use log::error;

const SPACE: u8 = 32;
const LINE_FEED: u8 = 10;
const ENTER: u8 = 13;
const NULL: u8 = 0;
const BACKSPACE: u8 = 8;
const ESC: u8 = 27;
const ANGLE_BRACKETS: u8 = 62;

#[inline]
fn is_digit(ch: u8) -> bool {
    ch >= 48 && ch < 58
}
#[inline]
fn is_letter(ch: u8) -> bool {
    (ch >= 65 && ch < 91) || (ch >= 97 && ch < 123)
}
#[inline]
fn is_symbol(ch: u8) -> bool {
    (ch >= 32 && ch <= 47) || (ch >= 91 && ch <= 96) || (ch >= 123 && ch <= 126)
}

const BUF_SIZE: usize = 16;
pub struct Console {
    buf: [u8; BUF_SIZE],
    len: usize,
    cursor: usize,
}

impl Console {
    pub fn new() -> Self {
        Self {
            buf: [0; BUF_SIZE],
            len: 0,
            cursor: 0,
        }
    }

    pub fn wait_for_input(&mut self) -> Option<&[u8]> {
        write_byte(ANGLE_BRACKETS);
        write_byte(ANGLE_BRACKETS);
        write_byte(SPACE);
        self.len = 0;
        self.cursor = 0;
        loop {
            if let Some(ch) = get_byte() {
                if ch == NULL || ch == LINE_FEED || ch == ENTER {
                    write_byte(LINE_FEED);
                    if self.len == 0 {
                        return None;
                    } else {
                        return Some(&self.buf[0..self.len]);
                    }
                } else if is_digit(ch) || is_letter(ch) || is_symbol(ch) {
                    if self.len == self.buf.len() {
                        error!("\nToo many characters entered, please re-enter!");
                        return None;
                    }
                    self.adjust_append();
                    self.buf[self.cursor] = ch;
                    self.len += 1;
                    self.re_print();
                    self.cursor += 1;
                    for _ in 0..self.len - self.cursor {
                        write_byte(BACKSPACE);
                    }
                } else if ch == BACKSPACE && self.len > 0 && self.cursor > 0 {
                    self.adjust_remove();
                    self.cursor -= 1;
                    write_byte(BACKSPACE);
                    self.re_print();
                    for _ in 0..self.len - self.cursor {
                        write_byte(BACKSPACE);
                    }
                    self.len -= 1;
                } else {
                    self.handle_transfer_char(ch);
                }
            }
        }
    }

    fn handle_transfer_char(&mut self, ch: u8) {
        if ch == ESC {
            loop {
                if let Some(c) = get_byte() {
                    let left_square_bracket = c;
                    loop {
                        if let Some(c) = get_byte() {
                            if left_square_bracket == 0x5b {
                                if c == 67 {
                                    if self.cursor < self.len {
                                        self.cursor += 1;
                                        write_byte(0x1B);
                                        write_byte(0x5B);
                                        write_byte(67);
                                    }
                                }
                                if c == 68 {
                                    if self.cursor > 0 {
                                        self.cursor -= 1;
                                        write_byte(BACKSPACE);
                                    }
                                }
                            }
                            break;
                        }
                    }
                    break;
                }
            }
        }
    }

    fn adjust_append(&mut self) {
        for index in (self.cursor..self.len).rev() {
            self.buf[index + 1] = self.buf[index];
        }
    }

    fn adjust_remove(&mut self) {
        for index in self.cursor..self.len {
            self.buf[index - 1] = self.buf[index];
        }
        self.buf[self.len - 1] = SPACE;
    }

    fn re_print(&mut self) {
        for index in self.cursor..self.len {
            write_byte(self.buf[index]);
        }
    }
}
