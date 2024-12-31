# vf2_bootloader

vf2_bootloader用于取代Uboot运行于Visionfive 2平台，它是一个非常简单的程序可以用来从SD卡EFI分区加载自定义的操作系统内核。

## 简介

在 VisionFive2 平台上，Uboot 通常分为两部分：第一部分是 SPL（Secondary Program Loader），用于加载后续的引导程序，第二部分则是 Uboot + OpenSBI（[VisionFive2启动流程](https://doc.rvspace.org/VisionFive2/Developing_and_Porting_Guide/JH7110_Boot_UG/JH7110_SDK/boot_flow.html)）。

然而，如果我们想要开发一个操作系统内核，并将其运行于 VisionFive2 平台，使用 Uboot 加载自定义内核可能会显得较为繁琐且不够灵活。为了解决这个问题，我们开发了一个简单且极易修改的引导程序，取代了 Uboot 引导程序第二部分。

使用该 `vf2_bootloader` 固件的启动流程为：BootRom -> SPL -> vf2_bootloader -> 内核（SD 卡）。使用者只需根据自身的内核加载逻辑进行修改，然后将修改后的固件写入 SD 卡，即可实现自定义内核的加载。

## 逻辑剖析

`vf2_bootloader`仅通过读取SD卡来加载内核，内部实现非常简单，大致有如下几个组件组成：

- uart驱动
- log日志系统
- sdio驱动
- mem只做分配不做回收的内存分配器
- fat只读FAT32文件系统

下面逐步的分析`vf2_bootloader`的逻辑。

### 如何替换原来的Uboot

VisionFive 2支持从三个设备引导，分别是QSPI flash、NVME设备、SD卡。无论哪个设备，都由上述的两个部分部分组成，这两个部分存储在两个分区中，要做替换就必须刷写设备，那么我们就需要考虑，哪种设备便于刷入程序，并且最好不要随意刷写焊接在板子上的硬件，因为硬件损坏不易维修。QSPI Flash和eMMC（不自带）被焊接在板子上，出现硬件问题不易解决，而NVME硬盘成本稍高，不宜反复插拔，而SD卡容易插拔、出问题更换的成本低。

因此，最好是将编写好的程序刷写到SD卡中，并从SD卡启动，是最为方便妥当的做法。

### 让spl加载vf2_bootloader

在VisionFive 2平台上spl将Uboot+OpenSBI加载至LPDDR内存始址，即0x40000000，之后Uboot会将本身重定位至高地址然后再将内核加载到0x40000000。而使用`vf2_bootloader`时，我们修改了spl加载`vf2_bootloader`的地址，直接将其加载到0xC0000000，这样做就省去了重定向自身的过程，可以直接将自定义的内核加载至0x40000000。

tools 目录下，`mkimage`工具用于给生成的裸二进制文件签名，在Uboot签名工具fit脚本`fit_img.its`中，修改了签名部分的加载地址，由原来的0x40000000变为0xC0000000，这个fit脚本在官方github仓库可以找到（[Tools/uboot_its/visionfive2-uboot-fit-image.its at master · starfive-tech/Tools](https://github.com/starfive-tech/Tools/blob/master/uboot_its/visionfive2-uboot-fit-image.its)）。

相应的`vf2_bootloader`链接脚本也将初始地址设为0xC0000000，在 链接脚本 link.ld 中可以看到。

### vf2_bootloader组件

uart：Uart设备作为用户交互的一个媒介，可以充当一个终端的角色，它的实现位于[QIUZHILEI/uart_8250: Uart 8250驱动程序](https://github.com/QIUZHILEI/uart_8250)。

log：日志系统用于打印一些日志输出，这个组件使用的是官方提供的log crate，位于[rust-lang/log: Logging implementation for Rust](https://github.com/rust-lang/log)。

sd设备：sdio驱动SD卡，用于SD的IO操作，作为引导程序和内核的存储媒介，它的实现位于[QIUZHILEI/dw_sd: DesignWare Cores Mobile Storage Host 驱动](https://github.com/QIUZHILEI/dw_sd)。

mem：内存分配器的设计非常简单——只做分配，不做回收。

程序从SD的EFI分区加载内核，fat文件中实现了一个简单的只读FAT32文件系统，用于存储和加载内核。

### 代码逻辑

程序从`entry` (src/entry.S) 开始执行，初始化每个Hard栈，并将代码的末地址作为参数传递给`rust_entry` (src/main.rs)，在`rust_entry`函数中，仅让hart 1，进行环境初始化和内核加载，其余hart均等待加载完成，最后一并跳转到内核执行。

hart 1首先调用init函数初始化设备和内存分配器，然后执行`load_kernel` (src/lib.rs) 函数，寻找SD卡的EFI分区并初始化FAT32文件系统，接着将内核加载至内存，解开BLOCK并与其他hart一同跳转到内核开始执行。

***如何使用vf_bootloader可以参考 [VisionFive 2上快速体验组件化的力量](https://github.com/lego-os/.github/blob/main/vf2_bootloader_quick_start.md)***