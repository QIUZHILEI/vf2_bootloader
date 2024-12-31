use lego_device::{CharDevice, Device};
use uart_8250::Uart;

pub const UART_BASE: usize = 0x10000000;

pub static mut UART: UartWrapper = UartWrapper::new(UART_BASE);
pub struct UartWrapper {
    uart: Uart,
}

impl UartWrapper {
    const fn new(base_addr: usize) -> Self {
        Self {
            uart: Uart::new(base_addr, 13),
        }
    }
}

impl core::fmt::Write for UartWrapper {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes()
            .iter()
            .for_each(|byte| while self.uart.put_char(*byte).is_err() {});
        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    () => {{
        use core::fmt::Write;
        writeln!(unsafe{(&raw mut $crate::UART).as_mut().unwrap()}).unwrap();
    }};
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        writeln!(unsafe{(&raw mut $crate::UART).as_mut().unwrap()},$($arg)*).unwrap();
    }};
}

pub fn init_uart() {
    let uart_ref = unsafe { (&raw mut UART).as_mut().unwrap() };
    uart_ref.uart.init().unwrap();
}
