; Minimal Multiboot header + entry stub.
; QEMU's -kernel flag loads Multiboot-compliant ELF binaries directly
; (no GRUB/bootloader needed) and hands control to us in 32-bit protected mode.

bits 32

MBALIGN     equ  1 << 0
MEMINFO     equ  1 << 1
FLAGS       equ  MBALIGN | MEMINFO
MAGIC       equ  0x1BADB002
CHECKSUM    equ -(MAGIC + FLAGS)

section .multiboot
align 4
    dd MAGIC
    dd FLAGS
    dd CHECKSUM

; The Multiboot spec doesn't guarantee a specific code/data selector, so we
; load our own GDT with known selector values -- the IDT (set up later, in
; Rust) needs a selector value it can rely on for interrupt gates.
section .data
align 8
gdt_start:
    dq 0x0000000000000000
gdt_code:
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 10011010b
    db 11001111b
    db 0x00
gdt_data:
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 10010010b
    db 11001111b
    db 0x00
gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start - 1
    dd gdt_start

CODE_SEG equ gdt_code - gdt_start
DATA_SEG equ gdt_data - gdt_start

section .bss
align 16
stack_bottom:
    resb 16384
stack_top:

section .text
global _start
extern kernel_main
_start:
    mov esp, stack_top
    lgdt [gdt_descriptor]
    jmp CODE_SEG:.reload_segments
.reload_segments:
    mov ax, DATA_SEG
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    call kernel_main
.hang:
    cli
    hlt
    jmp .hang

; Trampoline for the keyboard IRQ: saves general-purpose registers, calls the
; plain Rust handler, restores them, and returns via iretd. Avoids relying on
; the unstable `extern "x86-interrupt"` ABI.
global keyboard_isr_stub
extern keyboard_isr_rust
keyboard_isr_stub:
    pusha
    call keyboard_isr_rust
    popa
    iretd
