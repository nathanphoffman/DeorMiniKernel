#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

# This box only has GRUB's EFI modules installed (no grub-pc-bin), so
# grub-mkrescue produces a UEFI-only ISO -- boot it under OVMF with a
# 64-bit machine. GRUB still loads our 32-bit multiboot2 kernel fine; it
# switches back out of long mode before jumping to its entry point.
mkdir -p build
cp /usr/share/OVMF/OVMF_VARS_4M.fd build/OVMF_VARS.fd

qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_CODE_4M.fd \
    -drive if=pflash,format=raw,file=build/OVMF_VARS.fd \
    -cdrom minikernel.iso
