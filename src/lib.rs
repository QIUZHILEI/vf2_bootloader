#![no_std]
mod logger;
mod mem;
mod sd;
mod uart;
use core::slice;

mod fat;
use alloc::string::ToString;
use fat::Volume;
use gpt::{GptLayout, PRIMARY_HEADER_LBA};
use log::{debug, info};
pub use uart::*;
extern crate alloc;
const KERNEL_NAME: &str = "TOM.OS";
pub fn init(code_end: usize) {
    uart::init_uart();
    logger::init(log::Level::Info);
    sd::init();
    mem::init(code_end);
    info!("environment initialized");
}

pub fn load_kernel(load_addr: usize) -> usize {
    let mut buf = [0u8; 512];
    let mut gpt = GptLayout::new();
    let part_index = find_root_partition(&mut gpt, &mut buf);
    let part = gpt.partition(part_index).unwrap();
    let volume: Volume = init_fat(part.start_lba as usize);
    info!("fs init success");
    if let Some((lba, size)) = volume.find(KERNEL_NAME, unsafe { sd::blk_dev_mut() }) {
        load_to_mem(lba, size, load_addr);
        size
    } else {
        panic!("Can not find kernel {}", KERNEL_NAME)
    }
}

fn find_root_partition(gpt: &mut GptLayout, blk: &mut [u8]) -> usize {
    let root_uuid = "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7";
    info!("find root partition...");
    sd::read_block(PRIMARY_HEADER_LBA, blk);
    gpt.init_primary_header(blk).unwrap();
    let part_start = gpt.primary_header().part_start as usize;
    sd::read_block(part_start, blk);
    let part_entry_size = 128;
    for index in 0..(blk.len() / part_entry_size) {
        let start = part_entry_size * index;
        let end = start + part_entry_size;
        let entry_index = index + 1;
        gpt.init_partition(&blk[start..end], entry_index);
    }

    let root_part = gpt.partition(4).unwrap();
    if root_part.part_type_guid.to_string().eq(root_uuid) {
        info!("find root partition {}", 4);
    }
    4
}

fn init_fat(start_lba: usize) -> Volume {
    info!("init fat file system");
    let mut bpb = [0u8; 512];
    sd::read_block(start_lba, &mut bpb[..]);
    let mut volume = Volume::new(start_lba);
    volume.init_bpb(&bpb);
    debug!("{volume:?}");
    volume
}

fn load_to_mem(lba: usize, size: usize, load_addr: usize) {
    let blocks = if size % 512 == 0 {
        size / 512
    } else {
        size / 512 + 1
    };
    for blk_idx in 0..blocks {
        let block_lba = blk_idx + lba;
        let buf = unsafe {
            let ptr = (load_addr as *mut u8).add(blk_idx * 512);
            slice::from_raw_parts_mut(ptr, 512)
        };
        sd::read_block(block_lba, buf);
    }
    info!("kernel load success, and kernel size is {}", size);
}
