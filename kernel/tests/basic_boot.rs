#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(sos::test_runner)]
#![reexport_test_harness_main = "tests_main"]

use core::panic::PanicInfo;
use sos::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    tests_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    sos::test_panic_handler(info)
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}
