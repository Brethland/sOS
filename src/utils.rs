/// write to any device that implemented fmt::Writer trait
#[macro_export]
macro_rules! write_to {
    ($port:expr, $($arg:tt)*) => {
        x86_64::instructions::interrupts::without_interrupts(|| {
        $port
            .lock().write_fmt(format_args!($($arg)*))
            .expect("Writing failed");
        });
    };
}

/// some configs for QEMU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

/// loop forever
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}