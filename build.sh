#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

mkdir -p build
DEOR_LIB=lib deor main.deor build/main.rs

# Strip leading inner attributes (e.g. `#![allow(warnings)]`) since this file
# gets spliced into src/lib.rs via include! inside a `mod generated { ... }`
# block, where an inner attribute is only legal as the block's first token.
sed '/^#!\[/d' build/main.rs > build/main_body.rs

nasm -f elf32 boot.asm -o boot.o

cargo +nightly build \
    -Z build-std=core,alloc,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem \
    -Z json-target-spec \
    --target i686-mini_kernel.json \
    --release

ld -m elf_i386 -T linker.ld -o kernel.elf \
    boot.o \
    target/i686-mini_kernel/release/libdeor_mini_kernel.a

echo "Built kernel.elf"

# Package into a GRUB ISO so the kernel boots the same way (multiboot2 +
# GRUB) whether GRUB itself started under legacy BIOS or UEFI -- QEMU's
# `-kernel` flag only understands Multiboot1, so it can't load this directly.
mkdir -p build/isodir/boot/grub
cp kernel.elf build/isodir/boot/kernel.elf
cat > build/isodir/boot/grub/grub.cfg <<'GRUBCFG'
set timeout=0
insmod all_video
insmod efi_gop
insmod video_bochs
menuentry "MiniKernel" {
    multiboot2 /boot/kernel.elf
    boot
}
GRUBCFG

grub-mkrescue -o minikernel.iso build/isodir
echo "Built minikernel.iso"
