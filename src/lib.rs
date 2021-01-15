#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "tests_main"]

#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(const_in_array_repeat_expressions)]
#![feature(alloc_error_handler)]
#![feature(exclusive_range_pattern)]

extern crate alloc;

pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod allocator;
pub mod utils;
pub mod driver;

use core::panic::PanicInfo;
use memory::{ BootInfoFrameAllocator, PAGE_ALLOCATOR, MAPPER };
use utils::{QemuExitCode, exit_qemu, hlt_loop};
use x86_64::VirtAddr;
use bootloader::BootInfo;

pub fn init(boot_info: &'static BootInfo){
    gdt::init();

    unsafe {
        *PAGE_ALLOCATOR.lock() = Some(BootInfoFrameAllocator::init(&boot_info.memory_map));
        let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
        *MAPPER.lock() = Some(memory::init(phys_mem_offset));
    }
    interrupts::init_idt();

    unsafe {
        use x86_64::instructions::port::Port;

        let mut port = Port::new(0x21);
        port.write(0 as u8); // enable all interrupts on pics

        interrupts::PICS.lock().initialize()
    };

    x86_64::instructions::interrupts::enable();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[cfg(test)]
use bootloader::entry_point;

#[cfg(test)]
entry_point!(test_kernel_main);

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[cfg(test)]
pub fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);
    tests_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}