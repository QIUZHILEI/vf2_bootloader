use core::fmt::Write;

use lego_device::{CharDevice, Device};
use uart_8250::Uart;
const CLK_HZ: u64 = 24000000;
const BAUD_RATE: u64 = 115200;
pub const UART_BASE: usize = 0x10000000;

static mut UART: UartWrapper = UartWrapper(Uart::new(UART_BASE, CLK_HZ, BAUD_RATE));
struct UartWrapper(Uart);

impl Write for UartWrapper {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes()
            .iter()
            .for_each(|byte| while self.0.put_char(*byte).is_err() {});
        Ok(())
    }
}

pub fn init() {
    let uart_ref = unsafe { (&raw mut UART).as_mut().unwrap() };
    uart_ref.0.init().unwrap();
}

pub fn get_byte() -> Option<u8> {
    let uart = unsafe { (&raw mut UART).as_mut().unwrap() };
    uart.0.get_char().map_or(None, |c| Some(c))
}

pub fn write_byte(byte: u8) {
    let uart = unsafe { (&raw mut UART).as_mut().unwrap() };
    while uart.0.put_char(byte).is_err() {}
}

pub fn uart_mut() -> &'static mut dyn Write {
    let uart = unsafe { (&raw mut UART).as_mut().unwrap() };
    uart as _
}

#[macro_export]
macro_rules! println {
    () => {{
        writeln!($crate::uart_mut()).unwrap();
    }};
    ($($arg:tt)*) => {{
        writeln!($crate::uart_mut(),$($arg)*).unwrap();
    }};
}

#[macro_export]
macro_rules! print {
    () => {{
        write!($crate::uart_mut()).unwrap();
    }};
    ($($arg:tt)*) => {{
        write!($crate::uart_mut(),$($arg)*).unwrap();
    }};
}
