use alloc::string::{String, ToString};

use crate::interrupts::{inb, outb, set_idt_entry};

const LINE_BUF_SIZE: usize = 256;
static mut LINE_BUF: [u8; LINE_BUF_SIZE] = [0; LINE_BUF_SIZE];
static mut LINE_LEN: usize = 0;
static mut LINE_READY: bool = false;

extern "C" {
    fn keyboard_isr_stub();
}

// LINE_BUF/LINE_LEN/LINE_READY are written from keyboard_isr_rust, which the
// compiler can't see is ever called -- it's only reached via a hardware
// interrupt, invisible to normal call-graph analysis. Without volatile
// access, LLVM is free to cache or reorder reads of these statics inside
// read_line() as if nothing else could touch them, which loses keystrokes
// under real timing. Every access from either side must go through here.
unsafe fn line_ready() -> bool {
    core::ptr::read_volatile(core::ptr::addr_of!(LINE_READY))
}

unsafe fn set_line_ready(value: bool) {
    core::ptr::write_volatile(core::ptr::addr_of_mut!(LINE_READY), value)
}

unsafe fn line_len() -> usize {
    core::ptr::read_volatile(core::ptr::addr_of!(LINE_LEN))
}

unsafe fn set_line_len(value: usize) {
    core::ptr::write_volatile(core::ptr::addr_of_mut!(LINE_LEN), value)
}

unsafe fn set_line_buf(index: usize, byte: u8) {
    core::ptr::write_volatile(core::ptr::addr_of_mut!(LINE_BUF[index]), byte)
}

/// Registers the keyboard ISR at IRQ1's vector (0x21 after the PIC remap).
pub(crate) fn install() {
    set_idt_entry(0x21, keyboard_isr_stub as *const () as u32);
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
                    set_line_ready(true);
                    crate::vga::putc(b'\n');
                }
                0x08 => {
                    let len = line_len();
                    if len > 0 {
                        set_line_len(len - 1);
                        crate::vga::putc(0x08);
                    }
                }
                _ => {
                    let len = line_len();
                    if len < LINE_BUF_SIZE {
                        set_line_buf(len, c);
                        set_line_len(len + 1);
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
        set_line_ready(false);
        set_line_len(0);
        core::arch::asm!("sti");
    }

    loop {
        if unsafe { line_ready() } {
            break;
        }
        unsafe { core::arch::asm!("hlt") };
    }

    let len = unsafe { line_len() };
    let s = unsafe { core::str::from_utf8(&LINE_BUF[..len]).unwrap_or("") };
    s.to_string()
}
