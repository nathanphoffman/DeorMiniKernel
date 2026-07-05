use alloc::string::{String, ToString};

const LINE_BUF_SIZE: usize = 256;
static mut LINE_BUF: [u8; LINE_BUF_SIZE] = [0; LINE_BUF_SIZE];
static mut LINE_LEN: usize = 0;
static mut LINE_READY: bool = false;

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

extern "C" {
    fn keyboard_isr_stub();
}

unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    value
}

fn set_idt_entry(index: usize, handler: u32) {
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
/// before any code calls `read_line`.
pub fn init() {
    set_idt_entry(0x21, keyboard_isr_stub as *const () as u32);

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

/// US QWERTY scancode set 1, make codes only -- just enough for a demo line read.
fn scancode_to_ascii(code: u8) -> Option<u8> {
    match code {
        0x1E => Some(b'a'), 0x30 => Some(b'b'), 0x2E => Some(b'c'), 0x20 => Some(b'd'),
        0x12 => Some(b'e'), 0x21 => Some(b'f'), 0x22 => Some(b'g'), 0x23 => Some(b'h'),
        0x17 => Some(b'i'), 0x24 => Some(b'j'), 0x25 => Some(b'k'), 0x26 => Some(b'l'),
        0x32 => Some(b'm'), 0x31 => Some(b'n'), 0x18 => Some(b'o'), 0x19 => Some(b'p'),
        0x10 => Some(b'q'), 0x13 => Some(b'r'), 0x1F => Some(b's'), 0x14 => Some(b't'),
        0x16 => Some(b'u'), 0x2F => Some(b'v'), 0x11 => Some(b'w'), 0x2D => Some(b'x'),
        0x15 => Some(b'y'), 0x2C => Some(b'z'),
        0x02 => Some(b'1'), 0x03 => Some(b'2'), 0x04 => Some(b'3'), 0x05 => Some(b'4'),
        0x06 => Some(b'5'), 0x07 => Some(b'6'), 0x08 => Some(b'7'), 0x09 => Some(b'8'),
        0x0A => Some(b'9'), 0x0B => Some(b'0'),
        0x39 => Some(b' '),
        0x1C => Some(b'\n'),
        0x0E => Some(0x08), // backspace
        _ => None,
    }
}

#[no_mangle]
pub extern "C" fn keyboard_isr_rust() {
    let scancode = unsafe { inb(0x60) };
    unsafe { outb(0x20, 0x20) }; // EOI to master PIC

    if scancode & 0x80 != 0 {
        return; // key release
    }

    if let Some(c) = scancode_to_ascii(scancode) {
        unsafe {
            match c {
                b'\n' => {
                    LINE_READY = true;
                    crate::vga::putc(b'\n');
                }
                0x08 => {
                    if LINE_LEN > 0 {
                        LINE_LEN -= 1;
                        crate::vga::putc(0x08);
                    }
                }
                _ => {
                    if LINE_LEN < LINE_BUF_SIZE {
                        LINE_BUF[LINE_LEN] = c;
                        LINE_LEN += 1;
                        crate::vga::putc(c);
                    }
                }
            }
        }
    }
}

/// Blocks until Enter is pressed, returning the typed line (without the newline).
pub fn read_line() -> String {
    unsafe {
        core::arch::asm!("cli");
        LINE_READY = false;
        LINE_LEN = 0;
        core::arch::asm!("sti");
    }

    loop {
        if unsafe { LINE_READY } {
            break;
        }
        unsafe { core::arch::asm!("hlt") };
    }

    let s = unsafe { core::str::from_utf8(&LINE_BUF[..LINE_LEN]).unwrap_or("") };
    s.to_string()
}
