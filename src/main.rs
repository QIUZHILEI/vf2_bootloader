#![no_std]
#![no_main]
use core::arch::asm;
use core::panic::PanicInfo;
use core::{
    arch::global_asm,
    sync::atomic::{AtomicBool, Ordering},
};

use log::{error, info};
use riscv_utils::{csrc, csrs, mstatus::Mstatus, Mie, MIE, MSTATUS};

use vf2_bootloader::{init, load_kernel};
global_asm!(include_str!("./entry.S"));

unsafe extern "C" {
    fn _end();
    fn _bss_start();
    fn _bss_end();
}
/// 内核加载地址
const LOAD_ADDR: usize = 0x40000000;
const HART0_PLIC0_IE_BASE: usize = 0x0C00_2000;
const PLIC_IE_BASE: usize = 0x0C00_2080;
static BLOCK: AtomicBool = AtomicBool::new(true);

#[unsafe(no_mangle)]
pub extern "C" fn rust_entry(hart_id: usize) -> ! {
    csrc!(MSTATUS, Mstatus::mie.bits());
    csrs!(MSTATUS, Mstatus::mpp.bits());
    csrc!(MIE, (Mie::mtie | Mie::meie).bits());
    if hart_id == 0 {
        disable_interrupt(HART0_PLIC0_IE_BASE);
    } else {
        disable_interrupt(PLIC_IE_BASE + (hart_id as usize - 1) * 100);
    }
    // 让hart 1执行环境的初始化和内核加载过程，其余hart均循环
    if hart_id == 1 {
        clear_bss();
        init(_end as usize);
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
            mv a0, {hart_id}
            li a1, {load_addr}
            jr a1",
            hart_id = in(reg) hart_id,
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

fn clear_bss() {
    let mut addr = _bss_start as usize;
    while addr < _bss_end as usize {
        unsafe {
            (addr as *mut usize).write(0);
        }
        addr += size_of::<usize>();
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
