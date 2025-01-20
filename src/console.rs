use crate::{get_byte, print};
use log::error;

const LINE_FEED: u8 = 10;
const BUF_SIZE: usize = 32;
pub struct Console {
    buf: [u8; BUF_SIZE],
    tail: usize,
}

impl Console {
    pub fn new() -> Self {
        Self {
            buf: [0; BUF_SIZE],
            tail: 0,
        }
    }
    pub fn wait_for_input(&mut self) -> Option<&[u8]> {
        print!("$ ");
        loop {
            if self.tail >= self.buf.len() {
                error!("\nToo many characters entered, please re-enter!");
                self.tail = 0;
                self.buf = [0; BUF_SIZE];
                return None;
            }
            if let Some(c) = get_byte() {
                if c == LINE_FEED {
                    let res = &self.buf[0..self.tail];
                    self.tail = 0;
                    return Some(res);
                }
                self.buf[self.tail] = c;
                self.tail += 1;
            }
        }
    }
}
