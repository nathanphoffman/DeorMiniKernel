#[derive(Clone, Copy)]
#[repr(C, packed)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    zero: u8,
    type_attr: u8,
    offset_high: u16,
}

#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u32,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry {
    offset_low: 0,
    selector: 0,
    zero: 0,
    type_attr: 0,
    offset_high: 0,
}; 256];

pub(crate) unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

pub(crate) unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    value
}

pub(crate) fn set_idt_entry(index: usize, handler: u32) {
    unsafe {
        IDT[index] = IdtEntry {
            offset_low: (handler & 0xFFFF) as u16,
            selector: 0x08,
            zero: 0,
            type_attr: 0x8E, // present, ring 0, 32-bit interrupt gate
            offset_high: ((handler >> 16) & 0xFFFF) as u16,
        };
    }
}

unsafe fn remap_pic() {
    outb(0x20, 0x11); // ICW1: init, expect ICW4
    outb(0xA0, 0x11);
    outb(0x21, 0x20); // ICW2: master IRQs mapped to 0x20-0x27
    outb(0xA1, 0x28); // ICW2: slave IRQs mapped to 0x28-0x2F
    outb(0x21, 0x04); // ICW3: slave attached at IRQ2
    outb(0xA1, 0x02);
    outb(0x21, 0x01); // ICW4: 8086 mode
    outb(0xA1, 0x01);
    outb(0x21, 0xFD); // OCW1: mask all master IRQs except IRQ1 (keyboard)
    outb(0xA1, 0xFF); // mask all slave IRQs
}

/// Sets up the IDT and PIC and enables interrupts. Must run once at boot,
/// before any code calls `keyboard::read_line`.
pub fn init() {
    crate::keyboard::install();

    let idt_ptr = IdtPointer {
        limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
        base: core::ptr::addr_of!(IDT) as u32,
    };

    unsafe {
        core::arch::asm!("lidt [{0}]", in(reg) &idt_ptr, options(readonly, nostack, preserves_flags));
        remap_pic();
        core::arch::asm!("sti");
    }
}
