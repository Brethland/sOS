#![no_std]

extern crate alloc;

use alloc::vec::Vec;
pub use uefi::proto::console::gop::ModeInfo;
pub use uefi::table::boot::{MemoryAttribute, MemoryDescriptor, MemoryType};

#[repr(C)]
pub struct BootInfo {
    pub memory_map: Vec<&'static MemoryDescriptor>,
    pub physical_memory_offset: u64,
    pub acpi_addr: u64,
    pub smbios_addr: u64,
}