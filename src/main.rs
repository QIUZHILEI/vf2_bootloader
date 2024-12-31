#![no_std]
#![no_main]
use core::arch::asm;
use core::panic::PanicInfo;
use core::{
    arch::global_asm,
    sync::atomic::{AtomicBool, Ordering},
};
use lego_arch::mhartid;
use log::info;

use vf2_bootloader::{init, load_kernel, println};
global_asm!(include_str!("./entry.S"));
extern "C" {
    static _bss_start: usize;
    static _bss_end: usize;
}

const KERNEL_NAME: &str = "LEGO.OS";
static BLOCK: AtomicBool = AtomicBool::new(true);
const LOAD_ADDR: usize = 0x40000000;
#[no_mangle]
pub extern "C" fn rust_entry(code_end: usize) -> ! {
    // 让hart 1执行环境的初始化和内核加载过程，其余hart均循环
    if mhartid::read() == 1 {
        clear_bss();
        init(code_end);
        load_kernel(LOAD_ADDR, KERNEL_NAME);
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

fn clear_bss() {
    let mut bss = unsafe { _bss_start as *mut usize };
    let bss_end = unsafe { _bss_end as *mut usize };
    unsafe {
        while bss.lt(&bss_end) {
            (*bss) = 0;
            bss = bss.add(1);
        }
    }
}

#[panic_handler]
pub fn panic(println: &PanicInfo) -> ! {
    if let Some(location) = println.location() {
        println!(
            "panic occurred in file '{}' at line {}",
            location.file(),
            location.line(),
        );
    } else {
        println!("panic occurred but can't get location information");
    }

    println!("panic message: {:?}", println.message());
    loop {}
}
