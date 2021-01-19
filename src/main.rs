#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(sos::test_runner)]
#![reexport_test_harness_main = "tests_main"]

extern crate alloc;

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use sos::println;
use sos::task::{Task, executor::{Executor, SPAWNER}, keyboard::print_keypress};

entry_point!(kernel_start);

#[allow(unconditional_panic)]
pub fn kernel_start(boot_info: &'static BootInfo) -> ! {
    sos::init(boot_info);

    #[cfg(test)]
        tests_main();

    println!("Hello, world");
    sos::serial_println!("Hello from SOS");

    let mut executor = Executor::new();
    SPAWNER.add(Task::new(print_keypress()));
    executor.run();

}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use sos::utils::hlt_loop;

    println!("{}", info);
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    sos::test_panic_handler(info)
}
