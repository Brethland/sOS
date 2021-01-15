use x86_64::{
    structures::paging::{PageTable, OffsetPageTable, PhysFrame, Size4KiB, FrameAllocator},
    VirtAddr,
    PhysAddr,
};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static!{
    pub static ref PAGE_ALLOCATOR: Mutex<Option<BootInfoFrameAllocator>> = {
        Mutex::new(None)
    };
}

lazy_static!{
    pub static ref MAPPER: Mutex<Option<OffsetPageTable<'static>>> = {
        Mutex::new(None)
    };
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);

        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let physical_mem = level_4_table_frame.start_address();
    let virtual_mem = physical_memory_offset + physical_mem.as_u64();
    let page_table_ptr: *mut PageTable = virtual_mem.as_mut_ptr();

    &mut *page_table_ptr
}
