use esp32_sys::*;

use core::fmt::Write;

pub struct Uart {
    pub port: uart_port_t,
}

impl Write for Uart {
    fn write_str(&mut self, output: &str) -> core::fmt::Result {
        unsafe {
            uart_write_bytes(self.port, output.as_ptr() as *const _, output.len());
        }
        Ok(())
    }
}
