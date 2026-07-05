#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

nasm -f elf32 boot.asm -o boot.o

cargo +nightly build \
    -Z build-std=core,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem \
    -Z json-target-spec \
    --target i686-mini_kernel.json \
    --release

ld -m elf_i386 -T linker.ld -o kernel.elf \
    boot.o \
    target/i686-mini_kernel/release/libmini_kernel.a

echo "Built kernel.elf"
