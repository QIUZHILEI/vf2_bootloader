#!/bin/bash
# 生成的裸机二进制固件名称
fw="fw.bin"
# fit后的固件名称
fw_img="fw.img"
# 清理工作区
rm $fw_img
cargo clean
# 编译生成可执行elf文件
cargo +nightly build --release --target riscv64gc-unknown-none-elf
# 将可执行文件转为二进制可执行文件
riscv64-unknown-elf-objcopy ./target/riscv64gc-unknown-none-elf/release/vf2_bootloader -O binary $fw
# 替换./tools/fit_img.its文件中的字符firmware_abs_path为固件文件的绝对路径
replace_str="firmware_abs_path"
fw_abs_path="$(pwd)/$fw"
sed -i "s|$replace_str|$fw_abs_path|g" ./tools/fit_img.its
# 生成最终的固件映像
./tools/mkimage -f ./tools/fit_img.its -A riscv -O u-boot -T firmware $fw_img
rm $fw
sed -i "s|$fw_abs_path|$replace_str|g" ./tools/fit_img.its