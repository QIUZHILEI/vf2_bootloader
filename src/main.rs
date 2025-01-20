#![no_std]
#![no_main]
use core::arch::asm;
use core::panic::PanicInfo;
use core::{
    arch::global_asm,
    sync::atomic::{AtomicBool, Ordering},
};

use lego_arch::{mhartid, mie, mstatus};
use log::{error, info};

use vf2_bootloader::{init, load_kernel};
global_asm!(include_str!("./entry.S"));
/// 内核加载地址
const LOAD_ADDR: usize = 0x40000000;
const HART0_PLIC0_IE_BASE: usize = 0x0C00_2000;
const PLIC_IE_BASE: usize = 0x0C00_2080;
static BLOCK: AtomicBool = AtomicBool::new(true);
#[no_mangle]
pub extern "C" fn rust_entry(code_end: usize) -> ! {
    let mie = mstatus::MStatus::mie;
    mstatus::clear_mask(mie);
    let mpp = mstatus::MStatus::mpp;
    mstatus::set_mask(mpp);
    let mie_mask: u64 = 1 << 11 | 1 << 7;
    mie::clear(mie_mask);
    let hart = mhartid::read();
    if hart == 0 {
        disable_interrupt(HART0_PLIC0_IE_BASE);
    } else {
        disable_interrupt(PLIC_IE_BASE + (hart as usize - 1) * 100);
    }

    // 让hart 1执行环境的初始化和内核加载过程，其余hart均循环
    if hart == 1 {
        init(code_end);
        load_kernel(LOAD_ADDR);
        BLOCK.store(false, Ordering::Relaxed);
        info!("prepare to jump to kernel execution");
    } else {
        // 内核未加载完成，一直循环等待
        while BLOCK.load(Ordering::Relaxed) {
            core::hint::spin_loop();
        }
    }
    // 内核加载完毕，所有hart均跳转到内核的入口处开始执行
    unsafe {
        asm!("
            li a0, {load_addr}
            jr a0",
            load_addr = const LOAD_ADDR,
            options(noreturn)
        )
    }
}

fn disable_interrupt(plic_ie_base: usize) {
    unsafe {
        *(plic_ie_base as *mut u32) = 0;
        *((plic_ie_base + 4) as *mut u32) = 0;
        *((plic_ie_base + 8) as *mut u32) = 0;
        *((plic_ie_base + 12) as *mut u32) = 0;
        *((plic_ie_base + 16) as *mut u32) &= 0xF8000000;
    }
}

#[panic_handler]
pub fn panic(println: &PanicInfo) -> ! {
    if let Some(location) = println.location() {
        error!(
            "panic occurred in file '{}' at line {}",
            location.file(),
            location.line(),
        );
    } else {
        error!("panic occurred but can't get location information");
    }

    error!("panic message: {:?}", println.message());
    loop {}
}
