#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(abi_efiapi)]

mod memory;
mod file;

#[macro_use]
extern crate alloc;
extern crate rlibc;

use uefi::prelude::*;
use uefi::table::cfg::{ACPI2_GUID, SMBIOS_GUID};
use sos_boot::BootInfo;
use memory::UEFIFrameAllocator;
use x86_64::registers::control::*;
use xmas_elf::ElfFile;
use alloc::boxed::Box;
use alloc::vec::Vec;

// the address of loaded kernel
static mut ENTRY_BASE: usize = 0;

unsafe fn jump_to_entry(boot_info: *const BootInfo, rsp: u64) -> ! {
    llvm_asm!("call $0" :: "r"(ENTRY_BASE), "{rsp}"(rsp), "{rdi}"(boot_info) :: "intel");
    loop {}
}

#[entry]
fn efi_main(img: uefi::Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("unable to initialize service");

    let bs = st.boot_services();

    let acpi_addr = st.config_table().iter().find(|entry| entry.guid == ACPI2_GUID)
        .expect("failed to find ACPI RSDP").address;
    let smbios_addr = st.config_table().iter().find(|entry| entry.guid == SMBIOS_GUID)
        .expect("failed to find SMBIOS").address;

    let kernel = ElfFile::new({
        let mut file = file::open_file(bs, "\\EFI\\kernel.efi");
        file::load_file(bs, &mut file)
    }).expect("failed to parse ELF");
    unsafe {
        ENTRY_BASE = kernel.header.pt2.entry_point() as usize;
    }

    let mmap_size = st.boot_services().memory_map_size();
    let mmap_storage = Box::leak(vec![0; mmap_size * 2].into_boxed_slice());
    let (_, mmap_iter) = st.boot_services().memory_map(mmap_storage)
        .expect_success("failed to get memory map");
    let phys_addr = mmap_iter.map(|m| m.phys_start + m.page_count * 0x1000).max().unwrap().max(0x1_0000_0000);

    let mut level4_table = memory::level4_page_table();
    unsafe {
        Cr0::update(|f| f.remove(Cr0Flags::WRITE_PROTECT));
        Efer::update(|f| f.insert(EferFlags::NO_EXECUTE_ENABLE));
    }
    memory::map_elf(&kernel, &mut level4_table, &mut UEFIFrameAllocator(bs))
        .expect("failed to map ELF");
    memory::map_stack(0xFFFF_FF01_0000_0000, 512, &mut level4_table, &mut UEFIFrameAllocator(bs))
        .expect("failed to map kernel stack");
    memory::map_physical_memory(0xFFFF_8000_0000_0000, phys_addr, &mut level4_table, &mut UEFIFrameAllocator(bs));
    unsafe {
        Cr0::update(|f| f.insert(Cr0Flags::WRITE_PROTECT));
    }

    let mut memory_map = Vec::with_capacity(128);
    let (_, mmap_iter) = st.exit_boot_services(img, mmap_storage)
        .expect_success("failed to exit boot services");
    for m in mmap_iter {
        memory_map.push(m);
    }

    let rsp = 0xFFFF_FF01_0000_0000 + 512 * 0x1000;
    let boot_info = BootInfo {
        memory_map,
        physical_memory_offset: 0xFFFF_8000_0000_0000,
        acpi_addr: acpi_addr as u64,
        smbios_addr: smbios_addr as u64,
    };

    unsafe {
        jump_to_entry(&boot_info, rsp)
    }
}