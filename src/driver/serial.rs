use spin::Mutex;
use lazy_static::lazy_static;
use x86_64::instructions::port::Port;
use core::fmt::{Write, Result, Arguments};

pub struct Serial {
    data: Port<u8>,
    int_en: Port<u8>,
    fifo_ctrl: Port<u8>,
    line_ctrl: Port<u8>,
    modem_ctrl: Port<u8>,
    line_sts: Port<u8>,
}

impl Serial {
    pub const unsafe fn new(base: u16) -> Self {
        Self {
            data: Port::new(base),
            int_en: Port::new(base + 1),
            fifo_ctrl: Port::new(base + 2),
            line_ctrl: Port::new(base + 3),
            modem_ctrl: Port::new(base + 4),
            line_sts: Port::new(base + 5),
        }
    }

    pub fn init(&mut self, divisor: u16) {
        unsafe {
            self.int_en.write(0x00);
            self.line_ctrl.write(0x80);

            self.data.write(divisor as u8);
            self.int_en.write((divisor >> 8) as u8);

            self.line_ctrl.write(0x03);
            self.fifo_ctrl.write(0xC7);
            self.modem_ctrl.write(0x0B);

            self.int_en.write(0x01);
        }
    }

    pub fn send(&mut self, data: u8) {
        unsafe {
            while self.line_sts.read() & 0x20 == 0 {
                core::sync::atomic::spin_loop_hint()
            }
            self.data.write(data);
        }
    }

    pub fn receive(&mut self) -> u8 {
        unsafe {
            while self.line_sts.read() & 1 == 0 {
                core::sync::atomic::spin_loop_hint()
            }
            self.data.read()
        }
    }

    pub fn write_fmt(mut self: &mut Self, args: Arguments<'_>) -> Result {
        core::fmt::write(&mut self, args)
    }
}

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref COM1: Mutex<Serial> = {
        let mut serial_port = unsafe { Serial::new(0x3f8) };
        serial_port.init(3);
        Mutex::new(serial_port)
    };
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        {
            use $crate::driver::serial::COM1; // Fuck u rust
            $crate::write_to!(COM1, $($arg)*);
        }
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}