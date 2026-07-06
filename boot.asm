; Multiboot2 header + entry stub.
; GRUB (via grub-mkrescue) loads this ELF and hands control to us in 32-bit
; protected mode, with a pointer to the boot information structure in ebx --
; that structure carries the linear framebuffer address/pitch/depth GRUB set
; up from either VBE (BIOS) or GOP (UEFI), so the same kernel image renders
; the same way regardless of which firmware actually booted GRUB.

bits 32

MB2_MAGIC        equ 0xe85250d6
MB2_ARCH_I386    equ 0

section .multiboot
align 8
mb2_header_start:
    dd MB2_MAGIC
    dd MB2_ARCH_I386
    dd mb2_header_end - mb2_header_start
    dd -(MB2_MAGIC + MB2_ARCH_I386 + (mb2_header_end - mb2_header_start))

    ; Framebuffer request tag: ask for a 1024x768x32 linear framebuffer.
    ; GRUB may hand back a different mode if that exact one isn't available,
    ; so the kernel reads back whatever it actually got rather than assuming.
    align 8
    dw 5                                    ; MULTIBOOT_HEADER_TAG_FRAMEBUFFER
    dw 0                                    ; flags
    dd 20                                   ; size (8-byte tag header + 3 fields)
    dd 1024                                 ; width
    dd 768                                  ; height
    dd 32                                   ; depth

    ; End tag
    align 8
    dw 0
    dw 0
    dd 8
mb2_header_end:

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
    ; ebx holds GRUB's pointer to the multiboot2 boot info structure; none of
    ; lgdt/jmp/segment reloads below touch it, so it survives to the call.
    lgdt [gdt_descriptor]
    jmp CODE_SEG:.reload_segments
.reload_segments:
    mov ax, DATA_SEG
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    push ebx
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
