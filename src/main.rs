#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(sos::test_runner)]
#![reexport_test_harness_main = "tests_main"]

extern crate alloc;

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use sos::println;
use sos::utils::hlt_loop;

entry_point!(kernel_start);

#[allow(unconditional_panic)]
pub fn kernel_start(boot_info: &'static BootInfo) -> ! {
    sos::init(boot_info);

    println!("Hello, world");
    sos::serial_println!("Hello from SOS");

    #[cfg(test)]
    tests_main();

    hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    sos::test_panic_handler(info)
}
