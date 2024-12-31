use dw_sd::DwMmcHost;
use lego_device::BlockDevice;
const SDIO_BASE: usize = 0x16020000;
const MTIME_BASE: usize = 0x0200_BFF8;
const TIME_BASE: usize = 4000000;
fn get_macros() -> usize {
    let now = unsafe { (MTIME_BASE as *mut usize).read() };
    now * 1000000 / TIME_BASE
}

static mut MMC: DwMmcHost = DwMmcHost::new(SDIO_BASE, get_macros);
pub fn init() {
    let dw_mmc = unsafe { blk_dev_mut() };
    dw_mmc.init().unwrap();
}

pub fn read_block(lba: usize, blk: &mut [u8]) {
    let mmc = unsafe { blk_dev_mut() };
    mmc.read_block(lba, blk).unwrap();
}

#[inline]
pub unsafe fn blk_dev_mut() -> &'static mut dyn BlockDevice {
    (&raw mut MMC as *mut dyn BlockDevice).as_mut().unwrap()
}
