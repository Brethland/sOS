use uefi::proto::media::file::*;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::prelude::*;
use uefi::table::boot::{AllocateType, MemoryType};

pub fn open_file(bs: &BootServices, path: &str) -> RegularFile {
    let fs = bs.locate_protocol::<SimpleFileSystem>()
        .expect_success("failed to get file system");
    let fs = unsafe { &mut *fs.get() };

    let mut root_path = fs.open_volume().expect_success("failed to open volume");
    let handle = root_path.open(path, FileMode::Read, FileAttribute::empty())
        .expect_success("failed to open file");

    match handle.into_type().expect_success("failed to into_type") {
        FileType::Regular(regular) => regular,
        _ => panic!("invalid file type"),
    }
}

pub fn load_file(bs: &BootServices, file: &mut RegularFile) -> &'static mut [u8] {
    // our file name cannot exceed to 1000 chars.
    let mut info_buf = [0u8; 0x100];
    let info = file.get_info::<FileInfo>(&mut info_buf).expect_success("failed to get file info");

    // for preventing overflow
    let pages = info.file_size() as usize / 0x1000 + 1;
    let start_address = bs.allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, pages)
        .expect_success("failed to allocate pages");
    let buf = unsafe { core::slice::from_raw_parts_mut(start_address as *mut u8, pages * 0x1000) };
    let len = file.read(buf).expect_success("failed to read file");
    &mut buf[..len]
}